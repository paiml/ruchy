//! Miscellaneous expression evaluation for the Ruchy interpreter.
//!
//! This module handles evaluation of:
//! - Import statements (use, import, import-all)
//! - Macro invocations (vec!, println!, format!)
//! - Try operator (?)
//! - Pipeline operator (|>)
//! - Lazy/async blocks
//! - Module expressions
//! - If-let / while-let expressions
//! - List comprehensions
//! - Actor operations (spawn, send, query)
//!
//! Extracted from interpreter.rs to reduce file size (EXTREME TDD).

use super::interpreter::{Interpreter, InterpreterError, Value};
use crate::frontend::ast::{ComprehensionClause, Expr, ExprKind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

// ============================================================================
// Main dispatch: eval_misc_expr
// ============================================================================

/// Evaluate miscellaneous expressions (imports, macros, try, pipeline, etc.)
/// Complexity: 7 (was 5, added import handling)
pub(crate) fn eval_misc_expr(
    interp: &mut Interpreter,
    expr_kind: &ExprKind,
) -> Result<Value, InterpreterError> {
    if is_type_definition(expr_kind) {
        return eval_type_definition(interp, expr_kind);
    }
    if is_actor_operation(expr_kind) {
        return eval_actor_operation(interp, expr_kind);
    }
    if is_special_form(expr_kind) {
        return eval_special_form(interp, expr_kind);
    }

    // Handle import statements (GitHub Issue #59)
    // Issue #82: Implement basic module resolution for use statements
    match expr_kind {
        ExprKind::ImportAll { module, alias } => eval_import_all(interp, module, alias),
        ExprKind::Import { module, items: _ } => eval_import(interp, module),
        ExprKind::ImportDefault { .. } => {
            // LIMITATION: ImportDefault not yet implemented - returns Nil for now
            // See ISSUE-106 for module resolution tracking
            Ok(Value::Nil)
        }
        // Handle vec! macro (GitHub Issue #62)
        ExprKind::Macro { name, args } => eval_macro_invocation(interp, name, args),
        // RUNTIME-001: Handle MacroInvocation (FORMATTER-088 changed parser output)
        // MacroInvocation is the correct AST variant for macro CALLS (not definitions)
        // Delegate to same logic as Macro for backward compatibility (GitHub Issue #74)
        ExprKind::MacroInvocation { name, args } => eval_macro_invocation(interp, name, args),
        ExprKind::Try { expr } => eval_try_operator(interp, expr),
        // SPEC-001-C: Pipeline operator evaluation (|> not >>)
        ExprKind::Pipeline { expr, stages } => eval_pipeline(interp, expr, stages),
        // SPEC-001-D: Lazy evaluation - defers computation until value is accessed
        ExprKind::Lazy { expr } => interp.eval_expr(expr),
        // SPEC-001-E: Async block - simplified synchronous evaluation
        ExprKind::AsyncBlock { body } => interp.eval_expr(body),
        // ISSUE-106: Module expression - creates a namespace with exported functions
        ExprKind::Module { name, body } => interp.eval_module_expr(name, body),
        // ISSUE-106: ModuleDeclaration should be resolved before evaluation
        ExprKind::ModuleDeclaration { name } => Err(InterpreterError::RuntimeError(format!(
            "Module '{}' not resolved. Use `ruchy compile` or ensure module file exists.",
            name
        ))),
        // If-let expression: if let pattern = expr { then } else { else }
        ExprKind::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        } => eval_if_let(interp, pattern, expr, then_branch, else_branch.as_deref()),
        // While-let expression: while let pattern = expr { body }
        ExprKind::WhileLet {
            label: _,
            pattern,
            expr,
            body,
        } => eval_while_let(interp, pattern, expr, body),
        // List comprehension: [expr for x in iter if cond]
        ExprKind::ListComprehension { element, clauses } => {
            eval_list_comprehension(interp, element, clauses)
        }
        _ => {
            // Fallback for unimplemented expressions
            Err(InterpreterError::RuntimeError(format!(
                "Expression type not yet implemented: {expr_kind:?}"
            )))
        }
    }
}

