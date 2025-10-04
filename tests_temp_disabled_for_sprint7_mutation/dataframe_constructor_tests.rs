#![cfg(test)]
//! `DataFrame` constructor API tests (DF-002)
//!
//! TDD tests for `DataFrame` builder pattern:
//! `DataFrame::new().column(...).build()`

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
fn test_dataframe_new_empty() {
    let code = r"
        DataFrame::new().build()
    ";

    let result = eval(code).expect("DataFrame::new().build() should work");

    // Should return a DataFrame with 0 columns
    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 0, "Empty DataFrame should have 0 columns");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_single_column_builder() {
    let code = r#"
        DataFrame::new()
            .column("x", [1, 2, 3])
            .build()
    "#;

    let result = eval(code).expect("Single column builder should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1, "Should have 1 column");
            assert_eq!(columns[0].name, "x");
            assert_eq!(columns[0].values.len(), 3);
            assert_eq!(columns[0].values[0], Value::Integer(1));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_multi_column_builder() {
    let code = r#"
        DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .column("age", [25, 30])
            .build()
    "#;

    let result = eval(code).expect("Multi-column builder should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have 2 columns");

            // First column
            assert_eq!(columns[0].name, "name");
            assert_eq!(columns[0].values.len(), 2);

            // Second column
            assert_eq!(columns[1].name, "age");
            assert_eq!(columns[1].values.len(), 2);
            assert_eq!(columns[1].values[0], Value::Integer(25));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_rows_accessor() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3, 4])
            .build();
        df.rows()
    "#;

    let result = eval(code).expect("df.rows() should work");

    assert_eq!(result, Value::Integer(4), "Should return 4 rows");
}

#[test]
fn test_dataframe_columns_accessor() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2])
            .column("y", [3, 4])
            .column("z", [5, 6])
            .build();
        df.columns()
    "#;

    let result = eval(code).expect("df.columns() should work");

    assert_eq!(result, Value::Integer(3), "Should return 3 columns");
}

#[test]
fn test_dataframe_column_names_accessor() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice"])
            .column("age", [25])
            .column("city", ["NYC"])
            .build();
        df.column_names()
    "#;

    let result = eval(code).expect("df.column_names() should work");

    // Should return array of column names
    match result {
        Value::Array(names) => {
            assert_eq!(names.len(), 3);
            // Check each name (they should be strings)
            if let Value::String(ref s) = names[0] {
                assert_eq!(s.as_ref(), "name");
            } else {
                panic!("Expected string column name");
            }
        }
        other => panic!("Expected array of names, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_empty_rows_columns() {
    let code = r"
        let df = DataFrame::new().build();
        [df.rows(), df.columns()]
    ";

    let result = eval(code).expect("Empty DataFrame accessors should work");

    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Integer(0), "Empty DataFrame has 0 rows");
            assert_eq!(arr[1], Value::Integer(0), "Empty DataFrame has 0 columns");
        }
        other => panic!("Expected array, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_builder_chaining() {
    let code = r#"
        DataFrame::new()
            .column("a", [1])
            .column("b", [2])
            .column("c", [3])
            .column("d", [4])
            .build()
    "#;

    let result = eval(code).expect("Builder chaining should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 4, "Should have 4 columns");
            assert_eq!(columns[0].name, "a");
            assert_eq!(columns[3].name, "d");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_mixed_types_builder() {
    let code = r#"
        DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .column("age", [25, 30])
            .column("score", [95.5, 87.3])
            .build()
    "#;

    let result = eval(code).expect("Mixed type builder should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3);

            // Verify types are preserved
            assert!(matches!(columns[0].values[0], Value::String(_)));
            assert!(matches!(columns[1].values[0], Value::Integer(_)));
            assert!(matches!(columns[2].values[0], Value::Float(_)));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}
