
use super::*;
// Test Interpreter::new
#[test]
fn test_interpreter_new() {
    let interp = make_interpreter();
    assert!(interp.stack.is_empty());
    assert_eq!(interp.env_stack.len(), 1); // Global env
}

// Test Interpreter::default
#[test]
fn test_interpreter_default() {
    let interp = Interpreter::default();
    assert!(interp.stack.is_empty());
}

// Test eval_expr with literals
#[test]
fn test_eval_expr_integer() {
    let mut interp = make_interpreter();
    let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
    let result = interp.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_expr_float() {
    let mut interp = make_interpreter();
    let expr = make_expr(ExprKind::Literal(Literal::Float(3.14)));
    let result = interp.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_eval_expr_string() {
    let mut interp = make_interpreter();
    let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
    let result = interp.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::from_string("hello".to_string()));
}

#[test]
fn test_eval_expr_bool_true() {
    let mut interp = make_interpreter();
    let expr = make_expr(ExprKind::Literal(Literal::Bool(true)));
    let result = interp.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_eval_expr_bool_false() {
    let mut interp = make_interpreter();
    let expr = make_expr(ExprKind::Literal(Literal::Bool(false)));
    let result = interp.eval_expr(&expr).unwrap();
    assert_eq!(result, Value::Bool(false));
}

// Test is_type_definition
#[test]
fn test_is_type_definition_struct() {
    let kind = ExprKind::Struct {
        name: "Point".to_string(),
        type_params: vec![],
        fields: vec![],
        methods: vec![],
        derives: vec![],
        is_pub: true,
    };
    assert!(Interpreter::is_type_definition(&kind));
}

#[test]
fn test_is_type_definition_class() {
    let kind = ExprKind::Class {
        name: "MyClass".to_string(),
        type_params: vec![],
        superclass: None,
        traits: vec![],
        fields: vec![],
        constructors: vec![],
        methods: vec![],
        constants: vec![],
        properties: vec![],
        derives: vec![],
        decorators: vec![],
        is_pub: true,
        is_abstract: false,
        is_sealed: false,
    };
    assert!(Interpreter::is_type_definition(&kind));
}

#[test]
fn test_is_type_definition_enum() {
    let kind = ExprKind::Enum {
        name: "Color".to_string(),
        type_params: vec![],
        variants: vec![],
        is_pub: true,
    };
    assert!(Interpreter::is_type_definition(&kind));
}

#[test]
fn test_is_type_definition_actor() {
    let kind = ExprKind::Actor {
        name: "Counter".to_string(),
        state: vec![],
        handlers: vec![],
    };
    assert!(Interpreter::is_type_definition(&kind));
}

#[test]
fn test_is_type_definition_impl() {
    let kind = ExprKind::Impl {
        trait_name: None,
        for_type: "Point".to_string(),
        type_params: vec![],
        methods: vec![],
        is_pub: true,
    };
    assert!(Interpreter::is_type_definition(&kind));
}

#[test]
fn test_is_type_definition_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_type_definition(&kind));
}

// Test is_actor_operation
#[test]
fn test_is_actor_operation_spawn() {
    let inner = make_expr(ExprKind::Identifier("Counter".to_string()));
    let kind = ExprKind::Spawn {
        actor: Box::new(inner),
    };
    assert!(Interpreter::is_actor_operation(&kind));
}

#[test]
fn test_is_actor_operation_send() {
    let actor = Box::new(make_expr(ExprKind::Identifier("counter".to_string())));
    let msg = Box::new(make_expr(ExprKind::Identifier("Inc".to_string())));
    let kind = ExprKind::ActorSend {
        actor,
        message: msg,
    };
    assert!(Interpreter::is_actor_operation(&kind));
}

#[test]
fn test_is_actor_operation_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_actor_operation(&kind));
}

// Test is_special_form
#[test]
fn test_is_special_form_none() {
    let kind = ExprKind::None;
    assert!(Interpreter::is_special_form(&kind));
}

#[test]
fn test_is_special_form_some() {
    let inner = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
    let kind = ExprKind::Some {
        value: Box::new(inner),
    };
    assert!(Interpreter::is_special_form(&kind));
}

#[test]
fn test_is_special_form_struct_literal() {
    let kind = ExprKind::StructLiteral {
        name: "Point".to_string(),
        fields: vec![],
        base: None,
    };
    assert!(Interpreter::is_special_form(&kind));
}

