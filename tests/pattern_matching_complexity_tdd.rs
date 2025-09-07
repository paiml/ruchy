//! TDD Tests for Pattern Matching Complexity Reduction
//! 
//! These tests drive the implementation of complexity reduction from 25 → ≤10
//! Following strict TDD: Red -> Green -> Refactor

use ruchy::runtime::pattern_matching::match_pattern;
use ruchy::frontend::ast::{Pattern, Literal};
use ruchy::runtime::Value;

#[test]
fn test_match_wildcard_pattern() {
    let pattern = Pattern::Wildcard;
    let value = Value::Int(42);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Wildcard should match any value");
    assert_eq!(result.unwrap(), vec![], "Wildcard should return empty bindings");
}

#[test]
fn test_match_identifier_pattern() {
    let pattern = Pattern::Identifier("x".to_string());
    let value = Value::String("hello".to_string());
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Identifier should match any value");
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1, "Should have one binding");
    assert_eq!(bindings[0].0, "x", "Should bind to identifier name");
}

#[test]
fn test_match_literal_pattern_success() {
    let pattern = Pattern::Literal(Literal::Integer(42));
    let value = Value::Int(42);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Matching literals should succeed");
    assert_eq!(result.unwrap(), vec![], "Literal match should return empty bindings");
}

#[test]
fn test_match_literal_pattern_failure() {
    let pattern = Pattern::Literal(Literal::Integer(42));
    let value = Value::Int(24);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_none(), "Non-matching literals should fail");
}

#[test]
fn test_match_tuple_pattern_success() {
    let patterns = vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Literal(Literal::Integer(42))
    ];
    let pattern = Pattern::Tuple(patterns);
    let values = vec![Value::String("hello".to_string()), Value::Int(42)];
    let value = Value::Tuple(values);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Matching tuple should succeed");
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1, "Should have one binding from identifier");
    assert_eq!(bindings[0].0, "x", "Should bind identifier");
}

#[test]
fn test_match_list_pattern_success() {
    let patterns = vec![
        Pattern::Identifier("first".to_string()),
        Pattern::Identifier("second".to_string())
    ];
    let pattern = Pattern::List(patterns);
    let values = vec![Value::Int(1), Value::Int(2)];
    let value = Value::List(values);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Matching list should succeed");
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 2, "Should have two bindings");
}

#[test]
fn test_match_or_pattern_success() {
    let patterns = vec![
        Pattern::Literal(Literal::Integer(1)),
        Pattern::Literal(Literal::Integer(42))
    ];
    let pattern = Pattern::Or(patterns);
    let value = Value::Int(42);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Or pattern should match second alternative");
}

#[test]
fn test_match_or_pattern_failure() {
    let patterns = vec![
        Pattern::Literal(Literal::Integer(1)),
        Pattern::Literal(Literal::Integer(2))
    ];
    let pattern = Pattern::Or(patterns);
    let value = Value::Int(42);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_none(), "Or pattern should fail when no alternatives match");
}

#[test]
fn test_match_range_pattern_inclusive_success() {
    let start = Box::new(Pattern::Literal(Literal::Integer(1)));
    let end = Box::new(Pattern::Literal(Literal::Integer(10)));
    let pattern = Pattern::Range { start, end, inclusive: true };
    let value = Value::Int(5);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Value within inclusive range should match");
}

#[test]
fn test_match_range_pattern_exclusive_boundary() {
    let start = Box::new(Pattern::Literal(Literal::Integer(1)));
    let end = Box::new(Pattern::Literal(Literal::Integer(10)));
    let pattern = Pattern::Range { start, end, inclusive: false };
    let value = Value::Int(10);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_none(), "End boundary should not match in exclusive range");
}

#[test]
fn test_match_rest_pattern() {
    let pattern = Pattern::Rest;
    let value = Value::String("anything".to_string());
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Rest pattern should match anything");
    assert_eq!(result.unwrap(), vec![], "Rest pattern should return empty bindings");
}

#[test]
fn test_match_rest_named_pattern() {
    let pattern = Pattern::RestNamed("rest".to_string());
    let value = Value::List(vec![Value::Int(1), Value::Int(2)]);
    
    let result = match_pattern(&pattern, &value);
    assert!(result.is_some(), "Named rest pattern should match anything");
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1, "Should have one binding");
    assert_eq!(bindings[0].0, "rest", "Should bind to rest name");
}

#[test]
fn test_complexity_reduction_all_patterns() {
    // This test ensures all pattern types work after refactoring
    // Testing that complexity reduction doesn't break functionality
    
    // Test multiple pattern types in sequence
    let patterns = vec![
        (Pattern::Wildcard, Value::Int(1)),
        (Pattern::Identifier("x".to_string()), Value::String("test".to_string())),
        (Pattern::Rest, Value::Bool(true)),
        (Pattern::RestNamed("data".to_string()), Value::Float(3.14))
    ];
    
    for (pattern, value) in patterns {
        let result = match_pattern(&pattern, &value);
        assert!(result.is_some(), "All basic patterns should work after refactoring");
    }
}

#[test]
fn test_helper_function_isolation() {
    // Test that helper functions work independently
    // This ensures our complexity reduction maintains correctness
    
    let literal_pattern = Pattern::Literal(Literal::String("test".to_string()));
    let literal_value = Value::String("test".to_string());
    let literal_result = match_pattern(&literal_pattern, &literal_value);
    assert!(literal_result.is_some(), "Literal helper should work");
    
    let tuple_pattern = Pattern::Tuple(vec![Pattern::Wildcard]);
    let tuple_value = Value::Tuple(vec![Value::Int(42)]);
    let tuple_result = match_pattern(&tuple_pattern, &tuple_value);
    assert!(tuple_result.is_some(), "Tuple helper should work");
    
    let list_pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
    let list_value = Value::List(vec![Value::Bool(true)]);
    let list_result = match_pattern(&list_pattern, &list_value);
    assert!(list_result.is_some(), "List helper should work");
}