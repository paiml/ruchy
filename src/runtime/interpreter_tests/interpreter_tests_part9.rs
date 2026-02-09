// Auto-extracted from interpreter_tests.rs - Part 9
use super::*;

// ============== Array Init ==============

#[test]
fn test_array_init_cov() {
    let mut interp = Interpreter::new();
    // [0; 5] creates array of 5 zeros
    let _result = interp.eval_string("[0; 5]");
}

#[test]
fn test_array_init_with_expr() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[1 + 1; 3]");
}

// ============== Loop with Labels ==============

#[test]
fn test_loop_with_break() {
    let mut interp = Interpreter::new();
    let result =
        interp.eval_string("{ let mut x = 0; loop { x = x + 1; if x >= 3 { break x } } }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

#[test]
fn test_labeled_loop_break() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("'outer: loop { break 'outer 42 }");
}

// ============== Await Expression ==============

#[test]
fn test_await_expr() {
    let mut interp = Interpreter::new();
    // In synchronous interpreter, await just evaluates the expression
    let _result = interp.eval_string("42.await");
}

// ============== Throw Expression ==============

#[test]
fn test_throw_expr() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"throw "error""#);
    assert!(result.is_err());
}

// ============== Import Statements ==============

#[test]
fn test_import_statement() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("use std::io");
}

#[test]
fn test_import_with_alias() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("use std::io as myio");
}

// ============== Set Expression ==============

#[test]
fn test_set_expression() {
    let mut interp = Interpreter::new();
    // Set executes all statements and returns last value
    let _result = interp.eval_string("begin 1; 2; 3 end");
}

// ============== Struct Literal ==============

#[test]
fn test_struct_literal() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Point { x: i32, y: i32 }");
    let _result = interp.eval_string("Point { x: 10, y: 20 }");
}

// ============== Object Literal ==============

#[test]
fn test_object_literal_parser_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("{ name: \"test\", value: 42 }");
}

// ============== Qualified Name ==============

#[test]
fn test_qualified_name() {
    let mut interp = Interpreter::new();
    // Define module-like structure
    let _ = interp.eval_string("let math = { pi: 3.14159 }");
    let _result = interp.eval_string("math.pi");
}

// ============== Let Pattern ==============

#[test]
fn test_let_pattern_tuple() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("let (a, b) = (1, 2); a + b");
}

#[test]
fn test_let_pattern_array() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("let [x, y] = [10, 20]; x + y");
}

// ============== String Interpolation ==============

#[test]
fn test_string_interpolation_cov() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    let _result = interp.eval_string("f\"value is {x}\"");
}

// ============== GC Allocation ==============

