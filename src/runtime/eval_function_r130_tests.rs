use super::*;
use crate::frontend::ast::{ExprKind, Literal, Span};
use std::cell::RefCell;
use std::rc::Rc;

// EXTREME TDD Round 130: eval_function.rs coverage boost
// Target: 74.13% -> 90%+

// Helper to create a simple literal expression
fn make_lit_expr(val: i64) -> Expr {
    Expr::new(
        ExprKind::Literal(Literal::Integer(val, None)),
        Span::new(0, 0),
    )
}

fn make_unit_expr() -> Expr {
    Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))
}

// ==================== Recursion depth tests ====================

#[test]
fn test_set_max_recursion_depth_r130() {
    set_max_recursion_depth(500);
    let depth = MAX_DEPTH.with(|m| m.get());
    assert_eq!(depth, 500);
    set_max_recursion_depth(1000); // Reset
}

#[test]
fn test_get_current_depth_r130() {
    CALL_DEPTH.with(|d| d.set(5));
    let depth = get_current_depth();
    assert_eq!(depth, 5);
    CALL_DEPTH.with(|d| d.set(0)); // Reset
}

#[test]
fn test_check_recursion_depth_success_r130() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let result = check_recursion_depth();
    assert!(result.is_ok());

    let depth = get_current_depth();
    assert_eq!(depth, 1);
    CALL_DEPTH.with(|d| d.set(0)); // Reset
}

#[test]
fn test_check_recursion_depth_exceeded_r130() {
    CALL_DEPTH.with(|d| d.set(100));
    MAX_DEPTH.with(|m| m.set(100));

    let result = check_recursion_depth();
    assert!(result.is_err());

    CALL_DEPTH.with(|d| d.set(0)); // Reset
    MAX_DEPTH.with(|m| m.set(1000)); // Reset
}

#[test]
fn test_decrement_depth_r130() {
    CALL_DEPTH.with(|d| d.set(5));
    decrement_depth();
    let depth = get_current_depth();
    assert_eq!(depth, 4);
    CALL_DEPTH.with(|d| d.set(0)); // Reset
}

#[test]
fn test_decrement_depth_at_zero_r130() {
    CALL_DEPTH.with(|d| d.set(0));
    decrement_depth();
    let depth = get_current_depth();
    assert_eq!(depth, 0); // Should saturate at 0
}

// ==================== Closure struct tests ====================

#[test]
fn test_closure_new_r130() {
    let params = vec![Pattern::Identifier("x".to_string())];
    let body = make_lit_expr(42);
    let env = HashMap::new();

    let closure = Closure::new(params.clone(), body.clone(), env);

    assert_eq!(closure.params.len(), 1);
    assert!(closure.name.is_none());
}

#[test]
fn test_closure_with_name_r130() {
    let params = vec![Pattern::Identifier("x".to_string())];
    let body = make_lit_expr(42);
    let env = HashMap::new();

    let mut closure = Closure::new(params, body, env);
    closure.name = Some("my_func".to_string());

    assert_eq!(closure.name, Some("my_func".to_string()));
}

#[test]
fn test_closure_named_r130() {
    let params = vec![Pattern::Identifier("x".to_string())];
    let body = make_lit_expr(42);
    let env = HashMap::new();

    let closure = Closure::named(params, body, env, "test_fn".to_string());
    assert_eq!(closure.name, Some("test_fn".to_string()));
}

#[test]
fn test_closure_empty_params_r130() {
    let params: Vec<Pattern> = vec![];
    let body = make_unit_expr();
    let env = HashMap::new();
    let closure = Closure::new(params, body, env);
    assert_eq!(closure.params.len(), 0);
}

#[test]
fn test_closure_with_captured_env_r130() {
    let params = vec![Pattern::Identifier("x".to_string())];
    let body = make_lit_expr(0);
    let mut env = HashMap::new();
    env.insert("outer_var".to_string(), Value::Integer(100));

    let closure = Closure::new(params, body, env.clone());
    assert!(closure.captured_env.contains_key("outer_var"));
    assert_eq!(
        closure.captured_env.get("outer_var"),
        Some(&Value::Integer(100))
    );
}