#[test]
fn test_is_special_form_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_special_form(&kind));
}

// Test is_control_flow_expr
#[test]
fn test_is_control_flow_if() {
    let cond = Box::new(make_expr(ExprKind::Literal(Literal::Bool(true))));
    let then_branch = Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None))));
    let kind = ExprKind::If {
        condition: cond,
        then_branch,
        else_branch: None,
    };
    assert!(Interpreter::is_control_flow_expr(&kind));
}

#[test]
fn test_is_control_flow_return() {
    let kind = ExprKind::Return { value: None };
    assert!(Interpreter::is_control_flow_expr(&kind));
}

#[test]
fn test_is_control_flow_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_control_flow_expr(&kind));
}

// Test is_data_structure_expr
#[test]
fn test_is_data_structure_list() {
    let kind = ExprKind::List(vec![]);
    assert!(Interpreter::is_data_structure_expr(&kind));
}

#[test]
fn test_is_data_structure_tuple() {
    let kind = ExprKind::Tuple(vec![]);
    assert!(Interpreter::is_data_structure_expr(&kind));
}

#[test]
fn test_is_data_structure_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_data_structure_expr(&kind));
}

// Test is_assignment_expr
#[test]
fn test_is_assignment_assign() {
    let target = Box::new(make_expr(ExprKind::Identifier("x".to_string())));
    let value = Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None))));
    let kind = ExprKind::Assign { target, value };
    assert!(Interpreter::is_assignment_expr(&kind));
}

#[test]
fn test_is_assignment_false() {
    let kind = ExprKind::Literal(Literal::Integer(42, None));
    assert!(!Interpreter::is_assignment_expr(&kind));
}

// Test stack operations
#[test]
fn test_push_and_pop() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(42)).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_pop_empty_stack() {
    let mut interp = make_interpreter();
    let result = interp.pop();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Stack underflow"));
}

#[test]
fn test_peek() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(1)).unwrap();
    interp.push(Value::Integer(2)).unwrap();

    let top = interp.peek(0).unwrap();
    assert_eq!(top, Value::Integer(2));

    let second = interp.peek(1).unwrap();
    assert_eq!(second, Value::Integer(1));
}

#[test]
fn test_peek_out_of_bounds() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(1)).unwrap();
    let result = interp.peek(5);
    assert!(result.is_err());
}

// Test stdout capture
#[test]
fn test_capture_stdout() {
    let mut interp = make_interpreter();
    interp.capture_stdout("hello".to_string());
    assert!(interp.has_stdout());
    assert_eq!(interp.get_stdout(), "hello");
}

#[test]
fn test_get_stdout_multiple() {
    let mut interp = make_interpreter();
    interp.capture_stdout("line1".to_string());
    interp.capture_stdout("line2".to_string());
    assert_eq!(interp.get_stdout(), "line1\nline2");
}

#[test]
fn test_clear_stdout() {
    let mut interp = make_interpreter();
    interp.capture_stdout("test".to_string());
    interp.clear_stdout();
    assert!(!interp.has_stdout());
    assert_eq!(interp.get_stdout(), "");
}

// Test environment operations
#[test]
fn test_set_variable() {
    let mut interp = make_interpreter();
    interp.set_variable("x", Value::Integer(42));
    let result = interp.lookup_variable("x").unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_lookup_variable_not_found() {
    let interp = make_interpreter();
    let result = interp.lookup_variable("nonexistent");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Undefined variable"));
}

#[test]
fn test_current_env() {
    let interp = make_interpreter();
    let env = interp.current_env();
    // Should be the global environment
    assert!(env.borrow().contains_key("max"));
}

#[test]
fn test_env_push_pop() {
    let mut interp = make_interpreter();
    let initial_depth = interp.env_stack.len();

    let mut new_env = HashMap::new();
    new_env.insert("local".to_string(), Value::Integer(42));
    interp.env_push(new_env);

    assert_eq!(interp.env_stack.len(), initial_depth + 1);
    assert_eq!(interp.lookup_variable("local").unwrap(), Value::Integer(42));

    interp.env_pop();
    assert_eq!(interp.env_stack.len(), initial_depth);
    assert!(interp.lookup_variable("local").is_err());
}

#[test]
fn test_env_set() {
    let mut interp = make_interpreter();
    interp.env_set("x".to_string(), Value::Integer(1));
    assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(1));
}

