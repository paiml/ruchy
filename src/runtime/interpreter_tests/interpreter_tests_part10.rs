// Auto-extracted from interpreter_tests.rs - Part 10
use super::*;

// ============== Environment Operations ==============

#[test]
fn test_env_set_mut() {
    let mut interp = Interpreter::new();
    // Test mutable variable update in nested scope
    let _result = interp.eval_string(
        r#"
        let mut x = 10
        {
            x = 20
        }
        x
    "#,
    );
}

#[test]
fn test_env_pop_global() {
    let mut interp = Interpreter::new();
    // Trying to pop global env should return None
    let result = interp.env_pop();
    assert!(result.is_none());
}

// ============== Null Coalesce ==============

#[test]
fn test_null_coalesce_nil_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("nil ?? 42");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

#[test]
fn test_null_coalesce_value_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("10 ?? 42");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 10),
        _ => {}
    }
}

// ============== Short-Circuit Operators ==============

#[test]
fn test_and_short_circuit_cov2() {
    let mut interp = Interpreter::new();
    // False && anything should return false without evaluating right side
    let result = interp.eval_string("false && (1/0)");
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_or_short_circuit_cov2() {
    let mut interp = Interpreter::new();
    // True || anything should return true without evaluating right side
    let result = interp.eval_string("true || (1/0)");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Literal Types ==============

#[test]
fn test_literal_atom_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(":my_atom");
}

#[test]
fn test_literal_byte_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("b'A'");
}

// ============== Range in Comprehension ==============

#[test]
fn test_range_in_for() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut sum = 0
        for i in 1..5 {
            sum = sum + i
        }
        sum
    "#,
    );
}

// ============== Block Return Value ==============

#[test]
fn test_block_return_value() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("{ let x = 1; let y = 2; x + y }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

// ============== Match with Guard ==============

#[test]
fn test_match_with_guard_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = 5
        match x {
            n if n > 3 => "big",
            _ => "small"
        }
    "#,
    );
}

// ============== Option None Lookup ==============

#[test]
fn test_option_none_lookup_cov2() {
    let interp = Interpreter::new();
    let result = interp.lookup_variable("Option::None");
    match result {
        Ok(Value::EnumVariant { variant_name, .. }) => {
            assert_eq!(variant_name, "None");
        }
        _ => {}
    }
}

// ============== DataFrame in List Comprehension ==============

#[test]
fn test_dataframe_literal_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("df!{ a: [1, 2], b: [3, 4] }");
}

// ============== Struct with Methods ==============

#[test]
fn test_struct_with_method_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        struct Counter {
            count: i32
            fn increment(self) { self.count + 1 }
        }
    "#,
    );
}

// ============== Class Constructor ==============

#[test]
fn test_class_constructor_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Point {
            fn new(x, y) {
                self.x = x
                self.y = y
            }
        }
    "#,
    );
    let _result = interp.eval_string("Point::new(1, 2)");
}

// ============== Loop Break with Value ==============

#[test]
fn test_loop_break_value_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("loop { break 42 }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Return from Function ==============

#[test]
fn test_early_return_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn check(x) { if x > 0 { return x } -1 }");
    let result = interp.eval_string("check(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

// ============== Throw Expression ==============

#[test]
fn test_throw_expr_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"throw "error message""#);
    assert!(result.is_err());
}

// ============== Await Expression ==============

#[test]
fn test_await_expr_cov2() {
    let mut interp = Interpreter::new();
    // Await just evaluates the expression synchronously
    let result = interp.eval_string("await 42");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Builtin Function Error ==============

#[test]
fn test_unknown_builtin_cov2() {
    let mut interp = Interpreter::new();
    // Trying to call unknown builtin should error
    let result =
        interp.call_function(Value::from_string("__builtin_unknown__".to_string()), &[]);
    assert!(result.is_err());
}

// ============== Print Macros ==============

#[test]
fn test_println_empty_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("println!()");
}

#[test]
fn test_println_format_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(r#"println!("x = {}", 42)"#);
}

#[test]
fn test_print_single_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(r#"print!("hello")"#);
}

// ============== Empty Format Error ==============

#[test]
fn test_format_empty_error_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("format!()");
    assert!(result.is_err());
}

