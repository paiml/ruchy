//! Modular parser implementation for Ruchy
#![allow(clippy::wildcard_imports)]
#![allow(clippy::expect_used)]
//!
//! The parser is split into logical modules to improve maintainability:
//! - `core` - Main parser entry points and precedence handling
//! - `expressions` - Basic expressions (literals, binary/unary ops)
//! - `control_flow` - Control flow constructs (if, match, loops)
//! - `functions` - Function definitions, lambdas, and calls
//! - `types` - Type-related parsing (struct, trait, impl)
//! - `collections` - Collections (lists, dataframes, comprehensions)
//! - `actors` - Actor system constructs
//! - `utils` - Parsing utilities and error recovery

mod actors;
mod collections;
mod control_flow;
mod core;
mod expressions;
mod functions;
mod types;
mod utils;

// Re-export the main parser
pub use core::Parser;

use crate::frontend::arena::{Arena, StringInterner};
use crate::frontend::ast::{
    Attribute, BinaryOp, Expr, ExprKind, ImplMethod, Literal, MatchArm, Param, Pattern,
    PipelineStage, Span, StringPart, StructField, TraitMethod, Type, TypeKind, UnaryOp,
};
use crate::frontend::lexer::{Token, TokenStream};
use crate::parser::error_recovery::{ErrorNode, ErrorRecovery};
use anyhow::{bail, Result};

/// Shared parser state and utilities
pub(crate) struct ParserState<'a> {
    pub tokens: TokenStream<'a>,
    #[allow(dead_code)]
    pub error_recovery: ErrorRecovery,
    pub errors: Vec<ErrorNode>,
    /// Arena allocator for AST nodes
    #[allow(dead_code)]
    pub arena: Arena,
    /// String interner for deduplicating identifiers
    #[allow(dead_code)]
    pub interner: StringInterner,
}

impl<'a> ParserState<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
            error_recovery: ErrorRecovery::new(),
            errors: Vec::new(),
            arena: Arena::new(),
            interner: StringInterner::new(),
        }
    }

    /// Get all errors encountered during parsing
    pub fn get_errors(&self) -> &[ErrorNode] {
        &self.errors
    }

    /// Get arena statistics for performance monitoring
    #[allow(dead_code)]
    pub fn arena_stats(&self) -> (usize, usize) {
        (self.arena.total_allocated(), self.arena.num_items())
    }

    /// Get interner statistics
    #[allow(dead_code)]
    pub fn interner_stats(&self) -> (usize, usize) {
        self.interner.stats()
    }
}

/// Forward declarations for recursive parsing
pub(crate) fn parse_expr_recursive(state: &mut ParserState) -> Result<Expr> {
    parse_expr_with_precedence_recursive(state, 0)
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::cognitive_complexity)]
pub(crate) fn parse_expr_with_precedence_recursive(
    state: &mut ParserState,
    min_prec: i32,
) -> Result<Expr> {
    let mut left = expressions::parse_prefix(state)?;

    loop {
        // Handle postfix operators
        left = handle_postfix_operators(state, left)?;

        // Get current token for infix processing
        let Some((token, _)) = state.tokens.peek() else {
            break;
        };
        let token_clone = token.clone();

        // Try different infix operator types
        if let Some(new_left) = try_actor_operators(state, left.clone(), &token_clone, min_prec)? {
            left = new_left;
            continue;
        }

        if let Some(new_left) = try_binary_operators(state, left.clone(), &token_clone, min_prec)? {
            left = new_left;
            continue;
        }

        if let Some(new_left) =
            try_assignment_operators(state, left.clone(), &token_clone, min_prec)?
        {
            left = new_left;
            continue;
        }

        if let Some(new_left) = try_pipeline_operators(state, left.clone(), &token_clone, min_prec)?
        {
            left = new_left;
            continue;
        }

        if let Some(new_left) = try_range_operators(state, left.clone(), &token_clone, min_prec)? {
            left = new_left;
            continue;
        }

        break;
    }

    Ok(left)
}

