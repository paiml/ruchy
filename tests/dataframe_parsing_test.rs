//! Dataframe Parsing Tests - HYBRID-C-4
//! Empirical verification that dataframe literals already parse correctly

use ruchy::Parser;

#[test]
fn test_simple_dataframe_literal() {
    let code = r#"df!["name" => ["Alice"], "age" => [30]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Simple dataframe should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_multiple_rows() {
    let code = r#"df!["name" => ["Alice", "Bob"], "age" => [30, 25]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Multi-row dataframe should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_empty() {
    let code = r#"df![]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Empty dataframe should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_single_column() {
    let code = r#"df!["values" => [1, 2, 3, 4, 5]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Single column dataframe should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_mixed_types() {
    let code = r#"df!["name" => ["Alice"], "age" => [30], "score" => [95.5]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Mixed type dataframe should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_in_variable_assignment() {
    let code = r#"let data = df!["x" => [1, 2], "y" => [3, 4]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Dataframe in assignment should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_in_function_call() {
    let code = r#"process(df!["data" => [1, 2, 3]])"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Dataframe in function call should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_dataframe_with_string_values() {
    let code = r#"df!["names" => ["Alice", "Bob", "Charlie"]]"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Dataframe with strings should parse: {:?}",
        result.err()
    );
}
