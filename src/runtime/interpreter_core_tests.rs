use super::*;
use crate::frontend::ast::{Expr, ExprKind, Literal, Span, UnaryOp};

fn make_interpreter() -> Interpreter {
    Interpreter::new()
}

fn make_expr(kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

#[path = "interpreter_core_tests_part1.rs"]
mod part1;
#[path = "interpreter_core_tests_part2.rs"]
mod part2;