#[test]
fn test_env_set_mut_updates_parent() {
    let mut interp = make_interpreter();
    interp.set_variable("x", Value::Integer(1));

    // Push a new scope
    interp.env_push(HashMap::new());

    // env_set_mut should update the parent scope
    interp.env_set_mut("x".to_string(), Value::Integer(2));

    interp.env_pop();
    assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(2));
}

// Test format_string_with_values
#[test]
fn test_format_string_with_values() {
    let result = Interpreter::format_string_with_values(
        "Hello {} and {}!",
        &[Value::from_string("world".to_string()), Value::Integer(42)],
    );
    assert_eq!(result, "Hello world and 42!");
}

#[test]
fn test_format_string_with_values_fewer_args() {
    let result = Interpreter::format_string_with_values(
        "Hello {} and {}!",
        &[Value::from_string("world".to_string())],
    );
    assert_eq!(result, "Hello world and {}!");
}

// Test eval_literal
#[test]
fn test_eval_literal_unit() {
    let interp = make_interpreter();
    let result = interp.eval_literal(&Literal::Unit);
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_eval_literal_char() {
    let interp = make_interpreter();
    let result = interp.eval_literal(&Literal::Char('a'));
    assert_eq!(result, Value::from_string("a".to_string()));
}

// Test eval_list_expr
#[test]
fn test_eval_list_expr() {
    let mut interp = make_interpreter();
    let elements = vec![
        make_expr(ExprKind::Literal(Literal::Integer(1, None))),
        make_expr(ExprKind::Literal(Literal::Integer(2, None))),
    ];
    let result = interp.eval_list_expr(&elements).unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], Value::Integer(1));
        assert_eq!(arr[1], Value::Integer(2));
    } else {
        panic!("Expected Array");
    }
}

// Test eval_tuple_expr
#[test]
fn test_eval_tuple_expr() {
    let mut interp = make_interpreter();
    let elements = vec![
        make_expr(ExprKind::Literal(Literal::Integer(1, None))),
        make_expr(ExprKind::Literal(Literal::String("hello".to_string()))),
    ];
    let result = interp.eval_tuple_expr(&elements).unwrap();
    if let Value::Tuple(tuple) = result {
        assert_eq!(tuple.len(), 2);
    } else {
        panic!("Expected Tuple");
    }
}

// Test eval_block_expr
#[test]
fn test_eval_block_expr_empty() {
    let mut interp = make_interpreter();
    let result = interp.eval_block_expr(&[]).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_eval_block_expr_single() {
    let mut interp = make_interpreter();
    let stmts = vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))];
    let result = interp.eval_block_expr(&stmts).unwrap();
    assert_eq!(result, Value::Integer(42));
}

// Test eval_return_expr
#[test]
fn test_eval_return_expr_with_value() {
    let mut interp = make_interpreter();
    let val_expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
    let result = interp.eval_return_expr(Some(&val_expr));
    // eval_return_expr always returns Err(Return(...)) to signal early return
    match result {
        Err(InterpreterError::Return(v)) => assert_eq!(v, Value::Integer(42)),
        _ => panic!("Expected Return error"),
    }
}

#[test]
fn test_eval_return_expr_without_value() {
    let mut interp = make_interpreter();
    let result = interp.eval_return_expr(None);
    // eval_return_expr always returns Err(Return(...)) to signal early return
    match result {
        Err(InterpreterError::Return(v)) => assert_eq!(v, Value::Nil),
        _ => panic!("Expected Return error"),
    }
}

