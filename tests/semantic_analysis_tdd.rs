//! Comprehensive TDD test suite for semantic_analysis.rs
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every semantic analysis path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::middleend::semantic_analysis::{SemanticAnalyzer, SemanticError};
use ruchy::{Parser, frontend::Expr};
use std::collections::HashMap;

// ==================== SEMANTIC ANALYZER CREATION TESTS ====================

#[test]
fn test_semantic_analyzer_new() {
    let analyzer = SemanticAnalyzer::new();
    // Should create with empty symbol table
    assert!(analyzer.is_valid());
}

// ==================== VARIABLE DECLARATION TESTS ====================

#[test]
fn test_analyze_let_binding() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 42");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_variable("x"));
}

#[test]
fn test_analyze_let_binding_duplicate() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 1; let x = 2");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    // Should allow shadowing or error on duplicate
    assert!(result.is_ok() || matches!(result, Err(SemanticError::DuplicateVariable(_))));
}

#[test]
fn test_analyze_const_binding() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("const PI = 3.14");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_constant("PI"));
}

#[test]
fn test_analyze_const_reassignment() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("const x = 1; x = 2");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::ConstReassignment(_))));
}

// ==================== VARIABLE USAGE TESTS ====================

#[test]
fn test_analyze_undefined_variable() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("x + 1");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::UndefinedVariable(_))));
}

#[test]
fn test_analyze_variable_in_scope() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 1; x + 2");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== FUNCTION DECLARATION TESTS ====================

#[test]
fn test_analyze_function_declaration() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_function("add"));
}

#[test]
fn test_analyze_function_duplicate() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun f() { }; fun f() { }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::DuplicateFunction(_))));
}

#[test]
fn test_analyze_function_params() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test(x: i32, y: i32) { x + y }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_function_duplicate_params() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test(x: i32, x: i32) { x }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::DuplicateParameter(_))));
}

// ==================== FUNCTION CALL TESTS ====================

#[test]
fn test_analyze_function_call() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }; add(1, 2)");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_undefined_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("undefined_func()");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::UndefinedFunction(_))));
}

#[test]
fn test_analyze_function_arity_mismatch() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }; add(1)");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::ArityMismatch { .. })));
}

// ==================== SCOPE TESTS ====================

#[test]
fn test_analyze_block_scope() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("{ let x = 1; x + 1 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_block_scope_isolation() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("{ let x = 1; }; x");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::UndefinedVariable(_))));
}

#[test]
fn test_analyze_nested_scope() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 1; { let y = 2; { let z = 3; x + y + z } }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_function_scope() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test(x: i32) { let y = 1; x + y }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== CONTROL FLOW TESTS ====================

#[test]
fn test_analyze_if_statement() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 1; if x > 0 { x + 1 } else { x - 1 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_while_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = 0; while x < 10 { x = x + 1 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_for_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("for i in 0..10 { println(i) }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== RETURN STATEMENT TESTS ====================

#[test]
fn test_analyze_return_in_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test() -> i32 { return 42 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_return_outside_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("return 42");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::ReturnOutsideFunction)));
}

#[test]
fn test_analyze_multiple_returns() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test(x: i32) -> i32 { if x > 0 { return 1 } else { return -1 } }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== BREAK/CONTINUE TESTS ====================

#[test]
fn test_analyze_break_in_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("while true { break }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_break_outside_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("break");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::BreakOutsideLoop)));
}

#[test]
fn test_analyze_continue_in_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("while true { continue }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_continue_outside_loop() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("continue");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::ContinueOutsideLoop)));
}

// ==================== TYPE DECLARATION TESTS ====================

#[test]
fn test_analyze_struct_declaration() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("struct Point { x: i32, y: i32 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_type("Point"));
}

#[test]
fn test_analyze_enum_declaration() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("enum Option<T> { Some(T), None }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_type("Option"));
}

#[test]
fn test_analyze_type_alias() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("type Result<T> = std::result::Result<T, String>");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_type("Result"));
}

// ==================== PATTERN MATCHING TESTS ====================

#[test]
fn test_analyze_match_expression() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let x = Some(1); match x { Some(n) => n, None => 0 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_exhaustive_match() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("enum Color { Red, Green, Blue }; let c = Red; match c { Red => 1, Green => 2 }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::NonExhaustiveMatch(_))));
}

// ==================== IMPORT/MODULE TESTS ====================

#[test]
fn test_analyze_import() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("import std::io");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_use_statement() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("use std::collections::HashMap");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== LAMBDA/CLOSURE TESTS ====================

#[test]
fn test_analyze_lambda() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let f = |x| x + 1");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_closure_capture() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("let y = 10; let f = |x| x + y");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== ASYNC/AWAIT TESTS ====================

#[test]
fn test_analyze_async_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("async fun fetch() -> String { \"data\" }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_await_expression() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("async fun test() { let data = await fetch() }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_await_outside_async() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("fun test() { await fetch() }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    assert!(matches!(result, Err(SemanticError::AwaitOutsideAsync)));
}

// ==================== TRAIT TESTS ====================

#[test]
fn test_analyze_trait_declaration() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("trait Display { fun fmt(&self) -> String }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
    assert!(analyzer.has_trait("Display"));
}

#[test]
fn test_analyze_impl_block() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("struct Point { x: i32 }; impl Point { fun new(x: i32) -> Point { Point { x } } }");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_ok());
}

// ==================== ERROR RECOVERY TESTS ====================

#[test]
fn test_analyze_with_multiple_errors() {
    let mut analyzer = SemanticAnalyzer::new();
    let mut parser = Parser::new("x + y; undefined_func(); break");
    let ast = parser.parse().unwrap();
    
    let result = analyzer.analyze(&ast);
    assert!(result.is_err());
    // Should report first error but attempt to find all
}

// Run all tests with: cargo test semantic_analysis_tdd --test semantic_analysis_tdd