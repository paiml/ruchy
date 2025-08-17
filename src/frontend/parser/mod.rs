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

use crate::frontend::ast::{
    ActorHandler, Attribute, BinaryOp, Expr, ExprKind, ImplMethod, Literal, MatchArm, Param,
    Pattern, PipelineStage, Span, StringPart, StructField, TraitMethod, Type, TypeKind, UnaryOp,
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
}

impl<'a> ParserState<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
            error_recovery: ErrorRecovery::new(),
            errors: Vec::new(),
        }
    }

    /// Get all errors encountered during parsing
    pub fn get_errors(&self) -> &[ErrorNode] {
        &self.errors
    }
}

/// Forward declarations for recursive parsing
pub(crate) fn parse_expr_recursive(state: &mut ParserState) -> Result<Expr> {
    parse_expr_with_precedence_recursive(state, 0)
}

pub(crate) fn parse_expr_with_precedence_recursive(
    state: &mut ParserState,
    min_prec: i32,
) -> Result<Expr> {
    let mut left = expressions::parse_prefix(state)?;

    loop {
        let Some((token, _)) = state.tokens.peek() else {
            break;
        };

        let token_clone = token.clone();

        // Check for actor message passing operators (! and ?) when not followed by (
        // These are special binary-like operators for actors
        if matches!(token_clone, Token::Bang | Token::Question) {
            // Only treat as binary operator if NOT followed by (
            let is_binary_op = if let Some(next_token) = state.tokens.peek_nth(1) {
                !matches!(next_token.0, Token::LeftParen)
            } else {
                true // At end of input, treat as binary
            };

            if is_binary_op {
                let prec = 10; // Same precedence as method calls
                if prec < min_prec {
                    break;
                }

                if matches!(token_clone, Token::Bang) {
                    state.tokens.advance(); // consume !
                    let message = parse_expr_with_precedence_recursive(state, prec + 1)?;
                    left = Expr {
                        kind: ExprKind::Send {
                            actor: Box::new(left),
                            message: Box::new(message),
                        },
                        span: Span { start: 0, end: 0 },
                        attributes: Vec::new(),
                    };
                    continue;
                }
                // Question mark - ask operation
                state.tokens.advance(); // consume ?
                let message = parse_expr_with_precedence_recursive(state, prec + 1)?;
                left = Expr {
                    kind: ExprKind::Ask {
                        actor: Box::new(left),
                        message: Box::new(message),
                        timeout: None,
                    },
                    span: Span { start: 0, end: 0 },
                    attributes: Vec::new(),
                };
                continue;
            }
        }

        // Handle binary operators
        if let Some(bin_op) = expressions::token_to_binary_op(&token_clone) {
            let prec = expressions::get_precedence(bin_op);
            if prec < min_prec {
                break;
            }

            state.tokens.advance(); // consume operator
            let right = parse_expr_with_precedence_recursive(state, prec + 1)?;
            left = Expr {
                kind: ExprKind::Binary {
                    left: Box::new(left),
                    op: bin_op,
                    right: Box::new(right),
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            };
            continue;
        }

        // Check for pipeline operator (|>)
        if matches!(token_clone, Token::Pipeline) {
            let prec = 3; // Low precedence, right-associative
            if prec < min_prec {
                break;
            }
            state.tokens.advance(); // consume |>

            // Parse the next stage as a function/expression with higher precedence
            // to avoid parsing another pipeline
            let stage_expr = parse_expr_with_precedence_recursive(state, prec + 1)?;

            // Build pipeline - if left is already a pipeline, extend it
            left = if let ExprKind::Pipeline { expr, mut stages } = left.kind {
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
                // Create new pipeline
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
            continue;
        }

        // Check for range operators (.., ..=)
        if matches!(token_clone, Token::DotDot | Token::DotDotEqual) {
            let prec = 5; // Lower precedence
            if prec < min_prec {
                break;
            }
            let inclusive = matches!(token_clone, Token::DotDotEqual);
            state.tokens.advance(); // consume .. or ..=
            let end = parse_expr_with_precedence_recursive(state, prec + 1)?;
            left = Expr {
                kind: ExprKind::Range {
                    start: Box::new(left),
                    end: Box::new(end),
                    inclusive,
                },
                span: Span { start: 0, end: 0 },
                attributes: Vec::new(),
            };
            continue;
        }

        break;
    }

    // Handle postfix operators
    loop {
        match state.tokens.peek() {
            Some((Token::Dot, _)) => {
                state.tokens.advance(); // consume .
                left = functions::parse_method_call(state, left)?;
            }
            Some((Token::LeftParen, _)) => {
                left = functions::parse_call(state, left)?;
            }
            Some((Token::LeftBracket, _)) => {
                // Array/list indexing
                state.tokens.advance(); // consume [
                let index = parse_expr_recursive(state)?;
                state.tokens.expect(&Token::RightBracket)?;
                left = Expr {
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
                };
            }
            Some((Token::Try, _)) => {
                state.tokens.advance(); // consume ?
                left = Expr {
                    kind: ExprKind::Try {
                        expr: Box::new(left),
                    },
                    span: Span { start: 0, end: 0 },
                    attributes: Vec::new(),
                };
            }
            Some((Token::LeftBrace, _)) => {
                // Check if left is an identifier starting with uppercase - could be struct literal
                if let ExprKind::Identifier(name) = &left.kind {
                    if name.chars().next().is_some_and(char::is_uppercase) {
                        let name = name.clone();
                        let span = left.span;
                        left = types::parse_struct_literal(state, name, span)?;
                    } else {
                        break; // Not a struct literal
                    }
                } else {
                    break; // Not a struct literal
                }
            }
            _ => break,
        }
    }

    Ok(left)
}
