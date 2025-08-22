//! Property-based test generators for AST nodes

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, Span, UnaryOp};
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};

/// Maximum depth for recursive AST generation to avoid stack overflow
const MAX_DEPTH: u32 = 5;

/// Generate arbitrary literals
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
pub fn arb_identifier() -> BoxedStrategy<String> {
    "[a-z][a-z0-9_]{0,10}".prop_map(|s| s).boxed()
}

/// Generate arbitrary binary operators
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
pub fn arb_expr() -> BoxedStrategy<Expr> {
    arb_expr_with_depth(0)
}

/// Generate arbitrary patterns
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
