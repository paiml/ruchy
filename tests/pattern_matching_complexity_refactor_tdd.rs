//! EXTREME TDD: Pattern Matching Complexity Refactoring
//! Target: Reduce `pattern_matches_simple` complexity from 12 → ≤10
//!
//! This test-driven refactoring decomposes `pattern_matches_simple` into focused helper functions,
//! each with single responsibility and complexity ≤10.

use ruchy::frontend::ast::{Literal, Pattern};
use ruchy::runtime::eval_control_flow_new::pattern_matches_simple;
use ruchy::runtime::interpreter::Value;

/// Test suite for decomposed pattern matching functions
/// These tests will initially fail as the helper functions don't exist yet
#[cfg(test)]
mod pattern_matching_refactor_tests {
    use super::*;

    // =============================================================================
    // STEP 1: Tests for individual pattern type matchers (will fail initially)
    // =============================================================================

    #[test]
    fn test_match_wildcard_pattern() {
        // Helper function to match wildcard patterns
        let value = Value::Integer(42);
        assert!(ruchy::runtime::eval_control_flow_new::match_wildcard_pattern(&value));

        let value2 = Value::String("hello".to_string().into());
        assert!(ruchy::runtime::eval_control_flow_new::match_wildcard_pattern(&value2));
    }

    #[test]
    fn test_match_literal_pattern() {
        // Helper function to match literal patterns
        use ruchy::runtime::eval_control_flow_new::match_literal_pattern;

        let literal = Literal::Integer(42);
        let matching_value = Value::Integer(42);
        let non_matching_value = Value::Integer(43);

        assert!(match_literal_pattern(&literal, &matching_value).unwrap());
        assert!(!match_literal_pattern(&literal, &non_matching_value).unwrap());
    }

    #[test]
    fn test_match_identifier_pattern() {
        // Helper function to match identifier patterns (always matches, binds variable)
        use ruchy::runtime::eval_control_flow_new::match_identifier_pattern;

        let value = Value::Integer(42);
        assert!(match_identifier_pattern("x", &value)); // Should always return true

        let value2 = Value::String("hello".to_string().into());
        assert!(match_identifier_pattern("name", &value2)); // Should always return true
    }

    #[test]
    fn test_match_list_pattern() {
        // Helper function to match list patterns recursively
        use ruchy::runtime::eval_control_flow_new::match_list_pattern;

        let patterns = vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
        ];

        let matching_array = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());

        let non_matching_array = Value::Array(
            vec![
                Value::Integer(1),
                Value::Integer(3), // Different value
            ]
            .into(),
        );

        let wrong_length_array = Value::Array(vec![Value::Integer(1)].into());

        let non_array_value = Value::Integer(42);

        assert!(match_list_pattern(&patterns, &matching_array).unwrap());
        assert!(!match_list_pattern(&patterns, &non_matching_array).unwrap());
        assert!(!match_list_pattern(&patterns, &wrong_length_array).unwrap());
        assert!(!match_list_pattern(&patterns, &non_array_value).unwrap());
    }

    #[test]
    fn test_match_tuple_pattern() {
        // Helper function to match tuple patterns recursively
        use ruchy::runtime::eval_control_flow_new::match_tuple_pattern;

        let patterns = vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::String("hello".to_string())),
        ];

        let matching_tuple =
            Value::Tuple(vec![Value::Integer(1), Value::String("hello".to_string().into())].into());

        let non_matching_tuple = Value::Tuple(
            vec![
                Value::Integer(1),
                Value::String("world".to_string().into()), // Different value
            ]
            .into(),
        );

        let wrong_length_tuple = Value::Tuple(vec![Value::Integer(1)].into());

        let non_tuple_value = Value::Integer(42);

        assert!(match_tuple_pattern(&patterns, &matching_tuple).unwrap());
        assert!(!match_tuple_pattern(&patterns, &non_matching_tuple).unwrap());
        assert!(!match_tuple_pattern(&patterns, &wrong_length_tuple).unwrap());
        assert!(!match_tuple_pattern(&patterns, &non_tuple_value).unwrap());
    }

    // =============================================================================
    // STEP 2: Tests for the refactored main function (using helper functions)
    // =============================================================================

    #[test]
    fn test_refactored_pattern_matches_simple_integration() {
        // These tests should pass with the current implementation,
        // and continue to pass after refactoring (regression tests)

        // Test wildcard pattern
        let wildcard_pattern = Pattern::Wildcard;
        assert!(pattern_matches_simple(&wildcard_pattern, &Value::Integer(42)).unwrap());

        // Test literal pattern
        let literal_pattern = Pattern::Literal(Literal::Integer(42));
        assert!(pattern_matches_simple(&literal_pattern, &Value::Integer(42)).unwrap());
        assert!(!pattern_matches_simple(&literal_pattern, &Value::Integer(43)).unwrap());

        // Test identifier pattern
        let identifier_pattern = Pattern::Identifier("x".to_string());
        assert!(pattern_matches_simple(&identifier_pattern, &Value::Integer(42)).unwrap());

        // Test list pattern
        let list_pattern = Pattern::List(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
        ]);
        let array_value = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
        assert!(pattern_matches_simple(&list_pattern, &array_value).unwrap());

        // Test tuple pattern
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::String("hello".to_string())),
        ]);
        let tuple_value =
            Value::Tuple(vec![Value::Integer(1), Value::String("hello".to_string().into())].into());
        assert!(pattern_matches_simple(&tuple_pattern, &tuple_value).unwrap());
    }

    // =============================================================================
    // STEP 3: Complexity verification tests (meta-tests)
    // =============================================================================

    #[test]
    fn test_complexity_compliance_verification() {
        // This test documents that after refactoring, all functions have complexity ≤10
        // Manual verification required via PMAT analysis:
        //
        // Expected after refactoring:
        // - pattern_matches_simple: ≤10 (down from 12)
        // - match_wildcard_pattern: ≤3
        // - match_literal_pattern: ≤5
        // - match_identifier_pattern: ≤2
        // - match_list_pattern: ≤8
        // - match_tuple_pattern: ≤8
        //
        // Total complexity reduction: 12 → (≤10 + ≤3 + ≤5 + ≤2 + ≤8 + ≤8) distributed
        // but main function complexity target achieved: ≤10

        // Test passes without assertion - this is documentation
    }

    // =============================================================================
    // STEP 4: Property-based testing for robustness (EXTREME TDD)
    // =============================================================================

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use quickcheck::TestResult;
        use quickcheck_macros::quickcheck;

        #[quickcheck]
        fn test_pattern_matching_never_panics_on_valid_inputs(value: i32) -> TestResult {
            let pattern = Pattern::Literal(Literal::Integer(value.into()));
            let test_value = Value::Integer(value.into());

            // Should never panic on valid inputs
            let result = pattern_matches_simple(&pattern, &test_value);
            assert!(result.is_ok());
            assert!(result.unwrap()); // Should match

            TestResult::passed()
        }

        #[quickcheck]
        fn test_wildcard_always_matches(value: i32) -> TestResult {
            let wildcard = Pattern::Wildcard;
            let test_value = Value::Integer(value.into());

            let result = pattern_matches_simple(&wildcard, &test_value);
            assert!(result.is_ok());
            assert!(result.unwrap()); // Wildcard should always match

            TestResult::passed()
        }
    }
}
