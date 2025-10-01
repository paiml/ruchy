//! BOOK COMPATIBILITY: Interpreter TDD Tests
//!
//! Following CLAUDE.md EXTREME TDD Protocol for Book Compatibility Sprint
//! Tests written FIRST before implementation
//! Source: ruchy-book INTEGRATION.md + experiments/

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

// ============================================================================
// BOOK-001: STRING MULTIPLICATION OPERATOR
// ============================================================================

#[test]
fn test_string_multiply_positive() {
    // Test: "hello" * 3 should produce "hellohellohello"
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 3"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "hellohellohello",
                "String * 3 should repeat string 3 times"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_zero() {
    // Test: "hello" * 0 should produce empty string
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 0"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "", "String * 0 should produce empty string");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_one() {
    // Test: "hello" * 1 should produce "hello"
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * 1"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "hello", "String * 1 should produce same string");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_separator() {
    // Test: "=" * 50 (common pattern from experiments)
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""=" * 50"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.len(), 50, "String * 50 should have length 50");
            assert!(s.chars().all(|c| c == '='), "All characters should be '='");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_empty() {
    // Test: "" * 100 should produce empty string
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""" * 100"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "",
                "Empty string * n should produce empty string"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_negative() {
    // Test: "hello" * -1 should produce empty string (Python behavior)
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""hello" * -1"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(
                s.as_ref(),
                "",
                "String * negative should produce empty string"
            );
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_large() {
    // Test: Large multiplication doesn't panic
    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(r#""x" * 1000"#);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.len(), 1000, "String * 1000 should have length 1000");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

#[test]
fn test_string_multiply_with_variable() {
    // Test: Variable binding with string multiplication
    let code = r#"
        let sep = "="
        sep * 10
    "#;

    let mut interpreter = Interpreter::new();
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let result = interpreter.eval_expr(&ast).expect("Eval failed");

    match result {
        Value::String(s) => {
            assert_eq!(s.as_ref(), "==========", "Variable * 10 should work");
        }
        _ => panic!("Expected String, got {:?}", result),
    }
}

// ============================================================================
// PROPERTY-BASED TESTS: String Multiplication
// ============================================================================

#[cfg(test)]
mod string_multiply_properties {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_string_multiply_length_invariant(
            s in "[a-z]{0,10}",
            n in 0..100i64
        ) {
            // Property: (s * n).len() == s.len() * n for non-negative n
            let code = format!(r#""{}" * {}"#, s, n);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    let expected_len = s.len() * (n as usize);
                    prop_assert_eq!(result.len(), expected_len,
                        "String multiplication should preserve length property");
                }
            }
        }

        #[test]
        fn test_string_multiply_content_invariant(
            s in "[a-z]{1,5}",
            n in 1..20i64
        ) {
            // Property: Result consists only of repetitions of original string
            let code = format!(r#""{}" * {}"#, s, n);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    // Check that result can be evenly divided into chunks of s
                    for i in 0..(n as usize) {
                        let start = i * s.len();
                        let end = start + s.len();
                        if end <= result.len() {
                            let chunk = &result[start..end];
                            prop_assert_eq!(chunk, s.as_str(),
                                "Each chunk should be the original string");
                        }
                    }
                }
            }
        }

        #[test]
        fn test_string_multiply_zero_always_empty(s in "[a-z]{0,10}") {
            // Property: Any string * 0 = ""
            let code = format!(r#""{}" * 0"#, s);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    prop_assert_eq!(result.as_ref(), "",
                        "String * 0 should always be empty");
                }
            }
        }

        #[test]
        fn test_string_multiply_one_identity(s in "[a-z]{0,10}") {
            // Property: Any string * 1 = original string
            let code = format!(r#""{}" * 1"#, s);
            let mut interpreter = Interpreter::new();
            let mut parser = Parser::new(&code);

            if let Ok(ast) = parser.parse() {
                if let Ok(Value::String(result)) = interpreter.eval_expr(&ast) {
                    prop_assert_eq!(result.as_ref(), s.as_str(),
                        "String * 1 should be identity");
                }
            }
        }
    }
}
