//! Tests for Result type support
#![allow(clippy::unwrap_used)] // Tests need unwrap for assertions

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_ok_constructor() {
    assert!(is_valid_syntax("Ok(42)"));
    let result = compile("Ok(42)").unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("42"));
}

#[test]
fn test_err_constructor() {
    assert!(is_valid_syntax(r#"Err("error message")"#));
    let result = compile(r#"Err("error message")"#).unwrap();
    assert!(result.contains("Err"));
    assert!(result.contains("error message"));
}

#[test]
fn test_try_operator() {
    assert!(is_valid_syntax("result?"));
    let result = compile("result?").unwrap();
    assert!(result.contains("result"));
    assert!(result.contains('?'));
}

#[test]
fn test_result_chain() {
    let code = r"
        let x = Ok(10) in
        let y = x? in
        y + 5
    ";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("10"));
    assert!(result.contains('?'));
}

#[test]
fn test_function_returning_result() {
    let code = r#"
        fun divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
    "#;
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("Err"));
}

#[test]
fn test_result_pattern_matching() {
    let code = r"
        match result {
            Ok(value) => value * 2,
            Err(msg) => 0
        }
    ";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("match"));
}

#[test]
fn test_nested_results() {
    let code = "Ok(Ok(42))";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("42"));
}

#[test]
fn test_result_in_list() {
    let code = "[Ok(1), Err(2), Ok(3)]";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("Err"));
    assert!(result.contains('1'));
    assert!(result.contains('2'));
    assert!(result.contains('3'));
}

#[test]
fn test_try_in_expression() {
    let code = "let x = Ok(1) in x?";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains('?'));
}

#[test]
fn test_result_with_complex_types() {
    let code = r"Ok([1, 2, 3])";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("vec"));
    assert!(result.contains('!'));
}
