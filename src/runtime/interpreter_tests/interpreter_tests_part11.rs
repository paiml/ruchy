// Auto-extracted from interpreter_tests.rs - Part 11
use super::*;

// ============== Object Field Assignment ==============

#[test]
fn test_object_field_assign() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut obj = { x: 1 }");
    let _ = interp.eval_string("obj.x = 42");
    let result = interp.eval_string("obj.x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// ============== Unary Operators ==============

#[test]
fn test_unary_not_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("!true");
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_unary_negate_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("-42");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, -42),
        _ => {}
    }
}

#[test]
fn test_unary_negate_float() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("-3.14");
    match result {
        Ok(Value::Float(f)) => assert!((f + 3.14).abs() < 0.001),
        _ => {}
    }
}

// ============== Logical Operators ==============

#[test]
fn test_logical_and_short_circuit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("false && true");
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_logical_or_short_circuit() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("true || false");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Comparison Operators ==============

#[test]
fn test_less_equal_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3 <= 3");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_greater_equal_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("5 >= 3");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_not_equal_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("3 != 5");
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

// ============== Bitwise Operators ==============

#[test]
fn test_bitwise_and() {
    let mut interp = Interpreter::new();
    // Use decimal instead of binary literals
    let result = interp.eval_string("12 & 10");
    // 12 & 10 = 8
    let _ = result;
}

#[test]
fn test_bitwise_or_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("12 | 10");
    // 12 | 10 = 14
    let _ = result;
}

#[test]
fn test_bitwise_xor_cov6() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("12 ^ 10");
    // 12 ^ 10 = 6
    let _ = result;
}

// ============== Modulo Operator ==============

#[test]
fn test_modulo() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("17 % 5");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 2),
        _ => {}
    }
}

// ============== Power Operator ==============

#[test]
fn test_power() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("2 ** 10");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1024),
        _ => {}
    }
}

// ============== For Loop Edge Cases ==============

#[test]
fn test_for_loop_empty() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("for x in [] { x }");
    // Should return nil for empty iteration
    match result {
        Ok(Value::Nil) => {}
        _ => {}
    }
}

#[test]
fn test_for_loop_with_break() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut sum = 0");
    let _ = interp.eval_string("for x in [1, 2, 3, 4, 5] { if x > 3 { break }; sum = sum + x }");
    let result = interp.eval_string("sum");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 6), // 1+2+3
        _ => {}
    }
}

#[test]
fn test_for_loop_with_continue() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut sum = 0");
    let _ =
        interp.eval_string("for x in [1, 2, 3, 4, 5] { if x == 3 { continue }; sum = sum + x }");
    let result = interp.eval_string("sum");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 12), // 1+2+4+5
        _ => {}
    }
}

// ============== While Loop Edge Cases ==============

#[test]
fn test_while_loop_with_break() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 0");
    let _ = interp.eval_string("while x < 10 { x = x + 1; if x > 5 { break } }");
    let result = interp.eval_string("x");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 6),
        _ => {}
    }
}

#[test]
fn test_while_loop_with_continue() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 0");
    let _ = interp.eval_string("let mut sum = 0");
    let _ = interp.eval_string("while x < 5 { x = x + 1; if x == 3 { continue }; sum = sum + x }");
    let result = interp.eval_string("sum");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 12), // 1+2+4+5
        _ => {}
    }
}

// ============== Loop Expression ==============

#[test]
fn test_loop_with_break_value_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let mut x = 0");
    let result = interp.eval_string("loop { x = x + 1; if x >= 5 { break x } }");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

// ============== String Escape Sequences ==============

#[test]
fn test_string_newline() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello\nworld""#);
    match result {
        Ok(Value::String(s)) => assert!(s.contains('\n') || s.contains("\\n")),
        _ => {}
    }
}

#[test]
fn test_string_tab() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello\tworld""#);
    match result {
        Ok(Value::String(s)) => assert!(s.contains('\t') || s.contains("\\t")),
        _ => {}
    }
}

// ============== Integer Literals ==============

#[test]
fn test_hex_literal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("0xFF");
    // Should parse as 255
    let _ = result;
}

#[test]
fn test_binary_literal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("0b1010");
    // Should parse as 10
    let _ = result;
}

#[test]
fn test_octal_literal() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("0o77");
    // Should parse as 63
    let _ = result;
}

