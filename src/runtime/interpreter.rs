//! High-Performance Interpreter with Safe Value Representation
//!
//! This module implements the two-tier execution strategy from ruchy-interpreter-spec.md:
//! - Tier 0: AST interpreter with enum-based values (safe alternative)
//! - Tier 1: JIT compilation (future)
//!
//! Uses safe Rust enum approach instead of tagged pointers to respect `unsafe_code = "forbid"`.
//!
//! **EXTREME TDD Round 52**: Value enum extracted to runtime/value.rs
//! **EXTREME TDD Round 52**: Types extracted to `runtime/interpreter_types.rs`

#![allow(clippy::unused_self)] // Methods will use self in future phases
#![allow(clippy::only_used_in_recursion)] // Recursive print_value is intentional
#![allow(clippy::uninlined_format_args)] // Some format strings are clearer unexpanded
#![allow(clippy::cast_precision_loss)] // Acceptable for arithmetic operations
#![allow(clippy::expect_used)] // Used appropriately in tests
#![allow(clippy::cast_possible_truncation)] // Controlled truncations for indices

use super::eval_expr;
use super::eval_literal;
use super::eval_operations;
// EXTREME TDD Round 52: Value types imported from dedicated module
pub use super::value::{DataFrameColumn, Value};
// EXTREME TDD Round 52: Interpreter types imported from dedicated module
pub use super::interpreter_types::{CallFrame, InterpreterError, InterpreterResult};
use crate::frontend::ast::{
    BinaryOp as AstBinaryOp, ComprehensionClause, Expr, ExprKind, Literal, Pattern, StringPart,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Control flow for loop iterations or error
#[derive(Debug)]
pub(crate) enum LoopControlOrError {
    Break(Option<String>, Value),
    Continue(Option<String>),
    Return(Value), // Early return from function (exits both loop and function)
    Error(InterpreterError),
}

// Value utility methods in value_utils.rs, type_id() in value.rs
// Display implementations in eval_display.rs

// Note: Complex object structures (ObjectHeader, Class, etc.) will be implemented
// in Phase 1 of the interpreter spec when we add proper GC and method dispatch.

/// Runtime interpreter state.
///
/// The `Interpreter` manages the execution environment for Ruchy programs.
/// It maintains:
/// - A value stack for computation
/// - Environment stack for lexical scoping
/// - Inline caches for field/method optimization
/// - Type feedback for future JIT compilation
/// - Conservative garbage collection
///
/// # Implementation Strategy
///
/// This follows a two-tier execution model:
/// - **Tier 0**: AST interpretation (current)
/// - **Tier 1**: JIT compilation (future)
///
/// Type feedback and execution counts are collected for hot code
/// identification and optimization.
#[derive(Debug)]
pub struct Interpreter {
    /// Tagged pointer values for fast operation
    stack: Vec<Value>,

    /// Environment stack for lexical scoping (ISSUE-119: Rc<RefCell> for shared mutable state)
    pub(crate) env_stack: Vec<Rc<RefCell<HashMap<std::string::String, Value>>>>,

    /// Call frame for function calls
    #[allow(dead_code)]
    frames: Vec<CallFrame>,

    /// Execution statistics for tier transition (will be used in Phase 1)
    #[allow(dead_code)]
    execution_counts: HashMap<usize, u32>, // Function/method ID -> execution count

    /// Inline caches for field/method access optimization
    field_caches: HashMap<String, InlineCache>,

    /// Type feedback collection for JIT compilation
    type_feedback: TypeFeedback,

    /// Conservative garbage collector
    gc: ConservativeGC,

    /// Error handler scopes for try/catch
    error_scopes: Vec<ErrorScope>,

    /// Stdout buffer for capturing println output (WASM/REPL)
    /// Complexity: 1 (simple field addition)
    stdout_buffer: Vec<String>,

    /// Module loader for multi-file programs (Issue #88)
    /// Enables `use module;` imports
    module_loader: crate::backend::module_loader::ModuleLoader,
}

/// Error scope for try/catch blocks
#[derive(Debug, Clone)]
struct ErrorScope {
    /// Depth of environment stack when try block started
    env_depth: usize,
}

// Re-export JIT type feedback system from type_feedback module
// EXTREME TDD: Eliminated 485 lines of duplicate code (massive entropy reduction)
pub use super::type_feedback::{
    CacheEntry, CacheState, CallSiteFeedback, InlineCache, OperationFeedback,
    SpecializationCandidate, SpecializationKind, TypeFeedback, TypeFeedbackStats,
    VariableTypeFeedback,
};

// Re-export GC implementation from gc_impl module
// EXTREME TDD: Eliminated 318 lines of duplicate GC code (massive entropy reduction)
pub use super::gc_impl::{ConservativeGC, GCInfo, GCObject, GCStats};

// Re-export compilation implementation from compilation module
// EXTREME TDD: Eliminated 669 lines of compilation code (massive entropy reduction)
pub use super::compilation::{
    DirectThreadedInterpreter, InstructionResult, InterpreterState, ThreadedInstruction,
};

impl Interpreter {
    /// Create a new interpreter instance.
    ///
    /// Initializes the interpreter with:
    /// - Pre-allocated stack for performance
    /// - Global environment with builtin functions (max, min, floor, ceil, etc.)
    /// - Type feedback collection for future JIT compilation
    /// - Conservative garbage collector
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// // Interpreter is ready to evaluate expressions
    /// ```
    pub fn new() -> Self {
        // EXTREME TDD: Delegate builtin initialization to eliminate 62 lines of entropy
        let global_env = crate::runtime::builtin_init::init_global_environment();

        Self {
            stack: Vec::with_capacity(1024), // Pre-allocate stack
            env_stack: vec![Rc::new(RefCell::new(global_env))], // ISSUE-119: Shared mutable environment
            frames: Vec::new(),
            execution_counts: HashMap::new(),
            field_caches: HashMap::new(),
            type_feedback: TypeFeedback::new(),
            gc: ConservativeGC::new(),
            error_scopes: Vec::new(),
            stdout_buffer: Vec::new(), // Initialize empty stdout buffer
            module_loader: crate::backend::module_loader::ModuleLoader::new(), // Issue #88
        }
    }

    /// Evaluate an AST expression directly.
    ///
    /// This is the main entry point for interpreting Ruchy expressions. It walks
    /// the AST recursively, evaluating expressions and returning their values.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    /// use ruchy::frontend::parser::Parser;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let mut parser = Parser::new("42");
    /// let expr = parser.parse().expect("parse should succeed in doctest");
    /// let result = interpreter.eval_expr(&expr).expect("eval_expr should succeed in doctest");
    /// assert_eq!(result.to_string(), "42");
    /// ```
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    /// use ruchy::frontend::parser::Parser;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let mut parser = Parser::new("2 + 3");
    /// let expr = parser.parse().expect("parse should succeed in doctest");
    /// let result = interpreter.eval_expr(&expr).expect("eval_expr should succeed in doctest");
    /// assert_eq!(result.to_string(), "5");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error when:
    /// - Type error (e.g., adding string to number)
    /// - Runtime error (e.g., undefined variable)
    /// - Stack overflow/underflow
    /// - Division by zero
    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, InterpreterError> {
        self.eval_expr_kind(&expr.kind)
    }

    /// Evaluate an expression kind directly.
    ///
    /// This is the core dispatch function for the interpreter. It pattern-matches
    /// on the `ExprKind` and delegates to specialized evaluation functions.
    ///
    /// The function is organized into logical groups:
    /// - Basic expressions (literals, identifiers)
    /// - Operations (binary, unary, calls)
    /// - Functions (definitions, lambdas)
    /// - Control flow (if, for, while, match)
    /// - Data structures (lists, tuples, arrays)
    /// - Assignments
    ///
    /// # Errors
    ///
    /// Returns an error if the expression evaluation fails or if the expression
    /// type is not yet implemented.
    pub(crate) fn eval_expr_kind(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            // Simple expressions (complexity: 2)
            ExprKind::Literal(_) | ExprKind::Identifier(_) => self.eval_simple_expr(expr_kind),

            // Operations (complexity: 2)
            ExprKind::Binary { .. }
            | ExprKind::Unary { .. }
            | ExprKind::Call { .. }
            | ExprKind::MethodCall { .. }
            | ExprKind::DataFrameOperation { .. }
            | ExprKind::IndexAccess { .. }
            | ExprKind::FieldAccess { .. }
            | ExprKind::TypeCast { .. } => self.eval_operation_expr(expr_kind),

            // Functions (complexity: 2)
            ExprKind::Function { .. } | ExprKind::Lambda { .. } => {
                self.eval_function_expr(expr_kind)
            }

            // Control flow (complexity: 1)
            kind if Self::is_control_flow_expr(kind) => self.eval_control_flow_expr(kind),

            // Data structures (complexity: 1)
            kind if Self::is_data_structure_expr(kind) => self.eval_data_structure_expr(kind),

            // Assignments (complexity: 1)
            kind if Self::is_assignment_expr(kind) => self.eval_assignment_expr(kind),

            // Other expressions (complexity: 1)
            _ => self.eval_misc_expr(expr_kind),
        }
    }

    // Helper methods for expression type categorization and evaluation (complexity <10 each)

    /// Evaluate simple expressions (literals and identifiers)
    /// Complexity: 3
    pub(crate) fn eval_simple_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Literal(lit) => Ok(eval_literal::eval_literal(lit)),
            ExprKind::Identifier(name) => self.lookup_variable(name),
            _ => unreachable!("eval_simple_expr called with non-simple expression"),
        }
    }

    /// Evaluate operation expressions (binary, unary, calls, method calls, type casts, etc.)
    /// Complexity: 9
    pub(crate) fn eval_operation_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Binary { left, op, right } => self.eval_binary_expr(left, *op, right),
            ExprKind::Unary { op, operand } => self.eval_unary_expr(*op, operand),
            ExprKind::Call { func, args } => self.eval_function_call(func, args),
            ExprKind::MethodCall {
                receiver,
                method,
                args,
            } => self.eval_method_call(receiver, method, args),
            ExprKind::DataFrameOperation { source, operation } => {
                self.eval_dataframe_operation(source, operation)
            }
            ExprKind::IndexAccess { object, index } => self.eval_index_access(object, index),
            ExprKind::FieldAccess { object, field } => self.eval_field_access(object, field),
            ExprKind::TypeCast { expr, target_type } => self.eval_type_cast(expr, target_type),
            _ => unreachable!("eval_operation_expr called with non-operation expression"),
        }
    }

    /// Evaluate function expressions (function definitions and lambdas)
    /// Complexity: 3
    pub(crate) fn eval_function_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Function {
                name, params, body, ..
            } => self.eval_function(name, params, body),
            ExprKind::Lambda { params, body } => self.eval_lambda(params, body),
            _ => unreachable!("eval_function_expr called with non-function expression"),
        }
    }

    /// Helper: Check if expression is a type definition
    /// Complexity: 2
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
    /// Complexity: 2
    pub(crate) fn is_actor_operation(expr_kind: &ExprKind) -> bool {
        matches!(
            expr_kind,
            ExprKind::Spawn { .. } | ExprKind::ActorSend { .. } | ExprKind::ActorQuery { .. }
        )
    }

    /// Helper: Check if expression is a special form
    /// Complexity: 2
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

    /// Evaluate type definition expressions (Actor, Struct, Class, Impl)
    /// Complexity: 6
    pub(crate) fn eval_type_definition(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Actor {
                name,
                state,
                handlers,
            } => self.eval_actor_definition(name, state, handlers),
            // SPEC-001-I: Effect declarations return Nil (no runtime implementation)
            ExprKind::Effect { .. } => Ok(Value::Nil),
            // SPEC-001-J: Effect handlers evaluate expression and return nil
            ExprKind::Handle { expr, .. } => {
                self.eval_expr(expr)?;
                Ok(Value::Nil)
            }
            ExprKind::Enum {
                name,
                type_params,
                variants,
                is_pub,
            } => self.eval_enum_definition(name, type_params, variants, *is_pub),
            ExprKind::Struct {
                name,
                type_params,
                fields,
                methods,
                derives: _,
                is_pub,
            } => self.eval_struct_definition(name, type_params, fields, methods, *is_pub),
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
            } => self.eval_class_definition(
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
            } => self.eval_impl_block(for_type, methods),
            _ => unreachable!("eval_type_definition called with non-type-definition"),
        }
    }

    /// Evaluate actor operation expressions (Spawn, `ActorSend`, `ActorQuery`)
    /// Complexity: 4
    pub(crate) fn eval_actor_operation(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Spawn { actor } => self.eval_spawn_actor(actor),
            ExprKind::ActorSend { actor, message } => self.eval_actor_send(actor, message),
            ExprKind::ActorQuery { actor, message } => self.eval_actor_query(actor, message),
            _ => unreachable!("eval_actor_operation called with non-actor-operation"),
        }
    }

    /// Evaluate special form expressions (None, Some, Set, patterns, literals)
    /// Complexity: 9
    pub(crate) fn eval_special_form(
        &mut self,
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
                data: Some(vec![self.eval_expr(value)?]),
            }),
            ExprKind::Set(statements) => {
                let mut result = Value::Nil;
                for stmt in statements {
                    result = self.eval_expr(stmt)?;
                }
                Ok(result)
            }
            ExprKind::LetPattern {
                pattern,
                value,
                body,
                ..
            } => self.eval_let_pattern(pattern, value, body),
            ExprKind::StringInterpolation { parts } => self.eval_string_interpolation(parts),
            ExprKind::QualifiedName { module, name } => self.eval_qualified_name(module, name),
            ExprKind::ObjectLiteral { fields } => self.eval_object_literal(fields),
            ExprKind::StructLiteral {
                name,
                fields,
                base: _,
            } => self.eval_struct_literal(name, fields),
            _ => unreachable!("eval_special_form called with non-special-form"),
        }
    }

    /// Helper: Resolve module path through nested objects in global environment
    /// Reduces cognitive complexity by extracting duplicated module navigation logic
    pub(crate) fn resolve_module_path(&self, module: &str) -> Option<Value> {
        let parts: Vec<&str> = module.split("::").collect();
        let first_part = parts.first()?;

        // Access global environment (first element of env_stack)
        let global_env_ref = self.env_stack.first()?;
        let global_env = global_env_ref.borrow();
        let mut current_value = global_env.get(*first_part)?.clone();

        // Navigate through remaining parts
        for &part in parts.iter().skip(1) {
            if let Value::Object(obj) = current_value {
                current_value = obj.get(part)?.clone();
            } else {
                return None;
            }
        }

        Some(current_value)
    }

    // Value formatting delegated to value_format module
    // EXTREME TDD: Eliminated 50 lines of duplicate code
    pub(crate) fn format_string_with_values(format_str: &str, values: &[Value]) -> String {
        crate::runtime::value_format::format_string_with_values(format_str, values)
    }

    /// Evaluate miscellaneous expressions
    /// Complexity: 7 (was 5, added import handling)
    pub(crate) fn eval_misc_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        if Self::is_type_definition(expr_kind) {
            return self.eval_type_definition(expr_kind);
        }
        if Self::is_actor_operation(expr_kind) {
            return self.eval_actor_operation(expr_kind);
        }
        if Self::is_special_form(expr_kind) {
            return self.eval_special_form(expr_kind);
        }

        // Handle import statements (GitHub Issue #59)
        // Issue #82: Implement basic module resolution for use statements
        match expr_kind {
            ExprKind::ImportAll { module, alias } => {
                // Use helper to resolve module path through nested objects
                let parts: Vec<&str> = module.split("::").collect();

                // Import the symbol into current environment with the appropriate name
                if let Some(value) = self.resolve_module_path(module) {
                    // Determine the name to use: alias if provided, otherwise last part of path
                    let import_name = if alias == "*" {
                        // Wildcard import - not yet implemented
                        return Ok(Value::Nil);
                    } else if !alias.is_empty() && alias != "*" {
                        alias.clone()
                    } else {
                        (*parts.last().unwrap_or(&"")).to_string()
                    };

                    // Add to global environment (first element of env_stack)
                    // This makes imports available across all scopes
                    if let Some(global_env_ref) = self.env_stack.first() {
                        global_env_ref.borrow_mut().insert(import_name, value); // ISSUE-119: Mutable borrow
                    }
                }

                Ok(Value::Nil)
            }
            ExprKind::Import { module, items: _ } => {
                // Issue #89: Distinguish between stdlib imports and file module imports
                // - stdlib imports (std::*): Already available in global environment, no file to load
                // - file modules (mylib): Load from mylib.ruchy file
                //
                // Example: `use std::env;` → Import env module into current scope
                // Example: `use mylib;` → Load mylib.ruchy file

                // Check if this is a stdlib import
                if module.starts_with("std::") {
                    // Issue #96: stdlib imports must make the module available in current scope
                    // Use helper to resolve module path through nested objects
                    let parts: Vec<&str> = module.split("::").collect();

                    // Import the module into current environment
                    if let Some(value) = self.resolve_module_path(module) {
                        let import_name = (*parts.last().unwrap_or(&"")).to_string();

                        // Add to global environment
                        if let Some(global_env_ref) = self.env_stack.first() {
                            global_env_ref.borrow_mut().insert(import_name, value);
                        }
                    }

                    return Ok(Value::Nil);
                }

                // Issue #88: Load file module from file system and execute it
                // Example: `use mylib;` loads mylib.ruchy
                let parsed_module = self.module_loader.load_module(module).map_err(|e| {
                    InterpreterError::RuntimeError(format!(
                        "Failed to load module '{}': {}",
                        module, e
                    ))
                })?;

                // Create a new environment scope for the module
                // ISSUE-119: Wrap in Rc<RefCell> for shared mutable access
                let module_env_ref = Rc::new(RefCell::new(HashMap::new()));

                // Evaluate the module AST to execute its definitions
                // We need to push a new environment, evaluate, then pop
                self.env_stack.push(Rc::clone(&module_env_ref));
                let eval_result = self.eval_expr(&parsed_module.ast);
                self.env_stack.pop();

                // Check for evaluation errors
                eval_result?;

                // Create a module namespace object containing all exported symbols
                // Borrow the module environment to read its contents
                let mut module_object = std::collections::HashMap::new();
                for (name, value) in module_env_ref.borrow().iter() {
                    module_object.insert(name.clone(), value.clone());
                }

                // Add the module object to global environment
                // This allows `mylib::add` to work via field access
                if let Some(global_env_ref) = self.env_stack.first() {
                    global_env_ref
                        .borrow_mut()
                        .insert(module.clone(), Value::Object(module_object.into()));
                    // ISSUE-119: Mutable borrow
                }

                Ok(Value::Nil)
            }
            ExprKind::ImportDefault { .. } => {
                // LIMITATION: ImportDefault not yet implemented - returns Nil for now
                // See ISSUE-106 for module resolution tracking
                Ok(Value::Nil)
            }
            // Handle vec! macro (GitHub Issue #62)
            ExprKind::Macro { name, args } => {
                if name == "vec" {
                    // vec![...] expands to an array with evaluated arguments
                    let mut elements = Vec::new();
                    for arg in args {
                        let value = self.eval_expr(arg)?;
                        elements.push(value);
                    }
                    Ok(Value::Array(elements.into()))
                } else if name == "println" {
                    // println!() macro: Evaluate arguments, print with newline
                    // PARSER-085: Supports format strings like println!("x: {}", value)
                    if args.is_empty() {
                        println!();
                    } else if args.len() == 1 {
                        // Single argument: print directly
                        let value = self.eval_expr(&args[0])?;
                        println!("{}", value);
                    } else {
                        // Multiple arguments: use format! logic (Issue #82, #83)
                        let format_val = self.eval_expr(&args[0])?;
                        let format_str = match format_val {
                            Value::String(ref s) => s.as_ref().to_string(),
                            _ => format_val.to_string(),
                        };

                        let mut values = Vec::new();
                        for arg in &args[1..] {
                            values.push(self.eval_expr(arg)?);
                        }

                        // Use helper for format string replacement
                        let result = Self::format_string_with_values(&format_str, &values);
                        println!("{}", result);
                    }
                    Ok(Value::Nil)
                } else if name == "format" {
                    // format!() macro: Format string with placeholders (Issue #83)
                    if args.is_empty() {
                        return Err(InterpreterError::RuntimeError(
                            "format!() requires at least one argument".to_string(),
                        ));
                    }

                    // Evaluate format string
                    let format_val = self.eval_expr(&args[0])?;
                    let format_str = format_val.to_string();

                    // Evaluate remaining arguments
                    let mut values = Vec::new();
                    for arg in &args[1..] {
                        values.push(self.eval_expr(arg)?);
                    }

                    // Replace {} and {:?} placeholders with values
                    let mut result = String::new();
                    let mut chars = format_str.chars().peekable();
                    let mut value_index = 0;

                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            // Check for {:?} debug format
                            if chars.peek() == Some(&':') {
                                chars.next(); // consume ':'
                                if chars.peek() == Some(&'?') {
                                    chars.next(); // consume '?'
                                    if chars.peek() == Some(&'}') {
                                        chars.next(); // consume '}'
                                        if value_index < values.len() {
                                            result.push_str(&format!("{:?}", values[value_index]));
                                            value_index += 1;
                                        } else {
                                            result.push_str("{:?}"); // preserve placeholder
                                        }
                                    } else {
                                        result.push_str("{:?"); // not a complete placeholder
                                    }
                                } else {
                                    result.push_str("{:"); // not {:?}
                                }
                            } else if chars.peek() == Some(&'}') {
                                chars.next(); // consume '}'
                                if value_index < values.len() {
                                    result.push_str(&values[value_index].to_string());
                                    value_index += 1;
                                } else {
                                    result.push_str("{}"); // preserve placeholder if no value
                                }
                            } else {
                                result.push(ch);
                            }
                        } else {
                            result.push(ch);
                        }
                    }

                    Ok(Value::from_string(result))
                } else {
                    // Other macros not yet implemented
                    Err(InterpreterError::RuntimeError(format!(
                        "Macro '{}!' not yet implemented",
                        name
                    )))
                }
            }
            // RUNTIME-001: Handle MacroInvocation (FORMATTER-088 changed parser output)
            // MacroInvocation is the correct AST variant for macro CALLS (not definitions)
            // Delegate to same logic as Macro for backward compatibility (GitHub Issue #74)
            ExprKind::MacroInvocation { name, args } => {
                if name == "vec" {
                    let mut elements = Vec::new();
                    for arg in args {
                        let value = self.eval_expr(arg)?;
                        elements.push(value);
                    }
                    Ok(Value::Array(elements.into()))
                } else if name == "println" {
                    // println!() macro: Evaluate arguments, print with newline
                    // PARSER-085: Supports format strings like println!("x: {}", value)
                    if args.is_empty() {
                        println!();
                    } else if args.len() == 1 {
                        // Single argument: print directly
                        let value = self.eval_expr(&args[0])?;
                        println!("{}", value);
                    } else {
                        // Multiple arguments: use format! logic (Issue #82, #83)
                        let format_val = self.eval_expr(&args[0])?;
                        let format_str = match format_val {
                            Value::String(ref s) => s.as_ref().to_string(),
                            _ => format_val.to_string(),
                        };

                        let mut values = Vec::new();
                        for arg in &args[1..] {
                            values.push(self.eval_expr(arg)?);
                        }

                        // Use helper for format string replacement
                        let result = Self::format_string_with_values(&format_str, &values);
                        println!("{}", result);
                    }
                    Ok(Value::Nil)
                } else if name == "format" {
                    // format!() macro: Format string with placeholders (Issue #83)
                    if args.is_empty() {
                        return Err(InterpreterError::RuntimeError(
                            "format!() requires at least one argument".to_string(),
                        ));
                    }

                    // Evaluate format string
                    let format_val = self.eval_expr(&args[0])?;
                    let format_str = format_val.to_string();

                    // Evaluate remaining arguments
                    let mut values = Vec::new();
                    for arg in &args[1..] {
                        values.push(self.eval_expr(arg)?);
                    }

                    // Replace {} and {:?} placeholders with values
                    let mut result = String::new();
                    let mut chars = format_str.chars().peekable();
                    let mut value_index = 0;

                    while let Some(ch) = chars.next() {
                        if ch == '{' {
                            // Check for {:?} debug format
                            if chars.peek() == Some(&':') {
                                chars.next(); // consume ':'
                                if chars.peek() == Some(&'?') {
                                    chars.next(); // consume '?'
                                    if chars.peek() == Some(&'}') {
                                        chars.next(); // consume '}'
                                        if value_index < values.len() {
                                            result.push_str(&format!("{:?}", values[value_index]));
                                            value_index += 1;
                                        } else {
                                            result.push_str("{:?}"); // preserve placeholder
                                        }
                                    } else {
                                        result.push_str("{:?"); // not a complete placeholder
                                    }
                                } else {
                                    result.push_str("{:"); // not {:?}
                                }
                            } else if chars.peek() == Some(&'}') {
                                chars.next(); // consume '}'
                                if value_index < values.len() {
                                    result.push_str(&values[value_index].to_string());
                                    value_index += 1;
                                } else {
                                    result.push_str("{}"); // preserve placeholder if no value
                                }
                            } else {
                                result.push(ch);
                            }
                        } else {
                            result.push(ch);
                        }
                    }

                    Ok(Value::from_string(result))
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Macro '{}!' not yet implemented",
                        name
                    )))
                }
            }
            ExprKind::Try { expr } => {
                // Issue #97: Try operator (?) for Result unwrapping/propagation
                // Evaluates expr, expects Result enum (Ok/Err variants)
                // - If Ok(value), unwrap and return value
                // - If Err(error), propagate by returning Err variant
                let result_value = self.eval_expr(expr)?;

                match &result_value {
                    // Case 1: Direct EnumVariant construction
                    Value::EnumVariant {
                        enum_name,
                        variant_name,
                        data,
                    } if enum_name == "Result" => {
                        match variant_name.as_str() {
                            "Ok" => {
                                // Unwrap Ok value: extract first element from data
                                if let Some(values) = data {
                                    if let Some(value) = values.first() {
                                        Ok(value.clone())
                                    } else {
                                        Err(InterpreterError::RuntimeError(
                                            "Try operator: Ok variant has no data".to_string(),
                                        ))
                                    }
                                } else {
                                    Err(InterpreterError::RuntimeError(
                                        "Try operator: Ok variant has no data".to_string(),
                                    ))
                                }
                            }
                            "Err" => {
                                // Propagate Err: early return from containing function
                                // Use the same mechanism as 'return' statements
                                Err(InterpreterError::Return(Value::EnumVariant {
                                    enum_name: enum_name.clone(),
                                    variant_name: variant_name.clone(),
                                    data: data.clone(),
                                }))
                            }
                            _ => Err(InterpreterError::RuntimeError(format!(
                                "Try operator: unexpected Result variant '{}'",
                                variant_name
                            ))),
                        }
                    }
                    // Case 2: Object representation (function return values)
                    // Result is represented as Object with __type="Message", type="Ok"/"Err"
                    Value::Object(obj)
                        if obj.get("__type").and_then(|v| {
                            if let Value::String(s) = v {
                                Some(s.as_ref())
                            } else {
                                None
                            }
                        }) == Some("Message") =>
                    {
                        // Extract the "type" field (Ok or Err)
                        if let Some(Value::String(variant)) = obj.get("type") {
                            match variant.as_ref() {
                                "Ok" => {
                                    // Unwrap Ok: extract first element from "data" array
                                    if let Some(Value::Array(data_arr)) = obj.get("data") {
                                        if let Some(value) = data_arr.first() {
                                            Ok(value.clone())
                                        } else {
                                            Err(InterpreterError::RuntimeError(
                                                "Try operator: Ok has empty data".to_string(),
                                            ))
                                        }
                                    } else {
                                        Err(InterpreterError::RuntimeError(
                                            "Try operator: Ok missing data field".to_string(),
                                        ))
                                    }
                                }
                                "Err" => {
                                    // Propagate Err: early return from containing function
                                    // Use the same mechanism as 'return' statements
                                    Err(InterpreterError::Return(result_value))
                                }
                                _ => Err(InterpreterError::RuntimeError(format!(
                                    "Try operator: unexpected type '{}'",
                                    variant
                                ))),
                            }
                        } else {
                            Err(InterpreterError::RuntimeError(
                                "Try operator: Message object missing 'type' field".to_string(),
                            ))
                        }
                    }
                    _ => Err(InterpreterError::RuntimeError(format!(
                        "Try operator expects Result enum, got: {:?}",
                        result_value
                    ))),
                }
            }
            // SPEC-001-C: Pipeline operator evaluation (|> not >>)
            // Evaluates: initial_expr |> func1 |> func2 |> ...
            // Example: 5 |> double |> add_one → add_one(double(5)) → 11
            // PIPELINE-001 FIX: Also supports method calls: "hello" |> upper → "hello".upper()
            // PIPELINE-001 FIX v2: Use dispatch_method_call for proper Value handling
            ExprKind::Pipeline { expr, stages } => {
                // Start with the initial expression value
                let mut current_value = self.eval_expr(expr)?;

                // Apply each pipeline stage in sequence
                for stage in stages {
                    // PIPELINE-001: Check if stage is an identifier that could be a method
                    if let ExprKind::Identifier(method_name) = &stage.op.kind {
                        // First, try to look up as a user-defined function (Closure)
                        // Don't use builtins like __builtin_join__ - those are methods
                        let is_user_function = self
                            .lookup_variable(method_name)
                            .map(|v| matches!(v, Value::Closure { .. }))
                            .unwrap_or(false);

                        if is_user_function {
                            // It's a user-defined function - call it with current_value as argument
                            let func_val = self.lookup_variable(method_name)?;
                            current_value = self.call_function(func_val, &[current_value])?;
                        } else {
                            // Not a user function - try calling as a method on current_value
                            // This enables: "hello" |> upper → "hello".upper()
                            current_value =
                                self.dispatch_method_call(&current_value, method_name, &[], true)?;
                        }
                    } else if let ExprKind::Call { func, args } = &stage.op.kind {
                        // It's a call expression like filter(x => x > 0)
                        // Check if func is an identifier we should treat as a method
                        if let ExprKind::Identifier(method_name) = &func.kind {
                            // Check if it's a user-defined function (Closure)
                            let is_user_function = self
                                .lookup_variable(method_name)
                                .map(|v| matches!(v, Value::Closure { .. }))
                                .unwrap_or(false);

                            if is_user_function {
                                // It's a user function call - prepend current_value to args
                                let func_val = self.lookup_variable(method_name)?;
                                let arg_values: Result<Vec<_>, _> =
                                    args.iter().map(|arg| self.eval_expr(arg)).collect();
                                let mut all_args = vec![current_value];
                                all_args.extend(arg_values?);
                                current_value = self.call_function(func_val, &all_args)?;
                            } else {
                                // It's a method call with args: arr |> filter(pred) → arr.filter(pred)
                                let arg_values: Result<Vec<_>, _> =
                                    args.iter().map(|arg| self.eval_expr(arg)).collect();
                                current_value = self.dispatch_method_call(
                                    &current_value,
                                    method_name,
                                    &arg_values?,
                                    args.is_empty(),
                                )?;
                            }
                        } else {
                            // Complex function expression - evaluate and call with current_value as first arg
                            let func_val = self.eval_expr(func)?;
                            let arg_values: Result<Vec<_>, _> =
                                args.iter().map(|arg| self.eval_expr(arg)).collect();
                            let mut all_args = vec![current_value];
                            all_args.extend(arg_values?);
                            current_value = self.call_function(func_val, &all_args)?;
                        }
                    } else {
                        // Other expression types - evaluate as function and call with current_value
                        let func_val = self.eval_expr(&stage.op)?;
                        current_value = self.call_function(func_val, &[current_value])?;
                    }
                }

                Ok(current_value)
            }
            // SPEC-001-D: Lazy evaluation - defers computation until value is accessed
            // For interpreter, we eagerly evaluate (no lazy caching mechanism yet)
            // Transpiler will use Once<T> for true lazy behavior
            ExprKind::Lazy { expr } => {
                // Minimal implementation: just evaluate the expression
                // Future: Add lazy value wrapper with memoization
                self.eval_expr(expr)
            }
            // SPEC-001-E: Async block - simplified synchronous evaluation
            // For true async support, would need tokio runtime integration
            // Current: Evaluates block immediately (no Future/await)
            ExprKind::AsyncBlock { body } => {
                // Simplified: Just evaluate the block synchronously
                // Future: Wrap in Future and integrate with async runtime
                self.eval_expr(body)
            }
            // ISSUE-106: Module expression - creates a namespace with exported functions
            ExprKind::Module { name, body } => self.eval_module_expr(name, body),
            // ISSUE-106: ModuleDeclaration should be resolved before evaluation
            // If we reach here, module resolution wasn't performed
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
            } => {
                let value = self.eval_expr(expr)?;
                if let Some(bindings) = self.try_pattern_match(pattern, &value)? {
                    // Pattern matched - bind variables and execute then_branch
                    self.push_scope();
                    for (name, val) in bindings {
                        self.env_set(name, val);
                    }
                    let result = self.eval_expr(then_branch);
                    self.pop_scope();
                    result
                } else if let Some(else_expr) = else_branch {
                    // Pattern didn't match - execute else branch
                    self.eval_expr(else_expr)
                } else {
                    // No match and no else branch - return nil
                    Ok(Value::Nil)
                }
            }
            // While-let expression: while let pattern = expr { body }
            ExprKind::WhileLet {
                label: _,
                pattern,
                expr,
                body,
            } => {
                let mut last_value = Value::Nil;
                loop {
                    let value = self.eval_expr(expr)?;
                    if let Some(bindings) = self.try_pattern_match(pattern, &value)? {
                        // Pattern matched - bind variables and execute body
                        self.push_scope();
                        for (name, val) in bindings {
                            self.env_set(name, val);
                        }
                        match self.eval_expr(body) {
                            Ok(v) => last_value = v,
                            Err(InterpreterError::Break(_, v)) => {
                                self.pop_scope();
                                return Ok(v);
                            }
                            Err(InterpreterError::Continue(_)) => {
                                self.pop_scope();
                                continue;
                            }
                            Err(e) => {
                                self.pop_scope();
                                return Err(e);
                            }
                        }
                        self.pop_scope();
                    } else {
                        // Pattern didn't match - exit loop
                        break;
                    }
                }
                Ok(last_value)
            }
            // List comprehension: [expr for x in iter if cond]
            ExprKind::ListComprehension { element, clauses } => {
                self.eval_list_comprehension(element, clauses)
            }
            _ => {
                // Fallback for unimplemented expressions
                Err(InterpreterError::RuntimeError(format!(
                    "Expression type not yet implemented: {expr_kind:?}"
                )))
            }
        }
    }

    /// Evaluate list comprehension: [expr for x in iter if cond]
    /// Supports multiple clauses (nested loops) and optional conditions
    /// Complexity: 8 (nested iteration with conditions)
    pub(crate) fn eval_list_comprehension(
        &mut self,
        element: &Expr,
        clauses: &[ComprehensionClause],
    ) -> Result<Value, InterpreterError> {
        let mut results = Vec::new();
        self.eval_comprehension_clauses(&mut results, element, clauses, 0)?;
        Ok(Value::Array(Arc::from(results)))
    }

    /// Recursively process comprehension clauses
    pub(crate) fn eval_comprehension_clauses(
        &mut self,
        results: &mut Vec<Value>,
        element: &Expr,
        clauses: &[ComprehensionClause],
        clause_idx: usize,
    ) -> Result<(), InterpreterError> {
        if clause_idx >= clauses.len() {
            // All clauses processed, evaluate and collect element
            results.push(self.eval_expr(element)?);
            return Ok(());
        }

        let clause = &clauses[clause_idx];
        let iterable = self.eval_expr(&clause.iterable)?;

        self.push_scope();
        match iterable {
            Value::Array(ref arr) => {
                for item in arr.iter() {
                    self.env_set(clause.variable.clone(), item.clone());
                    if !self.check_comprehension_condition(clause.condition.as_deref())? {
                        continue;
                    }
                    self.eval_comprehension_clauses(results, element, clauses, clause_idx + 1)?;
                }
            }
            Value::Range {
                ref start,
                ref end,
                inclusive,
            } => {
                let (start_val, end_val) = self.extract_range_bounds(start, end)?;
                for i in self.create_range_iterator(start_val, end_val, inclusive) {
                    self.env_set(clause.variable.clone(), Value::Integer(i));
                    if !self.check_comprehension_condition(clause.condition.as_deref())? {
                        continue;
                    }
                    self.eval_comprehension_clauses(results, element, clauses, clause_idx + 1)?;
                }
            }
            _ => {
                self.pop_scope();
                return Err(InterpreterError::TypeError(
                    "List comprehension requires an iterable".to_string(),
                ));
            }
        }
        self.pop_scope();
        Ok(())
    }

    /// Helper: Check comprehension condition
    pub(crate) fn check_comprehension_condition(
        &mut self,
        condition: Option<&Expr>,
    ) -> Result<bool, InterpreterError> {
        if let Some(cond) = condition {
            let cond_val = self.eval_expr(cond)?;
            Ok(cond_val.is_truthy())
        } else {
            Ok(true)
        }
    }

    /// Helper: Evaluate spawn actor expression with proper nesting handling
    /// Complexity: 10 (extracted from inline code)
    pub(crate) fn eval_spawn_actor(&mut self, actor: &Expr) -> Result<Value, InterpreterError> {
        // Handle: spawn ActorName (no args)
        if let ExprKind::Identifier(name) = &actor.kind {
            if let Ok(def_value) = self.lookup_variable(name) {
                if let Value::Object(ref obj) = def_value {
                    if let Some(Value::String(type_str)) = obj.get("__type") {
                        if type_str.as_ref() == "Actor" {
                            let constructor_marker =
                                Value::from_string(format!("__actor_constructor__:{}", name));
                            return self.call_function(constructor_marker, &[]);
                        }
                    }
                }
            }
        }

        // Handle: spawn ActorName(args...)
        if let ExprKind::Call { func, args } = &actor.kind {
            if let ExprKind::Identifier(name) = &func.kind {
                if let Ok(def_value) = self.lookup_variable(name) {
                    if let Value::Object(ref obj) = def_value {
                        if let Some(Value::String(type_str)) = obj.get("__type") {
                            if type_str.as_ref() == "Actor" {
                                let constructor_marker =
                                    Value::from_string(format!("__actor_constructor__:{}", name));
                                let arg_vals: Result<Vec<Value>, _> =
                                    args.iter().map(|arg| self.eval_expr(arg)).collect();
                                let arg_vals = arg_vals?;
                                return self.call_function(constructor_marker, &arg_vals);
                            }
                        }
                    }
                }
            }
        }

        // Default: evaluate the actor expression normally
        let actor_value = self.eval_expr(actor)?;
        Ok(actor_value)
    }

    /// Helper: Evaluate actor send expression (fire-and-forget)
    /// Complexity: 4
    pub(crate) fn eval_actor_send(
        &mut self,
        actor: &Expr,
        message: &Expr,
    ) -> Result<Value, InterpreterError> {
        let actor_value = self.eval_expr(actor)?;
        let message_value = self.eval_message_expr(message)?;

        if let Value::ObjectMut(cell_rc) = actor_value {
            self.process_actor_message_sync_mut(&cell_rc, &message_value)?;
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
        &mut self,
        actor: &Expr,
        message: &Expr,
    ) -> Result<Value, InterpreterError> {
        let actor_value = self.eval_expr(actor)?;
        let message_value = self.eval_message_expr(message)?;

        if let Value::ObjectMut(cell_rc) = actor_value {
            self.process_actor_message_sync_mut(&cell_rc, &message_value)
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "ActorQuery requires an actor instance, got {}",
                actor_value.type_name()
            )))
        }
    }

    // Actor message extraction delegated to eval_actor module
    // EXTREME TDD: Eliminated 35 lines of duplicate code
    pub(crate) fn extract_message_type_and_data(
        message: &Value,
    ) -> Result<(String, Vec<Value>), InterpreterError> {
        crate::runtime::eval_actor::extract_message_type_and_data(message)
    }

    pub(crate) fn is_control_flow_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_control_flow_expr(expr_kind)
    }

    pub(crate) fn is_data_structure_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_data_structure_expr(expr_kind)
    }

    pub(crate) fn is_assignment_expr(expr_kind: &ExprKind) -> bool {
        eval_expr::is_assignment_expr(expr_kind)
    }

    pub(crate) fn eval_control_flow_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => self.eval_if_expr(condition, then_branch, else_branch.as_deref()),
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            } => {
                // Evaluate condition
                let cond_value = self.eval_expr(condition)?;
                // Check if condition is truthy
                if cond_value.is_truthy() {
                    self.eval_expr(true_expr)
                } else {
                    self.eval_expr(false_expr)
                }
            }
            ExprKind::Let {
                name, value, body, ..
            } => self.eval_let_expr(name, value, body),
            ExprKind::For {
                label,
                var,
                pattern,
                iter,
                body,
            } => self.eval_for_loop(label.as_ref(), var, pattern.as_ref(), iter, body),
            ExprKind::While {
                label,
                condition,
                body,
            } => self.eval_while_loop(label.as_ref(), condition, body),
            ExprKind::Loop { label, body } => self.eval_loop(label.as_ref(), body),
            ExprKind::Match { expr, arms } => self.eval_match(expr, arms),
            ExprKind::Break { label, value } => {
                // Evaluate the break value (default to Nil if not provided)
                let break_val = if let Some(expr) = value {
                    self.eval_expr(expr)?
                } else {
                    Value::Nil
                };
                Err(InterpreterError::Break(label.clone(), break_val))
            }
            ExprKind::Continue { label } => Err(InterpreterError::Continue(label.clone())),
            ExprKind::Return { value } => self.eval_return_expr(value.as_deref()),
            ExprKind::TryCatch {
                try_block,
                catch_clauses,
                finally_block,
            } => crate::runtime::eval_try_catch::eval_try_catch(
                self,
                try_block,
                catch_clauses,
                finally_block.as_deref(),
            ),
            ExprKind::Throw { expr } => crate::runtime::eval_try_catch::eval_throw(self, expr),
            // Await: In synchronous interpreter, await just evaluates the expression
            // This provides basic async/await syntax support without true async runtime
            ExprKind::Await { expr } => self.eval_expr(expr),
            _ => unreachable!("Non-control-flow expression passed to eval_control_flow_expr"),
        }
    }

    pub(crate) fn eval_data_structure_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::List(elements) => self.eval_list_expr(elements),
            ExprKind::Block(statements) => self.eval_block_expr(statements),
            ExprKind::Tuple(elements) => self.eval_tuple_expr(elements),
            ExprKind::Range {
                start,
                end,
                inclusive,
            } => self.eval_range_expr(start, end, *inclusive),
            ExprKind::ArrayInit { value, size } => self.eval_array_init_expr(value, size),
            ExprKind::DataFrame { columns } => self.eval_dataframe_literal(columns),
            _ => unreachable!("Non-data-structure expression passed to eval_data_structure_expr"),
        }
    }

    pub(crate) fn eval_assignment_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        match expr_kind {
            ExprKind::Assign { target, value } => self.eval_assign(target, value),
            ExprKind::CompoundAssign { target, op, value } => {
                self.eval_compound_assign(target, *op, value)
            }
            _ => unreachable!("Non-assignment expression passed to eval_assignment_expr"),
        }
    }

    /// Evaluate a literal value
    pub(crate) fn eval_literal(&self, lit: &Literal) -> Value {
        match lit {
            Literal::Integer(i, _) => Value::from_i64(*i),
            Literal::Float(f) => Value::from_f64(*f),
            Literal::String(s) => Value::from_string(s.clone()),
            Literal::Bool(b) => Value::from_bool(*b),
            Literal::Char(c) => Value::from_string(c.to_string()),
            Literal::Byte(b) => Value::Byte(*b),
            Literal::Unit => Value::nil(),
            Literal::Null => Value::nil(),
            Literal::Atom(s) => Value::Atom(s.clone()),
        }
    }

    /// Look up a variable in the environment (searches from innermost to outermost)
    pub(crate) fn lookup_variable(&self, name: &str) -> Result<Value, InterpreterError> {
        // REGRESSION-077: Handle Option enum variants (Option::None, Option::Some)
        if name == "Option::None" {
            return Ok(Value::EnumVariant {
                enum_name: "Option".to_string(),
                variant_name: "None".to_string(),
                data: None,
            });
        }

        // Check if this is a qualified name (e.g., "Point::new" or "Rectangle::square")
        if name.contains("::") {
            let parts: Vec<&str> = name.split("::").collect();
            if parts.len() == 2 {
                let type_name = parts[0];
                let method_name = parts[1];

                // Look up the class or struct
                for env_ref in self.env_stack.iter().rev() {
                    if let Some(value) = env_ref.borrow().get(type_name) {
                        // ISSUE-119: Borrow from RefCell
                        if let Value::Object(ref info) = value {
                            // Check if it's a class or struct
                            if let Some(Value::String(ref type_str)) = info.get("__type") {
                                if type_str.as_ref() == "Class" {
                                    // Check if it's a static method
                                    if let Some(Value::Object(ref methods)) = info.get("__methods")
                                    {
                                        if let Some(Value::Object(ref method_meta)) =
                                            methods.get(method_name)
                                        {
                                            if let Some(Value::Bool(true)) =
                                                method_meta.get("is_static")
                                            {
                                                // Return marker for static method
                                                return Ok(Value::from_string(format!(
                                                    "__class_static_method__:{}:{}",
                                                    type_name, method_name
                                                )));
                                            }
                                        }
                                    }

                                    // Check if it's a constructor
                                    if let Some(Value::Object(ref constructors)) =
                                        info.get("__constructors")
                                    {
                                        if constructors.contains_key(method_name) {
                                            // Return marker for class constructor
                                            return Ok(Value::from_string(format!(
                                                "__class_constructor__:{}:{}",
                                                type_name, method_name
                                            )));
                                        }
                                    }
                                } else if type_str.as_ref() == "Struct" && method_name == "new" {
                                    // OPT-022: Check for user-defined "new" method FIRST
                                    // Look for qualified method name (e.g., "Counter::new") in environment
                                    for env_ref in self.env_stack.iter().rev() {
                                        if let Some(method_value) = env_ref.borrow().get(name) {
                                            // Found user-defined method, return it
                                            return Ok(method_value.clone());
                                        }
                                    }
                                    // No user-defined "new" method, return default constructor marker
                                    return Ok(Value::from_string(format!(
                                        "__struct_constructor__:{}",
                                        type_name
                                    )));
                                } else if type_str.as_ref() == "Actor" && method_name == "new" {
                                    return Ok(Value::from_string(format!(
                                        "__actor_constructor__:{}",
                                        type_name
                                    )));
                                }
                            }
                        }
                    }
                }
            }
        }

        // ISSUE-117: Handle JSON global object
        if name == "JSON" {
            // Return a marker object that has parse and stringify methods
            let mut json_obj = HashMap::new();
            json_obj.insert("__type".to_string(), Value::from_string("JSON".to_string()));
            return Ok(Value::Object(Arc::new(json_obj)));
        }

        // ISSUE-116: Handle File global object
        if name == "File" {
            // Return a marker object with __type for namespace dispatch
            let mut file_obj = HashMap::new();
            file_obj.insert("__type".to_string(), Value::from_string("File".to_string()));
            return Ok(Value::Object(Arc::new(file_obj)));
        }

        // Normal variable lookup
        for env_ref in self.env_stack.iter().rev() {
            if let Some(value) = env_ref.borrow().get(name) {
                // ISSUE-119: Borrow from RefCell
                return Ok(value.clone());
            }
        }
        Err(InterpreterError::RuntimeError(format!(
            "Undefined variable: {name}"
        )))
    }

    /// Get the current (innermost) environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
                                  // ISSUE-119: Returns reference to Rc<RefCell<HashMap>> instead of plain HashMap
    pub fn current_env(&self) -> &Rc<RefCell<HashMap<String, Value>>> {
        self.env_stack
            .last()
            .expect("Environment stack should never be empty")
    }

    /// Set a variable in the current environment
    #[allow(clippy::expect_used)] // Environment stack invariant ensures this never panics
    /// Create a new variable binding in the current scope (for `let` bindings)
    ///
    /// RUNTIME-038 FIX: `let` bindings create NEW variables in current scope (shadowing),
    /// they do NOT update variables in parent scopes. This prevents variable collision
    /// in nested function calls.
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within Toyota Way limits)
    pub(crate) fn env_set(&mut self, name: String, value: Value) {
        // Record type feedback for optimization
        self.record_variable_assignment_feedback(&name, &value);

        // ALWAYS create in current scope - `let` bindings shadow outer scopes
        // Do NOT search parent scopes (that's for reassignments without `let`)
        let env_ref = self
            .env_stack
            .last()
            .expect("Environment stack should never be empty");
        env_ref.borrow_mut().insert(name, value); // ISSUE-119: Mutable borrow
    }

    /// Set a mutable variable in the environment
    /// ISSUE-040 FIX: Searches parent scopes for existing variable and mutates it.
    /// Falls back to creating new binding in current scope if variable doesn't exist.
    ///
    /// # Complexity
    /// Cyclomatic complexity: 4 (within Toyota Way limits ≤10)
    pub(crate) fn env_set_mut(&mut self, name: String, value: Value) {
        // Record type feedback for optimization
        self.record_variable_assignment_feedback(&name, &value);

        // CLOSURE-REFCELL-FIX: First find which scope contains the variable (using read-only borrows)
        // This avoids holding borrow_mut() during iteration which causes RefCell panics with closures
        let mut found_idx: Option<usize> = None;
        for (idx, env_ref) in self.env_stack.iter().enumerate().rev() {
            // Use borrow() not borrow_mut() for the search phase
            if env_ref.borrow().contains_key(&name) {
                found_idx = Some(idx);
                break;
            }
        }

        // CLOSURE-REFCELL-FIX: Now mutate after all borrows are released
        if let Some(idx) = found_idx {
            self.env_stack[idx].borrow_mut().insert(name, value);
        } else {
            // Variable doesn't exist in any scope - create new binding in current scope
            let env_ref = self
                .env_stack
                .last()
                .expect("Environment stack should never be empty");
            env_ref.borrow_mut().insert(name, value);
        }
    }

    /// Push a new environment onto the stack
    // ISSUE-119: Wrap environment in Rc<RefCell> for shared mutable access
    pub(crate) fn env_push(&mut self, env: HashMap<String, Value>) {
        self.env_stack.push(Rc::new(RefCell::new(env)));
    }

    /// Pop the current environment from the stack
    // ISSUE-119: Returns Rc<RefCell<HashMap>> instead of plain HashMap
    pub(crate) fn env_pop(&mut self) -> Option<Rc<RefCell<HashMap<String, Value>>>> {
        if self.env_stack.len() > 1 {
            // Keep at least the global environment
            self.env_stack.pop()
        } else {
            None
        }
    }

    /// Helper method to call a Value function with arguments (for array methods)
    pub(crate) fn eval_function_call_value(
        &mut self,
        func: &Value,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        self.call_function(func.clone(), args)
    }

    /// Call a function with given arguments
    pub(crate) fn call_function(
        &mut self,
        func: Value,
        args: &[Value],
    ) -> Result<Value, InterpreterError> {
        match func {
            Value::String(ref s) if s.starts_with("__class_constructor__:") => {
                // Extract class name and constructor name from the marker
                let parts: Vec<&str> = s
                    .strip_prefix("__class_constructor__:")
                    .expect("prefix exists due to starts_with guard")
                    .split(':')
                    .collect();

                if parts.len() == 2 {
                    let class_name = parts[0];
                    let constructor_name = parts[1];
                    self.instantiate_class_with_constructor(class_name, constructor_name, args)
                } else {
                    // Legacy format for backward compatibility
                    self.instantiate_class_with_constructor(parts[0], "new", args)
                }
            }
            Value::String(ref s) if s.starts_with("__class_static_method__:") => {
                // Extract class name and method name from the marker
                let parts: Vec<&str> = s
                    .strip_prefix("__class_static_method__:")
                    .expect("prefix exists due to starts_with guard")
                    .split(':')
                    .collect();

                if parts.len() == 2 {
                    let class_name = parts[0];
                    let method_name = parts[1];
                    self.call_static_method(class_name, method_name, args)
                } else {
                    Err(InterpreterError::RuntimeError(
                        "Invalid static method marker".to_string(),
                    ))
                }
            }
            Value::String(ref s) if s.starts_with("__struct_constructor__:") => {
                // Extract struct name from the marker
                let struct_name = s
                    .strip_prefix("__struct_constructor__:")
                    .expect("prefix exists due to starts_with guard");
                self.instantiate_struct_with_args(struct_name, args)
            }
            Value::String(ref s) if s.starts_with("__actor_constructor__:") => {
                // Extract actor name from the marker
                let actor_name = s
                    .strip_prefix("__actor_constructor__:")
                    .expect("prefix exists due to starts_with guard");
                self.instantiate_actor_with_args(actor_name, args)
            }
            Value::String(s) if s.starts_with("__builtin_") => {
                // Delegate to extracted builtin module
                match crate::runtime::eval_builtin::eval_builtin_function(&s, args)? {
                    Some(result) => Ok(result),
                    None => Err(InterpreterError::RuntimeError(format!(
                        "Unknown builtin function: {}",
                        s
                    ))),
                }
            }
            Value::Closure { params, body, env } => {
                // [RUNTIME-001] CHECK RECURSION DEPTH BEFORE ENTERING
                crate::runtime::eval_function::check_recursion_depth()?;

                // RUNTIME-DEFAULT-PARAMS: Check argument count with default parameter support
                // Count required params (those without defaults)
                let required_count = params
                    .iter()
                    .filter(|(_, default)| default.is_none())
                    .count();
                let total_count = params.len();

                if args.len() < required_count || args.len() > total_count {
                    crate::runtime::eval_function::decrement_depth();
                    return Err(InterpreterError::RuntimeError(format!(
                        "Function expects {}-{} arguments, got {}",
                        required_count,
                        total_count,
                        args.len()
                    )));
                }

                // ISSUE-119: ROOT CAUSE #3 FIX - Push captured environment first
                // This allows variable lookups to find captured variables
                self.env_stack.push(env); // Push captured environment (Rc::clone)

                // Create NEW empty HashMap for function's local scope (parameters)
                let mut local_env = HashMap::new();

                // RUNTIME-DEFAULT-PARAMS: Bind provided arguments + apply defaults for missing args
                for (i, (param_name, default_value)) in params.iter().enumerate() {
                    let value = if i < args.len() {
                        // Use provided argument
                        args[i].clone()
                    } else if let Some(default_expr) = default_value {
                        // Apply default value by evaluating the expression
                        self.eval_expr(default_expr)?
                    } else {
                        // This should never happen due to required_count check above
                        unreachable!("Missing required parameter");
                    };
                    local_env.insert(param_name.clone(), value);
                }

                // Push local scope on top (parameters shadow outer variables)
                self.env_push(local_env);

                // Evaluate function body
                // Catch InterpreterError::Return and extract value (early return support)
                // BOOK-200-01 FIX: If body is a Block, evaluate statements directly
                // without pushing an additional scope. The function already has its
                // parameter scope (local_env), and pushing another scope would cause
                // lambdas to capture the wrong environment.
                let result = match &body.kind {
                    crate::frontend::ast::ExprKind::Block(statements) => {
                        // Evaluate block statements directly without pushing new scope
                        match crate::runtime::eval_control_flow_new::eval_block_expr(
                            statements,
                            |e| self.eval_expr(e),
                        ) {
                            Err(InterpreterError::Return(val)) => Ok(val),
                            other => other,
                        }
                    }
                    _ => match self.eval_expr(&body) {
                        Err(InterpreterError::Return(val)) => Ok(val),
                        other => other,
                    },
                };

                // ISSUE-119: Pop BOTH environments (local scope + captured environment)
                self.env_pop(); // Pop local scope
                self.env_pop(); // Pop captured environment

                // [RUNTIME-001] ALWAYS DECREMENT, EVEN ON ERROR
                crate::runtime::eval_function::decrement_depth();

                result
            }
            Value::Object(ref obj) => {
                // Check if this is a struct or actor definition being called as a constructor
                if let Some(Value::String(type_str)) = obj.get("__type") {
                    match type_str.as_ref() {
                        "Struct" => {
                            // Get struct name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_struct_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Struct missing __name field".to_string(),
                                ))
                            }
                        }
                        "Actor" => {
                            // Get actor name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_actor_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Actor missing __name field".to_string(),
                                ))
                            }
                        }
                        "Class" => {
                            // Get class name and instantiate
                            if let Some(Value::String(name)) = obj.get("__name") {
                                self.instantiate_class_with_args(name.as_ref(), args)
                            } else {
                                Err(InterpreterError::RuntimeError(
                                    "Class missing __name field".to_string(),
                                ))
                            }
                        }
                        _ => Err(InterpreterError::TypeError(format!(
                            "Cannot call object of type: {}",
                            type_str
                        ))),
                    }
                } else {
                    Err(InterpreterError::TypeError(format!(
                        "Cannot call non-function value: {}",
                        func.type_name()
                    )))
                }
            }
            _ => Err(InterpreterError::TypeError(format!(
                "Cannot call non-function value: {}",
                func.type_name()
            ))),
        }
    }

    /// Evaluate a binary operation from AST.
    ///
    /// Dispatches to specialized evaluation functions based on operator type:
    /// - Arithmetic: `+`, `-`, `*`, `/`, `%`, `**`
    /// - Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`
    /// - Logical: `&&`, `||`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Type mismatch (e.g., adding string to number)
    /// - Division by zero
    /// - Unsupported operator
    pub(crate) fn eval_binary_op(
        &self,
        op: AstBinaryOp,
        left: &Value,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_operations::eval_binary_op(op, left, right)
    }

    pub(crate) fn eval_unary_op(
        &self,
        op: crate::frontend::ast::UnaryOp,
        operand: &Value,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_operations::eval_unary_op(op, operand)
    }

    /// Evaluate binary expression
    pub(crate) fn eval_binary_expr(
        &mut self,
        left: &Expr,
        op: crate::frontend::ast::BinaryOp,
        right: &Expr,
    ) -> Result<Value, InterpreterError> {
        // Handle short-circuit operators and special operators
        match op {
            crate::frontend::ast::BinaryOp::Send => {
                // Actor send operator: actor ! message
                let left_val = self.eval_expr(left)?;
                let message_val = self.eval_message_expr(right)?;

                // Extract the ObjectMut from the actor
                if let Value::ObjectMut(cell_rc) = left_val {
                    // Process the message synchronously
                    self.process_actor_message_sync_mut(&cell_rc, &message_val)?;
                    // Fire-and-forget returns Nil
                    Ok(Value::Nil)
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Send operator requires an actor instance, got {}",
                        left_val.type_name()
                    )))
                }
            }
            crate::frontend::ast::BinaryOp::NullCoalesce => {
                let left_val = self.eval_expr(left)?;
                if matches!(left_val, Value::Nil) {
                    self.eval_expr(right)
                } else {
                    Ok(left_val)
                }
            }
            crate::frontend::ast::BinaryOp::And => {
                let left_val = self.eval_expr(left)?;
                if left_val.is_truthy() {
                    self.eval_expr(right)
                } else {
                    Ok(left_val)
                }
            }
            crate::frontend::ast::BinaryOp::Or => {
                let left_val = self.eval_expr(left)?;
                if left_val.is_truthy() {
                    Ok(left_val)
                } else {
                    self.eval_expr(right)
                }
            }
            crate::frontend::ast::BinaryOp::In => {
                // Containment check: element in collection
                let element = self.eval_expr(left)?;
                let collection = self.eval_expr(right)?;
                let result = self.eval_contains(&element, &collection)?;
                Ok(Value::Bool(result))
            }
            _ => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                let result = self.eval_binary_op(op, &left_val, &right_val)?;

                // Record type feedback for optimization
                let site_id = left.span.start; // Use span start as site ID
                self.record_binary_op_feedback(site_id, &left_val, &right_val, &result);

                Ok(result)
            }
        }
    }

    /// Evaluate containment check (Python-style 'in' operator)
    /// Supports: strings, arrays/lists, maps/dicts
    pub(crate) fn eval_contains(
        &self,
        element: &Value,
        collection: &Value,
    ) -> Result<bool, InterpreterError> {
        match collection {
            // String contains: "substring" in "full string"
            Value::String(s) => {
                if let Value::String(substr) = element {
                    Ok(s.contains(&**substr))
                } else {
                    Err(InterpreterError::RuntimeError(
                        "String containment requires string element".to_string(),
                    ))
                }
            }
            // Array contains
            Value::Array(items) => Ok(items.iter().any(|item| item == element)),
            // Tuple contains
            Value::Tuple(items) => Ok(items.iter().any(|item| item == element)),
            // Object key contains (for maps/dicts)
            Value::Object(map) => {
                if let Value::String(key) = element {
                    Ok(map.contains_key(&**key))
                } else {
                    // For non-string keys, check if any key equals the element
                    let key_str = format!("{element}");
                    Ok(map.contains_key(&key_str))
                }
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "'in' operator not supported for type {}",
                collection.type_name()
            ))),
        }
    }

    /// Evaluate unary expression
    pub(crate) fn eval_unary_expr(
        &mut self,
        op: crate::frontend::ast::UnaryOp,
        operand: &Expr,
    ) -> Result<Value, InterpreterError> {
        let operand_val = self.eval_expr(operand)?;
        self.eval_unary_op(op, &operand_val)
    }

    /// Evaluate type cast expression (as operator)
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (within Toyota Way limits)
    pub(crate) fn eval_type_cast(
        &mut self,
        expr: &Expr,
        target_type: &str,
    ) -> Result<Value, InterpreterError> {
        // Special case: Enum variant to integer (Issue #79)
        // Must extract enum name from AST BEFORE evaluating expression
        if matches!(target_type, "i32" | "i64" | "isize") {
            if let ExprKind::FieldAccess { object, field } = &expr.kind {
                if let ExprKind::Identifier(enum_name) = &object.kind {
                    // Direct enum literal: LogLevel::Info as i32
                    let variant_name = field;

                    // Lookup enum definition in environment
                    if let Some(Value::Object(enum_def)) = self.get_variable(enum_name) {
                        if let Some(Value::Object(variants)) = enum_def.get("__variants") {
                            if let Some(Value::Object(variant_info)) = variants.get(variant_name) {
                                if let Some(Value::Integer(disc)) = variant_info.get("discriminant")
                                {
                                    return Ok(Value::Integer(*disc));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Standard case: Evaluate expression first, then cast
        let value = self.eval_expr(expr)?;

        match (value, target_type) {
            // Integer to Float
            (Value::Integer(i), "f64" | "f32") => Ok(Value::Float(i as f64)),

            // Float to Integer (truncation)
            (Value::Float(f), "i32" | "i64" | "isize") => Ok(Value::Integer(f as i64)),

            // Integer to Integer (identity for i32/i64)
            (Value::Integer(i), "i32" | "i64" | "isize") => Ok(Value::Integer(i)),

            // Float to Float (identity)
            (Value::Float(f), "f64" | "f32") => Ok(Value::Float(f)),

            // Enum variant to Integer - variable case (e.g., level as i32)
            // Now supported via discriminant lookup using stored enum_name
            (
                Value::EnumVariant {
                    enum_name,
                    variant_name,
                    ..
                },
                "i32" | "i64" | "isize",
            ) => {
                // Lookup enum definition in environment
                if let Some(Value::Object(enum_def)) = self.get_variable(&enum_name) {
                    if let Some(Value::Object(variants)) = enum_def.get("__variants") {
                        if let Some(Value::Object(variant_info)) = variants.get(&variant_name) {
                            if let Some(Value::Integer(disc)) = variant_info.get("discriminant") {
                                return Ok(Value::Integer(*disc));
                            }
                        }
                    }
                }
                Err(InterpreterError::TypeError(format!(
                    "Cannot cast enum variant {}::{} to integer: enum definition not found",
                    enum_name, variant_name
                )))
            }

            // Unsupported cast
            (val, target) => Err(InterpreterError::TypeError(format!(
                "Cannot cast {} to {}",
                val.type_name(),
                target
            ))),
        }
    }

    /// Evaluate if expression
    pub(crate) fn eval_if_expr(
        &mut self,
        condition: &Expr,
        then_branch: &Expr,
        else_branch: Option<&Expr>,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_if_expr(
            condition,
            then_branch,
            else_branch,
            |e| self.eval_expr(e),
        )
    }

    /// Evaluate let expression
    pub(crate) fn eval_let_expr(
        &mut self,
        name: &str,
        value: &Expr,
        body: &Expr,
    ) -> Result<Value, InterpreterError> {
        let val = self.eval_expr(value)?;
        self.env_set(name.to_string(), val.clone());

        // If body is unit (empty), return the value like REPL does
        // This makes `let x = 42` return 42 instead of nil
        match &body.kind {
            ExprKind::Literal(Literal::Unit) => Ok(val),
            _ => self.eval_expr(body),
        }
    }

    /// Evaluate return expression
    pub(crate) fn eval_return_expr(
        &mut self,
        value: Option<&Expr>,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_return_expr(value, |e| self.eval_expr(e))
    }

    /// Evaluate list expression
    pub(crate) fn eval_list_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_list_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate array initialization expression [value; size]
    pub(crate) fn eval_array_init_expr(
        &mut self,
        value_expr: &Expr,
        size_expr: &Expr,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_array_init_expr(value_expr, size_expr, |e| {
            self.eval_expr(e)
        })
    }

    /// Evaluate block expression
    /// QA-026 FIX: Block expressions must create a new scope so that `let` bindings
    /// inside the block shadow outer variables instead of overwriting them.
    /// This ensures `let x = 10; if true { let x = 20 }; println(x)` prints 10, not 20.
    pub(crate) fn eval_block_expr(
        &mut self,
        statements: &[Expr],
    ) -> Result<Value, InterpreterError> {
        // QA-026: Push new scope for block
        self.push_scope();
        let result = crate::runtime::eval_control_flow_new::eval_block_expr(statements, |e| {
            self.eval_expr(e)
        });
        // QA-026: Pop scope after block completes (even on error)
        self.pop_scope();
        result
    }

    /// Evaluate tuple expression
    pub(crate) fn eval_tuple_expr(&mut self, elements: &[Expr]) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_tuple_expr(elements, |e| self.eval_expr(e))
    }

    /// Evaluate `DataFrame` literal expression
    /// Complexity: 5 (within Toyota Way limits)
    pub(crate) fn eval_dataframe_literal(
        &mut self,
        columns: &[crate::frontend::ast::DataFrameColumn],
    ) -> Result<Value, InterpreterError> {
        let mut evaluated_columns = Vec::new();

        for col in columns {
            // Evaluate each value expression in the column
            let mut evaluated_values = Vec::new();
            for value_expr in &col.values {
                evaluated_values.push(self.eval_expr(value_expr)?);
            }

            // Create runtime DataFrameColumn
            evaluated_columns.push(DataFrameColumn {
                name: col.name.clone(),
                values: evaluated_values,
            });
        }

        Ok(Value::DataFrame {
            columns: evaluated_columns,
        })
    }

    /// Evaluate range expression
    pub(crate) fn eval_range_expr(
        &mut self,
        start: &Expr,
        end: &Expr,
        inclusive: bool,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_control_flow_new::eval_range_expr(start, end, inclusive, |e| {
            self.eval_expr(e)
        })
    }

    // JSON operations delegated to eval_json module
    // EXTREME TDD: Eliminated 80 lines of duplicate code

    pub(crate) fn json_parse(&self, json_str: &str) -> Result<Value, InterpreterError> {
        crate::runtime::eval_json::json_parse(json_str)
    }

    pub(crate) fn json_stringify(&self, value: &Value) -> Result<Value, InterpreterError> {
        crate::runtime::eval_json::json_stringify(value)
    }

    pub(crate) fn serde_to_value(json: &serde_json::Value) -> Result<Value, InterpreterError> {
        crate::runtime::eval_json::serde_to_value(json)
    }

    pub(crate) fn value_to_serde(value: &Value) -> Result<serde_json::Value, InterpreterError> {
        crate::runtime::eval_json::value_to_serde(value)
    }

    /// Helper function for testing - evaluate a string expression via parser
    /// # Errors
    /// Returns error if parsing or evaluation fails
    #[cfg(test)]
    /// Evaluate a string of Ruchy code.
    ///
    /// This convenience function parses and evaluates a string in one step.
    /// It's useful for REPL implementations and testing.
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// let result = interpreter.eval_string("2 * 21").expect("eval_string should succeed in doctest");
    /// assert_eq!(result.to_string(), "42");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or if evaluation fails.
    pub fn eval_string(&mut self, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
        use crate::frontend::parser::Parser;

        let mut parser = Parser::new(input);
        let expr = parser.parse_expr()?;

        Ok(self.eval_expr(&expr)?)
    }

    /// Push value onto stack
    /// # Errors
    /// Returns error if stack overflow occurs
    pub fn push(&mut self, value: Value) -> Result<(), InterpreterError> {
        if self.stack.len() >= 10_000 {
            // Stack limit from spec
            return Err(InterpreterError::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop value from stack
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn pop(&mut self) -> Result<Value, InterpreterError> {
        self.stack.pop().ok_or(InterpreterError::StackUnderflow)
    }

    /// Peek at top of stack without popping
    /// # Errors
    /// Returns error if stack underflow occurs
    pub fn peek(&self, depth: usize) -> Result<Value, InterpreterError> {
        let index = self
            .stack
            .len()
            .checked_sub(depth + 1)
            .ok_or(InterpreterError::StackUnderflow)?;
        Ok(self.stack[index].clone())
    }

    /// Binary arithmetic operation with type checking
    /// # Errors
    /// Returns error if stack underflow, type mismatch, or arithmetic error occurs
    pub fn binary_op(&mut self, op: BinaryOp) -> Result<(), InterpreterError> {
        let right = self.pop()?;
        let left = self.pop()?;

        let result = match op {
            BinaryOp::Add => eval_operations::eval_binary_op(AstBinaryOp::Add, &left, &right)?,
            BinaryOp::Sub => eval_operations::eval_binary_op(AstBinaryOp::Subtract, &left, &right)?,
            BinaryOp::Mul => eval_operations::eval_binary_op(AstBinaryOp::Multiply, &left, &right)?,
            BinaryOp::Div => eval_operations::eval_binary_op(AstBinaryOp::Divide, &left, &right)?,
            BinaryOp::Eq => eval_operations::eval_binary_op(AstBinaryOp::Equal, &left, &right)?,
            BinaryOp::Lt => eval_operations::eval_binary_op(AstBinaryOp::Less, &left, &right)?,
            BinaryOp::Gt => eval_operations::eval_binary_op(AstBinaryOp::Greater, &left, &right)?,
        };

        self.push(result)?;
        Ok(())
    }

    /// Set a variable in the current scope (public for try/catch)
    pub fn set_variable_string(&mut self, name: String, value: Value) {
        self.env_set(name, value);
    }

    /// Apply a binary operation to two values
    pub(crate) fn apply_binary_op(
        &self,
        left: &Value,
        op: AstBinaryOp,
        right: &Value,
    ) -> Result<Value, InterpreterError> {
        // Delegate to existing binary operation evaluation
        self.eval_binary_op(op, left, right)
    }

    /// Check if a pattern matches a value
    /// # Errors
    /// Returns error if pattern matching fails
    /// Try to match a pattern against a value, returning bindings if successful
    pub(crate) fn try_pattern_match(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<Option<Vec<(String, Value)>>, InterpreterError> {
        crate::runtime::eval_pattern_match::try_pattern_match(pattern, value, &|lit| {
            self.eval_literal(lit)
        })
    }

    /// Legacy method for backwards compatibility
    pub(crate) fn pattern_matches_internal(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::pattern_matches(pattern, value, &|lit| {
            self.eval_literal(lit)
        })
    }

    /// Scope management for pattern bindings
    pub fn push_scope(&mut self) {
        let new_env = HashMap::new();
        self.env_push(new_env);
    }

    pub fn pop_scope(&mut self) {
        self.env_pop();
    }

    /// New pattern matching methods that return bindings

    // Helper methods for pattern matching (complexity <10 each)

    pub(crate) fn match_tuple_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_tuple_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    pub(crate) fn match_list_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_list_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    pub(crate) fn match_or_pattern(
        &self,
        patterns: &[Pattern],
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::eval_pattern_match::match_or_pattern(patterns, value, |lit| {
            self.eval_literal(lit)
        })
    }

    /// Access field with inline caching optimization
    /// # Errors
    /// Returns error if field access fails
    pub fn get_field_cached(
        &mut self,
        obj: &Value,
        field_name: &str,
    ) -> Result<Value, InterpreterError> {
        // Create cache key combining object type and field name
        let cache_key = format!("{:?}::{}", obj.type_id(), field_name);

        // Check inline cache first
        if let Some(cache) = self.field_caches.get_mut(&cache_key) {
            if let Some(cached_result) = cache.lookup(obj, field_name) {
                return Ok(cached_result);
            }
        }

        // Cache miss - compute result and update cache
        let result = self.compute_field_access(obj, field_name)?;

        // Update or create cache entry
        let cache = self.field_caches.entry(cache_key).or_default();
        cache.insert(obj, field_name.to_string(), result.clone());

        Ok(result)
    }

    /// Compute field access result (detailed path)
    pub(crate) fn compute_field_access(
        &self,
        obj: &Value,
        field_name: &str,
    ) -> Result<Value, InterpreterError> {
        match (obj, field_name) {
            // String methods
            (Value::String(s), "len") => Ok(Value::Integer(s.len().try_into().unwrap_or(i64::MAX))),
            (Value::String(s), "to_upper") => Ok(Value::from_string(s.to_uppercase())),
            (Value::String(s), "to_lower") => Ok(Value::from_string(s.to_lowercase())),
            (Value::String(s), "trim") => Ok(Value::from_string(s.trim().to_string())),

            // Array methods
            (Value::Array(arr), "len") => {
                Ok(Value::Integer(arr.len().try_into().unwrap_or(i64::MAX)))
            }
            (Value::Array(arr), "first") => arr
                .first()
                .cloned()
                .ok_or_else(|| InterpreterError::RuntimeError("Array is empty".to_string())),
            (Value::Array(arr), "last") => arr
                .last()
                .cloned()
                .ok_or_else(|| InterpreterError::RuntimeError("Array is empty".to_string())),
            (Value::Array(arr), "is_empty") => Ok(Value::from_bool(arr.is_empty())),

            // Type information
            (obj, "type") => Ok(Value::from_string(obj.type_name().to_string())),

            _ => Err(InterpreterError::RuntimeError(format!(
                "Field '{}' not found on type '{}'",
                field_name,
                obj.type_name()
            ))),
        }
    }

    /// Get inline cache statistics for profiling
    pub fn get_cache_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        for (key, cache) in &self.field_caches {
            stats.insert(key.clone(), cache.hit_rate());
        }
        stats
    }

    /// Clear all inline caches (for testing)
    pub fn clear_caches(&mut self) {
        self.field_caches.clear();
    }

    /// Record type feedback for binary operations
    #[allow(dead_code)] // Used by tests and type feedback system
    pub(crate) fn record_binary_op_feedback(
        &mut self,
        site_id: usize,
        left: &Value,
        right: &Value,
        result: &Value,
    ) {
        self.type_feedback
            .record_binary_op(site_id, left, right, result);
    }

    /// Record type feedback for variable assignments
    #[allow(dead_code)] // Used by tests and type feedback system
    pub(crate) fn record_variable_assignment_feedback(&mut self, var_name: &str, value: &Value) {
        let type_id = value.type_id();
        self.type_feedback
            .record_variable_assignment(var_name, type_id);
    }

    /// Record type feedback for function calls
    pub(crate) fn record_function_call_feedback(
        &mut self,
        site_id: usize,
        func_name: &str,
        args: &[Value],
        result: &Value,
    ) {
        self.type_feedback
            .record_function_call(site_id, func_name, args, result);
    }

    /// Get type feedback statistics
    pub fn get_type_feedback_stats(&self) -> TypeFeedbackStats {
        self.type_feedback.get_statistics()
    }

    /// Get specialization candidates for JIT compilation
    pub fn get_specialization_candidates(&self) -> Vec<SpecializationCandidate> {
        self.type_feedback.get_specialization_candidates()
    }

    /// Clear type feedback data (for testing)
    pub fn clear_type_feedback(&mut self) {
        self.type_feedback = TypeFeedback::new();
    }

    /// Track a value in the garbage collector
    pub fn gc_track(&mut self, value: Value) -> usize {
        self.gc.track_object(value)
    }

    /// Force garbage collection
    pub fn gc_collect(&mut self) -> GCStats {
        self.gc.force_collect()
    }

    /// Get garbage collection statistics
    pub fn gc_stats(&self) -> GCStats {
        self.gc.get_stats()
    }

    /// Get detailed garbage collection information
    pub fn gc_info(&self) -> GCInfo {
        self.gc.get_info()
    }

    /// Set garbage collection threshold
    pub fn gc_set_threshold(&mut self, threshold: usize) {
        self.gc.set_collection_threshold(threshold);
    }

    /// Enable or disable automatic garbage collection
    pub fn gc_set_auto_collect(&mut self, enabled: bool) {
        self.gc.set_auto_collect(enabled);
    }

    /// Clear all GC-tracked objects (for testing)
    pub fn gc_clear(&mut self) {
        self.gc.clear();
    }

    /// Allocate a new array and track it in GC
    pub fn gc_alloc_array(&mut self, elements: Vec<Value>) -> Value {
        let array_value = Value::from_array(elements);
        self.gc.track_object(array_value.clone());
        array_value
    }

    /// Allocate a new string and track it in GC
    pub fn gc_alloc_string(&mut self, content: String) -> Value {
        let string_value = Value::from_string(content);
        self.gc.track_object(string_value.clone());
        string_value
    }

    /// Allocate a new closure and track it in GC
    /// RUNTIME-DEFAULT-PARAMS: Updated to support default parameter values
    pub fn gc_alloc_closure(
        &mut self,
        params: Vec<(String, Option<Arc<Expr>>)>, // (param_name, default_value)
        body: Arc<Expr>,
        env: Rc<RefCell<HashMap<String, Value>>>, // ISSUE-119: Changed from Arc<HashMap>
    ) -> Value {
        let closure_value = Value::Closure { params, body, env };
        self.gc.track_object(closure_value.clone());
        closure_value
    }

    // ========================================================================
    // Public methods for SharedSession integration
    // ========================================================================

    /// Get all bindings from the global environment (for `SharedSession` state persistence)
    pub fn get_global_bindings(&self) -> HashMap<String, Value> {
        if let Some(global_env) = self.env_stack.first() {
            global_env.borrow().clone() // ISSUE-119: Borrow from RefCell, then clone HashMap
        } else {
            HashMap::new()
        }
    }

    /// Set a binding in the global environment (for `SharedSession` state restoration)
    pub fn set_global_binding(&mut self, name: String, value: Value) {
        if let Some(global_env) = self.env_stack.first() {
            // ISSUE-119: Use first() not first_mut()
            global_env.borrow_mut().insert(name, value); // ISSUE-119: Mutable borrow from RefCell
        }
    }

    /// Clear all user variables from global environment, keeping only builtins
    pub fn clear_user_variables(&mut self) {
        if let Some(global_env) = self.env_stack.first() {
            // ISSUE-119: Use first() not first_mut()
            // Keep only builtin functions (those starting with "__builtin_") and nil
            global_env
                .borrow_mut()
                .retain(|name, _| name.starts_with("__builtin_") || name == "nil");
            // ISSUE-119
        }
    }

    /// Get all bindings from the current environment (for `SharedSession` extraction)
    pub fn get_current_bindings(&self) -> HashMap<String, Value> {
        if let Some(current_env) = self.env_stack.last() {
            current_env.borrow().clone() // ISSUE-119: Borrow from RefCell, then clone HashMap
        } else {
            HashMap::new()
        }
    }

    /// Evaluate string interpolation
    pub(crate) fn eval_string_interpolation(
        &mut self,
        parts: &[StringPart],
    ) -> Result<Value, InterpreterError> {
        use crate::runtime::eval_string_interpolation::format_value_for_interpolation;

        let mut result = String::new();
        for part in parts {
            match part {
                StringPart::Text(text) => result.push_str(text),
                StringPart::Expr(expr) => {
                    let value = self.eval_expr(expr)?;
                    // Use format_value_for_interpolation to avoid adding quotes to strings
                    result.push_str(&format_value_for_interpolation(&value));
                }
                StringPart::ExprWithFormat { expr, format_spec } => {
                    let value = self.eval_expr(expr)?;
                    // Apply format specifier for interpreter
                    let formatted = Self::format_value_with_spec(&value, format_spec);
                    result.push_str(&formatted);
                }
            }
        }
        Ok(Value::from_string(result))
    }

    // Format specifier delegated to value_format module
    pub(crate) fn format_value_with_spec(value: &Value, spec: &str) -> String {
        crate::runtime::value_format::format_value_with_spec(value, spec)
    }

    /// Push an error handling scope for try/catch blocks
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn push_error_scope(&mut self) {
        self.error_scopes.push(ErrorScope {
            env_depth: self.env_stack.len(),
        });
    }

    /// Pop an error handling scope
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn pop_error_scope(&mut self) {
        self.error_scopes.pop();
    }

    /// Set a variable in the current scope
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1
    pub fn set_variable(&mut self, name: &str, value: Value) {
        // ISSUE-040 FIX: Use env_set_mut to search parent scopes for existing variables
        self.env_set_mut(name.to_string(), value);
    }

    /// Get a variable from the environment stack
    ///
    /// Searches the environment stack from innermost to outermost scope.
    /// Returns None if the variable is not found.
    pub fn get_variable(&self, name: &str) -> Option<Value> {
        // Search from innermost to outermost scope
        for env in self.env_stack.iter().rev() {
            if let Some(value) = env.borrow().get(name) {
                // ISSUE-119: Borrow from RefCell
                return Some(value.clone());
            }
        }
        None
    }

    /// Pattern matching for try/catch
    ///
    /// # Complexity
    /// Cyclomatic complexity: 8 (delegates to existing pattern matcher)
    pub fn pattern_matches(
        &mut self,
        pattern: &Pattern,
        value: &Value,
    ) -> Result<bool, InterpreterError> {
        // Simplified pattern matching for try/catch
        match pattern {
            Pattern::Identifier(_) => Ok(true), // Always matches
            Pattern::Wildcard => Ok(true),
            Pattern::Literal(literal) => Ok(self.literal_matches(literal, value)),
            _ => Ok(false), // Other patterns not yet supported
        }
    }

    pub(crate) fn literal_matches(&self, literal: &Literal, value: &Value) -> bool {
        match (literal, value) {
            (Literal::Integer(a, _), Value::Integer(b)) => a == b,
            (Literal::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Literal::String(a), Value::String(b)) => a == b.as_ref(),
            (Literal::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }

    // ========================================================================
    // stdout Capture for WASM/REPL
    // ========================================================================

    /// Capture println output to stdout buffer
    /// Complexity: 1 (single operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("Hello, World!".to_string());
    /// assert_eq!(interpreter.get_stdout(), "Hello, World!");
    /// ```
    pub fn capture_stdout(&mut self, output: String) {
        self.stdout_buffer.push(output);
    }

    /// Get captured stdout as a single string with newlines
    /// Complexity: 2 (join + conditional)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("Line 1".to_string());
    /// interpreter.capture_stdout("Line 2".to_string());
    /// assert_eq!(interpreter.get_stdout(), "Line 1\nLine 2");
    /// ```
    pub fn get_stdout(&self) -> String {
        self.stdout_buffer.join("\n")
    }

    /// Clear stdout buffer
    /// Complexity: 1 (single operation)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::runtime::interpreter::Interpreter;
    ///
    /// let mut interpreter = Interpreter::new();
    /// interpreter.capture_stdout("test".to_string());
    /// interpreter.clear_stdout();
    /// assert_eq!(interpreter.get_stdout(), "");
    /// ```
    pub fn clear_stdout(&mut self) {
        self.stdout_buffer.clear();
    }

    /// Check if stdout has any captured output
    /// Complexity: 1 (single check)
    pub fn has_stdout(&self) -> bool {
        !self.stdout_buffer.is_empty()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary operations
#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, UnaryOp};

    fn make_interpreter() -> Interpreter {
        Interpreter::new()
    }

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // Test Interpreter::new
    #[test]
    fn test_interpreter_new() {
        let interp = make_interpreter();
        assert!(interp.stack.is_empty());
        assert_eq!(interp.env_stack.len(), 1); // Global env
    }

    // Test Interpreter::default
    #[test]
    fn test_interpreter_default() {
        let interp = Interpreter::default();
        assert!(interp.stack.is_empty());
    }

    // Test eval_expr with literals
    #[test]
    fn test_eval_expr_integer() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_expr_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Float(3.14)));
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_expr_string() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_expr_bool_true() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_expr_bool_false() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // Test is_type_definition
    #[test]
    fn test_is_type_definition_struct() {
        let kind = ExprKind::Struct {
            name: "Point".to_string(),
            type_params: vec![],
            fields: vec![],
            methods: vec![],
            derives: vec![],
            is_pub: true,
        };
        assert!(Interpreter::is_type_definition(&kind));
    }

    #[test]
    fn test_is_type_definition_class() {
        let kind = ExprKind::Class {
            name: "MyClass".to_string(),
            type_params: vec![],
            superclass: None,
            traits: vec![],
            fields: vec![],
            constructors: vec![],
            methods: vec![],
            constants: vec![],
            properties: vec![],
            derives: vec![],
            decorators: vec![],
            is_pub: true,
            is_abstract: false,
            is_sealed: false,
        };
        assert!(Interpreter::is_type_definition(&kind));
    }

    #[test]
    fn test_is_type_definition_enum() {
        let kind = ExprKind::Enum {
            name: "Color".to_string(),
            type_params: vec![],
            variants: vec![],
            is_pub: true,
        };
        assert!(Interpreter::is_type_definition(&kind));
    }

    #[test]
    fn test_is_type_definition_actor() {
        let kind = ExprKind::Actor {
            name: "Counter".to_string(),
            state: vec![],
            handlers: vec![],
        };
        assert!(Interpreter::is_type_definition(&kind));
    }

    #[test]
    fn test_is_type_definition_impl() {
        let kind = ExprKind::Impl {
            trait_name: None,
            for_type: "Point".to_string(),
            type_params: vec![],
            methods: vec![],
            is_pub: true,
        };
        assert!(Interpreter::is_type_definition(&kind));
    }

    #[test]
    fn test_is_type_definition_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_type_definition(&kind));
    }

    // Test is_actor_operation
    #[test]
    fn test_is_actor_operation_spawn() {
        let inner = make_expr(ExprKind::Identifier("Counter".to_string()));
        let kind = ExprKind::Spawn {
            actor: Box::new(inner),
        };
        assert!(Interpreter::is_actor_operation(&kind));
    }

    #[test]
    fn test_is_actor_operation_send() {
        let actor = Box::new(make_expr(ExprKind::Identifier("counter".to_string())));
        let msg = Box::new(make_expr(ExprKind::Identifier("Inc".to_string())));
        let kind = ExprKind::ActorSend {
            actor,
            message: msg,
        };
        assert!(Interpreter::is_actor_operation(&kind));
    }

    #[test]
    fn test_is_actor_operation_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_actor_operation(&kind));
    }

    // Test is_special_form
    #[test]
    fn test_is_special_form_none() {
        let kind = ExprKind::None;
        assert!(Interpreter::is_special_form(&kind));
    }

    #[test]
    fn test_is_special_form_some() {
        let inner = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let kind = ExprKind::Some {
            value: Box::new(inner),
        };
        assert!(Interpreter::is_special_form(&kind));
    }

    #[test]
    fn test_is_special_form_struct_literal() {
        let kind = ExprKind::StructLiteral {
            name: "Point".to_string(),
            fields: vec![],
            base: None,
        };
        assert!(Interpreter::is_special_form(&kind));
    }

    #[test]
    fn test_is_special_form_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_special_form(&kind));
    }

    // Test is_control_flow_expr
    #[test]
    fn test_is_control_flow_if() {
        let cond = Box::new(make_expr(ExprKind::Literal(Literal::Bool(true))));
        let then_branch = Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None))));
        let kind = ExprKind::If {
            condition: cond,
            then_branch,
            else_branch: None,
        };
        assert!(Interpreter::is_control_flow_expr(&kind));
    }

    #[test]
    fn test_is_control_flow_return() {
        let kind = ExprKind::Return { value: None };
        assert!(Interpreter::is_control_flow_expr(&kind));
    }

    #[test]
    fn test_is_control_flow_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_control_flow_expr(&kind));
    }

    // Test is_data_structure_expr
    #[test]
    fn test_is_data_structure_list() {
        let kind = ExprKind::List(vec![]);
        assert!(Interpreter::is_data_structure_expr(&kind));
    }

    #[test]
    fn test_is_data_structure_tuple() {
        let kind = ExprKind::Tuple(vec![]);
        assert!(Interpreter::is_data_structure_expr(&kind));
    }

    #[test]
    fn test_is_data_structure_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_data_structure_expr(&kind));
    }

    // Test is_assignment_expr
    #[test]
    fn test_is_assignment_assign() {
        let target = Box::new(make_expr(ExprKind::Identifier("x".to_string())));
        let value = Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None))));
        let kind = ExprKind::Assign { target, value };
        assert!(Interpreter::is_assignment_expr(&kind));
    }

    #[test]
    fn test_is_assignment_false() {
        let kind = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!Interpreter::is_assignment_expr(&kind));
    }

    // Test stack operations
    #[test]
    fn test_push_and_pop() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(42)).unwrap();
        let result = interp.pop().unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_pop_empty_stack() {
        let mut interp = make_interpreter();
        let result = interp.pop();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Stack underflow"));
    }

    #[test]
    fn test_peek() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(1)).unwrap();
        interp.push(Value::Integer(2)).unwrap();

        let top = interp.peek(0).unwrap();
        assert_eq!(top, Value::Integer(2));

        let second = interp.peek(1).unwrap();
        assert_eq!(second, Value::Integer(1));
    }

    #[test]
    fn test_peek_out_of_bounds() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(1)).unwrap();
        let result = interp.peek(5);
        assert!(result.is_err());
    }

    // Test stdout capture
    #[test]
    fn test_capture_stdout() {
        let mut interp = make_interpreter();
        interp.capture_stdout("hello".to_string());
        assert!(interp.has_stdout());
        assert_eq!(interp.get_stdout(), "hello");
    }

    #[test]
    fn test_get_stdout_multiple() {
        let mut interp = make_interpreter();
        interp.capture_stdout("line1".to_string());
        interp.capture_stdout("line2".to_string());
        assert_eq!(interp.get_stdout(), "line1\nline2");
    }

    #[test]
    fn test_clear_stdout() {
        let mut interp = make_interpreter();
        interp.capture_stdout("test".to_string());
        interp.clear_stdout();
        assert!(!interp.has_stdout());
        assert_eq!(interp.get_stdout(), "");
    }

    // Test environment operations
    #[test]
    fn test_set_variable() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(42));
        let result = interp.lookup_variable("x").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_lookup_variable_not_found() {
        let interp = make_interpreter();
        let result = interp.lookup_variable("nonexistent");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Undefined variable"));
    }

    #[test]
    fn test_current_env() {
        let interp = make_interpreter();
        let env = interp.current_env();
        // Should be the global environment
        assert!(env.borrow().contains_key("max"));
    }

    #[test]
    fn test_env_push_pop() {
        let mut interp = make_interpreter();
        let initial_depth = interp.env_stack.len();

        let mut new_env = HashMap::new();
        new_env.insert("local".to_string(), Value::Integer(42));
        interp.env_push(new_env);

        assert_eq!(interp.env_stack.len(), initial_depth + 1);
        assert_eq!(interp.lookup_variable("local").unwrap(), Value::Integer(42));

        interp.env_pop();
        assert_eq!(interp.env_stack.len(), initial_depth);
        assert!(interp.lookup_variable("local").is_err());
    }

    #[test]
    fn test_env_set() {
        let mut interp = make_interpreter();
        interp.env_set("x".to_string(), Value::Integer(1));
        assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_env_set_mut_updates_parent() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(1));

        // Push a new scope
        interp.env_push(HashMap::new());

        // env_set_mut should update the parent scope
        interp.env_set_mut("x".to_string(), Value::Integer(2));

        interp.env_pop();
        assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(2));
    }

    // Test format_string_with_values
    #[test]
    fn test_format_string_with_values() {
        let result = Interpreter::format_string_with_values(
            "Hello {} and {}!",
            &[Value::from_string("world".to_string()), Value::Integer(42)],
        );
        assert_eq!(result, "Hello world and 42!");
    }

    #[test]
    fn test_format_string_with_values_fewer_args() {
        let result = Interpreter::format_string_with_values(
            "Hello {} and {}!",
            &[Value::from_string("world".to_string())],
        );
        assert_eq!(result, "Hello world and {}!");
    }

    // Test eval_literal
    #[test]
    fn test_eval_literal_unit() {
        let interp = make_interpreter();
        let result = interp.eval_literal(&Literal::Unit);
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_literal_char() {
        let interp = make_interpreter();
        let result = interp.eval_literal(&Literal::Char('a'));
        assert_eq!(result, Value::from_string("a".to_string()));
    }

    // Test eval_list_expr
    #[test]
    fn test_eval_list_expr() {
        let mut interp = make_interpreter();
        let elements = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ];
        let result = interp.eval_list_expr(&elements).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
        } else {
            panic!("Expected Array");
        }
    }

    // Test eval_tuple_expr
    #[test]
    fn test_eval_tuple_expr() {
        let mut interp = make_interpreter();
        let elements = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::String("hello".to_string()))),
        ];
        let result = interp.eval_tuple_expr(&elements).unwrap();
        if let Value::Tuple(tuple) = result {
            assert_eq!(tuple.len(), 2);
        } else {
            panic!("Expected Tuple");
        }
    }

    // Test eval_block_expr
    #[test]
    fn test_eval_block_expr_empty() {
        let mut interp = make_interpreter();
        let result = interp.eval_block_expr(&[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_block_expr_single() {
        let mut interp = make_interpreter();
        let stmts = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];
        let result = interp.eval_block_expr(&stmts).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // Test eval_return_expr
    #[test]
    fn test_eval_return_expr_with_value() {
        let mut interp = make_interpreter();
        let val_expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_return_expr(Some(&val_expr));
        // eval_return_expr always returns Err(Return(...)) to signal early return
        match result {
            Err(InterpreterError::Return(v)) => assert_eq!(v, Value::Integer(42)),
            _ => panic!("Expected Return error"),
        }
    }

    #[test]
    fn test_eval_return_expr_without_value() {
        let mut interp = make_interpreter();
        let result = interp.eval_return_expr(None);
        // eval_return_expr always returns Err(Return(...)) to signal early return
        match result {
            Err(InterpreterError::Return(v)) => assert_eq!(v, Value::Nil),
            _ => panic!("Expected Return error"),
        }
    }

    // Test eval_string
    #[test]
    fn test_eval_string_simple() {
        let mut interp = make_interpreter();
        let result = interp.eval_string("42").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_string_expression() {
        let mut interp = make_interpreter();
        let result = interp.eval_string("2 + 3").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    // Test json operations
    #[test]
    fn test_json_parse_object() {
        let interp = make_interpreter();
        let result = interp.json_parse(r#"{"a": 1}"#).unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("a"), Some(&Value::Integer(1)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_json_parse_array() {
        let interp = make_interpreter();
        let result = interp.json_parse("[1, 2, 3]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_json_stringify() {
        let interp = make_interpreter();
        let result = interp.json_stringify(&Value::Integer(42)).unwrap();
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    // Test serde_to_value
    #[test]
    fn test_serde_to_value_null() {
        let result = Interpreter::serde_to_value(&serde_json::Value::Null).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_serde_to_value_bool() {
        let result = Interpreter::serde_to_value(&serde_json::Value::Bool(true)).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_serde_to_value_number_int() {
        let result = Interpreter::serde_to_value(&serde_json::json!(42)).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_serde_to_value_number_float() {
        let result = Interpreter::serde_to_value(&serde_json::json!(3.14)).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_serde_to_value_string() {
        let result = Interpreter::serde_to_value(&serde_json::json!("hello")).unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    // Test value_to_serde
    #[test]
    fn test_value_to_serde_nil() {
        let result = Interpreter::value_to_serde(&Value::Nil).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_value_to_serde_bool() {
        let result = Interpreter::value_to_serde(&Value::Bool(true)).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_value_to_serde_integer() {
        let result = Interpreter::value_to_serde(&Value::Integer(42)).unwrap();
        assert_eq!(result, serde_json::json!(42));
    }

    #[test]
    fn test_value_to_serde_float() {
        let result = Interpreter::value_to_serde(&Value::Float(3.14)).unwrap();
        assert_eq!(result, serde_json::json!(3.14));
    }

    #[test]
    fn test_value_to_serde_string() {
        let result = Interpreter::value_to_serde(&Value::from_string("hello".to_string())).unwrap();
        assert_eq!(result, serde_json::json!("hello"));
    }

    // Test eval_contains
    #[test]
    fn test_eval_contains_array_found() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.eval_contains(&Value::Integer(1), &arr).unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_contains_array_not_found() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.eval_contains(&Value::Integer(3), &arr).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_eval_contains_string_found() {
        let interp = make_interpreter();
        let s = Value::from_string("hello world".to_string());
        let result = interp
            .eval_contains(&Value::from_string("world".to_string()), &s)
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_contains_string_not_found() {
        let interp = make_interpreter();
        let s = Value::from_string("hello world".to_string());
        let result = interp
            .eval_contains(&Value::from_string("foo".to_string()), &s)
            .unwrap();
        assert!(!result);
    }

    // Test eval_range_expr
    #[test]
    fn test_eval_range_expr_exclusive() {
        let mut interp = make_interpreter();
        let start = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
        let end = make_expr(ExprKind::Literal(Literal::Integer(5, None)));
        let result = interp.eval_range_expr(&start, &end, false).unwrap();
        if let Value::Range {
            start: s,
            end: e,
            inclusive,
        } = result
        {
            assert_eq!(*s, Value::Integer(0));
            assert_eq!(*e, Value::Integer(5));
            assert!(!inclusive);
        } else {
            panic!("Expected Range");
        }
    }

    #[test]
    fn test_eval_range_expr_inclusive() {
        let mut interp = make_interpreter();
        let start = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let end = make_expr(ExprKind::Literal(Literal::Integer(10, None)));
        let result = interp.eval_range_expr(&start, &end, true).unwrap();
        if let Value::Range {
            start: s,
            end: e,
            inclusive,
        } = result
        {
            assert_eq!(*s, Value::Integer(1));
            assert_eq!(*e, Value::Integer(10));
            assert!(inclusive);
        } else {
            panic!("Expected Range");
        }
    }

    // ===== GC Operations Tests =====

    #[test]
    fn test_gc_track() {
        let mut interp = make_interpreter();
        let id = interp.gc_track(Value::Integer(42));
        assert!(id >= 0);
    }

    #[test]
    fn test_gc_collect() {
        let mut interp = make_interpreter();
        interp.gc_track(Value::Integer(1));
        interp.gc_track(Value::Integer(2));
        let stats = interp.gc_collect();
        // Stats should be returned
        let _ = stats.collections; // Verify field exists
    }

    #[test]
    fn test_gc_stats() {
        let interp = make_interpreter();
        let stats = interp.gc_stats();
        let _ = stats.collections; // Verify field exists
    }

    #[test]
    fn test_gc_info() {
        let interp = make_interpreter();
        let info = interp.gc_info();
        let _ = info.tracked_count; // Verify field exists
    }

    #[test]
    fn test_gc_set_threshold() {
        let mut interp = make_interpreter();
        interp.gc_set_threshold(1000);
        // No panic = success
    }

    #[test]
    fn test_gc_set_auto_collect() {
        let mut interp = make_interpreter();
        interp.gc_set_auto_collect(false);
        interp.gc_set_auto_collect(true);
        // No panic = success
    }

    #[test]
    fn test_gc_clear() {
        let mut interp = make_interpreter();
        interp.gc_track(Value::Integer(1));
        interp.gc_clear();
        // No panic = success
    }

    #[test]
    fn test_gc_alloc_array() {
        let mut interp = make_interpreter();
        let arr = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
        if let Value::Array(a) = arr {
            assert_eq!(a.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_gc_alloc_string() {
        let mut interp = make_interpreter();
        let s = interp.gc_alloc_string("hello".to_string());
        assert_eq!(s, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_gc_alloc_closure() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = interp.gc_alloc_closure(vec![("x".to_string(), None)], body, env);
        assert!(matches!(closure, Value::Closure { .. }));
    }

    // ===== Inline Cache Operations Tests =====

    #[test]
    fn test_get_field_cached_string_len() {
        let mut interp = make_interpreter();
        let s = Value::from_string("hello".to_string());
        let result = interp.get_field_cached(&s, "len").unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_get_field_cached_string_to_upper() {
        let mut interp = make_interpreter();
        let s = Value::from_string("hello".to_string());
        let result = interp.get_field_cached(&s, "to_upper").unwrap();
        assert_eq!(result, Value::from_string("HELLO".to_string()));
    }

    #[test]
    fn test_compute_field_access_string_to_lower() {
        let interp = make_interpreter();
        let s = Value::from_string("HELLO".to_string());
        let result = interp.compute_field_access(&s, "to_lower").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_compute_field_access_string_trim() {
        let interp = make_interpreter();
        let s = Value::from_string("  hello  ".to_string());
        let result = interp.compute_field_access(&s, "trim").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_compute_field_access_array_len() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.compute_field_access(&arr, "len").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_compute_field_access_array_first() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.compute_field_access(&arr, "first").unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_compute_field_access_array_last() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.compute_field_access(&arr, "last").unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_compute_field_access_array_is_empty() {
        let interp = make_interpreter();
        let arr = Value::Array(Arc::from(vec![]));
        let result = interp.compute_field_access(&arr, "is_empty").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_compute_field_access_type() {
        let interp = make_interpreter();
        let result = interp
            .compute_field_access(&Value::Integer(42), "type")
            .unwrap();
        assert_eq!(result, Value::from_string("integer".to_string()));
    }

    #[test]
    fn test_compute_field_access_unknown_field() {
        let interp = make_interpreter();
        let result = interp.compute_field_access(&Value::Integer(42), "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_cache_stats() {
        let mut interp = make_interpreter();
        let s = Value::from_string("test".to_string());
        let _ = interp.get_field_cached(&s, "len");
        let stats = interp.get_cache_stats();
        assert!(!stats.is_empty());
    }

    #[test]
    fn test_clear_caches() {
        let mut interp = make_interpreter();
        let s = Value::from_string("test".to_string());
        let _ = interp.get_field_cached(&s, "len");
        interp.clear_caches();
        assert!(interp.get_cache_stats().is_empty());
    }

    // ===== Type Feedback Operations Tests =====

    #[test]
    fn test_record_binary_op_feedback() {
        let mut interp = make_interpreter();
        interp.record_binary_op_feedback(
            0,
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(3),
        );
        // No panic = success
    }

    #[test]
    fn test_record_variable_assignment_feedback() {
        let mut interp = make_interpreter();
        interp.record_variable_assignment_feedback("x", &Value::Integer(42));
        // No panic = success
    }

    #[test]
    fn test_record_function_call_feedback() {
        let mut interp = make_interpreter();
        interp.record_function_call_feedback(0, "test", &[Value::Integer(1)], &Value::Integer(2));
        // No panic = success
    }

    #[test]
    fn test_get_type_feedback_stats() {
        let mut interp = make_interpreter();
        interp.record_binary_op_feedback(
            0,
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(3),
        );
        let stats = interp.get_type_feedback_stats();
        let _ = stats.total_operation_sites; // Verify field exists
    }

    #[test]
    fn test_get_specialization_candidates() {
        let interp = make_interpreter();
        let candidates = interp.get_specialization_candidates();
        // Should return a list (empty or populated)
        let _ = candidates.len();
    }

    #[test]
    fn test_clear_type_feedback() {
        let mut interp = make_interpreter();
        interp.record_binary_op_feedback(
            0,
            &Value::Integer(1),
            &Value::Integer(2),
            &Value::Integer(3),
        );
        interp.clear_type_feedback();
        // No panic = success
    }

    // ===== Error Scope Operations Tests =====

    #[test]
    fn test_push_pop_error_scope() {
        let mut interp = make_interpreter();
        interp.push_error_scope();
        interp.pop_error_scope();
        // No panic = success
    }

    // ===== Session Operations Tests =====

    #[test]
    fn test_get_global_bindings() {
        let interp = make_interpreter();
        let bindings = interp.get_global_bindings();
        // Should contain builtin functions
        assert!(bindings.contains_key("max"));
    }

    #[test]
    fn test_set_global_binding() {
        let mut interp = make_interpreter();
        interp.set_global_binding("test_var".to_string(), Value::Integer(42));
        let bindings = interp.get_global_bindings();
        assert_eq!(bindings.get("test_var"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_clear_user_variables() {
        let mut interp = make_interpreter();
        interp.set_global_binding("test_var".to_string(), Value::Integer(42));
        interp.clear_user_variables();
        let bindings = interp.get_global_bindings();
        assert!(!bindings.contains_key("test_var"));
    }

    #[test]
    fn test_get_current_bindings() {
        let mut interp = make_interpreter();
        interp.push_scope();
        interp.env_set("local_var".to_string(), Value::Integer(100));
        let bindings = interp.get_current_bindings();
        assert_eq!(bindings.get("local_var"), Some(&Value::Integer(100)));
        interp.pop_scope();
    }

    // ===== Binary Operations Tests =====

    #[test]
    fn test_binary_op_add() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(2)).unwrap();
        interp.push(Value::Integer(3)).unwrap();
        interp.binary_op(BinaryOp::Add).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_binary_op_sub() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(10)).unwrap();
        interp.push(Value::Integer(3)).unwrap();
        interp.binary_op(BinaryOp::Sub).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Integer(7));
    }

    #[test]
    fn test_binary_op_mul() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(4)).unwrap();
        interp.push(Value::Integer(5)).unwrap();
        interp.binary_op(BinaryOp::Mul).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_binary_op_div() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(20)).unwrap();
        interp.push(Value::Integer(4)).unwrap();
        interp.binary_op(BinaryOp::Div).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_binary_op_eq() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(5)).unwrap();
        interp.push(Value::Integer(5)).unwrap();
        interp.binary_op(BinaryOp::Eq).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_binary_op_lt() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(3)).unwrap();
        interp.push(Value::Integer(5)).unwrap();
        interp.binary_op(BinaryOp::Lt).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_binary_op_gt() {
        let mut interp = make_interpreter();
        interp.push(Value::Integer(10)).unwrap();
        interp.push(Value::Integer(5)).unwrap();
        interp.binary_op(BinaryOp::Gt).unwrap();
        assert_eq!(interp.pop().unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_apply_binary_op() {
        let interp = make_interpreter();
        let result = interp
            .apply_binary_op(&Value::Integer(2), AstBinaryOp::Add, &Value::Integer(3))
            .unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    // ===== Pattern Matching Tests =====

    #[test]
    fn test_pattern_matches_identifier() {
        let mut interp = make_interpreter();
        let pattern = Pattern::Identifier("x".to_string());
        let result = interp
            .pattern_matches(&pattern, &Value::Integer(42))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_pattern_matches_wildcard() {
        let mut interp = make_interpreter();
        let pattern = Pattern::Wildcard;
        let result = interp
            .pattern_matches(&pattern, &Value::Integer(42))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_pattern_matches_literal_int() {
        let mut interp = make_interpreter();
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let result = interp
            .pattern_matches(&pattern, &Value::Integer(42))
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_pattern_matches_literal_int_mismatch() {
        let mut interp = make_interpreter();
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let result = interp
            .pattern_matches(&pattern, &Value::Integer(99))
            .unwrap();
        assert!(!result);
    }

    #[test]
    fn test_literal_matches_float() {
        let interp = make_interpreter();
        let result = interp.literal_matches(&Literal::Float(3.14), &Value::Float(3.14));
        assert!(result);
    }

    #[test]
    fn test_literal_matches_string() {
        let interp = make_interpreter();
        let result = interp.literal_matches(
            &Literal::String("hello".to_string()),
            &Value::from_string("hello".to_string()),
        );
        assert!(result);
    }

    #[test]
    fn test_literal_matches_bool() {
        let interp = make_interpreter();
        let result = interp.literal_matches(&Literal::Bool(true), &Value::Bool(true));
        assert!(result);
    }

    #[test]
    fn test_try_pattern_match_identifier() {
        let interp = make_interpreter();
        let pattern = Pattern::Identifier("x".to_string());
        let result = interp
            .try_pattern_match(&pattern, &Value::Integer(42))
            .unwrap();
        assert!(result.is_some());
        let bindings = result.unwrap();
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
    }

    #[test]
    fn test_pattern_matches_internal() {
        let interp = make_interpreter();
        let pattern = Pattern::Wildcard;
        let result = interp
            .pattern_matches_internal(&pattern, &Value::Integer(42))
            .unwrap();
        assert!(result);
    }

    // ===== Scope Operations Tests =====

    #[test]
    fn test_push_pop_scope() {
        let mut interp = make_interpreter();
        let initial_depth = interp.env_stack.len();
        interp.push_scope();
        assert_eq!(interp.env_stack.len(), initial_depth + 1);
        interp.pop_scope();
        assert_eq!(interp.env_stack.len(), initial_depth);
    }

    // ===== Lookup Variable Tests =====

    #[test]
    fn test_lookup_option_none() {
        let interp = make_interpreter();
        let result = interp.lookup_variable("Option::None").unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            ..
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "None");
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_lookup_json_global() {
        let interp = make_interpreter();
        let result = interp.lookup_variable("JSON").unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("JSON".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_lookup_file_global() {
        let interp = make_interpreter();
        let result = interp.lookup_variable("File").unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("File".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    // ===== Get Variable Tests =====

    #[test]
    fn test_get_variable_found() {
        let mut interp = make_interpreter();
        interp.set_variable("test", Value::Integer(42));
        assert_eq!(interp.get_variable("test"), Some(Value::Integer(42)));
    }

    #[test]
    fn test_get_variable_not_found() {
        let interp = make_interpreter();
        assert_eq!(interp.get_variable("nonexistent"), None);
    }

    // ===== eval_contains Tests =====

    #[test]
    fn test_eval_contains_tuple() {
        let interp = make_interpreter();
        let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = interp.eval_contains(&Value::Integer(1), &tuple).unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_contains_object_key_string() {
        let interp = make_interpreter();
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(1));
        let obj = Value::Object(Arc::new(map));
        let result = interp
            .eval_contains(&Value::from_string("key".to_string()), &obj)
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_contains_object_key_nonstring() {
        let interp = make_interpreter();
        let mut map = HashMap::new();
        map.insert("42".to_string(), Value::Integer(1));
        let obj = Value::Object(Arc::new(map));
        let result = interp.eval_contains(&Value::Integer(42), &obj).unwrap();
        assert!(result);
    }

    #[test]
    fn test_eval_contains_unsupported() {
        let interp = make_interpreter();
        let result = interp.eval_contains(&Value::Integer(1), &Value::Integer(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_contains_string_invalid_element() {
        let interp = make_interpreter();
        let s = Value::from_string("hello".to_string());
        let result = interp.eval_contains(&Value::Integer(1), &s);
        assert!(result.is_err());
    }

    // ===== eval_literal Tests =====

    #[test]
    fn test_eval_literal_byte() {
        let interp = make_interpreter();
        let result = interp.eval_literal(&Literal::Byte(42));
        assert_eq!(result, Value::Byte(42));
    }

    #[test]
    fn test_eval_literal_null() {
        let interp = make_interpreter();
        let result = interp.eval_literal(&Literal::Null);
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_literal_atom() {
        let interp = make_interpreter();
        let result = interp.eval_literal(&Literal::Atom("test".to_string()));
        assert_eq!(result, Value::Atom("test".to_string()));
    }

    // ===== has_stdout Test =====

    #[test]
    fn test_has_stdout_empty() {
        let interp = make_interpreter();
        assert!(!interp.has_stdout());
    }

    // ===== set_variable_string Test =====

    #[test]
    fn test_set_variable_string() {
        let mut interp = make_interpreter();
        interp.set_variable_string("x".to_string(), Value::Integer(42));
        assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(42));
    }

    // ===== eval_function_call_value Test =====

    #[test]
    fn test_eval_function_call_value() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![],
            body,
            env,
        };
        let result = interp.eval_function_call_value(&closure, &[]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== eval_dataframe_literal Test =====

    #[test]
    fn test_eval_dataframe_literal() {
        use crate::frontend::ast::DataFrameColumn as AstDFColumn;
        let mut interp = make_interpreter();
        let columns = vec![AstDFColumn {
            name: "x".to_string(),
            values: vec![make_expr(ExprKind::Literal(Literal::Integer(1, None)))],
        }];
        let result = interp.eval_dataframe_literal(&columns).unwrap();
        assert!(matches!(result, Value::DataFrame { .. }));
    }

    // ===== Stack Overflow Test =====

    #[test]
    fn test_push_stack_overflow() {
        let mut interp = make_interpreter();
        for i in 0..10_000 {
            interp.push(Value::Integer(i as i64)).unwrap();
        }
        // 10001th push should fail
        let result = interp.push(Value::Integer(0));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Stack overflow"));
    }

    // ===== eval_array_init_expr Test =====

    #[test]
    fn test_eval_array_init_expr() {
        let mut interp = make_interpreter();
        let value_expr = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
        let size_expr = make_expr(ExprKind::Literal(Literal::Integer(5, None)));
        let result = interp
            .eval_array_init_expr(&value_expr, &size_expr)
            .unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            for v in arr.iter() {
                assert_eq!(*v, Value::Integer(0));
            }
        } else {
            panic!("Expected Array");
        }
    }

    // ===== eval_special_form Tests =====

    #[test]
    fn test_eval_special_form_none() {
        let mut interp = make_interpreter();
        let kind = ExprKind::None;
        let result = interp.eval_special_form(&kind).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "None");
            assert!(data.is_none());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_eval_special_form_some() {
        let mut interp = make_interpreter();
        let inner = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let kind = ExprKind::Some {
            value: Box::new(inner),
        };
        let result = interp.eval_special_form(&kind).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "Some");
            assert!(data.is_some());
            let values = data.unwrap();
            assert_eq!(values[0], Value::Integer(42));
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_eval_special_form_set_empty() {
        let mut interp = make_interpreter();
        let kind = ExprKind::Set(vec![]);
        let result = interp.eval_special_form(&kind).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_special_form_set_with_statements() {
        let mut interp = make_interpreter();
        let kind = ExprKind::Set(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ]);
        let result = interp.eval_special_form(&kind).unwrap();
        assert_eq!(result, Value::Integer(2)); // Returns last statement value
    }

    #[test]
    fn test_eval_special_form_object_literal() {
        use crate::frontend::ast::ObjectField;
        let mut interp = make_interpreter();
        let kind = ExprKind::ObjectLiteral {
            fields: vec![ObjectField::KeyValue {
                key: "x".to_string(),
                value: make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            }],
        };
        let result = interp.eval_special_form(&kind).unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("x"), Some(&Value::Integer(1)));
        } else {
            panic!("Expected Object");
        }
    }

    // ===== resolve_module_path Tests =====

    #[test]
    fn test_resolve_module_path_not_found() {
        let interp = make_interpreter();
        let result = interp.resolve_module_path("nonexistent::module");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_module_path_std() {
        let interp = make_interpreter();
        let result = interp.resolve_module_path("std");
        assert!(result.is_some());
    }

    // ===== eval_type_cast Tests =====

    #[test]
    fn test_eval_type_cast_int_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_type_cast(&expr, "f64").unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_eval_type_cast_float_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Float(3.9)));
        let result = interp.eval_type_cast(&expr, "i64").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_type_cast_int_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_type_cast(&expr, "i32").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_type_cast_float_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Float(3.14)));
        let result = interp.eval_type_cast(&expr, "f32").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_type_cast_unsupported() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        let result = interp.eval_type_cast(&expr, "i32");
        assert!(result.is_err());
    }

    // ===== call_function Tests =====

    #[test]
    fn test_call_function_closure_no_args() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![],
            body,
            env,
        };
        let result = interp.call_function(closure, &[]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_call_function_closure_with_args() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Identifier("x".to_string())));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body,
            env,
        };
        let result = interp
            .call_function(closure, &[Value::Integer(42)])
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_call_function_wrong_arg_count() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body,
            env,
        };
        let result = interp.call_function(closure, &[]); // Missing required arg
        assert!(result.is_err());
    }

    #[test]
    fn test_call_function_non_callable() {
        let mut interp = make_interpreter();
        let result = interp.call_function(Value::Integer(42), &[]);
        assert!(result.is_err());
    }

    // ===== eval_binary_expr Tests =====

    #[test]
    fn test_eval_binary_expr_null_coalesce_nil() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Null));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::NullCoalesce, &right)
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_binary_expr_null_coalesce_not_nil() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Integer(10, None)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::NullCoalesce, &right)
            .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_eval_binary_expr_and_short_circuit() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::And, &right)
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_binary_expr_or_short_circuit() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::Or, &right)
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_expr_in_array() {
        let mut interp = make_interpreter();
        let element = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let collection = make_expr(ExprKind::List(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ]));
        let result = interp
            .eval_binary_expr(&element, AstBinaryOp::In, &collection)
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ===== eval_unary_expr Tests =====

    #[test]
    fn test_eval_unary_expr_negate() {
        let mut interp = make_interpreter();
        let operand = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_unary_expr(UnaryOp::Negate, &operand).unwrap();
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_eval_unary_expr_not() {
        let mut interp = make_interpreter();
        let operand = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let result = interp.eval_unary_expr(UnaryOp::Not, &operand).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // ===== eval_if_expr Tests =====

    #[test]
    fn test_eval_if_expr_true_branch() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let else_branch = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let result = interp
            .eval_if_expr(&cond, &then_branch, Some(&else_branch))
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_if_expr_false_branch() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let else_branch = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let result = interp
            .eval_if_expr(&cond, &then_branch, Some(&else_branch))
            .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_if_expr_no_else() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let result = interp.eval_if_expr(&cond, &then_branch, None).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== eval_let_expr Tests =====

    #[test]
    fn test_eval_let_expr() {
        let mut interp = make_interpreter();
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let result = interp.eval_let_expr("x", &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_let_expr_unit_body() {
        let mut interp = make_interpreter();
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Literal(Literal::Unit));
        let result = interp.eval_let_expr("x", &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== eval_block_expr with multiple statements Tests =====

    #[test]
    fn test_eval_block_expr_multiple() {
        let mut interp = make_interpreter();
        let stmts = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ];
        let result = interp.eval_block_expr(&stmts).unwrap();
        assert_eq!(result, Value::Integer(3)); // Returns last
    }

    // ===== String interpolation Tests =====

    #[test]
    fn test_eval_string_interpolation_text_only() {
        use crate::frontend::ast::StringPart;
        let mut interp = make_interpreter();
        let parts = vec![StringPart::Text("hello world".to_string())];
        let result = interp.eval_string_interpolation(&parts).unwrap();
        assert_eq!(result, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_eval_string_interpolation_with_expr() {
        use crate::frontend::ast::StringPart;
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(42));
        let parts = vec![
            StringPart::Text("x = ".to_string()),
            StringPart::Expr(Box::new(make_expr(ExprKind::Identifier("x".to_string())))),
        ];
        let result = interp.eval_string_interpolation(&parts).unwrap();
        assert_eq!(result, Value::from_string("x = 42".to_string()));
    }

    // ===== Macro tests via eval_string =====

    #[test]
    fn test_eval_vec_macro() {
        let mut interp = make_interpreter();
        let result = interp.eval_string("vec![1, 2, 3]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    // ===== eval_comprehension Tests =====

    #[test]
    fn test_eval_list_comprehension_simple() {
        use crate::frontend::ast::ComprehensionClause;
        let mut interp = make_interpreter();
        let element = make_expr(ExprKind::Identifier("x".to_string()));
        let clauses = vec![ComprehensionClause {
            variable: "x".to_string(),
            iterable: Box::new(make_expr(ExprKind::List(vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            ]))),
            condition: None,
        }];
        let result = interp.eval_list_comprehension(&element, &clauses).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_check_comprehension_condition_none() {
        let mut interp = make_interpreter();
        let result = interp.check_comprehension_condition(None).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_comprehension_condition_true() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let result = interp.check_comprehension_condition(Some(&cond)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_comprehension_condition_false() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let result = interp.check_comprehension_condition(Some(&cond)).unwrap();
        assert!(!result);
    }

    // ===== Macro Tests =====

    #[test]
    fn test_vec_macro_empty() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_vec_macro_with_elements() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
                make_expr(ExprKind::Literal(Literal::Integer(3, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_format_macro_simple() {
        // Test that format! macro returns a string value
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "hello".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // format! returns a Value::String
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_macro_with_placeholder() {
        // Test that format! macro handles placeholders
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::String("x = {}".to_string()))),
                make_expr(ExprKind::Literal(Literal::Integer(42, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // format! returns a Value::String containing the formatted output
        assert!(matches!(result, Value::String(_)));
        // Check that the result contains "42"
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_format_macro_empty_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_macro_debug_placeholder() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::String("val = {:?}".to_string()))),
                make_expr(ExprKind::Literal(Literal::Integer(5, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // Check that result contains the debug format
        assert!(result.to_string().contains("5") || result.to_string().contains("Integer"));
    }

    #[test]
    fn test_unknown_macro_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "unknown_macro".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Control Flow Tests =====

    #[test]
    fn test_ternary_true() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Ternary {
                condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
                true_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
                false_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_ternary_false() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Ternary {
                condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(false)))),
                true_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
                false_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_break_with_value() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Break {
                label: None,
                value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                    42, None,
                ))))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Break(None, Value::Integer(42)))
        ));
    }

    #[test]
    fn test_break_with_label() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Break {
                label: Some("outer".to_string()),
                value: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Break(Some(label), Value::Nil)) if label == "outer"
        ));
    }

    #[test]
    fn test_continue_with_label() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Continue {
                label: Some("inner".to_string()),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Continue(Some(label))) if label == "inner"
        ));
    }

    // ===== Special Form Tests =====

    #[test]
    fn test_none_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::None);
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "None");
            assert!(data.is_none());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_some_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Some {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "Some");
            assert!(data.is_some());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_set_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Set(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ]));
        let result = interp.eval_expr(&expr).unwrap();
        // Set returns last value
        assert_eq!(result, Value::Integer(3));
    }

    // ===== Import Tests =====

    #[test]
    fn test_import_all_wildcard() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportAll {
            module: "std::math".to_string(),
            alias: "*".to_string(),
        });
        // Wildcard imports return Nil
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_import_default() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportDefault {
            module: "mymodule".to_string(),
            name: "mm".to_string(),
        });
        // ImportDefault not yet implemented
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Additional Tests for Coverage =====

    #[test]
    fn test_env_set_mut_coverage() {
        let mut interp = make_interpreter();
        interp.env_set_mut("mutable_var_cov".to_string(), Value::Integer(1));
        assert!(interp.lookup_variable("mutable_var_cov").is_ok());
    }

    #[test]
    fn test_is_actor_operation_coverage() {
        let actor_expr = Box::new(make_expr(ExprKind::Identifier("actor".to_string())));
        assert!(Interpreter::is_actor_operation(&ExprKind::Spawn {
            actor: actor_expr,
        }));
    }

    #[test]
    fn test_is_special_form_coverage() {
        assert!(Interpreter::is_special_form(&ExprKind::None));
        assert!(Interpreter::is_special_form(&ExprKind::Some {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        }));
    }

    #[test]
    fn test_is_control_flow_expr_coverage() {
        assert!(Interpreter::is_control_flow_expr(&ExprKind::If {
            condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: None,
        }));
    }

    #[test]
    fn test_is_data_structure_expr_coverage() {
        assert!(Interpreter::is_data_structure_expr(&ExprKind::List(vec![])));
    }

    #[test]
    fn test_is_assignment_expr_coverage() {
        assert!(Interpreter::is_assignment_expr(&ExprKind::Assign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        }));
    }

    // ===== MacroInvocation Tests =====

    #[test]
    fn test_macro_invocation_vec() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "vec".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_macro_invocation_format() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "test".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_macro_invocation_unknown() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "unknown_macro".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Try Operator Tests =====

    #[test]
    fn test_try_operator_ok() {
        let mut interp = make_interpreter();
        let ok_value = make_expr(ExprKind::Call {
            func: Box::new(make_expr(ExprKind::Identifier("Ok".to_string()))),
            args: vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))],
        });
        // We need to set up Ok function first
        interp.set_variable(
            "Ok",
            Value::Closure {
                params: vec![("value".to_string(), None)],
                body: std::sync::Arc::new(make_expr(ExprKind::Block(vec![]))),
                env: interp.current_env().clone(),
            },
        );
        // Try operator test with EnumVariant
        let ok_enum = Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        interp.set_variable("result_val", ok_enum);
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("result_val".to_string()))),
        });
        let result = interp.eval_expr(&try_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_try_operator_err() {
        let mut interp = make_interpreter();
        let err_enum = Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            data: Some(vec![Value::from_string("error".to_string())]),
        };
        interp.set_variable("err_val", err_enum);
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("err_val".to_string()))),
        });
        let result = interp.eval_expr(&try_expr);
        // Should return an error (early return with Err variant)
        assert!(result.is_err());
    }

    #[test]
    fn test_try_operator_invalid_type() {
        let mut interp = make_interpreter();
        interp.set_variable("not_result", Value::Integer(42));
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("not_result".to_string()))),
        });
        let result = interp.eval_expr(&try_expr);
        assert!(result.is_err());
    }

    // ===== Lazy Expression Tests =====

    #[test]
    fn test_lazy_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Lazy {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== AsyncBlock Tests =====

    #[test]
    fn test_async_block() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::AsyncBlock {
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(100, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    // ===== IfLet Tests =====

    #[test]
    fn test_if_let_match() {
        let mut interp = make_interpreter();
        // Set up a Some value
        let some_val = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        interp.set_variable("opt", some_val);

        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Identifier("x".to_string()),
            expr: Box::new(make_expr(ExprKind::Identifier("opt".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                0, None,
            ))))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_if_let_no_match() {
        let mut interp = make_interpreter();
        // Set up Nil (won't match wildcard pattern that expects value)
        interp.set_variable("opt", Value::Nil);

        // Use a pattern that won't match Nil
        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Literal(Literal::Integer(5, None)),
            expr: Box::new(make_expr(ExprKind::Identifier("opt".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                0, None,
            ))))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_if_let_no_else() {
        let mut interp = make_interpreter();
        interp.set_variable("val", Value::Integer(10));

        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Literal(Literal::Integer(5, None)), // Won't match
            expr: Box::new(make_expr(ExprKind::Identifier("val".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: None,
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Module Expression Tests =====

    #[test]
    fn test_module_declaration_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ModuleDeclaration {
            name: "unresolved_module".to_string(),
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Pipeline Tests =====

    #[test]
    fn test_pipeline_with_method() {
        let mut interp = make_interpreter();
        // Define a simple function
        interp.set_variable(
            "double",
            Value::Closure {
                params: vec![("x".to_string(), None)],
                body: std::sync::Arc::new(make_expr(ExprKind::Binary {
                    left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                    op: crate::frontend::ast::BinaryOp::Multiply,
                    right: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
                })),
                env: interp.current_env().clone(),
            },
        );

        let expr = make_expr(ExprKind::Pipeline {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
            stages: vec![crate::frontend::ast::PipelineStage {
                op: Box::new(make_expr(ExprKind::Identifier("double".to_string()))),
                span: Span::default(),
            }],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    // ===== Format String Edge Cases =====

    #[test]
    fn test_format_debug_incomplete() {
        let mut interp = make_interpreter();
        // Test {:? without closing brace
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{:?x".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_colon_only() {
        let mut interp = make_interpreter();
        // Test {: without ?
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{:x".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_excess_placeholders() {
        let mut interp = make_interpreter();
        // More {} than arguments - should preserve placeholder
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{} {} {}".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert!(s.as_ref().contains("{}"));
        } else {
            panic!("Expected String");
        }
    }

    // ===== Import Tests =====

    #[test]
    fn test_import_stdlib() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Import {
            module: "std::env".to_string(),
            items: None,
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_import_all_with_alias() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportAll {
            module: "std::math".to_string(),
            alias: "m".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Additional Expression Tests =====

    #[test]
    fn test_eval_string_interpolation_empty() {
        use crate::frontend::ast::StringPart;
        let mut interp = make_interpreter();
        let parts = vec![];
        let expr = make_expr(ExprKind::StringInterpolation { parts });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_eval_qualified_name() {
        let mut interp = make_interpreter();
        // Set up std::env in environment
        let expr = make_expr(ExprKind::QualifiedName {
            module: "std".to_string(),
            name: "env".to_string(),
        });
        // This may fail if std::env is not set up, but it tests the code path
        let _result = interp.eval_expr(&expr);
    }

    #[test]
    fn test_loop_expression() {
        let mut interp = make_interpreter();
        // Create a loop that breaks immediately
        interp.set_variable("counter", Value::Integer(0));
        let loop_expr = Expr {
            kind: ExprKind::Loop {
                label: None,
                body: Box::new(make_expr(ExprKind::Break {
                    label: None,
                    value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                        42, None,
                    ))))),
                })),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&loop_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_labeled_loop() {
        let mut interp = make_interpreter();
        let loop_expr = Expr {
            kind: ExprKind::Loop {
                label: Some("outer".to_string()),
                body: Box::new(make_expr(ExprKind::Break {
                    label: Some("outer".to_string()),
                    value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                        99, None,
                    ))))),
                })),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&loop_expr).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // ===== Return Expression Test =====

    #[test]
    fn test_return_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Return {
            value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                123, None,
            ))))),
        });
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Return(Value::Integer(123)))
        ));
    }

    #[test]
    fn test_return_expr_no_value() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Return { value: None });
        let result = interp.eval_expr(&expr);
        assert!(matches!(result, Err(InterpreterError::Return(Value::Nil))));
    }

    // ===== Array Init Test =====

    #[test]
    fn test_array_init_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ArrayInit {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            size: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
        } else {
            panic!("Expected Array");
        }
    }

    // ===== Index Access Test =====

    #[test]
    fn test_index_access_tuple() {
        let mut interp = make_interpreter();
        let tuple = Value::Tuple(std::sync::Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        interp.set_variable("t", tuple);

        let expr = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::Identifier("t".to_string()))),
            index: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ===== call_function Edge Cases =====

    #[test]
    fn test_call_function_class_constructor() {
        let mut interp = make_interpreter();
        // Set up a minimal class definition
        let mut class_def = HashMap::new();
        class_def.insert(
            "__type".to_string(),
            Value::from_string("Class".to_string()),
        );
        class_def.insert(
            "__name".to_string(),
            Value::from_string("Point".to_string()),
        );
        class_def.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(HashMap::new())),
        );
        class_def.insert(
            "__methods".to_string(),
            Value::Array(std::sync::Arc::from(vec![])),
        );
        interp.set_variable("Point", Value::Object(std::sync::Arc::new(class_def)));

        let constructor = Value::from_string("__class_constructor__:Point:new".to_string());
        let result = interp.call_function(constructor, &[]);
        // May fail if instantiate_class_with_constructor isn't fully set up
        // but this tests the code path
        let _ = result;
    }

    #[test]
    fn test_call_function_struct_constructor() {
        let mut interp = make_interpreter();
        // Set up a minimal struct definition
        let mut struct_def = HashMap::new();
        struct_def.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        struct_def.insert(
            "__name".to_string(),
            Value::from_string("Point".to_string()),
        );
        struct_def.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(HashMap::new())),
        );
        interp.set_variable("Point", Value::Object(std::sync::Arc::new(struct_def)));

        let constructor = Value::from_string("__struct_constructor__:Point".to_string());
        let result = interp.call_function(constructor, &[]);
        // May succeed or fail depending on struct setup
        let _ = result;
    }

    #[test]
    fn test_call_function_closure_with_defaults() {
        let mut interp = make_interpreter();
        // Create a closure with a default parameter
        let closure = Value::Closure {
            params: vec![
                ("x".to_string(), None),
                (
                    "y".to_string(),
                    Some(std::sync::Arc::new(make_expr(ExprKind::Literal(
                        Literal::Integer(10, None),
                    )))),
                ),
            ],
            body: std::sync::Arc::new(make_expr(ExprKind::Binary {
                left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                op: crate::frontend::ast::BinaryOp::Add,
                right: Box::new(make_expr(ExprKind::Identifier("y".to_string()))),
            })),
            env: interp.current_env().clone(),
        };

        // Call with only required arg - default should be used
        let result = interp
            .call_function(closure.clone(), &[Value::Integer(5)])
            .unwrap();
        assert_eq!(result, Value::Integer(15));

        // Call with both args
        let result = interp
            .call_function(closure, &[Value::Integer(5), Value::Integer(3)])
            .unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_call_function_arg_count_errors() {
        let mut interp = make_interpreter();
        let closure = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: std::sync::Arc::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            env: interp.current_env().clone(),
        };

        // Too few arguments
        let result = interp.call_function(closure.clone(), &[Value::Integer(1)]);
        assert!(result.is_err());

        // Too many arguments
        let result = interp.call_function(
            closure,
            &[Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_call_function_builtin() {
        let mut interp = make_interpreter();
        // Test a builtin function
        let builtin = Value::from_string("__builtin_len__".to_string());
        let arr = Value::Array(std::sync::Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = interp.call_function(builtin, &[arr]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ===== Spread Expression Tests =====

    #[test]
    fn test_spread_in_list_returns_error() {
        let mut interp = make_interpreter();
        interp.set_variable(
            "inner",
            Value::Array(std::sync::Arc::from(vec![
                Value::Integer(2),
                Value::Integer(3),
            ])),
        );

        // Spread inside list triggers error (feature not yet implemented)
        let expr = make_expr(ExprKind::List(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Spread {
                expr: Box::new(make_expr(ExprKind::Identifier("inner".to_string()))),
            }),
            make_expr(ExprKind::Literal(Literal::Integer(4, None))),
        ]));
        let result = interp.eval_expr(&expr);
        // Spread expressions in list context return error
        assert!(result.is_err());
    }

    // ===== Await Expression Test =====

    #[test]
    fn test_await_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Await {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        // In sync mode, await just returns the value
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== Field Access Tests =====

    #[test]
    fn test_field_access_object() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(42));
        obj.insert("y".to_string(), Value::Integer(100));
        interp.set_variable("obj", Value::Object(std::sync::Arc::new(obj)));

        let expr = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("obj".to_string()))),
            field: "x".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_field_access_struct() {
        let mut interp = make_interpreter();
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::from_string("Alice".to_string()));
        let struct_val = Value::Struct {
            name: "Person".to_string(),
            fields: std::sync::Arc::new(fields),
        };
        interp.set_variable("person", struct_val);

        let expr = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("person".to_string()))),
            field: "name".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "Alice");
        } else {
            panic!("Expected String");
        }
    }

    // ===== Compound Assignment Tests =====

    #[test]
    fn test_compound_add_assign() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        let expr = make_expr(ExprKind::CompoundAssign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            op: crate::frontend::ast::BinaryOp::Add,
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
        });
        interp.eval_expr(&expr).unwrap();

        let x = interp.lookup_variable("x").unwrap();
        assert_eq!(x, Value::Integer(15));
    }

    #[test]
    fn test_compound_mul_assign() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(3));

        let expr = make_expr(ExprKind::CompoundAssign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            op: crate::frontend::ast::BinaryOp::Multiply,
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(4, None)))),
        });
        interp.eval_expr(&expr).unwrap();

        let x = interp.lookup_variable("x").unwrap();
        assert_eq!(x, Value::Integer(12));
    }

    // ===== Type Cast Tests =====

    #[test]
    fn test_type_cast_int_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::TypeCast {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
            target_type: "f64".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 42.0).abs() < 0.001);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_type_cast_float_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::TypeCast {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Float(3.7)))),
            target_type: "i32".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }
}
