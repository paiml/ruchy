//! Fast fuzz test for pattern matching - ensure no panics
//!
//! Tests that pattern matching is robust against random inputs

use ruchy::frontend::ast::{Literal, Pattern};
use ruchy::runtime::{eval_pattern_match::try_pattern_match, InterpreterError, Value};

fn test_eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Integer(i, _) => Value::Integer(*i),
        Literal::String(s) => Value::from_string(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        _ => Value::Nil,
    }
}

#[test]
fn fuzz_pattern_never_panics_basic() {
    // Test various pattern/value combinations don't panic
    let patterns = vec![
        Pattern::Wildcard,
        Pattern::Identifier("x".to_string()),
        Pattern::Literal(Literal::Integer(42, None)),
        Pattern::QualifiedName(vec!["Type".to_string(), "Variant".to_string()]),
        Pattern::Tuple(vec![Pattern::Wildcard, Pattern::Wildcard]),
        Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
        ]),
    ];

    let values = vec![
        Value::Integer(42),
        Value::Integer(-999),
        Value::Integer(i64::MAX),
        Value::Integer(i64::MIN),
        Value::Bool(true),
        Value::Bool(false),
        Value::from_string("test".to_string()),
        Value::from_string("".to_string()),
        Value::Nil,
        Value::Tuple(std::sync::Arc::from(vec![])),
        Value::Tuple(std::sync::Arc::from(vec![Value::Integer(1)])),
        Value::Array(std::sync::Arc::from(vec![])),
        Value::EnumVariant {
            variant_name: "Success".to_string(),
            data: None,
        },
        Value::EnumVariant {
            variant_name: "Error".to_string(),
            data: Some(vec![Value::from_string("msg".to_string())]),
        },
    ];

    let mut tested = 0;
    for pattern in &patterns {
        for value in &values {
            // Should never panic
            let _ = try_pattern_match(pattern, value, &test_eval_literal);
            tested += 1;
        }
    }

    println!("Fuzz tested {} pattern/value combinations without panic", tested);
    assert!(tested > 0);
}

#[test]
fn fuzz_nested_enum_robustness() {
    // Test tuple variant matching with various data shapes
    let pattern = Pattern::TupleVariant {
        path: vec!["Token".to_string(), "Char".to_string()],
        patterns: vec![Pattern::Identifier("ch".to_string())],
    };

    let test_values = vec![
        // Correct: matching variant with 1 element
        Value::EnumVariant {
            variant_name: "Char".to_string(),
            data: Some(vec![Value::from_string("a".to_string())]),
        },
        // Wrong arity: 0 elements
        Value::EnumVariant {
            variant_name: "Char".to_string(),
            data: Some(vec![]),
        },
        // Wrong arity: 2 elements
        Value::EnumVariant {
            variant_name: "Char".to_string(),
            data: Some(vec![
                Value::from_string("a".to_string()),
                Value::Integer(1),
            ]),
        },
        // No data
        Value::EnumVariant {
            variant_name: "Char".to_string(),
            data: None,
        },
        // Wrong variant name
        Value::EnumVariant {
            variant_name: "EOF".to_string(),
            data: None,
        },
        // Not an enum at all
        Value::Integer(42),
    ];

    for value in test_values {
        // Should not panic, just return Some or None
        let result = try_pattern_match(&pattern, &value, &test_eval_literal);
        assert!(result.is_ok());
    }
}
