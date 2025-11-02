// PERF-002-A: Constant Folding Optimization (GREEN Phase)
// Minimal implementation to make RED tests pass
// Complexity target: ≤10 per function

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
#[cfg(test)]
use crate::frontend::ast::Span;

/// Fold constant expressions at compile-time
///
/// Examples:
/// - `2 + 3` → `5`
/// - `10 > 5` → `true`
/// - `(10 - 2) * 4` → `32`
///
/// # Arguments
/// * `expr` - Expression to potentially fold
///
/// # Returns
/// Folded expression if possible, otherwise original expression
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
pub fn fold_constants(expr: Expr) -> Expr {
    match &expr.kind {
        ExprKind::Binary { left, op, right } => {
            // Recursively fold children first
            let left_folded = fold_constants((**left).clone());
            let right_folded = fold_constants((**right).clone());

            // Try to fold if both are literals
            if let (ExprKind::Literal(l), ExprKind::Literal(r)) =
                (&left_folded.kind, &right_folded.kind) {
                if let Some(result) = fold_binary_op(l, *op, r) {
                    return Expr::new(ExprKind::Literal(result), expr.span);
                }
            }

            // Return with folded children even if we can't fold this level
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(left_folded),
                    op: *op,
                    right: Box::new(right_folded),
                },
                expr.span,
            )
        }
        _ => expr, // Other expressions: no folding yet
    }
}

/// Fold binary operation on two literals
///
/// # Complexity
/// Cyclomatic: 8 (≤10 target)
fn fold_binary_op(left: &Literal, op: BinaryOp, right: &Literal) -> Option<Literal> {
    match (left, right) {
        // Integer operations (both arithmetic and comparison)
        (Literal::Integer(a, None), Literal::Integer(b, None)) => {
            fold_integer_comparison(*a, op, *b).or_else(|| fold_integer_arithmetic(*a, op, *b))
        }

        _ => None, // Other combinations: not folded yet
    }
}

/// Fold integer arithmetic operations
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn fold_integer_arithmetic(a: i64, op: BinaryOp, b: i64) -> Option<Literal> {
    let result = match op {
        BinaryOp::Add => a.checked_add(b)?,
        BinaryOp::Subtract => a.checked_sub(b)?,
        BinaryOp::Multiply => a.checked_mul(b)?,
        BinaryOp::Divide if b != 0 => a.checked_div(b)?,
        _ => return None,
    };
    Some(Literal::Integer(result, None))
}

/// Fold integer comparison operations
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
fn fold_integer_comparison(a: i64, op: BinaryOp, b: i64) -> Option<Literal> {
    let result = match op {
        BinaryOp::Equal => a == b,
        BinaryOp::NotEqual => a != b,
        BinaryOp::Less => a < b,
        BinaryOp::LessEqual => a <= b,
        BinaryOp::Greater => a > b,
        BinaryOp::GreaterEqual => a >= b,
        _ => return None,
    };
    Some(Literal::Bool(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_simple_add() {
        // 2 + 3 → 5
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span::new(0, 1),
                )),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::new(4, 5),
                )),
            },
            Span::new(0, 5),
        );

        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Integer(5, None))
        ));
    }

    #[test]
    fn test_fold_comparison() {
        // 10 > 5 → true
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::new(0, 2),
                )),
                op: BinaryOp::Greater,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::new(5, 6),
                )),
            },
            Span::new(0, 6),
        );

        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Bool(true))
        ));
    }
}
