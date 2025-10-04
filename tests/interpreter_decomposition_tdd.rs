//! TDD tests for systematic interpreter decomposition
//!
//! GOAL: Move eval_expr_kind from 2659 lines to modularized approach
//! Target: < 1500 total lines in interpreter.rs

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};

#[test]
fn test_control_flow_still_works_after_decomposition() {
    let mut interpreter = Interpreter::new();

    // Check if expressions
    let mut parser = Parser::new("if true { 42 } else { 0 }");
    let ast = parser.parse().expect("Should parse if expression");
    let result = interpreter
        .eval_expr(&ast)
        .expect("Should evaluate if expression");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_data_structures_still_work_after_decomposition() {
    let mut interpreter = Interpreter::new();

    // Check arrays
    let mut parser = Parser::new("[1, 2, 3]");
    let ast = parser.parse().expect("Should parse array");
    let result = interpreter.eval_expr(&ast).expect("Should evaluate array");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Integer(1));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_function_calls_still_work_after_decomposition() {
    let mut interpreter = Interpreter::new();

    // Check builtin function calls (these should work after our fix)
    let mut parser = Parser::new("len([1, 2, 3])");
    let ast = parser.parse().expect("Should parse len call");
    let result = interpreter
        .eval_expr(&ast)
        .expect("Should evaluate len call");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_binary_operations_still_work_after_decomposition() {
    let mut interpreter = Interpreter::new();

    // Check arithmetic
    let mut parser = Parser::new("2 + 3 * 4");
    let ast = parser.parse().expect("Should parse arithmetic");
    let result = interpreter
        .eval_expr(&ast)
        .expect("Should evaluate arithmetic");
    assert_eq!(result, Value::Integer(14));
}
