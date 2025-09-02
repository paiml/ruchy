// TDD Test Suite for Interpreter::pattern_matches Complexity Reduction
// REGRESSION TESTS for refactored pattern matching system
// Tests all pattern types to ensure zero functionality loss

use ruchy::runtime::interpreter::{Interpreter, InterpreterError};
use ruchy::frontend::value::Value;
use ruchy::frontend::ast::{Pattern, Literal};
use std::rc::Rc;

#[cfg(test)]
mod pattern_matches_tdd {
    use super::*;

    fn create_test_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_wildcard_pattern_always_matches() {
        let interpreter = create_test_interpreter();
        let pattern = Pattern::Wildcard;
        
        // Test different value types
        assert!(interpreter.pattern_matches(&pattern, &Value::Integer(42)).unwrap());
        assert!(interpreter.pattern_matches(&pattern, &Value::String(Rc::new("hello".to_string()))).unwrap());
        assert!(interpreter.pattern_matches(&pattern, &Value::Bool(true)).unwrap());
        assert!(interpreter.pattern_matches(&pattern, &Value::Nil).unwrap());
    }

    #[test]
    fn test_literal_integer_pattern() {
        let interpreter = create_test_interpreter();
        let pattern = Pattern::Literal(Literal::Integer(42));
        
        // Should match exact value
        assert!(interpreter.pattern_matches(&pattern, &Value::Integer(42)).unwrap());
        
        // Should not match different value
        assert!(!interpreter.pattern_matches(&pattern, &Value::Integer(43)).unwrap());
        assert!(!interpreter.pattern_matches(&pattern, &Value::String(Rc::new("42".to_string()))).unwrap());
    }

    #[test]
    fn test_literal_string_pattern() {
        let interpreter = create_test_interpreter();
        let pattern = Pattern::Literal(Literal::String("hello".to_string()));
        
        // Should match exact string
        assert!(interpreter.pattern_matches(&pattern, &Value::String(Rc::new("hello".to_string()))).unwrap());
        
        // Should not match different string
        assert!(!interpreter.pattern_matches(&pattern, &Value::String(Rc::new("world".to_string()))).unwrap());
        assert!(!interpreter.pattern_matches(&pattern, &Value::Integer(42)).unwrap());
    }

    #[test]
    fn test_literal_bool_pattern() {
        let interpreter = create_test_interpreter();
        let true_pattern = Pattern::Literal(Literal::Bool(true));
        let false_pattern = Pattern::Literal(Literal::Bool(false));
        
        assert!(interpreter.pattern_matches(&true_pattern, &Value::Bool(true)).unwrap());
        assert!(!interpreter.pattern_matches(&true_pattern, &Value::Bool(false)).unwrap());
        assert!(interpreter.pattern_matches(&false_pattern, &Value::Bool(false)).unwrap());
        assert!(!interpreter.pattern_matches(&false_pattern, &Value::Bool(true)).unwrap());
    }

    #[test]
    fn test_identifier_pattern_always_matches() {
        let interpreter = create_test_interpreter();
        let pattern = Pattern::Identifier("x".to_string());
        
        // Identifier patterns should always match (binding is handled separately)
        assert!(interpreter.pattern_matches(&pattern, &Value::Integer(42)).unwrap());
        assert!(interpreter.pattern_matches(&pattern, &Value::String(Rc::new("hello".to_string()))).unwrap());
        assert!(interpreter.pattern_matches(&pattern, &Value::Nil).unwrap());
    }

