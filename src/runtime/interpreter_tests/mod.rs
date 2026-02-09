//! Tests for the interpreter module
//!
//! EXTREME TDD Round 86: Comprehensive tests for interpreter.rs
//! Coverage target: 95% for interpreter module
//!
//! This module contains all tests for the interpreter, extracted from interpreter.rs
//! for maintainability and to reduce the main module size.

#[allow(unused_imports)]
use crate::frontend::ast::{
    BinaryOp as AstBinaryOp, ComprehensionClause, Expr, ExprKind, Literal, Param, Pattern,
    Span, Type, TypeKind, UnaryOp,
};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::Value;
#[allow(unused_imports)]
use std::sync::Arc;

// ============== Helper Functions ==============

/// Helper to create a simple integer literal expression
fn make_int(val: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(val, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a float literal expression
fn make_float(val: f64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Float(val)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a bool literal expression
fn make_bool(val: bool) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Bool(val)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a string literal expression
fn make_string(val: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(val.to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create an identifier expression
fn make_ident(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a binary expression
fn make_binary(left: Expr, op: AstBinaryOp, right: Expr) -> Expr {
    Expr {
        kind: ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a unary expression
fn make_unary(op: UnaryOp, operand: Expr) -> Expr {
    Expr {
        kind: ExprKind::Unary {
            op,
            operand: Box::new(operand),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create an if expression
fn make_if(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
    Expr {
        kind: ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a block expression
fn make_block(exprs: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Block(exprs),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a list/array expression
fn make_list(elements: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::List(elements),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a tuple expression
fn make_tuple(elements: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Tuple(elements),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create an index access expression
fn make_index(object: Expr, index: Expr) -> Expr {
    Expr {
        kind: ExprKind::IndexAccess {
            object: Box::new(object),
            index: Box::new(index),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a let expression
fn make_let(name: &str, value: Expr, body: Expr) -> Expr {
    Expr {
        kind: ExprKind::Let {
            name: name.to_string(),
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: false,
            else_block: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a range expression
fn make_range(start: Expr, end: Expr, inclusive: bool) -> Expr {
    Expr {
        kind: ExprKind::Range {
            start: Box::new(start),
            end: Box::new(end),
            inclusive,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create an array/list expression (alias)
fn make_array(elements: Vec<Expr>) -> Expr {
    make_list(elements)
}

/// Helper to create a for expression
fn make_for(var: &str, iter: Expr, body: Expr) -> Expr {
    Expr {
        kind: ExprKind::For {
            var: var.to_string(),
            iter: Box::new(iter),
            body: Box::new(body),
            label: None,
            pattern: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a while expression
fn make_while(condition: Expr, body: Expr) -> Expr {
    Expr {
        kind: ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
            label: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create an assign expression
fn make_assign(name: &str, value: Expr) -> Expr {
    Expr {
        kind: ExprKind::Assign {
            target: Box::new(make_ident(name)),
            value: Box::new(value),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a mutable let expression
fn make_let_mut(name: &str, value: Expr, body: Expr) -> Expr {
    Expr {
        kind: ExprKind::Let {
            name: name.to_string(),
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: true,
            else_block: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a compound assign expression
fn make_compound_assign(name: &str, op: AstBinaryOp, value: Expr) -> Expr {
    Expr {
        kind: ExprKind::CompoundAssign {
            target: Box::new(make_ident(name)),
            op,
            value: Box::new(value),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a break expression
fn make_break() -> Expr {
    Expr {
        kind: ExprKind::Break {
            label: None,
            value: None,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a continue expression
fn make_continue() -> Expr {
    Expr {
        kind: ExprKind::Continue { label: None },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create a unit expression
fn make_unit() -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}




// ---------- Additional Helper Functions ----------

/// Helper to create lambda with parameters
fn make_lambda_with_params(params: Vec<String>, body: Expr) -> Expr {
    Expr {
        kind: ExprKind::Lambda {
            params: params
                .into_iter()
                .map(|name| Param {
                    pattern: Pattern::Identifier(name),
                    ty: Type {
                        kind: TypeKind::Named("Any".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                })
                .collect(),
            body: Box::new(body),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create return expression
fn make_return(value: Option<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Return {
            value: value.map(Box::new),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}

/// Helper to create call expression
fn make_call(func: Expr, args: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Call {
            func: Box::new(func),
            args,
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }
}


#[path = "interpreter_tests_part1.rs"]
mod part1;

#[path = "interpreter_tests_part2.rs"]
mod part2;

#[path = "interpreter_tests_part3.rs"]
mod part3;

#[path = "interpreter_tests_part4.rs"]
mod part4;

#[path = "interpreter_tests_part5.rs"]
mod part5;

#[path = "interpreter_tests_part6.rs"]
mod part6;

#[path = "interpreter_tests_part7.rs"]
mod part7;

#[path = "interpreter_tests_part8.rs"]
mod part8;

#[path = "interpreter_tests_part9.rs"]
mod part9;

#[path = "interpreter_tests_part10.rs"]
mod part10;

#[path = "interpreter_tests_part11.rs"]
mod part11;

#[path = "interpreter_tests_part12.rs"]
mod part12;

#[path = "interpreter_tests_part13.rs"]
mod part13;

#[path = "interpreter_tests_part14.rs"]
mod part14;

#[path = "interpreter_tests_part15.rs"]
mod part15;

#[path = "interpreter_tests_part16.rs"]
mod part16;
