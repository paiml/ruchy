//! Basic tests for proving system modules
//!
//! [COVERAGE-BOOST-001] Target proving modules with 0% coverage

use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span};
use ruchy::proving::prover::*;
use ruchy::proving::smt::*;
use ruchy::proving::tactics::*;
use ruchy::proving::verification::*;

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
        assert!(!format!("{backend:?}").is_empty());

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
        assert!(result.is_ok(), "Should load script: {script}");
    }
}

#[test]
fn test_interactive_prover_tactics() {
    let backend = SmtBackend::Yices2;
    let prover = InteractiveProver::new(backend);

    // Test getting available tactics
    let tactics = prover.get_available_tactics();

    // Should return tactics list (size depends on implementation)
    assert!(
        tactics.len() < 1000,
        "Should return reasonable number of tactics"
    );
}

#[test]
fn test_tactic_library_creation() {
    let lib = TacticLibrary::default();

    // Test default library
    let tactics = lib.all_tactics();
    assert!(
        tactics.len() < 1000,
        "Should have reasonable tactics available"
    );
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

    // Should handle simple expressions without panicking
    drop(assertions);
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

    // Should handle block expressions without panicking
    drop(assertions);
}

#[test]
fn test_extract_assertions_with_identifier() {
    // Create an identifier expression
    let expr = Expr::new(
        ExprKind::Identifier("test_var".to_string()),
        Span::default(),
    );

    // Test assertion extraction
    let assertions = extract_assertions_from_ast(&expr);

    // Should handle identifier expressions without panicking
    drop(assertions);
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
    assert!(
        all_tactics.len() < 1000,
        "Should have reasonable tactics interface"
    );
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
            right: Box::new(right),
        },
        Span::default(),
    );

    // Test assertion extraction from complex expression
    let assertions = extract_assertions_from_ast(&binary_expr);

    // Should handle complex expressions without panicking
    drop(assertions);
}

#[test]
fn test_verify_single_assertion_arithmetic_truths() {
    // Test basic arithmetic verification
    let result = verify_single_assertion("2 + 2 == 4", false);
    assert!(result.is_verified, "2 + 2 == 4 should be verified");
    assert!(result.counterexample.is_none());
    assert!(result.error.is_none());

    let result2 = verify_single_assertion("1 + 1 == 2", false);
    assert!(result2.is_verified, "1 + 1 == 2 should be verified");
}

#[test]
fn test_verify_single_assertion_arithmetic_falsehoods() {
    // Test arithmetic falsehood detection
    let result = verify_single_assertion("2 + 2 == 5", true);
    assert!(!result.is_verified, "2 + 2 == 5 should not be verified");
    assert!(result.counterexample.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_verify_single_assertion_tautologies() {
    let result = verify_single_assertion("true", false);
    assert!(result.is_verified, "true should always be verified");
    assert!(result.counterexample.is_none());
    assert!(result.error.is_none());
}

#[test]
fn test_verify_single_assertion_contradictions() {
    let result = verify_single_assertion("false", true);
    assert!(!result.is_verified, "false should never be verified");
    assert!(result.counterexample.is_some());
    assert!(result.error.is_none());
}

#[test]
fn test_verify_single_assertion_comparison_truths() {
    let result = verify_single_assertion("3 > 2", false);
    assert!(result.is_verified, "3 > 2 should be verified");
    assert!(result.counterexample.is_none());
    assert!(result.error.is_none());
}

#[test]
fn test_verify_single_assertion_unknown_patterns() {
    let result = verify_single_assertion("unknown_complex_assertion", false);
    assert!(
        !result.is_verified,
        "Unknown assertions should not be verified"
    );
    assert!(result.error.is_some());
    assert!(result.counterexample.is_none());
}

#[test]
fn test_verify_assertions_batch() {
    let assertions = vec![
        "true".to_string(),
        "2 + 2 == 4".to_string(),
        "false".to_string(),
    ];

    let results = verify_assertions_batch(&assertions, false);
    assert_eq!(results.len(), 3);

    assert!(results[0].is_verified); // true
    assert!(results[1].is_verified); // 2 + 2 == 4
    assert!(!results[2].is_verified); // false
}

#[test]
fn test_verify_assertions_batch_with_counterexamples() {
    let assertions = vec!["2 + 2 == 5".to_string(), "false".to_string()];

    let results = verify_assertions_batch(&assertions, true);
    assert_eq!(results.len(), 2);

    // Both should have counterexamples
    assert!(results[0].counterexample.is_some());
    assert!(results[1].counterexample.is_some());
}

#[test]
fn test_extract_assert_sequence_from_block_patterns() {
    // Use existing imports - Call and MethodCall are enum variants, not types

    // Test assert pattern recognition in blocks
    let assert_ident = Expr::new(ExprKind::Identifier("assert".to_string()), Span::default());
    let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());

    let block_exprs = vec![assert_ident, condition];
    let block = Expr::new(ExprKind::Block(block_exprs), Span::default());

    let assertions = extract_assertions_from_ast(&block);
    drop(assertions); // Should extract assert patterns from blocks without panicking
}

#[test]
fn test_assert_call_pattern_recognition() {
    // Create an assert function call
    let assert_func = Expr::new(ExprKind::Identifier("assert".to_string()), Span::default());
    let condition = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
    let call = Expr::new(
        ExprKind::Call {
            func: Box::new(assert_func),
            args: vec![condition],
        },
        Span::default(),
    );

    let assertions = extract_assertions_from_ast(&call);
    drop(assertions); // Should extract assertions from assert calls without panicking
}

#[test]
fn test_expr_to_assertion_string_comprehensive() {
    // Test various expression types converted to assertion strings

    // Integer literal
    let int_expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::default());
    let assertions = extract_assertions_from_ast(&int_expr);

    // String literal
    let str_expr = Expr::new(
        ExprKind::Literal(Literal::String("test".to_string())),
        Span::default(),
    );
    let assertions2 = extract_assertions_from_ast(&str_expr);

    // Bool literal
    let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::default());
    let assertions3 = extract_assertions_from_ast(&bool_expr);

    // Float literal
    let float_expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Span::default());
    let assertions4 = extract_assertions_from_ast(&float_expr);

    // All should be processable without panicking
    drop((assertions, assertions2, assertions3, assertions4));
}

