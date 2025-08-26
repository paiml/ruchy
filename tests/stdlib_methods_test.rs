// BOOK-002: Standard Library Methods Tests
// Following Toyota Way TDD - RED-GREEN-REFACTOR phases

use ruchy::runtime::interpreter::Interpreter;
use ruchy::frontend::parser::Parser;

// Helper function to evaluate code through interpreter
fn eval_code(code: &str) -> String {
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");
    result.to_string()
}

#[test]
fn test_string_to_string() {
    assert_eq!(eval_code(r#"("hello").to_string()"#), "hello");
}

#[test]
fn test_integer_to_string() {
    assert_eq!(eval_code("42.to_string()"), "42");
}

#[test]
fn test_string_len() {
    assert_eq!(eval_code(r#""hello".len()"#), "5");
}

#[test]
fn test_string_trim() {
    assert_eq!(eval_code(r#""  hello  ".trim()"#), "hello");
}

#[test]
fn test_string_to_upper() {
    assert_eq!(eval_code(r#""hello".to_upper()"#), "HELLO");
}

#[test]
fn test_string_to_lower() {
    assert_eq!(eval_code(r#""HELLO".to_lower()"#), "hello");
}

#[test]
fn test_string_contains() {
    assert_eq!(eval_code(r#""hello world".contains("world")"#), "true");
}

#[test]
fn test_string_starts_with() {
    assert_eq!(eval_code(r#""hello world".starts_with("hello")"#), "true");
}

#[test]
fn test_string_ends_with() {
    assert_eq!(eval_code(r#""hello world".ends_with("world")"#), "true");
}

#[test]
fn test_string_replace() {
    assert_eq!(eval_code(r#""hello world".replace("world", "rust")"#), "hello rust");
}

#[test]
fn test_string_split() {
    assert_eq!(eval_code(r#""hello,world,rust".split(",")"#), "[hello, world, rust]");
}

#[test]
fn test_array_len() {
    assert_eq!(eval_code("[1, 2, 3].len()"), "3");
}

#[test]
fn test_array_push() {
    assert_eq!(eval_code("[1, 2].push(3)"), "[1, 2, 3]");
}

#[test]
fn test_array_pop() {
    assert_eq!(eval_code("[1, 2, 3].pop()"), "[1, 2]");
}

#[test]
fn test_array_get() {
    assert_eq!(eval_code("[10, 20, 30].get(1)"), "20");
}

#[test]
fn test_array_first() {
    assert_eq!(eval_code("[10, 20, 30].first()"), "10");
}

#[test]
fn test_array_last() {
    assert_eq!(eval_code("[10, 20, 30].last()"), "30");
}

#[test]
fn test_array_map() {
    assert_eq!(eval_code("[1, 2, 3].map(|x| x * 2)"), "[2, 4, 6]");
}

#[test]
fn test_array_filter() {
    assert_eq!(eval_code("[1, 2, 3, 4, 5].filter(|x| x > 2)"), "[3, 4, 5]");
}

#[test]
fn test_array_reduce() {
    assert_eq!(eval_code("[1, 2, 3, 4].reduce(|a, b| a + b, 0)"), "10");
}

#[test]
fn test_float_sqrt() {
    assert_eq!(eval_code("16.0.sqrt()"), "4");
}

#[test]
fn test_float_abs() {
    assert_eq!(eval_code("(-3.14).abs()"), "3.14");
}

#[test]
fn test_float_round() {
    assert_eq!(eval_code("3.7.round()"), "4");
}

#[test]
fn test_float_floor() {
    assert_eq!(eval_code("3.7.floor()"), "3");
}

#[test]
fn test_float_ceil() {
    assert_eq!(eval_code("3.2.ceil()"), "4");
}

// Format macro equivalent tests
#[test]
fn test_format_basic() {
    let code = r#"format("Hello {}, you are {} years old", "Alice", 30)"#;
    assert_eq!(eval_code(code), "Hello Alice, you are 30 years old");
}

#[test]
fn test_hashmap_new() {
    assert_eq!(eval_code("HashMap::new()"), "{}");
}

#[test]
fn test_hashmap_operations() {
    // Note: This test would need multiple statements
    // which requires a different test approach
}

// Higher-order function tests
#[test]
fn test_array_any() {
    assert_eq!(eval_code("[1, 2, 3].any(|x| x > 2)"), "true");
    assert_eq!(eval_code("[1, 2, 3].any(|x| x > 5)"), "false");
}

#[test]
fn test_array_all() {
    assert_eq!(eval_code("[2, 4, 6].all(|x| x % 2 == 0)"), "true");
    assert_eq!(eval_code("[1, 2, 3].all(|x| x > 2)"), "false");
}

#[test]
fn test_array_find() {
    assert_eq!(eval_code("[1, 2, 3, 4].find(|x| x > 2)"), "3");
}

// Chained method tests
#[test]
fn test_method_chaining() {
    assert_eq!(eval_code(r#""  HELLO  ".trim().to_lower()"#), "hello");
}

#[test]
fn test_integer_abs() {
    assert_eq!(eval_code("(-42).abs()"), "42");
}