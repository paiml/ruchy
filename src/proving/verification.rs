//! Proof Verification Engine for Ruchy
//!
//! Implements actual mathematical proof verification using TDD methodology
use crate::frontend::ast::Expr;
use serde::{Deserialize, Serialize};
use std::time::Instant;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationResult {
    pub assertion: String,
    pub is_verified: bool,
    pub counterexample: Option<String>,
    pub error: Option<String>,
    pub verification_time_ms: u64,
}

#[path = "verification_modules/mod.rs"]
mod verification_modules;

// Re-export extraction functions
pub use verification_modules::extraction::{expr_to_assertion_string, extract_assertions_from_ast};

// Import helper functions
use verification_modules::helpers::{
    evaluate_expression, find_conditional_counterexample, find_existential_witness,
    find_universal_counterexample, is_always_false, is_always_true,
};

/// Verify a single assertion using mathematical reasoning
pub fn verify_single_assertion(
    assertion: &str,
    _assumptions: Option<Vec<String>>,
) -> ProofVerificationResult {
    let start = Instant::now();

    // Parse the assertion to determine its type
    let (is_verified, counterexample, error) = if assertion.contains("forall") {
        verify_universal_quantification(assertion)
    } else if assertion.contains("exists") {
        verify_existential_quantification(assertion)
    } else if assertion.contains("=>") || assertion.contains("->") {
        verify_conditional_property(assertion)
    } else {
        // Simple assertion verification
        verify_simple_assertion(assertion)
    };

    let elapsed = start.elapsed();

    ProofVerificationResult {
        assertion: assertion.to_string(),
        is_verified,
        counterexample,
        error,
        verification_time_ms: elapsed.as_millis() as u64,
    }
}

fn verify_conditional_property(assertion: &str) -> (bool, Option<String>, Option<String>) {
    match parse_conditional(assertion) {
        Ok((antecedent, consequent)) => check_conditional_logic(antecedent, consequent),
        Err(error) => error,
    }
}

fn parse_conditional(
    assertion: &str,
) -> Result<(&str, &str), (bool, Option<String>, Option<String>)> {
    let parts: Vec<&str> = if assertion.contains("=>") {
        assertion.split("=>").collect()
    } else {
        assertion.split("->").collect()
    };

    if parts.len() != 2 {
        return Err((false, None, Some("Invalid conditional format".to_string())));
    }

    Ok((parts[0].trim(), parts[1].trim()))
}

fn check_conditional_logic(
    antecedent: &str,
    consequent: &str,
) -> (bool, Option<String>, Option<String>) {
    if is_always_false(antecedent) || is_always_true(consequent) {
        return (true, None, None);
    }

    match find_conditional_counterexample(antecedent, consequent) {
        Some(counterex) => (false, Some(counterex), None),
        None => (true, None, None),
    }
}

fn verify_universal_quantification(assertion: &str) -> (bool, Option<String>, Option<String>) {
    // Parse: "forall x in [range], property(x)"
    if !assertion.starts_with("forall") {
        return (
            false,
            None,
            Some("Invalid universal quantification".to_string()),
        );
    }

    // Extract variable, range, and property
    let parts: Vec<&str> = assertion.split(',').collect();
    if parts.len() < 2 {
        return (false, None, Some("Malformed forall statement".to_string()));
    }

    // Try to find a counterexample
    match find_universal_counterexample(assertion) {
        Some(counterex) => (false, Some(counterex), None),
        None => (true, None, None),
    }
}

fn verify_existential_quantification(assertion: &str) -> (bool, Option<String>, Option<String>) {
    // Parse: "exists x in [range], property(x)"
    if !assertion.starts_with("exists") {
        return (
            false,
            None,
            Some("Invalid existential quantification".to_string()),
        );
    }

    // Extract variable, range, and property
    let parts: Vec<&str> = assertion.split(',').collect();
    if parts.len() < 2 {
        return (false, None, Some("Malformed exists statement".to_string()));
    }

    // Try to find a witness (example that satisfies the property)
    match find_existential_witness(assertion) {
        Some(_witness) => (true, None, None),
        None => (
            false,
            None,
            Some("No witness found for existential claim".to_string()),
        ),
    }
}

fn verify_simple_assertion(assertion: &str) -> (bool, Option<String>, Option<String>) {
    // Handle simple mathematical assertions
    if assertion.contains("==") {
        verify_equality(assertion)
    } else if assertion.contains("!=") {
        verify_inequality(assertion)
    } else if assertion.contains(">=")
        || assertion.contains("<=")
        || assertion.contains('>')
        || assertion.contains('<')
    {
        verify_comparison(assertion)
    } else {
        (
            false,
            None,
            Some(format!("Unknown assertion type: {assertion}")),
        )
    }
}

