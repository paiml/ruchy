//! Function evaluation module
//!
//! This module handles all function-related operations in the interpreter.
//! Provides function definition, calling, closure capture, and parameter binding.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Pattern};
use crate::runtime::eval_pattern::match_pattern;
use crate::runtime::{InterpreterError, Value};
use std::cell::{Cell, RefCell}; // ISSUE-119: Add RefCell for environment wrapping
use std::collections::HashMap;
use std::rc::Rc; // ISSUE-119: Add Rc for shared ownership
use std::sync::Arc;

// ============================================================================
// Recursion Depth Tracking ([RUNTIME-001] Fix)
// ============================================================================

thread_local! {
    /// Current recursion depth for this thread
    static CALL_DEPTH: Cell<usize> = const { Cell::new(0) };

    /// Maximum allowed recursion depth for this thread
    static MAX_DEPTH: Cell<usize> = const { Cell::new(1000) };
}

/// Set the maximum recursion depth limit
///
/// Default is 1000 calls (matches Python's limit).
/// Can be configured via REPL config or programmatically.
///
/// # Complexity
/// Cyclomatic: 1
pub fn set_max_recursion_depth(depth: usize) {
    MAX_DEPTH.with(|max| max.set(depth));
}

/// Get current recursion depth (for debugging/monitoring)
///
/// # Complexity
/// Cyclomatic: 1
pub fn get_current_depth() -> usize {
    CALL_DEPTH.with(std::cell::Cell::get)
}

/// Check recursion depth before entering function
///
/// Returns `RecursionLimitExceeded` error if depth would exceed limit.
/// Increments depth counter on success.
///
/// # Complexity
/// Cyclomatic: 2
pub fn check_recursion_depth() -> Result<(), InterpreterError> {
    CALL_DEPTH.with(|depth| {
        let current = depth.get();
        MAX_DEPTH.with(|max| {
            let max_val = max.get();
            if current >= max_val {
                Err(InterpreterError::RecursionLimitExceeded(current, max_val))
            } else {
                depth.set(current + 1);
                Ok(())
            }
        })
    })
}

/// Decrement recursion depth after exiting function
///
/// Must be called on ALL exit paths (success, error, return, etc.)
///
/// # Complexity
/// Cyclomatic: 1
pub fn decrement_depth() {
    CALL_DEPTH.with(|depth| {
        depth.set(depth.get().saturating_sub(1));
    });
}

/// Function closure with captured environment
#[derive(Debug, Clone)]
pub struct Closure {
    pub params: Vec<Pattern>,
    pub body: Expr,
    pub captured_env: HashMap<String, Value>,
    pub name: Option<String>,
}

impl Closure {
    /// Create a new closure with captured environment
    pub fn new(params: Vec<Pattern>, body: Expr, captured_env: HashMap<String, Value>) -> Self {
        Self {
            params,
            body,
            captured_env,
            name: None,
        }
    }

    /// Create a named closure (for recursive functions)
    pub fn named(
        params: Vec<Pattern>,
        body: Expr,
        captured_env: HashMap<String, Value>,
        name: String,
    ) -> Self {
        Self {
            params,
            body,
            captured_env,
            name: Some(name),
        }
    }
}

/// Evaluate a function definition and return a closure
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_function_def<F>(
    name: &str,
    params: &[crate::frontend::ast::Param],
    body: &Expr,
    capture_environment: F,
) -> Result<Value, InterpreterError>
where
    F: FnOnce() -> HashMap<String, Value>,
{
    let captured_env = capture_environment();
    let param_patterns: Vec<Pattern> = params.iter().map(|p| p.pattern.clone()).collect();
    let closure = Closure::named(param_patterns, body.clone(), captured_env, name.to_string());

    // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values from original params
    let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
        .iter()
        .map(|p| {
            let name = match &p.pattern {
                Pattern::Identifier(name) => name.clone(),
                _ => "_".to_string(), // Complex patterns converted to placeholder
            };
            let default = p.default_value.clone().map(|expr| Arc::new((*expr).clone()));
            (name, default)
        })
        .collect();

    Ok(Value::Closure {
        params: params_with_defaults,
        body: Arc::new(closure.body),
        env: Rc::new(RefCell::new(closure.captured_env)), // ISSUE-119: Wrap HashMap in Rc<RefCell>
    })
}