// Test eval_string
#[test]
fn test_eval_string_simple() {
    let mut interp = make_interpreter();
    let result = interp.eval_string("42").unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_eval_string_expression() {
    let mut interp = make_interpreter();
    let result = interp.eval_string("2 + 3").unwrap();
    assert_eq!(result, Value::Integer(5));
}

// Test json operations
#[test]
fn test_json_parse_object() {
    let interp = make_interpreter();
    let result = interp.json_parse(r#"{"a": 1}"#).unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(obj.get("a"), Some(&Value::Integer(1)));
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_json_parse_array() {
    let interp = make_interpreter();
    let result = interp.json_parse("[1, 2, 3]").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_json_stringify() {
    let interp = make_interpreter();
    let result = interp.json_stringify(&Value::Integer(42)).unwrap();
    assert_eq!(result, Value::from_string("42".to_string()));
}

// Test serde_to_value
#[test]
fn test_serde_to_value_null() {
    let result = Interpreter::serde_to_value(&serde_json::Value::Null).unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_serde_to_value_bool() {
    let result = Interpreter::serde_to_value(&serde_json::Value::Bool(true)).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_serde_to_value_number_int() {
    let result = Interpreter::serde_to_value(&serde_json::json!(42)).unwrap();
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_serde_to_value_number_float() {
    let result = Interpreter::serde_to_value(&serde_json::json!(3.14)).unwrap();
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_serde_to_value_string() {
    let result = Interpreter::serde_to_value(&serde_json::json!("hello")).unwrap();
    assert_eq!(result, Value::from_string("hello".to_string()));
}

// Test value_to_serde
#[test]
fn test_value_to_serde_nil() {
    let result = Interpreter::value_to_serde(&Value::Nil).unwrap();
    assert_eq!(result, serde_json::Value::Null);
}

#[test]
fn test_value_to_serde_bool() {
    let result = Interpreter::value_to_serde(&Value::Bool(true)).unwrap();
    assert_eq!(result, serde_json::Value::Bool(true));
}

#[test]
fn test_value_to_serde_integer() {
    let result = Interpreter::value_to_serde(&Value::Integer(42)).unwrap();
    assert_eq!(result, serde_json::json!(42));
}

#[test]
fn test_value_to_serde_float() {
    let result = Interpreter::value_to_serde(&Value::Float(3.14)).unwrap();
    assert_eq!(result, serde_json::json!(3.14));
}

#[test]
fn test_value_to_serde_string() {
    let result = Interpreter::value_to_serde(&Value::from_string("hello".to_string())).unwrap();
    assert_eq!(result, serde_json::json!("hello"));
}

// Test eval_contains
#[test]
fn test_eval_contains_array_found() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.eval_contains(&Value::Integer(1), &arr).unwrap();
    assert!(result);
}

#[test]
fn test_eval_contains_array_not_found() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.eval_contains(&Value::Integer(3), &arr).unwrap();
    assert!(!result);
}

#[test]
fn test_eval_contains_string_found() {
    let interp = make_interpreter();
    let s = Value::from_string("hello world".to_string());
    let result = interp
        .eval_contains(&Value::from_string("world".to_string()), &s)
        .unwrap();
    assert!(result);
}

#[test]
fn test_eval_contains_string_not_found() {
    let interp = make_interpreter();
    let s = Value::from_string("hello world".to_string());
    let result = interp
        .eval_contains(&Value::from_string("foo".to_string()), &s)
        .unwrap();
    assert!(!result);
}

// Test eval_range_expr
#[test]
fn test_eval_range_expr_exclusive() {
    let mut interp = make_interpreter();
    let start = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
    let end = make_expr(ExprKind::Literal(Literal::Integer(5, None)));
    let result = interp.eval_range_expr(&start, &end, false).unwrap();
    if let Value::Range {
        start: s,
        end: e,
        inclusive,
    } = result
    {
        assert_eq!(*s, Value::Integer(0));
        assert_eq!(*e, Value::Integer(5));
        assert!(!inclusive);
    } else {
        panic!("Expected Range");
    }
}

#[test]
fn test_eval_range_expr_inclusive() {
    let mut interp = make_interpreter();
    let start = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
    let end = make_expr(ExprKind::Literal(Literal::Integer(10, None)));
    let result = interp.eval_range_expr(&start, &end, true).unwrap();
    if let Value::Range {
        start: s,
        end: e,
        inclusive,
    } = result
    {
        assert_eq!(*s, Value::Integer(1));
        assert_eq!(*e, Value::Integer(10));
        assert!(inclusive);
    } else {
        panic!("Expected Range");
    }
}

// ===== GC Operations Tests =====

#[test]
fn test_gc_track() {
    let mut interp = make_interpreter();
    // gc_track returns a usize ID for the tracked value
    let _id = interp.gc_track(Value::Integer(42));
    // Successfully tracking a value is the test - ID is a valid usize
}

#[test]
fn test_gc_collect() {
    let mut interp = make_interpreter();
    interp.gc_track(Value::Integer(1));
    interp.gc_track(Value::Integer(2));
    let stats = interp.gc_collect();
    // Stats should be returned
    let _ = stats.collections; // Verify field exists
}

#[test]
fn test_gc_stats() {
    let interp = make_interpreter();
    let stats = interp.gc_stats();
    let _ = stats.collections; // Verify field exists
}

#[test]
fn test_gc_info() {
    let interp = make_interpreter();
    let info = interp.gc_info();
    let _ = info.tracked_count; // Verify field exists
}

#[test]
fn test_gc_set_threshold() {
    let mut interp = make_interpreter();
    interp.gc_set_threshold(1000);
    // No panic = success
}

#[test]
fn test_gc_set_auto_collect() {
    let mut interp = make_interpreter();
    interp.gc_set_auto_collect(false);
    interp.gc_set_auto_collect(true);
    // No panic = success
}

#[test]
fn test_gc_clear() {
    let mut interp = make_interpreter();
    interp.gc_track(Value::Integer(1));
    interp.gc_clear();
    // No panic = success
}

#[test]
fn test_gc_alloc_array() {
    let mut interp = make_interpreter();
    let arr = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
    if let Value::Array(a) = arr {
        assert_eq!(a.len(), 2);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_gc_alloc_string() {
    let mut interp = make_interpreter();
    let s = interp.gc_alloc_string("hello".to_string());
    assert_eq!(s, Value::from_string("hello".to_string()));
}

#[test]
fn test_gc_alloc_closure() {
    let mut interp = make_interpreter();
    let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
    let env = Rc::new(RefCell::new(HashMap::new()));
    let closure = interp.gc_alloc_closure(vec![("x".to_string(), None)], body, env);
    assert!(matches!(closure, Value::Closure { .. }));
}

// ===== Inline Cache Operations Tests =====

#[test]
fn test_get_field_cached_string_len() {
    let mut interp = make_interpreter();
    let s = Value::from_string("hello".to_string());
    let result = interp.get_field_cached(&s, "len").unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_get_field_cached_string_to_upper() {
    let mut interp = make_interpreter();
    let s = Value::from_string("hello".to_string());
    let result = interp.get_field_cached(&s, "to_upper").unwrap();
    assert_eq!(result, Value::from_string("HELLO".to_string()));
}

#[test]
fn test_compute_field_access_string_to_lower() {
    let interp = make_interpreter();
    let s = Value::from_string("HELLO".to_string());
    let result = interp.compute_field_access(&s, "to_lower").unwrap();
    assert_eq!(result, Value::from_string("hello".to_string()));
}

#[test]
fn test_compute_field_access_string_trim() {
    let interp = make_interpreter();
    let s = Value::from_string("  hello  ".to_string());
    let result = interp.compute_field_access(&s, "trim").unwrap();
    assert_eq!(result, Value::from_string("hello".to_string()));
}

#[test]
fn test_compute_field_access_array_len() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.compute_field_access(&arr, "len").unwrap();
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_compute_field_access_array_first() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.compute_field_access(&arr, "first").unwrap();
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_compute_field_access_array_last() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.compute_field_access(&arr, "last").unwrap();
    assert_eq!(result, Value::Integer(2));
}

#[test]
fn test_compute_field_access_array_is_empty() {
    let interp = make_interpreter();
    let arr = Value::Array(Arc::from(vec![]));
    let result = interp.compute_field_access(&arr, "is_empty").unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_compute_field_access_type() {
    let interp = make_interpreter();
    let result = interp
        .compute_field_access(&Value::Integer(42), "type")
        .unwrap();
    assert_eq!(result, Value::from_string("integer".to_string()));
}

#[test]
fn test_compute_field_access_unknown_field() {
    let interp = make_interpreter();
    let result = interp.compute_field_access(&Value::Integer(42), "unknown");
    assert!(result.is_err());
}

#[test]
fn test_get_cache_stats() {
    let mut interp = make_interpreter();
    let s = Value::from_string("test".to_string());
    let _ = interp.get_field_cached(&s, "len");
    let stats = interp.get_cache_stats();
    assert!(!stats.is_empty());
}

#[test]
fn test_clear_caches() {
    let mut interp = make_interpreter();
    let s = Value::from_string("test".to_string());
    let _ = interp.get_field_cached(&s, "len");
    interp.clear_caches();
    assert!(interp.get_cache_stats().is_empty());
}

// ===== Type Feedback Operations Tests =====

#[test]
fn test_record_binary_op_feedback() {
    let mut interp = make_interpreter();
    interp.record_binary_op_feedback(
        0,
        &Value::Integer(1),
        &Value::Integer(2),
        &Value::Integer(3),
    );
    // No panic = success
}

#[test]
fn test_record_variable_assignment_feedback() {
    let mut interp = make_interpreter();
    interp.record_variable_assignment_feedback("x", &Value::Integer(42));
    // No panic = success
}

#[test]
fn test_record_function_call_feedback() {
    let mut interp = make_interpreter();
    interp.record_function_call_feedback(0, "test", &[Value::Integer(1)], &Value::Integer(2));
    // No panic = success
}

#[test]
fn test_get_type_feedback_stats() {
    let mut interp = make_interpreter();
    interp.record_binary_op_feedback(
        0,
        &Value::Integer(1),
        &Value::Integer(2),
        &Value::Integer(3),
    );
    let stats = interp.get_type_feedback_stats();
    let _ = stats.total_operation_sites; // Verify field exists
}

#[test]
fn test_get_specialization_candidates() {
    let interp = make_interpreter();
    let candidates = interp.get_specialization_candidates();
    // Should return a list (empty or populated)
    let _ = candidates.len();
}

#[test]
fn test_clear_type_feedback() {
    let mut interp = make_interpreter();
    interp.record_binary_op_feedback(
        0,
        &Value::Integer(1),
        &Value::Integer(2),
        &Value::Integer(3),
    );
    interp.clear_type_feedback();
    // No panic = success
}

// ===== Error Scope Operations Tests =====

#[test]
fn test_push_pop_error_scope() {
    let mut interp = make_interpreter();
    interp.push_error_scope();
    interp.pop_error_scope();
    // No panic = success
}

// ===== Session Operations Tests =====

#[test]
fn test_get_global_bindings() {
    let interp = make_interpreter();
    let bindings = interp.get_global_bindings();
    // Should contain builtin functions
    assert!(bindings.contains_key("max"));
}

#[test]
fn test_set_global_binding() {
    let mut interp = make_interpreter();
    interp.set_global_binding("test_var".to_string(), Value::Integer(42));
    let bindings = interp.get_global_bindings();
    assert_eq!(bindings.get("test_var"), Some(&Value::Integer(42)));
}

#[test]
fn test_clear_user_variables() {
    let mut interp = make_interpreter();
    interp.set_global_binding("test_var".to_string(), Value::Integer(42));
    interp.clear_user_variables();
    let bindings = interp.get_global_bindings();
    assert!(!bindings.contains_key("test_var"));
}

#[test]
fn test_get_current_bindings() {
    let mut interp = make_interpreter();
    interp.push_scope();
    interp.env_set("local_var".to_string(), Value::Integer(100));
    let bindings = interp.get_current_bindings();
    assert_eq!(bindings.get("local_var"), Some(&Value::Integer(100)));
    interp.pop_scope();
}

// ===== Binary Operations Tests =====

#[test]
fn test_binary_op_add() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(2)).unwrap();
    interp.push(Value::Integer(3)).unwrap();
    interp.binary_op(BinaryOp::Add).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Integer(5));
}

#[test]
fn test_binary_op_sub() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(3)).unwrap();
    interp.binary_op(BinaryOp::Sub).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Integer(7));
}

#[test]
fn test_binary_op_mul() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(4)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Mul).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Integer(20));
}