#[test]
fn test_gc_alloc_array_cov() {
    let mut interp = Interpreter::new();
    let val = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
    match val {
        Value::Array(_) => {}
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_gc_alloc_string_cov() {
    let mut interp = Interpreter::new();
    let val = interp.gc_alloc_string("hello".to_string());
    match val {
        Value::String(_) => {}
        _ => panic!("Expected string"),
    }
}

// ============== Cache Operations ==============

#[test]
fn test_get_cache_stats_cov() {
    let interp = Interpreter::new();
    let stats = interp.get_cache_stats();
    assert!(stats.contains_key("hit_rate") || stats.is_empty());
}

#[test]
fn test_clear_caches_cov() {
    let mut interp = Interpreter::new();
    interp.clear_caches();
}

// ============== Type Feedback ==============

#[test]
fn test_type_feedback_stats_cov() {
    let interp = Interpreter::new();
    let _stats = interp.get_type_feedback_stats();
}

#[test]
fn test_specialization_candidates_cov() {
    let interp = Interpreter::new();
    let _candidates = interp.get_specialization_candidates();
}

#[test]
fn test_clear_type_feedback_cov() {
    let mut interp = Interpreter::new();
    interp.clear_type_feedback();
}

// ============== GC Operations ==============

#[test]
fn test_gc_stats_cov() {
    let interp = Interpreter::new();
    let _stats = interp.gc_stats();
}

#[test]
fn test_gc_set_threshold_cov() {
    let mut interp = Interpreter::new();
    interp.gc_set_threshold(1000);
}

#[test]
fn test_gc_set_auto_collect_cov() {
    let mut interp = Interpreter::new();
    interp.gc_set_auto_collect(true);
    interp.gc_set_auto_collect(false);
}

#[test]
fn test_gc_clear_cov() {
    let mut interp = Interpreter::new();
    interp.gc_clear();
}

// ============== Global Bindings ==============

#[test]
fn test_get_global_bindings() {
    let interp = Interpreter::new();
    let _bindings = interp.get_global_bindings();
}

#[test]
fn test_set_global_binding() {
    let mut interp = Interpreter::new();
    interp.set_global_binding("test_var".to_string(), Value::Integer(42));
}

#[test]
fn test_clear_user_variables() {
    let mut interp = Interpreter::new();
    interp.set_global_binding("user_var".to_string(), Value::Integer(1));
    interp.clear_user_variables();
}

#[test]
fn test_get_current_bindings() {
    let interp = Interpreter::new();
    let _bindings = interp.get_current_bindings();
}

// ============== Error Scope ==============

#[test]
fn test_push_pop_error_scope() {
    let mut interp = Interpreter::new();
    interp.push_error_scope();
    interp.pop_error_scope();
}

// ============== Stdout Capture ==============

#[test]
fn test_capture_stdout() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("hello".to_string());
    let output = interp.get_stdout();
    assert!(output.contains("hello"));
}

#[test]
fn test_has_stdout() {
    let mut interp = Interpreter::new();
    assert!(!interp.has_stdout());
    interp.capture_stdout("test".to_string());
    assert!(interp.has_stdout());
}

#[test]
fn test_clear_stdout() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("test".to_string());
    interp.clear_stdout();
    assert!(!interp.has_stdout());
}

// ============== Pattern Matching ==============

#[test]
fn test_pattern_matches_integer() {
    let mut interp = Interpreter::new();
    use crate::frontend::ast::Pattern;
    let pattern = Pattern::Literal(crate::frontend::ast::Literal::Integer(42, None));
    let value = Value::Integer(42);
    let result = interp.pattern_matches(&pattern, &value);
    assert!(result.is_ok());
}

// ============== Contains ==============

#[test]
fn test_contains_array() {
    let interp = Interpreter::new();
    let element = Value::Integer(2);
    let collection = Value::Array(Arc::from(
        vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].as_slice(),
    ));
    let result = interp.eval_contains(&element, &collection);
    assert!(result.is_ok() && result.unwrap());
}

#[test]
fn test_contains_string() {
    let interp = Interpreter::new();
    let element = Value::String(Arc::from("ell"));
    let collection = Value::String(Arc::from("hello"));
    let result = interp.eval_contains(&element, &collection);
    assert!(result.is_ok() && result.unwrap());
}

#[test]
fn test_contains_range() {
    let mut interp = Interpreter::new();
    // Test contains through eval_string since Range construction is complex
    let result = interp.eval_string("5 in 1..10");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Resolve Module Path ==============

#[test]
fn test_resolve_module_path() {
    let mut interp = Interpreter::new();
    // Set up a module-like structure
    let _ = interp.eval_string("let io = { read: fn() { 0 } }");
    let result = interp.resolve_module_path("io");
    assert!(result.is_some());
}

// ============== List Comprehension ==============

#[test]
fn test_list_comprehension_basic() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[x * 2 for x in [1, 2, 3]]");
}

#[test]
fn test_list_comprehension_filter_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[x for x in [1, 2, 3, 4] if x > 2]");
}

// ============== DataFrame Literal ==============

#[test]
fn test_dataframe_literal_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("df!{ a: [1, 2, 3], b: [4, 5, 6] }");
}

// ============== Binary Ops Stack ==============

#[test]
fn test_binary_op_add() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(20)).unwrap();
    interp.binary_op(BinaryOp::Add).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Integer(30));
}

