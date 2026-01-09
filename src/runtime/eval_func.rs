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

    // ============================================================================
    // EXTREME TDD Round 132: Additional comprehensive tests
    // Target: 15 → 40+ tests
    // ============================================================================

    // --- Lambda edge cases ---
    #[test]
    fn test_eval_lambda_single_param() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(1);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].0, "x");
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_many_params() {
        let params: Vec<Param> = (0..10).map(|i| make_param(&format!("p{}", i))).collect();
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 10);
                for i in 0..10 {
                    assert_eq!(params[i].0, format!("p{}", i));
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_mixed_default_params() {
        let params = vec![
            make_param("a"),
            make_param_with_default("b", 10),
            make_param("c"),
            make_param_with_default("d", 20),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert!(params[0].1.is_none());
                assert!(params[1].1.is_some());
                assert!(params[2].1.is_none());
                assert!(params[3].1.is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_preserves_body() {
        let params = vec![];
        let body = Expr::new(ExprKind::Literal(Literal::String("test".to_string())), Span::default());
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { body, .. } => {
                match &body.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "test"),
                    _ => panic!("Expected string literal body"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_with_float_default() {
        let params = vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type { kind: TypeKind::Named(String::new()), span: Span::default() },
            span: Span::default(),
            is_mutable: false,
            default_value: Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Float(3.14)),
                Span::default(),
            ))),
        }];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert!(params[0].1.is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function definition edge cases ---
    #[test]
    fn test_eval_function_binds_to_env() {
        let params = vec![];
        let body = make_literal_expr(42);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_value: Option<Value> = None;

        let _ = eval_function("my_func", &params, &body, &env, |name, value| {
            assert_eq!(name, "my_func");
            bound_value = Some(value);
        });

        assert!(bound_value.is_some());
    }

    #[test]
    fn test_eval_function_returns_same_as_bound() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(100);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_value: Option<Value> = None;

        let result = eval_function("test", &params, &body, &env, |_, value| {
            bound_value = Some(value);
        }).unwrap();

        // The returned closure should match the bound one
        match (&result, &bound_value) {
            (Value::Closure { params: p1, .. }, Some(Value::Closure { params: p2, .. })) => {
                assert_eq!(p1.len(), p2.len());
            }
            _ => panic!("Expected matching closures"),
        }
    }

    #[test]
    fn test_eval_function_with_complex_body() {
        let params = vec![make_param("x")];
        let body = Expr::new(
            ExprKind::Block(vec![make_literal_expr(1), make_literal_expr(2)]),
            Span::default(),
        );
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("complex", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { body, .. } => {
                match &body.kind {
                    ExprKind::Block(expressions) => {
                        assert_eq!(expressions.len(), 2);
                    }
                    _ => panic!("Expected block body"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_long_name() {
        let long_name = "a".repeat(100);
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name = String::new();

        let _ = eval_function(&long_name, &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert_eq!(bound_name.len(), 100);
    }

    #[test]
    fn test_eval_function_unicode_name() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name = String::new();

        let _ = eval_function("函数", &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert_eq!(bound_name, "函数");
    }

    #[test]
    fn test_eval_function_special_chars_name() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name = String::new();

        let _ = eval_function("my_func_123", &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert_eq!(bound_name, "my_func_123");
    }

    // --- Environment capture tests ---
    #[test]
    fn test_eval_lambda_env_with_multiple_types() {
        let params = vec![];
        let body = make_literal_expr(0);
        let mut env_map = HashMap::new();
        env_map.insert("int_val".to_string(), Value::Integer(42));
        env_map.insert("float_val".to_string(), Value::Float(3.14));
        env_map.insert("bool_val".to_string(), Value::Bool(true));
        env_map.insert("str_val".to_string(), Value::from_string("hello".to_string()));
        let env = Rc::new(RefCell::new(env_map));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                let borrowed = captured.borrow();
                assert_eq!(borrowed.len(), 4);
                assert!(borrowed.contains_key("int_val"));
                assert!(borrowed.contains_key("float_val"));
                assert!(borrowed.contains_key("bool_val"));
                assert!(borrowed.contains_key("str_val"));
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_empty_env() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                assert!(captured.borrow().is_empty());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_env_shared_reference() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("test", &params, &body, &env, |_, _| {}).unwrap();

        // Modify original env after function creation
        env.borrow_mut().insert("new_key".to_string(), Value::Integer(999));

        // Closure should see the update (shared reference)
        match result {
            Value::Closure { env: captured, .. } => {
                assert!(captured.borrow().contains_key("new_key"));
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function call tests ---
    #[test]
    fn test_eval_function_call_evaluates_func_first() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1)];
        let call_order = RefCell::new(vec![]);

        let _ = eval_function_call(
            &func_expr,
            &args,
            |_| {
                call_order.borrow_mut().push("eval");
                Ok(Value::Integer(0))
            },
            |_, _| {
                call_order.borrow_mut().push("call");
                Ok(Value::Integer(0))
            },
        );

        assert_eq!(*call_order.borrow(), vec!["eval", "eval", "call"]);
    }

    #[test]
    fn test_eval_function_call_arg_error_stops_evaluation() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1), make_literal_expr(2)];
        let eval_count = RefCell::new(0);

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| {
                let mut count = eval_count.borrow_mut();
                *count += 1;
                if *count == 2 {
                    Err(InterpreterError::RuntimeError("arg error".to_string()))
                } else {
                    Ok(Value::Integer(0))
                }
            },
            |_, _| Ok(Value::Integer(0)),
        );

        assert!(result.is_err());
        assert_eq!(*eval_count.borrow(), 2);
    }

    #[test]
    fn test_eval_function_call_passes_evaluated_values() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(10), make_literal_expr(20)];
        let received_args: RefCell<Vec<Value>> = RefCell::new(vec![]);

        let _ = eval_function_call(
            &func_expr,
            &args,
            |expr| {
                match &expr.kind {
                    ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n * 2)),
                    _ => Ok(Value::Integer(0)),
                }
            },
            |_, arg_vals| {
                *received_args.borrow_mut() = arg_vals.to_vec();
                Ok(Value::Integer(0))
            },
        );

        // Args should be doubled by eval
        assert_eq!(*received_args.borrow(), vec![Value::Integer(20), Value::Integer(40)]);
    }

    #[test]
    fn test_eval_function_call_returns_call_result() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::from_string("result".to_string())),
        ).unwrap();

        assert_eq!(result, Value::from_string("result".to_string()));
    }

    #[test]
    fn test_eval_function_call_func_value_passed() {
        let func_expr = make_literal_expr(42);
        let args = vec![];
        let mut received_func: Option<Value> = None;

        let _ = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(42)),
            |func, _| {
                received_func = Some(func);
                Ok(Value::Integer(0))
            },
        );

        assert_eq!(received_func, Some(Value::Integer(42)));
    }

    // --- Param pattern tests ---
    #[test]
    fn test_eval_lambda_preserves_param_order() {
        let params = vec![
            make_param("z"),
            make_param("y"),
            make_param("x"),
            make_param("w"),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params[0].0, "z");
                assert_eq!(params[1].0, "y");
                assert_eq!(params[2].0, "x");
                assert_eq!(params[3].0, "w");
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_param_names_extracted() {
        let params = vec![
            make_param("first"),
            make_param("second"),
            make_param("third"),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("f", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params, .. } => {
                let names: Vec<&str> = params.iter().map(|(n, _)| n.as_str()).collect();
                assert_eq!(names, vec!["first", "second", "third"]);
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_default_value_expression_stored() {
        let params = vec![make_param_with_default("x", 999)];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                let default_expr = params[0].1.as_ref().unwrap();
                match &default_expr.kind {
                    ExprKind::Literal(Literal::Integer(n, _)) => assert_eq!(*n, 999),
                    _ => panic!("Expected integer literal default"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_with_bool_body() {
        let params = vec![];
        let body = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("bool_fn", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { body, .. } => {
                match &body.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(*b),
                    _ => panic!("Expected bool literal body"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_with_nil_in_env() {
        let params = vec![];
        let body = make_literal_expr(0);
        let mut env_map = HashMap::new();
        env_map.insert("nil_val".to_string(), Value::Nil);
        let env = Rc::new(RefCell::new(env_map));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                assert_eq!(captured.borrow().get("nil_val"), Some(&Value::Nil));
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_call_single_arg() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(42)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(42)),
            |_, arg_vals| {
                assert_eq!(arg_vals.len(), 1);
                Ok(arg_vals[0].clone())
            },
        ).unwrap();

        assert_eq!(result, Value::Integer(42));
    }
}

// ============================================================================
// EXTREME TDD Round 134: Additional comprehensive tests
// Target: 40 → 60+ tests
// ============================================================================
#[cfg(test)]
mod round_134_tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind};
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn make_literal_expr(val: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(val, None)), Span::default())
    }

    fn make_string_literal_expr(s: &str) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::String(s.to_string())),
            Span::default(),
        )
    }

    fn make_bool_literal_expr(b: bool) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Bool(b)), Span::default())
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

    // --- Lambda with different body types ---
    #[test]
    fn test_eval_lambda_with_string_body() {
        let params = vec![];
        let body = make_string_literal_expr("hello");
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { body, .. } => {
                match &body.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string literal body"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_with_bool_body() {
        let params = vec![];
        let body = make_bool_literal_expr(false);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { body, .. } => {
                match &body.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(!*b),
                    _ => panic!("Expected bool literal body"),
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function with many params ---
    #[test]
    fn test_eval_function_with_10_params() {
        let params: Vec<Param> = (0..10).map(|i| make_param(&format!("p{}", i))).collect();
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("multi_param", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 10);
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_with_10_params() {
        let params: Vec<Param> = (0..10).map(|i| make_param(&format!("arg{}", i))).collect();
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 10);
                for i in 0..10 {
                    assert_eq!(params[i].0, format!("arg{}", i));
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function with different default values ---
    #[test]
    fn test_eval_function_mixed_default_params() {
        let params = vec![
            make_param("a"),
            make_param_with_default("b", 10),
            make_param("c"),
            make_param_with_default("d", 20),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("mixed", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert!(params[0].1.is_none());
                assert!(params[1].1.is_some());
                assert!(params[2].1.is_none());
                assert!(params[3].1.is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function call with many arguments ---
    #[test]
    fn test_eval_function_call_many_args() {
        let func_expr = make_literal_expr(0);
        let args: Vec<Expr> = (1..=5).map(|i| make_literal_expr(i)).collect();
        let received_count = RefCell::new(0);

        let _ = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, arg_vals| {
                *received_count.borrow_mut() = arg_vals.len();
                Ok(Value::Integer(0))
            },
        );

        assert_eq!(*received_count.borrow(), 5);
    }

    #[test]
    fn test_eval_function_call_empty_args() {
        let func_expr = make_literal_expr(0);
        let args: Vec<Expr> = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(42)),
            |_, arg_vals| {
                assert!(arg_vals.is_empty());
                Ok(Value::from_string("no args".to_string()))
            },
        ).unwrap();

        assert_eq!(result, Value::from_string("no args".to_string()));
    }

    // --- Environment with complex values ---
    #[test]
    fn test_eval_lambda_env_with_array() {
        let params = vec![];
        let body = make_literal_expr(0);
        let mut env_map = HashMap::new();
        env_map.insert("arr".to_string(), Value::from_array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let env = Rc::new(RefCell::new(env_map));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                let borrowed = captured.borrow();
                assert!(borrowed.contains_key("arr"));
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function call error propagation ---
    #[test]
    fn test_eval_function_call_call_fn_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Err(InterpreterError::RuntimeError("call error".to_string())),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("call error"));
    }

    #[test]
    fn test_eval_function_call_func_eval_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![];
        let eval_count = RefCell::new(0);

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| {
                *eval_count.borrow_mut() += 1;
                if *eval_count.borrow() == 1 {
                    Err(InterpreterError::RuntimeError("func eval error".to_string()))
                } else {
                    Ok(Value::Integer(0))
                }
            },
            |_, _| Ok(Value::Integer(0)),
        );

        assert!(result.is_err());
        assert_eq!(*eval_count.borrow(), 1); // Only function evaluated
    }

    // --- Param name edge cases ---
    #[test]
    fn test_eval_lambda_single_char_params() {
        let params = vec![
            make_param("a"),
            make_param("b"),
            make_param("c"),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params[0].0, "a");
                assert_eq!(params[1].0, "b");
                assert_eq!(params[2].0, "c");
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_underscore_param() {
        let params = vec![make_param("_")];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("with_underscore", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params[0].0, "_");
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function/lambda result types ---
    #[test]
    fn test_eval_lambda_returns_closure_type() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_eval_function_returns_closure_type() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("fn", &params, &body, &env, |_, _| {}).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    // --- Call with different value types ---
    #[test]
    fn test_eval_function_call_returns_float() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::Float(3.14)),
        ).unwrap();

        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_function_call_returns_bool() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::Bool(true)),
        ).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_function_call_returns_nil() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::Nil),
        ).unwrap();

        assert_eq!(result, Value::Nil);
    }

    // --- Environment mutation after capture ---
    #[test]
    fn test_eval_lambda_env_mutation_visible() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        env.borrow_mut().insert("x".to_string(), Value::Integer(1));

        let result = eval_lambda(&params, &body, &env).unwrap();

        // Mutate original env
        env.borrow_mut().insert("y".to_string(), Value::Integer(2));

        match result {
            Value::Closure { env: captured, .. } => {
                let borrowed = captured.borrow();
                assert!(borrowed.contains_key("x"));
                assert!(borrowed.contains_key("y")); // Should see mutation
            }
            _ => panic!("Expected closure"),
        }
    }

    // --- Function call argument order ---
    #[test]
    fn test_eval_function_call_preserves_arg_order() {
        let func_expr = make_literal_expr(0);
        let args = vec![
            make_literal_expr(1),
            make_literal_expr(2),
            make_literal_expr(3),
        ];
        let eval_order = RefCell::new(vec![]);

        let _ = eval_function_call(
            &func_expr,
            &args,
            |expr| {
                if let ExprKind::Literal(Literal::Integer(n, _)) = &expr.kind {
                    eval_order.borrow_mut().push(*n);
                }
                Ok(Value::Integer(0))
            },
            |_, _| Ok(Value::Integer(0)),
        );

        // First eval is for func, then args in order
        let order = eval_order.borrow();
        assert_eq!(order.len(), 4); // func + 3 args
        assert_eq!(order[1], 1);
        assert_eq!(order[2], 2);
        assert_eq!(order[3], 3);
    }

    // --- Empty function name ---
    #[test]
    fn test_eval_function_empty_name() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name = String::from("not_empty");

        let _ = eval_function("", &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert!(bound_name.is_empty());
    }

    // === EXTREME TDD Round 137 - Push to 75+ Tests ===

    #[test]
    fn test_eval_lambda_single_param_with_default() {
        let params = vec![make_param_with_default("x", 42)];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 1);
                assert!(p[0].1.is_some()); // Default is present
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_multiple_params_with_defaults() {
        let params = vec![
            make_param("a"),
            make_param_with_default("b", 10),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("multi", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 2);
                assert!(p[0].1.is_none());
                assert!(p[1].1.is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_env_with_multiple_vars() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        env.borrow_mut().insert("a".to_string(), Value::Integer(1));
        env.borrow_mut().insert("b".to_string(), Value::Integer(2));
        env.borrow_mut().insert("c".to_string(), Value::Integer(3));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                let borrowed = captured.borrow();
                assert_eq!(borrowed.len(), 3);
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_call_single_arg() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(42)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, args| {
                assert_eq!(args.len(), 1);
                Ok(Value::Integer(100))
            },
        ).unwrap();

        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_eval_function_call_ten_args() {
        let func_expr = make_literal_expr(0);
        let args: Vec<_> = (0..10).map(|i| make_literal_expr(i)).collect();

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, args| {
                assert_eq!(args.len(), 10);
                Ok(Value::Integer(args.len() as i64))
            },
        ).unwrap();

        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_eval_function_call_returns_string() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::from_string("hello".to_string())),
        ).unwrap();

        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_function_call_returns_array() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Ok(Value::Integer(0)),
            |_, _| Ok(Value::Array(std::sync::Arc::from(vec![Value::Integer(1), Value::Integer(2)]))),
        ).unwrap();

        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_lambda_body_is_captured() {
        let params = vec![];
        let body = Expr::new(
            ExprKind::Identifier("captured_var".to_string()),
            Span::default(),
        );
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { body: b, .. } => {
                if let ExprKind::Identifier(name) = &b.kind {
                    assert_eq!(name, "captured_var");
                } else {
                    panic!("Body should be identifier");
                }
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_special_chars_in_name() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound_name = String::new();

        let _ = eval_function("my_func_123", &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert_eq!(bound_name, "my_func_123");
    }

    #[test]
    fn test_eval_function_long_name() {
        let params = vec![];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let long_name = "a".repeat(100);
        let mut bound_name = String::new();

        let _ = eval_function(&long_name, &params, &body, &env, |name, _| {
            bound_name = name;
        });

        assert_eq!(bound_name.len(), 100);
    }

    #[test]
    fn test_eval_lambda_empty_env() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: captured, .. } => {
                assert!(captured.borrow().is_empty());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_call_arg_evaluation_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![make_literal_expr(1)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| {
                if let ExprKind::Literal(Literal::Integer(1, _)) = &expr.kind {
                    Err(InterpreterError::RuntimeError("arg error".to_string()))
                } else {
                    Ok(Value::Integer(0))
                }
            },
            |_, _| Ok(Value::Integer(0)),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_function_call_func_evaluation_error() {
        let func_expr = make_literal_expr(0);
        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |_| Err(InterpreterError::RuntimeError("func error".to_string())),
            |_, _| Ok(Value::Integer(0)),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_function_three_params() {
        let params = vec![
            make_param("x"),
            make_param("y"),
            make_param("z"),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("triple", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 3);
                assert_eq!(p[0].0, "x");
                assert_eq!(p[1].0, "y");
                assert_eq!(p[2].0, "z");
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_five_params() {
        let params = vec![
            make_param("a"),
            make_param("b"),
            make_param("c"),
            make_param("d"),
            make_param("e"),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 5);
            }
            _ => panic!("Expected closure"),
        }
    }

    // === EXTREME TDD Round 159 - Coverage Push Tests ===

    #[test]
    fn test_eval_function_no_params_r159() {
        let params = vec![];
        let body = make_literal_expr(100);
        let env = Rc::new(RefCell::new(HashMap::new()));
        let mut bound: Option<(String, Value)> = None;

        let result = eval_function("constant", &params, &body, &env, |name, val| {
            bound = Some((name, val));
        }).unwrap();

        assert!(bound.is_some());
        match result {
            Value::Closure { params: p, .. } => assert_eq!(p.len(), 0),
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_single_param_r159() {
        let params = vec![make_param("x")];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("identity", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].0, "x");
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_empty_body_r159() {
        let params = vec![];
        let body = Expr::new(ExprKind::Literal(Literal::Unit), Span::default());
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        assert!(matches!(result, Value::Closure { .. }));
    }

    #[test]
    fn test_eval_lambda_all_default_params_r159() {
        let params = vec![
            make_param_with_default("a", 1),
            make_param_with_default("b", 2),
            make_param_with_default("c", 3),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 3);
                assert!(p[0].1.is_some());
                assert!(p[1].1.is_some());
                assert!(p[2].1.is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_with_env_r159() {
        let mut env_map = HashMap::new();
        env_map.insert("existing".to_string(), Value::Integer(42));
        let env = Rc::new(RefCell::new(env_map));

        let params = vec![make_param("x")];
        let body = make_literal_expr(0);

        let result = eval_function("test_fn", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { env: closure_env, .. } => {
                // Closure should capture the environment
                assert!(closure_env.borrow().get("existing").is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_lambda_captures_env_r159() {
        let mut env_map = HashMap::new();
        env_map.insert("captured".to_string(), Value::from_string("value".to_string()));
        let env = Rc::new(RefCell::new(env_map));

        let params = vec![];
        let body = make_literal_expr(0);

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { env: closure_env, .. } => {
                assert!(closure_env.borrow().get("captured").is_some());
            }
            _ => panic!("Expected closure"),
        }
    }

    #[test]
    fn test_eval_function_call_success_r159() {
        let func_expr = Expr::new(ExprKind::Identifier("my_fn".to_string()), Span::default());
        let args = vec![make_literal_expr(10)];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| match &expr.kind {
                ExprKind::Identifier(_) => Ok(Value::Closure {
                    params: vec![("x".to_string(), None)],
                    body: Arc::new(make_literal_expr(0)),
                    env: Rc::new(RefCell::new(HashMap::new())),
                }),
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Nil),
            },
            |_func, args| {
                // Simple function that returns the first argument
                Ok(args.get(0).cloned().unwrap_or(Value::Nil))
            },
        ).unwrap();

        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_eval_function_mixed_params_r159() {
        let params = vec![
            make_param("required"),
            make_param_with_default("optional", 99),
        ];
        let body = make_literal_expr(0);
        let env = Rc::new(RefCell::new(HashMap::new()));

        let result = eval_function("mixed", &params, &body, &env, |_, _| {}).unwrap();
        match result {
            Value::Closure { params: p, .. } => {
                assert_eq!(p.len(), 2);
                assert!(p[0].1.is_none()); // required - no default
                assert!(p[1].1.is_some()); // optional - has default
            }
            _ => panic!("Expected closure"),
        }
    }
}
