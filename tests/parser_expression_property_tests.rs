// Property-based tests for parser expression handling
// PROPTEST-003 Part 1: Expression parsing properties (10 tests)
//
// Properties tested:
// 1. Literal parsing preserves values (int, float, bool)
// 2. Binary operators respect precedence
// 3. Unary operators bind correctly
// 4. Parentheses override precedence
// 5. Nested expressions balance correctly
// 6. String literals preserve content
// 7. Array literals preserve order
// 8. Tuple literals preserve arity
// 9. Range expressions parse correctly
// 10. Valid identifiers always parse

use proptest::prelude::*;
use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property 1: Literal parsing preserves values
// ============================================================================

proptest! {
    #[test]
    fn prop_integer_literal_parsing_preserves_value(n in 0i64..1000i64) {
        // NOTE: Negative literals are parsed as Unary(Negate, Literal(positive))
        // This is expected parser behavior, so we test only non-negative integers here
        let code = format!("{}", n);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse integer literal: {}", n);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Integer(parsed)) = expr.kind {
                prop_assert_eq!(parsed, n, "Integer literal mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected integer literal, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_float_literal_parsing_preserves_value(n in 0.1f64..1000.0f64) {
        // NOTE: Use explicit .1 precision to ensure decimal point in format
        // Avoid 0.0 which formats as "0" and parses as integer
        let code = format!("{:.1}", n);  // Force decimal point
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse float literal: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Float(parsed)) = expr.kind {
                // Use approximate equality for floats
                prop_assert!((parsed - n).abs() < 0.1, "Float literal mismatch: {} vs {}", parsed, n);
            } else {
                return Err(TestCaseError::fail(format!("Expected float literal for {}, got {:?}", code, expr.kind)));
            }
        }
    }

    #[test]
    fn prop_bool_literal_parsing_preserves_value(b: bool) {
        let code = format!("{}", b);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse boolean literal: {}", b);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Bool(parsed)) = expr.kind {
                prop_assert_eq!(parsed, b, "Boolean literal mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected boolean literal, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 2: Binary operators respect precedence
// ============================================================================

proptest! {
    #[test]
    fn prop_addition_has_lower_precedence_than_multiplication(a in 1i64..100, b in 1i64..100, c in 1i64..100) {
        // Parse: a + b * c
        // Should be: a + (b * c), not (a + b) * c
        let code = format!("{} + {} * {}", a, b, c);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse expression: {}", code);
        if let Ok(expr) = result {
            // Verify structure is Binary(Add, a, Binary(Mul, b, c))
            if let ExprKind::Binary { op: BinaryOp::Add, right, .. } = expr.kind {
                if let ExprKind::Binary { op: BinaryOp::Multiply, .. } = right.kind {
                    // Correct precedence
                } else {
                    return Err(TestCaseError::fail("Multiplication not evaluated before addition"));
                }
            } else {
                return Err(TestCaseError::fail(format!("Expected addition at top level, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_comparison_has_lower_precedence_than_arithmetic(a in 1i64..100, b in 1i64..100, c in 1i64..100) {
        // Parse: a + b < c
        // Should be: (a + b) < c, not a + (b < c)
        let code = format!("{} + {} < {}", a, b, c);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse comparison: {}", code);
        if let Ok(expr) = result {
            // Verify structure is Binary(Less, Binary(Add, a, b), c)
            if let ExprKind::Binary { op: BinaryOp::Less, left, .. } = expr.kind {
                if let ExprKind::Binary { op: BinaryOp::Add, .. } = left.kind {
                    // Correct precedence
                } else {
                    return Err(TestCaseError::fail("Addition not evaluated before comparison"));
                }
            } else {
                return Err(TestCaseError::fail(format!("Expected comparison at top level, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 3: Unary operators bind correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_unary_negation_binds_tightly(n in 1i64..1000) {
        let code = format!("-{}", n);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse unary negation: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Unary { .. } = expr.kind {
                // Correct - unary operator parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected unary expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_logical_not_binds_tightly(b: bool) {
        let code = format!("!{}", b);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse logical not: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Unary { .. } = expr.kind {
                // Correct - unary operator parsed
            } else {
                return Err(TestCaseError::fail(format!("Expected unary expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 4: Parentheses override precedence
// ============================================================================

proptest! {
    #[test]
    fn prop_parentheses_override_multiplication_precedence(a in 1i64..100, b in 1i64..100, c in 1i64..100) {
        // Parse: (a + b) * c
        // Should force addition to happen first
        let code = format!("({} + {}) * {}", a, b, c);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse parenthesized expression: {}", code);
        if let Ok(expr) = result {
            // Verify structure is Binary(Mul, Binary(Add, a, b), c)
            if let ExprKind::Binary { op: BinaryOp::Multiply, left, .. } = expr.kind {
                if let ExprKind::Binary { op: BinaryOp::Add, .. } = left.kind {
                    // Correct - parentheses forced order
                } else {
                    return Err(TestCaseError::fail("Parentheses did not override precedence"));
                }
            } else {
                return Err(TestCaseError::fail(format!("Expected multiplication at top level, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 5: Nested expressions balance correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_deeply_nested_parentheses_balance(depth in 1usize..10) {
        // Generate ((((42))))
        let mut code = "42".to_string();
        for _ in 0..depth {
            code = format!("({})", code);
        }

        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse nested expression with depth {}: {}", depth, code);

        // Verify we can extract the inner literal
        if let Ok(expr) = result {
            let inner = extract_innermost(&expr);
            if let ExprKind::Literal(Literal::Integer(42)) = inner.kind {
                // Correct - nested structure preserved
            } else {
                return Err(TestCaseError::fail(format!("Lost inner value in nesting, got {:?}", inner.kind)));
            }
        }
    }
}

// Helper: Extract innermost expression from nested structure
fn extract_innermost(expr: &Expr) -> &Expr {
    expr // Parentheses don't create nodes in our AST, so this is trivial
}

// ============================================================================
// Property 6: String literals preserve content
// ============================================================================

proptest! {
    #[test]
    fn prop_string_literals_preserve_content(s in "[a-zA-Z0-9 ]{1,20}") {
        let code = format!("\"{}\"", s);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse string literal: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::String(parsed)) = expr.kind {
                prop_assert_eq!(parsed, s, "String literal content mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected string literal, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 7: Array literals preserve order
// ============================================================================

proptest! {
    #[test]
    fn prop_array_literals_preserve_element_order(
        elem1 in 1i64..100,
        elem2 in 1i64..100,
        elem3 in 1i64..100
    ) {
        let code = format!("[{}, {}, {}]", elem1, elem2, elem3);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse array literal: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::List(elements) = expr.kind {
                prop_assert_eq!(elements.len(), 3, "Array should have 3 elements");

                // Verify order
                if let ExprKind::Literal(Literal::Integer(v1)) = elements[0].kind {
                    prop_assert_eq!(v1, elem1, "First element mismatch");
                }
                if let ExprKind::Literal(Literal::Integer(v2)) = elements[1].kind {
                    prop_assert_eq!(v2, elem2, "Second element mismatch");
                }
                if let ExprKind::Literal(Literal::Integer(v3)) = elements[2].kind {
                    prop_assert_eq!(v3, elem3, "Third element mismatch");
                }
            } else {
                return Err(TestCaseError::fail(format!("Expected list literal, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 8: Tuple literals preserve arity
// ============================================================================

proptest! {
    #[test]
    fn prop_tuple_literals_preserve_arity(
        elem1 in 1i64..100,
        elem2 in 1i64..100
    ) {
        let code = format!("({}, {})", elem1, elem2);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse tuple literal: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Tuple(elements) = expr.kind {
                prop_assert_eq!(elements.len(), 2, "Tuple should have 2 elements");
            } else {
                return Err(TestCaseError::fail(format!("Expected tuple literal, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 9: Range expressions parse correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_exclusive_range_parses(start in 1i64..100, end in 101i64..200) {
        let code = format!("{}..{}", start, end);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse range: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Range { inclusive, .. } = expr.kind {
                prop_assert!(!inclusive, "Range should be exclusive");
            } else {
                return Err(TestCaseError::fail(format!("Expected range expression, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_inclusive_range_parses(start in 1i64..100, end in 101i64..200) {
        let code = format!("{}..={}", start, end);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse inclusive range: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Range { inclusive, .. } = expr.kind {
                prop_assert!(inclusive, "Range should be inclusive");
            } else {
                return Err(TestCaseError::fail(format!("Expected range expression, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 10: Valid identifiers always parse
// ============================================================================

proptest! {
    #[test]
    fn prop_valid_identifiers_parse(
        first in "[a-zA-Z_]",
        rest in "[a-zA-Z0-9_]{0,19}"
    ) {
        let ident = format!("{}{}", first, rest);

        // Skip reserved keywords
        if is_reserved_keyword(&ident) {
            return Ok(());
        }

        let mut parser = Parser::new(&ident);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse valid identifier: {}", ident);
        if let Ok(expr) = result {
            if let ExprKind::Identifier(name) = expr.kind {
                prop_assert_eq!(name, ident, "Identifier name mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected identifier, got {:?}", expr.kind)));
            }
        }
    }
}

// Helper: Check if identifier is a reserved keyword
fn is_reserved_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "let"
            | "mut"
            | "fn"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
            | "match"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "pub"
            | "use"
            | "true"
            | "false"
            | "nil"
            | "null"
    )
}