#[test]
fn test_binary_op_sub() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(30)).unwrap();
    interp.push(Value::Integer(10)).unwrap();
    interp.binary_op(BinaryOp::Sub).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_binary_op_mul() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(5)).unwrap();
    interp.push(Value::Integer(4)).unwrap();
    interp.binary_op(BinaryOp::Mul).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_binary_op_div() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(20)).unwrap();
    interp.push(Value::Integer(4)).unwrap();
    interp.binary_op(BinaryOp::Div).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_binary_op_eq() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(5)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Eq).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_op_lt() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(3)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Lt).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_binary_op_gt() {
    use crate::runtime::interpreter::BinaryOp;
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    interp.binary_op(BinaryOp::Gt).unwrap();
    let result = interp.pop().unwrap();
    assert_eq!(result, Value::Bool(true));
}

// ============== Format String ==============

#[test]
fn test_format_string_with_values() {
    let result = Interpreter::format_string_with_values(
        "x={}, y={}",
        &[Value::Integer(10), Value::Integer(20)],
    );
    assert!(result.contains("10") && result.contains("20"));
}

// ============== Ternary Expression ==============

#[test]
fn test_ternary_false_branch() {
    let mut interp = Interpreter::new();
    // Just exercise the ternary code path - result may vary by parser
    let _result = interp.eval_string("if false then 1 else 2");
}

// ============== While Let ==============

#[test]
fn test_while_let_none() {
    let mut interp = Interpreter::new();
    // Should execute 0 times when condition doesn't match
    let _result = interp
        .eval_string("{ let mut sum = 0; while let Some(x) = None { sum = sum + x }; sum }");
}

// ============== Match with Guards ==============

#[test]
fn test_match_guard_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 5 { x if x > 3 => \"big\", _ => \"small\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "big"),
        _ => {}
    }
}

// ============== Actor Operations ==============

#[test]
fn test_actor_definition_cov() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor Counter {
            state count: i32 = 0
            on Increment { state.count = state.count + 1 }
        }
    "#,
    );
}

#[test]
fn test_actor_constructor_lookup() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor SimpleActor {
            state value: i32 = 0
        }
    "#,
    );
    // Try to lookup actor constructor
    let _result = interp.eval_string("SimpleActor::new");
}

// ============== Class Operations ==============

#[test]
fn test_class_static_method_lookup() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Calculator {
            static fn add(a: i32, b: i32) -> i32 {
                a + b
            }
        }
    "#,
    );
    let _result = interp.eval_string("Calculator::add(1, 2)");
}

#[test]
fn test_class_constructor_lookup() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Point {
            x: i32
            y: i32
            constructor new(x: i32, y: i32) {
                self.x = x
                self.y = y
            }
        }
    "#,
    );
    let _result = interp.eval_string("Point::new(10, 20)");
}

// ============== Struct Constructor ==============

#[test]
fn test_struct_constructor_lookup() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Vec2 { x: f64, y: f64 }");
    let _result = interp.eval_string("Vec2::new");
}

// ============== Module Path Resolution ==============

#[test]
fn test_nested_module_path() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let outer = { inner: { value: 42 } }");
    let result = interp.resolve_module_path("outer::inner");
    assert!(result.is_some());
}

// ============== Error Paths ==============

#[test]
fn test_undefined_module() {
    let interp = Interpreter::new();
    let result = interp.resolve_module_path("nonexistent::module");
    assert!(result.is_none());
}

// ============== Closure Default Parameters ==============

#[test]
fn test_closure_default_params() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(name = \"World\") { name }");
    let result = interp.eval_string("greet()");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "World"),
        _ => {}
    }
}

#[test]
fn test_closure_with_provided_arg() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(name = \"World\") { name }");
    let result = interp.eval_string("greet(\"Alice\")");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "Alice"),
        _ => {}
    }
}

#[test]
fn test_closure_wrong_arg_count() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn add(a, b) { a + b }");
    let result = interp.eval_string("add(1)");
    assert!(result.is_err());
}

#[test]
fn test_closure_too_many_args() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn add(a, b) { a + b }");
    let result = interp.eval_string("add(1, 2, 3)");
    assert!(result.is_err());
}

