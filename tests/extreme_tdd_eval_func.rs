// EXTREME TDD: eval_func Module Coverage Tests
// Target: src/runtime/eval_func.rs - Improve coverage from 48.68% to 100%
// Method: Comprehensive test coverage for function evaluation

use ruchy::frontend::ast::{Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind};
use ruchy::runtime::eval_func::{eval_function, eval_function_call, eval_lambda};
use ruchy::runtime::interpreter::{InterpreterError, Value};
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod eval_func_tests {
    use super::*;

    fn make_param(name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named("Any".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    #[test]
    fn test_eval_lambda_no_params() {
        let params = vec![];
        let body = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::default());
        let env = HashMap::new();

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, body, .. } => {
                assert_eq!(params.len(), 0);
                match body.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Expected integer literal"),
                }
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_lambda_with_params() {
        let params = vec![make_param("x"), make_param("y")];
        let body = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::default(),
                )),
                op: ruchy::frontend::ast::BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Identifier("y".to_string()),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let env = HashMap::new();

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], "x");
                assert_eq!(params[1], "y");
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_lambda_captures_environment() {
        let params = vec![make_param("x")];
        let body = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::default(),
                )),
                op: ruchy::frontend::ast::BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Identifier("captured".to_string()),
                    Span::default(),
                )),
            },
            Span::default(),
        );

        let mut env = HashMap::new();
        env.insert("captured".to_string(), Value::Integer(10));

        let result = eval_lambda(&params, &body, &env).unwrap();
        match result {
            Value::Closure {
                env: closure_env, ..
            } => {
                assert!(closure_env.contains_key("captured"));
                assert_eq!(closure_env.get("captured").unwrap(), &Value::Integer(10));
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_simple() {
        let name = "add";
        let params = vec![make_param("a"), make_param("b")];
        let body = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Identifier("a".to_string()),
                    Span::default(),
                )),
                op: ruchy::frontend::ast::BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Identifier("b".to_string()),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let env = HashMap::new();
        let mut stored_values = HashMap::new();

        let result = eval_function(name, &params, &body, &env, |k, v| {
            stored_values.insert(k, v);
        })
        .unwrap();

        // Check that function was stored in environment
        assert!(stored_values.contains_key("add"));

        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0], "a");
                assert_eq!(params[1], "b");
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_recursive() {
        let name = "factorial";
        let params = vec![make_param("n")];
        let body = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("n".to_string()),
                            Span::default(),
                        )),
                        op: ruchy::frontend::ast::BinaryOp::LessEqual,
                        right: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(1)),
                            Span::default(),
                        )),
                    },
                    Span::default(),
                )),
                then_branch: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1)),
                    Span::default(),
                )),
                else_branch: Some(Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("n".to_string()),
                            Span::default(),
                        )),
                        op: ruchy::frontend::ast::BinaryOp::Multiply,
                        right: Box::new(Expr::new(
                            ExprKind::Call {
                                func: Box::new(Expr::new(
                                    ExprKind::Identifier("factorial".to_string()),
                                    Span::default(),
                                )),
                                args: vec![Expr::new(
                                    ExprKind::Binary {
                                        left: Box::new(Expr::new(
                                            ExprKind::Identifier("n".to_string()),
                                            Span::default(),
                                        )),
                                        op: ruchy::frontend::ast::BinaryOp::Subtract,
                                        right: Box::new(Expr::new(
                                            ExprKind::Literal(Literal::Integer(1)),
                                            Span::default(),
                                        )),
                                    },
                                    Span::default(),
                                )],
                            },
                            Span::default(),
                        )),
                    },
                    Span::default(),
                ))),
            },
            Span::default(),
        );

        let env = HashMap::new();
        let mut stored_values = HashMap::new();

        let result = eval_function(name, &params, &body, &env, |k, v| {
            stored_values.insert(k, v);
        })
        .unwrap();

        // Function should be stored for recursion
        assert!(stored_values.contains_key("factorial"));

        match result {
            Value::Closure { params, .. } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0], "n");
            }
            _ => panic!("Expected closure value"),
        }
    }

    #[test]
    fn test_eval_function_call_simple() {
        // Create a simple identity function
        let func_expr = Expr::new(
            ExprKind::Identifier("identity".to_string()),
            Span::default(),
        );

        let args = vec![Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::default(),
        )];

        let mut eval_count = 0;
        let mut call_count = 0;

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| {
                eval_count += 1;
                match &expr.kind {
                    ExprKind::Identifier(name) if name == "identity" => Ok(Value::Closure {
                        params: vec!["x".to_string()],
                        body: Rc::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            Span::default(),
                        )),
                        env: Rc::new(HashMap::new()),
                    }),
                    ExprKind::Literal(Literal::Integer(n)) => Ok(Value::Integer(*n)),
                    _ => Err(InterpreterError::RuntimeError(
                        "Unexpected expr".to_string(),
                    )),
                }
            },
            |_func, args| {
                call_count += 1;
                // Simple identity function - return first arg
                Ok(args.first().cloned().unwrap_or(Value::nil()))
            },
        )
        .unwrap();

        assert_eq!(eval_count, 2); // Function + 1 arg
        assert_eq!(call_count, 1);
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_function_call_multiple_args() {
        let func_expr = Expr::new(ExprKind::Identifier("add".to_string()), Span::default());

        let args = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(10)), Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(20)), Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(30)), Span::default()),
        ];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| match &expr.kind {
                ExprKind::Identifier(name) if name == "add" => Ok(Value::Closure {
                    params: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                    body: Rc::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(0)),
                        Span::default(),
                    )),
                    env: Rc::new(HashMap::new()),
                }),
                ExprKind::Literal(Literal::Integer(n)) => Ok(Value::Integer(*n)),
                _ => Err(InterpreterError::RuntimeError(
                    "Unexpected expr".to_string(),
                )),
            },
            |_func, args| {
                // Sum all arguments
                let sum = args.iter().fold(0, |acc, v| {
                    if let Value::Integer(n) = v {
                        acc + n
                    } else {
                        acc
                    }
                });
                Ok(Value::Integer(sum))
            },
        )
        .unwrap();

        assert_eq!(result, Value::Integer(60));
    }

    #[test]
    fn test_eval_function_call_no_args() {
        let func_expr = Expr::new(
            ExprKind::Identifier("get_constant".to_string()),
            Span::default(),
        );

        let args = vec![];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| match &expr.kind {
                ExprKind::Identifier(name) if name == "get_constant" => Ok(Value::Closure {
                    params: vec![],
                    body: Rc::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(999)),
                        Span::default(),
                    )),
                    env: Rc::new(HashMap::new()),
                }),
                _ => Err(InterpreterError::RuntimeError(
                    "Unexpected expr".to_string(),
                )),
            },
            |_func, _args| Ok(Value::Integer(999)),
        )
        .unwrap();

        assert_eq!(result, Value::Integer(999));
    }

    #[test]
    fn test_eval_function_call_error_in_eval() {
        let func_expr = Expr::new(
            ExprKind::Identifier("bad_func".to_string()),
            Span::default(),
        );

        let args = vec![Expr::new(
            ExprKind::Identifier("undefined".to_string()),
            Span::default(),
        )];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| match &expr.kind {
                ExprKind::Identifier(name) if name == "bad_func" => Ok(Value::Closure {
                    params: vec!["x".to_string()],
                    body: Rc::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(0)),
                        Span::default(),
                    )),
                    env: Rc::new(HashMap::new()),
                }),
                ExprKind::Identifier(name) if name == "undefined" => Err(
                    InterpreterError::RuntimeError("Undefined variable".to_string()),
                ),
                _ => Err(InterpreterError::RuntimeError(
                    "Unexpected expr".to_string(),
                )),
            },
            |_func, _args| Ok(Value::nil()),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Undefined variable"));
    }

    #[test]
    fn test_eval_function_call_error_in_call() {
        let func_expr = Expr::new(ExprKind::Identifier("divide".to_string()), Span::default());

        let args = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(10)), Span::default()),
            Expr::new(ExprKind::Literal(Literal::Integer(0)), Span::default()),
        ];

        let result = eval_function_call(
            &func_expr,
            &args,
            |expr| match &expr.kind {
                ExprKind::Identifier(name) if name == "divide" => Ok(Value::Closure {
                    params: vec!["a".to_string(), "b".to_string()],
                    body: Rc::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(0)),
                        Span::default(),
                    )),
                    env: Rc::new(HashMap::new()),
                }),
                ExprKind::Literal(Literal::Integer(n)) => Ok(Value::Integer(*n)),
                _ => Err(InterpreterError::RuntimeError(
                    "Unexpected expr".to_string(),
                )),
            },
            |_func, args| {
                if let (Some(Value::Integer(_a)), Some(Value::Integer(b))) =
                    (args.get(0), args.get(1))
                {
                    if *b == 0 {
                        Err(InterpreterError::RuntimeError(
                            "Division by zero".to_string(),
                        ))
                    } else {
                        Ok(Value::Integer(_a / b))
                    }
                } else {
                    Err(InterpreterError::TypeError("Expected integers".to_string()))
                }
            },
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Division by zero"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    fn make_param(name: &str) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: TypeKind::Named("Any".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    fn prop_lambda_preserves_param_count(param_count: usize) -> TestResult {
        if param_count > 100 {
            return TestResult::discard();
        }

        let params: Vec<Param> = (0..param_count)
            .map(|i| make_param(&format!("p{}", i)))
            .collect();

        let body = Expr::new(ExprKind::Literal(Literal::Integer(0)), Span::default());

        let env = HashMap::new();

        let result = eval_lambda(&params, &body, &env).unwrap();

        match result {
            Value::Closure {
                params: closure_params,
                ..
            } => TestResult::from_bool(closure_params.len() == param_count),
            _ => TestResult::failed(),
        }
    }

    fn prop_function_stores_in_env(name_len: usize) -> TestResult {
        if name_len == 0 || name_len > 50 {
            return TestResult::discard();
        }

        let name: String = (0..name_len).map(|_| 'a').collect();
        let params = vec![];
        let body = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::default());
        let env = HashMap::new();
        let mut stored_values = HashMap::new();

        let _result = eval_function(&name, &params, &body, &env, |k, v| {
            stored_values.insert(k, v);
        });

        TestResult::from_bool(stored_values.contains_key(&name))
    }

    quickcheck! {
        fn test_lambda_param_count(count: usize) -> TestResult {
            prop_lambda_preserves_param_count(count)
        }

        fn test_function_name_storage(len: usize) -> TestResult {
            prop_function_stores_in_env(len)
        }
    }
}

// Summary: 12+ comprehensive tests for eval_func module
// This should boost coverage from 48.68% to near 100%
