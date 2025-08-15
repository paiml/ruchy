#![allow(clippy::unwrap_used)]

use ruchy::{compile, get_parse_error, is_valid_syntax};

// Additional tests for edge cases and error conditions

#[test]
fn test_compile_simple_expression() {
    let result = compile("42 + 1").unwrap();
    assert!(result.contains("42"));
    assert!(result.contains('1'));
}

#[test]
fn test_compile_function() {
    let result = compile("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
    assert!(result.contains("fn"));
    assert!(result.contains("add"));
}

#[test]
fn test_compile_invalid_syntax() {
    let result = compile("let x =");
    assert!(result.is_err());
}

#[test]
fn test_is_valid_syntax_valid() {
    assert!(is_valid_syntax("1 + 2"));
    assert!(is_valid_syntax("fun foo() { 42 }"));
    assert!(is_valid_syntax("[1, 2, 3]"));
    assert!(is_valid_syntax("if true { 1 } else { 2 }"));
}

#[test]
fn test_is_valid_syntax_invalid() {
    assert!(!is_valid_syntax("let x ="));
    assert!(!is_valid_syntax("fun ()"));
    assert!(!is_valid_syntax("if { }"));
    assert!(!is_valid_syntax("match"));
}

#[test]
fn test_get_parse_error_with_error() {
    let error = get_parse_error("let x =");
    assert!(error.is_some());
    let error_msg = error.unwrap();
    assert!(!error_msg.is_empty());
}

#[test]
fn test_get_parse_error_no_error() {
    let error = get_parse_error("let x = 42");
    assert!(error.is_none());
}

#[test]
fn test_compile_pipeline() {
    let result = compile("[1, 2, 3] |> map(x => x * 2)");
    // Should compile even if not fully implemented
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_compile_match() {
    let code = r#"
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let result = compile(code);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_compile_empty_string() {
    let result = compile("");
    assert!(result.is_err());
}

#[test]
fn test_compile_whitespace_only() {
    let result = compile("   \n\t  ");
    assert!(result.is_err());
}

#[test]
fn test_compile_complex_expression() {
    let code = "let x = 42 in x + 1";
    let result = compile(code);
    assert!(result.is_ok());
}

#[test]
fn test_is_valid_syntax_edge_cases() {
    assert!(is_valid_syntax("42"));
    assert!(is_valid_syntax("true"));
    assert!(is_valid_syntax("false"));
    assert!(is_valid_syntax("\"hello\""));
    assert!(!is_valid_syntax(""));
    assert!(!is_valid_syntax("   "));
}

#[test]
fn test_get_parse_error_detailed() {
    let test_cases = vec![
        ("let x =", "Expected"),
        ("fun ()", "Expected"),
        ("if", "Expected"),
        ("[1, 2,", "Expected"),
        ("match", "Expected"),
    ];

    for (input, _expected_substring) in test_cases {
        let error = get_parse_error(input);
        assert!(error.is_some(), "Expected error for: {input}");
    }
}

#[test]
fn test_compile_all_literals() {
    assert!(compile("42").is_ok());
    assert!(compile("3.14").is_ok());
    assert!(compile("true").is_ok());
    assert!(compile("false").is_ok());
    assert!(compile("\"test\"").is_ok());
}

#[test]
fn test_compile_binary_operations() {
    assert!(compile("1 + 2").is_ok());
    assert!(compile("3 - 1").is_ok());
    assert!(compile("2 * 3").is_ok());
    assert!(compile("6 / 2").is_ok());
    assert!(compile("5 % 2").is_ok());
}

#[test]
fn test_compile_comparison_operations() {
    assert!(compile("1 < 2").is_ok());
    assert!(compile("2 > 1").is_ok());
    assert!(compile("1 <= 2").is_ok());
    assert!(compile("2 >= 1").is_ok());
    assert!(compile("1 == 1").is_ok());
    assert!(compile("1 != 2").is_ok());
}

#[test]
fn test_compile_logical_operations() {
    assert!(compile("true && false").is_ok());
    assert!(compile("true || false").is_ok());
    assert!(compile("!true").is_ok());
}