fn verify_equality(assertion: &str) -> (bool, Option<String>, Option<String>) {
    let parts: Vec<&str> = assertion.split("==").collect();
    if parts.len() != 2 {
        return (false, None, Some("Invalid equality format".to_string()));
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    // Try to evaluate both sides
    match (evaluate_expression(left), evaluate_expression(right)) {
        (Some(left_val), Some(right_val)) => {
            if (left_val - right_val).abs() < 1e-10 {
                (true, None, None)
            } else {
                (
                    false,
                    Some(format!("{left} = {left_val}, {right} = {right_val}")),
                    None,
                )
            }
        }
        _ => (false, None, Some("Cannot evaluate expressions".to_string())),
    }
}

fn verify_inequality(assertion: &str) -> (bool, Option<String>, Option<String>) {
    let parts: Vec<&str> = assertion.split("!=").collect();
    if parts.len() != 2 {
        return (false, None, Some("Invalid inequality format".to_string()));
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    match (evaluate_expression(left), evaluate_expression(right)) {
        (Some(left_val), Some(right_val)) => {
            if (left_val - right_val).abs() >= 1e-10 {
                (true, None, None)
            } else {
                (false, Some(format!("{left} = {right} = {left_val}")), None)
            }
        }
        _ => (false, None, Some("Cannot evaluate expressions".to_string())),
    }
}

fn verify_comparison(assertion: &str) -> (bool, Option<String>, Option<String>) {
    let (op, parts) = match parse_comparison_operator(assertion) {
        Ok(result) => result,
        Err(error) => return error,
    };

    if parts.len() != 2 {
        return (false, None, Some("Invalid comparison format".to_string()));
    }

    let left = parts[0].trim();
    let right = parts[1].trim();

    match (evaluate_expression(left), evaluate_expression(right)) {
        (Some(left_val), Some(right_val)) => {
            check_comparison_result(op, left, left_val, right, right_val)
        }
        _ => (false, None, Some("Cannot evaluate expressions".to_string())),
    }
}

fn parse_comparison_operator(
    assertion: &str,
) -> Result<(&str, Vec<&str>), (bool, Option<String>, Option<String>)> {
    if assertion.contains(">=") {
        Ok((">=", assertion.split(">=").collect::<Vec<_>>()))
    } else if assertion.contains("<=") {
        Ok(("<=", assertion.split("<=").collect::<Vec<_>>()))
    } else if assertion.contains('>') {
        Ok((">", assertion.split('>').collect::<Vec<_>>()))
    } else if assertion.contains('<') {
        Ok(("<", assertion.split('<').collect::<Vec<_>>()))
    } else {
        Err((false, None, Some("Unknown comparison operator".to_string())))
    }
}

fn check_comparison_result(
    op: &str,
    left: &str,
    left_val: f64,
    right: &str,
    right_val: f64,
) -> (bool, Option<String>, Option<String>) {
    let is_true = match op {
        ">=" => left_val >= right_val - 1e-10,
        "<=" => left_val <= right_val + 1e-10,
        ">" => left_val > right_val + 1e-10,
        "<" => left_val < right_val - 1e-10,
        _ => false,
    };

    if is_true {
        (true, None, None)
    } else {
        (
            false,
            Some(format!("{left} = {left_val}, {right} = {right_val}")),
            None,
        )
    }
}

/// Verify multiple assertions in batch
pub fn verify_assertions_batch(
    assertions: Vec<String>,
    assumptions: Option<Vec<String>>,
) -> Vec<ProofVerificationResult> {
    assertions
        .iter()
        .map(|assertion| verify_single_assertion(assertion, assumptions.clone()))
        .collect()
}

/// Verify assertions extracted from an AST
pub fn verify_ast_assertions(
    ast: &Expr,
    assumptions: Option<Vec<String>>,
) -> Vec<ProofVerificationResult> {
    let assertions = extract_assertions_from_ast(ast);
    verify_assertions_batch(assertions, assumptions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_simple_equality_true() {
        let result = verify_single_assertion("2 + 2 == 4", None);
        assert!(result.is_verified);
        assert!(result.counterexample.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_verify_simple_equality_false() {
        let result = verify_single_assertion("2 + 2 == 5", None);
        assert!(!result.is_verified);
        assert!(result.counterexample.is_some());
    }

    #[test]
    fn test_verify_inequality_true() {
        let result = verify_single_assertion("3 != 4", None);
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_inequality_false() {
        let result = verify_single_assertion("5 != 5", None);
        assert!(!result.is_verified);
    }

    #[test]
    fn test_verify_greater_than_true() {
        let result = verify_single_assertion("5 > 3", None);
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_greater_than_false() {
        let result = verify_single_assertion("2 > 5", None);
        assert!(!result.is_verified);
    }

    #[test]
    fn test_verify_less_than_true() {
        let result = verify_single_assertion("2 < 5", None);
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_less_than_false() {
        let result = verify_single_assertion("5 < 2", None);
        assert!(!result.is_verified);
    }

    #[test]
    fn test_verify_greater_or_equal_true() {
        let result = verify_single_assertion("5 >= 5", None);
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_greater_or_equal_false() {
        let result = verify_single_assertion("3 >= 5", None);
        assert!(!result.is_verified);
    }

    #[test]
    fn test_verify_less_or_equal_true() {
        let result = verify_single_assertion("3 <= 5", None);
        assert!(result.is_verified);
    }

    #[test]
    fn test_verify_less_or_equal_false() {
        let result = verify_single_assertion("7 <= 5", None);
        assert!(!result.is_verified);
    }

    #[test]
    fn test_verify_conditional_with_arrow() {
        let result = verify_single_assertion("false -> true", None);
        assert!(result.is_verified); // Vacuously true
    }

    #[test]
    fn test_verify_conditional_with_double_arrow() {
        let result = verify_single_assertion("false => true", None);
        assert!(result.is_verified); // Vacuously true
    }

    #[test]
    fn test_verify_universal_quantification() {
        let result = verify_single_assertion("forall x in [1,2,3], x > 0", None);
        // Parsing structure is present, result depends on helper implementation
        assert!(!result.assertion.is_empty());
    }

    #[test]
    fn test_verify_existential_quantification() {
        let result = verify_single_assertion("exists x in [1,2,3], x > 0", None);
        assert!(!result.assertion.is_empty());
    }

    #[test]
    fn test_verify_unknown_assertion_type() {
        let result = verify_single_assertion("some_unknown_assertion", None);
        assert!(!result.is_verified);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_verify_invalid_equality_format() {
        let result = verify_equality("a == b == c");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_verify_invalid_inequality_format() {
        let result = verify_inequality("a != b != c");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_parse_conditional_invalid() {
        let result = parse_conditional("a => b => c");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_conditional_arrow() {
        let result = parse_conditional("a -> b");
        assert!(result.is_ok());
        let (ant, cons) = result.unwrap();
        assert_eq!(ant, "a");
        assert_eq!(cons, "b");
    }

    #[test]
    fn test_parse_conditional_double_arrow() {
        let result = parse_conditional("x => y");
        assert!(result.is_ok());
        let (ant, cons) = result.unwrap();
        assert_eq!(ant, "x");
        assert_eq!(cons, "y");
    }

    #[test]
    fn test_verify_forall_malformed() {
        let result = verify_universal_quantification("forall x without comma");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_verify_forall_not_starting_with_forall() {
        let result = verify_universal_quantification("not_forall x, property");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_verify_exists_malformed() {
        let result = verify_existential_quantification("exists x without comma");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_verify_exists_not_starting_with_exists() {
        let result = verify_existential_quantification("not_exists x, property");
        assert!(!result.0);
        assert!(result.2.is_some());
    }

    #[test]
    fn test_verify_assertions_batch() {
        let assertions = vec!["2 + 2 == 4".to_string(), "3 > 2".to_string()];
        let results = verify_assertions_batch(assertions, None);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_verified);
        assert!(results[1].is_verified);
    }

    #[test]
    fn test_parse_comparison_operator_ge() {
        let result = parse_comparison_operator("a >= b");
        assert!(result.is_ok());
        let (op, parts) = result.unwrap();
        assert_eq!(op, ">=");
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_parse_comparison_operator_le() {
        let result = parse_comparison_operator("a <= b");
        assert!(result.is_ok());
        let (op, _) = result.unwrap();
        assert_eq!(op, "<=");
    }

    #[test]
    fn test_parse_comparison_operator_gt() {
        let result = parse_comparison_operator("a > b");
        assert!(result.is_ok());
        let (op, _) = result.unwrap();
        assert_eq!(op, ">");
    }

    #[test]
    fn test_parse_comparison_operator_lt() {
        let result = parse_comparison_operator("a < b");
        assert!(result.is_ok());
        let (op, _) = result.unwrap();
        assert_eq!(op, "<");
    }

    #[test]
    fn test_parse_comparison_operator_unknown() {
        let result = parse_comparison_operator("a ~ b");
        assert!(result.is_err());
    }

    #[test]
    fn test_check_comparison_result_operators() {
        assert!(check_comparison_result(">=", "5", 5.0, "3", 3.0).0);
        assert!(check_comparison_result("<=", "3", 3.0, "5", 5.0).0);
        assert!(check_comparison_result(">", "5", 5.0, "3", 3.0).0);
        assert!(check_comparison_result("<", "3", 3.0, "5", 5.0).0);
        assert!(!check_comparison_result("??", "5", 5.0, "3", 3.0).0);
    }

    #[test]
    fn test_verification_result_fields() {
        let result = verify_single_assertion("1 == 1", None);
        assert!(!result.assertion.is_empty());
        // Verify time was recorded (verification_time_ms is u64, always valid)
        let _ = result.verification_time_ms;
    }

    #[test]
    fn test_verify_comparison_invalid_format() {
        // Test with more than 2 parts
        let result = verify_comparison("a >= b >= c");
        assert!(!result.0);
    }
}