#[test]
fn test_closure_multiple_params_r130() {
    let params = vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
        Pattern::Identifier("c".to_string()),
    ];
    let body = make_unit_expr();
    let closure = Closure::new(params, body, HashMap::new());
    assert_eq!(closure.params.len(), 3);
}

// ==================== is_callable tests ====================

#[test]
fn test_is_callable_closure_r130() {
    // Create a Value::Closure
    let value = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_lit_expr(42)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    assert!(is_callable(&value));
}

#[test]
fn test_is_callable_integer_r130() {
    assert!(!is_callable(&Value::Integer(42)));
}

#[test]
fn test_is_callable_string_r130() {
    assert!(!is_callable(&Value::String(Arc::from("hello"))));
}

#[test]
fn test_is_callable_nil_r130() {
    assert!(!is_callable(&Value::Nil));
}

#[test]
fn test_is_callable_bool_r130() {
    assert!(!is_callable(&Value::Bool(true)));
    assert!(!is_callable(&Value::Bool(false)));
}

#[test]
fn test_is_callable_float_r130() {
    assert!(!is_callable(&Value::Float(3.14)));
}

#[test]
fn test_is_callable_array_r130() {
    let arr = Value::from_array(vec![Value::Integer(1)]);
    assert!(!is_callable(&arr));
}

#[test]
fn test_is_callable_builtin_r130() {
    // BuiltinFunction is just a name string
    let builtin = Value::BuiltinFunction("print".to_string());
    assert!(!is_callable(&builtin)); // is_callable only checks Closure
}

// ==================== get_arity tests ====================

