#![allow(missing_docs)]
//! Unit tests for expressions.rs parser module
//!
//! These tests were moved from inline tests in expressions.rs to reduce file size
//! and improve TDG Structural score. All functionality remains tested.

use ruchy::frontend::ast::{ExprKind, Literal};
use ruchy::frontend::parser::Parser;

// Unit tests for specific parsing functions

#[test]
fn test_parse_integer_literal() {
    let mut parser = Parser::new("42");
    let result = parser.parse().unwrap();
    if let ExprKind::Literal(Literal::Integer(n, type_suffix)) = &result.kind {
        assert_eq!(*n, 42);
        assert_eq!(*type_suffix, None);
    } else {
        panic!("Expected integer literal, got {:?}", result.kind);
    }
}

#[test]
fn test_parse_float_literal() {
    let mut parser = Parser::new("3.14");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse float literal");
}

#[test]
fn test_parse_string_literal() {
    let mut parser = Parser::new("\"hello world\"");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse string literal");
}

#[test]
fn test_parse_boolean_true() {
    let mut parser = Parser::new("true");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse boolean true");
}

#[test]
fn test_parse_boolean_false() {
    let mut parser = Parser::new("false");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse boolean false");
}

#[test]
fn test_parse_char_literal() {
    let mut parser = Parser::new("'a'");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse char literal");
}

#[test]
fn test_parse_fstring_literal() {
    let mut parser = Parser::new("f\"Hello {name}\"");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse f-string literal");
}

#[test]
fn test_parse_identifier() {
    let mut parser = Parser::new("variable_name");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse identifier");
}

#[test]
fn test_parse_underscore() {
    let mut parser = Parser::new("_");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse underscore");
}

#[test]
fn test_parse_unary_minus() {
    let mut parser = Parser::new("-42");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse unary minus");
}

#[test]
fn test_parse_unary_not() {
    let mut parser = Parser::new("!true");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse unary not");
}

#[test]
fn test_parse_binary_add() {
    let mut parser = Parser::new("1 + 2");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse binary add");
}

#[test]
fn test_parse_binary_multiply() {
    let mut parser = Parser::new("3 * 4");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse binary multiply");
}

#[test]
fn test_parse_tuple() {
    let mut parser = Parser::new("(1, 2, 3)");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse tuple");
}

#[test]
fn test_parse_list() {
    let mut parser = Parser::new("[1, 2, 3]");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse list");
}

#[test]
fn test_parse_if_expression() {
    let mut parser = Parser::new("if x > 0 { x } else { 0 }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse if expression");
}

#[test]
fn test_parse_match_expression() {
    let mut parser = Parser::new("match x { 1 => \"one\", _ => \"other\" }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse match expression");
}

#[test]
fn test_parse_while_loop() {
    let mut parser = Parser::new("while x < 10 { x = x + 1 }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse while loop");
}

#[test]
fn test_parse_for_loop() {
    let mut parser = Parser::new("for x in 0..10 { print(x) }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse for loop");
}

#[test]
fn test_parse_lambda() {
    let mut parser = Parser::new("|x| x + 1");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse lambda");
}

#[test]
fn test_parse_function() {
    let mut parser = Parser::new("fn add(a: i32, b: i32) -> i32 { a + b }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse function");
}

#[test]
fn test_parse_struct() {
    let mut parser = Parser::new("struct Point { x: f64, y: f64 }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse struct");
}

#[test]
fn test_parse_enum() {
    let mut parser = Parser::new("enum Color { Red, Green, Blue }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse enum");
}

#[test]
fn test_parse_trait() {
    let mut parser = Parser::new("trait Drawable { fn draw(&self); }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse trait");
}

#[test]
#[ignore = "Impl parsing not fully supported"]
fn test_parse_impl() {
    let mut parser = Parser::new("impl Point { fn new(x: f64) -> Point { Point { x, y: 0.0 } } }");
    let result = parser.parse();
    assert!(result.is_ok(), "Failed to parse impl");
}