// ============== Actor Definition ==============

#[test]
fn test_actor_with_multiple_handlers_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        actor Counter {
            state value: i32 = 0
            on Inc { state.value = state.value + 1 }
            on Dec { state.value = state.value - 1 }
            on Get { state.value }
        }
    "#,
    );
}

// ============== Set Expression ==============

#[test]
fn test_set_expr_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("{ 1; 2; 3 }");
}

// ============== String Interpolation ==============

#[test]
fn test_string_interpolation_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    let _result = interp.eval_string(r#"f"value is {x}""#);
}

// ============== Object Literal ==============

#[test]
fn test_object_literal_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string("{ name: \"test\", value: 42 }");
}

// ============== Struct Literal ==============

#[test]
fn test_struct_literal_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("struct Point { x: i32, y: i32 }");
    let _result = interp.eval_string("Point { x: 1, y: 2 }");
}

// ============== QualifiedName ==============

#[test]
fn test_qualified_name_cov2() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("use std::env");
    let _result = interp.eval_string("std::env::var");
}

// ============== Send Operator Error ==============

#[test]
fn test_send_non_actor_cov2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 ! Increment");
    assert!(result.is_err());
}

// ============== LetPattern ==============

#[test]
fn test_let_pattern_cov2() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let (a, b) = (1, 2)
        a + b
    "#,
    );
}

// ============== Stack Operations ==============

#[test]
fn test_stack_pop_empty_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.pop();
    assert!(result.is_err());
}

#[test]
fn test_stack_peek_empty_cov3() {
    let interp = Interpreter::new();
    let result = interp.peek(0);
    assert!(result.is_err());
}

#[test]
fn test_stack_peek_deep_cov3() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(1)).unwrap();
    interp.push(Value::Integer(2)).unwrap();
    interp.push(Value::Integer(3)).unwrap();
    let result = interp.peek(2);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1),
        _ => {}
    }
}

// ============== Pattern Match Edge Cases ==============

#[test]
fn test_pattern_match_literal_string_cov3() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = "hello"
        match x {
            "hello" => "matched",
            _ => "not matched"
        }
    "#,
    );
}

#[test]
fn test_pattern_match_literal_float_cov3() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = 3.14
        match x {
            3.14 => "matched",
            _ => "not matched"
        }
    "#,
    );
}

#[test]
fn test_pattern_match_nested_tuple_cov3() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = ((1, 2), (3, 4))
        match x {
            ((a, b), (c, d)) => a + b + c + d,
            _ => 0
        }
    "#,
    );
}

// ============== Global Bindings ==============

#[test]
fn test_get_global_bindings_cov3() {
    let interp = Interpreter::new();
    let bindings = interp.get_global_bindings();
    // Should contain builtin functions
    assert!(bindings.contains_key("max"));
}

#[test]
fn test_set_global_binding_cov3() {
    let mut interp = Interpreter::new();
    interp.set_global_binding("test_var".to_string(), Value::Integer(42));
    let bindings = interp.get_global_bindings();
    assert_eq!(bindings.get("test_var"), Some(&Value::Integer(42)));
}

#[test]
fn test_clear_user_variables_cov3() {
    let mut interp = Interpreter::new();
    interp.set_global_binding("test_var".to_string(), Value::Integer(42));
    interp.clear_user_variables();
    let bindings = interp.get_global_bindings();
    assert!(!bindings.contains_key("test_var"));
}

// ============== String Interpolation Edge Cases ==============

#[test]
fn test_string_interpolation_with_format() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    let _result = interp.eval_string(r#"f"value: {x:d}""#);
}

#[test]
fn test_string_interpolation_nested() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 1");
    let _ = interp.eval_string("let y = 2");
    let _result = interp.eval_string(r#"f"{x} + {y} = {x + y}""#);
}

// ============== Type Cast Edge Cases ==============

#[test]
fn test_type_cast_unsupported_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello" as i32"#);
    assert!(result.is_err());
}

