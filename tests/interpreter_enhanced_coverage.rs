//! Enhanced interpreter tests targeting uncovered code paths
//! Goal: Boost interpreter coverage from 69.57% to 85%

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::rc::Rc;
use std::rc::Rc;

// Helper to parse and eval
fn eval(interpreter: &mut Interpreter, input: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr()?;
    Ok(interpreter.eval_expr(&expr)?)
}

/// Test float operations
#[test]
fn test_float_operations() {
    let mut interpreter = Interpreter::new();

    // Float arithmetic
    let result = eval(&mut interpreter, "3.5 + 2.5").unwrap();
    if let Value::Float(f) = result {
        assert!((f - 6.0).abs() < 0.001);
    } else {
        panic!("Expected float");
    }

    // Mixed int/float operations
    let result = eval(&mut interpreter, "5 * 2.5").unwrap();
    if let Value::Float(f) = result {
        assert!((f - 12.5).abs() < 0.001);
    } else {
        panic!("Expected float");
    }
}

/// Test comparison operations
#[test]
fn test_comparison_operations() {
    let mut interpreter = Interpreter::new();

    // Integer comparisons
    assert!(eval(&mut interpreter, "5 > 3").unwrap().is_truthy());
    assert!(eval(&mut interpreter, "3 < 5").unwrap().is_truthy());
    assert!(eval(&mut interpreter, "5 >= 5").unwrap().is_truthy());
    assert!(eval(&mut interpreter, "5 <= 5").unwrap().is_truthy());
    assert!(eval(&mut interpreter, "5 == 5").unwrap().is_truthy());
    assert!(eval(&mut interpreter, "5 != 3").unwrap().is_truthy());
}

/// Test value methods
#[test]
fn test_value_methods() {
    // Test as_f64 - integers can't be converted to float
    let int_val = Value::from_i64(42);
    assert!(int_val.as_f64().is_err());

    let float_val = Value::from_f64(3.14);
    assert!((float_val.as_f64().unwrap() - 3.14).abs() < 0.001);

    let bool_val = Value::from_bool(true);
    assert!(bool_val.as_f64().is_err());

    // Test as_bool - skipped as method may not be public

    // Test type_name
    assert_eq!(Value::from_i64(42).type_name(), "integer");
    assert_eq!(Value::from_f64(3.14).type_name(), "float");
    assert_eq!(Value::from_bool(true).type_name(), "boolean");
    assert_eq!(Value::nil().type_name(), "nil");
    assert_eq!(
        Value::from_string("hello".to_string()).type_name(),
        "string"
    );
    assert_eq!(Value::from_array(vec![]).type_name(), "array");
    assert_eq!(Value::Tuple(std::rc::Rc::new(vec![])).type_name(), "tuple");
}

/// Test range expressions
#[test]
fn test_range_expressions() {
    let mut interpreter = Interpreter::new();

    // Exclusive range
    let result = eval(&mut interpreter, "1..5").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_i64().unwrap(), 1);
        assert_eq!(arr[3].as_i64().unwrap(), 4);
    } else {
        panic!("Expected array");
    }

    // Inclusive range
    let result = eval(&mut interpreter, "1..=5").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_i64().unwrap(), 1);
        assert_eq!(arr[4].as_i64().unwrap(), 5);
    } else {
        panic!("Expected array");
    }

    // Empty range
    let result = eval(&mut interpreter, "5..5").unwrap();
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 0);
    } else {
        panic!("Expected array");
    }
}

/// Test method calls
#[test]
fn test_method_calls() {
    let mut interpreter = Interpreter::new();

    // String methods
    let result = eval(&mut interpreter, "\"hello\".len()").unwrap();
    assert_eq!(result.as_i64().unwrap(), 5);

    let result = eval(&mut interpreter, "\"hello\".to_upper()").unwrap();
    if let Value::String(s) = result {
        assert_eq!(s.as_ref(), "HELLO");
    } else {
        panic!("Expected string");
    }

    let result = eval(&mut interpreter, "\"WORLD\".to_lower()").unwrap();
    if let Value::String(s) = result {
        assert_eq!(s.as_ref(), "world");
    } else {
        panic!("Expected string");
    }

    let result = eval(&mut interpreter, "\"  trim  \".trim()").unwrap();
    if let Value::String(s) = result {
        assert_eq!(s.as_ref(), "trim");
    } else {
        panic!("Expected string");
    }
}

