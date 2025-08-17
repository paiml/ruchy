//! `DataFrame` functionality tests

#![allow(clippy::unwrap_used)] // OK in tests

use ruchy::{compile, get_parse_error, is_valid_syntax};

#[test]
fn test_dataframe_empty() {
    assert!(is_valid_syntax("df![]"));
    let result = compile("df![]").unwrap();
    assert!(result.contains("DataFrame") && result.contains("empty"));
}

#[test]
fn test_dataframe_single_column() {
    // New syntax: column => values
    assert!(is_valid_syntax("df![age => [25, 30, 35]]"));
    let result = compile("df![age => [25, 30, 35]]").unwrap();
    assert!(result.contains("Series") && result.contains("new"));
    assert!(result.contains("\"age\""));
}

#[test]
fn test_dataframe_multiple_columns() {
    // Multiple columns with arrow syntax
    let code = "df![
        name => [\"Alice\", \"Bob\", \"Charlie\"],
        age => [25, 30, 35],
        score => [95.5, 87.3, 92.1]
    ]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("DataFrame") && result.contains("new"));
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"age\""));
    assert!(result.contains("\"score\""));
}

#[test]
fn test_dataframe_with_expressions() {
    // Using expressions in values
    let code = "df![
        x => [1, 2, 3],
        y => [1 * 2, 2 * 2, 3 * 2]
    ]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("DataFrame") && result.contains("new"));
}

#[test]
fn test_dataframe_legacy_syntax() {
    // Test backward compatibility with semicolon-separated rows
    let code = "df![name, age; \"Alice\", 25; \"Bob\", 30]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("DataFrame") && result.contains("new"));
    assert!(result.contains("\"name\""));
    assert!(result.contains("\"age\""));
}

#[test]
fn test_dataframe_operations() {
    // Filter operation - will test this when method chaining is implemented
    // let code = "df![age => [25, 30, 35]].filter(age > 25)";
    // For now, just test that the DataFrame literal compiles
    let df_code = "df![age => [25, 30, 35]]";
    assert!(is_valid_syntax(df_code));
}

#[test]
fn test_dataframe_invalid_syntax() {
    // Missing closing bracket
    assert!(get_parse_error("df![name => [\"Alice\"").is_some());

    // Invalid arrow syntax
    assert!(get_parse_error("df![name -> [\"Alice\"]]").is_some());

    // Empty column name
    assert!(get_parse_error("df![ => [1, 2, 3]]").is_some());
}

#[test]
fn test_dataframe_nested_lists() {
    // Column with list values
    let code = "df![
        id => [1, 2, 3],
        tags => [[\"a\", \"b\"], [\"c\"], [\"d\", \"e\", \"f\"]]
    ]";
    assert!(is_valid_syntax(code));
}

#[test]
#[cfg(feature = "dataframe")]
fn test_dataframe_polars_integration() {
    // This test only runs when the dataframe feature is enabled
    let code = "df![
        product => [\"Apple\", \"Banana\", \"Orange\"],
        price => [1.5, 0.8, 2.0],
        quantity => [10, 20, 15]
    ]";

    let result = compile(code).unwrap();
    // Check for polars-related types
    assert!(result.contains("DataFrame") || result.contains("Series"));
    assert!(result.contains("DataFrame") && result.contains("new"));
    assert!(result.contains("Series") && result.contains("new"));
}