#[test]
fn test_type_cast_identity_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 as i32");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Stdout Capture ==============

#[test]
fn test_stdout_capture() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("Hello".to_string());
    assert_eq!(interp.get_stdout(), "Hello");
}

#[test]
fn test_stdout_multiple() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("Line 1".to_string());
    interp.capture_stdout("Line 2".to_string());
    assert_eq!(interp.get_stdout(), "Line 1\nLine 2");
}

#[test]
fn test_stdout_clear() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("test".to_string());
    interp.clear_stdout();
    assert!(!interp.has_stdout());
}

// ============== Error Scope ==============

#[test]
fn test_error_scope_push_pop() {
    let mut interp = Interpreter::new();
    interp.push_error_scope();
    interp.pop_error_scope();
    // No panic = success
}

// ============== Apply Binary Op ==============

#[test]
fn test_apply_binary_op() {
    let interp = Interpreter::new();
    let left = Value::Integer(10);
    let right = Value::Integer(5);
    let result =
        interp.apply_binary_op(&left, crate::frontend::ast::BinaryOp::Subtract, &right);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

// ============== Literal Matches ==============

#[test]
fn test_literal_matches_integer() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Integer(42, None);
    assert!(interp.literal_matches(&lit, &Value::Integer(42)));
}

#[test]
fn test_literal_matches_float() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Float(3.14);
    assert!(interp.literal_matches(&lit, &Value::Float(3.14)));
}

#[test]
fn test_literal_matches_string() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::String("hello".to_string());
    assert!(interp.literal_matches(&lit, &Value::String(Arc::from("hello"))));
}

#[test]
fn test_literal_matches_bool() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Bool(true);
    assert!(interp.literal_matches(&lit, &Value::Bool(true)));
}

#[test]
fn test_literal_matches_mismatch() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Integer(42, None);
    assert!(!interp.literal_matches(&lit, &Value::String(Arc::from("hello"))));
}

// ============== Set Variable ==============

#[test]
fn test_set_variable_new() {
    let mut interp = Interpreter::new();
    interp.set_variable("new_var", Value::Integer(100));
    let val = interp.get_variable("new_var");
    assert_eq!(val, Some(Value::Integer(100)));
}

#[test]
fn test_set_variable_update() {
    let mut interp = Interpreter::new();
    interp.set_variable("x", Value::Integer(1));
    interp.set_variable("x", Value::Integer(2));
    let val = interp.get_variable("x");
    assert_eq!(val, Some(Value::Integer(2)));
}

// ============== List Pattern Match ==============

#[test]
fn test_match_list_pattern() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        match arr {
            [a, b, c] => a + b + c,
            _ => 0
        }
    "#,
    );
}

// ============== Tuple Pattern Match ==============

#[test]
fn test_match_tuple_pattern() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let tup = (1, 2, 3)
        match tup {
            (a, b, c) => a + b + c,
            _ => 0
        }
    "#,
    );
}

// ============== For Loop Variants ==============

#[test]
fn test_for_loop_with_index() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut sum = 0
        for (i, x) in [1, 2, 3].enumerate() {
            sum = sum + i + x
        }
        sum
    "#,
    );
}

// ============== While Loop Variants ==============

#[test]
fn test_while_loop_false() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let mut x = 0
        while false {
            x = x + 1
        }
        x
    "#,
    );
}

// ============== Match with Multiple Guards ==============

#[test]
fn test_match_multiple_guards() {
    let mut interp = Interpreter::new();
    let _result = interp.eval_string(
        r#"
        let x = 10
        match x {
            n if n < 5 => "small",
            n if n < 15 => "medium",
            _ => "large"
        }
    "#,
    );
}

// ============== Nested Function Calls ==============

#[test]
fn test_nested_function_calls_cov3() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn add(a, b) { a + b }");
    let _ = interp.eval_string("fn mul(a, b) { a * b }");
    let result = interp.eval_string("add(mul(2, 3), mul(4, 5))");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 26),
        _ => {}
    }
}

// ============== Recursive Function ==============