    #[test]
    fn test_tuple_pattern_matching() {
        let interpreter = create_test_interpreter();
        
        // Empty tuple
        let empty_pattern = Pattern::Tuple(vec![]);
        let empty_tuple = Value::Tuple(vec![]);
        assert!(interpreter.pattern_matches(&empty_pattern, &empty_tuple).unwrap());
        
        // Single element tuple
        let single_pattern = Pattern::Tuple(vec![Pattern::Literal(Literal::Integer(42))]);
        let single_tuple = Value::Tuple(vec![Value::Integer(42)]);
        let wrong_tuple = Value::Tuple(vec![Value::Integer(43)]);
        
        assert!(interpreter.pattern_matches(&single_pattern, &single_tuple).unwrap());
        assert!(!interpreter.pattern_matches(&single_pattern, &wrong_tuple).unwrap());
        
        // Multi-element tuple
        let multi_pattern = Pattern::Tuple(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::String("hello".to_string())),
            Pattern::Wildcard,
        ]);
        let multi_tuple = Value::Tuple(vec![
            Value::Integer(1),
            Value::String(Rc::new("hello".to_string())),
            Value::Bool(true),
        ]);
        assert!(interpreter.pattern_matches(&multi_pattern, &multi_tuple).unwrap());
        
        // Wrong length should not match
        let short_tuple = Value::Tuple(vec![Value::Integer(1)]);
        assert!(!interpreter.pattern_matches(&multi_pattern, &short_tuple).unwrap());
    }

    #[test]
    fn test_list_pattern_matching() {
        let interpreter = create_test_interpreter();
        
        // Empty list
        let empty_pattern = Pattern::List(vec![]);
        let empty_array = Value::Array(Rc::new(vec![]));
        assert!(interpreter.pattern_matches(&empty_pattern, &empty_array).unwrap());
        
        // Single element list
        let single_pattern = Pattern::List(vec![Pattern::Literal(Literal::Integer(42))]);
        let single_array = Value::Array(Rc::new(vec![Value::Integer(42)]));
        let wrong_array = Value::Array(Rc::new(vec![Value::Integer(43)]));
        
        assert!(interpreter.pattern_matches(&single_pattern, &single_array).unwrap());
        assert!(!interpreter.pattern_matches(&single_pattern, &wrong_array).unwrap());
        
        // Multi-element list
        let multi_pattern = Pattern::List(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Wildcard,
        ]);
        let multi_array = Value::Array(Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(99),
        ]));
        assert!(interpreter.pattern_matches(&multi_pattern, &multi_array).unwrap());
        
        // Wrong length should not match
        let short_array = Value::Array(Rc::new(vec![Value::Integer(1)]));
        assert!(!interpreter.pattern_matches(&multi_pattern, &short_array).unwrap());
    }

    #[test]
    fn test_or_pattern_matching() {
        let interpreter = create_test_interpreter();
        
        let or_pattern = Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1)),
            Pattern::Literal(Literal::Integer(2)),
            Pattern::Literal(Literal::String("hello".to_string())),
        ]);
        
        // Should match any of the alternatives
        assert!(interpreter.pattern_matches(&or_pattern, &Value::Integer(1)).unwrap());
        assert!(interpreter.pattern_matches(&or_pattern, &Value::Integer(2)).unwrap());
        assert!(interpreter.pattern_matches(&or_pattern, &Value::String(Rc::new("hello".to_string()))).unwrap());
        
        // Should not match values not in alternatives
        assert!(!interpreter.pattern_matches(&or_pattern, &Value::Integer(3)).unwrap());
        assert!(!interpreter.pattern_matches(&or_pattern, &Value::String(Rc::new("world".to_string()))).unwrap());
        
        // Empty or pattern should never match
        let empty_or = Pattern::Or(vec![]);
        assert!(!interpreter.pattern_matches(&empty_or, &Value::Integer(1)).unwrap());
    }

    #[test]
    fn test_range_pattern_inclusive() {
        let interpreter = create_test_interpreter();
        
        let range_pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::Integer(5))),
            inclusive: true,
        };
        
        // Should match values within inclusive range
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(1)).unwrap());
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(3)).unwrap());
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(5)).unwrap());
        
        // Should not match values outside range
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::Integer(0)).unwrap());
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::Integer(6)).unwrap());
        
        // Should not match non-integer values
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::String(Rc::new("3".to_string()))).unwrap());
    }

    #[test]
    fn test_range_pattern_exclusive() {
        let interpreter = create_test_interpreter();
        
        let range_pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::Integer(5))),
            inclusive: false,
        };
        
        // Should match values within exclusive range
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(1)).unwrap());
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(3)).unwrap());
        assert!(interpreter.pattern_matches(&range_pattern, &Value::Integer(4)).unwrap());
        
        // Should not match end value in exclusive range
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::Integer(5)).unwrap());
        
        // Should not match values outside range
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::Integer(0)).unwrap());
        assert!(!interpreter.pattern_matches(&range_pattern, &Value::Integer(6)).unwrap());
    }

    #[test]
    fn test_range_pattern_invalid_bounds() {
        let interpreter = create_test_interpreter();
        
        // Range with non-integer start should return error
        let invalid_start = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::String("not_int".to_string()))),
            end: Box::new(Pattern::Literal(Literal::Integer(5))),
            inclusive: true,
        };
        
        let result = interpreter.pattern_matches(&invalid_start, &Value::Integer(3));
        assert!(result.is_err());
        
        // Range with non-integer end should return error
        let invalid_end = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1))),
            end: Box::new(Pattern::Literal(Literal::String("not_int".to_string()))),
            inclusive: true,
        };
        
        let result = interpreter.pattern_matches(&invalid_end, &Value::Integer(3));
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_patterns() {
        let interpreter = create_test_interpreter();
        
        // Tuple containing an or pattern and a list pattern
        let nested_pattern = Pattern::Tuple(vec![
            Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1)),
                Pattern::Literal(Literal::Integer(2)),
            ]),
            Pattern::List(vec![
                Pattern::Wildcard,
                Pattern::Literal(Literal::String("test".to_string())),
            ]),
        ]);
        
        let matching_tuple = Value::Tuple(vec![
            Value::Integer(1),
            Value::Array(Rc::new(vec![
                Value::Integer(99),
                Value::String(Rc::new("test".to_string())),
            ])),
        ]);
        
        assert!(interpreter.pattern_matches(&nested_pattern, &matching_tuple).unwrap());
        
        let non_matching_tuple = Value::Tuple(vec![
            Value::Integer(3), // Not in or pattern
            Value::Array(Rc::new(vec![
                Value::Integer(99),
                Value::String(Rc::new("test".to_string())),
            ])),
        ]);
        
        assert!(!interpreter.pattern_matches(&nested_pattern, &non_matching_tuple).unwrap());
    }

    #[test]
    fn test_unimplemented_patterns() {
        let interpreter = create_test_interpreter();
        
        // Test that unimplemented patterns return false rather than crashing
        // This is a placeholder test - actual unimplemented patterns would need
        // to be added here based on the Pattern enum definition
        
        // For now, test that the catch-all works
        // (This would need actual unimplemented pattern variants to test properly)
    }

    // Test error propagation
    #[test]
    fn test_error_propagation() {
        let interpreter = create_test_interpreter();
        
        // Test that errors in nested pattern matching propagate correctly
        let nested_range = Pattern::Tuple(vec![
            Pattern::Range {
                start: Box::new(Pattern::Literal(Literal::String("invalid".to_string()))),
                end: Box::new(Pattern::Literal(Literal::Integer(5))),
                inclusive: true,
            },
        ]);
        
        let test_tuple = Value::Tuple(vec![Value::Integer(3)]);
        let result = interpreter.pattern_matches(&nested_range, &test_tuple);
        assert!(result.is_err());
    }
}

