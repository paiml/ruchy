#![allow(missing_docs)]
// RUNTIME-062: vec! Macro Implementation (GitHub Issue #62)
// Tests for vec! macro evaluation in interpreter
// Ticket: RUNTIME-062
// GitHub Issue: https://github.com/paiml/ruchy/issues/62

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper to parse and execute Ruchy code
fn run_ruchy(code: &str) -> Result<Value, String> {
    let ast = Parser::new(code)
        .parse()
        .map_err(|e| format!("Parse error: {e:?}"))?;
    let mut interpreter = Interpreter::new();
    interpreter
        .eval_expr(&ast)
        .map_err(|e| format!("Runtime error: {e}"))
}

#[test]
fn test_runtime_062_01_vec_macro_empty_vector() {
    // RED TEST: vec![] should create an empty vector
    let result = run_ruchy("vec![]");

    // Should succeed (not return "Expression type not yet implemented" error)
    assert!(
        result.is_ok(),
        "vec![] should be implemented, got error: {result:?}"
    );

    // Should return an empty array
    match result.unwrap() {
        Value::Array(arr) => assert_eq!(arr.len(), 0, "vec![] should create empty array"),
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_02_vec_macro_single_element() {
    // RED TEST: vec![42] should create a vector with one element
    let result = run_ruchy("vec![42]");

    assert!(
        result.is_ok(),
        "vec![42] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 1, "vec![42] should create array with 1 element");
            assert_eq!(arr[0], Value::Integer(42), "Element should be 42");
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_03_vec_macro_multiple_elements() {
    // RED TEST: vec![1, 2, 3] should create a vector with three elements
    let result = run_ruchy("vec![1, 2, 3]");

    assert!(
        result.is_ok(),
        "vec![1, 2, 3] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(arr) => {
            assert_eq!(
                arr.len(),
                3,
                "vec![1, 2, 3] should create array with 3 elements"
            );
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_04_vec_macro_string_elements() {
    // RED TEST: vec!["hello", "world"] should work with strings
    let result = run_ruchy(r#"vec!["hello", "world"]"#);

    assert!(
        result.is_ok(),
        "vec![strings] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::from_string("hello".to_string()));
            assert_eq!(arr[1], Value::from_string("world".to_string()));
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_05_vec_macro_mixed_types() {
    // RED TEST: vec![1, "hello", true] should work with mixed types
    let result = run_ruchy(r#"vec![1, "hello", true]"#);

    assert!(
        result.is_ok(),
        "vec![mixed types] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::from_string("hello".to_string()));
            assert_eq!(arr[2], Value::from_bool(true));
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_06_vec_macro_nested_vectors() {
    // RED TEST: vec![vec![1, 2], vec![3, 4]] should work with nested vectors
    let result = run_ruchy("vec![vec![1, 2], vec![3, 4]]");

    assert!(
        result.is_ok(),
        "vec![nested] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(outer) => {
            assert_eq!(outer.len(), 2, "Outer array should have 2 elements");

            // Check first nested vec
            match &outer[0] {
                Value::Array(inner1) => {
                    assert_eq!(inner1.len(), 2);
                    assert_eq!(inner1[0], Value::Integer(1));
                    assert_eq!(inner1[1], Value::Integer(2));
                }
                other => panic!("Expected nested Array, got: {other:?}"),
            }

            // Check second nested vec
            match &outer[1] {
                Value::Array(inner2) => {
                    assert_eq!(inner2.len(), 2);
                    assert_eq!(inner2[0], Value::Integer(3));
                    assert_eq!(inner2[1], Value::Integer(4));
                }
                other => panic!("Expected nested Array, got: {other:?}"),
            }
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_07_vec_macro_with_expressions() {
    // RED TEST: vec![1 + 1, 2 * 3, 10 - 5] should evaluate expressions
    let result = run_ruchy("vec![1 + 1, 2 * 3, 10 - 5]");

    assert!(
        result.is_ok(),
        "vec![expressions] should be implemented, got error: {result:?}"
    );

    match result.unwrap() {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2)); // 1 + 1
            assert_eq!(arr[1], Value::Integer(6)); // 2 * 3
            assert_eq!(arr[2], Value::Integer(5)); // 10 - 5
        }
        other => panic!("Expected Array, got: {other:?}"),
    }
}

#[test]
fn test_runtime_062_08_vec_macro_github_issue_reproduction() {
    // RED TEST: Exact reproduction from GitHub Issue #62
    // From: bootstrap/stage1/pratt_parser_full.ruchy
    // Simplified version without complex types
    let result = run_ruchy(r"vec![1, 2, 3]");

    assert!(
        result.is_ok(),
        "GitHub Issue #62 reproduction should work, got error: {result:?}"
    );

    // Verify it's an array (not "Expression type not yet implemented" error)
    match result.unwrap() {
        Value::Array(_) => {} // Success
        other => panic!("Expected Array for GitHub Issue #62 case, got: {other:?}"),
    }
}
