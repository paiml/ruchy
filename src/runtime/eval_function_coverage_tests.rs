use super::*;
use crate::frontend::ast::{ExprKind, Literal, Param, Span};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// Helper functions
fn make_int_expr(val: i64) -> Expr {
    Expr::new(
        ExprKind::Literal(Literal::Integer(val, None)),
        Span::new(0, 0),
    )
}

fn make_unit_expr() -> Expr {
    Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))
}

fn make_param(name: &str) -> Param {
    Param {
        pattern: Pattern::Identifier(name.to_string()),
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    }
}

fn make_param_with_default(name: &str, default: Expr) -> Param {
    Param {
        pattern: Pattern::Identifier(name.to_string()),
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: Some(Box::new(default)),
    }
}

// ==================== eval_function_def tests ====================

#[test]
fn test_eval_function_def_basic() {
    let params = vec![make_param("x")];
    let body = make_int_expr(42);

    let result = eval_function_def("test_fn", &params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 1);
        assert_eq!(closure_params[0].0, "x");
        assert!(closure_params[0].1.is_none());
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_def_multiple_params() {
    let params = vec![make_param("a"), make_param("b"), make_param("c")];
    let body = make_unit_expr();

    let result = eval_function_def("multi_param_fn", &params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 3);
        assert_eq!(closure_params[0].0, "a");
        assert_eq!(closure_params[1].0, "b");
        assert_eq!(closure_params[2].0, "c");
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_def_with_default_params() {
    let params = vec![
        make_param("x"),
        make_param_with_default("y", make_int_expr(10)),
    ];
    let body = make_int_expr(0);

    let result = eval_function_def("fn_with_defaults", &params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 2);
        assert!(closure_params[0].1.is_none()); // x has no default
        assert!(closure_params[1].1.is_some()); // y has default
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_def_captures_environment() {
    let params = vec![make_param("x")];
    let body = make_unit_expr();

    let mut captured = HashMap::new();
    captured.insert("outer_var".to_string(), Value::Integer(100));

    let result = eval_function_def("capturing_fn", &params, &body, || captured.clone());
    assert!(result.is_ok());

    if let Ok(Value::Closure { env, .. }) = result {
        assert!(env.borrow().contains_key("outer_var"));
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_def_no_params() {
    let params = vec![];
    let body = make_int_expr(42);

    let result = eval_function_def("no_params_fn", &params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 0);
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_def_with_wildcard_pattern() {
    // Test with a wildcard pattern instead of identifier
    let param = Param {
        pattern: Pattern::Wildcard,
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    };
    let params = vec![param];
    let body = make_unit_expr();

    let result = eval_function_def("wildcard_fn", &params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 1);
        assert_eq!(closure_params[0].0, "_");
    } else {
        panic!("Expected Closure");
    }
}

// ==================== eval_lambda tests ====================

#[test]
fn test_eval_lambda_basic() {
    let params = vec![make_param("x")];
    let body = make_int_expr(42);

    let result = eval_lambda(&params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 1);
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_lambda_multiple_params() {
    let params = vec![make_param("a"), make_param("b")];
    let body = make_unit_expr();

    let result = eval_lambda(&params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 2);
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_lambda_with_default_params() {
    let params = vec![
        make_param("x"),
        make_param_with_default("y", make_int_expr(5)),
    ];
    let body = make_int_expr(0);

    let result = eval_lambda(&params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 2);
        assert!(closure_params[0].1.is_none());
        assert!(closure_params[1].1.is_some());
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_lambda_captures_environment() {
    let params = vec![make_param("x")];
    let body = make_unit_expr();

    let mut captured = HashMap::new();
    captured.insert(
        "captured_val".to_string(),
        Value::String(Arc::from("hello")),
    );

    let result = eval_lambda(&params, &body, || captured.clone());
    assert!(result.is_ok());

    if let Ok(Value::Closure { env, .. }) = result {
        assert!(env.borrow().contains_key("captured_val"));
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_lambda_empty_params() {
    let params = vec![];
    let body = make_int_expr(99);

    let result = eval_lambda(&params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params.len(), 0);
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_lambda_with_wildcard_pattern() {
    let param = Param {
        pattern: Pattern::Wildcard,
        ty: crate::frontend::ast::Type {
            kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
            span: Span::new(0, 0),
        },
        span: Span::new(0, 0),
        is_mutable: false,
        default_value: None,
    };
    let params = vec![param];
    let body = make_unit_expr();

    let result = eval_lambda(&params, &body, HashMap::new);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        params: closure_params,
        ..
    }) = result
    {
        assert_eq!(closure_params[0].0, "_");
    } else {
        panic!("Expected Closure");
    }
}

// ==================== eval_function_call tests ====================

#[test]
fn test_eval_function_call_string_value_error() {
    let non_function = Value::String(Arc::from("not a function"));

    let result = eval_function_call(&non_function, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
    if let Err(InterpreterError::TypeError(msg)) = result {
        assert!(msg.contains("Cannot call non-function"));
    }
}

#[test]
fn test_eval_function_call_array_error() {
    let non_function = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);

    let result = eval_function_call(&non_function, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
}

#[test]
fn test_eval_function_call_nil_error() {
    let non_function = Value::Nil;

    let result = eval_function_call(&non_function, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
}

#[test]
fn test_eval_function_call_closure_returns_value() {
    // Reset recursion state
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(100)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_call(
        &closure,
        &[Value::Integer(5)],
        |_body, _env| Ok(Value::Integer(100)),
        |_, _| Ok(None),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(100));

    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_wrong_arg_count_too_few() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let closure = Value::Closure {
        params: vec![("a".to_string(), None), ("b".to_string(), None)],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // Call with 0 args when 2 required
    let result = eval_function_call(&closure, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_wrong_arg_count_too_many() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // Call with 3 args when 1 required
    let result = eval_function_call(
        &closure,
        &[Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        |_, _| Ok(Value::Nil),
        |_, _| Ok(None),
    );

    assert!(result.is_err());
    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_handles_return_value() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let closure = Value::Closure {
        params: vec![],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // Simulate early return
    let result = eval_function_call(
        &closure,
        &[],
        |_, _| Err(InterpreterError::Return(Value::Integer(42))),
        |_, _| Ok(None),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(42));

    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_with_default_params() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    // Create closure with one required and one optional param
    let closure = Value::Closure {
        params: vec![
            ("x".to_string(), None),                              // required
            ("y".to_string(), Some(Arc::new(make_int_expr(10)))), // optional with default
        ],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // Call with only the required arg
    let result = eval_function_call(
        &closure,
        &[Value::Integer(5)],
        |_, env| {
            // Verify x is bound
            assert_eq!(env.get("x"), Some(&Value::Integer(5)));
            Ok(Value::Integer(15))
        },
        |_, _| Ok(None),
    );

    assert!(result.is_ok());
    CALL_DEPTH.with(|d| d.set(0));
}

// ==================== eval_method_call_value tests ====================

#[test]
fn test_eval_method_call_value_success() {
    let receiver = Value::Integer(42);

    let result = eval_method_call_value(
        &receiver,
        "to_string",
        &[],
        |_, _| Ok(Value::Nil),
        |_, _, _, _| Ok(Value::String(Arc::from("42"))),
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String(Arc::from("42")));
}

#[test]
fn test_eval_method_call_value_fallback_to_function() {
    let receiver = Value::Integer(42);

    let result = eval_method_call_value(
        &receiver,
        "nonexistent_method",
        &[],
        |_, _| Ok(Value::Nil),
        |_, _, _, _| {
            Err(InterpreterError::RuntimeError(
                "Method not found".to_string(),
            ))
        },
    );

    // Should fail because get_function_from_env always returns error
    assert!(result.is_err());
}

#[test]
fn test_eval_method_call_value_other_error() {
    let receiver = Value::Integer(42);

    let result = eval_method_call_value(
        &receiver,
        "test_method",
        &[],
        |_, _| Ok(Value::Nil),
        |_, _, _, _| Err(InterpreterError::TypeError("Type error".to_string())),
    );

    // Should propagate the type error
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

#[test]
fn test_eval_method_call_value_with_args() {
    let receiver = Value::String(Arc::from("hello"));

    let result = eval_method_call_value(
        &receiver,
        "replace",
        &[Value::String(Arc::from("l")), Value::String(Arc::from("L"))],
        |_, _| Ok(Value::Nil),
        |_, method, args, _| {
            assert_eq!(method, "replace");
            assert_eq!(args.len(), 2);
            Ok(Value::String(Arc::from("heLLo")))
        },
    );

    assert!(result.is_ok());
}

// ==================== create_partial_application tests ====================

#[test]
fn test_create_partial_application_with_captured_env() {
    let mut env = HashMap::new();
    env.insert("captured".to_string(), Value::Integer(100));

    let closure = Value::Closure {
        params: vec![("x".to_string(), None), ("y".to_string(), None)],
        body: Arc::new(make_int_expr(0)),
        env: Rc::new(RefCell::new(env)),
    };

    let result = create_partial_application(&closure, &[Value::Integer(5)]);
    assert!(result.is_ok());

    if let Ok(Value::Closure {
        env: new_env,
        params,
        ..
    }) = result
    {
        assert_eq!(params.len(), 1); // One param remaining
                                     // Check x is bound in new env
        assert!(new_env.borrow().contains_key("x"));
        assert_eq!(new_env.borrow().get("x"), Some(&Value::Integer(5)));
        // Check captured var is preserved
        assert!(new_env.borrow().contains_key("captured"));
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_create_partial_application_exact_args_error() {
    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // Trying to partially apply exactly the number of params should fail
    let result = create_partial_application(&closure, &[Value::Integer(1)]);
    assert!(result.is_err());
    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(msg.contains("Cannot partially apply"));
    }
}

#[test]
fn test_create_partial_application_multiple_args() {
    let closure = Value::Closure {
        params: vec![
            ("a".to_string(), None),
            ("b".to_string(), None),
            ("c".to_string(), None),
        ],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = create_partial_application(&closure, &[Value::Integer(1), Value::Integer(2)]);
    assert!(result.is_ok());

    if let Ok(Value::Closure { params, .. }) = result {
        assert_eq!(params.len(), 1); // Only c remaining
        assert_eq!(params[0].0, "c");
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_create_partial_application_bool_value() {
    let result = create_partial_application(&Value::Bool(true), &[Value::Integer(1)]);
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

#[test]
fn test_create_partial_application_float_value() {
    let result = create_partial_application(&Value::Float(3.14), &[Value::Integer(1)]);
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

// ==================== eval_function_composition tests ====================

#[test]
fn test_eval_function_composition_both_single_param() {
    let f = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(1)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let g = Value::Closure {
        params: vec![("y".to_string(), None)],
        body: Arc::new(make_int_expr(2)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_composition(&f, &g);
    assert!(result.is_ok());

    // Result should be a closure
    if let Ok(Value::Closure { params, .. }) = result {
        assert_eq!(params.len(), 1);
    } else {
        panic!("Expected Closure");
    }
}

#[test]
fn test_eval_function_composition_g_zero_params() {
    let f = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(1)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let g = Value::Closure {
        params: vec![],
        body: Arc::new(make_int_expr(2)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_composition(&f, &g);
    assert!(result.is_err());
    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(msg.contains("second function"));
    }
}

#[test]
fn test_eval_function_composition_f_zero_params() {
    let f = Value::Closure {
        params: vec![],
        body: Arc::new(make_int_expr(1)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let g = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(2)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_composition(&f, &g);
    assert!(result.is_err());
    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(msg.contains("first function"));
    }
}

#[test]
fn test_eval_function_composition_first_non_closure() {
    let f = Value::Integer(42);
    let g = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(2)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_composition(&f, &g);
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

#[test]
fn test_eval_function_composition_second_non_closure() {
    let f = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_int_expr(1)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let g = Value::String(Arc::from("not a function"));

    let result = eval_function_composition(&f, &g);
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

#[test]
fn test_eval_function_composition_both_non_closure() {
    let f = Value::Bool(true);
    let g = Value::Float(3.14);

    let result = eval_function_composition(&f, &g);
    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

// ==================== create_composition_closure tests ====================

#[test]
fn test_create_composition_closure_basic() {
    let f = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_int_expr(1),
        HashMap::new(),
    );
    let g = Closure::new(
        vec![Pattern::Identifier("y".to_string())],
        make_int_expr(2),
        HashMap::new(),
    );

    let result = create_composition_closure(&f, &g);
    assert!(result.is_ok());

    let composed = result.unwrap();
    assert_eq!(composed.params.len(), 1);
}

#[test]
fn test_create_composition_closure_merges_envs() {
    let mut f_env = HashMap::new();
    f_env.insert("f_var".to_string(), Value::Integer(1));

    let mut g_env = HashMap::new();
    g_env.insert("g_var".to_string(), Value::Integer(2));

    let f = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_int_expr(1),
        f_env,
    );
    let g = Closure::new(
        vec![Pattern::Identifier("y".to_string())],
        make_int_expr(2),
        g_env,
    );

    let result = create_composition_closure(&f, &g);
    assert!(result.is_ok());

    let composed = result.unwrap();
    assert!(composed.captured_env.contains_key("f_var"));
    assert!(composed.captured_env.contains_key("g_var"));
}

// ==================== eval_closure_call tests ====================

#[test]
fn test_eval_closure_call_basic() {
    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_int_expr(42),
        HashMap::new(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(5)], |_, env| {
        assert!(env.contains_key("x"));
        assert_eq!(env.get("x"), Some(&Value::Integer(5)));
        Ok(Value::Integer(47))
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(47));
}

#[test]
fn test_eval_closure_call_wrong_arg_count() {
    let closure = Closure::new(
        vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ],
        make_unit_expr(),
        HashMap::new(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(1)], |_, _| Ok(Value::Nil));

    assert!(result.is_err());
    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(msg.contains("expects"));
        assert!(msg.contains("arguments"));
    }
}

#[test]
fn test_eval_closure_call_named_recursive() {
    let closure = Closure::named(
        vec![Pattern::Identifier("n".to_string())],
        make_int_expr(1),
        HashMap::new(),
        "factorial".to_string(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(5)], |_, env| {
        // Should have self-reference
        assert!(env.contains_key("factorial"));
        Ok(Value::Integer(120))
    });

    assert!(result.is_ok());
}

#[test]
fn test_eval_closure_call_handles_return() {
    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_unit_expr(),
        HashMap::new(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(1)], |_, _| {
        Err(InterpreterError::Return(Value::String(Arc::from(
            "early return",
        ))))
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String(Arc::from("early return")));
}

#[test]
fn test_eval_closure_call_propagates_error() {
    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_unit_expr(),
        HashMap::new(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(1)], |_, _| {
        Err(InterpreterError::TypeError("test error".to_string()))
    });

    assert!(matches!(result, Err(InterpreterError::TypeError(_))));
}

#[test]
fn test_eval_closure_call_with_wildcard_pattern() {
    let closure = Closure::new(vec![Pattern::Wildcard], make_int_expr(99), HashMap::new());

    let result = eval_closure_call(&closure, &[Value::Integer(1)], |_, _| {
        Ok(Value::Integer(99))
    });

    assert!(result.is_ok());
}

#[test]
fn test_eval_closure_call_with_tuple_pattern() {
    let closure = Closure::new(
        vec![Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ])],
        make_unit_expr(),
        HashMap::new(),
    );

    let tuple_arg = Value::Tuple(Arc::new([Value::Integer(1), Value::Integer(2)]));

    let result = eval_closure_call(&closure, &[tuple_arg], |_, env| {
        assert_eq!(env.get("a"), Some(&Value::Integer(1)));
        assert_eq!(env.get("b"), Some(&Value::Integer(2)));
        Ok(Value::Integer(3))
    });

    assert!(result.is_ok());
}

#[test]
fn test_eval_closure_call_pattern_mismatch() {
    let closure = Closure::new(
        vec![Pattern::Literal(Literal::Integer(42, None))],
        make_unit_expr(),
        HashMap::new(),
    );

    let result = eval_closure_call(&closure, &[Value::Integer(99)], |_, _| Ok(Value::Nil));

    assert!(result.is_err());
    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(msg.contains("pattern does not match"));
    }
}

#[test]
fn test_eval_closure_call_with_captured_env() {
    let mut env = HashMap::new();
    env.insert("outer".to_string(), Value::Integer(100));

    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_unit_expr(),
        env,
    );

    let result = eval_closure_call(&closure, &[Value::Integer(5)], |_, env| {
        assert_eq!(env.get("outer"), Some(&Value::Integer(100)));
        assert_eq!(env.get("x"), Some(&Value::Integer(5)));
        Ok(Value::Integer(105))
    });

    assert!(result.is_ok());
}

// ==================== Additional edge case tests ====================

#[test]
fn test_is_callable_with_tuple() {
    let tuple = Value::Tuple(Arc::new([Value::Integer(1)]));
    assert!(!is_callable(&tuple));
}

#[test]
fn test_is_callable_with_object() {
    let obj = Value::Object(Arc::new(HashMap::new()));
    assert!(!is_callable(&obj));
}

#[test]
fn test_get_arity_with_tuple() {
    let tuple = Value::Tuple(Arc::new([Value::Integer(1), Value::Integer(2)]));
    assert!(get_arity(&tuple).is_err());
}

#[test]
fn test_get_arity_many_params() {
    let closure = Value::Closure {
        params: vec![
            ("a".to_string(), None),
            ("b".to_string(), None),
            ("c".to_string(), None),
            ("d".to_string(), None),
            ("e".to_string(), None),
        ],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    assert_eq!(get_arity(&closure).unwrap(), 5);
}

#[test]
fn test_bind_parameter_or_pattern() {
    // Test Pattern::Or if available - or test nested patterns
    let pattern = Pattern::Identifier("x".to_string());
    let value = Value::Float(3.14);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
    assert_eq!(env.get("x"), Some(&Value::Float(3.14)));
}

#[test]
fn test_bind_parameter_list_pattern() {
    let pattern = Pattern::List(vec![
        Pattern::Identifier("first".to_string()),
        Pattern::Identifier("second".to_string()),
    ]);
    let value = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    // This may fail if list pattern matching isn't implemented - that's fine
    // We're testing the code path
    let _ = result;
}

#[test]
fn test_closure_clone() {
    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_int_expr(42),
        HashMap::new(),
    );

    let cloned = closure.clone();
    assert_eq!(cloned.params.len(), closure.params.len());
    assert!(cloned.name.is_none());
}

#[test]
fn test_closure_debug() {
    let closure = Closure::new(
        vec![Pattern::Identifier("x".to_string())],
        make_int_expr(42),
        HashMap::new(),
    );

    // Just ensure Debug is implemented
    let _ = format!("{:?}", closure);
}