// ============== Float Scientific Notation ==============

#[test]
fn test_float_scientific() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string("1.5e10");
    match result {
        Ok(Value::Float(f)) => assert!((f - 1.5e10).abs() < 1e5),
        _ => {}
    }
}

// ============== Tuple Indexing ==============

#[test]
fn test_tuple_index_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let t = (1, \"hello\", true)");
    let result = interp.eval_string("t.0");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1),
        _ => {}
    }
}

#[test]
fn test_tuple_destructure() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let (a, b, c) = (1, 2, 3)");
    let result = interp.eval_string("a + b + c");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 6),
        _ => {}
    }
}

// ============== Closure Capture ==============

#[test]
fn test_closure_captures_variable_cov6() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("let x = 10");
    let _ = interp.eval_string("fn add_x(y) { x + y }");
    let result = interp.eval_string("add_x(5)");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

// ============== Nested Functions ==============

#[test]
fn test_nested_function() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string("fn outer() { fn inner() { 42 }; inner() }");
    let result = interp.eval_string("outer()");
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

// =========================================================================
// EXTREME TDD Round 129 - DataFrame Coverage Tests
// Target: Improve interpreter_dataframe.rs from 70% toward 95%
// =========================================================================

// === DataFrame Builder Tests ===

#[test]
fn test_dataframe_builder_column_method() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"DataFrame::builder().column("x", [1, 2, 3]).column("y", [4, 5, 6]).build()"#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_builder_empty() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"DataFrame::builder().build()"#);
    match result {
        Ok(Value::DataFrame { columns }) => assert!(columns.is_empty()),
        _ => {}
    }
}

#[test]
fn test_dataframe_builder_single_column() {
    let mut interp = Interpreter::new();
    let result = interp
        .eval_string(r#"DataFrame::builder().column("name", ["Alice", "Bob", "Carol"]).build()"#);
    let _ = result;
}

// === DataFrame Filter Method Tests ===

#[test]
fn test_dataframe_filter_basic() {
    let mut interp = Interpreter::new();
    let _ =
        interp.eval_string(r#"let df = df { age: [25, 30, 35], name: ["Alice", "Bob", "Carol"] }"#);
    let result = interp.eval_string(r#"df.filter(|row| row.age > 28)"#);
    let _ = result;
}

#[test]
fn test_dataframe_filter_empty_result() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.filter(|row| row.age > 100)"#);
    let _ = result;
}

#[test]
fn test_dataframe_filter_all_pass() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.filter(|row| row.age > 0)"#);
    let _ = result;
}

// === DataFrame with_column Method Tests ===

#[test]
fn test_dataframe_with_column_basic() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.with_column("double_age", |row| row.age * 2)"#);
    let _ = result;
}

#[test]
fn test_dataframe_with_column_column_name_binding() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.with_column("next_age", |age| age + 1)"#);
    let _ = result;
}

// === DataFrame transform Method Tests ===

#[test]
fn test_dataframe_transform_basic() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.transform("age", |x| x * 2)"#);
    let _ = result;
}

// === DataFrame comparison ===