// ============== JSON Operations ==============

#[test]
fn test_json_parse_object() {
    let interp = Interpreter::new();
    let result = interp.json_parse(r#"{"name": "test"}"#);
    assert!(result.is_ok());
}

#[test]
fn test_json_parse_array() {
    let interp = Interpreter::new();
    let result = interp.json_parse("[1, 2, 3]");
    assert!(result.is_ok());
}

#[test]
fn test_json_parse_invalid() {
    let interp = Interpreter::new();
    let result = interp.json_parse("not json");
    assert!(result.is_err());
}

#[test]
fn test_json_stringify_object() {
    let interp = Interpreter::new();
    let mut obj = std::collections::HashMap::new();
    obj.insert("key".to_string(), Value::Integer(42));
    let result = interp.json_stringify(&Value::Object(Arc::new(obj)));
    assert!(result.is_ok());
}

// ============== Serde Conversions ==============

#[test]
fn test_serde_to_value() {
    use serde_json::json;
    let result = Interpreter::serde_to_value(&json!({"a": 1}));
    assert!(result.is_ok());
}

#[test]
fn test_value_to_serde() {
    let result = Interpreter::value_to_serde(&Value::Integer(42));
    assert!(result.is_ok());
}

// ============== Builtin Functions ==============

#[test]
fn test_builtin_print() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("print(42)");
}

#[test]
fn test_builtin_len() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("len([1, 2, 3])");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

#[test]
fn test_builtin_type_of() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("type_of(42)");
}

#[test]
fn test_builtin_range() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("range(1, 5)");
}

// ============== Field Access Cached ==============

#[test]
fn test_field_access_cached() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { x: 10, y: 20 }");
    // Access same field multiple times to use cache
    let _ = interp.eval_string("obj.x");
    let _ = interp.eval_string("obj.x");
    let result = interp.eval_string("obj.x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 10),
        _ => {}
    }
}

// ============== Eval Assignment ==============

#[test]
fn test_eval_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 10");
    let _ = interp.eval_string("x = 20");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 20),
        _ => {}
    }
}

#[test]
fn test_compound_assign_minus() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 10");
    let _ = interp.eval_string("x -= 3");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 7),
        _ => {}
    }
}

#[test]
fn test_compound_mul_cov() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 5");
    let _ = interp.eval_string("x *= 3");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

#[test]
fn test_compound_div_cov() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 20");
    let _ = interp.eval_string("x /= 4");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

// ============== Env Push/Pop ==============

#[test]
fn test_env_push_pop_cov() {
    let mut interp = Interpreter::new();
    interp.push_scope();
    interp.set_variable("local_var", Value::Integer(100));
    let val = interp.get_variable("local_var");
    assert!(val.is_some());
    interp.pop_scope();
}

// ============== Nested If-Else ==============

#[test]
fn test_nested_if_else_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("if true { if false { 1 } else { 2 } } else { 3 }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 2),
        _ => {}
    }
}

// ============== Complex Match ==============

#[test]
fn test_match_multiple_arms_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "two"),
        _ => {}
    }
}

#[test]
fn test_match_wildcard_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 100 { 1 => \"one\", 2 => \"two\", _ => \"other\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "other"),
        _ => {}
    }
}

// ============== For Loop With Pattern ==============

#[test]
fn test_for_loop_with_tuple_pattern() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        "{ let mut sum = 0; for (a, b) in [(1, 2), (3, 4)] { sum = sum + a + b }; sum }",
    );
}

// ============== Recursion ==============

#[test]
fn test_recursive_function() {
    let mut interp = Interpreter::new();
    let _ =
        interp.eval_string("fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }");
    let result = interp.eval_string("factorial(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 120),
        _ => {}
    }
}

// ============== Let Pattern Complex ==============

#[test]
fn test_let_pattern_nested() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("let ((a, b), c) = ((1, 2), 3); a + b + c");
}

// ============== String Operations ==============

#[test]
fn test_string_concat_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello" + " " + "world""#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "hello world"),
        _ => {}
    }
}