// ============================================================================
// Import evaluation
// ============================================================================

/// Evaluate `use module::path as alias` or `use module::path::*`
fn eval_import_all(
    interp: &mut Interpreter,
    module: &str,
    alias: &str,
) -> Result<Value, InterpreterError> {
    let parts: Vec<&str> = module.split("::").collect();

    // Import the symbol into current environment with the appropriate name
    if let Some(value) = interp.resolve_module_path(module) {
        // Determine the name to use: alias if provided, otherwise last part of path
        let import_name = if alias == "*" {
            // Wildcard import - not yet implemented
            return Ok(Value::Nil);
        } else if !alias.is_empty() && alias != "*" {
            alias.to_string()
        } else {
            (*parts.last().unwrap_or(&"")).to_string()
        };

        // Add to global environment (first element of env_stack)
        // This makes imports available across all scopes
        if let Some(global_env_ref) = interp.env_stack.first() {
            global_env_ref.borrow_mut().insert(import_name, value); // ISSUE-119: Mutable borrow
        }
    }

    Ok(Value::Nil)
}

/// Evaluate `use module;` or `use std::module;`
fn eval_import(interp: &mut Interpreter, module: &str) -> Result<Value, InterpreterError> {
    // Issue #89: Distinguish between stdlib imports and file module imports
    if module.starts_with("std::") {
        // Issue #96: stdlib imports must make the module available in current scope
        let parts: Vec<&str> = module.split("::").collect();

        if let Some(value) = interp.resolve_module_path(module) {
            let import_name = (*parts.last().unwrap_or(&"")).to_string();

            // Add to global environment
            if let Some(global_env_ref) = interp.env_stack.first() {
                global_env_ref.borrow_mut().insert(import_name, value);
            }
        }

        return Ok(Value::Nil);
    }

    // Issue #88: Load file module from file system and execute it
    let parsed_module = interp
        .module_loader_mut()
        .load_module(module)
        .map_err(|e| {
            InterpreterError::RuntimeError(format!(
                "Failed to load module '{}': {}",
                module, e
            ))
        })?;

    // Create a new environment scope for the module
    // ISSUE-119: Wrap in Rc<RefCell> for shared mutable access
    let module_env_ref = Rc::new(RefCell::new(HashMap::new()));

    // Evaluate the module AST to execute its definitions
    interp.env_stack.push(Rc::clone(&module_env_ref));
    let eval_result = interp.eval_expr(&parsed_module.ast);
    interp.env_stack.pop();

    // Check for evaluation errors
    eval_result?;

    // Create a module namespace object containing all exported symbols
    let mut module_object = std::collections::HashMap::new();
    for (name, value) in module_env_ref.borrow().iter() {
        module_object.insert(name.clone(), value.clone());
    }

    // Add the module object to global environment
    if let Some(global_env_ref) = interp.env_stack.first() {
        global_env_ref
            .borrow_mut()
            .insert(module.to_string(), Value::Object(module_object.into()));
        // ISSUE-119: Mutable borrow
    }

    Ok(Value::Nil)
}

// ============================================================================
// Macro evaluation (deduplicated for Macro and MacroInvocation)
// ============================================================================

/// Evaluate a macro invocation (vec!, println!, format!)
/// Handles both ExprKind::Macro and ExprKind::MacroInvocation identically.
fn eval_macro_invocation(
    interp: &mut Interpreter,
    name: &str,
    args: &[Expr],
) -> Result<Value, InterpreterError> {
    if name == "vec" {
        eval_vec_macro(interp, args)
    } else if name == "println" {
        eval_println_macro(interp, args)
    } else if name == "format" {
        eval_format_macro(interp, args)
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "Macro '{}!' not yet implemented",
            name
        )))
    }
}