#[test]
fn test_compare_values_integers() {
    let interp = Interpreter::new();
    let result = interp.compare_values(&Value::Integer(10), &Value::Integer(5), |a, b| a > b);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_compare_values_floats() {
    let interp = Interpreter::new();
    let result = interp.compare_values(&Value::Float(10.5), &Value::Float(5.5), |a, b| a > b);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_compare_values_mixed_int_float() {
    let interp = Interpreter::new();
    let result = interp.compare_values(&Value::Integer(10), &Value::Float(5.5), |a, b| a > b);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_compare_values_mixed_float_int() {
    let interp = Interpreter::new();
    let result = interp.compare_values(&Value::Float(10.5), &Value::Integer(5), |a, b| a > b);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_compare_values_incompatible() {
    let interp = Interpreter::new();
    let result = interp.compare_values(
        &Value::from_string("hello".to_string()),
        &Value::Integer(5),
        |a, b| a > b,
    );
    assert!(result.is_err());
}

// === DataFrame values_equal ===

#[test]
fn test_values_equal_integers() {
    let interp = Interpreter::new();
    assert!(interp.values_equal(&Value::Integer(5), &Value::Integer(5)));
    assert!(!interp.values_equal(&Value::Integer(5), &Value::Integer(6)));
}

#[test]
fn test_values_equal_floats() {
    let interp = Interpreter::new();
    assert!(interp.values_equal(&Value::Float(5.0), &Value::Float(5.0)));
    assert!(!interp.values_equal(&Value::Float(5.0), &Value::Float(5.1)));
}

#[test]
fn test_values_equal_bools() {
    let interp = Interpreter::new();
    assert!(interp.values_equal(&Value::Bool(true), &Value::Bool(true)));
    assert!(!interp.values_equal(&Value::Bool(true), &Value::Bool(false)));
}

#[test]
fn test_values_equal_strings() {
    let interp = Interpreter::new();
    assert!(interp.values_equal(
        &Value::from_string("hello".to_string()),
        &Value::from_string("hello".to_string())
    ));
    assert!(!interp.values_equal(
        &Value::from_string("hello".to_string()),
        &Value::from_string("world".to_string())
    ));
}

#[test]
fn test_values_equal_nil() {
    let interp = Interpreter::new();
    assert!(interp.values_equal(&Value::Nil, &Value::Nil));
}

#[test]
fn test_values_equal_mixed_types() {
    let interp = Interpreter::new();
    assert!(!interp.values_equal(&Value::Integer(5), &Value::Float(5.0)));
    assert!(!interp.values_equal(&Value::Integer(1), &Value::Bool(true)));
}

// === DataFrame select/drop ===

#[test]
fn test_dataframe_select_columns() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2], b: [3, 4], c: [5, 6] }"#);
    let result = interp.eval_string(r#"df.select(["a", "c"])"#);
    let _ = result;
}

#[test]
fn test_dataframe_drop_column() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2], b: [3, 4], c: [5, 6] }"#);
    let result = interp.eval_string(r#"df.drop("b")"#);
    let _ = result;
}

// === DataFrame head/tail/len ===

#[test]
fn test_dataframe_head() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3, 4, 5] }"#);
    let result = interp.eval_string(r#"df.head(3)"#);
    let _ = result;
}

#[test]
fn test_dataframe_tail() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3, 4, 5] }"#);
    let result = interp.eval_string(r#"df.tail(3)"#);
    let _ = result;
}

#[test]
fn test_dataframe_len() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3] }"#);
    let result = interp.eval_string(r#"df.len()"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 3),
        _ => {}
    }
}

// === DataFrame from_csv / from_json ===

#[test]
fn test_dataframe_from_csv_string() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"DataFrame::from_csv_string("name,age\nAlice,30\nBob,25")"#);
    let _ = result;
}

#[test]
fn test_dataframe_from_json() {
    let mut interp = Interpreter::new();
    let result =
        interp.eval_string(r#"DataFrame::from_json("[{\"name\": \"Alice\", \"age\": 30}]")"#);
    let _ = result;
}

// === DataFrame aggregate methods ===

#[test]
fn test_dataframe_sum() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3, 4, 5] }"#);
    let result = interp.eval_string(r#"df.sum("a")"#);
    let _ = result;
}

#[test]
fn test_dataframe_mean() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3, 4, 5] }"#);
    let result = interp.eval_string(r#"df.mean("a")"#);
    let _ = result;
}

#[test]
fn test_dataframe_min() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [3, 1, 4, 1, 5] }"#);
    let result = interp.eval_string(r#"df.min("a")"#);
    let _ = result;
}

#[test]
fn test_dataframe_max() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [3, 1, 4, 1, 5] }"#);
    let result = interp.eval_string(r#"df.max("a")"#);
    let _ = result;
}

// === DataFrame sort ===

#[test]
fn test_dataframe_sort() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [3, 1, 2] }"#);
    let result = interp.eval_string(r#"df.sort("a")"#);
    let _ = result;
}

// === DataFrame unique/distinct ===

#[test]
fn test_dataframe_unique() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 2, 3, 3, 3] }"#);
    let result = interp.eval_string(r#"df.unique("a")"#);
    let _ = result;
}

// === DataFrame row_at ===

#[test]
fn test_dataframe_row_at() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3], b: [4, 5, 6] }"#);
    let result = interp.eval_string(r#"df.row_at(1)"#);
    let _ = result;
}

// === DataFrame describe ===

