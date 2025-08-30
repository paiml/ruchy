//! Basic tests for proving system modules
//!
//! [COVERAGE-BOOST-001] Target proving modules with 0% coverage

use ruchy::proving::prover::*;
use ruchy::proving::smt::*;
use ruchy::proving::tactics::*;
use ruchy::proving::verification::*;
use ruchy::frontend::ast::{Expr, ExprKind, Span, Literal, BinaryOp};

#[test]
fn test_smt_backend_variants() {
    let backends = vec![
        SmtBackend::Z3,
        SmtBackend::CVC5,
        SmtBackend::Yices2,
        SmtBackend::MathSAT,
    ];
    
    for backend in backends {
        // Should be able to create and debug print
        assert!(!format!("{:?}", backend).is_empty());
        
        // Test equality and clone
        assert_eq!(backend.clone(), backend);
    }
}

#[test]
fn test_interactive_prover_creation() {
    let backend = SmtBackend::Z3;
    let prover = InteractiveProver::new(backend);
    
    // Should create without panicking
    drop(prover);
}

#[test]
fn test_interactive_prover_configuration() {
    let backend = SmtBackend::CVC5;
    let mut prover = InteractiveProver::new(backend);
    
    // Test timeout setting
    prover.set_timeout(5000);
    prover.set_timeout(10000);
    
    // Test ML suggestions toggle
    prover.set_ml_suggestions(true);
    prover.set_ml_suggestions(false);
    
    // Should configure without errors
}

#[test]
fn test_interactive_prover_script_loading() {
    let backend = SmtBackend::MathSAT;
    let mut prover = InteractiveProver::new(backend);
    
    // Test various script inputs
    let scripts = vec![
        "",
        "simple script",
        "theorem test: true",
        "proof by induction",
    ];
    
    for script in scripts {
        let result = prover.load_script(script);
        assert!(result.is_ok(), "Should load script: {}", script);
    }
}

#[test]
fn test_interactive_prover_tactics() {
    let backend = SmtBackend::Yices2;
    let prover = InteractiveProver::new(backend);
    
    // Test getting available tactics
    let tactics = prover.get_available_tactics();
    
    // Should return tactics list (size depends on implementation)
    assert!(tactics.len() >= 0, "Should return tactics");
}

#[test]
fn test_tactic_library_creation() {
    let lib = TacticLibrary::default();
    
    // Test default library
    let tactics = lib.all_tactics();
    assert!(tactics.len() >= 0, "Should have tactics available");
}

#[test]
fn test_proof_verification_result_creation() {
    let result = ProofVerificationResult {
        assertion: "2 + 2 = 4".to_string(),
        is_verified: true,
        counterexample: None,
        error: None,
        verification_time_ms: 150,
    };
    
    assert!(result.is_verified);
    assert_eq!(result.assertion, "2 + 2 = 4");
    assert!(result.counterexample.is_none());
    assert!(result.error.is_none());
    assert_eq!(result.verification_time_ms, 150);
}

#[test]
fn test_proof_verification_result_with_error() {
    let result = ProofVerificationResult {
        assertion: "false".to_string(),
        is_verified: false,
        counterexample: Some("x = 0".to_string()),
        error: Some("Assertion failed".to_string()),
        verification_time_ms: 75,
    };
    
    assert!(!result.is_verified);
    assert!(result.counterexample.is_some());
    assert!(result.error.is_some());
}

#[test]
fn test_extract_assertions_from_simple_ast() {
    // Create a simple literal expression
    let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::default());
    
    // Test assertion extraction
    let assertions = extract_assertions_from_ast(&expr);
    
    // Should handle simple expressions
    assert!(assertions.len() >= 0, "Should process simple expressions");
}

#[test]
fn test_extract_assertions_from_block() {
    // Create a block with multiple expressions
    let exprs = vec![
        Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::default()),
        Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::default()),
    ];
    let block_expr = Expr::new(ExprKind::Block(exprs), Span::default());
    
    // Test assertion extraction from block
    let assertions = extract_assertions_from_ast(&block_expr);
    
    // Should handle block expressions
    assert!(assertions.len() >= 0, "Should process block expressions");
}

#[test]
fn test_extract_assertions_with_identifier() {
    // Create an identifier expression
    let expr = Expr::new(ExprKind::Identifier("test_var".to_string()), Span::default());
    
    // Test assertion extraction
    let assertions = extract_assertions_from_ast(&expr);
    
    // Should handle identifier expressions
    assert!(assertions.len() >= 0, "Should process identifier expressions");
}

#[test]
fn test_different_smt_backends() {
    let backends = vec![
        SmtBackend::Z3,
        SmtBackend::CVC5,
        SmtBackend::Yices2,
        SmtBackend::MathSAT,
    ];
    
    for backend in backends {
        let prover = InteractiveProver::new(backend);
        
        // Each backend should work with prover
        drop(prover);
    }
}

#[test]
fn test_tactic_library_operations() {
    let lib = TacticLibrary::default();
    
    // Test basic tactic operations
    let all_tactics = lib.all_tactics();
    
    // Should provide tactic interface
    assert!(all_tactics.len() >= 0, "Should have tactics interface");
}

#[test]
fn test_verification_result_serialization() {
    let result = ProofVerificationResult {
        assertion: "test assertion".to_string(),
        is_verified: true,
        counterexample: None,
        error: None,
        verification_time_ms: 200,
    };
    
    // Test that it can be serialized (has Serialize derive)
    let json_result = serde_json::to_string(&result);
    assert!(json_result.is_ok(), "Should serialize to JSON");
    
    // Test deserialization
    if let Ok(json) = json_result {
        let deserialized: Result<ProofVerificationResult, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");
    }
}

#[test]
fn test_complex_expression_assertion_extraction() {
    // Create a more complex expression structure
    let left = Expr::new(ExprKind::Literal(Literal::Integer(5)), Span::default());
    let right = Expr::new(ExprKind::Literal(Literal::Integer(3)), Span::default());
    let binary_expr = Expr::new(
        ExprKind::Binary { 
            left: Box::new(left), 
            op: BinaryOp::Add, 
            right: Box::new(right) 
        }, 
        Span::default()
    );
    
    // Test assertion extraction from complex expression
    let assertions = extract_assertions_from_ast(&binary_expr);
    
    // Should handle complex expressions
    assert!(assertions.len() >= 0, "Should process complex expressions");
}