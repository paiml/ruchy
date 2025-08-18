//! Property-based test generators for AST nodes

use crate::frontend::ast::*;
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};

/// Maximum depth for recursive AST generation to avoid stack overflow
const MAX_DEPTH: u32 = 5;

/// Configuration for AST generation
#[derive(Debug, Clone)]
pub struct AstGenConfig {
    pub max_depth: u32,
    pub max_list_size: usize,
    pub max_identifier_len: usize,
    pub favor_well_typed: bool,
}

impl Default for AstGenConfig {
    fn default() -> Self {
        Self {
            max_depth: MAX_DEPTH,
            max_list_size: 10,
            max_identifier_len: 20,
            favor_well_typed: true,
        }
    }
}

/// Generate arbitrary literals
pub fn arb_literal() -> BoxedStrategy<Literal> {
    prop_oneof![
        (any::<i64>()).prop_map(Literal::Integer),
        (any::<f64>().prop_filter("not NaN", |f| !f.is_nan())).prop_map(Literal::Float),
        ("[a-zA-Z0-9 ]{0,50}").prop_map(Literal::String),
        (any::<bool>()).prop_map(Literal::Bool),
        Just(Literal::Unit),
    ]
    .boxed()
}

/// Generate arbitrary identifiers
pub fn arb_identifier() -> BoxedStrategy<String> {
    "[a-z][a-z0-9_]{0,19}"
        .prop_filter("not keyword", |s| !is_keyword(s))
        .boxed()
}

/// Generate arbitrary string parts for interpolation
pub fn arb_string_part() -> BoxedStrategy<StringPart> {
    prop_oneof![
        // Text parts (avoiding braces to prevent confusion)
        "[a-zA-Z0-9 .,!?]+".prop_map(StringPart::Text),
        // Expression parts (simple expressions for now)
        arb_simple_expr().prop_map(|expr| StringPart::Expr(Box::new(expr))),
    ]
    .boxed()
}

/// Generate simple expressions for use in string interpolation
pub fn arb_simple_expr() -> BoxedStrategy<Expr> {
    prop_oneof![
        arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::default())),
        arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::default())),
    ]
    .boxed()
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "fun"
            | "let"
            | "if"
            | "else"
            | "match"
            | "for"
            | "in"
            | "while"
            | "return"
            | "break"
            | "continue"
            | "struct"
            | "impl"
            | "trait"
            | "type"
            | "const"
            | "static"
            | "mut"
            | "pub"
            | "import"
            | "use"
            | "as"
    )
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
        Just(BinaryOp::RightShift),
    ]
    .boxed()
}

/// Generate arbitrary unary operators
pub fn arb_unary_op() -> BoxedStrategy<UnaryOp> {
    prop_oneof![
        Just(UnaryOp::Not),
        Just(UnaryOp::Negate),
        Just(UnaryOp::BitwiseNot),
    ]
    .boxed()
}

/// Generate arbitrary patterns for match expressions
pub fn arb_pattern() -> BoxedStrategy<Pattern> {
    prop_oneof![
        Just(Pattern::Wildcard),
        arb_literal().prop_map(Pattern::Literal),
        arb_identifier().prop_map(Pattern::Identifier),
        // Simple list patterns (non-recursive)
        prop::collection::vec(arb_literal().prop_map(Pattern::Literal), 0..3)
            .prop_map(Pattern::List),
    ]
    .boxed()
}

/// Generate arbitrary types
pub fn arb_type() -> BoxedStrategy<Type> {
    let base_types = prop_oneof![
        Just("i32"),
        Just("i64"),
        Just("f32"),
        Just("f64"),
        Just("bool"),
        Just("String"),
    ];

    base_types
        .prop_map(|name| Type {
            kind: TypeKind::Named(name.to_string()),
            span: Span::new(0, 0),
        })
        .boxed()
}

/// Generate arbitrary parameters
pub fn arb_param() -> BoxedStrategy<Param> {
    (arb_identifier(), arb_type())
        .prop_map(|(name, ty)| Param {
            name,
            ty,
            span: Span::new(0, 0),
            is_mutable: false,
        })
        .boxed()
}