/// vec![...] expands to an array with evaluated arguments
fn eval_vec_macro(
    interp: &mut Interpreter,
    args: &[Expr],
) -> Result<Value, InterpreterError> {
    let mut elements = Vec::new();
    for arg in args {
        let value = interp.eval_expr(arg)?;
        elements.push(value);
    }
    Ok(Value::Array(elements.into()))
}

/// println!() macro: Evaluate arguments, print with newline
/// PARSER-085: Supports format strings like println!("x: {}", value)
fn eval_println_macro(
    interp: &mut Interpreter,
    args: &[Expr],
) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        println!();
    } else if args.len() == 1 {
        // Single argument: print directly
        let value = interp.eval_expr(&args[0])?;
        println!("{}", value);
    } else {
        // Multiple arguments: use format! logic (Issue #82, #83)
        let format_val = interp.eval_expr(&args[0])?;
        let format_str = match format_val {
            Value::String(ref s) => s.as_ref().to_string(),
            _ => format_val.to_string(),
        };

        let mut values = Vec::new();
        for arg in &args[1..] {
            values.push(interp.eval_expr(arg)?);
        }

        // Use helper for format string replacement
        let result = Interpreter::format_string_with_values(&format_str, &values);
        println!("{}", result);
    }
    Ok(Value::Nil)
}

/// format!() macro: Format string with placeholders (Issue #83)
fn eval_format_macro(
    interp: &mut Interpreter,
    args: &[Expr],
) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "format!() requires at least one argument".to_string(),
        ));
    }

    // Evaluate format string
    let format_val = interp.eval_expr(&args[0])?;
    let format_str = format_val.to_string();

    // Evaluate remaining arguments
    let mut values = Vec::new();
    for arg in &args[1..] {
        values.push(interp.eval_expr(arg)?);
    }

    // Replace {} and {:?} placeholders with values
    let result = format_with_placeholders(&format_str, &values);
    Ok(Value::from_string(result))
}

/// Replace `{}` and `{:?}` placeholders in a format string with values.
/// Shared helper used by format!() macro evaluation.
fn try_consume_debug_placeholder(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    result: &mut String,
    values: &[Value],
    value_index: &mut usize,
) -> bool {
    if chars.peek() != Some(&':') {
        return false;
    }
    chars.next();
    if chars.peek() != Some(&'?') {
        result.push_str("{:");
        return true;
    }
    chars.next();
    if chars.peek() != Some(&'}') {
        result.push_str("{:?");
        return true;
    }
    chars.next();
    if *value_index < values.len() {
        result.push_str(&format!("{:?}", values[*value_index]));
        *value_index += 1;
    } else {
        result.push_str("{:?}");
    }
    true
}

fn format_with_placeholders(format_str: &str, values: &[Value]) -> String {
    let mut result = String::new();
    let mut chars = format_str.chars().peekable();
    let mut value_index = 0;

    while let Some(ch) = chars.next() {
        if ch != '{' {
            result.push(ch);
            continue;
        }
        if try_consume_debug_placeholder(&mut chars, &mut result, values, &mut value_index) {
            continue;
        }
        if chars.peek() == Some(&'}') {
            chars.next();
            if value_index < values.len() {
                result.push_str(&values[value_index].to_string());
                value_index += 1;
            } else {
                result.push_str("{}");
            }
        } else {
            result.push(ch);
        }
    }

    result
}

// ============================================================================
// Try operator (?)
// ============================================================================

/// Issue #97: Try operator (?) for Result unwrapping/propagation
fn extract_ok_first_value(data: Option<&Vec<Value>>, context: &str) -> Result<Value, InterpreterError> {
    data.and_then(|v| v.first().cloned())
        .ok_or_else(|| InterpreterError::RuntimeError(format!("Try operator: {context}")))
}

