//! TDD Tests for Proof Verification Engine
//! 
//! Tests the actual proof verification logic that extracts assertions from AST
//! and verifies them using mathematical reasoning

use ruchy::{
    Parser as RuchyParser,
    proving::{ProofVerificationResult, extract_assertions_from_ast, verify_single_assertion, verify_assertions_batch}
};

#[test]
fn test_extract_assertions_from_simple_assert() {
    // RED: Test that we can extract assert statements from AST
    let source = r#"
assert true
assert 2 + 2 == 4
assert "hello".len() > 0
"#;
    
    let mut parser = RuchyParser::new(source);
    let ast = parser.parse().expect("Should parse simple assertions");
    
    let assertions = extract_assertions_from_ast(&ast);
    
    assert_eq!(assertions.len(), 3, "Should find 3 assert statements");
    
    // Check that we extracted the right assertions
    assert!(assertions.iter().any(|a| a.contains("true")), "Should find 'assert true'");
    assert!(assertions.iter().any(|a| a.contains("2 + 2 == 4")), "Should find arithmetic assertion");
    assert!(assertions.iter().any(|a| a.contains("len()")), "Should find string length assertion");
}

#[test]
fn test_verify_tautology() {
    // RED: Test verification of obviously true statements
    let assertion = "true";
    
    let result = verify_single_assertion(assertion, false);
    
    assert!(result.is_verified, "Tautology 'true' should always verify");
    assert!(result.counterexample.is_none(), "Tautology should not have counterexample");
    assert_eq!(result.error, None, "Tautology should not have error");
}

#[test] 
fn test_verify_contradiction() {
    // RED: Test verification of obviously false statements
    let assertion = "false";
    
    let result = verify_single_assertion(assertion, true);
    
    assert!(!result.is_verified, "Contradiction 'false' should never verify");
    assert!(result.counterexample.is_some(), "Contradiction should have counterexample");
}

#[test]
fn test_verify_arithmetic_truth() {
    // RED: Test verification of arithmetic truths
    let assertion = "2 + 2 == 4";
    
    let result = verify_single_assertion(assertion, false);
    
    assert!(result.is_verified, "Arithmetic truth should verify");
    assert!(result.counterexample.is_none(), "True arithmetic should not have counterexample");
}

#[test]
fn test_verify_arithmetic_falsehood() {
    // RED: Test verification of arithmetic falsehood with counterexample
    let assertion = "2 + 2 == 5";
    
    let result = verify_single_assertion(assertion, true);
    
    assert!(!result.is_verified, "Arithmetic falsehood should not verify");
    assert!(result.counterexample.is_some(), "False arithmetic should have counterexample");
    
    let counterexample = result.counterexample.unwrap();
    assert!(counterexample.contains('2') || counterexample.contains('4'), 
            "Counterexample should show actual values: {counterexample}");
}

#[test]
fn test_verify_conditional_property() {
    // RED: Test verification of conditional properties
    let assertion = "if x > 0 then x + 1 > x";
    
    let result = verify_single_assertion(assertion, false);
    
    // This should be true for all positive x
    assert!(result.is_verified, "Conditional property should verify");
}

#[test]
fn test_verify_universal_quantification() {
    // RED: Test verification of universal quantification
    let assertion = "forall x: i32. x + 0 == x";
    
    let result = verify_single_assertion(assertion, false);
    
    assert!(result.is_verified, "Universal quantification should verify");
}

#[test]
fn test_verify_existential_with_counterexample() {
    // RED: Test false existential should produce counterexample
    let assertion = "exists x: i32. x > x";  // This is impossible
    
    let result = verify_single_assertion(assertion, true);
    
    assert!(!result.is_verified, "Impossible existential should not verify");
    assert!(result.counterexample.is_some(), "Should have counterexample showing no such x exists");
}

#[test]
fn test_json_output_format() {
    // RED: Test that proof results can be serialized to JSON
    let result = ProofVerificationResult {
        assertion: "2 + 2 == 4".to_string(),
        is_verified: true,
        counterexample: None,
        error: None,
        verification_time_ms: 42,
    };
    
    let json = serde_json::to_string(&result).expect("Should serialize to JSON");
    
    assert!(json.contains("2 + 2 == 4"), "JSON should contain assertion");
    assert!(json.contains("true"), "JSON should contain verification result");
    assert!(json.contains("42"), "JSON should contain timing info");
}

#[test]
fn test_proof_batch_verification() {
    // RED: Test verifying multiple assertions at once
    let assertions = vec![
        "true".to_string(),
        "1 + 1 == 2".to_string(), 
        "false".to_string(),
        "3 > 2".to_string(),
    ];
    
    let results = verify_assertions_batch(&assertions, true);
    
    assert_eq!(results.len(), 4, "Should have 4 results");
    
    // Check individual results
    assert!(results[0].is_verified, "true should verify");
    assert!(results[1].is_verified, "1 + 1 == 2 should verify"); 
    assert!(!results[2].is_verified, "false should not verify");
    assert!(results[3].is_verified, "3 > 2 should verify");
    
    // Check counterexamples
    assert!(results[2].counterexample.is_some(), "false should have counterexample");
}

// Functions are now imported from ruchy::proving module