/// Evaluate a lambda expression and return a closure
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_lambda<F>(
    params: &[crate::frontend::ast::Param],
    body: &Expr,
    capture_environment: F,
) -> Result<Value, InterpreterError>
where
    F: FnOnce() -> HashMap<String, Value>,
{
    let captured_env = capture_environment();
    let param_patterns: Vec<Pattern> = params.iter().map(|p| p.pattern.clone()).collect();
    let closure = Closure::new(param_patterns, body.clone(), captured_env);

    // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values from original params
    let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
        .iter()
        .map(|p| {
            let name = match &p.pattern {
                Pattern::Identifier(name) => name.clone(),
                _ => "_".to_string(), // Complex patterns converted to placeholder
            };
            let default = p.default_value.clone().map(|expr| Arc::new((*expr).clone()));
            (name, default)
        })
        .collect();

    Ok(Value::Closure {
        params: params_with_defaults,
        body: Arc::new(closure.body),
        env: Rc::new(RefCell::new(closure.captured_env)), // ISSUE-119: Wrap HashMap in Rc<RefCell>
    })
}

/// Call a function with given arguments
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_function_call<F1, F2>(
    function: &Value,
    args: &[Value],
    eval_with_env: F1,
    _eval_builtin: F2,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Expr, &HashMap<String, Value>) -> Result<Value, InterpreterError>,
    F2: FnMut(&str, &[Value]) -> Result<Option<Value>, InterpreterError>,
{
    match function {
        Value::Closure { params, body, env } => {
            eval_closure_call_direct(params, body, env, args, eval_with_env)
        }
        // BuiltinFunction and NativeFunction variants not yet implemented
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot call non-function value of type {}",
            function.type_name()
        ))),
    }
}

/// Evaluate a closure call directly with `Value::Closure` fields
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
///
/// # [RUNTIME-001] Fix
/// Now checks recursion depth before entering function body
fn eval_closure_call_direct<F>(
    params: &[(String, Option<Arc<Expr>>)], // RUNTIME-DEFAULT-PARAMS: Support default parameters
    body: &Expr,
    env: &Rc<RefCell<HashMap<String, Value>>>, // ISSUE-119: Accept Rc<RefCell<HashMap>>
    args: &[Value],
    mut eval_with_env: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr, &HashMap<String, Value>) -> Result<Value, InterpreterError>,
{
    // [RUNTIME-001] CHECK RECURSION DEPTH BEFORE ENTERING
    check_recursion_depth()?;

    // Ensure depth is decremented on ALL exit paths
    let result = (|| {
        // RUNTIME-DEFAULT-PARAMS: Check argument count with default parameter support
        let required_count = params.iter().filter(|(_, default)| default.is_none()).count();
        let total_count = params.len();

        if args.len() < required_count || args.len() > total_count {
            return Err(InterpreterError::RuntimeError(format!(
                "Function expects {}-{} arguments, got {}",
                required_count,
                total_count,
                args.len()
            )));
        }

        // Create call environment with captured environment
        let mut call_env = env.borrow().clone(); // ISSUE-119: Borrow from RefCell then clone

        // RUNTIME-DEFAULT-PARAMS: Bind parameters to arguments + apply defaults for missing args
        for (i, (param_name, default_value)) in params.iter().enumerate() {
            let value = if i < args.len() {
                args[i].clone()
            } else if let Some(default_expr) = default_value {
                eval_with_env(default_expr, &call_env)?
            } else {
                unreachable!("Missing required parameter");
            };
            call_env.insert(param_name.clone(), value);
        }

        // Evaluate function body with bound environment
        // Catch InterpreterError::Return and extract the value (early return support)
        match eval_with_env(body, &call_env) {
            Err(InterpreterError::Return(val)) => Ok(val),
            other => other,
        }
    })();

    // [RUNTIME-001] ALWAYS DECREMENT, EVEN ON ERROR
    decrement_depth();

    result
}

