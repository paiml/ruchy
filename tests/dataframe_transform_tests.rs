#![cfg(test)]
//! `DataFrame` transformation operation tests (DF-004)
//!
//! TDD tests for `DataFrame` transformation methods:
//! - .`with_column(name`, closure) - Add new computed column
//! - .transform(name, closure) - Modify existing column
//! - .`sort_by(column)` - Sort by column values

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
fn test_with_column_simple() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .build();
        df.with_column("y", x => x * 2)
    "#;

    let result = eval(code).expect("with_column should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 2, "Should have 2 columns (x and y)");
            assert_eq!(columns[0].name, "x");
            assert_eq!(columns[1].name, "y");

            // Check y values are doubled
            assert_eq!(columns[1].values[0], Value::Integer(2));
            assert_eq!(columns[1].values[1], Value::Integer(4));
            assert_eq!(columns[1].values[2], Value::Integer(6));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_with_column_using_row_context() {
    let code = r#"
        let df = DataFrame::new()
            .column("price", [10, 20, 30])
            .column("quantity", [2, 3, 1])
            .build();
        df.with_column("total", row => row["price"] * row["quantity"])
    "#;

    let result = eval(code).expect("with_column with row context should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3, "Should have 3 columns");

            let total_col = columns.iter().find(|c| c.name == "total").unwrap();
            assert_eq!(total_col.values[0], Value::Integer(20)); // 10 * 2
            assert_eq!(total_col.values[1], Value::Integer(60)); // 20 * 3
            assert_eq!(total_col.values[2], Value::Integer(30)); // 30 * 1
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_with_column_chaining() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .build();
        df.with_column("y", x => x * 2)
          .with_column("z", y => y + 10)
    "#;

    let result = eval(code).expect("Chaining with_column should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 3, "Should have 3 columns");

            let z_col = columns.iter().find(|c| c.name == "z").unwrap();
            assert_eq!(z_col.values[0], Value::Integer(12)); // (1*2) + 10
            assert_eq!(z_col.values[2], Value::Integer(16)); // (3*2) + 10
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_transform_simple() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .build();
        df.transform("x", val => val * 10)
    "#;

    let result = eval(code).expect("transform should work");

    match result {
        Value::DataFrame { columns } => {
            assert_eq!(columns.len(), 1, "Should still have 1 column");
            assert_eq!(columns[0].name, "x");

            // Check values are transformed
            assert_eq!(columns[0].values[0], Value::Integer(10));
            assert_eq!(columns[0].values[1], Value::Integer(20));
            assert_eq!(columns[0].values[2], Value::Integer(30));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_transform_multiple_columns() {
    let code = r#"
        let df = DataFrame::new()
            .column("price", [10.0, 20.0, 30.0])
            .column("discount", [0.1, 0.2, 0.15])
            .build();
        df.transform("price", val => val * 1.08)
          .transform("discount", val => val * 100.0)
    "#;

    let result = eval(code).expect("Multiple transforms should work");

    match result {
        Value::DataFrame { columns } => {
            let price_col = columns.iter().find(|c| c.name == "price").unwrap();
            let discount_col = columns.iter().find(|c| c.name == "discount").unwrap();

            // Check price transformed
            if let Value::Float(f) = price_col.values[0] {
                assert!((f - 10.8).abs() < 0.01);
            } else {
                panic!("Expected float");
            }

            // Check discount transformed
            if let Value::Float(f) = discount_col.values[0] {
                assert!((f - 10.0).abs() < 0.01);
            } else {
                panic!("Expected float");
            }
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_transform_nonexistent_column_error() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .build();
        df.transform("y", val => val * 2)
    "#;

    let result = eval(code);
    assert!(result.is_err(), "Should fail for nonexistent column");
    assert!(
        result.unwrap_err().contains("not found"),
        "Error should mention column not found"
    );
}

#[test]
fn test_sort_by_ascending() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [3, 1, 2])
            .column("y", ["c", "a", "b"])
            .build();
        df.sort_by("x")
    "#;

    let result = eval(code).expect("sort_by should work");

    match result {
        Value::DataFrame { columns } => {
            let x_col = columns.iter().find(|c| c.name == "x").unwrap();
            let y_col = columns.iter().find(|c| c.name == "y").unwrap();

            // Check x is sorted
            assert_eq!(x_col.values[0], Value::Integer(1));
            assert_eq!(x_col.values[1], Value::Integer(2));
            assert_eq!(x_col.values[2], Value::Integer(3));

            // Check y follows same order
            if let Value::String(s) = &y_col.values[0] {
                assert_eq!(s.as_ref(), "a");
            }
            if let Value::String(s) = &y_col.values[2] {
                assert_eq!(s.as_ref(), "c");
            }
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_sort_by_descending() {
    let code = r#"
        let df = DataFrame::new()
            .column("score", [85, 92, 78])
            .column("name", ["Bob", "Alice", "Charlie"])
            .build();
        df.sort_by("score", true)
    "#;

    let result = eval(code).expect("sort_by descending should work");

    match result {
        Value::DataFrame { columns } => {
            let score_col = columns.iter().find(|c| c.name == "score").unwrap();

            // Check descending order
            assert_eq!(score_col.values[0], Value::Integer(92));
            assert_eq!(score_col.values[1], Value::Integer(85));
            assert_eq!(score_col.values[2], Value::Integer(78));
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_sort_by_strings() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Charlie", "Alice", "Bob"])
            .column("age", [30, 25, 28])
            .build();
        df.sort_by("name")
    "#;

    let result = eval(code).expect("Sorting by strings should work");

    match result {
        Value::DataFrame { columns } => {
            let name_col = columns.iter().find(|c| c.name == "name").unwrap();

            // Check alphabetical order
            if let Value::String(s) = &name_col.values[0] {
                assert_eq!(s.as_ref(), "Alice");
            }
            if let Value::String(s) = &name_col.values[1] {
                assert_eq!(s.as_ref(), "Bob");
            }
            if let Value::String(s) = &name_col.values[2] {
                assert_eq!(s.as_ref(), "Charlie");
            }
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}

#[test]
fn test_sort_by_nonexistent_column_error() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .build();
        df.sort_by("y")
    "#;

    let result = eval(code);
    assert!(result.is_err(), "Should fail for nonexistent column");
}

#[test]
fn test_combined_operations() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [3, 1, 2])
            .build();
        df.with_column("y", x => x * 10)
          .sort_by("x")
          .transform("y", val => val + 5)
    "#;

    let result = eval(code).expect("Combined operations should work");

    match result {
        Value::DataFrame { columns } => {
            let x_col = columns.iter().find(|c| c.name == "x").unwrap();
            let y_col = columns.iter().find(|c| c.name == "y").unwrap();

            // x should be sorted
            assert_eq!(x_col.values[0], Value::Integer(1));
            assert_eq!(x_col.values[2], Value::Integer(3));

            // y should be transformed after sorting
            assert_eq!(y_col.values[0], Value::Integer(15)); // (1*10) + 5
            assert_eq!(y_col.values[2], Value::Integer(35)); // (3*10) + 5
        }
        other => panic!("Expected DataFrame, got: {other:?}"),
    }
}
