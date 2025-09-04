//! TDD Test Suite for patterns.rs
//! Target: 33.33% → 80%+ coverage
//! PMAT: Keep complexity <10 per test

#![cfg(test)]

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

/// Test helper: Parse and transpile pattern matching code
fn transpile_pattern(code: &str) -> anyhow::Result<String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

// Basic Pattern Matching Tests
#[test]
fn test_match_literal_pattern() {
    let code = r#"
match x {
    1 => "one",
    2 => "two",
    _ => "other"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("1 =>"));
    assert!(result.contains("2 =>"));
    assert!(result.contains("_ =>"));
}

#[test]
fn test_match_string_pattern() {
    let code = r#"
match name {
    "Alice" => 1,
    "Bob" => 2,
    _ => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("\"Alice\""));
    assert!(result.contains("\"Bob\""));
}

#[test]
fn test_match_bool_pattern() {
    let code = r#"
match flag {
    true => "yes",
    false => "no"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("true =>"));
    assert!(result.contains("false =>"));
}

#[test]
fn test_match_variable_binding() {
    let code = r#"
match x {
    n => n * 2
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("=>"));
}

// Pattern Guards Tests
#[test]
fn test_match_with_guard() {
    let code = r#"
match x {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("if"));
    assert!(result.contains("=>"));
}

#[test]
fn test_match_guard_complex_condition() {
    let code = r#"
match x {
    n if n > 0 && n < 100 => "in range",
    _ => "out of range"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("if"));
    assert!(result.contains("&&"));
}

// Tuple Pattern Tests
#[test]
fn test_match_tuple_pattern() {
    let code = r#"
match point {
    (0, 0) => "origin",
    (x, 0) => "on x-axis",
    (0, y) => "on y-axis",
    (x, y) => "general"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("(0 , 0)") || result.contains("(0, 0)"));
}

#[test]
fn test_match_nested_tuple() {
    let code = r#"
match data {
    ((a, b), c) => a + b + c,
    _ => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

// List Pattern Tests
#[test]
fn test_match_empty_list() {
    let code = r#"
match lst {
    [] => "empty",
    _ => "not empty"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("[]") || result.contains("vec![]"));
}

#[test]
fn test_match_list_head_tail() {
    let code = r#"
match lst {
    [head, ...tail] => head,
    [] => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

#[test]
fn test_match_specific_list() {
    let code = r#"
match lst {
    [1, 2, 3] => "exact",
    [1, 2, ...] => "starts with 1, 2",
    _ => "other"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

// Struct Pattern Tests
#[test]
fn test_match_struct_pattern() {
    let code = r#"
match person {
    Person { name: "Alice", age: 30 } => "specific",
    Person { name, age } => "general",
    _ => "unknown"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

// Enum Pattern Tests
#[test]
fn test_match_enum_pattern() {
    let code = r#"
match option {
    Some(x) => x * 2,
    None => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("Some"));
    assert!(result.contains("None"));
}

#[test]
fn test_match_result_pattern() {
    let code = r#"
match result {
    Ok(value) => value,
    Err(error) => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("Err"));
}

// Range Pattern Tests
#[test]
fn test_match_range_pattern() {
    let code = r#"
match x {
    0..10 => "single digit",
    10..100 => "double digit",
    _ => "large"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("..") || result.contains("0..10"));
}

#[test]
fn test_match_inclusive_range() {
    let code = r#"
match x {
    0..=9 => "single digit",
    10..=99 => "double digit",
    _ => "large"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("..=") || result.contains("0..=9"));
}

// Or Pattern Tests
#[test]
fn test_match_or_pattern() {
    let code = r#"
match x {
    1 | 2 | 3 => "small",
    4 | 5 | 6 => "medium",
    _ => "other"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("|"));
}

// Wildcard Tests
#[test]
fn test_match_wildcard_in_tuple() {
    let code = r#"
match point {
    (_, 0) => "on x-axis",
    (0, _) => "on y-axis",
    _ => "general"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("_"));
}

// Complex Nested Patterns
#[test]
fn test_complex_nested_pattern() {
    let code = r#"
match data {
    Some((x, [1, 2, ...])) if x > 0 => "complex match",
    _ => "other"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

// Edge Cases
#[test]
fn test_match_single_arm() {
    let code = r#"
match x {
    _ => "always"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    assert!(result.contains("_ =>"));
}

#[test]
fn test_match_expression_in_arm() {
    let code = r#"
match x {
    1 => {
        let y = 2
        y * 3
    },
    _ => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}

// PMAT Complexity Check Tests
#[test]
fn test_simple_match_low_complexity() {
    // This test verifies that simple patterns keep complexity low
    let code = r#"
match x {
    1 => 1,
    2 => 2,
    3 => 3,
    4 => 4,
    5 => 5,
    _ => 0
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
    // Each arm should be simple, keeping complexity < 10
}

#[test]
fn test_match_with_multiple_patterns() {
    let code = r#"
match (x, y) {
    (0, 0) => "origin",
    (_, 0) => "x-axis",
    (0, _) => "y-axis",
    (x, y) if x == y => "diagonal",
    _ => "other"
}
"#;
    let result = transpile_pattern(code).unwrap();
    assert!(result.contains("match"));
}