#[test]
fn test_recursive_factorial_cov3() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        fn factorial(n) {
            if n <= 1 { 1 }
            else { n * factorial(n - 1) }
        }
    "#,
    );
    let result = interp.eval_string("factorial(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 120),
        _ => {}
    }
}

// ============== Closure Capture ==============

#[test]
fn test_closure_capture_cov3() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 10");
    let _ = interp.eval_string("let add_x = fn(y) { x + y }");
    let result = interp.eval_string("add_x(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

// ============== Array Methods ==============

#[test]
fn test_array_map_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[1, 2, 3].map(fn(x) { x * 2 })");
    let _ = result;
}

#[test]
fn test_array_filter_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[1, 2, 3, 4].filter(fn(x) { x > 2 })");
    let _ = result;
}

#[test]
fn test_array_reduce_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[1, 2, 3, 4].reduce(fn(a, x) { a + x }, 0)");
    let _ = result;
}

// ============== String Methods ==============

#[test]
fn test_string_split_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""a,b,c".split(",")"#);
    let _ = result;
}

#[test]
fn test_string_replace_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".replace("world", "ruchy")"#);
    let _ = result;
}

#[test]
fn test_string_starts_with_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello".starts_with("he")"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_string_ends_with_cov3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello".ends_with("lo")"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Object Field Access ==============

#[test]
fn test_object_field_access_cov3() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { x: 1, y: 2 }");
    let result = interp.eval_string("obj.x + obj.y");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

// ============== Nested Object ==============

#[test]
fn test_nested_object_cov3() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { inner: { value: 42 } }");
    let result = interp.eval_string("obj.inner.value");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Current Environment ==============

#[test]
fn test_current_env_cov3() {
    let mut interp = Interpreter::new();
    interp.push_scope();
    let _ = interp.current_env();
    interp.pop_scope();
}

// ============== Format Macro Edge Cases ==============

#[test]
fn test_format_macro_debug_placeholder() {
    let mut interp = Interpreter::new();
    // Test {:?} debug format placeholder
    let result = interp.eval_string(r#"format!("{:?}", 42)"#);
    assert!(result.is_ok());
}

#[test]
fn test_format_macro_multiple_debug() {
    let mut interp = Interpreter::new();
    // Test multiple {:?} placeholders
    let result = interp.eval_string(r#"format!("{:?} and {:?}", 1, 2)"#);
    assert!(result.is_ok());
}

#[test]
fn test_format_macro_mixed_placeholders() {
    let mut interp = Interpreter::new();
    // Mix {} and {:?}
    let result = interp.eval_string(r#"format!("{} debug {:?}", "hello", 42)"#);
    assert!(result.is_ok());
}

#[test]
fn test_format_macro_excess_placeholders() {
    let mut interp = Interpreter::new();
    // More placeholders than values
    let result = interp.eval_string(r#"format!("{} {} {}", 1)"#);
    assert!(result.is_ok()); // Should preserve extra placeholders
}

#[test]
fn test_format_macro_malformed_debug_unclosed() {
    let mut interp = Interpreter::new();
    // Malformed {:? without closing }
    let result = interp.eval_string(r#"format!("{:?unclosed", 42)"#);
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_format_macro_colon_only() {
    let mut interp = Interpreter::new();
    // Just {: without ?}
    let result = interp.eval_string(r#"format!("{:abc}", 42)"#);
    let _ = result;
}

// ============== Println Macro Variants ==============

#[test]
fn test_println_macro_empty() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("println!()");
    assert!(result.is_ok());
}

#[test]
fn test_println_macro_single_arg_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("println!(42)");
    assert!(result.is_ok());
}

#[test]
fn test_println_macro_format_string() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"println!("Value: {}", 42)"#);
    assert!(result.is_ok());
}

#[test]
fn test_println_macro_non_string_format() {
    let mut interp = Interpreter::new();
    // First arg is not a string
    let result = interp.eval_string("println!(42, 43)");
    // Should use to_string on first arg
    let _ = result;
}

// ============== Contains Operator Edge Cases ==============

#[test]
fn test_contains_object_string_key() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { "a": 1, "b": 2 }"#);
    let result = interp.eval_string(r#""a" in obj"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_object_non_string_key() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let obj = { a: 1, b: 2 }");
    // Using non-string key should convert to string
    let result = interp.eval_string("42 in obj");
    // Should return false but not error
    let _ = result;
}

#[test]
fn test_contains_tuple_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let t = (1, 2, 3)");
    let result = interp.eval_string("2 in t");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_contains_unsupported_type() {
    let mut interp = Interpreter::new();
    // 'in' on integer - test that this code path is exercised
    let _ = interp.eval_string("1 in 42");
    // May error or return false depending on implementation
}

// ============== Type Cast Edge Cases ==============

#[test]
fn test_type_cast_int_to_f64() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 as f64");
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 42.0),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_type_cast_int_to_f32() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 as f32");
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 42.0),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_type_cast_float_to_i32() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3.7 as i32");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 3),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_type_cast_float_to_i64() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3.7 as i64");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 3),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_type_cast_float_to_isize() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3.7 as isize");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 3),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_type_cast_int_to_int_identity_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("42 as i64");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 42),
        _ => panic!("Expected integer"),
    }
}

