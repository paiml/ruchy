//! PRIORITY-3: Zero Coverage - eval_control_flow_new.rs
//! Integration tests exercising control flow through runtime

use ruchy::frontend::parser::Parser;
use ruchy::runtime::{Interpreter, Value};

fn eval_code(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let mut interpreter = Interpreter::new();
    Ok(interpreter.eval_expr(&ast)?)
}

#[test]
fn test_if_true() {
    let v = eval_code("if true { 42 } else { 0 }").unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_if_false() {
    let v = eval_code("if false { 42 } else { 99 }").unwrap();
    assert_eq!(v, Value::Integer(99));
}

#[test]
fn test_while_loop() {
    let v = eval_code("let mut x = 0; while x < 3 { x = x + 1 }; x").unwrap();
    assert_eq!(v, Value::Integer(3));
}

#[test]
fn test_for_range() {
    let v = eval_code("let mut s = 0; for i in 0..3 { s = s + i }; s").unwrap();
    assert_eq!(v, Value::Integer(3)); // 0+1+2
}

#[test]
fn test_match_literal() {
    let v = eval_code("match 2 { 1 => 10, 2 => 20, _ => 30 }").unwrap();
    assert_eq!(v, Value::Integer(20));
}

#[test]
fn test_break() {
    let v = eval_code("let mut x = 0; while true { if x > 2 { break }; x = x + 1 }; x").unwrap();
    assert_eq!(v, Value::Integer(3));
}

#[test]
fn test_block() {
    let v = eval_code("{ let x = 1; let y = 2; x + y }").unwrap();
    assert_eq!(v, Value::Integer(3));
}

#[test]
fn test_list() {
    let v = eval_code("[1, 2, 3]").unwrap();
    match v {
        Value::Array(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_tuple() {
    let v = eval_code("(1, 2, 3)").unwrap();
    match v {
        Value::Tuple(t) => assert_eq!(t.len(), 3),
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_return() {
    let v = eval_code("fn f() { return 42 }; f()").unwrap();
    assert_eq!(v, Value::Integer(42));
}