/// Evaluate a closure call with parameter binding
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_closure_call<F>(
    closure: &Closure,
    args: &[Value],
    mut eval_with_env: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr, &HashMap<String, Value>) -> Result<Value, InterpreterError>,
{
    if args.len() != closure.params.len() {
        return Err(InterpreterError::RuntimeError(format!(
            "Function expects {} arguments, got {}",
            closure.params.len(),
            args.len()
        )));
    }

    // Start with captured environment
    let mut call_env = closure.captured_env.clone(); // Clone HashMap from internal Closure struct

    // Add self-reference for recursive functions
    if let Some(ref name) = closure.name {
        // RUNTIME-DEFAULT-PARAMS: Convert Closure to Value::Closure with None defaults
        let closure_value = Value::Closure {
            params: closure
                .params
                .iter()
                .map(|p| match p {
                    Pattern::Identifier(name) => (name.clone(), None),
                    _ => ("_".to_string(), None),
                })
                .collect(),
            body: Arc::new(closure.body.clone()),
            env: Rc::new(RefCell::new(closure.captured_env.clone())), // ISSUE-119: Wrap HashMap in Rc<RefCell>
        };
        call_env.insert(name.clone(), closure_value);
    }

    // Bind parameters to arguments using pattern matching
    for (param, arg) in closure.params.iter().zip(args.iter()) {
        bind_parameter(param, arg, &mut call_env)?;
    }

    // Evaluate function body with bound environment
    // Catch InterpreterError::Return and extract the value (early return support)
    match eval_with_env(&closure.body, &call_env) {
        Err(InterpreterError::Return(val)) => Ok(val),
        other => other,
    }
}

/// Bind a parameter pattern to an argument value
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn bind_parameter(
    pattern: &Pattern,
    value: &Value,
    env: &mut HashMap<String, Value>,
) -> Result<(), InterpreterError> {
    let match_result = match_pattern(pattern, value)?;
    if !match_result.matches {
        return Err(InterpreterError::RuntimeError(
            "Parameter pattern does not match argument value".to_string(),
        ));
    }

    // Add all bindings from pattern match to environment
    env.extend(match_result.bindings);
    Ok(())
}

/// Evaluate a method call on a value
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
pub fn eval_method_call_value<F1, F2>(
    receiver: &Value,
    method: &str,
    args: &[Value],
    eval_function_call_value: F1,
    mut eval_method_dispatch: F2,
) -> Result<Value, InterpreterError>
where
    F1: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
    F2: FnMut(&Value, &str, &[Value], bool) -> Result<Value, InterpreterError>,
{
    // Try method dispatch first (type-specific methods)
    match eval_method_dispatch(receiver, method, args, args.is_empty()) {
        Ok(result) => Ok(result),
        Err(InterpreterError::RuntimeError(msg)) if msg.contains("not found") => {
            // Fallback to function call if method exists as function
            try_method_as_function(receiver, method, args, eval_function_call_value)
        }
        Err(e) => Err(e),
    }
}

/// Try to call a method as a function with receiver as first argument
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn try_method_as_function<F>(
    receiver: &Value,
    method: &str,
    args: &[Value],
    mut eval_function_call_value: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    // Check if method exists as a function in environment
    if let Ok(function_value) = get_function_from_env(method) {
        let mut all_args = vec![receiver.clone()];
        all_args.extend_from_slice(args);
        eval_function_call_value(&function_value, &all_args)
    } else {
        Err(InterpreterError::RuntimeError(format!(
            "Method '{}' not found for type {}",
            method,
            receiver.type_name()
        )))
    }
}

/// Get a function value from the current environment
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn get_function_from_env(name: &str) -> Result<Value, InterpreterError> {
    // This would normally look up in the current environment
    // For now, return error to indicate function not found
    Err(InterpreterError::RuntimeError(format!(
        "Function '{name}' not found in environment"
    )))
}

/// Create a partial application of a function
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn create_partial_application(
    function: &Value,
    partial_args: &[Value],
) -> Result<Value, InterpreterError> {
    match function {
        Value::Closure { params, body, env } => {
            // RUNTIME-DEFAULT-PARAMS: Convert to internal Closure structure for partial application
            let closure = Closure {
                params: params
                    .iter()
                    .map(|(param_name, _default_value)| Pattern::Identifier(param_name.clone()))
                    .collect(),
                body: body.as_ref().clone(),
                captured_env: env.borrow().clone(), // ISSUE-119: Borrow from RefCell then clone HashMap
                name: None,
            };
            if partial_args.len() >= closure.params.len() {
                return Err(InterpreterError::RuntimeError(
                    "Cannot partially apply more arguments than function parameters".to_string(),
                ));
            }

            // Create new closure with pre-bound parameters
            let remaining_params = closure.params[partial_args.len()..].to_vec();
            let mut new_captured_env = closure.captured_env.clone();

            // Bind partial arguments to parameters
            for (param, arg) in closure.params.iter().zip(partial_args.iter()) {
                bind_parameter(param, arg, &mut new_captured_env)?;
            }

            let partial_closure =
                Closure::new(remaining_params, closure.body.clone(), new_captured_env);

            // RUNTIME-DEFAULT-PARAMS: Convert Closure to Value::Closure with None defaults
            Ok(Value::Closure {
                params: partial_closure
                    .params
                    .iter()
                    .map(|p| match p {
                        Pattern::Identifier(name) => (name.clone(), None),
                        _ => ("_".to_string(), None),
                    })
                    .collect(),
                body: Arc::new(partial_closure.body),
                env: Rc::new(RefCell::new(partial_closure.captured_env)), // ISSUE-119: Wrap HashMap in Rc<RefCell>
            })
        }
        _ => Err(InterpreterError::TypeError(
            "Cannot create partial application of non-function".to_string(),
        )),
    }
}