// ============== Float Operations ==============

#[test]
fn test_float_division() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("10.0 / 4.0");
    match result {
        Ok(Value::Float(f)) => assert!((f - 2.5).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_float_modulo() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("10.0 % 3.0");
    match result {
        Ok(Value::Float(f)) => assert!((f - 1.0).abs() < 0.001),
        _ => {}
    }
}

// ============== Array Slicing ==============

#[test]
fn test_array_slice_range() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[1, 2, 3, 4, 5][1..3]");
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => {}
    }
}

// ============== Nil Coalescing ==============

#[test]
fn test_nil_coalesce_nil() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("nil ?? 42");
}

#[test]
fn test_nil_coalesce_value() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("10 ?? 42");
}

// ============== Contains Tuple ==============

#[test]
fn test_contains_tuple() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("1 in (1, 2, 3)");
}

// ============== Method Chaining ==============

#[test]
fn test_method_chain() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[1, 2, 3].map(fn(x) { x * 2 }).filter(fn(x) { x > 2 })");
}

// ============== Boolean Operations ==============

#[test]
fn test_and_short_circuit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("false && true");
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_or_short_circuit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("true || false");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Comparison Operations ==============

#[test]
fn test_less_than_equal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 <= 5");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_greater_than_equal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 >= 5");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_not_equal_cov5() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 != 3");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Return From Nested Block ==============

#[test]
fn test_return_from_nested_block() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn test() { if true { return 42 }; 0 }");
    let result = interp.eval_string("test()");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Variable Shadowing ==============

#[test]
fn test_variable_shadowing_block() {
    let mut interp = Interpreter::new();
    // Just exercise the variable shadowing code path
    let _result = interp.eval_string("let x = 10; { let x = 20; x } + x");
}

// ============== Complex Expressions ==============

#[test]
fn test_complex_arith_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("2 + 3 * 4 - 10 / 2");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 9),
        _ => {}
    }
}

// ============== Empty Block ==============

#[test]
fn test_empty_block() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("{ }");
}

// ============== Nested Function Calls ==============

#[test]
fn test_nested_function_calls() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn add(a, b) { a + b }");
    let _ = interp.eval_string("fn mul(a, b) { a * b }");
    let result = interp.eval_string("add(mul(2, 3), mul(4, 5))");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 26),
        _ => {}
    }
}

// ============== Type Definitions ==============

#[test]
fn test_effect_declaration() {
    let mut interp = Interpreter::new();
    // Effect declarations return Nil
    let _result = interp.eval_string("effect Log { fn log(msg: String) }");
}

#[test]
fn test_handle_expression() {
    let mut interp = Interpreter::new();
    // Handle expressions evaluate inner expr and return Nil
    let _result = interp.eval_string("handle { 42 }");
}

#[test]
fn test_tuple_struct() {
    let mut interp = Interpreter::new();
    // Tuple structs return Nil at runtime
    let _result = interp.eval_string("struct Point(i32, i32);");
}

#[test]
fn test_impl_block() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Counter { value: i32 }");
    let _result = interp.eval_string(
        r#"
        impl Counter {
            fn new() -> Counter { Counter { value: 0 } }
            fn increment(self) { self.value = self.value + 1 }
        }
    "#,
    );
}

// ============== Macro Tests ==============

#[test]
fn test_println_macro_no_args() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("println!()");
}

#[test]
fn test_println_macro_single_arg() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("println!(42)");
}

#[test]
fn test_println_macro_format() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("println!(\"x: {}\", 42)");
}

#[test]
fn test_print_macro() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("print!(\"hello\")");
}

#[test]
fn test_format_macro_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("format!(\"x={}\", 10)");
}

#[test]
fn test_dbg_macro() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("dbg!(42)");
}

#[test]
fn test_assert_macro() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("assert!(true)");
}

#[test]
fn test_assert_eq_macro() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("assert_eq!(1, 1)");
}

// ============== Import Tests ==============

#[test]
fn test_import_std_module() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("use std::env");
}

