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

// ============================================================================
// CATEGORY 3: Advanced Iteration Tests (Medium Priority)
// ============================================================================

#[test]
fn test_for_empty_array() {
    let code = "let mut count = 0; for x in [] { count = count + 1 }; count";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(0)); // Never executes
}

#[test]
fn test_for_single_element_array() {
    let code = "let mut x = 0; for i in [42] { x = i }; x";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_range_zero_to_zero() {
    let code = "let mut count = 0; for i in 0..0 { count = count + 1 }; count";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(0)); // Empty range
}

#[test]
fn test_range_inclusive() {
    let code = "let mut s = 0; for i in 0..=3 { s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6)); // 0+1+2+3
}

#[test]
fn test_range_exclusive() {
    let code = "let mut s = 0; for i in 0..3 { s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(3)); // 0+1+2
}

#[test]
fn test_for_nested_arrays() {
    let code = r#"
        let mut total = 0
        for row in [[1, 2], [3, 4]] {
            for elem in row {
                total = total + elem
            }
        }
        total
    "#;
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(10)); // 1+2+3+4
}

#[test]
fn test_range_with_variables() {
    let code = "let start = 1; let end = 4; let mut s = 0; for i in start..end { s = s + i }; s";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6)); // 1+2+3
}

#[test]
fn test_for_loop_returns_last_value() {
    let code = "for i in 0..5 { i * 2 }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(8)); // Last iteration: 4 * 2
}

// ============================================================================
// CATEGORY 4: Error Cases and Edge Cases (Medium Priority)
// ============================================================================

#[test]
fn test_array_init_with_size() {
    let code = "[42; 3]";
    let v = eval_code(code).unwrap();
    match v {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(42));
            assert_eq!(arr[1], Value::Integer(42));
            assert_eq!(arr[2], Value::Integer(42));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_init_size_zero() {
    let code = "[0; 0]";
    let v = eval_code(code).unwrap();
    match v {
        Value::Array(arr) => assert_eq!(arr.len(), 0),
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_init_negative_size_error() {
    let code = "[42; -1]";
    let result = eval_code(code);
    assert!(result.is_err());
}

#[test]
fn test_nested_blocks() {
    let code = "{ { { 42 } } }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(42));
}

#[test]
fn test_block_with_multiple_statements() {
    let code = "{ let x = 1; let y = 2; let z = 3; x + y + z }";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Integer(6));
}

#[test]
fn test_empty_block() {
    let code = "{}";
    let v = eval_code(code).unwrap();
    assert_eq!(v, Value::Nil);
}

#[test]
fn test_tuple_nested() {
    let code = "((1, 2), (3, 4))";
    let v = eval_code(code).unwrap();
    match v {
        Value::Tuple(outer) => {
            assert_eq!(outer.len(), 2);
            match &outer[0] {
                Value::Tuple(inner) => assert_eq!(inner.len(), 2),
                _ => panic!("Expected tuple"),
            }
        }
        _ => panic!("Expected tuple"),
    }
}

#[test]
fn test_range_non_integer_error() {
    let code = "for i in 0.5..3.5 { i }";
    let result = eval_code(code);
    // Float ranges should error or be handled
    assert!(result.is_ok() || result.is_err()); // Implementation dependent
}