/// Generate arbitrary expressions with depth control
pub fn arb_expr_with_depth(depth: u32) -> BoxedStrategy<Expr> {
    if depth == 0 {
        // Base case: only literals and identifiers
        prop_oneof![
            arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::new(0, 0))),
            arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::new(0, 0))),
        ]
        .boxed()
    } else {
        // Recursive cases with reduced depth
        let smaller_expr = arb_expr_with_depth(depth - 1);

        prop_oneof![
            // Literals and identifiers (base cases)
            arb_literal().prop_map(|lit| Expr::new(ExprKind::Literal(lit), Span::new(0, 0))),
            arb_identifier().prop_map(|id| Expr::new(ExprKind::Identifier(id), Span::new(0, 0))),
            // Binary operations
            (smaller_expr.clone(), arb_binary_op(), smaller_expr.clone()).prop_map(
                |(left, op, right)| {
                    Expr::new(
                        ExprKind::Binary {
                            left: Box::new(left),
                            op,
                            right: Box::new(right),
                        },
                        Span::new(0, 0),
                    )
                }
            ),
            // Unary operations
            (arb_unary_op(), smaller_expr.clone()).prop_map(|(op, operand)| {
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
                smaller_expr.clone(),
                smaller_expr.clone(),
                prop::option::of(smaller_expr.clone())
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
            // Let bindings
            (arb_identifier(), smaller_expr.clone(), smaller_expr.clone()).prop_map(
                |(name, value, body)| {
                    Expr::new(
                        ExprKind::Let {
                            name,
                            value: Box::new(value),
                            body: Box::new(body),
                            is_mutable: false,
                        },
                        Span::new(0, 0),
                    )
                }
            ),
            // Lists
            prop::collection::vec(smaller_expr.clone(), 0..5)
                .prop_map(|elements| { Expr::new(ExprKind::List(elements), Span::new(0, 0)) }),
            // Blocks
            prop::collection::vec(smaller_expr.clone(), 1..4)
                .prop_map(|exprs| { Expr::new(ExprKind::Block(exprs), Span::new(0, 0)) }),
            // Ranges
            (smaller_expr.clone(), smaller_expr, any::<bool>()).prop_map(
                |(start, end, inclusive)| {
                    Expr::new(
                        ExprKind::Range {
                            start: Box::new(start),
                            end: Box::new(end),
                            inclusive,
                        },
                        Span::new(0, 0),
                    )
                }
            ),
        ]
        .boxed()
    }
}

/// Generate arbitrary expressions
pub fn arb_expr() -> BoxedStrategy<Expr> {
    arb_expr_with_depth(MAX_DEPTH)
}

/// Generate well-typed expressions (more likely to be valid)
pub fn arb_well_typed_expr() -> BoxedStrategy<Expr> {
    // Focus on expressions that are more likely to type-check
    prop_oneof![
        // Simple arithmetic
        (1i64..100, 1i64..100).prop_map(|(a, b)| {
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
        // Simple let binding
        (arb_identifier(), 1i64..100).prop_map(|(name, value)| {
            Expr::new(
                ExprKind::Let {
                    name: name.clone(),
                    value: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(value)),
                        Span::new(0, 0),
                    )),
                    body: Box::new(Expr::new(ExprKind::Identifier(name), Span::new(0, 0))),
                    is_mutable: false,
                },
                Span::new(0, 0),
            )
        }),
        // Simple if expression
        any::<bool>().prop_map(|b| {
            Expr::new(
                ExprKind::If {
                    condition: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Bool(b)),
                        Span::new(0, 0),
                    )),
                    then_branch: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(1)),
                        Span::new(0, 0),
                    )),
                    else_branch: Some(Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(0)),
                        Span::new(0, 0),
                    ))),
                },
                Span::new(0, 0),
            )
        }),
    ]
    .boxed()
}

/// Shrinking strategy for expressions
impl Expr {
    /// Shrink an expression to simpler forms
    #[must_use]
    pub fn shrink_expr(&self) -> Vec<Expr> {
        let mut shrunk = Vec::new();

        match &self.kind {
            ExprKind::Binary { left, right, .. } => {
                // Try just the left operand
                shrunk.push((**left).clone());
                // Try just the right operand
                shrunk.push((**right).clone());
                // Try with simpler operands
                for l in left.shrink_expr() {
                    shrunk.push(Expr::new(self.kind.clone_with_left(Box::new(l)), self.span));
                }
                for r in right.shrink_expr() {
                    shrunk.push(Expr::new(
                        self.kind.clone_with_right(Box::new(r)),
                        self.span,
                    ));
                }
            }
            ExprKind::Unary { operand, .. } => {
                // Try just the operand
                shrunk.push((**operand).clone());
                // Try with simpler operand
                for o in operand.shrink_expr() {
                    shrunk.push(Expr::new(
                        self.kind.clone_with_operand(Box::new(o)),
                        self.span,
                    ));
                }
            }
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // Try just the then branch
                shrunk.push((**then_branch).clone());
                // Try just the else branch if it exists
                if let Some(else_br) = else_branch {
                    shrunk.push((**else_br).clone());
                }
                // Try simpler conditions
                for c in condition.shrink_expr() {
                    shrunk.push(Expr::new(
                        ExprKind::If {
                            condition: Box::new(c),
                            then_branch: then_branch.clone(),
                            else_branch: else_branch.clone(),
                        },
                        self.span,
                    ));
                }
            }
            ExprKind::List(elements) if !elements.is_empty() => {
                // Try with fewer elements
                for i in 1..elements.len() {
                    shrunk.push(Expr::new(ExprKind::List(elements[..i].to_vec()), self.span));
                }
                // Try empty list
                shrunk.push(Expr::new(ExprKind::List(vec![]), self.span));
            }
            ExprKind::Block(exprs) if exprs.len() > 1 => {
                // Try with fewer expressions
                for i in 1..exprs.len() {
                    shrunk.push(Expr::new(ExprKind::Block(exprs[..i].to_vec()), self.span));
                }
                // Try just the last expression
                if let Some(last) = exprs.last() {
                    shrunk.push(last.clone());
                }
            }
            _ => {
                // For literals and identifiers, try common simple values
                shrunk.push(Expr::new(
                    ExprKind::Literal(Literal::Integer(0)),
                    Span::new(0, 0),
                ));
                shrunk.push(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::new(0, 0),
                ));
            }
        }

        shrunk
    }
}

