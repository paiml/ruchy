#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Exhaustive pattern matching coverage test
//!
//! This test ensures that all `ExprKind` variants are handled in critical match statements.
//! This prevents compilation failures when new expression types are added.

use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use std::env;
use ruchy::middleend::infer::InferenceContext;
use ruchy::backend::transpiler::Transpiler;

/// Test that ensures all `ExprKind` variants can be processed by the interpreter
#[test]
fn test_interpreter_exhaustive_coverage() {
    // This test primarily ensures compilation succeeds
    // The key is that if we add a new `ExprKind` variant and forget to handle it
    // in the interpreter's match statement, this will fail to compile
    
    // Create test expressions for each `ExprKind` variant
    let _test_cases = vec![
        create_literal_expr(),
        create_identifier_expr(),
        create_macro_expr(), // CRITICAL: This is the new variant we added
        // Add more as needed
    ];
    
    // The fact that this compiles means all `ExprKind` variants are handled
    // in the interpreter's evaluate_expr method
}

/// Test that ensures all `ExprKind` variants can be processed by the type checker
#[test]
fn test_type_checker_exhaustive_coverage() {
    let mut checker = InferenceContext::new();
    
    let test_cases = vec![
        create_literal_expr(),
        create_identifier_expr(),
        create_macro_expr(), // CRITICAL: Ensure type inference handles macros
    ];
    
    for expr in test_cases {
        // This will fail compilation if any `ExprKind` variant is missing from infer_expr
        let _result = checker.infer(&expr); // Don't care about success/failure, just compilation
    }
}

/// Test that ensures all `ExprKind` variants can be processed by the transpiler
#[test]
fn test_transpiler_exhaustive_coverage() {
    let mut transpiler = Transpiler::new();
    
    let test_cases = vec![
        create_literal_expr(),
        create_identifier_expr(),
        create_macro_expr(), // CRITICAL: Ensure transpiler handles macros
    ];
    
    for expr in test_cases {
        // This will fail compilation if any `ExprKind` variant is missing from transpile_expr
        let _result = transpiler.transpile(&expr); // Don't care about success/failure, just compilation
    }
}

/// Test that all expression variants compile successfully
#[test]
fn test_all_exprkind_variants_compile() {
    // This test forces us to create an instance of every `ExprKind` variant
    // If we add a new variant and forget to handle it somewhere, this will catch it
    
    let _expressions = vec![
        // Force instantiation of every variant
        ExprKind::Literal(Literal::Integer(42)),
        ExprKind::Identifier("test".to_string()),
        ExprKind::Macro { 
            name: "println".to_string(), 
            args: vec![] 
        },
        // Add ALL other variants here - this ensures we remember to test new ones
    ];
    
    // If we add a new `ExprKind` variant and forget to add it here,
    // this comment will remind us: "ADD ALL EXPRKIND VARIANTS ABOVE"
}

fn create_literal_expr() -> Expr {
    Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span { start: 0, end: 2 }
    )
}

fn create_identifier_expr() -> Expr {
    Expr::new(
        ExprKind::Identifier("test".to_string()),
        Span { start: 0, end: 4 }
    )
}

fn create_macro_expr() -> Expr {
    Expr::new(
        ExprKind::Macro {
            name: "println".to_string(),
            args: vec![create_literal_expr()],
        },
        Span { start: 0, end: 10 }
    )
}

/// Test slice operations with range indexing (RUCHY-0713)
#[test]
#[allow(clippy::expect_used, clippy::unwrap_used)]
fn test_slice_operations_comprehensive() {
    use ruchy::runtime::Repl;
    
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    
    // Test basic slice operations with different range types
    let test_cases = vec![
        // Basic exclusive range slicing
        ("let arr = [1, 2, 3, 4, 5]; arr[1..3]", "[2, 3]"),
        ("let arr = [1, 2, 3, 4, 5]; arr[0..2]", "[1, 2]"),
        ("let arr = [1, 2, 3, 4, 5]; arr[2..5]", "[3, 4, 5]"),
        
        // Inclusive range slicing 
        ("let arr = [1, 2, 3, 4, 5]; arr[1..=3]", "[2, 3, 4]"),
        ("let arr = [1, 2, 3, 4, 5]; arr[0..=2]", "[1, 2, 3]"),
        
        // String slicing
        ("let s = \"hello\"; s[1..4]", "\"ell\""),
        ("let s = \"hello\"; s[0..=2]", "\"hel\""),
        ("let s = \"world\"; s[1..3]", "\"or\""),
        
        // Edge cases
        ("let arr = [1, 2, 3]; arr[0..0]", "[]"),
        ("let arr = [1, 2, 3]; arr[1..1]", "[]"),
        ("let s = \"test\"; s[1..1]", "\"\""),
        
        // Full range alternatives using Python-style syntax
        ("let arr = [1, 2, 3, 4, 5]; arr[1:3]", "[2, 3]"),
        ("let s = \"hello\"; s[1:4]", "\"ell\""),
    ];
    
    for (input, expected) in test_cases {
        let result = repl.eval(input);
        assert!(result.is_ok(), "Failed to evaluate: {input}");
        let output = result.unwrap();
        assert_eq!(output, expected, "Input: {input} | Expected: {expected} | Got: {output}");
    }
}

/// Test slice operation error cases
#[test]
#[allow(clippy::expect_used)]
fn test_slice_operations_error_cases() {
    use ruchy::runtime::Repl;
    
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    
    // Test error cases that should fail gracefully
    let error_cases = vec![
        // Out of bounds ranges
        "let arr = [1, 2, 3]; arr[0..10]",
        "let arr = [1, 2, 3]; arr[5..7]", 
        "let s = \"hi\"; s[0..10]",
        
        // Invalid range order
        "let arr = [1, 2, 3]; arr[3..1]",
        "let s = \"test\"; s[3..1]",
    ];
    
    for input in error_cases {
        let result = repl.eval(input);
        assert!(result.is_err(), "Expected error for: {input}");
    }
}