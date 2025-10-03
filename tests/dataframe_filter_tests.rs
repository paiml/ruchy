//! `DataFrame` .`filter()` method tests (DF-005)
//!
//! Tests for the DataFrame.filter(closure) method.
//! Follows EXTREME TDD methodology with <10 cyclomatic complexity.
//!
//! Expected behavior:
//! - Accepts a closure that receives a row object
//! - Row object supports dictionary-style access: row["column"]
//! - Returns new `DataFrame` with only rows matching predicate
//! - Preserves column order and structure

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
fn test_filter_basic_numeric() {
    let code = r#"
        let df = DataFrame::new()
            .column("age", [25, 30, 35, 40])
            .column("name", ["Alice", "Bob", "Charlie", "Dave"])
            .build();

        let filtered = df.filter(|row| row["age"] > 30);
        filtered.rows()
    "#;

    let result = eval(code).expect("Should filter DataFrame");
    assert_eq!(result.to_string(), "2"); // Charlie and Dave
}

#[test]
fn test_filter_string_comparison() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice", "Bob", "Charlie"])
            .column("city", ["NYC", "SF", "NYC"])
            .build();

        let nyc_only = df.filter(|row| row["city"] == "NYC");
        nyc_only.rows()
    "#;

    let result = eval(code).expect("Should filter by string");
    assert_eq!(result.to_string(), "2"); // Alice and Charlie
}

#[test]
fn test_filter_multiple_conditions() {
    let code = r#"
        let df = DataFrame::new()
            .column("price", [10, 50, 100, 200])
            .column("quantity", [5, 10, 15, 20])
            .build();

        let filtered = df.filter(|row| row["price"] > 25 && row["quantity"] < 20);
        filtered.rows()
    "#;

    let result = eval(code).expect("Should filter with multiple conditions");
    assert_eq!(result.to_string(), "2"); // 50 and 100
}

#[test]
fn test_filter_preserves_columns() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 2, 3])
            .column("b", [4, 5, 6])
            .column("c", [7, 8, 9])
            .build();

        let filtered = df.filter(|row| row["a"] > 1);
        filtered.columns()
    "#;

    let result = eval(code).expect("Should preserve columns");
    assert_eq!(result.to_string(), "3"); // Still has 3 columns
}

#[test]
fn test_filter_no_matches() {
    let code = r#"
        let df = DataFrame::new()
            .column("value", [1, 2, 3])
            .build();

        let filtered = df.filter(|row| row["value"] > 10);
        filtered.rows()
    "#;

    let result = eval(code).expect("Should return empty DataFrame");
    assert_eq!(result.to_string(), "0");
}

#[test]
fn test_filter_all_match() {
    let code = r#"
        let df = DataFrame::new()
            .column("value", [5, 10, 15])
            .build();

        let filtered = df.filter(|row| row["value"] > 0);
        filtered.rows()
    "#;

    let result = eval(code).expect("Should return all rows");
    assert_eq!(result.to_string(), "3");
}

#[test]
fn test_filter_chaining() {
    let code = r#"
        let df = DataFrame::new()
            .column("age", [20, 30, 40, 50])
            .column("score", [60, 70, 80, 90])
            .build();

        let filtered = df
            .filter(|row| row["age"] > 25)
            .filter(|row| row["score"] < 85);

        filtered.rows()
    "#;

    let result = eval(code).expect("Should chain filters");
    assert_eq!(result.to_string(), "2"); // 30/70 and 40/80
}

#[test]
fn test_filter_with_get() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice", "Bob", "Charlie"])
            .column("age", [25, 35, 45])
            .build();

        let filtered = df.filter(|row| row["age"] >= 35);
        filtered.get("name", 0)
    "#;

    let result = eval(code).expect("Should work with .get()");
    assert_eq!(result.to_string(), "\"Bob\"");
}

#[test]
fn test_filter_from_csv() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,price\nApple,1.5\nBanana,0.5\nOrange,2.0");
        let expensive = df.filter(|row| row["price"] > 1.0);
        expensive.rows()
    "#;

    let result = eval(code).expect("Should filter CSV data");
    assert_eq!(result.to_string(), "2"); // Apple and Orange
}

#[test]
fn test_filter_complex_expression() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3, 4, 5])
            .column("y", [10, 20, 30, 40, 50])
            .build();

        // Filter where x*10 + y > 50
        let filtered = df.filter(|row| row["x"] * 10 + row["y"] > 50);
        filtered.rows()
    "#;

    let result = eval(code).expect("Should handle complex expressions");
    assert_eq!(result.to_string(), "3"); // x=3,4,5
}
