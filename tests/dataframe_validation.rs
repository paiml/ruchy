#![allow(missing_docs)]
//! `DataFrame` Access Validation Tests
//!
//! Comprehensive validation that DEFECT-DATAFRAME-001 and DEFECT-DATAFRAME-002 are fixed
//! Tests `DataFrame` indexing and field access with realistic scenarios

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

fn eval_code(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let mut interpreter = Interpreter::new();
    Ok(interpreter.eval_expr(&ast)?)
}

#[test]
fn test_dataframe_row_indexing_returns_object() {
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df[0]
    "#;

    let result = eval_code(code).expect("Should work");

    // Should return an Object with keys "id" and "name"
    match result {
        Value::Object(obj) => {
            assert!(obj.contains_key("id"), "Row should have 'id' key");
            assert!(obj.contains_key("name"), "Row should have 'name' key");

            // Verify values
            assert_eq!(obj.get("id"), Some(&Value::Integer(1)));
            assert_eq!(obj.get("name"), Some(&Value::from_string("Alice".to_string())));
        }
        _ => panic!("Expected Object, got {result:?}"),
    }
}

#[test]
fn test_dataframe_column_access_returns_array() {
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df.id
    "#;

    let result = eval_code(code).expect("Should work");

    // Should return an Array with [1, 2, 3]
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3, "Array should have 3 elements");
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        }
        _ => panic!("Expected Array, got {result:?}"),
    }
}

#[test]
fn test_dataframe_string_column_access() {
    let code = r#"
        let df = df![
            "employee_id" => [101, 102, 103],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df.name
    "#;

    let result = eval_code(code).expect("Should work");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::from_string("Alice".to_string()));
            assert_eq!(arr[1], Value::from_string("Bob".to_string()));
            assert_eq!(arr[2], Value::from_string("Charlie".to_string()));
        }
        _ => panic!("Expected Array of strings, got {result:?}"),
    }
}

#[test]
fn test_dataframe_column_via_bracket_indexing() {
    let code = r#"
        let df = df![
            "id" => [10, 20, 30],
            "value" => [100, 200, 300]
        ];
        df["value"]
    "#;

    let result = eval_code(code).expect("Should work");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(100));
            assert_eq!(arr[1], Value::Integer(200));
            assert_eq!(arr[2], Value::Integer(300));
        }
        _ => panic!("Expected Array, got {result:?}"),
    }
}

#[test]
fn test_dataframe_second_row_access() {
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df[1]
    "#;

    let result = eval_code(code).expect("Should work");

    match result {
        Value::Object(obj) => {
            assert_eq!(obj.get("id"), Some(&Value::Integer(2)));
            assert_eq!(obj.get("name"), Some(&Value::from_string("Bob".to_string())));
        }
        _ => panic!("Expected Object, got {result:?}"),
    }
}

#[test]
fn test_dataframe_realistic_scenario() {
    // Exact scenario from Gemini audit report
    let code = r#"
        let df = df![
            "employee_id" => [101, 102, 103, 104],
            "name" => ["Alice", "Bob", "Charlie", "Diana"],
            "salary" => [95000, 75000, 105000, 65000]
        ];

        // Access first employee
        let first = df[0];

        // Access all employee IDs
        let ids = df.employee_id;

        ids
    "#;

    let result = eval_code(code).expect("Should work");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], Value::Integer(101));
            assert_eq!(arr[1], Value::Integer(102));
            assert_eq!(arr[2], Value::Integer(103));
            assert_eq!(arr[3], Value::Integer(104));
        }
        _ => panic!("Expected Array, got {result:?}"),
    }
}