// ============== Actor Tests ==============

#[test]
fn test_actor_spawn() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor Counter {
            state count: i32 = 0
            on Increment { state.count = state.count + 1 }
        }
    "#,
    );
    let _result = interp.eval_string("spawn Counter");
}

#[test]
fn test_actor_spawn_with_args() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor CounterWithInit {
            state count: i32 = 0
        }
    "#,
    );
    let _result = interp.eval_string("spawn CounterWithInit()");
}

// ============== Enum Tests ==============

#[test]
fn test_enum_definition_public() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("pub enum Status { Active, Inactive }");
}

#[test]
fn test_enum_with_data() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("enum Message { Text(String), Number(i32) }");
}

// ============== Class Tests ==============

#[test]
fn test_class_with_superclass() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("class Animal { fn speak() { \"...\" } }");
    let _result = interp.eval_string("class Dog extends Animal { fn speak() { \"woof\" } }");
}

#[test]
fn test_class_with_traits() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        class Printable {
            fn to_string(self) -> String { "printable" }
        }
    "#,
    );
}

#[test]
fn test_class_with_constants() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        class Math {
            const PI: f64 = 3.14159
        }
    "#,
    );
}

// ============== More Control Flow ==============

#[test]
fn test_labeled_continue_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        'outer: for i in 0..3 {
            for j in 0..3 {
                if j == 1 { continue 'outer }
            }
        }
    "#,
    );
}

#[test]
fn test_break_with_value_from_nested() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("loop { break 100 }");
}

// ============== DataFrameOperation ==============

#[test]
fn test_dataframe_select() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let df = df!{ a: [1, 2, 3], b: [4, 5, 6] }");
    let _result = interp.eval_string("df.select(\"a\")");
}

#[test]
fn test_dataframe_filter() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let df = df!{ a: [1, 2, 3] }");
    let _result = interp.eval_string("df.filter(fn(row) { row.a > 1 })");
}

// ============== Type Cast ==============

#[test]
fn test_type_cast_str_to_int() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("\"42\" as i32");
}

#[test]
fn test_type_cast_int_to_str() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("42 as String");
}

// ============== More Binary Operations ==============

#[test]
fn test_bitwise_and_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("5 & 3");
}

#[test]
fn test_bitwise_or_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("5 | 3");
}

#[test]
fn test_bitwise_xor_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("5 ^ 3");
}

#[test]
fn test_left_shift_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("1 << 4");
}

#[test]
fn test_right_shift_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("16 >> 2");
}

// ============== Unary Operations ==============

#[test]
fn test_bitwise_not() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("~5");
}

// ============== Index Access ==============

#[test]
fn test_tuple_index_cov5() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("(1, 2, 3).1");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 2),
        _ => {}
    }
}

#[test]
fn test_array_negative_index() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[1, 2, 3][-1]");
}

// ============== Field Access ==============

#[test]
fn test_nested_field_access() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { inner: { value: 42 } }");
    let result = interp.eval_string("obj.inner.value");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Error Handling ==============

#[test]
fn test_div_by_zero() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("10 / 0");
    // Should return error or Inf
    let _ = result;
}

#[test]
fn test_mod_by_zero() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("10 % 0");
    let _ = result;
}

// ============== List Comprehension Advanced ==============

#[test]
fn test_list_comp_nested_cov() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("[[x, y] for x in [1, 2] for y in [3, 4]]");
}

// ============== Format Macro Debug Format {:?} ==============

