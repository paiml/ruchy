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

// ============================================================================
// CATEGORY 1: Loop Control Tests (High Priority)
// ============================================================================

#[test]
fn test_while_break_with_value() {
    let v = eval_code("while true { break 42 }").unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_while_continue() {
    let code = "let mut x = 0; let mut c = 0; while x < 5 { x = x + 1; if x == 3 { continue }; c = c + 1 }; c";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(4)); // Skips increment when x==3
}

#[test]
fn test_loop_empty_body() {
    let v = eval_code("let mut x = 0; while x < 3 { x = x + 1 }; x").unwrap();
    assert_eq!(v, Value::Integer(3));
}

#[test]
fn test_for_with_break() {
    let code = "let mut s = 0; for i in 0..10 { if i > 3 { break }; s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6)); // 0+1+2+3
}

#[test]
fn test_for_with_continue() {
    let code = "let mut s = 0; for i in 0..5 { if i == 2 { continue }; s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(8)); // 0+1+3+4
}

#[test]
fn test_nested_loops_break() {
    let code = r#"
        let mut found = false
        for i in 0..5 {
            for j in 0..5 {
                if i == 2 && j == 3 {
                    found = true
                    break
                }
            }
        }
        found
    "#;
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Bool(true));
}

#[test]
fn test_while_false_never_executes() {
    let code = "let mut x = 0; while false { x = x + 1 }; x";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(0));
}

// ============================================================================
// CATEGORY 2: Pattern Matching Tests (High Priority)
// ============================================================================

#[test]
fn test_match_with_guard_true() {
    let code = "match 10 { x if x > 5 => 100, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(100));
}

#[test]
fn test_match_with_guard_false() {
    let code = "match 3 { x if x > 5 => 100, _ => 99 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(99));
}

#[test]
fn test_match_identifier_binding() {
    let code = "match 42 { x => x + 1 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(43));
}

#[test]
fn test_match_tuple_destructure() {
    let code = "match (1, 2, 3) { (a, b, c) => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_array_destructure() {
    let code = "match [1, 2, 3] { [a, b, c] => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_nested_pattern() {
    let code = "match (1, (2, 3)) { (a, (b, c)) => a + b + c, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_match_wildcard_in_pattern() {
    let code = "match (1, 2, 3) { (1, _, 3) => 100, _ => 0 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(100));
}

#[test]
fn test_match_multiple_arms() {
    let code = r#"
        match 2 {
            0 => 10,
            1 => 20,
            2 => 30,
            3 => 40,
            _ => 50,
        }
    "#;
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(30));
}