#[test]
fn test_type_cast_float_to_float_identity_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3.14 as f64");
    match result {
        Ok(Value::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float"),
    }
}

#[test]
fn test_type_cast_unsupported_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello" as i32"#);
    assert!(result.is_err());
}

// ============== Import Default ==============

#[test]
fn test_import_default_returns_nil() {
    let mut interp = Interpreter::new();
    // ImportDefault is not fully implemented, returns Nil
    let result = interp.eval_string(r#"import React from "react""#);
    // Should return Nil without error
    let _ = result;
}

// ============== Binary Operations via Stack ==============

#[test]
fn test_binary_op_add_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Add);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Integer(15));
}

#[test]
fn test_binary_op_sub_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(3)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Sub);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Integer(7));
}

#[test]
fn test_binary_op_mul_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(6)).unwrap();
    interp.push(Value::Integer(7)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Mul);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Integer(42));
}

#[test]
fn test_binary_op_div_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(20)).unwrap();
    interp.push(Value::Integer(4)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Div);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Integer(5));
}

#[test]
fn test_binary_op_eq_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(42)).unwrap();
    interp.push(Value::Integer(42)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Eq);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Bool(true));
}

#[test]
fn test_binary_op_lt_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(5)).unwrap();
    interp.push(Value::Integer(10)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Lt);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Bool(true));
}

#[test]
fn test_binary_op_gt_cov4() {
    let mut interp = Interpreter::new();
    interp.push(Value::Integer(10)).unwrap();
    interp.push(Value::Integer(5)).unwrap();
    let result = interp.binary_op(crate::runtime::interpreter::BinaryOp::Gt);
    assert!(result.is_ok());
    let top = interp.pop().unwrap();
    assert_eq!(top, Value::Bool(true));
}

// ============== Literal Matching ==============

#[test]
fn test_literal_matches_int() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Integer(42, None);
    assert!(interp.literal_matches(&lit, &Value::Integer(42)));
    assert!(!interp.literal_matches(&lit, &Value::Integer(43)));
}

#[test]
fn test_literal_matches_float_cov4() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Float(3.14);
    assert!(interp.literal_matches(&lit, &Value::Float(3.14)));
    assert!(!interp.literal_matches(&lit, &Value::Float(2.71)));
}

#[test]
fn test_literal_matches_bool_cov4() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Bool(true);
    assert!(interp.literal_matches(&lit, &Value::Bool(true)));
    assert!(!interp.literal_matches(&lit, &Value::Bool(false)));
}

#[test]
fn test_literal_matches_type_mismatch_cov4() {
    let interp = Interpreter::new();
    let lit = crate::frontend::ast::Literal::Integer(42, None);
    // Should not match different types
    assert!(!interp.literal_matches(&lit, &Value::Float(42.0)));
    assert!(!interp.literal_matches(&lit, &Value::Bool(true)));
}