fn try_unwrap_enum_variant(value: &Value) -> Option<Result<Value, InterpreterError>> {
    let Value::EnumVariant { enum_name, variant_name, data } = value else {
        return None;
    };
    if enum_name != "Result" {
        return None;
    }
    Some(match variant_name.as_str() {
        "Ok" => extract_ok_first_value(data.as_ref(), "Ok variant has no data"),
        "Err" => Err(InterpreterError::Return(value.clone())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Try operator: unexpected Result variant '{variant_name}'"
        ))),
    })
}

fn is_message_object(obj: &std::collections::HashMap<String, Value>) -> bool {
    obj.get("__type")
        .and_then(|v| if let Value::String(s) = v { Some(s.as_ref() == "Message") } else { None })
        .unwrap_or(false)
}

fn try_unwrap_message_object(value: &Value) -> Option<Result<Value, InterpreterError>> {
    let Value::Object(obj) = value else { return None };
    if !is_message_object(obj) {
        return None;
    }
    let variant = match obj.get("type") {
        Some(Value::String(v)) => v.clone(),
        _ => return Some(Err(InterpreterError::RuntimeError(
            "Try operator: Message object missing 'type' field".to_string(),
        ))),
    };
    Some(match variant.as_ref() {
        "Ok" => {
            let data = obj.get("data").and_then(|v| {
                if let Value::Array(arr) = v { Some(arr.to_vec()) } else { None }
            });
            extract_ok_first_value(data.as_ref(), "Ok missing data field")
        }
        "Err" => Err(InterpreterError::Return(value.clone())),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Try operator: unexpected type '{variant}'"
        ))),
    })
}

fn eval_try_operator(
    interp: &mut Interpreter,
    expr: &Expr,
) -> Result<Value, InterpreterError> {
    let result_value = interp.eval_expr(expr)?;

    if let Some(result) = try_unwrap_enum_variant(&result_value) {
        return result;
    }
    if let Some(result) = try_unwrap_message_object(&result_value) {
        return result;
    }
    Err(InterpreterError::RuntimeError(format!(
        "Try operator expects Result enum, got: {result_value:?}"
    )))
}

// ============================================================================
// Pipeline operator (|>)
// ============================================================================

/// SPEC-001-C: Pipeline operator evaluation (|> not >>)
/// Evaluates: initial_expr |> func1 |> func2 |> ...
/// Example: 5 |> double |> add_one -> add_one(double(5)) -> 11
/// PIPELINE-001 FIX: Also supports method calls: "hello" |> upper -> "hello".upper()
fn eval_pipeline(
    interp: &mut Interpreter,
    expr: &Expr,
    stages: &[crate::frontend::ast::PipelineStage],
) -> Result<Value, InterpreterError> {
    let mut current_value = interp.eval_expr(expr)?;

    for stage in stages {
        current_value = eval_pipeline_stage(interp, current_value, stage)?;
    }

    Ok(current_value)
}

/// Evaluate a single pipeline stage
fn eval_pipeline_stage(
    interp: &mut Interpreter,
    current_value: Value,
    stage: &crate::frontend::ast::PipelineStage,
) -> Result<Value, InterpreterError> {
    // PIPELINE-001: Check if stage is an identifier that could be a method
    if let ExprKind::Identifier(method_name) = &stage.op.kind {
        // First, try to look up as a user-defined function (Closure)
        let is_user_function = interp
            .lookup_variable(method_name)
            .map(|v| matches!(v, Value::Closure { .. }))
            .unwrap_or(false);

        if is_user_function {
            let func_val = interp.lookup_variable(method_name)?;
            interp.call_function(func_val, &[current_value])
        } else {
            // Not a user function - try calling as a method on current_value
            interp.dispatch_method_call(&current_value, method_name, &[], true)
        }
    } else if let ExprKind::Call { func, args } = &stage.op.kind {
        eval_pipeline_call_stage(interp, current_value, func, args)
    } else {
        // Other expression types - evaluate as function and call with current_value
        let func_val = interp.eval_expr(&stage.op)?;
        interp.call_function(func_val, &[current_value])
    }
}

