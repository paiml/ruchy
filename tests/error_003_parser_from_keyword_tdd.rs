// [ERROR-003] TDD tests for `from` keyword as method name
// Tests FIRST, then implementation (Toyota Way)
// Root Cause: `from` is a reserved keyword but should be allowed as method name after ::

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Evaluate Ruchy code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

// Test 1: Basic String::from
#[test]
fn test_string_from_basic() {
    let code = r#"
        String::from("hello")
    "#;

    let result = eval(code).expect("Should parse String::from");
    assert_eq!(result.to_string(), "\"hello\"");
}

// Test 2: String::from in function
#[test]
fn test_string_from_in_function() {
    let code = r#"
        fun create_message() -> String {
            String::from("Hello, World!")
        }

        create_message()
    "#;

    let result = eval(code).expect("Should use String::from in function");
    assert_eq!(result.to_string(), "\"Hello, World!\"");
}

// Test 3: String::from with early return
#[test]
fn test_string_from_with_return() {
    let code = r#"
        fun get_default(flag: bool) -> String {
            if flag {
                return String::from("default");
            }
            String::from("other")
        }

        get_default(true)
    "#;

    let result = eval(code).expect("Should handle String::from with return");
    assert_eq!(result.to_string(), "\"default\"");
}

// Test 4: Multiple String::from calls
#[test]
fn test_multiple_string_from() {
    let code = r#"
        fun concat_strings() -> String {
            let a = String::from("Hello");
            let b = String::from("World");
            a + " " + b
        }

        concat_strings()
    "#;

    let result = eval(code).expect("Should handle multiple String::from");
    assert!(result.to_string().contains("Hello"));
}
