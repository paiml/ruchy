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
    pub(crate) fn is_type_definition(expr_kind: &ExprKind) -> bool {
        crate::runtime::interpreter_misc_eval::is_type_definition(expr_kind)
    }

    /// Helper: Check if expression is an actor operation
    pub(crate) fn is_actor_operation(expr_kind: &ExprKind) -> bool {
        crate::runtime::interpreter_misc_eval::is_actor_operation(expr_kind)
    }

    /// Helper: Check if expression is a special form
    pub(crate) fn is_special_form(expr_kind: &ExprKind) -> bool {
        crate::runtime::interpreter_misc_eval::is_special_form(expr_kind)
    }

    /// Evaluate type definition expressions (Actor, Struct, Class, Impl)
    /// Delegates to `interpreter_misc_eval` module.
    pub(crate) fn eval_type_definition(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_type_definition(self, expr_kind)
    }

    /// Evaluate actor operation expressions (Spawn, `ActorSend`, `ActorQuery`)
    /// Delegates to `interpreter_misc_eval` module.
    pub(crate) fn eval_actor_operation(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_actor_operation(self, expr_kind)
    }

    /// Evaluate special form expressions (None, Some, Set, patterns, literals)
    /// Delegates to `interpreter_misc_eval` module.
    pub(crate) fn eval_special_form(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_special_form(self, expr_kind)
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
    /// Evaluate miscellaneous expressions.
    /// Delegates to `interpreter_misc_eval` module.
    pub(crate) fn eval_misc_expr(
        &mut self,
        expr_kind: &ExprKind,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_misc_expr(self, expr_kind)
    }

    /// Evaluate list comprehension: [expr for x in iter if cond]
    pub(crate) fn eval_list_comprehension(
        &mut self,
        element: &Expr,
        clauses: &[ComprehensionClause],
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_list_comprehension(self, element, clauses)
    }

    /// Recursively process comprehension clauses
    pub(crate) fn eval_comprehension_clauses(
        &mut self,
        results: &mut Vec<Value>,
        element: &Expr,
        clauses: &[ComprehensionClause],
        clause_idx: usize,
    ) -> Result<(), InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_comprehension_clauses(self, results, element, clauses, clause_idx)
    }

    /// Helper: Check comprehension condition
    pub(crate) fn check_comprehension_condition(
        &mut self,
        condition: Option<&Expr>,
    ) -> Result<bool, InterpreterError> {
        crate::runtime::interpreter_misc_eval::check_comprehension_condition(self, condition)
    }

    /// Helper: Evaluate spawn actor expression with proper nesting handling
    pub(crate) fn eval_spawn_actor(&mut self, actor: &Expr) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_spawn_actor(self, actor)
    }

    /// Helper: Evaluate actor send expression (fire-and-forget)
    pub(crate) fn eval_actor_send(
        &mut self,
        actor: &Expr,
        message: &Expr,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_actor_send(self, actor, message)
    }

    /// Helper: Evaluate actor query expression (ask pattern)
    pub(crate) fn eval_actor_query(
        &mut self,
        actor: &Expr,
        message: &Expr,
    ) -> Result<Value, InterpreterError> {
        crate::runtime::interpreter_misc_eval::eval_actor_query(self, actor, message)
    }

    // Actor message extraction delegated to eval_actor module
    // EXTREME TDD: Eliminated 35 lines of duplicate code
    pub(crate) fn extract_message_type_and_data(
        message: &Value,
    ) -> Result<(String, Vec<Value>), InterpreterError> {
        crate::runtime::eval_actor::extract_message_type_and_data(message)
    }

    /// Provide mutable access to the module loader (for interpreter_misc_eval)
    pub(crate) fn module_loader_mut(&mut self) -> &mut crate::backend::module_loader::ModuleLoader {
        &mut self.module_loader
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
#[path = "interpreter_core_tests.rs"]
mod tests;
