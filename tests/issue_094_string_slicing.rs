//! ISSUE-094: String Slicing Not Available
//!
//! Test suite for string slicing feature: `text[0..5]`
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

/// Helper to evaluate Ruchy code and return result
fn eval_ruchy(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => return Err(format!("Parse error: {:?}", e)),
    };

    let mut interpreter = Interpreter::new();
    match interpreter.eval_expr(&ast) {
        Ok(value) => {
            // Extract actual string content without quotes
            match value {
                ruchy::runtime::interpreter::Value::String(s) => Ok(s.to_string()),
                ruchy::runtime::interpreter::Value::Integer(i) => Ok(i.to_string()),
                _ => Ok(value.to_string()),
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[test]
fn test_issue_094_basic_string_slice() {
    // Basic string slicing: text[0..5]
    let code = r#"
        let text = "Hello, World!";
        text[0..5]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "String slicing should work: {:?}", result);
    assert_eq!(result.unwrap(), "Hello", "Should slice first 5 characters");
}

#[test]
fn test_issue_094_mid_string_slice() {
    // Slicing from middle of string
    let code = r#"
        let text = "Hello, World!";
        text[7..12]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Mid-string slicing should work: {:?}", result);
    assert_eq!(result.unwrap(), "World", "Should slice middle section");
}

#[test]
fn test_issue_094_open_ended_slice_from_start() {
    // Open range from start: text[..5]
    let code = r#"
        let text = "Hello, World!";
        text[..5]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Open-start slice should work: {:?}", result);
    assert_eq!(result.unwrap(), "Hello", "Should slice from beginning");
}

#[test]
fn test_issue_094_open_ended_slice_to_end() {
    // Open range to end: text[7..]
    let code = r#"
        let text = "Hello, World!";
        text[7..]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Open-end slice should work: {:?}", result);
    assert_eq!(result.unwrap(), "World!", "Should slice to end");
}

#[test]
fn test_issue_094_full_range_slice() {
    // Full range: text[..]
    let code = r#"
        let text = "Hello, World!";
        text[..]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Full range slice should work: {:?}", result);
    assert_eq!(result.unwrap(), "Hello, World!", "Should return full string");
}

#[test]
fn test_issue_094_empty_slice() {
    // Empty slice: text[5..5]
    let code = r#"
        let text = "Hello, World!";
        text[5..5]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Empty slice should work: {:?}", result);
    assert_eq!(result.unwrap(), "", "Empty slice should return empty string");
}

#[test]
fn test_issue_094_out_of_bounds_slice() {
    // Out of bounds should error gracefully
    let code = r#"
        let text = "Hello";
        text[0..100]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_err(), "Out of bounds slice should error");
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("out of bounds") || err_msg.contains("range"),
        "Error should mention bounds, got: {}",
        err_msg
    );
}

#[test]
fn test_issue_094_reversed_range() {
    // Reversed range (start > end) should error
    let code = r#"
        let text = "Hello";
        text[5..2]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_err(), "Reversed range should error");
}

#[test]
fn test_issue_094_slice_assignment() {
    // String slicing in variable assignment
    let code = r#"
        let text = "Ruchy Language";
        let name = text[0..5];
        let type_str = text[6..14];
        name + " " + type_str
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Slice assignment should work: {:?}", result);
    assert_eq!(result.unwrap(), "Ruchy Language", "Should concatenate slices");
}

#[test]
fn test_issue_094_slice_in_function() {
    // String slicing inside function
    let code = r#"
        fun get_first_word(s: String) -> String {
            s[0..5]
        }

        get_first_word("Hello World")
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Slicing in function should work: {:?}", result);
    assert_eq!(result.unwrap(), "Hello", "Should return sliced string");
}

#[test]
fn test_issue_094_chained_operations() {
    // Chaining slice with other operations
    let code = r#"
        let text = "hello world";
        let slice = text[0..5];
        slice.len()
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "Chained operations should work: {:?}", result);
    assert_eq!(result.unwrap(), "5", "Slice length should be 5");
}

#[test]
fn test_issue_094_utf8_ascii_slice() {
    // UTF-8 slicing with ASCII characters
    let code = r#"
        let text = "ABC";
        text[0..2]
    "#;

    let result = eval_ruchy(code);
    assert!(result.is_ok(), "ASCII UTF-8 slice should work: {:?}", result);
    assert_eq!(result.unwrap(), "AB", "Should slice ASCII correctly");
}

// Property-based tests would go here using proptest
// For now, we have comprehensive unit tests covering the core functionality