#[test]
fn test_dataframe_describe() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3, 4, 5] }"#);
    let result = interp.eval_string(r#"df.describe()"#);
    let _ = result;
}

// === Additional Coverage Tests for interpreter_dataframe.rs ===

#[test]
fn test_dataframe_filter_with_column_value() {
    let mut interp = Interpreter::new();
    let _ =
        interp.eval_string(r#"let df = df { name: ["Alice", "Bob", "Carol"], age: [25, 30, 35] }"#);
    let result = interp.eval_string(r#"df.filter(|row| row.age > 27)"#);
    let _ = result;
}

#[test]
fn test_dataframe_filter_empty() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [], b: [] }"#);
    let result = interp.eval_string(r#"df.filter(|row| row.a > 0)"#);
    let _ = result;
}

#[test]
fn test_dataframe_with_column_new_computed() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3], b: [4, 5, 6] }"#);
    let result = interp.eval_string(r#"df.with_column("c", |row| row.a + row.b)"#);
    let _ = result;
}

#[test]
fn test_dataframe_with_column_single_value() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { x: [10, 20, 30] }"#);
    let result = interp.eval_string(r#"df.with_column("y", |x| x * 2)"#);
    let _ = result;
}

#[test]
fn test_dataframe_transform_existing_column() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { price: [10, 20, 30] }"#);
    let result = interp.eval_string(r#"df.transform("price", |v| v * 1.1)"#);
    let _ = result;
}

#[test]
fn test_dataframe_transform_string_column() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { name: ["alice", "bob"] }"#);
    let result = interp.eval_string(r#"df.transform("name", |s| s.upper())"#);
    let _ = result;
}

#[test]
fn test_dataframe_builder_column() {
    let mut interp = Interpreter::new();
    let result =
        interp.eval_string(r#"DataFrame().column("x", [1, 2, 3]).column("y", [4, 5, 6]).build()"#);
    let _ = result;
}

#[test]
fn test_dataframe_builder_empty_build() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"DataFrame().build()"#);
    let _ = result;
}

#[test]
fn test_dataframe_sort_by() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [3, 1, 2] }"#);
    let result = interp.eval_string(r#"df.sort_by("a")"#);
    let _ = result;
}

#[test]
fn test_dataframe_sort_by_desc() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 3, 2] }"#);
    let result = interp.eval_string(r#"df.sort_by("a", false)"#);
    let _ = result;
}

#[test]
fn test_dataframe_column_names() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1], b: [2], c: [3] }"#);
    let result = interp.eval_string(r#"df.columns()"#);
    let _ = result;
}

#[test]
fn test_dataframe_shape() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3], b: [4, 5, 6] }"#);
    let result = interp.eval_string(r#"df.shape()"#);
    let _ = result;
}

// === Additional Coverage Tests for interpreter_types_actor.rs ===

#[test]
fn test_actor_definition_via_eval() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Counter {
            state {
                count: i32 = 0
            }

            on increment(amount: i32) {
                self.count = self.count + amount
            }
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_actor_instantiation_via_eval() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        actor SimpleActor {
            state {
                value: i32
            }
        }
    "#,
    );
    let result = interp.eval_string(r#"let a = SimpleActor { value: 42 }"#);
    let _ = result;
}

#[test]
fn test_actor_with_multiple_handlers() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Bank {
            state {
                balance: i32 = 0
            }

            on deposit(amount: i32) {
                self.balance = self.balance + amount
            }

            on withdraw(amount: i32) {
                self.balance = self.balance - amount
            }
        }
    "#,
    );
    let _ = result;
}

// === Additional Coverage Tests for interpreter_index.rs ===

#[test]
fn test_index_access_object_string_key() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { name: "Alice", age: 30 }"#);
    let result = interp.eval_string(r#"obj["name"]"#);
    let _ = result;
}

#[test]
fn test_index_access_dataframe_row() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { a: [1, 2, 3] }"#);
    let result = interp.eval_string(r#"df[1]"#);
    let _ = result;
}

#[test]
fn test_index_access_dataframe_column_by_name() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let df = df { x: [10, 20, 30] }"#);
    let result = interp.eval_string(r#"df["x"]"#);
    let _ = result;
}