#[test]
fn test_binary_op_div() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(20)).unwrap();
    interp.push(Value::Integer(4)).unwrap();
    interp.binary_op(BinaryOp::Div).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Integer(5));
}

#[test]
fn test_binary_op_eq() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(5)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Eq).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Bool(true));
}

#[test]
fn test_binary_op_lt() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(3)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Lt).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Bool(true));
}

#[test]
fn test_binary_op_gt() {
    let mut interp = make_interpreter();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Gt).unwrap();
    assert_eq!(interp.pop().unwrap(), Value::Bool(true));
}

#[test]
fn test_apply_binary_op() {
    let interp = make_interpreter();
    let result = interp
        .apply_binary_op(&Value::Integer(2), AstBinaryOp::Add, &Value::Integer(3))
        .unwrap();
    assert_eq!(result, Value::Integer(5));
}

// ===== Pattern Matching Tests =====

#[test]
fn test_pattern_matches_identifier() {
    let mut interp = make_interpreter();
    let pattern = Pattern::Identifier("x".to_string());
    let result = interp
        .pattern_matches(&pattern, &Value::Integer(42))
        .unwrap();
    assert!(result);
}

#[test]
fn test_pattern_matches_wildcard() {
    let mut interp = make_interpreter();
    let pattern = Pattern::Wildcard;
    let result = interp
        .pattern_matches(&pattern, &Value::Integer(42))
        .unwrap();
    assert!(result);
}

