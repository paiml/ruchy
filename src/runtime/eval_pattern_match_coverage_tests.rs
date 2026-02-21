
use super::*;
use std::sync::Arc;

fn test_eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Integer(i, _) => Value::Integer(*i),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::from_string(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Char(c) => Value::from_string(c.to_string()),
        Literal::Byte(b) => Value::Byte(*b),
        Literal::Unit => Value::Nil,
        Literal::Null => Value::Nil,
        Literal::Atom(s) => Value::from_string(s.clone()),
    }
}

// List pattern tests
#[test]
fn test_list_pattern_exact_match() {
    let patterns = vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ];
    let pattern = Pattern::List(patterns);
    let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].0, "a");
    assert_eq!(bindings[0].1, Value::Integer(1));
}

#[test]
fn test_list_pattern_wrong_length() {
    let patterns = vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ];
    let pattern = Pattern::List(patterns);
    let value = Value::Array(Arc::from(vec![Value::Integer(1)]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

#[test]
fn test_list_pattern_with_rest() {
    let patterns = vec![
        Pattern::Identifier("first".to_string()),
        Pattern::Rest,
        Pattern::Identifier("last".to_string()),
    ];
    let pattern = Pattern::List(patterns);
    let value = Value::Array(Arc::from(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
        Value::Integer(4),
    ]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].0, "first");
    assert_eq!(bindings[0].1, Value::Integer(1));
    assert_eq!(bindings[1].0, "last");
    assert_eq!(bindings[1].1, Value::Integer(4));
}

#[test]
fn test_list_pattern_with_named_rest() {
    let patterns = vec![
        Pattern::Identifier("head".to_string()),
        Pattern::RestNamed("tail".to_string()),
    ];
    let pattern = Pattern::List(patterns);
    let value = Value::Array(Arc::from(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].0, "head");
    assert_eq!(bindings[0].1, Value::Integer(1));
    assert_eq!(bindings[1].0, "tail");
    // tail should be [2, 3]
}

#[test]
fn test_list_pattern_rest_not_enough_elements() {
    let patterns = vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
        Pattern::Rest,
        Pattern::Identifier("c".to_string()),
    ];
    let pattern = Pattern::List(patterns);
    // Only 2 elements, but need at least 3 (a, b, c)
    let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

#[test]
fn test_list_pattern_on_non_array() {
    let patterns = vec![Pattern::Identifier("x".to_string())];
    let pattern = Pattern::List(patterns);
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// AtBinding pattern tests
#[test]
fn test_at_binding_pattern() {
    let pattern = Pattern::AtBinding {
        pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
        name: "x".to_string(),
    };
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "x");
    assert_eq!(bindings[0].1, Value::Integer(42));
}

#[test]
fn test_at_binding_pattern_no_match() {
    let pattern = Pattern::AtBinding {
        pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
        name: "x".to_string(),
    };
    let value = Value::Integer(43);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

#[test]
fn test_at_binding_with_inner_bindings() {
    let pattern = Pattern::AtBinding {
        pattern: Box::new(Pattern::Identifier("inner".to_string())),
        name: "outer".to_string(),
    };
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 2);
    // inner binding first, outer binding second
    assert_eq!(bindings[0].0, "inner");
    assert_eq!(bindings[1].0, "outer");
}

// Ok pattern tests
#[test]
fn test_ok_pattern_match() {
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("val".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "val");
    assert_eq!(bindings[0].1, Value::Integer(42));
}

#[test]
fn test_ok_pattern_no_match_on_err() {
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("val".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Err".to_string(),
        data: Some(vec![Value::from_string("error".to_string())]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// Err pattern tests
#[test]
fn test_err_pattern_match() {
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("err".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Err".to_string(),
        data: Some(vec![Value::from_string("error message".to_string())]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "err");
}

#[test]
fn test_err_pattern_no_match_on_ok() {
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("err".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// Struct pattern tests
#[test]
fn test_struct_pattern_match() {
    use std::collections::HashMap;
    let fields = vec![
        StructPatternField {
            name: "x".to_string(),
            pattern: Some(Pattern::Identifier("x_val".to_string())),
        },
        StructPatternField {
            name: "y".to_string(),
            pattern: Some(Pattern::Identifier("y_val".to_string())),
        },
    ];
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields,
        has_rest: false,
    };

    let mut struct_fields = HashMap::new();
    struct_fields.insert("x".to_string(), Value::Integer(10));
    struct_fields.insert("y".to_string(), Value::Integer(20));

    let value = Value::Struct {
        name: "Point".to_string(),
        fields: struct_fields.into(),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
    let bindings = result.expect("result should be Some");
    assert_eq!(bindings.len(), 2);
}

#[test]
fn test_struct_pattern_wrong_name() {
    let fields = vec![StructPatternField {
        name: "x".to_string(),
        pattern: Some(Pattern::Identifier("x_val".to_string())),
    }];
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields,
        has_rest: false,
    };

    let mut struct_fields = std::collections::HashMap::new();
    struct_fields.insert("x".to_string(), Value::Integer(10));

    let value = Value::Struct {
        name: "Vector".to_string(),
        fields: struct_fields.into(),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

#[test]
fn test_struct_pattern_missing_field() {
    let fields = vec![
        StructPatternField {
            name: "x".to_string(),
            pattern: Some(Pattern::Identifier("x_val".to_string())),
        },
        StructPatternField {
            name: "z".to_string(), // Field doesn't exist in value
            pattern: Some(Pattern::Identifier("z_val".to_string())),
        },
    ];
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields,
        has_rest: false,
    };

    let mut struct_fields = std::collections::HashMap::new();
    struct_fields.insert("x".to_string(), Value::Integer(10));
    struct_fields.insert("y".to_string(), Value::Integer(20));

    let value = Value::Struct {
        name: "Point".to_string(),
        fields: struct_fields.into(),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// Range pattern edge cases
#[test]
fn test_range_pattern_exclusive() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: false,
    };

    // Should not match 5 (exclusive)
    let value = Value::Integer(5);
    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());

    // Should match 4
    let value = Value::Integer(4);
    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
}

#[test]
fn test_range_pattern_at_start() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: true,
    };

    let value = Value::Integer(1);
    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
}

#[test]
fn test_range_pattern_before_start() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: true,
    };

    let value = Value::Integer(0);
    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// Pattern matching function tests
#[test]
fn test_pattern_matches_returns_bool() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(42);

    let result = pattern_matches(&pattern, &value, &test_eval_literal)
        .expect("pattern_matches should succeed");
    assert!(result);
}

#[test]
fn test_pattern_matches_returns_false() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(43);

    let result = pattern_matches(&pattern, &value, &test_eval_literal)
        .expect("pattern_matches should succeed");
    assert!(!result);
}

// Float literal pattern tests
#[test]
fn test_float_literal_pattern() {
    let pattern = Pattern::Literal(Literal::Float(3.14));
    let value = Value::Float(3.14);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
}

// String literal pattern tests
#[test]
fn test_string_literal_pattern() {
    let pattern = Pattern::Literal(Literal::String("hello".to_string()));
    let value = Value::from_string("hello".to_string());

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
}

// Bool literal pattern tests
#[test]
fn test_bool_literal_pattern() {
    let pattern = Pattern::Literal(Literal::Bool(true));
    let value = Value::Bool(true);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());

    let value = Value::Bool(false);
    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_none());
}

