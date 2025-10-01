//! DataFrame .get() method tests
//!
//! Tests for the DataFrame.get(column, row) accessor method.
//! Follows EXTREME TDD methodology with <10 cyclomatic complexity.

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
fn test_dataframe_get_basic_string() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,city\nAlice,NYC\nBob,SF");
        df.get("name", 0)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "\"Alice\"");
}

#[test]
fn test_dataframe_get_basic_integer() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25\nBob,30");
        df.get("age", 1)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "30");
}

#[test]
fn test_dataframe_get_float_value() {
    let code = r#"
        let df = DataFrame::from_csv_string("product,price\nWidget,99.99\nGadget,149.50");
        df.get("price", 0)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "99.99");
}

#[test]
fn test_dataframe_get_last_row() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,score\nAlice,85\nBob,92\nCharlie,78");
        df.get("name", 2)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "\"Charlie\"");
}

#[test]
fn test_dataframe_get_from_builder() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [10, 20, 30])
            .column("y", [100, 200, 300])
            .build();
        df.get("y", 1)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "200");
}

#[test]
fn test_dataframe_get_from_json() {
    let code = r#"
        let json_str = "[{\"city\": \"NYC\", \"temp\": 72}]";
        let df = DataFrame::from_json(json_str);
        df.get("temp", 0)
    "#;
    let result = eval(code).expect("DataFrame.get() should work");
    assert_eq!(result.to_string(), "72");
}

// Edge cases and error handling

#[test]
fn test_dataframe_get_column_not_found() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25");
        df.get("nonexistent", 0)
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Column 'nonexistent' not found"));
}

#[test]
fn test_dataframe_get_row_out_of_bounds() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25\nBob,30");
        df.get("name", 5)
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("out of bounds"));
}

#[test]
fn test_dataframe_get_negative_index() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25");
        df.get("name", -1)
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be non-negative"));
}

#[test]
fn test_dataframe_get_wrong_arg_count() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25");
        df.get("name")
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("exactly 2 arguments"));
}

#[test]
fn test_dataframe_get_non_string_column() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25");
        df.get(123, 0)
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be a string"));
}

#[test]
fn test_dataframe_get_non_integer_index() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,age\nAlice,25");
        df.get("name", "zero")
    "#;
    let result = eval(code);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be an integer"));
}

// Integration tests with other DataFrame methods

#[test]
fn test_dataframe_get_after_sort() {
    let code = r#"
        let df = DataFrame::from_csv_string("name,score\nCharlie,78\nAlice,92\nBob,85");
        let sorted = df.sort_by("score");
        sorted.get("name", 0)
    "#;
    let result = eval(code).expect("DataFrame.get() after sort should work");
    assert_eq!(result.to_string(), "\"Charlie\"");
}

#[test]
fn test_dataframe_get_after_with_column() {
    let code = r#"
        let df = DataFrame::from_csv_string("x,y\n10,20\n30,40");
        let with_sum = df.with_column("sum", row => row["x"] + row["y"]);
        with_sum.get("sum", 1)
    "#;
    let result = eval(code).expect("DataFrame.get() after with_column should work");
    assert_eq!(result.to_string(), "70");
}

#[test]
fn test_dataframe_get_multiple_calls() {
    let code = r#"
        let df = DataFrame::from_csv_string("product,qty\nWidget,10\nGadget,5");
        let name = df.get("product", 0);
        let qty = df.get("qty", 1);
        [name, qty]
    "#;
    let result = eval(code).expect("Multiple DataFrame.get() calls should work");
    assert_eq!(result.to_string(), "[\"Widget\", 5]");
}
