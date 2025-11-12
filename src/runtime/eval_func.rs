//! Function and lambda evaluation module
//!
//! This module handles function definition, lambda creation, and function calls.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Param};
use crate::runtime::{InterpreterError, Value};
use std::cell::RefCell; // ISSUE-119: Added for RefCell
use std::collections::HashMap;
use std::rc::Rc; // ISSUE-119: Added for Rc
use std::sync::Arc;

/// Evaluate a function definition
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_function(
    name: &str,
    params: &[Param],
    body: &Expr,
    current_env_ref: &Rc<RefCell<HashMap<String, Value>>>, // ISSUE-119: Changed from &HashMap
    mut env_set: impl FnMut(String, Value),
) -> Result<Value, InterpreterError> {
    // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
    let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
        .iter()
        .map(|p| (p.name(), p.default_value.clone().map(|expr| Arc::new((*expr).clone()))))
        .collect();

    let closure = Value::Closure {
        params: params_with_defaults,
        body: Arc::new(body.clone()),
        env: current_env_ref.clone(), // ISSUE-119: ROOT CAUSE #1 FIX - Rc::clone (shallow copy, no data clone)
    };

    // Bind function name in environment for recursion
    env_set(name.to_string(), closure.clone());
    Ok(closure)
}

/// Evaluate a lambda expression
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
pub fn eval_lambda(
    params: &[Param],
    body: &Expr,
    current_env_ref: &Rc<RefCell<HashMap<String, Value>>>, // ISSUE-119: Changed from &HashMap
) -> Result<Value, InterpreterError> {
    // RUNTIME-DEFAULT-PARAMS: Extract both param names AND default values
    let params_with_defaults: Vec<(String, Option<Arc<Expr>>)> = params
        .iter()
        .map(|p| (p.name(), p.default_value.clone().map(|expr| Arc::new((*expr).clone()))))
        .collect();

    let closure = Value::Closure {
        params: params_with_defaults,
        body: Arc::new(body.clone()),
        env: current_env_ref.clone(), // ISSUE-119: ROOT CAUSE #1 FIX - Rc::clone for lambdas
    };

    Ok(closure)
}

/// Evaluate a function call
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_function_call<F>(
    func: &Expr,
    args: &[Expr],
    mut eval_expr: F,
    mut call_function: impl FnMut(Value, &[Value]) -> Result<Value, InterpreterError>,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let func_val = eval_expr(func)?;
    let arg_vals: Result<Vec<Value>, InterpreterError> = args.iter().map(eval_expr).collect();
    let arg_vals = arg_vals?;

    call_function(func_val, &arg_vals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_lambda() {
        let params = vec![];
        let body = Expr::new(
            crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
                42, None,
            )),
            crate::frontend::ast::Span::default(),
        );
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 0, "Expected 0 params");
                // Body should be the literal expression
            }
            _ => panic!("Expected closure value"),
        }
    }
}
