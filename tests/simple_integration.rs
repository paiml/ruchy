//! Simple integration tests that exercise the compiler pipeline
//! These tests are designed to actually compile and run

use ruchy::frontend::{ast::*, Parser};
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;

#[test]
fn test_parse_and_eval_integer() {
    // Parse
    let mut parser = Parser::new("42");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_parse_and_eval_addition() {
    // Parse
    let mut parser = Parser::new("10 + 32");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_parse_and_eval_boolean() {
    // Parse true
    let mut parser = Parser::new("true");
    let expr = parser.parse().expect("Failed to parse");

    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");
    assert_eq!(result, Value::Bool(true));

    // Parse false
    let mut parser = Parser::new("false");
    let expr = parser.parse().expect("Failed to parse");

    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_parse_and_eval_comparison() {
    // Parse
    let mut parser = Parser::new("5 > 3");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_parse_and_eval_string() {
    // Parse
    let mut parser = Parser::new(r#""hello world""#);
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::String(Rc::from("hello world")));
}

#[test]
fn test_parse_and_eval_parentheses() {
    // Parse (2 + 3) * 4
    let mut parser = Parser::new("(2 + 3) * 4");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Integer(20));
}

#[test]
fn test_parse_and_eval_nested_arithmetic() {
    // Parse complex expression
    let mut parser = Parser::new("1 + 2 * 3 - 4");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate (should be 1 + 6 - 4 = 3)
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_parse_and_eval_logical_and() {
    // Parse
    let mut parser = Parser::new("true && false");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_parse_and_eval_logical_or() {
    // Parse
    let mut parser = Parser::new("false || true");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_parse_and_eval_equality() {
    // Parse
    let mut parser = Parser::new("42 == 42");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Bool(true));

    // Parse inequality
    let mut parser = Parser::new("42 != 24");
    let expr = parser.parse().expect("Failed to parse");

    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_parse_and_eval_unary_negation() {
    // Parse
    let mut parser = Parser::new("-42");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Integer(-42));
}

#[test]
fn test_parse_and_eval_unary_not() {
    // Parse
    let mut parser = Parser::new("!true");
    let expr = parser.parse().expect("Failed to parse");

    // Evaluate
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&expr).expect("Failed to evaluate");

    assert_eq!(result, Value::Bool(false));
}

// Property-based integration tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_parse_eval_integer(n in i32::MIN/2..i32::MAX/2) {
            let input = n.to_string();
            let mut parser = Parser::new(&input);

            if let Ok(expr) = parser.parse() {
                let mut interpreter = Interpreter::new();
                if let Ok(result) = interpreter.eval_expr(&expr) {
                    prop_assert_eq!(result, Value::Integer(n as i64));
                }
            }
        }

        #[test]
        fn prop_parse_eval_addition(a in -1000i32..1000, b in -1000i32..1000) {
            let input = format!("{} + {}", a, b);
            let mut parser = Parser::new(&input);

            if let Ok(expr) = parser.parse() {
                let mut interpreter = Interpreter::new();
                if let Ok(result) = interpreter.eval_expr(&expr) {
                    prop_assert_eq!(result, Value::Integer((a + b) as i64));
                }
            }
        }

        #[test]
        fn prop_parse_eval_subtraction(a in -1000i32..1000, b in -1000i32..1000) {
            let input = format!("{} - {}", a, b);
            let mut parser = Parser::new(&input);

            if let Ok(expr) = parser.parse() {
                let mut interpreter = Interpreter::new();
                if let Ok(result) = interpreter.eval_expr(&expr) {
                    prop_assert_eq!(result, Value::Integer((a - b) as i64));
                }
            }
        }
    }
}