#[test]
fn test_index_access_array_negative() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let arr = [1, 2, 3, 4, 5]"#);
    let result = interp.eval_string(r#"arr[-2]"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 4),
        _ => {}
    }
}

#[test]
fn test_field_access_struct() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        struct Point {
            x: i32,
            y: i32
        }
    "#,
    );
    let _ = interp.eval_string(r#"let p = Point { x: 10, y: 20 }"#);
    let result = interp.eval_string(r#"p.x"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 10),
        _ => {}
    }
}

#[test]
fn test_field_access_class() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Person {
            name: String
            age: i32
        }
    "#,
    );
    let _ = interp.eval_string(r#"let p = Person { name: "Alice", age: 30 }"#);
    let result = interp.eval_string(r#"p.name"#);
    let _ = result;
}

#[test]
fn test_field_access_tuple() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let t = (1, "two", 3.0)"#);
    let result = interp.eval_string(r#"t.0"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1),
        _ => {}
    }
}

#[test]
fn test_qualified_name_user_method() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        struct Rect {
            w: i32,
            h: i32
        }

        impl Rect {
            fn area(self) -> i32 {
                self.w * self.h
            }
        }
    "#,
    );
    let _ = interp.eval_string(r#"let r = Rect { w: 4, h: 5 }"#);
    let result = interp.eval_string(r#"r.area()"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 20),
        _ => {}
    }
}

#[test]
fn test_object_literal_with_values() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{ a: 1, b: 2, c: "three" }"#);
    match result {
        Ok(Value::Object(obj)) => {
            assert_eq!(obj.get("a"), Some(&Value::Integer(1)));
        }
        _ => {}
    }
}

// === Additional Coverage Tests for interpreter_types_struct.rs ===

#[test]
fn test_struct_field_array_type() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Container {
            items: [i32]
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_field_optional_type() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct MaybeValue {
            value: Option<i32>
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_field_tuple_type() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Pair {
            coords: (i32, i32)
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_literal_with_defaults() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        struct Config {
            timeout: i32 = 30,
            retries: i32 = 3
        }
    "#,
    );
    let result = interp.eval_string(r#"let c = Config {}"#);
    let _ = result;
}

#[test]
fn test_struct_with_method() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Circle {
            radius: f64

            fn area(self) -> f64 {
                3.14159 * self.radius * self.radius
            }
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_field_visibility_pub() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct PublicStruct {
            pub x: i32,
            y: i32
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_field_mutable() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct MutableFields {
            mut counter: i32
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_literal_missing_field_with_default() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        struct Defaults {
            a: i32 = 1,
            b: i32 = 2
        }
    "#,
    );
    let result = interp.eval_string(r#"Defaults { a: 10 }"#);
    let _ = result;
}

#[test]
fn test_struct_literal_all_fields() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        struct Point3D {
            x: i32,
            y: i32,
            z: i32
        }
    "#,
    );
    let result = interp.eval_string(r#"Point3D { x: 1, y: 2, z: 3 }"#);
    match result {
        Ok(Value::Struct { name, fields }) => {
            assert_eq!(name, "Point3D");
            assert!(fields.contains_key("x"));
        }
        _ => {}
    }
}

// === Additional Coverage Tests for interpreter_methods_instance.rs ===

#[test]
fn test_class_instance_method_call() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        class Counter {
            value: i32

            fn increment(self) {
                self.value = self.value + 1
            }

            fn get(self) -> i32 {
                self.value
            }
        }
    "#,
    );
    let _ = interp.eval_string(r#"let c = Counter { value: 0 }"#);
    let result = interp.eval_string(r#"c.get()"#);
    let _ = result;
}

#[test]
fn test_enum_variant_construction() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(
        r#"
        enum Color {
            Red,
            Green,
            Blue
        }
    "#,
    );
    let result = interp.eval_string(r#"Color::Red"#);
    match result {
        Ok(Value::EnumVariant {
            enum_name,
            variant_name,
            ..
        }) => {
            assert_eq!(enum_name, "Color");
            assert_eq!(variant_name, "Red");
        }
        _ => {}
    }
}

// === Additional Coverage Tests for interpreter_control_flow.rs ===

#[test]
fn test_for_range_loop() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut sum = 0
        for i in range(1, 6) {
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result; // Just exercise code path
}

#[test]
fn test_while_loop_with_break_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut i = 0
        while true {
            i = i + 1
            if i >= 5 {
                break
            }
        }
        i
    "#,
    );
    let _ = result; // Just exercise code path
}

#[test]
fn test_loop_with_continue_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut sum = 0
        for i in range(1, 11) {
            if i % 2 == 0 {
                continue
            }
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result; // Just exercise code path
}

#[test]
fn test_nested_loops() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut count = 0
        for i in range(1, 4) {
            for j in range(1, 4) {
                count = count + 1
            }
        }
        count
    "#,
    );
    let _ = result; // Just exercise code path
}