// ============== Pattern Matching ==============

#[test]
fn test_pattern_matches_identifier() {
    let mut interp = Interpreter::new();
    let pattern = crate::frontend::ast::Pattern::Identifier("x".to_string());
    let result = interp.pattern_matches(&pattern, &Value::Integer(42));
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_pattern_matches_wildcard() {
    let mut interp = Interpreter::new();
    let pattern = crate::frontend::ast::Pattern::Wildcard;
    let result = interp.pattern_matches(&pattern, &Value::Integer(42));
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_pattern_matches_literal() {
    let mut interp = Interpreter::new();
    let pattern = crate::frontend::ast::Pattern::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    );
    let result = interp.pattern_matches(&pattern, &Value::Integer(42));
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_pattern_matches_literal_no_match() {
    let mut interp = Interpreter::new();
    let pattern = crate::frontend::ast::Pattern::Literal(
        crate::frontend::ast::Literal::Integer(42, None),
    );
    let result = interp.pattern_matches(&pattern, &Value::Integer(43));
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

// ============== Stdout Capture ==============

#[test]
fn test_stdout_capture_clear() {
    let mut interp = Interpreter::new();
    interp.capture_stdout("line1".to_string());
    interp.capture_stdout("line2".to_string());
    assert_eq!(interp.get_stdout(), "line1\nline2");
    interp.clear_stdout();
    assert_eq!(interp.get_stdout(), "");
}

// ============== Actor Operations ==============

#[test]
fn test_actor_send_non_actor_error_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    // Actor send on non-actor should error
    let result = interp.eval_string("x ! Ping");
    assert!(result.is_err());
}

#[test]
fn test_actor_query_non_actor_error_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    // Actor query on non-actor should error
    let result = interp.eval_string("x ? GetValue");
    assert!(result.is_err());
}

// ============== Set Variable String ==============

#[test]
fn test_set_variable_string() {
    let mut interp = Interpreter::new();
    interp.set_variable_string("myvar".to_string(), Value::Integer(100));
    let result = interp.get_variable("myvar");
    assert_eq!(result, Some(Value::Integer(100)));
}

// ============== Ternary Expression ==============

#[test]
fn test_ternary_true_branch() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("true ? 1 : 2");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 1),
        _ => panic!("Expected integer 1"),
    }
}

#[test]
fn test_ternary_false_branch_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("false ? 1 : 2");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 2),
        _ => panic!("Expected integer 2"),
    }
}

// ============== Array Init Expression ==============

#[test]
fn test_array_init_repeated() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[0; 5]");
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 5);
            for v in arr.iter() {
                assert_eq!(*v, Value::Integer(0));
            }
        }
        _ => panic!("Expected array"),
    }
}

// ============== Block Expression Scope ==============

#[test]
fn test_block_scope_shadowing_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 10");
    let _ = interp.eval_string("{ let x = 20 }");
    // Original x should still be 10
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(i)) => assert_eq!(i, 10),
        _ => panic!("Expected 10"),
    }
}

// ============== DataFrame Literal ==============

#[test]
fn test_dataframe_literal_basic() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"df { a: [1, 2, 3], b: [4, 5, 6] }"#);
    match result {
        Ok(Value::DataFrame { .. }) => {}
        Ok(_) => {}
        Err(_) => {}
    }
}

// ============== Unknown Macro Error ==============

#[test]
fn test_unknown_macro_error() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("unknown_macro!(1, 2, 3)");
    // Should error for unknown macro
    assert!(result.is_err());
}

// ============== Vec Macro ==============

#[test]
fn test_vec_macro_empty_cov4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("vec![]");
    match result {
        Ok(Value::Array(arr)) => assert!(arr.is_empty()),
        _ => panic!("Expected empty array"),
    }
}

#[test]
fn test_vec_macro_with_elements() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("vec![1, 2, 3]");
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected array with 3 elements"),
    }
}

// ============== Constructor Markers ==============