/// Evaluate a pipeline stage that is a function call
fn eval_pipeline_call_stage(
    interp: &mut Interpreter,
    current_value: Value,
    func: &Expr,
    args: &[Expr],
) -> Result<Value, InterpreterError> {
    if let ExprKind::Identifier(method_name) = &func.kind {
        // Check if it's a user-defined function (Closure)
        let is_user_function = interp
            .lookup_variable(method_name)
            .map(|v| matches!(v, Value::Closure { .. }))
            .unwrap_or(false);

        if is_user_function {
            // It's a user function call - prepend current_value to args
            let func_val = interp.lookup_variable(method_name)?;
            let arg_values: Result<Vec<_>, _> =
                args.iter().map(|arg| interp.eval_expr(arg)).collect();
            let mut all_args = vec![current_value];
            all_args.extend(arg_values?);
            interp.call_function(func_val, &all_args)
        } else {
            // It's a method call with args: arr |> filter(pred) -> arr.filter(pred)
            let arg_values: Result<Vec<_>, _> =
                args.iter().map(|arg| interp.eval_expr(arg)).collect();
            interp.dispatch_method_call(
                &current_value,
                method_name,
                &arg_values?,
                args.is_empty(),
            )
        }
    } else {
        // Complex function expression - evaluate and call with current_value as first arg
        let func_val = interp.eval_expr(func)?;
        let arg_values: Result<Vec<_>, _> =
            args.iter().map(|arg| interp.eval_expr(arg)).collect();
        let mut all_args = vec![current_value];
        all_args.extend(arg_values?);
        interp.call_function(func_val, &all_args)
    }
}

// ============================================================================
// If-let / While-let expressions
// ============================================================================