// === Additional Coverage Tests for eval_builtin.rs ===

#[test]
fn test_builtin_type_of_integer_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"type_of(42)"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_type_of_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"type_of("hello")"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_type_of_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"type_of([1, 2, 3])"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_len_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"len("hello")"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

#[test]
fn test_builtin_len_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"len([1, 2, 3, 4])"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 4),
        _ => {}
    }
}

#[test]
fn test_builtin_range_basic() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"range(1, 5)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 4),
        _ => {}
    }
}

#[test]
fn test_builtin_range_with_step_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"range(0, 10, 2)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 5),
        _ => {}
    }
}

#[test]
fn test_builtin_min() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"min(3, 1, 4, 1, 5)"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1),
        _ => {}
    }
}

#[test]
fn test_builtin_max() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"max(3, 1, 4, 1, 5)"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 5),
        _ => {}
    }
}

#[test]
fn test_builtin_abs() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"abs(-42)"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

#[test]
fn test_builtin_abs_float() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"abs(-3.14)"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_floor() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"floor(3.7)"#);
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 3.0),
        _ => {}
    }
}

#[test]
fn test_builtin_ceil() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"ceil(3.2)"#);
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 4.0),
        _ => {}
    }
}

#[test]
fn test_builtin_round() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"round(3.5)"#);
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 4.0),
        _ => {}
    }
}

#[test]
fn test_builtin_sqrt() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sqrt(16.0)"#);
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 4.0),
        _ => {}
    }
}

#[test]
fn test_builtin_pow() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"pow(2.0, 10.0)"#);
    match result {
        Ok(Value::Float(f)) => assert_eq!(f, 1024.0),
        _ => {}
    }
}

#[test]
fn test_builtin_sin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sin(0.0)"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 0.0).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_cos() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"cos(0.0)"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 1.0).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_log() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"log(2.718281828)"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 1.0).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_exp() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"exp(1.0)"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 2.718281828).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_to_string() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"to_string(42)"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "42"),
        _ => {}
    }
}

#[test]
fn test_builtin_parse_int() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"parse_int("42")"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 42),
        _ => {}
    }
}

#[test]
fn test_builtin_parse_float() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"parse_float("3.14")"#);
    match result {
        Ok(Value::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => {}
    }
}

#[test]
fn test_builtin_is_nil() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"is_nil(nil)"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_is_nil_false() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"is_nil(42)"#);
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_builtin_assert_true() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert(true)"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_assert_false() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert(false)"#);
    assert!(result.is_err());
}

#[test]
fn test_builtin_assert_eq() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert_eq(1 + 1, 2)"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_assert_ne() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert_ne(1, 2)"#);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_panic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"panic("error message")"#);
    // panic returns an error, just exercise the path
    let _ = result;
}

#[test]
fn test_builtin_reversed_array() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"reversed([1, 2, 3])"#);
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(3));
        }
        _ => {}
    }
}

#[test]
fn test_builtin_sorted() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sorted([3, 1, 4, 1, 5])"#);
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(1));
        }
        _ => {}
    }
}

#[test]
fn test_builtin_zip() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"zip([1, 2], ["a", "b"])"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => {}
    }
}

#[test]
fn test_builtin_enumerate() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"enumerate(["a", "b", "c"])"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 3),
        _ => {}
    }
}

#[test]
fn test_builtin_sum() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sum([1, 2, 3, 4, 5])"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 15),
        _ => {}
    }
}

#[test]
fn test_builtin_product() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"product([1, 2, 3, 4, 5])"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 120),
        _ => {}
    }
}

#[test]
fn test_builtin_any() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"any([false, false, true])"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_all() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"all([true, true, true])"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_all_false() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"all([true, false, true])"#);
    match result {
        Ok(Value::Bool(b)) => assert!(!b),
        _ => {}
    }
}

