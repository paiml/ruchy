use super::*;
use proptest::prelude::*;

proptest! {
    /// Property: AstBuilder::new() never panics
    #[test]
    fn test_ast_builder_new_never_panics(_input: String) {
        let _ = AstBuilder::new();
    }

    /// Property: Integer literals preserve their values
    #[test]
    fn test_int_literal_roundtrip(value: i64) {
        let builder = AstBuilder::new();
        let expr = builder.int(value);

        if let ExprKind::Literal(Literal::Integer(actual, None)) = expr.kind {
            prop_assert_eq!(actual, value);
        } else {
            prop_assert!(false, "Expected integer literal");
        }
    }

    /// Property: Float literals preserve their values (excluding NaN)
    #[test]
    fn test_float_literal_roundtrip(value in prop::num::f64::ANY.prop_filter("exclude NaN", |x| !x.is_nan())) {
        let builder = AstBuilder::new();
        let expr = builder.float(value);

        if let ExprKind::Literal(Literal::Float(actual)) = expr.kind {
            if value.is_infinite() {
                prop_assert!(actual.is_infinite());
                prop_assert_eq!(actual.is_sign_positive(), value.is_sign_positive());
            } else {
                prop_assert!((actual - value).abs() < f64::EPSILON);
            }
        } else {
            prop_assert!(false, "Expected float literal");
        }
    }

    /// Property: String literals preserve their content
    #[test]
    fn test_string_literal_roundtrip(value: String) {
        let builder = AstBuilder::new();
        let expr = builder.string(&value);

        if let ExprKind::Literal(Literal::String(actual)) = expr.kind {
            prop_assert_eq!(actual, value);
        } else {
            prop_assert!(false, "Expected string literal");
        }
    }

    /// Property: Boolean literals preserve their values
    #[test]
    fn test_bool_literal_roundtrip(value: bool) {
        let builder = AstBuilder::new();
        let expr = builder.bool(value);

        if let ExprKind::Literal(Literal::Bool(actual)) = expr.kind {
            prop_assert_eq!(actual, value);
        } else {
            prop_assert!(false, "Expected boolean literal");
        }
    }

    /// Property: Identifier names are preserved
    #[test]
    fn test_identifier_roundtrip(name in "[a-zA-Z_][a-zA-Z0-9_]*") {
        let builder = AstBuilder::new();
        let expr = builder.ident(&name);

        if let ExprKind::Identifier(actual) = expr.kind {
            prop_assert_eq!(actual, name);
        } else {
            prop_assert!(false, "Expected identifier");
        }
    }

    /// Property: Binary operations preserve operands and operator
    #[test]
    fn test_binary_operation_structure(left: i64, right: i64) {
        let builder = AstBuilder::new();
        let expr = builder.binary(
            builder.int(left),
            BinaryOp::Add,
            builder.int(right),
        );

        if let ExprKind::Binary { left: l, op, right: r } = expr.kind {
            prop_assert!(matches!(op, BinaryOp::Add));

            if let ExprKind::Literal(Literal::Integer(l_val, None)) = l.kind {
                prop_assert_eq!(l_val, left);
            } else {
                prop_assert!(false, "Expected left operand to be integer");
            }

            if let ExprKind::Literal(Literal::Integer(r_val, None)) = r.kind {
                prop_assert_eq!(r_val, right);
            } else {
                prop_assert!(false, "Expected right operand to be integer");
            }
        } else {
            prop_assert!(false, "Expected binary expression");
        }
    }

    /// Property: List construction preserves element count
    #[test]
    fn test_list_element_count(elements: Vec<i64>) {
        let builder = AstBuilder::new();
        let expr_elements: Vec<_> = elements.iter().map(|&x| builder.int(x)).collect();
        let list_expr = builder.list(expr_elements);

        if let ExprKind::List(actual_elements) = list_expr.kind {
            prop_assert_eq!(actual_elements.len(), elements.len());

            for (i, &expected) in elements.iter().enumerate() {
                if let ExprKind::Literal(Literal::Integer(actual, None)) = actual_elements[i].kind {
                    prop_assert_eq!(actual, expected);
                } else {
                    prop_assert!(false, "Expected integer literal at index {}", i);
                }
            }
        } else {
            prop_assert!(false, "Expected list expression");
        }
    }

    /// Property: Tuple construction preserves element count
    #[test]
    fn test_tuple_element_count(elements: Vec<i64>) {
        let builder = AstBuilder::new();
        let expr_elements: Vec<_> = elements.iter().map(|&x| builder.int(x)).collect();
        let tuple_expr = builder.tuple(expr_elements);

        if let ExprKind::Tuple(actual_elements) = tuple_expr.kind {
            prop_assert_eq!(actual_elements.len(), elements.len());
        } else {
            prop_assert!(false, "Expected tuple expression");
        }
    }

    /// Property: Pattern or-patterns preserve sub-pattern count
    #[test]
    fn test_pattern_or_count(values: Vec<i64>) {
        let builder = AstBuilder::new();
        let patterns: Vec<_> = values.iter().map(|&x| {
            builder.pattern_literal(Literal::Integer(x, None))
        }).collect();

        let or_pattern = builder.pattern_or(patterns);

        if let Pattern::Or(actual_patterns) = or_pattern {
            prop_assert_eq!(actual_patterns.len(), values.len());
        } else {
            prop_assert!(false, "Expected or pattern");
        }
    }

    /// Property: All expressions have empty attributes by default
    #[test]
    fn test_default_attributes(value: i64) {
        let builder = AstBuilder::new();
        let expressions = vec![
            builder.int(value),
            builder.bool(true),
            builder.string("test"),
            builder.ident("var"),
        ];

        for expr in expressions {
            prop_assert!(expr.attributes.is_empty());
        }
    }

    /// Property: All expressions have default span
    #[test]
    fn test_default_span(value: i64) {
        let builder = AstBuilder::new();
        let expressions = vec![
            builder.int(value),
            builder.bool(true),
            builder.string("test"),
            builder.ident("var"),
        ];

        for expr in expressions {
            prop_assert_eq!(expr.span, Span::default());
        }
    }
}