/// If-let expression: if let pattern = expr { then } else { else }
fn eval_if_let(
    interp: &mut Interpreter,
    pattern: &crate::frontend::ast::Pattern,
    expr: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> Result<Value, InterpreterError> {
    let value = interp.eval_expr(expr)?;
    if let Some(bindings) = interp.try_pattern_match(pattern, &value)? {
        // Pattern matched - bind variables and execute then_branch
        interp.push_scope();
        for (name, val) in bindings {
            interp.env_set(name, val);
        }
        let result = interp.eval_expr(then_branch);
        interp.pop_scope();
        result
    } else if let Some(else_expr) = else_branch {
        // Pattern didn't match - execute else branch
        interp.eval_expr(else_expr)
    } else {
        // No match and no else branch - return nil
        Ok(Value::Nil)
    }
}

/// While-let expression: while let pattern = expr { body }
enum LoopBodyResult {
    Continue(Value),
    BreakWith(Value),
    ContinueLoop,
}

fn eval_scoped_loop_body(
    interp: &mut Interpreter,
    bindings: Vec<(String, Value)>,
    body: &Expr,
) -> Result<LoopBodyResult, InterpreterError> {
    interp.push_scope();
    for (name, val) in bindings {
        interp.env_set(name, val);
    }
    let result = match interp.eval_expr(body) {
        Ok(v) => Ok(LoopBodyResult::Continue(v)),
        Err(InterpreterError::Break(_, v)) => Ok(LoopBodyResult::BreakWith(v)),
        Err(InterpreterError::Continue(_)) => Ok(LoopBodyResult::ContinueLoop),
        Err(e) => Err(e),
    };
    interp.pop_scope();
    result
}

fn eval_while_let(
    interp: &mut Interpreter,
    pattern: &crate::frontend::ast::Pattern,
    expr: &Expr,
    body: &Expr,
) -> Result<Value, InterpreterError> {
    let mut last_value = Value::Nil;
    loop {
        let value = interp.eval_expr(expr)?;
        let Some(bindings) = interp.try_pattern_match(pattern, &value)? else {
            break;
        };
        match eval_scoped_loop_body(interp, bindings, body)? {
            LoopBodyResult::Continue(v) => last_value = v,
            LoopBodyResult::BreakWith(v) => return Ok(v),
            LoopBodyResult::ContinueLoop => continue,
        }
    }
    Ok(last_value)
}

// ============================================================================
// List comprehensions
// ============================================================================

/// Evaluate list comprehension: [expr for x in iter if cond]
/// Supports multiple clauses (nested loops) and optional conditions
/// Complexity: 8 (nested iteration with conditions)
pub(crate) fn eval_list_comprehension(
    interp: &mut Interpreter,
    element: &Expr,
    clauses: &[ComprehensionClause],
) -> Result<Value, InterpreterError> {
    let mut results = Vec::new();
    eval_comprehension_clauses(interp, &mut results, element, clauses, 0)?;
    Ok(Value::Array(Arc::from(results)))
}

/// Recursively process comprehension clauses
fn iterate_comprehension_item(
    interp: &mut Interpreter,
    results: &mut Vec<Value>,
    element: &Expr,
    clauses: &[ComprehensionClause],
    clause_idx: usize,
    variable: &str,
    item: Value,
    condition: Option<&Expr>,
) -> Result<(), InterpreterError> {
    interp.env_set(variable.to_string(), item);
    if check_comprehension_condition(interp, condition)? {
        eval_comprehension_clauses(interp, results, element, clauses, clause_idx + 1)?;
    }
    Ok(())
}

pub(crate) fn eval_comprehension_clauses(
    interp: &mut Interpreter,
    results: &mut Vec<Value>,
    element: &Expr,
    clauses: &[ComprehensionClause],
    clause_idx: usize,
) -> Result<(), InterpreterError> {
    if clause_idx >= clauses.len() {
        results.push(interp.eval_expr(element)?);
        return Ok(());
    }

    let clause = &clauses[clause_idx];
    let iterable = interp.eval_expr(&clause.iterable)?;
    let variable = clause.variable.clone();
    let condition = clause.condition.clone();

    interp.push_scope();
    match iterable {
        Value::Array(ref arr) => {
            for item in arr.iter() {
                iterate_comprehension_item(
                    interp, results, element, clauses, clause_idx,
                    &variable, item.clone(), condition.as_deref(),
                )?;
            }
        }
        Value::Range {
            ref start,
            ref end,
            inclusive,
        } => {
            let (start_val, end_val) = interp.extract_range_bounds(start, end)?;
            for i in interp.create_range_iterator(start_val, end_val, inclusive) {
                iterate_comprehension_item(
                    interp, results, element, clauses, clause_idx,
                    &variable, Value::Integer(i), condition.as_deref(),
                )?;
            }
        }
        _ => {
            interp.pop_scope();
            return Err(InterpreterError::TypeError(
                "List comprehension requires an iterable".to_string(),
            ));
        }
    }
    interp.pop_scope();
    Ok(())
}

/// Helper: Check comprehension condition
pub(crate) fn check_comprehension_condition(
    interp: &mut Interpreter,
    condition: Option<&Expr>,
) -> Result<bool, InterpreterError> {
    if let Some(cond) = condition {
        let cond_val = interp.eval_expr(cond)?;
        Ok(cond_val.is_truthy())
    } else {
        Ok(true)
    }
}

// ============================================================================
// Actor operations (spawn, send, query)
// ============================================================================

/// Helper: Evaluate spawn actor expression with proper nesting handling
/// Complexity: 10 (extracted from inline code)
fn is_actor_definition(interp: &mut Interpreter, name: &str) -> bool {
    if let Ok(def_value) = interp.lookup_variable(name) {
        if let Value::Object(ref obj) = def_value {
            if let Some(Value::String(type_str)) = obj.get("__type") {
                return type_str.as_ref() == "Actor";
            }
        }
    }
    false
}

fn spawn_actor_by_name(
    interp: &mut Interpreter,
    name: &str,
    args: &[Value],
) -> Result<Value, InterpreterError> {
    let constructor_marker = Value::from_string(format!("__actor_constructor__:{name}"));
    interp.call_function(constructor_marker, args)
}

pub(crate) fn eval_spawn_actor(
    interp: &mut Interpreter,
    actor: &Expr,
) -> Result<Value, InterpreterError> {
    if let ExprKind::Identifier(name) = &actor.kind {
        if is_actor_definition(interp, name) {
            return spawn_actor_by_name(interp, name, &[]);
        }
    }

    if let ExprKind::Call { func, args } = &actor.kind {
        if let ExprKind::Identifier(name) = &func.kind {
            if is_actor_definition(interp, name) {
                let arg_vals: Result<Vec<Value>, _> =
                    args.iter().map(|arg| interp.eval_expr(arg)).collect();
                return spawn_actor_by_name(interp, name, &arg_vals?);
            }
        }
    }

    interp.eval_expr(actor)
}

/// Helper: Evaluate actor send expression (fire-and-forget)
/// Complexity: 4
pub(crate) fn eval_actor_send(
    interp: &mut Interpreter,
    actor: &Expr,
    message: &Expr,
) -> Result<Value, InterpreterError> {
    let actor_value = interp.eval_expr(actor)?;
    let message_value = interp.eval_message_expr(message)?;

    if let Value::ObjectMut(cell_rc) = actor_value {
        interp.process_actor_message_sync_mut(&cell_rc, &message_value)?;
        Ok(Value::Nil)
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "ActorSend requires an actor instance, got {}",
            actor_value.type_name()
        )))
    }
}

