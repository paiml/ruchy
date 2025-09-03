//! Comprehensive TDD test suite for codegen_minimal.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every code generation path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::{Parser, Transpiler};

// ==================== BASIC CODEGEN TESTS ====================

#[test]
fn test_codegen_integer_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("42"));
}

#[test]
fn test_codegen_float_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("3.14");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("3.14"));
}

#[test]
fn test_codegen_string_literal() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new(r#""hello""#);
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains(r#""hello""#));
}

#[test]
fn test_codegen_identifier() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("variable_name");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("variable_name"));
}

// ==================== OPERATOR CODEGEN TESTS ====================

#[test]
fn test_codegen_binary_add() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("1 + 2");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("1") && code.contains("+") && code.contains("2"));
}

#[test]
fn test_codegen_binary_subtract() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("5 - 3");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("5") && code.contains("-") && code.contains("3"));
}

#[test]
fn test_codegen_unary_negate() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("-42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("-") && code.contains("42"));
}

#[test]
fn test_codegen_unary_not() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("!true");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("!") && code.contains("true"));
}

// ==================== VARIABLE CODEGEN TESTS ====================

#[test]
fn test_codegen_let_binding() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("let x = 42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("let") && code.contains("x") && code.contains("42"));
}

#[test]
fn test_codegen_let_with_type() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("let x: i32 = 42");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("let") && code.contains("42"));
}

// ==================== FUNCTION CODEGEN TESTS ====================

#[test]
fn test_codegen_function() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("fun add(x, y) { x + y }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("fn add"));
}

#[test]
fn test_codegen_lambda() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("|x| x + 1");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("|"));
}

#[test]
fn test_codegen_function_call() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("add(1, 2)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("add") && code.contains("(") && code.contains(")"));
}

// ==================== CONTROL FLOW CODEGEN TESTS ====================

#[test]
fn test_codegen_if_else() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("if x > 0 { 1 } else { 2 }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("if") && code.contains("else"));
}

#[test]
fn test_codegen_match() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("match x { 1 => true, _ => false }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("match") && code.contains("=>"));
}

#[test]
fn test_codegen_for_loop() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("for x in [1, 2, 3] { print(x) }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("for") && code.contains("in"));
}

#[test]
fn test_codegen_while_loop() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("while x > 0 { x = x - 1 }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("while"));
}

// ==================== BLOCK CODEGEN TESTS ====================

#[test]
fn test_codegen_block() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("{ let x = 1; x + 2 }");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("{") && code.contains("}"));
}

// ==================== COLLECTION CODEGEN TESTS ====================

#[test]
fn test_codegen_list() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("[1, 2, 3]");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("vec!"));
}

#[test]
fn test_codegen_tuple() {
    let transpiler = Transpiler::new();
    let mut parser = Parser::new("(1, 2, 3)");
    let ast = parser.parse().unwrap();
    
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok());
    let code = result.unwrap().to_string();
    assert!(code.contains("(") && code.contains(",") && code.contains(")"));
}

// Run all tests with: cargo test codegen_minimal_tdd --test codegen_minimal_tdd