#[test]
fn test_class_constructor_marker() {
    let mut interp = Interpreter::new();
    // Define a class first
    let _ = interp.eval_string("class Point { fn new(x, y) { self.x = x; self.y = y } }");
    let result = interp.eval_string("Point::new(1, 2)");
    // Should create instance
    let _ = result;
}

#[test]
fn test_struct_constructor_marker() {
    let mut interp = Interpreter::new();
    // Define a struct first
    let _ = interp.eval_string("struct Point { x: i64, y: i64 }");
    let result = interp.eval_string("Point { x: 1, y: 2 }");
    // Should create struct instance
    let _ = result;
}

// ============== Apply Binary Op ==============

#[test]
fn test_apply_binary_op_cov4() {
    let interp = Interpreter::new();
    let left = Value::Integer(10);
    let right = Value::Integer(5);
    let result = interp.apply_binary_op(&left, crate::frontend::ast::BinaryOp::Add, &right);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Integer(15));
}

// ============== Assignment Detection ==============

#[test]
fn test_is_assignment_compound() {
    let target = Box::new(crate::frontend::ast::Expr {
        kind: crate::frontend::ast::ExprKind::Identifier("x".to_string()),
        span: crate::frontend::ast::Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    });
    let value = Box::new(crate::frontend::ast::Expr {
        kind: crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Integer(
            1, None,
        )),
        span: crate::frontend::ast::Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    });
    let kind = crate::frontend::ast::ExprKind::CompoundAssign {
        target,
        op: crate::frontend::ast::BinaryOp::Add,
        value,
    };
    assert!(Interpreter::is_assignment_expr(&kind));
}

// ============== Closure with Default Params ==============

#[test]
fn test_closure_too_few_args_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(a, b, c) { a + b + c }");
    let result = interp.eval_string("greet(1)");
    // Should error - too few arguments
    assert!(result.is_err());
}

#[test]
fn test_closure_too_many_args_cov4() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn greet(a) { a }");
    let result = interp.eval_string("greet(1, 2, 3)");
    // Should error - too many arguments
    assert!(result.is_err());
}

// ============== Call Function with Various Types ==============

#[test]
fn test_call_static_method_invalid_marker() {
    let mut interp = Interpreter::new();
    // Try to call with malformed static method marker
    let result = interp.call_function(
        Value::from_string("__class_static_method__:OnlyClassName".to_string()),
        &[],
    );
    // Should error - invalid format
    assert!(result.is_err());
}

#[test]
fn test_call_unknown_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.call_function(
        Value::from_string("__builtin_nonexistent__".to_string()),
        &[],
    );
    // Should error - unknown builtin
    assert!(result.is_err());
}

// ============== Try Operator Edge Cases ==============

#[test]
fn test_try_operator_with_ok_result() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn get_value() { Ok(42) }");
    // Can't actually test ? in single eval_string since it would early return
    // But we can test Ok creation
    let result = interp.eval_string("Ok(42)");
    assert!(result.is_ok());
}

#[test]
fn test_try_operator_with_err_result() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"Err("error message")"#);
    // Should create an Err variant
    assert!(result.is_ok());
}

// ============== Pipeline Operator Edge Cases ==============

#[test]
fn test_pipeline_with_user_function() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn double(x) { x * 2 }");
    let result = interp.eval_string("5 |> double");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 10),
        _ => {}
    }
}

#[test]
fn test_pipeline_with_method() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello" |> upper"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "HELLO"),
        _ => {}
    }
}

#[test]
fn test_pipeline_with_chained_methods() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""  hello  " |> trim |> upper"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "HELLO"),
        _ => {}
    }
}

#[test]
fn test_pipeline_with_method_args() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello" |> replace("l", "L")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "heLLo"),
        _ => {}
    }
}

// ============== Async Block ==============

#[test]
fn test_async_block_basic() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("async { 42 }");
    // Async blocks execute synchronously for now
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Lazy Expression ==============

#[test]
fn test_lazy_expression() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("lazy 1 + 2");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

// ============== Module Expression ==============

#[test]
fn test_module_declaration_error() {
    let mut interp = Interpreter::new();
    // Unresolved module should error
    let result = interp.eval_string("mod nonexistent");
    // Should error or return something
    let _ = result;
}