// Helper methods for cloning with modifications (needed for shrinking)
impl ExprKind {
    fn clone_with_left(&self, new_left: Box<Expr>) -> ExprKind {
        match self {
            ExprKind::Binary { op, right, .. } => ExprKind::Binary {
                left: new_left,
                op: *op,
                right: right.clone(),
            },
            _ => self.clone(),
        }
    }

    fn clone_with_right(&self, new_right: Box<Expr>) -> ExprKind {
        match self {
            ExprKind::Binary { left, op, .. } => ExprKind::Binary {
                left: left.clone(),
                op: *op,
                right: new_right,
            },
            _ => self.clone(),
        }
    }

    fn clone_with_operand(&self, new_operand: Box<Expr>) -> ExprKind {
        match self {
            ExprKind::Unary { op, .. } => ExprKind::Unary {
                op: *op,
                operand: new_operand,
            },
            _ => self.clone(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic, clippy::expect_used)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_generated_expr_has_valid_structure(expr in arb_expr()) {
            // Every generated expression should have some content
            match expr.kind {
                ExprKind::Binary { ref left, ref right, .. } => {
                    // Binary ops should have both operands
                    prop_assert!(left.span.start <= left.span.end);
                    prop_assert!(right.span.start <= right.span.end);
                }
                ExprKind::List(ref elements) => {
                    // Lists can be empty or have elements
                    prop_assert!(elements.len() <= 100); // Reasonable size
                }
                _ => {
                    // All other cases (literals, identifiers, etc.) are valid by construction
                }
            }
        }

        #[test]
        fn test_well_typed_expr_simpler(expr in arb_well_typed_expr()) {
            // Well-typed expressions should be relatively simple
            let depth = measure_depth(&expr);
            prop_assert!(depth <= 3, "Well-typed expressions should be simple, got depth {}", depth);
        }

        #[test]
        fn test_shrinking_reduces_size(expr in arb_expr()) {
            let shrunk = expr.shrink_expr();
            for s in shrunk {
                let original_size = expr_size(&expr);
                let shrunk_size = expr_size(&s);
                prop_assert!(shrunk_size <= original_size,
                    "Shrunk expression should be smaller: {} > {}", shrunk_size, original_size);
            }
        }
    }

    fn measure_depth(expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::Binary { left, right, .. } => {
                1 + measure_depth(left).max(measure_depth(right))
            }
            ExprKind::Unary { operand, .. } => 1 + measure_depth(operand),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let if_depth = 1 + measure_depth(condition).max(measure_depth(then_branch));
                if let Some(else_br) = else_branch {
                    if_depth.max(1 + measure_depth(else_br))
                } else {
                    if_depth
                }
            }
            ExprKind::Block(exprs) | ExprKind::List(exprs) => {
                1 + exprs.iter().map(measure_depth).max().unwrap_or(0)
            }
            _ => 1, // Literals, identifiers, and other leaf expressions
        }
    }

    fn expr_size(expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::Binary { left, right, .. } => 1 + expr_size(left) + expr_size(right),
            ExprKind::Unary { operand, .. } => 1 + expr_size(operand),
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let size = 1 + expr_size(condition) + expr_size(then_branch);
                if let Some(else_br) = else_branch {
                    size + expr_size(else_br)
                } else {
                    size
                }
            }
            ExprKind::Block(exprs) | ExprKind::List(exprs) => {
                1 + exprs.iter().map(expr_size).sum::<usize>()
            }
            _ => 1, // Literals, identifiers, and other single-node expressions
        }
    }
}
