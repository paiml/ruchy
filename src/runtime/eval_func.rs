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
        .map(|p| {
            (
                p.name(),
                p.default_value
                    .clone()
                    .map(|expr| Arc::new((*expr).clone())),
            )
        })
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
        .map(|p| {
            (
                p.name(),
                p.default_value
                    .clone()
                    .map(|expr| Arc::new((*expr).clone())),
            )
        })
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
    use crate::frontend::ast::{ExprKind, Literal, Pattern, Span, Type, TypeKind};

    fn make_literal_expr(n: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(n, None)), Span::default())
    }

    fn make_param(name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named(String::new()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn make_param_with_default(name: &str, default: i64) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named(String::new()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(make_literal_expr(default))),
        }
    }

    #[test]
    fn test_eval_lambda() {
        let params = vec![];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 0, "Expected 0 params");
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_lambda_with_params() {
        let params = vec![make_param("x"), make_param("y")];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].0, "x");
                assert_eq!(params[1].0, "y");
                assert!(params[0].1.is_none());
                assert!(params[1].1.is_none());
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_lambda_with_default_params() {
        let params = vec![make_param("x"), make_param_with_default("y", 10)];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 2);
                assert!(params[0].1.is_none());
                assert!(params[1].1.is_some());
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function() {
        let params = vec![make_param("a"), make_param("b")];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name: Option<String> = None;

        let result = eval_function("add", &params, &body, &env, |name, _value| {
            bound_name = Some(name);
        })
        .unwrap();

        assert_eq!(bound_name, Some("add".to_string()));
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 2);
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_with_defaults() {
        let params = vec![
            make_param("a"),
            make_param_with_default("b", 5),
            make_param_with_default("c", 10),
        ];
        let body = make_literal_expr(100);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("test_fn", &params, &body, &env, |_, _| {}).unwrap();

        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 3);
                assert!(params[0].1.is_none());
                assert!(params[1].1.is_some());
                assert!(params[2].1.is_some());
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_call() {
        let func_expr = make_literal_expr(0); // placeholder
        let args = vec![make_literal_expr(1), make_literal_expr(2)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_expr| Ok(Value::Integer(42)),
            |_func, arg_vals| {
                assert_eq!(arg_vals.len(), 2);
                Ok(Value::Integer(100))
            },
        )
        .unwrap();

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_eval_function_call_no_args() {
        let func_expr = make_literal_expr(0);
        let args: Vec<Expr> = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, arg_vals| {
                assert!(arg_vals.is_empty());
                Ok(Value::Integer(999))
            },
        )
        .unwrap();

        assert_eq!(result, Value::Integer(999));
    }

    #[test]
    fn test_eval_function_call_eval_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_expr| Err(InterpreterError::RuntimeError("eval error".to_string())),
            |_, _| Ok(Value::Integer(0)),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_function_call_call_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(42)),
            |_, _| Err(InterpreterError::RuntimeError("call error".to_string())),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_lambda_captures_environment() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(42);
        let mut env_map = HashMap::new();
        env_map.insert("outer_var".to_string(), Value::Integer(100));
        let env = Rc::new(RefCell::new(env_map));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured_env, .. } => {
                let borrowed = captured_env.borrow();
                assert!(borrowed.contains_key("outer_var"));
                assert_eq!(borrowed.get("outer_var"), Some(&Value::Integer(100)));
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_closure_captures_env() {
        let params = vec![make_param("a")];
        let body = make_literal_expr(42);
        let mut env_map = HashMap::new();
        env_map.insert("captured".to_string(), Value::String("hello".into()));
        let env = Rc::new(RefCell::new(env_map));

        let result = eval_function("my_fn", &params, &body, &env, |_, _| {}).unwrap();

        match result {
            Value::Closure { env: captured_env, .. } => {
                let borrowed = captured_env.borrow();
                assert!(borrowed.contains_key("captured"));
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_lambda_empty_body() {
        let params = vec![];
        // Use unit as empty body
        let body = Expr::new(ExprKind::Literal(Literal::Unit), Span::default());
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { body, .. } => {
                // Verify body is stored
                match &body.kind {
                    ExprKind::Literal(Literal::Unit) => {}
                    _ => panic!("Expected Unit literal body"),
                }
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_empty_params() {
        let params = vec![];
        let body = make_literal_expr(1);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound = false;

        let result = eval_function("no_params", &params, &body, &env, |_, _| {
            bound = true;
        })
        .unwrap();

        assert!(bound);
        match result {
            Value::Closure { params, .. } => {
                assert!(params.is_empty());
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_call_multiple_args() {
        let func_expr = make_literal_expr(0);
        let args = vec![
            make_literal_expr(1),
            make_literal_expr(2),
            make_literal_expr(3),
            make_literal_expr(4),
        ];

        let mut arg_count = 0;
        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, arg_vals| {
                arg_count = arg_vals.len();
                Ok(Value::Integer(arg_count as i64))
            },
        )
        .unwrap();

        assert_eq!(arg_count, 4);
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_eval_lambda_all_default_params() {
        let params = vec![
            make_param_with_default("a", 1),
            make_param_with_default("b", 2),
            make_param_with_default("c", 3),
        ];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 3);
                assert!(params.iter().all(|(_, default)| default.is_some()));
            }
            _ => panic!("Expected closure value"),
        }
    }
}
