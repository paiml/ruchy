
use super::*;
use proptest::prelude::*;

fn test_eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Integer(i, _) => Value::Integer(*i),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::from_string(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Unit => Value::Nil,
        _ => Value::Nil,
    }
}

proptest! {
    /// Property: Wildcard pattern always matches any value
    #[test]
    fn prop_wildcard_always_matches(value: i64) {
        let pattern = Pattern::Wildcard;
        let val = Value::Integer(value);
        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_some());
        prop_assert_eq!(
            result.expect("result should be Some in test").len(),
            0
        ); // No bindings
    }

    /// Property: Identifier pattern always matches and binds
    #[test]
    fn prop_identifier_always_binds(name in "[a-z]{1,10}", value: i64) {
        let pattern = Pattern::Identifier(name.clone());
        let val = Value::Integer(value);
        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        prop_assert_eq!(bindings.len(), 1);
        prop_assert_eq!(&bindings[0].0, &name);
        prop_assert_eq!(&bindings[0].1, &Value::Integer(value));
    }

    /// Property: Literal pattern matches only exact values
    #[test]
    fn prop_literal_exact_match(target: i64, test: i64) {
        let pattern = Pattern::Literal(Literal::Integer(target, None));
        let val = Value::Integer(test);
        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");

        if target == test {
            prop_assert!(result.is_some());
        } else {
            prop_assert!(result.is_none());
        }
    }

    /// Property: Tuple pattern matches only correct arity
    #[test]
    fn prop_tuple_arity_must_match(size in 1usize..5) {
        let patterns: Vec<Pattern> = (0..size)
            .map(|i| Pattern::Identifier(format!("x{i}")))
            .collect();
        let pattern = Pattern::Tuple(patterns);

        // Correct arity - should match
        let values: Vec<Value> = (0..size).map(|i| Value::Integer(i as i64)).collect();
        let val = Value::Tuple(std::sync::Arc::from(values));
        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_some());

        // Wrong arity (one less) - should not match
        if size > 1 {
            let wrong_values: Vec<Value> = (0..size-1).map(|i| Value::Integer(i as i64)).collect();
            let wrong_val = Value::Tuple(std::sync::Arc::from(wrong_values));
            let result = try_pattern_match(&pattern, &wrong_val, &test_eval_literal)
                .expect("try_pattern_match should succeed in test");
            prop_assert!(result.is_none());
        }
    }

    /// Property: EnumVariant unit pattern matches only unit variants
    #[test]
    fn prop_unit_variant_no_data(variant_name in "[A-Z][a-z]{1,10}") {
        let pattern = Pattern::QualifiedName(vec!["Enum".to_string(), variant_name.clone()]);

        // Unit variant (no data) - should match
        let val_unit = Value::EnumVariant {
            enum_name: "Enum".to_string(),
            variant_name: variant_name.clone(),
            data: None,
        };
        let result = try_pattern_match(&pattern, &val_unit, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_some());

        // Tuple variant (with data) - should NOT match
        let val_tuple = Value::EnumVariant {
            enum_name: "Enum".to_string(),
            variant_name,
            data: Some(vec![Value::Integer(42)]),
        };
        let result = try_pattern_match(&pattern, &val_tuple, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_none());
    }

    /// Property: TupleVariant pattern binds all elements
    #[test]
    fn prop_tuple_variant_binds_all(count in 1usize..4, values in prop::collection::vec(any::<i64>(), 1..4)) {
        let count = count.min(values.len());
        let patterns: Vec<Pattern> = (0..count)
            .map(|i| Pattern::Identifier(format!("x{i}")))
            .collect();

        let pattern = Pattern::TupleVariant {
            path: vec!["Type".to_string(), "Variant".to_string()],
            patterns,
        };

        let variant_values: Vec<Value> = values[..count].iter()
            .map(|&v| Value::Integer(v))
            .collect();

        let val = Value::EnumVariant {
            enum_name: "Type".to_string(),
            variant_name: "Variant".to_string(),
            data: Some(variant_values),
        };

        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        prop_assert!(result.is_some());

        let bindings = result.expect("result should be Some in test");
        prop_assert_eq!(bindings.len(), count);

        for i in 0..count {
            prop_assert_eq!(&bindings[i].0, &format!("x{i}"));
            prop_assert_eq!(&bindings[i].1, &Value::Integer(values[i]));
        }
    }

    /// Property: Or pattern matches if ANY subpattern matches
    #[test]
    fn prop_or_pattern_any_match(target: i64, options in prop::collection::vec(any::<i64>(), 1..5)) {
        let patterns: Vec<Pattern> = options.iter()
            .map(|&v| Pattern::Literal(Literal::Integer(v, None)))
            .collect();

        let pattern = Pattern::Or(patterns);
        let val = Value::Integer(target);
        let result = try_pattern_match(&pattern, &val, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");

        if options.contains(&target) {
            prop_assert!(result.is_some());
        } else {
            prop_assert!(result.is_none());
        }
    }

    /// Property: Pattern matching never panics (robustness)
    #[test]
    fn prop_never_panics(value: i64) {
        let patterns = vec![
            Pattern::Wildcard,
            Pattern::Identifier("x".to_string()),
            Pattern::Literal(Literal::Integer(value, None)),
            Pattern::QualifiedName(vec!["Type".to_string(), "Variant".to_string()]),
        ];

        let val = Value::Integer(value);

        for pattern in patterns {
            // Should not panic
            let _ = try_pattern_match(&pattern, &val, &test_eval_literal);
        }
    }
}