#[test]
fn test_pattern_matches_literal_int() {
    let mut interp = make_interpreter();
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let result = interp
        .pattern_matches(&pattern, &Value::Integer(42))
        .unwrap();
    assert!(result);
}

#[test]
fn test_pattern_matches_literal_int_mismatch() {
    let mut interp = make_interpreter();
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let result = interp
        .pattern_matches(&pattern, &Value::Integer(99))
        .unwrap();
    assert!(!result);
}

#[test]
fn test_literal_matches_float() {
    let interp = make_interpreter();
    let result = interp.literal_matches(&Literal::Float(3.14), &Value::Float(3.14));
    assert!(result);
}

#[test]
fn test_literal_matches_string() {
    let interp = make_interpreter();
    let result = interp.literal_matches(
        &Literal::String("hello".to_string()),
        &Value::from_string("hello".to_string()),
    );
    assert!(result);
}

#[test]
fn test_literal_matches_bool() {
    let interp = make_interpreter();
    let result = interp.literal_matches(&Literal::Bool(true), &Value::Bool(true));
    assert!(result);
}

#[test]
fn test_try_pattern_match_identifier() {
    let interp = make_interpreter();
    let pattern = Pattern::Identifier("x".to_string());
    let result = interp
        .try_pattern_match(&pattern, &Value::Integer(42))
        .unwrap();
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
}

