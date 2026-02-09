    use super::*;
    use std::sync::Arc;

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

    #[test]
    fn test_wildcard_pattern() {
        let pattern = Pattern::Wildcard;
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        assert!(result.expect("result should be Some in test").is_empty());
    }

    #[test]
    fn test_identifier_pattern() {
        let pattern = Pattern::Identifier("x".to_string());
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(42));
    }

    #[test]
    fn test_literal_pattern_match() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let value = Value::Integer(42);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
    }

    #[test]
    fn test_literal_pattern_no_match() {
        let pattern = Pattern::Literal(Literal::Integer(42, None));
        let value = Value::Integer(43);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_tuple_pattern() {
        let patterns = vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
        ];
        let pattern = Pattern::Tuple(patterns);
        let value = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(1));
        assert_eq!(bindings[1].0, "y");
        assert_eq!(bindings[1].1, Value::Integer(2));
    }

    #[test]
    fn test_range_pattern_inclusive() {
        let pattern = Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(5, None))),
            inclusive: true,
        };

        let value = Value::Integer(3);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());

        let value = Value::Integer(5);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());

        let value = Value::Integer(6);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_or_pattern() {
        let patterns = vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
            Pattern::Literal(Literal::Integer(3, None)),
        ];
        let pattern = Pattern::Or(patterns);

        let value = Value::Integer(2);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());

        let value = Value::Integer(4);
        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_some_pattern_match() {
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(42));
    }

    #[test]
    fn test_some_pattern_no_match_on_none() {
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("x".to_string())));
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_none_pattern_match() {
        let pattern = Pattern::None;
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "None".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_none_pattern_no_match_on_some() {
        let pattern = Pattern::None;
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_qualified_name_unit_variant_match() {
        let pattern = Pattern::QualifiedName(vec!["Status".to_string(), "Success".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Status".to_string(),
            variant_name: "Success".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert!(bindings.is_empty()); // Unit variants have no bindings
    }

    #[test]
    fn test_qualified_name_unit_variant_no_match_wrong_name() {
        let pattern = Pattern::QualifiedName(vec!["Status".to_string(), "Success".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Status".to_string(),
            variant_name: "Failed".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_qualified_name_no_match_on_tuple_variant() {
        let pattern = Pattern::QualifiedName(vec!["Response".to_string(), "Error".to_string()]);
        let value = Value::EnumVariant {
            enum_name: "Response".to_string(),
            variant_name: "Error".to_string(),
            data: Some(vec![Value::from_string("failed".to_string())]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none()); // Should not match - has data
    }

    #[test]
    fn test_tuple_variant_single_element() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Response".to_string(), "Error".to_string()],
            patterns: vec![Pattern::Identifier("msg".to_string())],
        };
        let value = Value::EnumVariant {
            enum_name: "Response".to_string(),
            variant_name: "Error".to_string(),
            data: Some(vec![Value::from_string("failed".to_string())]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].0, "msg");
        assert_eq!(bindings[0].1, Value::from_string("failed".to_string()));
    }

    #[test]
    fn test_tuple_variant_multiple_elements() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Point".to_string(), "Pos".to_string()],
            patterns: vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ],
        };
        let value = Value::EnumVariant {
            enum_name: "Point".to_string(),
            variant_name: "Pos".to_string(),
            data: Some(vec![Value::Integer(10), Value::Integer(20)]),
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_some());
        let bindings = result.expect("result should be Some in test");
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, "x");
        assert_eq!(bindings[0].1, Value::Integer(10));
        assert_eq!(bindings[1].0, "y");
        assert_eq!(bindings[1].1, Value::Integer(20));
    }

    #[test]
    fn test_tuple_variant_no_match_wrong_variant_name() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Message".to_string(), "Move".to_string()],
            patterns: vec![Pattern::Identifier("dir".to_string())],
        };
        let value = Value::EnumVariant {
            enum_name: "Message".to_string(),
            variant_name: "Quit".to_string(),
            data: None,
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

    #[test]
    fn test_tuple_variant_no_match_arity_mismatch() {
        let pattern = Pattern::TupleVariant {
            path: vec!["Point".to_string(), "Pos".to_string()],
            patterns: vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ],
        };
        let value = Value::EnumVariant {
            enum_name: "Point".to_string(),
            variant_name: "Pos".to_string(),
            data: Some(vec![Value::Integer(10)]), // Only 1 element, pattern expects 2
        };

        let result = try_pattern_match(&pattern, &value, &test_eval_literal)
            .expect("try_pattern_match should succeed in test");
        assert!(result.is_none());
    }