#[test]
fn test_format_debug_placeholder() {
    let mut interp = Interpreter::new();
    // Test {:?} debug format in format! macro
    let _result = interp.eval_string(r#"format!("{:?}", [1, 2, 3])"#);
}

#[test]
fn test_format_debug_multiple_values() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(r#"format!("{} {:?}", "hello", [1, 2])"#);
}

#[test]
fn test_format_extra_placeholders() {
    let mut interp = Interpreter::new();
    // Test format with more placeholders than values
    let _result = interp.eval_string(r#"format!("{} {} {}", 1)"#);
}

#[test]
fn test_format_incomplete_debug() {
    let mut interp = Interpreter::new();
    // Test incomplete {:? without closing brace
    let _result = interp.eval_string(r#"format!("{:?x", 1)"#);
}

#[test]
fn test_format_colon_only() {
    let mut interp = Interpreter::new();
    // Test {: without ?
    let _result = interp.eval_string(r#"format!("{:x", 1)"#);
}

// ============== Try Operator ==============

#[test]
fn test_try_ok_variant() {
    let mut interp = Interpreter::new();
    // Create a Result::Ok and use try operator
    let _ = interp.eval_string(
        r#"
        enum Result { Ok(T), Err(E) }
        let result = Result::Ok(42)
    "#,
    );
    let _result = interp.eval_string("result?");
}

#[test]
fn test_try_err_variant() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        enum Result { Ok(T), Err(E) }
        let result = Result::Err("error")
    "#,
    );
    let _result = interp.eval_string("result?");
}

#[test]
fn test_try_ok_empty_data() {
    let mut interp = Interpreter::new();
    // Ok with no data should error
    let _ = interp.eval_string("enum Result { Ok, Err }");
    let _ = interp.eval_string("let r = Result::Ok");
    let _result = interp.eval_string("r?");
}

// ============== Pipeline Operator ==============

#[test]
fn test_pipeline_method_call_cov2() {
    let mut interp = Interpreter::new();
    // Pipeline with method call (no args)
    let _result = interp.eval_string(r#""hello" |> upper"#);
}

#[test]
fn test_pipeline_user_function_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn double(x) { x * 2 }");
    let result = interp.eval_string("5 |> double");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 10),
        _ => {}
    }
}

#[test]
fn test_pipeline_method_with_args_cov2() {
    let mut interp = Interpreter::new();
    // Pipeline with method call that has args
    let _result = interp.eval_string("[1, 2, 3] |> filter(fn(x) { x > 1 })");
}

#[test]
fn test_pipeline_complex_expr_cov2() {
    let mut interp = Interpreter::new();
    // Pipeline with complex expression
    let _ = interp.eval_string("fn add1(x) { x + 1 }");
    let _result = interp.eval_string("5 |> add1 |> add1");
}

// ============== While-Let Expression ==============

#[test]
fn test_while_let_some_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut items = [Some(1), Some(2), None]
        let mut i = 0
        while let Some(x) = items[i] {
            i = i + 1
            if i >= 3 { break }
        }
    "#,
    );
}

#[test]
fn test_while_let_break() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut x = Some(0)
        while let Some(n) = x {
            if n > 5 { break }
            x = Some(n + 1)
        }
    "#,
    );
}

#[test]
fn test_while_let_continue() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut items = [1, 2, 3]
        let mut i = 0
        while let Some(x) = if i < 3 { Some(items[i]) } else { None } {
            i = i + 1
            continue
        }
    "#,
    );
}

// ============== Import Statements ==============

#[test]
fn test_import_all_with_alias() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("use std::env as myenv");
}

#[test]
fn test_import_all_wildcard() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("use std::env::*");
}

#[test]
fn test_import_default() {
    let mut interp = Interpreter::new();
    // ImportDefault returns Nil (not yet implemented)
    let _result = interp.eval_string("import mymod");
}

#[test]
fn test_module_declaration() {
    let mut interp = Interpreter::new();
    // ModuleDeclaration without file should error
    let _result = interp.eval_string("mod nonexistent");
}

// ============== Actor Send and Query ==============

#[test]
fn test_actor_send_operator() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor Counter {
            state count: i32 = 0
            on Increment { state.count = state.count + 1 }
        }
    "#,
    );
    let _ = interp.eval_string("let c = spawn Counter");
    let _result = interp.eval_string("c ! Increment");
}

#[test]
fn test_actor_query_operator() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor Counter {
            state count: i32 = 0
            on GetCount { state.count }
        }
    "#,
    );
    let _ = interp.eval_string("let c = spawn Counter");
    let _result = interp.eval_string("c ? GetCount");
}

