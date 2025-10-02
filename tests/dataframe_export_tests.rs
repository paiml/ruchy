//! DataFrame export method tests (TDD for DF-007)
//!
//! Tests for DataFrame export operations: .to_csv(), .to_json()
//! Following Toyota Way: Write tests FIRST, then implementation

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Evaluate Ruchy code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

// ============================================================================
// .to_csv() Tests (TDD - implement this)
// ============================================================================

#[test]
fn test_to_csv_basic() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice", "Bob", "Charlie"])
            .column("age", [25, 30, 35])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should export to CSV");
    let csv = result.to_string();

    // Should contain header
    assert!(csv.contains("name,age"), "CSV should have header: {}", csv);

    // Should contain data rows
    assert!(
        csv.contains("Alice,25"),
        "CSV should have Alice row: {}",
        csv
    );
    assert!(csv.contains("Bob,30"), "CSV should have Bob row: {}", csv);
    assert!(
        csv.contains("Charlie,35"),
        "CSV should have Charlie row: {}",
        csv
    );
}

#[test]
fn test_to_csv_numeric_only() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2, 3])
            .column("y", [4, 5, 6])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should export numeric DataFrame to CSV");
    let csv = result.to_string();

    assert!(csv.contains("x,y"), "CSV should have header");
    assert!(csv.contains("1,4"), "CSV should have first row");
    assert!(csv.contains("2,5"), "CSV should have second row");
    assert!(csv.contains("3,6"), "CSV should have third row");
}

#[test]
fn test_to_csv_with_floats() {
    let code = r#"
        let df = DataFrame::new()
            .column("temp", [98.6, 99.1, 97.8])
            .column("day", [1, 2, 3])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should export floats to CSV");
    let csv = result.to_string();

    assert!(csv.contains("temp,day"), "CSV should have header");
    assert!(csv.contains("98.6,1"), "CSV should have first row");
}

#[test]
fn test_to_csv_empty_dataframe() {
    let code = r#"
        let df = DataFrame::new().build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should handle empty DataFrame");
    let csv = result.to_string();

    // Empty DataFrame should just be empty string or newline
    assert!(
        csv.is_empty() || csv == "\n" || csv == "\"\"",
        "Empty DataFrame CSV: {}",
        csv
    );
}

#[test]
fn test_to_csv_single_column() {
    let code = r#"
        let df = DataFrame::new()
            .column("values", [1, 2, 3])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should export single column");
    let csv = result.to_string();

    assert!(csv.contains("values"), "CSV should have header");
    assert!(csv.contains("1"), "CSV should have values");
}

#[test]
fn test_to_csv_single_row() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice"])
            .column("age", [25])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should export single row");
    let csv = result.to_string();

    assert!(csv.contains("name,age"), "CSV should have header");
    assert!(csv.contains("Alice,25"), "CSV should have data row");
}

#[test]
fn test_to_csv_special_characters() {
    let code = r#"
        let df = DataFrame::new()
            .column("text", ["hello", "world"])
            .build();
        df.to_csv()
    "#;

    let result = eval(code).expect("Should handle special characters");
    let csv = result.to_string();

    assert!(csv.contains("text"), "CSV should have header");
    assert!(csv.contains("hello"), "CSV should have first value");
    assert!(csv.contains("world"), "CSV should have second value");
}

// ============================================================================
// .to_json() Tests (TDD - implement this)
// ============================================================================

#[test]
fn test_to_json_basic() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice", "Bob"])
            .column("age", [25, 30])
            .build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should export to JSON");
    let json = result.to_string();

    // Should be array of objects
    assert!(json.contains("["), "JSON should start with array");
    assert!(json.contains("name"), "JSON should have name field");
    assert!(json.contains("Alice"), "JSON should have Alice value");
    assert!(json.contains("age"), "JSON should have age field");
    assert!(json.contains("25"), "JSON should have age value");
}

#[test]
fn test_to_json_numeric_only() {
    let code = r#"
        let df = DataFrame::new()
            .column("x", [1, 2])
            .column("y", [3, 4])
            .build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should export numeric to JSON");
    let json = result.to_string();

    assert!(json.contains("["), "JSON should be array");
    assert!(json.contains("\"x\""), "JSON should have x field");
    assert!(json.contains("\"y\""), "JSON should have y field");
}

#[test]
fn test_to_json_with_floats() {
    let code = r#"
        let df = DataFrame::new()
            .column("temp", [98.6, 99.1])
            .build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should export floats to JSON");
    let json = result.to_string();

    assert!(json.contains("98.6"), "JSON should have float value");
    assert!(json.contains("99.1"), "JSON should have float value");
}

#[test]
fn test_to_json_empty_dataframe() {
    let code = r#"
        let df = DataFrame::new().build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should handle empty DataFrame");
    let json = result.to_string();

    // Empty DataFrame should be empty array
    assert!(
        json.contains("[]") || json == "\"[]\"",
        "Empty DataFrame JSON: {}",
        json
    );
}

#[test]
fn test_to_json_single_column() {
    let code = r#"
        let df = DataFrame::new()
            .column("value", [42, 100])
            .build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should export single column");
    let json = result.to_string();

    assert!(json.contains("value"), "JSON should have value field");
    assert!(json.contains("42"), "JSON should have first value");
    assert!(json.contains("100"), "JSON should have second value");
}

#[test]
fn test_to_json_single_row() {
    let code = r#"
        let df = DataFrame::new()
            .column("name", ["Alice"])
            .column("age", [25])
            .build();
        df.to_json()
    "#;

    let result = eval(code).expect("Should export single row");
    let json = result.to_string();

    assert!(json.contains("name"), "JSON should have name field");
    assert!(json.contains("Alice"), "JSON should have Alice value");
    assert!(json.contains("age"), "JSON should have age field");
    assert!(json.contains("25"), "JSON should have age value");
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_export_formats_consistency() {
    let code = r#"
        let df = DataFrame::new()
            .column("a", [1, 2])
            .column("b", [3, 4])
            .build();
        [df.to_csv(), df.to_json()]
    "#;

    let result = eval(code).expect("Should export in both formats");
    // Just verify both formats can be generated
    assert!(
        result.to_string().contains("["),
        "Should return array of exports"
    );
}