#[test]
fn test_pattern_matches_internal() {
    let interp = make_interpreter();
    let pattern = Pattern::Wildcard;
    let result = interp
        .pattern_matches_internal(&pattern, &Value::Integer(42))
        .unwrap();
    assert!(result);
}

// ===== Scope Operations Tests =====

#[test]
fn test_push_pop_scope() {
    let mut interp = make_interpreter();
    let initial_depth = interp.env_stack.len();
    interp.push_scope();
    assert_eq!(interp.env_stack.len(), initial_depth + 1);
    interp.pop_scope();
    assert_eq!(interp.env_stack.len(), initial_depth);
}

// ===== Lookup Variable Tests =====

#[test]
fn test_lookup_option_none() {
    let interp = make_interpreter();
    let result = interp.lookup_variable("Option::None").unwrap();
    if let Value::EnumVariant {
        enum_name,
        variant_name,
        ..
    } = result
    {
        assert_eq!(enum_name, "Option");
        assert_eq!(variant_name, "None");
    } else {
        panic!("Expected EnumVariant");
    }
}

#[test]
fn test_lookup_json_global() {
    let interp = make_interpreter();
    let result = interp.lookup_variable("JSON").unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(
            obj.get("__type"),
            Some(&Value::from_string("JSON".to_string()))
        );
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_lookup_file_global() {
    let interp = make_interpreter();
    let result = interp.lookup_variable("File").unwrap();
    if let Value::Object(obj) = result {
        assert_eq!(
            obj.get("__type"),
            Some(&Value::from_string("File".to_string()))
        );
    } else {
        panic!("Expected Object");
    }
}

// ===== Get Variable Tests =====

#[test]
fn test_get_variable_found() {
    let mut interp = make_interpreter();
    interp.set_variable("test", Value::Integer(42));
    assert_eq!(interp.get_variable("test"), Some(Value::Integer(42)));
}

#[test]
fn test_get_variable_not_found() {
    let interp = make_interpreter();
    assert_eq!(interp.get_variable("nonexistent"), None);
}

// ===== eval_contains Tests =====

#[test]
fn test_eval_contains_tuple() {
    let interp = make_interpreter();
    let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = interp.eval_contains(&Value::Integer(1), &tuple).unwrap();
    assert!(result);
}

#[test]
fn test_eval_contains_object_key_string() {
    let interp = make_interpreter();
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(1));
    let obj = Value::Object(Arc::new(map));
    let result = interp
        .eval_contains(&Value::from_string("key".to_string()), &obj)
        .unwrap();
    assert!(result);
}

#[test]
fn test_eval_contains_object_key_nonstring() {
    let interp = make_interpreter();
    let mut map = HashMap::new();
    map.insert("42".to_string(), Value::Integer(1));
    let obj = Value::Object(Arc::new(map));
    let result = interp.eval_contains(&Value::Integer(42), &obj).unwrap();
    assert!(result);
}

