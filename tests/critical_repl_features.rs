//! Critical REPL Feature Tests
//! These tests MUST pass or the product is broken

use ruchy::runtime::repl::Repl;
use ruchy::runtime::Value;
use std::time::{Duration, Instant};

#[test]
fn test_one_liner_execution() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    let result = repl.evaluate_expr_str("2 + 2", deadline).unwrap();
    assert_eq!(result, Value::Int(4));
}

#[test]
fn test_function_definition_and_call() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    // Define function
    repl.evaluate_expr_str("fun add(a: i32, b: i32) -> i32 { a + b }", deadline).unwrap();
    
    // Call function
    let result = repl.evaluate_expr_str("add(5, 3)", deadline).unwrap();
    assert_eq!(result, Value::Int(8));
}

#[test]
fn test_match_expressions() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    let result = repl.evaluate_expr_str(
        r#"match 2 { 1 => "one", 2 => "two", _ => "other" }"#,
        deadline
    ).unwrap();
    assert_eq!(result, Value::String("two".to_string()));
}

#[test]
fn test_block_returns_last_value() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    let result = repl.evaluate_expr_str(
        "{ let a = 5; let b = 10; a + b }",
        deadline
    ).unwrap();
    assert_eq!(result, Value::Int(15));
}

#[test]
fn test_for_loops() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    // For loop should return unit
    let result = repl.evaluate_expr_str(
        "for i in [1, 2, 3] { println(i) }",
        deadline
    ).unwrap();
    assert_eq!(result, Value::Unit);
}

#[test]
fn test_while_loops() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    repl.evaluate_expr_str("let mut x = 0", deadline).unwrap();
    repl.evaluate_expr_str("while x < 3 { x = x + 1 }", deadline).unwrap();
    let result = repl.evaluate_expr_str("x", deadline).unwrap();
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_string_interpolation() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    repl.evaluate_expr_str(r#"let name = "World""#, deadline).unwrap();
    let result = repl.evaluate_expr_str(r#"f"Hello {name}""#, deadline).unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_list_display() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    let result = repl.evaluate_expr_str("[1, 2, 3]", deadline).unwrap();
    assert_eq!(
        result,
        Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
    );
}

#[test]
fn test_variable_persistence() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    repl.evaluate_expr_str("let x = 42", deadline).unwrap();
    repl.evaluate_expr_str("let y = 58", deadline).unwrap();
    let result = repl.evaluate_expr_str("x + y", deadline).unwrap();
    assert_eq!(result, Value::Int(100));
}

#[test]
fn test_nested_functions() {
    let mut repl = Repl::new().unwrap();
    let deadline = Some(Instant::now() + Duration::from_secs(1));
    
    repl.evaluate_expr_str("fun double(x: i32) -> i32 { x * 2 }", deadline).unwrap();
    repl.evaluate_expr_str("fun quadruple(x: i32) -> i32 { double(double(x)) }", deadline).unwrap();
    let result = repl.evaluate_expr_str("quadruple(5)", deadline).unwrap();
    assert_eq!(result, Value::Int(20));
}