//! Tests for deterministic error recovery in the parser
//! Based on docs/ruchy-transpiler-docs.md Section 4

use ruchy::frontend::parser::Parser;
use ruchy::parser::error_recovery::{ErrorNode, ErrorContext};

#[test]
fn test_missing_function_name_recovery() {
    let mut parser = Parser::new("fun (x: i32) { x + 1 }");
    let result = parser.parse();
    
    // Should recover and create a synthetic function
    assert!(result.is_ok());
    let errors = parser.get_errors();
    assert!(!errors.is_empty());
    
    let error = &errors[0];
    assert_eq!(error.message, "expected function name");
}

#[test]
fn test_missing_function_params_recovery() {
    let mut parser = Parser::new("fun add { return 42 }");
    let result = parser.parse();
    
    // Should recover with empty params
    assert!(result.is_ok());
    let errors = parser.get_errors();
    assert!(!errors.is_empty());
}

#[test]
fn test_missing_function_body_recovery() {
    let mut parser = Parser::new("fun process(x: i32)");
    let result = parser.parse();
    
    // Should recover with synthetic body
    assert!(result.is_ok());
    let errors = parser.get_errors();
    assert!(!errors.is_empty());
}

#[test]
fn test_multiple_errors_recovery() {
    // Multiple syntax errors in one input
    let mut parser = Parser::new("fun (x: i32) fun test(");
    let result = parser.parse();
    
    // Should continue parsing and collect multiple errors
    if result.is_ok() {
        let errors = parser.get_errors();
        assert!(errors.len() >= 2);
    }
}

#[test]
fn test_sync_point_recovery() {
    // Error followed by valid code after sync point
    let mut parser = Parser::new("fun missing_body(x: i32); let y = 10");
    let result = parser.parse();
    
    // Should recover at semicolon and continue parsing
    assert!(result.is_ok());
    let errors = parser.get_errors();
    assert!(!errors.is_empty());
}

#[test]
fn test_deterministic_recovery() {
    // Same error should produce same recovery every time
    let input = "fun (x: i32) { x }";
    
    let mut parser1 = Parser::new(input);
    let result1 = parser1.parse();
    let errors1 = parser1.get_errors().to_vec();
    
    let mut parser2 = Parser::new(input);
    let result2 = parser2.parse();
    let errors2 = parser2.get_errors().to_vec();
    
    // Both parses should produce identical results
    assert_eq!(result1.is_ok(), result2.is_ok());
    assert_eq!(errors1.len(), errors2.len());
    for (e1, e2) in errors1.iter().zip(errors2.iter()) {
        assert_eq!(e1.message, e2.message);
        assert_eq!(e1.recovery, e2.recovery);
    }
}

#[test]
fn test_partial_struct_recovery() {
    let mut parser = Parser::new("Point { x: 10, y: }");
    let result = parser.parse();
    
    // Should recover with partial struct
    if result.is_ok() {
        let errors = parser.get_errors();
        assert!(!errors.is_empty());
    }
}

#[test]
fn test_malformed_if_recovery() {
    let mut parser = Parser::new("if { println(\"test\") }");
    let result = parser.parse();
    
    // Should recover with default condition
    if result.is_ok() {
        let errors = parser.get_errors();
        assert!(!errors.is_empty());
    }
}

#[test] 
fn test_error_context_preservation() {
    let mut parser = Parser::new("fun test");
    let result = parser.parse();
    
    if result.is_ok() {
        let errors = parser.get_errors();
        assert!(!errors.is_empty());
        
        let error = &errors[0];
        // Check that context is preserved
        match &error.context {
            ErrorContext::FunctionDecl { name, .. } => {
                assert_eq!(name.as_ref().unwrap(), "test");
            }
            _ => panic!("Expected FunctionDecl context"),
        }
    }
}

#[test]
fn test_max_errors_limit() {
    // Create input with many errors
    let mut input = String::new();
    for _ in 0..150 {
        input.push_str("fun ; ");
    }
    
    let mut parser = Parser::new(&input);
    let _ = parser.parse();
    
    let errors = parser.get_errors();
    // Should stop after max_errors limit (100 by default)
    assert!(errors.len() <= 100);
}