#[test]
fn test_eval_contains_unsupported() {
    let interp = make_interpreter();
    let result = interp.eval_contains(&Value::Integer(1), &Value::Integer(42));
    assert!(result.is_err());
}

#[test]
fn test_eval_contains_string_invalid_element() {
    let interp = make_interpreter();
    let s = Value::from_string("hello".to_string());
    let result = interp.eval_contains(&Value::Integer(1), &s);
    assert!(result.is_err());
}

// ===== eval_literal Tests =====

#[test]
fn test_eval_literal_byte() {
    let interp = make_interpreter();
    let result = interp.eval_literal(&Literal::Byte(42));
    assert_eq!(result, Value::Byte(42));
}

#[test]
fn test_eval_literal_null() {
    let interp = make_interpreter();
    let result = interp.eval_literal(&Literal::Null);
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_eval_literal_atom() {
    let interp = make_interpreter();
    let result = interp.eval_literal(&Literal::Atom("test".to_string()));
    assert_eq!(result, Value::Atom("test".to_string()));
}

// ===== has_stdout Test =====

#[test]
fn test_has_stdout_empty() {
    let interp = make_interpreter();
    assert!(!interp.has_stdout());
}

// ===== set_variable_string Test =====

#[test]
fn test_set_variable_string() {
    let mut interp = make_interpreter();
    interp.set_variable_string("x".to_string(), Value::Integer(42));
    assert_eq!(interp.lookup_variable("x").unwrap(), Value::Integer(42));
}

// ===== eval_function_call_value Test =====

#[test]
fn test_eval_function_call_value() {
    let mut interp = make_interpreter();
    let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
    let env = Rc::new(RefCell::new(HashMap::new()));
    let closure = Value::Closure {
        params: vec![],
        body,
        env,
    };
    let result = interp.eval_function_call_value(&closure, &[]).unwrap();
    assert_eq!(result, Value::Integer(42));
}

// ===== eval_dataframe_literal Test =====

#[test]
fn test_eval_dataframe_literal() {
    use crate::frontend::ast::DataFrameColumn as AstDFColumn;
    let mut interp = make_interpreter();
    let columns = vec![AstDFColumn {
        name: "x".to_string(),
        values: vec![make_expr(ExprKind::Literal(Literal::Integer(1, None)))],
    }];
    let result = interp.eval_dataframe_literal(&columns).unwrap();
    assert!(matches!(result, Value::DataFrame { .. }));
}

// ===== Stack Overflow Test =====

#[test]
fn test_push_stack_overflow() {
    let mut interp = make_interpreter();
    for i in 0..10_000 {
        interp.push(Value::Integer(i as i64)).unwrap();
    }
    // 10001th push should fail
    let result = interp.push(Value::Integer(0));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Stack overflow"));
}

// ===== eval_array_init_expr Test =====

#[test]
fn test_eval_array_init_expr() {
    let mut interp = make_interpreter();
    let value_expr = make_expr(ExprKind::Literal(Literal::Integer(0, None)));
    let size_expr = make_expr(ExprKind::Literal(Literal::Integer(5, None)));
    let result = interp
        .eval_array_init_expr(&value_expr, &size_expr)
        .unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        for v in arr.iter() {
            assert_eq!(*v, Value::Integer(0));
        }
    } else {
        panic!("Expected Array");
    }
}

// ===== eval_special_form Tests =====

#[test]
fn test_eval_special_form_none() {
    let mut interp = make_interpreter();
    let kind = ExprKind::None;
    let result = interp.eval_special_form(&kind).unwrap();
    if let Value::EnumVariant {
        enum_name,
        variant_name,
        data,
    } = result
    {
        assert_eq!(enum_name, "Option");
        assert_eq!(variant_name, "None");
        assert!(data.is_none());
    } else {
        panic!("Expected EnumVariant");
    }
}

#[test]
fn test_eval_special_form_some() {
    let mut interp = make_interpreter();
    let inner = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
    let kind = ExprKind::Some {
        value: Box::new(inner),
    };
    let result = interp.eval_special_form(&kind).unwrap();
    if let Value::EnumVariant {
        enum_name,
        variant_name,
        data,
    } = result
    {
        assert_eq!(enum_name, "Option");
        assert_eq!(variant_name, "Some");
        assert!(data.is_some());
        let values = data.unwrap();
        assert_eq!(values[0], Value::Integer(42));
    } else {
        panic!("Expected EnumVariant");
    }
}
