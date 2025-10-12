//! Function and lambda evaluation module
//!
//! This module handles function definition, lambda creation, and function calls.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, Param};
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Evaluate a function definition
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_function(
    name: &str,
    params: &[Param],
    body: &Expr,
    current_env: &HashMap<String, Value>,
    mut env_set: impl FnMut(String, Value),
) -> Result<Value, InterpreterError> {
    let param_names: Vec<String> = params
        .iter()
        .map(super::super::frontend::ast::Param::name)
        .collect();

    let closure = Value::Closure {
        params: param_names,
        body: Arc::new(body.clone()),
        env: Arc::new(current_env.clone()),
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
    current_env: &HashMap<String, Value>,
) -> Result<Value, InterpreterError> {
    let param_names: Vec<String> = params
        .iter()
        .map(super::super::frontend::ast::Param::name)
        .collect();

    let closure = Value::Closure {
        params: param_names,
        body: Arc::new(body.clone()),
        env: Arc::new(current_env.clone()),
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
        let env = HashMap::new();

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 0);
                // Body should be the literal expression
            }
            _ => panic!("Expected closure value"),
        }
    }
}