// Nil/Unit pattern tests
#[test]
fn test_nil_nil_match() {
    let pattern = Pattern::Literal(Literal::Unit);
    let value = Value::Nil;

    let result = try_pattern_match(&pattern, &value, &test_eval_literal)
        .expect("try_pattern_match should succeed");
    assert!(result.is_some());
}

// ============================================================================
// EXTREME TDD Round 131: Comprehensive pattern matching coverage tests
// Target: 89.11% → 95%+ coverage
// ============================================================================

// --- Wildcard pattern tests ---
#[test]
fn test_wildcard_pattern_matches_integer() {
    let pattern = Pattern::Wildcard;
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    assert!(result.unwrap().is_empty()); // No bindings
}

#[test]
fn test_wildcard_pattern_matches_string() {
    let pattern = Pattern::Wildcard;
    let value = Value::from_string("hello".to_string());

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_wildcard_pattern_matches_nil() {
    let pattern = Pattern::Wildcard;
    let value = Value::Nil;

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

// --- Identifier pattern tests ---
#[test]
fn test_identifier_pattern_binds_value() {
    let pattern = Pattern::Identifier("x".to_string());
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].0, "x");
    assert_eq!(bindings[0].1, Value::Integer(42));
}

#[test]
fn test_identifier_pattern_binds_array() {
    let pattern = Pattern::Identifier("arr".to_string());
    let value = Value::from_array(vec![Value::Integer(1), Value::Integer(2)]);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings[0].0, "arr");
}