#[test]
fn test_get_arity_closure_r130() {
    let value = Value::Closure {
        params: vec![("x".to_string(), None), ("y".to_string(), None)],
        body: Arc::new(make_lit_expr(42)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let result = get_arity(&value);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_get_arity_nil_r130() {
    let result = get_arity(&Value::Nil);
    assert!(result.is_err());
}

#[test]
fn test_get_arity_bool_r130() {
    let result = get_arity(&Value::Bool(true));
    assert!(result.is_err());
}

#[test]
fn test_get_arity_float_r130() {
    let result = get_arity(&Value::Float(3.14));
    assert!(result.is_err());
}

#[test]
fn test_get_arity_array_r130() {
    let result = get_arity(&Value::from_array(vec![Value::Integer(1)]));
    assert!(result.is_err());
}

#[test]
fn test_get_arity_string_r130() {
    let result = get_arity(&Value::String(Arc::from("hello")));
    assert!(result.is_err());
}

#[test]
fn test_get_arity_object_r130() {
    let obj = Value::Object(Arc::new(HashMap::new()));
    let result = get_arity(&obj);
    assert!(result.is_err());
}

#[test]
fn test_get_arity_zero_params_r130() {
    let value = Value::Closure {
        params: vec![],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };
    let result = get_arity(&value);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

// ==================== Recursion depth edge cases ====================

#[test]
fn test_max_recursion_depth_boundary_r130() {
    CALL_DEPTH.with(|d| d.set(999));
    MAX_DEPTH.with(|m| m.set(1000));

    // Should succeed at depth 999 (becomes 1000 which == max)
    let result = check_recursion_depth();
    assert!(result.is_ok());

    // Now at depth 1000, next call should fail
    let result2 = check_recursion_depth();
    assert!(result2.is_err());

    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_decrement_depth_multiple_r130() {
    CALL_DEPTH.with(|d| d.set(10));

    decrement_depth();
    assert_eq!(get_current_depth(), 9);

    decrement_depth();
    assert_eq!(get_current_depth(), 8);

    decrement_depth();
    assert_eq!(get_current_depth(), 7);

    CALL_DEPTH.with(|d| d.set(0));
}

// ==================== eval_function_call tests ====================

#[test]
fn test_eval_function_call_non_callable_r130() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let non_callable = Value::Integer(42);

    let result = eval_function_call(&non_callable, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_closure_r130() {
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));

    let closure = Value::Closure {
        params: vec![("x".to_string(), None)],
        body: Arc::new(make_lit_expr(42)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    // The eval_closure callback returns the body value
    let result = eval_function_call(
        &closure,
        &[Value::Integer(1)],
        |_expr, _env| Ok(Value::Integer(42)),
        |_, _| Ok(None),
    );

    assert!(result.is_ok());
    CALL_DEPTH.with(|d| d.set(0));
}

#[test]
fn test_eval_function_call_depth_exceeded_r130() {
    CALL_DEPTH.with(|d| d.set(100));
    MAX_DEPTH.with(|m| m.set(100));

    let closure = Value::Closure {
        params: vec![],
        body: Arc::new(make_unit_expr()),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = eval_function_call(&closure, &[], |_, _| Ok(Value::Nil), |_, _| Ok(None));

    assert!(result.is_err());
    CALL_DEPTH.with(|d| d.set(0));
    MAX_DEPTH.with(|m| m.set(1000));
}

// ==================== bind_parameter tests ====================

#[test]
fn test_bind_parameter_identifier_r130() {
    let pattern = Pattern::Identifier("x".to_string());
    let value = Value::Integer(42);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
    assert_eq!(env.get("x"), Some(&Value::Integer(42)));
}

#[test]
fn test_bind_parameter_wildcard_r130() {
    let pattern = Pattern::Wildcard;
    let value = Value::Integer(42);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
    // Wildcard doesn't bind anything
    assert!(env.is_empty());
}

#[test]
fn test_bind_parameter_tuple_r130() {
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let value = Value::Tuple(Arc::new([Value::Integer(1), Value::Integer(2)]));
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
    assert_eq!(env.get("a"), Some(&Value::Integer(1)));
    assert_eq!(env.get("b"), Some(&Value::Integer(2)));
}

#[test]
fn test_bind_parameter_literal_match_r130() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(42);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
}

#[test]
fn test_bind_parameter_literal_no_match_r130() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(99);
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_err());
}

#[test]
fn test_bind_parameter_nested_tuple_r130() {
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Tuple(vec![
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string()),
        ]),
    ]);
    let value = Value::Tuple(Arc::new([
        Value::Integer(1),
        Value::Tuple(Arc::new([Value::Integer(2), Value::Integer(3)])),
    ]));
    let mut env = HashMap::new();

    let result = bind_parameter(&pattern, &value, &mut env);
    assert!(result.is_ok());
    assert_eq!(env.get("a"), Some(&Value::Integer(1)));
    assert_eq!(env.get("b"), Some(&Value::Integer(2)));
    assert_eq!(env.get("c"), Some(&Value::Integer(3)));
}

#[test]
fn test_bind_parameter_multiple_r130() {
    let mut env = HashMap::new();

    bind_parameter(
        &Pattern::Identifier("a".to_string()),
        &Value::Integer(1),
        &mut env,
    )
    .unwrap();

    bind_parameter(
        &Pattern::Identifier("b".to_string()),
        &Value::Float(2.5),
        &mut env,
    )
    .unwrap();

    bind_parameter(
        &Pattern::Identifier("c".to_string()),
        &Value::String(Arc::from("test")),
        &mut env,
    )
    .unwrap();

    assert_eq!(env.len(), 3);
    assert_eq!(env.get("a"), Some(&Value::Integer(1)));
    assert_eq!(env.get("b"), Some(&Value::Float(2.5)));
    assert_eq!(env.get("c"), Some(&Value::String(Arc::from("test"))));
}

// ==================== Partial application tests ====================

#[test]
fn test_create_partial_application_r130() {
    let closure = Value::Closure {
        params: vec![("x".to_string(), None), ("y".to_string(), None)],
        body: Arc::new(make_lit_expr(42)),
        env: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = create_partial_application(&closure, &[Value::Integer(1)]);
    assert!(result.is_ok());

    // Result should be a new Closure with fewer params
    if let Ok(Value::Closure { params, .. }) = result {
        assert_eq!(params.len(), 1);
    } else {
        panic!("Expected Closure result");
    }
}

#[test]
fn test_create_partial_application_non_closure_r130() {
    let non_closure = Value::Integer(42);

    let result = create_partial_application(&non_closure, &[Value::Integer(1)]);
    assert!(result.is_err());
}
