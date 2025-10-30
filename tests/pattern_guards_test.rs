#![allow(missing_docs)]
//! Pattern Guards Tests - HYBRID-C-5
//! Empirical verification that pattern guards already work

use ruchy::Parser;

#[test]
fn test_pattern_guard_positive() {
    let code = r#"
match 5 {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Pattern guards should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pattern_guard_negative() {
    let code = r#"
match -3 {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Pattern guards should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pattern_guard_zero() {
    let code = r#"
match 0 {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Pattern guards should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pattern_guard_complex_condition() {
    let code = r#"
match 42 {
    n if n > 10 && n < 100 => "medium",
    n if n >= 100 => "large",
    _ => "small"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Complex pattern guards should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_pattern_guard_string_match() {
    let code = r#"
match "hello" {
    s if s.len() > 3 => "long",
    _ => "short"
}
"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(
        result.is_ok(),
        "Pattern guards on strings should parse: {:?}",
        result.err()
    );
}
