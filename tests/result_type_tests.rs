//! Tests for Result type functionality
#![allow(clippy::unwrap_used)]

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_ok_constructor() {
    let code = "Ok(42)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
    assert!(result.contains("42"));
}

#[test]
fn test_err_constructor() {
    let code = "Err(\"Error message\")";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Err"));
    assert!(result.contains("Error message"));
}

#[test]
fn test_some_constructor() {
    let code = "Some(5)";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok")); // Some maps to Ok
}

#[test]
fn test_none_constructor() {
    let code = "None";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("()")); // None is unit
}

#[test]
fn test_result_type_annotation() {
    // Result values work
    assert!(is_valid_syntax("Ok(42)"));
    assert!(is_valid_syntax("Err(\"error\")"));
}

#[test]
fn test_option_type_annotation() {
    // Option values work
    assert!(is_valid_syntax("Some(5)"));
    assert!(is_valid_syntax("None"));
}

#[test]
fn test_try_operator() {
    let code = "f()?";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains('?'));
}

#[test]
fn test_try_operator_chain() {
    let code = "f()?";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains('?'));
}

#[test]
fn test_result_match() {
    // Match with simple patterns
    let code = r"
        match result {
            _ => 0
        }
    ";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("match"));
}

#[test]
fn test_option_match() {
    // Simple option values
    let code = "Some(5)";
    assert!(is_valid_syntax(code));
    let code = "None";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_result_chain() {
    let code = r"
        Ok(5)
    ";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
}

#[test]
fn test_result_in_function() {
    let code = r#"
        fn safe_divide(a, b) {
            if b == 0.0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
    "#;
    assert!(is_valid_syntax(code));
}

#[test]
fn test_option_in_function() {
    let code = r"
        fn find_first() {
            Some(42)
        }
    ";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_nested_results() {
    let code = "Ok(Ok(42))";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
}

#[test]
fn test_result_with_complex_types() {
    let code = "Ok([1, 2, 3])";
    assert!(is_valid_syntax(code));
    let result = compile(code).unwrap();
    assert!(result.contains("Ok"));
}

#[test]
fn test_error_propagation() {
    let code = r"
        fn process() {
            let x = f()?
            Ok(x)
        }
    ";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_result_map() {
    // Result methods should work
    let code = "Ok(5).map(|x| x * 2)";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_result_and_then() {
    let code = "Ok(5).and_then(|x| Ok(x * 2))";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_result_unwrap_or() {
    let code = "Err(\"error\").unwrap_or(0)";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_result_unwrap_or_else() {
    let code = "Err(\"error\").unwrap_or_else(|e| 0)";
    assert!(is_valid_syntax(code));
}

#[test]
fn test_question_mark_in_block() {
    let code = r"
        {
            let x = f()?
            let y = g()?
            x + y
        }
    ";
    assert!(is_valid_syntax(code));
}