// --- Tuple pattern tests ---
#[test]
fn test_tuple_pattern_empty() {
    let pattern = Pattern::Tuple(vec![]);
    let value = Value::Tuple(Arc::from(vec![]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_tuple_pattern_single_element() {
    let pattern = Pattern::Tuple(vec![Pattern::Identifier("x".to_string())]);
    let value = Value::Tuple(Arc::from(vec![Value::Integer(1)]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(1)));
}

#[test]
fn test_tuple_pattern_length_mismatch() {
    let pattern = Pattern::Tuple(vec![Pattern::Wildcard, Pattern::Wildcard]);
    let value = Value::Tuple(Arc::from(vec![Value::Integer(1)]));

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_tuple_pattern_wrong_type() {
    let pattern = Pattern::Tuple(vec![Pattern::Wildcard]);
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- List pattern tests ---
#[test]
fn test_list_pattern_empty() {
    let pattern = Pattern::List(vec![]);
    let value = Value::from_array(vec![]);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_list_pattern_single() {
    let pattern = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
    let value = Value::from_array(vec![Value::Integer(42)]);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
}

#[test]
fn test_list_pattern_length_mismatch() {
    let pattern = Pattern::List(vec![Pattern::Wildcard, Pattern::Wildcard]);
    let value = Value::from_array(vec![Value::Integer(1)]);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_list_pattern_wrong_type() {
    let pattern = Pattern::List(vec![Pattern::Wildcard]);
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Or pattern tests ---
#[test]
fn test_or_pattern_first_matches() {
    let pattern = Pattern::Or(vec![
        Pattern::Literal(Literal::Integer(1, None)),
        Pattern::Literal(Literal::Integer(2, None)),
    ]);
    let value = Value::Integer(1);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_or_pattern_second_matches() {
    let pattern = Pattern::Or(vec![
        Pattern::Literal(Literal::Integer(1, None)),
        Pattern::Literal(Literal::Integer(2, None)),
    ]);
    let value = Value::Integer(2);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_or_pattern_none_match() {
    let pattern = Pattern::Or(vec![
        Pattern::Literal(Literal::Integer(1, None)),
        Pattern::Literal(Literal::Integer(2, None)),
    ]);
    let value = Value::Integer(3);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_or_pattern_empty() {
    let pattern = Pattern::Or(vec![]);
    let value = Value::Integer(1);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Range pattern tests ---
#[test]
fn test_range_pattern_exclusive_in_range() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: false,
    };
    let value = Value::Integer(3);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_range_pattern_exclusive_at_end_fails() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: false,
    };
    let value = Value::Integer(5);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_range_pattern_inclusive_at_end_succeeds() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: true,
    };
    let value = Value::Integer(5);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_range_pattern_below_start() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: true,
    };
    let value = Value::Integer(0);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_range_pattern_wrong_type() {
    let pattern = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
        inclusive: true,
    };
    let value = Value::from_string("hello".to_string());

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- AtBinding pattern tests ---
#[test]
fn test_at_binding_literal_matches() {
    let pattern = Pattern::AtBinding {
        pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
        name: "x".to_string(),
    };
    let value = Value::Integer(42);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
}

#[test]
fn test_at_binding_literal_no_match() {
    let pattern = Pattern::AtBinding {
        pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
        name: "x".to_string(),
    };
    let value = Value::Integer(43);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Some/None pattern tests ---
#[test]
fn test_some_pattern_matches_some() {
    let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
}

#[test]
fn test_some_pattern_not_matches_none() {
    let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "None".to_string(),
        data: None,
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_none_pattern_matches_none() {
    let pattern = Pattern::None;
    let value = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "None".to_string(),
        data: None,
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_none_pattern_not_matches_some() {
    let pattern = Pattern::None;
    let value = Value::EnumVariant {
        enum_name: "Option".to_string(),
        variant_name: "Some".to_string(),
        data: Some(vec![Value::Integer(1)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Ok/Err pattern tests ---
#[test]
fn test_ok_pattern_matches_ok() {
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("x".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(bindings[0], ("x".to_string(), Value::Integer(42)));
}

#[test]
fn test_ok_pattern_not_matches_err() {
    let pattern = Pattern::Ok(Box::new(Pattern::Identifier("x".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Err".to_string(),
        data: Some(vec![Value::from_string("error".to_string())]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

#[test]
fn test_err_pattern_matches_err() {
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("e".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Err".to_string(),
        data: Some(vec![Value::from_string("error".to_string())]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
    let bindings = result.unwrap();
    assert_eq!(
        bindings[0],
        ("e".to_string(), Value::from_string("error".to_string()))
    );
}

#[test]
fn test_err_pattern_not_matches_ok() {
    let pattern = Pattern::Err(Box::new(Pattern::Identifier("e".to_string())));
    let value = Value::EnumVariant {
        enum_name: "Result".to_string(),
        variant_name: "Ok".to_string(),
        data: Some(vec![Value::Integer(42)]),
    };

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Char literal pattern test ---
#[test]
fn test_char_literal_pattern() {
    let pattern = Pattern::Literal(Literal::Char('a'));
    let value = Value::from_string("a".to_string());

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_char_literal_pattern_no_match() {
    let pattern = Pattern::Literal(Literal::Char('a'));
    let value = Value::from_string("b".to_string());

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Byte literal pattern test ---
#[test]
fn test_byte_literal_pattern() {
    let pattern = Pattern::Literal(Literal::Byte(255));
    let value = Value::Byte(255);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

#[test]
fn test_byte_literal_pattern_no_match() {
    let pattern = Pattern::Literal(Literal::Byte(255));
    let value = Value::Byte(254);

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_none());
}

// --- Null pattern test ---
#[test]
fn test_null_literal_pattern() {
    let pattern = Pattern::Literal(Literal::Null);
    let value = Value::Nil;

    let result = try_pattern_match(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result.is_some());
}

// --- pattern_matches helper test ---
#[test]
fn test_pattern_matches_helper_true() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(42);

    let result = pattern_matches(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(result);
}

#[test]
fn test_pattern_matches_helper_false() {
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    let value = Value::Integer(43);

    let result = pattern_matches(&pattern, &value, &test_eval_literal).expect("should succeed");
    assert!(!result);
}
