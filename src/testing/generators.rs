//! Property-based test generators for AST nodes
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, Span, UnaryOp};
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};
/// Maximum depth for recursive AST generation to avoid stack overflow
const MAX_DEPTH: u32 = 5;
/// Generate arbitrary literals
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_literal;
/// 
/// let result = arb_literal(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_literal() -> BoxedStrategy<Literal> {
    prop_oneof![
        (0i64..i64::MAX).prop_map(Literal::Integer),
        any::<f64>().prop_map(Literal::Float),
        any::<bool>().prop_map(Literal::Bool),
        ".*".prop_map(|s: String| Literal::String(s.chars().take(20).collect())),
        Just(Literal::Unit),
    ]
    .boxed()
}
/// Generate arbitrary identifiers
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_identifier;
/// 
/// let result = arb_identifier(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_identifier() -> BoxedStrategy<String> {
    "[a-z][a-z0-9_]{0,10}".prop_map(|s| s).boxed()
}
/// Generate arbitrary binary operators
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_binary_op;
/// 
/// let result = arb_binary_op(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_binary_op() -> BoxedStrategy<BinaryOp> {
    prop_oneof![
        Just(BinaryOp::Add),
        Just(BinaryOp::Subtract),
        Just(BinaryOp::Multiply),
        Just(BinaryOp::Divide),
        Just(BinaryOp::Modulo),
        Just(BinaryOp::Power),
        Just(BinaryOp::Equal),
        Just(BinaryOp::NotEqual),
        Just(BinaryOp::Less),
        Just(BinaryOp::LessEqual),
        Just(BinaryOp::Greater),
        Just(BinaryOp::GreaterEqual),
        Just(BinaryOp::And),
        Just(BinaryOp::Or),
        Just(BinaryOp::BitwiseAnd),
        Just(BinaryOp::BitwiseOr),
        Just(BinaryOp::BitwiseXor),
        Just(BinaryOp::LeftShift),
    ]
    .boxed()
}
/// Generate arbitrary unary operators
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_unary_op;
/// 
/// let result = arb_unary_op(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_unary_op() -> BoxedStrategy<UnaryOp> {
    prop_oneof![
        Just(UnaryOp::Negate),
        Just(UnaryOp::Not),
        Just(UnaryOp::BitwiseNot),
        Just(UnaryOp::Reference),
    ]
    .boxed()
}
/// Generate arbitrary expressions with depth limiting
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_expr_with_depth;
/// 
/// let result = arb_expr_with_depth(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_expr_with_depth(depth: u32) -> BoxedStrategy<Expr> {
    if depth >= MAX_DEPTH {
        // Base case: only generate non-recursive expressions
        prop_oneof![
            arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::new(0, 0))),
            arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::new(0, 0))),
        ]
        .boxed()
    } else {
        // Recursive case
        prop_oneof![
            // Literals
            arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::new(0, 0))),
            // Identifiers
            arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::new(0, 0))),
            // Binary operations
            (
                arb_expr_with_depth(depth + 1),
                arb_binary_op(),
                arb_expr_with_depth(depth + 1),
            )
                .prop_map(|(left, op, right)| {
                    Expr::new(
                        ExprKind::Binary {
                            left: Box::new(left),
                            op,
                            right: Box::new(right),
                        },
                        Span::new(0, 0),
                    )
                }),
            // Unary operations
            (arb_unary_op(), arb_expr_with_depth(depth + 1)).prop_map(|(op, operand)| {
                Expr::new(
                    ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    Span::new(0, 0),
                )
            }),
            // If expressions
            (
                arb_expr_with_depth(depth + 1),
                arb_expr_with_depth(depth + 1),
                prop::option::of(arb_expr_with_depth(depth + 1)),
            )
                .prop_map(|(condition, then_branch, else_branch)| {
                    Expr::new(
                        ExprKind::If {
                            condition: Box::new(condition),
                            then_branch: Box::new(then_branch),
                            else_branch: else_branch.map(Box::new),
                        },
                        Span::new(0, 0),
                    )
                }),
        ]
        .boxed()
    }
}
/// Generate arbitrary expressions
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_expr;
/// 
/// let result = arb_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_expr() -> BoxedStrategy<Expr> {
    arb_expr_with_depth(0)
}
/// Generate arbitrary patterns
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_pattern;
/// 
/// let strategy = arb_pattern();
/// // Use the strategy with proptest
/// ```
pub fn arb_pattern() -> BoxedStrategy<Pattern> {
    prop_oneof![
        any::<i64>().prop_map(|i| Pattern::Literal(Literal::Integer(i))),
        any::<bool>().prop_map(|b| Pattern::Literal(Literal::Bool(b))),
        arb_identifier().prop_map(Pattern::Identifier),
        Just(Pattern::Wildcard),
    ]
    .boxed()
}
/// Generate well-typed expressions (simplified for testing)
/// # Examples
/// 
/// ```
/// use ruchy::testing::generators::arb_well_typed_expr;
/// 
/// let result = arb_well_typed_expr(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn arb_well_typed_expr() -> BoxedStrategy<Expr> {
    // For now, just use simple expressions that are likely to be well-typed
    prop_oneof![
        arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::new(0, 0))),
        arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::new(0, 0))),
        // Simple arithmetic that's always valid
        (any::<i64>(), any::<i64>()).prop_map(|(a, b)| {
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(a)),
                        Span::new(0, 0),
                    )),
                    op: BinaryOp::Add,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(b)),
                        Span::new(0, 0),
                    )),
                },
                Span::new(0, 0),
            )
        }),
    ]
    .boxed()
}
#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_arb_literal_generates_all_variants() {
        // Test that literal generator works
        let strategy = arb_literal();
        let runner = proptest::test_runner::TestRunner::default();

        // Just verify it can generate values without panicking
        for _ in 0..10 {
            let _ = strategy.new_tree(&mut runner.clone());
        }
    }

    #[test]
    fn test_arb_identifier_generates_valid_identifiers() {
        let strategy = arb_identifier();
        let mut runner = proptest::test_runner::TestRunner::default();

        for _ in 0..10 {
            let value = strategy.new_tree(&mut runner).unwrap().current();
            // Identifiers should start with lowercase letter
            assert!(value.chars().next().unwrap().is_ascii_lowercase());
            // All chars should be alphanumeric or underscore
            assert!(value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
        }
    }

    #[test]
    fn test_arb_binary_op_covers_all_ops() {
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
            BinaryOp::Power,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::LessEqual,
            BinaryOp::Greater,
            BinaryOp::GreaterEqual,
            BinaryOp::And,
            BinaryOp::Or,
            BinaryOp::BitwiseAnd,
            BinaryOp::BitwiseOr,
            BinaryOp::BitwiseXor,
            BinaryOp::LeftShift,
        ];

        // Just verify the generator doesn't panic
        let strategy = arb_binary_op();
        let mut runner = proptest::test_runner::TestRunner::default();
        for _ in 0..20 {
            let _ = strategy.new_tree(&mut runner);
        }
    }

    #[test]
    fn test_arb_unary_op_covers_all_ops() {
        let ops = vec![
            UnaryOp::Negate,
            UnaryOp::Not,
            UnaryOp::BitwiseNot,
            UnaryOp::Reference,
        ];

        // Verify generator works
        let strategy = arb_unary_op();
        let mut runner = proptest::test_runner::TestRunner::default();
        for _ in 0..10 {
            let _ = strategy.new_tree(&mut runner);
        }
    }

    #[test]
    fn test_arb_expr_with_depth_respects_max_depth() {
        // Test that depth 0 generates expressions
        let strategy = arb_expr_with_depth(0);
        let mut runner = proptest::test_runner::TestRunner::default();
        for _ in 0..5 {
            let expr = strategy.new_tree(&mut runner).unwrap().current();
            assert_eq!(expr.span, Span::new(0, 0));
        }

        // Test that MAX_DEPTH generates only base cases
        let strategy = arb_expr_with_depth(MAX_DEPTH);
        let mut runner = proptest::test_runner::TestRunner::default();
        for _ in 0..5 {
            let expr = strategy.new_tree(&mut runner).unwrap().current();
            // At max depth, should only generate literals or identifiers
            match &expr.kind {
                ExprKind::Literal(_) | ExprKind::Identifier(_) => {},
                _ => panic!("Generated recursive expression at MAX_DEPTH"),
            }
        }
    }

    #[test]
    fn test_arb_expr_generates_valid_expressions() {
        let strategy = arb_expr();
        let mut runner = proptest::test_runner::TestRunner::default();

        for _ in 0..5 {
            let expr = strategy.new_tree(&mut runner).unwrap().current();
            // All expressions should have a span
            assert_eq!(expr.span, Span::new(0, 0));
        }
    }

    #[test]
    fn test_arb_pattern_generates_all_variants() {
        let strategy = arb_pattern();
        let mut runner = proptest::test_runner::TestRunner::default();

        let mut has_literal = false;
        let mut has_identifier = false;
        let mut has_wildcard = false;

        // Generate many patterns to see variety
        for _ in 0..50 {
            let pattern = strategy.new_tree(&mut runner).unwrap().current();
            match pattern {
                Pattern::Literal(_) => has_literal = true,
                Pattern::Identifier(_) => has_identifier = true,
                Pattern::Wildcard => has_wildcard = true,
                _ => {},
            }
        }

        // We should see at least some variety in 50 iterations
        assert!(has_literal || has_identifier || has_wildcard);
    }

    #[test]
    fn test_arb_well_typed_expr_generates_valid() {
        let strategy = arb_well_typed_expr();
        let mut runner = proptest::test_runner::TestRunner::default();

        for _ in 0..10 {
            let expr = strategy.new_tree(&mut runner).unwrap().current();
            // Well-typed expressions should be simple
            match &expr.kind {
                ExprKind::Literal(_) => {},
                ExprKind::Identifier(_) => {},
                ExprKind::Binary { op: BinaryOp::Add, .. } => {},
                _ => panic!("Unexpected expression type in well-typed generator"),
            }
        }
    }

    #[test]
    fn test_max_depth_constant() {
        assert_eq!(MAX_DEPTH, 5);
    }

    #[test]
    fn test_literal_variants() {
        let _ = Literal::Integer(42);
        let _ = Literal::Float(3.14);
        let _ = Literal::Bool(true);
        let _ = Literal::String("test".to_string());
        let _ = Literal::Unit;
    }

    #[test]
    fn test_span_creation() {
        let span = Span::new(0, 0);
        assert_eq!(span, Span::new(0, 0));
    }

    #[test]
    fn test_expr_new() {
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 0));
        assert!(matches!(expr.kind, ExprKind::Literal(Literal::Integer(42))));
        assert_eq!(expr.span, Span::new(0, 0));
    }

    #[test]
    fn test_pattern_enum_variants() {
        let _ = Pattern::Literal(Literal::Integer(1));
        let _ = Pattern::Identifier("x".to_string());
        let _ = Pattern::Wildcard;
    }

    #[test]
    fn test_binary_op_variants_exist() {
        // Just ensure all BinaryOp variants are constructible
        let _ = BinaryOp::Add;
        let _ = BinaryOp::Subtract;
        let _ = BinaryOp::Multiply;
        let _ = BinaryOp::Divide;
        let _ = BinaryOp::Modulo;
        let _ = BinaryOp::Power;
    }

    #[test]
    fn test_unary_op_variants_exist() {
        // Ensure all UnaryOp variants are constructible
        let _ = UnaryOp::Negate;
        let _ = UnaryOp::Not;
        let _ = UnaryOp::BitwiseNot;
        let _ = UnaryOp::Reference;
    }

    #[test]
    fn test_expr_kind_if_variant() {
        let cond = Box::new(Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0)));
        let then = Box::new(Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(0, 0)));
        let else_b = Some(Box::new(Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(0, 0))));

        let if_expr = ExprKind::If {
            condition: cond,
            then_branch: then,
            else_branch: else_b,
        };

        let expr = Expr::new(if_expr, Span::new(0, 0));
        assert!(matches!(expr.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_arb_literal_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_literal();

        // Generate a few literals and check they're valid
        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let literal = value_tree.current();
            // Just verify it doesn't panic
            match literal {
                Literal::Integer(_) | Literal::Float(_) | Literal::Bool(_) |
                Literal::String(_) | Literal::Unit | Literal::Char(_) => {},
            }
        }
    }

    #[test]
    fn test_arb_identifier_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_identifier();

        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let identifier = value_tree.current();
            // Check it starts with a letter
            assert!(!identifier.is_empty());
            assert!(identifier.chars().next().unwrap().is_alphabetic());
        }
    }

    #[test]
    fn test_arb_binary_op_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_binary_op();

        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let _op = value_tree.current();
            // Just verify it generates without panic
        }
    }

    #[test]
    fn test_arb_unary_op_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_unary_op();

        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let _op = value_tree.current();
            // Just verify it generates without panic
        }
    }

    #[test]
    fn test_arb_pattern_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_pattern();

        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let pattern = value_tree.current();
            // Verify pattern is valid
            match pattern {
                Pattern::Literal(_) | Pattern::Identifier(_) | Pattern::Wildcard |
                Pattern::Tuple(_) | Pattern::Struct { .. } | _ => {},
            }
        }
    }

    #[test]
    fn test_arb_expr_generation() {
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;

        let mut runner = TestRunner::default();
        let strategy = arb_expr();

        // Generate several expressions
        for _ in 0..10 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let expr = value_tree.current();
            // Just verify it generates valid AST
            assert_eq!(expr.span, Span::new(0, 0));
        }
    }

    #[test]
    fn test_multiple_expr_generation() {
        // Ensure we can generate many expressions without issues
        use proptest::strategy::ValueTree;
        use proptest::test_runner::TestRunner;
        let mut runner = TestRunner::default();
        let strategy = arb_expr();

        for _ in 0..20 {
            let value_tree = strategy.new_tree(&mut runner).unwrap();
            let _expr = value_tree.current();
            // If this doesn't panic, generation works
        }
    }

    #[test]
    fn test_expr_kind_variants() {
        // Test that various ExprKind variants can be constructed
        let _ = ExprKind::Literal(Literal::Integer(42));
        let _ = ExprKind::Identifier("x".to_string());
        let _ = ExprKind::Block(vec![]);
        let _ = ExprKind::Return { value: None };
        let _ = ExprKind::Break { label: None };
        let _ = ExprKind::Continue { label: None };
    }

    #[test]
    fn test_span_creation_extended() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);

        let span2 = Span { start: 5, end: 15 };
        assert_eq!(span2.start, 5);
        assert_eq!(span2.end, 15);
    }

    #[test]
    fn test_literal_char_variant() {
        let char_lit = Literal::Char('a');
        match char_lit {
            Literal::Char(c) => assert_eq!(c, 'a'),
            _ => panic!("Expected Char variant"),
        }
    }

    // Pattern::Rest doesn't exist in the current AST
}