/// Test environment operations
#[test]
fn test_environment_operations() {
    let mut interpreter = Interpreter::new();

    // Define and lookup variables
    eval(&mut interpreter, "let x = 42").unwrap();
    assert_eq!(eval(&mut interpreter, "x").unwrap().as_i64().unwrap(), 42);

    // Update variables
    eval(&mut interpreter, "x = 100").unwrap();
    assert_eq!(eval(&mut interpreter, "x").unwrap().as_i64().unwrap(), 100);

    // Undefined variable
    assert!(eval(&mut interpreter, "undefined").is_err());
}

/// Test error conditions
#[test]
fn test_error_conditions() {
    let mut interpreter = Interpreter::new();

    // Type errors in binary operations
    assert!(eval(&mut interpreter, "5 + \"hello\"").is_err());
    assert!(eval(&mut interpreter, "true * 5").is_err());

    // Division by zero
    assert!(eval(&mut interpreter, "5 / 0").is_err());
    assert!(eval(&mut interpreter, "5 % 0").is_err());
}

/// Test closure capture
#[test]
fn test_closure_capture() {
    let mut interpreter = Interpreter::new();

    // Define outer variable
    eval(&mut interpreter, "let outer = 10").unwrap();

    // Create closure that captures outer
    eval(&mut interpreter, "let closure = |x| x + outer").unwrap();

    // Call closure
    let result = eval(&mut interpreter, "closure(5)").unwrap();
    assert_eq!(result.as_i64().unwrap(), 15);

    // Modify outer (shouldn't affect closure if it captures by value)
    eval(&mut interpreter, "outer = 20").unwrap();

    // Call closure again - should still use captured value
    // Note: This behavior depends on implementation
    let result = eval(&mut interpreter, "closure(5)").unwrap();
    assert_eq!(result.as_i64().unwrap(), 15); // or 25 if capture by reference
}

/// Test match expressions with patterns
#[test]
fn test_match_patterns() {
    let mut interpreter = Interpreter::new();

    // Match with wildcard
    let result = eval(
        &mut interpreter,
        "
        match 999 {
            1 => \"one\",
            2 => \"two\",
            _ => \"other\"
        }
    ",
    )
    .unwrap();
    if let Value::String(s) = result {
        assert_eq!(s.as_ref(), "other");
    } else {
        panic!("Expected string");
    }

    // Match with literal
    let result = eval(
        &mut interpreter,
        "
        match 5 {
            5 => 10,
            _ => 0
        }
    ",
    )
    .unwrap();
    assert_eq!(result.as_i64().unwrap(), 10);
}

/// Test complex nested expressions
#[test]
fn test_nested_expressions() {
    let mut interpreter = Interpreter::new();

    // Nested arithmetic
    let result = eval(&mut interpreter, "((1 + 2) * (3 + 4)) - ((5 - 2) * 2)").unwrap();
    assert_eq!(result.as_i64().unwrap(), 15); // (3 * 7) - (3 * 2) = 21 - 6 = 15

    // Nested if expressions
    let result = eval(
        &mut interpreter,
        "
        if true {
            if false {
                1
            } else {
                if true {
                    2
                } else {
                    3
                }
            }
        } else {
            4
        }
    ",
    )
    .unwrap();
    assert_eq!(result.as_i64().unwrap(), 2);

    // Nested function calls
    eval(&mut interpreter, "fun add(x, y) { x + y }").unwrap();
    eval(&mut interpreter, "fun multiply(x, y) { x * y }").unwrap();
    let result = eval(&mut interpreter, "multiply(add(2, 3), add(4, 5))").unwrap();
    assert_eq!(result.as_i64().unwrap(), 45); // 5 * 9 = 45
}