// Integration tests to verify refactoring preserved all functionality
#[cfg(test)]
mod pattern_matches_regression_tests {
    use super::*;

    #[test]
    fn test_complex_real_world_patterns() {
        let interpreter = create_test_interpreter();
        
        // Simulate a realistic pattern from match expressions
        let complex_pattern = Pattern::Or(vec![
            Pattern::Tuple(vec![
                Pattern::Literal(Literal::String("success".to_string())),
                Pattern::Identifier("value".to_string()),
            ]),
            Pattern::Tuple(vec![
                Pattern::Literal(Literal::String("error".to_string())),
                Pattern::Wildcard,
            ]),
        ]);
        
        // Test success case
        let success_value = Value::Tuple(vec![
            Value::String(Rc::new("success".to_string())),
            Value::Integer(42),
        ]);
        assert!(interpreter.pattern_matches(&complex_pattern, &success_value).unwrap());
        
        // Test error case
        let error_value = Value::Tuple(vec![
            Value::String(Rc::new("error".to_string())),
            Value::String(Rc::new("something went wrong".to_string())),
        ]);
        assert!(interpreter.pattern_matches(&complex_pattern, &error_value).unwrap());
        
        // Test non-matching case
        let invalid_value = Value::Tuple(vec![
            Value::String(Rc::new("unknown".to_string())),
            Value::Integer(42),
        ]);
        assert!(!interpreter.pattern_matches(&complex_pattern, &invalid_value).unwrap());
    }

    #[test]
    fn test_performance_regression() {
        let interpreter = create_test_interpreter();
        
        // Create a deeply nested pattern to test performance
        let mut nested_pattern = Pattern::Wildcard;
        for _ in 0..10 {
            nested_pattern = Pattern::Tuple(vec![nested_pattern]);
        }
        
        let mut nested_value = Value::Nil;
        for _ in 0..10 {
            nested_value = Value::Tuple(vec![nested_value]);
        }
        
        // Should complete quickly without stack overflow
        let start = std::time::Instant::now();
        let result = interpreter.pattern_matches(&nested_pattern, &nested_value);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100); // Should be very fast
    }
}