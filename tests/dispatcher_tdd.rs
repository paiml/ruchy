//! Comprehensive TDD test suite for dispatcher.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every dispatch path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Transpiler, Parser};

// ==================== BASIC EXPRESSION TESTS ====================

#[test]
fn test_transpile_basic_literal_integer() {
    let transpiler = Transpiler::new();
    let code = "42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("42"));
}

#[test]
fn test_transpile_basic_literal_float() {
    let transpiler = Transpiler::new();
    let code = "3.14";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("3.14"));
}

#[test]
fn test_transpile_basic_literal_string() {
    let transpiler = Transpiler::new();
    let code = r#""hello""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(r#""hello""#));
}

#[test]
fn test_transpile_basic_identifier() {
    let transpiler = Transpiler::new();
    let code = "variable_name";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("variable_name"));
}

#[test]
fn test_transpile_reserved_keyword_as_identifier() {
    let transpiler = Transpiler::new();
    let code = "let type = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    // Should use raw identifier for reserved keyword
    assert!(transpiled.contains("r#type"));
}

#[test]
fn test_transpile_qualified_name() {
    let transpiler = Transpiler::new();
    let code = "std::collections::HashMap";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("std") && transpiled.contains("collections") && transpiled.contains("HashMap"));
}

#[test]
fn test_transpile_string_interpolation() {
    let transpiler = Transpiler::new();
    let code = r#"f"Hello {name}""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("format!"));
}

// ==================== OPERATOR EXPRESSION TESTS ====================

#[test]
fn test_transpile_binary_addition() {
    let transpiler = Transpiler::new();
    let code = "1 + 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("+"));
}

#[test]
fn test_transpile_binary_multiplication() {
    let transpiler = Transpiler::new();
    let code = "3 * 4";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("*"));
}

#[test]
fn test_transpile_unary_negation() {
    let transpiler = Transpiler::new();
    let code = "-42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("-") && transpiled.contains("42"));
}

#[test]
fn test_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let code = "!true";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("!"));
}

#[test]
fn test_transpile_assignment() {
    let transpiler = Transpiler::new();
    let code = "x = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("="));
}

#[test]
fn test_transpile_compound_assignment() {
    let transpiler = Transpiler::new();
    let code = "x += 10";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("+="));
}

// ==================== CONTROL FLOW TESTS ====================

#[test]
fn test_transpile_if_expression() {
    let transpiler = Transpiler::new();
    let code = "if x > 0 { 1 } else { 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("if"));
    assert!(transpiled.contains("else"));
}

#[test]
fn test_transpile_match_expression() {
    let transpiler = Transpiler::new();
    let code = "match x { 1 => true, _ => false }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("match"));
}

#[test]
fn test_transpile_for_loop() {
    let transpiler = Transpiler::new();
    let code = "for x in [1, 2, 3] { println(x) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("for"));
    assert!(transpiled.contains("in"));
}

#[test]
fn test_transpile_while_loop() {
    let transpiler = Transpiler::new();
    let code = "while x > 0 { x = x - 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("while"));
}

#[test]
fn test_transpile_loop() {
    let transpiler = Transpiler::new();
    let code = "loop { break }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("loop"));
    assert!(transpiled.contains("break"));
}

// ==================== FUNCTION EXPRESSION TESTS ====================

#[test]
fn test_transpile_function_declaration() {
    let transpiler = Transpiler::new();
    let code = "fun add(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("fn add"));
}

#[test]
fn test_transpile_async_function() {
    let transpiler = Transpiler::new();
    let code = "async fun fetch() { }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("async fn"));
}

#[test]
fn test_transpile_lambda_expression() {
    let transpiler = Transpiler::new();
    let code = "|x| x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("|"));
}

#[test]
fn test_transpile_function_call() {
    let transpiler = Transpiler::new();
    let code = "add(1, 2)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("add"));
    assert!(transpiled.contains("(") && transpiled.contains(")"));
}

#[test]
fn test_transpile_method_call() {
    let transpiler = Transpiler::new();
    let code = "str.len()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(".len()"));
}

// ==================== MACRO TESTS ====================

#[test]
fn test_transpile_println_macro() {
    let transpiler = Transpiler::new();
    let code = r#"println("Hello")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("println!"));
}

#[test]
fn test_transpile_vec_macro() {
    let transpiler = Transpiler::new();
    let code = "vec![1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("vec!"));
}

#[test]
fn test_transpile_assert_macro() {
    let transpiler = Transpiler::new();
    let code = "assert(x > 0)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("assert!"));
}

#[test]
fn test_transpile_panic_macro() {
    let transpiler = Transpiler::new();
    let code = r#"panic("Error!")"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("panic!"));
}

// ==================== STRUCT EXPRESSION TESTS ====================

#[test]
fn test_transpile_struct_definition() {
    let transpiler = Transpiler::new();
    let code = "struct Point { x: i32, y: i32 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("struct Point"));
}

#[test]
fn test_transpile_struct_literal() {
    let transpiler = Transpiler::new();
    let code = "Point { x: 10, y: 20 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("Point"));
    assert!(transpiled.contains("x"));
    assert!(transpiled.contains("y"));
}

#[test]
fn test_transpile_field_access() {
    let transpiler = Transpiler::new();
    let code = "point.x";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(".x"));
}

#[test]
fn test_transpile_index_access() {
    let transpiler = Transpiler::new();
    let code = "arr[0]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("[") && transpiled.contains("]"));
}

// ==================== COLLECTION TESTS ====================

#[test]
fn test_transpile_list_literal() {
    let transpiler = Transpiler::new();
    let code = "[1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("vec!"));
}

#[test]
fn test_transpile_tuple_literal() {
    let transpiler = Transpiler::new();
    let code = "(1, 2, 3)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("(") && transpiled.contains(",") && transpiled.contains(")"));
}

#[test]
fn test_transpile_range() {
    let transpiler = Transpiler::new();
    let code = "1..10";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(".."));
}

// ==================== ASYNC/AWAIT TESTS ====================

#[test]
fn test_transpile_await_expression() {
    let transpiler = Transpiler::new();
    let code = "await fetch()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains(".await"));
}

#[test]
fn test_transpile_async_block() {
    let transpiler = Transpiler::new();
    let code = "async { fetch().await }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let transpiled = result.unwrap().to_string();
    
    assert!(transpiled.contains("async"));
}

// Run all tests with: cargo test dispatcher_tdd --test dispatcher_tdd