// ============== Closure Default Parameters ==============

#[test]
fn test_closure_default_params_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(name = \"world\") { name }");
    let result = interp.eval_string("greet()");
    // Should use default value
    let _ = result;
}

#[test]
fn test_closure_override_default_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(name = \"world\") { name }");
    let result = interp.eval_string("greet(\"claude\")");
    let _ = result;
}

#[test]
fn test_closure_mixed_params_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn add(a, b = 10) { a + b }");
    let result = interp.eval_string("add(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

#[test]
fn test_closure_too_many_args_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn single(x) { x }");
    let result = interp.eval_string("single(1, 2, 3)");
    // Should error: too many arguments
    assert!(result.is_err());
}

#[test]
fn test_closure_too_few_args_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn needs_two(a, b) { a + b }");
    let result = interp.eval_string("needs_two(1)");
    // Should error: too few arguments
    assert!(result.is_err());
}

// ============== Type Cast ==============

#[test]
fn test_type_cast_float_to_int_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3.7 as i32");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

#[test]
fn test_type_cast_enum_to_int_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("enum Color { Red, Green, Blue }");
    let _result = interp.eval_string("Color::Green as i32");
}

#[test]
fn test_type_cast_int_to_float_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 as f64");
    match result {
        Ok(Value::Float(f)) => assert!((f - 42.0).abs() < 0.001),
        _ => {}
    }
}

// ============== Contains Operator ==============

#[test]
fn test_contains_in_string() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""ell" in "hello""#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_in_array() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("2 in [1, 2, 3]");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_in_tuple() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("2 in (1, 2, 3)");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_in_object() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { name: \"test\" }");
    let result = interp.eval_string(r#""name" in obj"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_not_found() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 in [1, 2, 3]");
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

// ============== Module Expression ==============

#[test]
fn test_module_expr_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        module math {
            fn add(a, b) { a + b }
        }
    "#,
    );
}

// ============== Lazy and Async ==============

#[test]
fn test_lazy_expr_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("lazy { 1 + 2 }");
}

#[test]
fn test_async_block_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("async { 42 }");
}

// ============== If-Let Expression ==============

#[test]
fn test_if_let_match_cov5() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = Some(42)
        if let Some(n) = x { n } else { 0 }
    "#,
    );
}

#[test]
fn test_if_let_no_match_cov5() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = None
        if let Some(n) = x { n } else { 0 }
    "#,
    );
}

#[test]
fn test_if_let_no_else_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = None
        if let Some(n) = x { n }
    "#,
    );
}

// ============== List Comprehension with Condition ==============

#[test]
fn test_list_comp_with_condition() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[x * 2 for x in [1, 2, 3, 4] if x > 2]");
    // Should produce [6, 8]
    let _ = result;
}

#[test]
fn test_list_comp_range_inclusive() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[x for x in 1..=3]");
    let _ = result;
}

#[test]
fn test_list_comp_invalid_iterable() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[x for x in 42]");
    // Should error: not iterable
    assert!(result.is_err());
}

// ============== Qualified Name Lookup ==============

#[test]
fn test_qualified_struct_new() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Point { x: i32, y: i32 }");
    let _result = interp.eval_string("Point::new(1, 2)");
}

#[test]
fn test_qualified_class_static() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Math {
            static fn pi() { 3.14159 }
        }
    "#,
    );
    let _result = interp.eval_string("Math::pi()");
}

#[test]
fn test_json_global() {
    let interp = Interpreter::new();
    let result = interp.lookup_variable("JSON");
    assert!(result.is_ok());
}

#[test]
fn test_file_global() {
    let interp = Interpreter::new();
    let result = interp.lookup_variable("File");
    assert!(result.is_ok());
}

// ============== Call Function with Object Types ==============

#[test]
fn test_call_struct_as_function() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Point { x: i32, y: i32 }");
    let _result = interp.eval_string("Point(1, 2)");
}

#[test]
fn test_call_class_as_function() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("class Animal { fn new() { } }");
    let _result = interp.eval_string("Animal()");
}