/// Handle all postfix operators in a loop
fn handle_postfix_operators(state: &mut ParserState, mut left: Expr) -> Result<Expr> {
    let mut handled_postfix = true;
    while handled_postfix {
        handled_postfix = false;
        match state.tokens.peek() {
            Some((Token::Dot, _)) => {
                state.tokens.advance();
                left = functions::parse_method_call(state, left)?;
                handled_postfix = true;
            }
            Some((Token::LeftParen, _)) => {
                left = functions::parse_call(state, left)?;
                handled_postfix = true;
            }
            Some((Token::LeftBracket, _)) => {
                left = handle_array_indexing(state, left)?;
                handled_postfix = true;
            }
            Some((Token::Question, _)) => {
                if let Some(new_left) = try_parse_try_operator(state, left.clone()) {
                    left = new_left;
                    handled_postfix = true;
                }
            }
            Some((Token::LeftBrace, _)) => {
                if let Some(new_left) = try_parse_struct_literal(state, &left)? {
                    left = new_left;
                    handled_postfix = true;
                }
            }
            Some((Token::Increment, _)) => {
                state.tokens.advance();
                left = create_post_increment(left);
                handled_postfix = true;
            }
            Some((Token::Decrement, _)) => {
                state.tokens.advance();
                left = create_post_decrement(left);
                handled_postfix = true;
            }
            _ => {}
        }
    }
    Ok(left)
}