/// Evaluate function composition (f ∘ g)
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_function_composition(f: &Value, g: &Value) -> Result<Value, InterpreterError> {
    match (f, g) {
        (
            Value::Closure {
                params: f_params,
                body: _f_body,
                env: _f_env,
            },
            Value::Closure {
                params: g_params,
                body: g_body,
                env: g_env,
            },
        ) => {
            // Create composed function: f(g(x))
            if g_params.len() != 1 {
                return Err(InterpreterError::RuntimeError(
                    "Function composition requires second function to have exactly one parameter"
                        .to_string(),
                ));
            }

            if f_params.len() != 1 {
                return Err(InterpreterError::RuntimeError(
                    "Function composition requires first function to have exactly one parameter"
                        .to_string(),
                ));
            }

            // Create simple composition placeholder
            Ok(Value::Closure {
                params: g_params.clone(),
                body: g_body.clone(), // Simplified composition
                env: g_env.clone(),
            })
        }
        _ => Err(InterpreterError::TypeError(
            "Function composition requires two functions".to_string(),
        )),
    }
}

/// Create a closure representing function composition
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn create_composition_closure(f: &Closure, g: &Closure) -> Result<Closure, InterpreterError> {
    // The composed function takes g's parameter and applies f(g(x))
    let param = g.params[0].clone();

    // Create body that represents f(g(x))
    // This is simplified - in practice would need to construct proper AST
    let composed_body = Expr::new(
        crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Unit),
        crate::frontend::ast::Span::new(0, 0),
    );

    // Combine captured environments
    let mut combined_env = g.captured_env.clone();
    combined_env.extend(f.captured_env.clone());

    Ok(Closure::new(vec![param], composed_body, combined_env))
}

/// Check if a value is callable (function, builtin, etc.)
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn is_callable(value: &Value) -> bool {
    matches!(value, Value::Closure { .. })
}

/// Get arity (number of parameters) of a callable value
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn get_arity(value: &Value) -> Result<usize, InterpreterError> {
    match value {
        Value::Closure { params, .. } => Ok(params.len()),
        // BuiltinFunction, NativeFunction, Method variants not yet implemented
        _ => Err(InterpreterError::TypeError(
            "Cannot get arity of non-callable value".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    #[test]
    fn test_closure_creation() {
        let params = vec![Pattern::Identifier("x".to_string())];
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(3, 5),
        );
        let env = HashMap::new();

        let closure = Closure::new(params, body, env);
        assert_eq!(closure.params.len(), 1);
        assert!(closure.name.is_none());
    }

    #[test]
    fn test_named_closure_creation() {
        let params = vec![Pattern::Identifier("x".to_string())];
        let body = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(3, 5),
        );
        let env = HashMap::new();

        let closure = Closure::named(params, body, env, "factorial".to_string());
        assert_eq!(closure.params.len(), 1);
        assert_eq!(closure.name, Some("factorial".to_string()));
    }

    #[test]
    fn test_parameter_binding() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Integer(42);
        let mut env = HashMap::new();

        bind_parameter(&pattern, &value, &mut env).unwrap();
        assert_eq!(env.get("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_is_callable() {
        let _closure = Closure::new(
            vec![],
            Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0)),
            HashMap::new(),
        );
        let function_value = Value::Closure {
            params: vec![],
            body: Arc::new(Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        assert!(is_callable(&function_value));

        let non_callable = Value::Integer(42);
        assert!(!is_callable(&non_callable));
    }

    #[test]
    fn test_get_arity() {
        let _params = [
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
        ];
        // RUNTIME-DEFAULT-PARAMS: Test closure with tuple format
        let function_value = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: Arc::new(Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        assert_eq!(get_arity(&function_value).unwrap(), 2);

        let non_callable = Value::Integer(42);
        assert!(get_arity(&non_callable).is_err());
    }
}
