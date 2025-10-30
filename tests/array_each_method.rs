// STDLIB-010: Array.each() method implementation
// Missing feature discovered during STDLIB-005 example validation
// Ticket: STDLIB-010
//
// Extreme TDD: RED → GREEN → REFACTOR
// This test file demonstrates the .each() method
//
// NOTE: Ruchy closures don't support mutable capture of external variables,
// so .each() is primarily useful for I/O side effects (println) or
// when/if Ruchy gains mutable closure capture in the future.

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

#[test]
fn test_stdlib010_each_basic_iteration() {
    // Test that .each() iterates without errors and returns Nil
    let code = r"
        let items = [1, 2, 3]
        items.each(fn(x) { x * 2 })
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    // .each() should return Nil
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_empty_array() {
    // Test that .each() handles empty arrays without errors
    let code = r"
        let items = []
        items.each(fn(x) { x * 2 })
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_returns_nil() {
    // Explicitly test that .each() returns Nil (not the array)
    let code = r"
        let items = [1, 2, 3]
        let result = items.each(fn(x) { x })
        result
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_with_strings() {
    // Test that .each() works with string arrays
    let code = r#"
        let items = ["a", "b", "c"]
        items.each(fn(x) { x })
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_with_objects() {
    // Test that .each() works with object arrays
    let code = r#"
        let items = [
            { name: "Alice", age: 30 },
            { name: "Bob", age: 25 }
        ]
        items.each(fn(person) { person.name })
    "#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_nested() {
    // Test that .each() can be nested
    let code = r"
        let matrix = [[1, 2], [3, 4]]
        matrix.each(fn(row) {
            row.each(fn(val) { val * 2 })
        })
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_after_filter() {
    // Test that .each() can be chained after .filter()
    let code = r"
        let items = [1, 2, 3, 4, 5]
        items.filter(fn(x) { x % 2 == 0 }).each(fn(x) { x * 10 })
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}

#[test]
fn test_stdlib010_each_after_map() {
    // Test that .each() can be chained after .map()
    let code = r"
        let items = [1, 2, 3]
        items.map(fn(x) { x * 2 }).each(fn(x) { x + 1 })
    ";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).unwrap();

    assert_eq!(result, Value::Nil);
}