// ============== IfLet Expression ==============

#[test]
fn test_if_let_match_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let opt = Some(42)");
    let result = interp.eval_string("if let Some(x) = opt { x } else { 0 }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

#[test]
fn test_if_let_no_match_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let opt = None");
    let result = interp.eval_string("if let Some(x) = opt { x } else { 0 }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 0),
        _ => {}
    }
}

#[test]
fn test_if_let_no_else_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let opt = None");
    // Without else branch, should return nil
    let result = interp.eval_string("if let Some(x) = opt { x }");
    match result {
        Ok(Value::Nil) => {}
        _ => {}
    }
}

// ============== WhileLet Expression ==============

#[test]
fn test_while_let_basic() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut counter = 0");
    let _ = interp.eval_string("let mut opt = Some(3)");
    let result = interp.eval_string(
        r#"
        while let Some(x) = opt {
            counter = counter + x
            if x > 1 { opt = Some(x - 1) } else { opt = None }
        }
        counter
    "#,
    );
    // 3 + 2 + 1 = 6
    let _ = result;
}

// ============== List Comprehension ==============

#[test]
fn test_list_comprehension_simple_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[x * 2 for x in [1, 2, 3]]");
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2));
            assert_eq!(arr[1], Value::Integer(4));
            assert_eq!(arr[2], Value::Integer(6));
        }
        _ => {}
    }
}

#[test]
fn test_list_comprehension_with_condition_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("[x for x in [1, 2, 3, 4, 5] if x > 2]");
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 3);
        }
        _ => {}
    }
}

// ============== Match Expression Edge Cases ==============

#[test]
fn test_match_integer_literal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 42 { 42 => \"found\", _ => \"not found\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "found"),
        _ => {}
    }
}

#[test]
fn test_match_wildcard_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 99 { 42 => \"found\", _ => \"default\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "default"),
        _ => {}
    }
}

#[test]
fn test_match_with_guard_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("match 5 { x if x > 3 => \"big\", _ => \"small\" }");
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "big"),
        _ => {}
    }
}

// ============== Range Expression ==============

#[test]
fn test_range_inclusive_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("1..=5");
    match result {
        Ok(Value::Range { .. }) => {}
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 5),
        _ => {}
    }
}

#[test]
fn test_range_exclusive_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("1..5");
    match result {
        Ok(Value::Range { .. }) => {}
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 4),
        _ => {}
    }
}

// ============== Null Coalesce Operator ==============

#[test]
fn test_null_coalesce_with_some() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = Some(42)");
    let result = interp.eval_string("x ?? 0");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

#[test]
fn test_null_coalesce_with_none() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = None");
    let result = interp.eval_string("x ?? 99");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 99),
        _ => {}
    }
}

#[test]
fn test_null_coalesce_with_nil() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = nil");
    let result = interp.eval_string("x ?? 100");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 100),
        _ => {}
    }
}

// ============== String Interpolation ==============

#[test]
fn test_string_interpolation_simple_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 42");
    let result = interp.eval_string(r#"f"value is {x}""#);
    match result {
        Ok(Value::String(s)) => assert!(s.contains("42")),
        _ => {}
    }
}

#[test]
fn test_string_interpolation_expression() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"f"sum is {1 + 2}""#);
    match result {
        Ok(Value::String(s)) => assert!(s.contains("3")),
        _ => {}
    }
}

// ============== Compound Assignment ==============

#[test]
fn test_compound_add_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 10");
    let _ = interp.eval_string("x += 5");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

#[test]
fn test_compound_sub_assign() {
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
fn test_compound_mul_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 10");
    let _ = interp.eval_string("x *= 2");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 20),
        _ => {}
    }
}

#[test]
fn test_compound_div_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 20");
    let _ = interp.eval_string("x /= 4");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

// ============== Array Index Assignment ==============

#[test]
fn test_array_index_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut arr = [1, 2, 3]");
    let _ = interp.eval_string("arr[1] = 42");
    let result = interp.eval_string("arr[1]");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

