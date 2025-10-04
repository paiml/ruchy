#![cfg(test)]
//! `DataFrame` literal evaluation tests (DF-001)
//!
//! TDD tests for `DataFrame` literal syntax: df![...]
//! These tests must pass before implementation is considered complete.

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
fn test_empty_dataframe_literal() {
    let result = eval("df![]").expect("df![] should evaluate successfully");

    // Should return a DataFrame with 0 columns
    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 0, "Empty DataFrame should have 0 columns");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_single_column_dataframe_integers() {
    let result = eval("df![age => [25, 30, 35]]").expect("Single column DataFrame should evaluate");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1, "Should have exactly 1 column");

            let col = &columns[0];
            assert_eq!(col.name, "age", "Column name should be 'age'");
            assert_eq!(col.values.len(), 3, "Should have 3 values");

            // Verify values
            assert_eq!(col.values[0], Value::Integer(25));
            assert_eq!(col.values[1], Value::Integer(30));
            assert_eq!(col.values[2], Value::Integer(35));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_single_column_dataframe_strings() {
    let result = eval(r#"df![name => ["Alice", "Bob", "Charlie"]]"#)
        .expect("String column DataFrame should evaluate");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1, "Should have exactly 1 column");

            let col = &columns[0];
            assert_eq!(col.name, "name", "Column name should be 'name'");
            assert_eq!(col.values.len(), 3, "Should have 3 values");

            // Verify string values
            if let Value::String(s) = &col.values[0] {
                assert_eq!(s.as_ref(), "Alice");
            } else {
                panic!("Expected String value");
            }
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_multi_column_dataframe() {
    let code = r#"df![
        name => ["Alice", "Bob"],
        age => [25, 30]
    ]"#;

    let result = eval(code).expect("Multi-column DataFrame should evaluate");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have exactly 2 columns");

            // Check first column (name)
            let name_col = &columns[0];
            assert_eq!(name_col.name, "name");
            assert_eq!(name_col.values.len(), 2);

            // Check second column (age)
            let age_col = &columns[1];
            assert_eq!(age_col.name, "age");
            assert_eq!(age_col.values.len(), 2);
            assert_eq!(age_col.values[0], Value::Integer(25));
            assert_eq!(age_col.values[1], Value::Integer(30));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_mixed_types() {
    let code = r#"df![
        name => ["Alice", "Bob"],
        age => [25, 30],
        score => [95.5, 87.3]
    ]"#;

    let result = eval(code).expect("Mixed-type DataFrame should evaluate");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3, "Should have 3 columns");

            // Verify types
            let name_col = &columns[0];
            assert!(matches!(name_col.values[0], Value::String(_)));

            let age_col = &columns[1];
            assert!(matches!(age_col.values[0], Value::Integer(_)));

            let score_col = &columns[2];
            assert!(matches!(score_col.values[0], Value::Float(_)));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_with_variables() {
    let code = r"
        let ages = [20, 25, 30];
        df![age => ages]
    ";

    let result = eval(code).expect("DataFrame with variable should evaluate");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1);
            // ages is treated as a single array value, not spread
            assert_eq!(columns[0].values.len(), 1);
            // The single value should be an array
            assert!(matches!(columns[0].values[0], Value::Array(_)));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_dataframe_display() {
    let result = eval("df![x => [1, 2, 3]]").expect("DataFrame should evaluate");

    let display = result.to_string();

    // Should have some reasonable string representation
    assert!(
        display.contains("DataFrame") || display.contains('x'),
        "Display should mention DataFrame or column name, got: {display}"
    );
}

#[test]
fn test_dataframe_assignment() {
    let code = r"
        let data = df![x => [1, 2, 3]];
        data
    ";

    let result = eval(code).expect("DataFrame assignment should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].name, "x");
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}