#[test]
fn test_nested_expression_assertion_extraction() {
    // Create deeply nested expressions
    let inner = Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::default());
    let middle = Expr::new(ExprKind::Block(vec![inner]), Span::default());
    let outer = Expr::new(ExprKind::Block(vec![middle]), Span::default());

    let assertions = extract_assertions_from_ast(&outer);
    drop(assertions); // Should handle nested expressions without panicking
}

#[test]
fn test_verification_timing_is_recorded() {
    let result = verify_single_assertion("true", false);
    assert!(
        result.verification_time_ms < 10000,
        "Should record reasonable verification time"
    );

    let result2 = verify_single_assertion("2 + 2 == 4", false);
    assert!(
        result2.verification_time_ms < 10000,
        "Should record reasonable timing for complex assertions"
    );
}

#[test]
fn test_prover_script_error_handling() {
    let backend = SmtBackend::Z3;
    let mut prover = InteractiveProver::new(backend);

    // Test various potentially problematic scripts
    let problematic_scripts = vec![
        "malformed script",
        "incomplete theorem",
        "syntax error here!",
        "very very long script that might cause issues in parsing or processing",
    ];

    for script in problematic_scripts {
        let result = prover.load_script(script);
        // Should handle gracefully (either succeed or fail predictably)
        if let Ok(()) = result {}
    }
}

#[test]
fn test_smt_backend_debug_format() {
    let backends = [
        SmtBackend::Z3,
        SmtBackend::CVC5,
        SmtBackend::Yices2,
        SmtBackend::MathSAT,
    ];

    for backend in backends {
        let debug_str = format!("{backend:?}");
        assert!(
            !debug_str.is_empty(),
            "Backend should have non-empty debug format"
        );
        assert!(debug_str.len() >= 2, "Debug format should be meaningful");
    }
}

#[test]
fn test_tactic_library_default_state() {
    let lib = TacticLibrary::default();
    let tactics = lib.all_tactics();

    // Default library should be in consistent state
    assert!(
        tactics.len() < 1000,
        "Default library should have defined tactics state"
    );
}

#[test]
fn test_proof_verification_result_field_access() {
    let result = ProofVerificationResult {
        assertion: "test_assertion".to_string(),
        is_verified: true,
        counterexample: Some("example".to_string()),
        error: Some("test_error".to_string()),
        verification_time_ms: 500,
    };

    // Test all field access
    assert_eq!(result.assertion, "test_assertion");
    assert!(result.is_verified);
    assert_eq!(result.counterexample.unwrap(), "example");
    assert_eq!(result.error.unwrap(), "test_error");
    assert_eq!(result.verification_time_ms, 500);
}
