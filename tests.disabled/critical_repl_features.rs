#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Critical REPL Feature Tests
//! These tests MUST pass or the product is broken

#![allow(clippy::unwrap_used)]

use ruchy::runtime::repl::Repl;
use ruchy::runtime::Value;
use std::env;
use std::rc::Rc;

#[test]
fn test_one_liner_execution() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.evaluate_expr_str("2 + 2", None).unwrap();
    assert_eq!(result, Value::Integer(4));
}

#[test]
fn test_function_definition_and_call() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    // Define function
    repl.evaluate_expr_str("fun add(a: i32, b: i32) -> i32 { a + b }", None)
        .unwrap();

    // Call function
    let result = repl.evaluate_expr_str("add(5, 3)", None).unwrap();
    assert_eq!(result, Value::Integer(8));
}

#[test]
fn test_match_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl
        .evaluate_expr_str(r#"match 2 { 1 => "one", 2 => "two", _ => "other" }"#, None)
        .unwrap();
    assert_eq!(result, Value::String(Rc::new("two".to_string())));
}

#[test]
fn test_block_returns_last_value() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl
        .evaluate_expr_str("{ let a = 5; let b = 10; a + b }", None)
        .unwrap();
    assert_eq!(result, Value::Integer(15));
}

#[test]
fn test_for_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    // For loop should return unit
    let result = repl
        .evaluate_expr_str("for i in [1, 2, 3] { println(i) }", None)
        .unwrap();
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_while_loops() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.evaluate_expr_str("let mut x = 0", None).unwrap();
    repl.evaluate_expr_str("while x < 3 { x = x + 1 }", None)
        .unwrap();
    let result = repl.evaluate_expr_str("x", None).unwrap();
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_string_interpolation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.evaluate_expr_str(r#"let name = "World""#, None)
        .unwrap();
    let result = repl.evaluate_expr_str(r#"f"Hello {name}""#, None).unwrap();
    assert_eq!(result, Value::String(Rc::new("Hello World".to_string())));
}

#[test]
fn test_list_display() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let result = repl.evaluate_expr_str("[1, 2, 3]", None).unwrap();
    assert_eq!(
        result,
        Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into())
    );
}

#[test]
fn test_variable_persistence() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.evaluate_expr_str("let x = 42", None).unwrap();
    repl.evaluate_expr_str("let y = 58", None).unwrap();
    let result = repl.evaluate_expr_str("x + y", None).unwrap();
    assert_eq!(result, Value::Integer(100));
}

#[test]
fn test_nested_functions() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    repl.evaluate_expr_str("fun double(x: i32) -> i32 { x * 2 }", None)
        .unwrap();
    repl.evaluate_expr_str("fun quadruple(x: i32) -> i32 { double(double(x)) }", None)
        .unwrap();
    let result = repl.evaluate_expr_str("quadruple(5)", None).unwrap();
    assert_eq!(result, Value::Integer(20));
}