/// Handle array indexing syntax [expr]
fn handle_array_indexing(state: &mut ParserState, left: Expr) -> Result<Expr> {
    state.tokens.advance(); // consume [
    let index = parse_expr_recursive(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Expr {
        kind: ExprKind::Call {
            func: Box::new(Expr {
                kind: ExprKind::Identifier("get".to_string()),
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            }),
            args: vec![left, index],
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    })
}

/// Try to parse try operator (?)
fn try_parse_try_operator(state: &mut ParserState, left: Expr) -> Option<Expr> {
    let next_token = state.tokens.peek_nth(1);
    let is_try = match next_token {
        None => true,
        Some((Token::Identifier(_), _)) => false,
        Some((token, _)) => matches!(
            token,
            Token::Semicolon
                | Token::Comma
                | Token::RightParen
                | Token::RightBracket
                | Token::RightBrace
                | Token::Else
                | Token::In
        ),
    };
    if is_try {
        state.tokens.advance();
        Some(Expr {
            kind: ExprKind::Try {
                expr: Box::new(left),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        })
    } else {
        None
    }
}

/// Try to parse struct literal
fn try_parse_struct_literal(state: &mut ParserState, left: &Expr) -> Result<Option<Expr>> {
    if let ExprKind::Identifier(name) = &left.kind {
        if name.chars().next().is_some_and(char::is_uppercase) {
            let name = name.clone();
            let span = left.span;
            return Ok(Some(types::parse_struct_literal(state, name, span)?));
        }
    }
    Ok(None)
}

/// Create post-increment expression
fn create_post_increment(left: Expr) -> Expr {
    Expr {
        kind: ExprKind::PostIncrement {
            target: Box::new(left),
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    }
}

/// Create post-decrement expression
fn create_post_decrement(left: Expr) -> Expr {
    Expr {
        kind: ExprKind::PostDecrement {
            target: Box::new(left),
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    }
}

/// Try to parse actor message operators (! and ?)
fn try_actor_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::Bang | Token::Question) {
        return Ok(None);
    }

    let is_binary_op = if let Some(next_token) = state.tokens.peek_nth(1) {
        !matches!(next_token.0, Token::LeftParen)
    } else {
        true
    };

    if !is_binary_op {
        return Ok(None);
    }

    let prec = 10;
    if prec < min_prec {
        return Ok(None);
    }

    state.tokens.advance();
    let message = expressions::parse_prefix(state)?;

    let expr = if matches!(token, Token::Bang) {
        Expr {
            kind: ExprKind::Send {
                actor: Box::new(left),
                message: Box::new(message),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    } else {
        Expr {
            kind: ExprKind::Ask {
                actor: Box::new(left),
                message: Box::new(message),
                timeout: None,
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    };
    Ok(Some(expr))
}

/// Try to parse binary operators
fn try_binary_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if let Some(bin_op) = expressions::token_to_binary_op(token) {
        let prec = expressions::get_precedence(bin_op);
        if prec < min_prec {
            return Ok(None);
        }

        state.tokens.advance();
        let right = parse_expr_with_precedence_recursive(state, prec + 1)?;
        Ok(Some(Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op: bin_op,
                right: Box::new(right),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }))
    } else {
        Ok(None)
    }
}

/// Try to parse assignment operators
fn try_assignment_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !token.is_assignment_op() {
        return Ok(None);
    }

    let prec = 1;
    if prec < min_prec {
        return Ok(None);
    }

    state.tokens.advance();
    let value = parse_expr_with_precedence_recursive(state, prec)?;

    let expr = if *token == Token::Equal {
        Expr {
            kind: ExprKind::Assign {
                target: Box::new(left),
                value: Box::new(value),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    } else {
        let bin_op = get_compound_assignment_op(token);
        Expr {
            kind: ExprKind::CompoundAssign {
                target: Box::new(left),
                op: bin_op,
                value: Box::new(value),
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    };
    Ok(Some(expr))
}

/// Get binary operator for compound assignment
fn get_compound_assignment_op(token: &Token) -> BinaryOp {
    match token {
        Token::PlusEqual => BinaryOp::Add,
        Token::MinusEqual => BinaryOp::Subtract,
        Token::StarEqual => BinaryOp::Multiply,
        Token::SlashEqual => BinaryOp::Divide,
        Token::PercentEqual => BinaryOp::Modulo,
        Token::PowerEqual => BinaryOp::Power,
        Token::AmpersandEqual => BinaryOp::BitwiseAnd,
        Token::PipeEqual => BinaryOp::BitwiseOr,
        Token::CaretEqual => BinaryOp::BitwiseXor,
        Token::LeftShiftEqual => BinaryOp::LeftShift,
        Token::RightShiftEqual => BinaryOp::RightShift,
        _ => unreachable!("Already checked is_assignment_op"),
    }
}

/// Try to parse pipeline operators (|>)
fn try_pipeline_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::Pipeline) {
        return Ok(None);
    }

    let prec = 3;
    if prec < min_prec {
        return Ok(None);
    }

    state.tokens.advance();
    let stage_expr = parse_expr_with_precedence_recursive(state, prec + 1)?;

    let expr = if let ExprKind::Pipeline { expr, mut stages } = left.kind {
        stages.push(PipelineStage {
            op: Box::new(stage_expr),
            span: Span { start: 0, end: 0 },
        });
        Expr {
            kind: ExprKind::Pipeline { expr, stages },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    } else {
        Expr {
            kind: ExprKind::Pipeline {
                expr: Box::new(left),
                stages: vec![PipelineStage {
                    op: Box::new(stage_expr),
                    span: Span { start: 0, end: 0 },
                }],
            },
            span: Span { start: 0, end: 0 },
            attributes: Vec::new(),
        }
    };
    Ok(Some(expr))
}

/// Try to parse range operators (.., ..=)
fn try_range_operators(
    state: &mut ParserState,
    left: Expr,
    token: &Token,
    min_prec: i32,
) -> Result<Option<Expr>> {
    if !matches!(token, Token::DotDot | Token::DotDotEqual) {
        return Ok(None);
    }

    let prec = 5;
    if prec < min_prec {
        return Ok(None);
    }

    let inclusive = matches!(token, Token::DotDotEqual);
    state.tokens.advance();
    let end = parse_expr_with_precedence_recursive(state, prec + 1)?;
    Ok(Some(Expr {
        kind: ExprKind::Range {
            start: Box::new(left),
            end: Box::new(end),
            inclusive,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    }))
}