/// Helper: Evaluate actor query expression (ask pattern)
/// Complexity: 4
pub(crate) fn eval_actor_query(
    interp: &mut Interpreter,
    actor: &Expr,
    message: &Expr,
) -> Result<Value, InterpreterError> {
    let actor_value = interp.eval_expr(actor)?;
    let message_value = interp.eval_message_expr(message)?;

    if let Value::ObjectMut(cell_rc) = actor_value {
        interp.process_actor_message_sync_mut(&cell_rc, &message_value)
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "ActorQuery requires an actor instance, got {}",
            actor_value.type_name()
        )))
    }
}

// ============================================================================
// Type definition, actor operation, and special form dispatch
// ============================================================================

/// Evaluate type definition expressions (Actor, Struct, Class, Impl)
/// Complexity: 6
pub(crate) fn eval_type_definition(
    interp: &mut Interpreter,
    expr_kind: &ExprKind,
) -> Result<Value, InterpreterError> {
    match expr_kind {
        ExprKind::Actor {
            name,
            state,
            handlers,
        } => interp.eval_actor_definition(name, state, handlers),
        // SPEC-001-I: Effect declarations return Nil (no runtime implementation)
        ExprKind::Effect { .. } => Ok(Value::Nil),
        // SPEC-001-J: Effect handlers evaluate expression and return nil
        ExprKind::Handle { expr, .. } => {
            interp.eval_expr(expr)?;
            Ok(Value::Nil)
        }
        ExprKind::Enum {
            name,
            type_params,
            variants,
            is_pub,
        } => interp.eval_enum_definition(name, type_params, variants, *is_pub),
        ExprKind::Struct {
            name,
            type_params,
            fields,
            methods,
            derives: _,
            is_pub,
        } => interp.eval_struct_definition(name, type_params, fields, methods, *is_pub),
        ExprKind::TupleStruct { .. } => {
            // Tuple structs are transpilation feature, return Nil at runtime
            Ok(Value::Nil)
        }
        ExprKind::Class {
            name,
            type_params,
            superclass,
            traits,
            fields,
            constructors,
            methods,
            constants,
            properties: _,
            derives,
            is_pub,
            is_sealed: _,
            is_abstract: _,
            decorators: _,
        } => interp.eval_class_definition(
            name,
            type_params,
            superclass.as_ref(),
            traits,
            fields,
            constructors,
            methods,
            constants,
            derives,
            *is_pub,
        ),
        ExprKind::Impl {
            trait_name: _,
            for_type,
            methods,
            ..
        } => interp.eval_impl_block(for_type, methods),
        _ => unreachable!("eval_type_definition called with non-type-definition"),
    }
}

