#![cfg(test)]
//! `DataFrame` CSV/JSON import tests (DF-003)
//!
//! TDD tests for `DataFrame` data import functionality:
//! - `DataFrame::from_csv_string()` - Parse CSV with headers
//! - `DataFrame::from_json()` - Parse JSON array of objects
//! - Type inference for numeric/string columns

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Create interpreter for tests
fn new_interpreter() -> Interpreter {
    Interpreter::new()
}

/// Helper: Evaluate code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = new_interpreter();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

#[test]
fn test_from_csv_simple() {
    let code = r#"
        let csv_data = "name,age
Alice,25
Bob,30";
        DataFrame::from_csv_string(csv_data)
    "#;

    let result = eval(code).expect("CSV import should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have 2 columns");
            assert_eq!(columns[0].name, "name");
            assert_eq!(columns[1].name, "age");
            assert_eq!(columns[0].values.len(), 2, "Should have 2 rows");

            // Check first row
            assert_eq!(
                columns[0].values[0],
                Value::from_string("Alice".to_string())
            );
            assert_eq!(columns[1].values[0], Value::Integer(25));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_csv_with_strings_and_numbers() {
    let code = r#"
        let csv = "product,price,quantity
Apple,1.50,100
Orange,2.00,50
Banana,0.75,150";
        DataFrame::from_csv_string(csv)
    "#;

    let result = eval(code).expect("CSV with mixed types should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3);
            assert_eq!(columns[0].name, "product");
            assert_eq!(columns[1].name, "price");
            assert_eq!(columns[2].name, "quantity");

            // Verify 3 rows
            assert_eq!(columns[0].values.len(), 3);

            // Check types
            assert!(matches!(columns[0].values[0], Value::String(_)));
            assert!(matches!(columns[1].values[0], Value::Float(_)));
            assert!(matches!(columns[2].values[0], Value::Integer(_)));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_csv_empty() {
    let code = r#"
        DataFrame::from_csv_string("name,age")
    "#;

    let result = eval(code).expect("Empty CSV should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have 2 columns");
            assert_eq!(columns[0].values.len(), 0, "Should have 0 rows");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_csv_type_inference_integers() {
    let code = r#"
        let csv = "id,count
1,100
2,200
3,300";
        DataFrame::from_csv_string(csv)
    "#;

    let result = eval(code).expect("CSV with integers should work");

    match result {
        Value::DataFrame { columns } => {
            // Both columns should be integers
            assert_eq!(columns[0].values[0], Value::Integer(1));
            assert_eq!(columns[1].values[0], Value::Integer(100));
            assert_eq!(columns[0].values[2], Value::Integer(3));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_csv_type_inference_floats() {
    let code = r#"
        let csv = "price,discount
19.99,0.15
29.99,0.20
9.99,0.10";
        DataFrame::from_csv_string(csv)
    "#;

    let result = eval(code).expect("CSV with floats should work");

    match result {
        Value::DataFrame { columns } => {
            // Both columns should be floats
            assert!(matches!(columns[0].values[0], Value::Float(_)));
            assert!(matches!(columns[1].values[0], Value::Float(_)));

            if let Value::Float(f) = columns[0].values[0] {
                assert!((f - 19.99).abs() < 0.01);
            }
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_json_array_simple() {
    let code = r#"
        let json_data = "[
            {\"name\": \"Alice\", \"age\": 25},
            {\"name\": \"Bob\", \"age\": 30}
        ]";
        DataFrame::from_json(json_data)
    "#;

    let result = eval(code).expect("JSON import should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have 2 columns");

            // Column names could be in any order (JSON objects are unordered)
            let has_name = columns.iter().any(|c| c.name == "name");
            let has_age = columns.iter().any(|c| c.name == "age");
            assert!(has_name, "Should have 'name' column");
            assert!(has_age, "Should have 'age' column");

            // Each column should have 2 values
            assert_eq!(columns[0].values.len(), 2);
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_json_single_row() {
    let code = r#"
        let json = "[{\"x\": 42, \"y\": \"hello\"}]";
        DataFrame::from_json(json)
    "#;

    let result = eval(code).expect("Single row JSON should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].values.len(), 1);
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_json_empty_array() {
    let code = r#"
        DataFrame::from_json("[]")
    "#;

    let result = eval(code).expect("Empty JSON array should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 0, "Empty JSON should create empty DataFrame");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_from_json_type_preservation() {
    let code = r#"
        let json = "[
            {\"id\": 1, \"price\": 19.99, \"name\": \"Widget\"},
            {\"id\": 2, \"price\": 29.99, \"name\": \"Gadget\"}
        ]";
        DataFrame::from_json(json)
    "#;

    let result = eval(code).expect("JSON with mixed types should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3);

            // Find each column by name
            let id_col = columns.iter().find(|c| c.name == "id").unwrap();
            let price_col = columns.iter().find(|c| c.name == "price").unwrap();
            let name_col = columns.iter().find(|c| c.name == "name").unwrap();

            // Verify types
            assert!(matches!(id_col.values[0], Value::Integer(_)));
            assert!(matches!(price_col.values[0], Value::Float(_)));
            assert!(matches!(name_col.values[0], Value::String(_)));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_csv_accessor_integration() {
    let code = r#"
        let csv = "name,score
Alice,95
Bob,87
Charlie,92";
        let df = DataFrame::from_csv_string(csv);
        [df.rows(), df.columns()]
    "#;

    let result = eval(code).expect("CSV with accessors should work");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(3), "Should have 3 rows");
            assert_eq!(arr[1], Value::Integer(2), "Should have 2 columns");
        }
        other => panic!("Expected array, got: {other:?}"),
    }
}

#[test]
fn test_json_accessor_integration() {
    let code = r#"
        let json = "[{\"a\": 1}, {\"a\": 2}, {\"a\": 3}, {\"a\": 4}]";
        let df = DataFrame::from_json(json);
        df.rows()
    "#;

    let result = eval(code).expect("JSON with accessors should work");

    assert_eq!(result, Value::Integer(4), "Should have 4 rows");
}
