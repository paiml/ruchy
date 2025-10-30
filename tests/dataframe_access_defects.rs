//! DEFECT-DATAFRAME-001 & 002: `DataFrame` Indexing and Field Access
//! 
//! RED Phase Tests - Following EXTREME TDD
//! 
//! Root Cause: `eval_index_access()` and `eval_field_access()` don't handle `DataFrame`
//! Location: src/runtime/interpreter.rs:1306, 1413
//!
//! Tests will FAIL until we implement `DataFrame` indexing/field access

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

#[test]
fn test_dataframe_indexing_row_access() {
    // DEFECT-DATAFRAME-001: df[0] should return first row as array/object
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df[0]
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should return first row (not error)
    assert!(result.is_ok(), "DataFrame indexing should work: {result:?}");
}

#[test]
fn test_dataframe_field_access_column() {
    // DEFECT-DATAFRAME-002: df.column_name should return column as array
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df.id
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should return [1, 2, 3] column (not error)
    assert!(result.is_ok(), "DataFrame field access should work: {result:?}");
}

#[test]
fn test_dataframe_column_access_via_string_index() {
    // Alternative: df["column_name"] should also work
    let code = r#"
        let df = df![
            "id" => [1, 2, 3],
            "name" => ["Alice", "Bob", "Charlie"]
        ];
        df["name"]
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should return ["Alice", "Bob", "Charlie"] column
    assert!(result.is_ok(), "DataFrame string indexing should work: {result:?}");
}

#[test]
fn test_dataframe_out_of_bounds_index() {
    // Error handling: df[999] should fail gracefully
    let code = r#"
        let df = df![
            "id" => [1, 2, 3]
        ];
        df[999]
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should return error (not panic)
    assert!(result.is_err(), "Out of bounds should error gracefully");
}

#[test]
fn test_dataframe_nonexistent_field() {
    // Error handling: df.nonexistent should fail gracefully
    let code = r#"
        let df = df![
            "id" => [1, 2, 3]
        ];
        df.nonexistent
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast);

    // Should return error (not panic)
    assert!(result.is_err(), "Nonexistent field should error gracefully");
}
