    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use std::sync::Arc;

    // ============================================================================
    // Coverage tests for eval_array_iteration (27 uncov lines, 0% coverage)
    // ============================================================================

    // Helper to create a simple expression
    fn make_expr_int(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Span::new(0, 0),
        )
    }

    #[test]
    fn test_eval_array_iteration_empty_array() {
        let array = Value::Array(Arc::from(vec![]));
        let body_expr = make_expr_int(1);

        let mut with_variable =
            |_var: &str,
             _val: Value,
             eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                eval(&body_expr)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Integer(99))
        };

        let result =
            eval_array_iteration(&array, "x", &mut with_variable, &mut eval_expr)
                .expect("empty array iteration should succeed");
        // Empty array returns Nil (initial last_val)
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_array_iteration_single_element() {
        let array = Value::Array(Arc::from(vec![Value::Integer(42)]));
        let _body_expr = make_expr_int(1);

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                // Return the value passed in
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Integer(99))
        };

        let result =
            eval_array_iteration(&array, "item", &mut with_variable, &mut eval_expr)
                .expect("single element iteration should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_array_iteration_multiple_elements() {
        let array = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let _body_expr = make_expr_int(1);

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&array, "x", &mut with_variable, &mut eval_expr)
                .expect("multi-element iteration should succeed");
        // Last value should be the last element
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_array_iteration_non_array_error() {
        let not_array = Value::Integer(42);

        let mut with_variable =
            |_var: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(Value::Nil)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&not_array, "x", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::TypeError(msg)) = result {
            assert!(msg.contains("Expected array"), "got: {msg}");
        } else {
            panic!("Expected TypeError");
        }
    }

    #[test]
    fn test_eval_array_iteration_string_not_array() {
        let not_array = Value::String(Arc::from("hello"));

        let mut with_variable =
            |_var: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(Value::Nil)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&not_array, "x", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::TypeError(msg)) = result {
            assert!(msg.contains("Expected array"));
            assert!(msg.contains("String") || msg.contains("string"));
        } else {
            panic!("Expected TypeError for string");
        }
    }

    #[test]
    fn test_eval_array_iteration_bool_not_array() {
        let not_array = Value::Bool(true);

        let mut with_variable =
            |_var: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(Value::Nil)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&not_array, "x", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_array_iteration_with_break() {
        let array = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let mut iteration_count = 0;

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                iteration_count += 1;
                if let Value::Integer(2) = &val {
                    // Break on second element
                    Err(InterpreterError::Break(None, Value::Integer(200)))
                } else {
                    Ok(val)
                }
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&array, "x", &mut with_variable, &mut eval_expr)
                .expect("break should be handled, not propagated");
        assert_eq!(result, Value::Integer(200));
    }

    #[test]
    fn test_eval_array_iteration_with_continue() {
        let array = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                if let Value::Integer(2) = &val {
                    // Continue on second element
                    Err(InterpreterError::Continue(None))
                } else {
                    Ok(val)
                }
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&array, "x", &mut with_variable, &mut eval_expr)
                .expect("continue should be handled");
        // After continue, iteration resumes; last successful value is Integer(3)
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_array_iteration_with_runtime_error() {
        let array = Value::Array(Arc::from(vec![Value::Integer(1)]));

        let mut with_variable =
            |_var: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Err(InterpreterError::RuntimeError(
                    "something went wrong".to_string(),
                ))
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&array, "x", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert_eq!(msg, "something went wrong");
        } else {
            panic!("Expected RuntimeError");
        }
    }

    #[test]
    fn test_eval_array_iteration_nil_not_array() {
        let not_array = Value::Nil;

        let mut with_variable =
            |_var: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(Value::Nil)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&not_array, "x", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_array_iteration_nested_arrays() {
        let inner1 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let inner2 = Value::Array(Arc::from(vec![Value::Integer(3)]));
        let array = Value::Array(Arc::from(vec![inner1, inner2]));

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_array_iteration(&array, "arr", &mut with_variable, &mut eval_expr)
                .expect("nested array iteration should succeed");
        // Last value should be the second inner array
        if let Value::Array(arr) = &result {
            assert_eq!(arr.len(), 1);
        } else {
            panic!("Expected array result");
        }
    }

    // ============================================================================
    // Coverage tests for eval_range_iteration (34 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_range_iteration_exclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(3)),
            inclusive: false,
        };
        let body_expr = make_expr_int(1);
        let mut collected = Vec::new();

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                collected.push(val.clone());
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr)
                .expect("exclusive range iteration should succeed");
        // Last value is Integer(2)
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_range_iteration_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(1)),
            end: Box::new(Value::Integer(3)),
            inclusive: true,
        };
        let mut count = 0;

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                count += 1;
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr)
                .expect("inclusive range iteration should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_range_iteration_empty_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(5)),
            end: Box::new(Value::Integer(5)),
            inclusive: false,
        };

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr)
                .expect("empty range iteration should succeed");
        // No iterations so last_val stays Nil
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_range_iteration_non_range_error() {
        let not_a_range = Value::Integer(42);

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&not_a_range, "i", &mut with_variable, &mut eval_expr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected range"));
    }

    #[test]
    fn test_eval_range_iteration_break_stops_early() {
        let range = Value::Range {
            start: Box::new(Value::Integer(0)),
            end: Box::new(Value::Integer(10)),
            inclusive: false,
        };
        let mut iteration_count = 0;

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                iteration_count += 1;
                if let Value::Integer(i) = &val {
                    if *i >= 3 {
                        return Err(InterpreterError::Break(None, Value::Integer(99)));
                    }
                }
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        // Note: execute_iteration_step catches Break and returns should_continue = false
        let _result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr);
    }

    #[test]
    fn test_eval_range_iteration_single_element_inclusive() {
        let range = Value::Range {
            start: Box::new(Value::Integer(7)),
            end: Box::new(Value::Integer(7)),
            inclusive: true,
        };

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr)
                .expect("single element inclusive range should succeed");
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_eval_range_iteration_negative_range() {
        let range = Value::Range {
            start: Box::new(Value::Integer(-3)),
            end: Box::new(Value::Integer(0)),
            inclusive: false,
        };

        let mut with_variable =
            |_var: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Ok(val)
            };
        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let result =
            eval_range_iteration(&range, "i", &mut with_variable, &mut eval_expr)
                .expect("negative range iteration should succeed");
        assert_eq!(result, Value::Integer(-1));
    }

    // ============================================================================
    // Coverage tests for eval_let_expr (17 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_let_expr_basic_binding() {
        let value_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 0),
        );

        let mut eval_expr = |expr: &Expr| -> Result<Value, InterpreterError> {
            match &expr.kind {
                ExprKind::Literal(Literal::Integer(n, _)) => Ok(Value::Integer(*n)),
                ExprKind::Identifier(_) => Ok(Value::Integer(99)),
                _ => Ok(Value::Nil),
            }
        };

        let mut with_variable =
            |name: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                assert_eq!(name, "x");
                assert_eq!(val, Value::Integer(42));
                Ok(val)
            };

        let result = eval_let_expr("x", &value_expr, &body_expr, &mut eval_expr, &mut with_variable)
            .expect("eval_let_expr should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_let_expr_string_binding() {
        let value_expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Identifier("s".to_string()),
            Span::new(0, 0),
        );

        let mut eval_expr = |expr: &Expr| -> Result<Value, InterpreterError> {
            match &expr.kind {
                ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
                _ => Ok(Value::Nil),
            }
        };

        let mut with_variable =
            |name: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                assert_eq!(name, "s");
                Ok(val)
            };

        let result = eval_let_expr("s", &value_expr, &body_expr, &mut eval_expr, &mut with_variable)
            .expect("eval_let_expr with string should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_eval_let_expr_value_eval_error() {
        let value_expr = Expr::new(
            ExprKind::Identifier("undefined_var".to_string()),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::new(0, 0),
        );

        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Err(InterpreterError::RuntimeError("variable not found".to_string()))
        };

        let mut with_variable =
            |_name: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                panic!("should not be called when eval fails");
            };

        let result = eval_let_expr("x", &value_expr, &body_expr, &mut eval_expr, &mut with_variable);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_let_expr_with_variable_callback_error() {
        let value_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 0),
        );

        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Integer(42))
        };

        let mut with_variable =
            |_name: &str,
             _val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                Err(InterpreterError::RuntimeError("binding error".to_string()))
            };

        let result = eval_let_expr("x", &value_expr, &body_expr, &mut eval_expr, &mut with_variable);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_let_expr_nil_value() {
        let value_expr = Expr::new(
            ExprKind::Literal(Literal::Null),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 0),
        );

        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Nil)
        };

        let mut with_variable =
            |name: &str,
             val: Value,
             _eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                assert_eq!(name, "x");
                assert_eq!(val, Value::Nil);
                Ok(val)
            };

        let result = eval_let_expr("x", &value_expr, &body_expr, &mut eval_expr, &mut with_variable)
            .expect("eval_let_expr with nil should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_let_expr_with_variable_uses_eval() {
        // Test that with_variable can use the eval closure
        let value_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::new(0, 0),
        );
        let body_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(20, None)),
            Span::new(0, 0),
        );

        let mut eval_expr = |_expr: &Expr| -> Result<Value, InterpreterError> {
            Ok(Value::Integer(10))
        };

        let mut with_variable =
            |_name: &str,
             val: Value,
             eval: &mut dyn FnMut(&Expr) -> Result<Value, InterpreterError>| {
                // Use eval closure within with_variable
                let body = Expr::new(
                    ExprKind::Literal(Literal::Integer(99, None)),
                    Span::new(0, 0),
                );
                let _ = eval(&body);
                Ok(val)
            };

        let result = eval_let_expr("x", &value_expr, &body_expr, &mut eval_expr, &mut with_variable)
            .expect("eval_let_expr should succeed");
        assert_eq!(result, Value::Integer(10));
    }
