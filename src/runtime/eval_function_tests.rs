    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};

    // Helper to create a simple literal expression
    fn lit_expr(val: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(val, None)),
            Span::new(0, 0),
        )
    }

    fn unit_expr() -> Expr {
        Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))
    }

    // ==================== Recursion depth tests ====================

    #[test]
    fn test_set_max_recursion_depth() {
        set_max_recursion_depth(500);
        // Check it doesn't panic and can be set
        set_max_recursion_depth(1000); // Reset to default
    }

    #[test]
    fn test_get_current_depth_initially_zero() {
        // Reset depth state
        CALL_DEPTH.with(|d| d.set(0));
        assert_eq!(get_current_depth(), 0);
    }

    #[test]
    fn test_check_recursion_depth_increments() {
        // Reset state
        CALL_DEPTH.with(|d| d.set(0));
        MAX_DEPTH.with(|m| m.set(1000));

        check_recursion_depth().unwrap();
        assert_eq!(get_current_depth(), 1);

        check_recursion_depth().unwrap();
        assert_eq!(get_current_depth(), 2);

        // Clean up
        decrement_depth();
        decrement_depth();
    }

    #[test]
    fn test_check_recursion_depth_limit_exceeded() {
        // Reset and set low limit
        CALL_DEPTH.with(|d| d.set(5));
        MAX_DEPTH.with(|m| m.set(5));

        let result = check_recursion_depth();
        assert!(matches!(
            result,
            Err(InterpreterError::RecursionLimitExceeded(5, 5))
        ));

        // Clean up
        CALL_DEPTH.with(|d| d.set(0));
        MAX_DEPTH.with(|m| m.set(1000));
    }

    #[test]
    fn test_decrement_depth() {
        // Reset state
        CALL_DEPTH.with(|d| d.set(5));

        decrement_depth();
        assert_eq!(get_current_depth(), 4);

        decrement_depth();
        assert_eq!(get_current_depth(), 3);

        // Clean up
        CALL_DEPTH.with(|d| d.set(0));
    }

    #[test]
    fn test_decrement_depth_saturating() {
        // Reset state to zero
        CALL_DEPTH.with(|d| d.set(0));

        // Should not go negative
        decrement_depth();
        assert_eq!(get_current_depth(), 0);
    }

    // ==================== Closure creation tests ====================

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
    fn test_closure_with_captured_env() {
        let params = vec![Pattern::Identifier("x".to_string())];
        let body = unit_expr();
        let mut env = HashMap::new();
        env.insert("captured_var".to_string(), Value::Integer(100));

        let closure = Closure::new(params, body, env);
        assert_eq!(
            closure.captured_env.get("captured_var"),
            Some(&Value::Integer(100))
        );
    }

    // ==================== Parameter binding tests ====================

    #[test]
    fn test_parameter_binding() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Integer(42);
        let mut env = HashMap::new();

        bind_parameter(&pattern, &value, &mut env).unwrap();
        assert_eq!(env.get("x"), Some(&Value::Integer(42)));
    }

    #[test]
    fn test_parameter_binding_string_value() {
        let pattern = Pattern::Identifier("name".to_string());
        let value = Value::String("hello".into());
        let mut env = HashMap::new();

        bind_parameter(&pattern, &value, &mut env).unwrap();
        assert_eq!(env.get("name"), Some(&Value::String("hello".into())));
    }

    #[test]
    fn test_parameter_binding_multiple() {
        let mut env = HashMap::new();

        bind_parameter(
            &Pattern::Identifier("a".to_string()),
            &Value::Integer(1),
            &mut env,
        )
        .unwrap();
        bind_parameter(
            &Pattern::Identifier("b".to_string()),
            &Value::Integer(2),
            &mut env,
        )
        .unwrap();
        bind_parameter(
            &Pattern::Identifier("c".to_string()),
            &Value::Integer(3),
            &mut env,
        )
        .unwrap();

        assert_eq!(env.len(), 3);
        assert_eq!(env.get("a"), Some(&Value::Integer(1)));
        assert_eq!(env.get("b"), Some(&Value::Integer(2)));
        assert_eq!(env.get("c"), Some(&Value::Integer(3)));
    }

    // ==================== Callable tests ====================

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
    fn test_is_callable_string_not_callable() {
        assert!(!is_callable(&Value::String("hello".into())));
    }

    #[test]
    fn test_is_callable_bool_not_callable() {
        assert!(!is_callable(&Value::Bool(true)));
    }

    #[test]
    fn test_is_callable_nil_not_callable() {
        assert!(!is_callable(&Value::Nil));
    }

    // ==================== Arity tests ====================

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

    #[test]
    fn test_get_arity_zero_params() {
        let function_value = Value::Closure {
            params: vec![],
            body: Arc::new(unit_expr()),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        assert_eq!(get_arity(&function_value).unwrap(), 0);
    }

    #[test]
    fn test_get_arity_three_params() {
        let function_value = Value::Closure {
            params: vec![
                ("a".to_string(), None),
                ("b".to_string(), None),
                ("c".to_string(), None),
            ],
            body: Arc::new(unit_expr()),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        assert_eq!(get_arity(&function_value).unwrap(), 3);
    }

    #[test]
    fn test_get_arity_error_on_string() {
        let result = get_arity(&Value::String("not a function".into()));
        assert!(result.is_err());
    }

    // ==================== Function call tests ====================

    #[test]
    fn test_eval_function_call_non_function() {
        let non_function = Value::Integer(42);

        let result = eval_function_call(&non_function, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

        assert!(matches!(result, Err(InterpreterError::TypeError(_))));
    }

    #[test]
    fn test_eval_function_call_closure() {
        // Reset recursion state
        CALL_DEPTH.with(|d| d.set(0));
        MAX_DEPTH.with(|m| m.set(1000));

        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(lit_expr(42)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = eval_function_call(
            &closure,
            &[Value::Integer(10)],
            |_body, _env| Ok(Value::Integer(42)),
            |_, _| Ok(None),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    // ==================== Partial application tests ====================

    #[test]
    fn test_create_partial_application_non_function() {
        let non_function = Value::Integer(42);
        let result = create_partial_application(&non_function, &[Value::Integer(1)]);
        assert!(matches!(result, Err(InterpreterError::TypeError(_))));
    }

    #[test]
    fn test_create_partial_application_too_many_args() {
        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(lit_expr(42)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = create_partial_application(&closure, &[Value::Integer(1), Value::Integer(2)]);
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_create_partial_application_success() {
        let closure = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: Arc::new(lit_expr(42)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = create_partial_application(&closure, &[Value::Integer(1)]);
        assert!(result.is_ok());

        // Verify the resulting closure has one less parameter
        if let Ok(Value::Closure { params, .. }) = result {
            assert_eq!(params.len(), 1);
        } else {
            panic!("Expected closure");
        }
    }

    // ==================== Function composition tests ====================

    #[test]
    fn test_eval_function_composition_non_functions() {
        let non_function = Value::Integer(42);
        let result = eval_function_composition(&non_function, &non_function);
        assert!(matches!(result, Err(InterpreterError::TypeError(_))));
    }

    #[test]
    fn test_eval_function_composition_wrong_arity_first() {
        let f = Value::Closure {
            params: vec![("a".to_string(), None), ("b".to_string(), None)],
            body: Arc::new(lit_expr(1)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        let g = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(lit_expr(2)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = eval_function_composition(&f, &g);
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_eval_function_composition_wrong_arity_second() {
        let f = Value::Closure {
            params: vec![("a".to_string(), None)],
            body: Arc::new(lit_expr(1)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        let g = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: Arc::new(lit_expr(2)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = eval_function_composition(&f, &g);
        assert!(matches!(result, Err(InterpreterError::RuntimeError(_))));
    }

    #[test]
    fn test_eval_function_composition_success() {
        let f = Value::Closure {
            params: vec![("a".to_string(), None)],
            body: Arc::new(lit_expr(1)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };
        let g = Value::Closure {
            params: vec![("x".to_string(), None)],
            body: Arc::new(lit_expr(2)),
            env: Rc::new(RefCell::new(HashMap::new())),
        };

        let result = eval_function_composition(&f, &g);
        assert!(result.is_ok());
    }

    // ==================== get_function_from_env tests ====================

    #[test]
    fn test_get_function_from_env_not_found() {
        let result = get_function_from_env("nonexistent");
        assert!(result.is_err());
    }