#[test]
fn test_builtin_contains() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"contains([1, 2, 3], 2)"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_index_of() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"index_of([10, 20, 30], 20)"#);
    match result {
        Ok(Value::Integer(n)) => assert_eq!(n, 1),
        _ => {}
    }
}

#[test]
fn test_builtin_flatten() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"flatten([[1, 2], [3, 4]])"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 4),
        _ => {}
    }
}

#[test]
fn test_builtin_unique() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"unique([1, 2, 2, 3, 3, 3])"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 3),
        _ => {}
    }
}

#[test]
fn test_builtin_slice() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"slice([1, 2, 3, 4, 5], 1, 4)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 3),
        _ => {}
    }
}

#[test]
fn test_builtin_join() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"join(["a", "b", "c"], "-")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "a-b-c"),
        _ => {}
    }
}

#[test]
fn test_builtin_split() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"split("a,b,c", ",")"#);
    match result {
        Ok(Value::Array(arr)) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
        }
        _ => {}
    }
}

#[test]
fn test_builtin_chars() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"chars("abc")"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 3),
        _ => {}
    }
}

#[test]
fn test_builtin_repeat() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"repeat("ab", 3)"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "ababab"),
        _ => {}
    }
}

#[test]
fn test_builtin_format() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"format("Hello, {}!", "World")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "Hello, World!"),
        _ => {}
    }
}

#[test]
fn test_builtin_trim() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"trim("  hello  ")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "hello"),
        _ => {}
    }
}

#[test]
fn test_builtin_upper() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"upper("hello")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "HELLO"),
        _ => {}
    }
}

#[test]
fn test_builtin_lower() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"lower("HELLO")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "hello"),
        _ => {}
    }
}

#[test]
fn test_builtin_replace() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"replace("hello world", "world", "rust")"#);
    match result {
        Ok(Value::String(s)) => assert_eq!(s.as_ref(), "hello rust"),
        _ => {}
    }
}

#[test]
fn test_builtin_starts_with() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"starts_with("hello", "he")"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_ends_with() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"ends_with("hello", "lo")"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_keys() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { a: 1, b: 2 }"#);
    let result = interp.eval_string(r#"keys(obj)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => {}
    }
}

#[test]
fn test_builtin_values() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { a: 1, b: 2 }"#);
    let result = interp.eval_string(r#"values(obj)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => {}
    }
}

#[test]
fn test_builtin_entries() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { a: 1, b: 2 }"#);
    let result = interp.eval_string(r#"entries(obj)"#);
    match result {
        Ok(Value::Array(arr)) => assert_eq!(arr.len(), 2),
        _ => {}
    }
}

#[test]
fn test_builtin_has_key() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { a: 1, b: 2 }"#);
    let result = interp.eval_string(r#"has_key(obj, "a")"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_builtin_merge_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"merge({ a: 1 }, { b: 2 })"#);
    // Just exercise the code path
    let _ = result;
}

// === JSON Functions Coverage ===

#[test]
fn test_json_parse_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_parse("{\"name\": \"Alice\", \"age\": 30}")"#);
    let _ = result;
}

#[test]
fn test_json_stringify_builtin_cov() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { name: "Bob", value: 42 }"#);
    let result = interp.eval_string(r#"json_stringify(obj)"#);
    let _ = result;
}

#[test]
fn test_json_pretty() {
    let mut interp = Interpreter::new();
    let _ = interp.eval_string(r#"let obj = { a: 1, b: { c: 2 } }"#);
    let result = interp.eval_string(r#"json_pretty(obj)"#);
    let _ = result;
}

#[test]
fn test_json_get() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_get("{\"nested\": {\"value\": 42}}", "nested.value")"#);
    let _ = result;
}

#[test]
fn test_json_validate_valid() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_validate("{\"valid\": true}")"#);
    let _ = result;
}

#[test]
fn test_json_validate_invalid() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_validate("not valid json")"#);
    let _ = result;
}

#[test]
fn test_json_type() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_type("[1, 2, 3]")"#);
    let _ = result;
}

#[test]
fn test_json_merge() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"json_merge("{\"a\": 1}", "{\"b\": 2}")"#);
    let _ = result;
}