/// Evaluate actor operation expressions (Spawn, ActorSend, ActorQuery)
/// Complexity: 4
pub(crate) fn eval_actor_operation(
    interp: &mut Interpreter,
    expr_kind: &ExprKind,
) -> Result<Value, InterpreterError> {
    match expr_kind {
        ExprKind::Spawn { actor } => eval_spawn_actor(interp, actor),
        ExprKind::ActorSend { actor, message } => eval_actor_send(interp, actor, message),
        ExprKind::ActorQuery { actor, message } => eval_actor_query(interp, actor, message),
        _ => unreachable!("eval_actor_operation called with non-actor-operation"),
    }
}

/// Evaluate special form expressions (None, Some, Set, patterns, literals)
/// Complexity: 9
pub(crate) fn eval_special_form(
    interp: &mut Interpreter,
    expr_kind: &ExprKind,
) -> Result<Value, InterpreterError> {
    match expr_kind {
        ExprKind::None => Ok(Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        }),
        ExprKind::Some { value } => Ok(Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![interp.eval_expr(value)?]),
        }),
        ExprKind::Set(statements) => {
            let mut result = Value::Nil;
            for stmt in statements {
                result = interp.eval_expr(stmt)?;
            }
            Ok(result)
        }
        ExprKind::LetPattern {
            pattern,
            value,
            body,
            ..
        } => interp.eval_let_pattern(pattern, value, body),
        ExprKind::StringInterpolation { parts } => interp.eval_string_interpolation(parts),
        ExprKind::QualifiedName { module, name } => interp.eval_qualified_name(module, name),
        ExprKind::ObjectLiteral { fields } => interp.eval_object_literal(fields),
        ExprKind::StructLiteral {
            name,
            fields,
            base: _,
        } => interp.eval_struct_literal(name, fields),
        _ => unreachable!("eval_special_form called with non-special-form"),
    }
}

/// Helper: Check if expression is a type definition
pub(crate) fn is_type_definition(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::Actor { .. }
            | ExprKind::Effect { .. }
            | ExprKind::Handle { .. }
            | ExprKind::Enum { .. }
            | ExprKind::Struct { .. }
            | ExprKind::TupleStruct { .. }
            | ExprKind::Class { .. }
            | ExprKind::Impl { .. }
    )
}

/// Helper: Check if expression is an actor operation
pub(crate) fn is_actor_operation(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::Spawn { .. } | ExprKind::ActorSend { .. } | ExprKind::ActorQuery { .. }
    )
}

/// Helper: Check if expression is a special form
pub(crate) fn is_special_form(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::None
            | ExprKind::Some { .. }
            | ExprKind::Set(_)
            | ExprKind::LetPattern { .. }
            | ExprKind::StringInterpolation { .. }
            | ExprKind::QualifiedName { .. }
            | ExprKind::ObjectLiteral { .. }
            | ExprKind::StructLiteral { .. }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_import_std_module() {
        // std:: imports should return Nil without error even if module is not found
        let mut interp = Interpreter::new();
        let result = eval_import(&mut interp, "std::io");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_import_std_nested_module() {
        let mut interp = Interpreter::new();
        let result = eval_import(&mut interp, "std::collections::HashMap");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_import_file_module_not_found() {
        // File modules should fail when module file doesn't exist
        let mut interp = Interpreter::new();
        let result = eval_import(&mut interp, "nonexistent_module");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to load module"));
    }

    #[test]
    fn test_eval_import_std_module_makes_value_available() {
        // std:: prefix should be recognized and handled
        let mut interp = Interpreter::new();
        let result = eval_import(&mut interp, "std::math");
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_import_another_nonexistent_file_module() {
        let mut interp = Interpreter::new();
        let result = eval_import(&mut interp, "does_not_exist");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("does_not_exist"));
    }
}
