//! Unary operator parsing
//!
//! Handles parsing of prefix unary operators:
//! - `-expr` - negation
//! - `!expr` - logical not
//! - `*expr` - dereference
//! - `&expr` - reference
//! - `**expr` - double dereference
//! - `await expr` - await expression
//! - `~expr` - bitwise not
//! - `spawn expr` - spawn actor
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span, UnaryOp};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import parse_expr_with_precedence_recursive from parent module
use crate::frontend::parser::parse_expr_with_precedence_recursive;

/// Parse unary prefix expressions
///
/// Dispatches to specific unary operator handlers.
///
/// # Examples
/// ```ruchy
/// -42        // Negate
/// !true      // Logical not
/// *ptr       // Dereference
/// &value     // Reference
/// **ptr_ptr  // Double deref
/// await fut  // Await
/// ~bits      // Bitwise not
/// spawn actor // Spawn
/// ```
pub(in crate::frontend::parser) fn parse_unary_prefix(
    state: &mut ParserState,
    token: Token,
    span: Span,
) -> Result<Expr> {
    match token {
        Token::Minus => parse_unary_negate(state, span),
        Token::Plus => parse_unary_plus(state, span),
        Token::Bang => parse_unary_not(state, span),
        Token::Star => parse_unary_deref(state, span),
        Token::Ampersand => parse_unary_reference(state, span),
        Token::Power => parse_double_deref(state, span),
        Token::Await => parse_await_expr(state, span),
        Token::Tilde => parse_bitwise_not(state, span),
        Token::Spawn => parse_spawn_expr(state, span),
        _ => unreachable!(),
    }
}

/// Parse unary negation: `-expr`
fn parse_unary_negate(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse unary plus: `+expr` (identity operation - returns operand unchanged)
fn parse_unary_plus(state: &mut ParserState, _span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Unary plus is identity operation - just parse and return the operand
    parse_expr_with_precedence_recursive(state, 13)
}

/// Parse logical not: `!expr`
fn parse_unary_not(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse dereference: `*expr`
fn parse_unary_deref(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse reference: `&expr`
fn parse_unary_reference(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume &

    // PARSER-085: Issue #71 - Check for optional 'mut' keyword after '&'
    let op = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume mut
        UnaryOp::MutableReference
    } else {
        UnaryOp::Reference
    };

    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse double dereference: `**expr`
fn parse_double_deref(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    let inner_deref = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(expr),
        },
        span,
    );
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(inner_deref),
        },
        span,
    ))
}

/// Parse await expression: `await expr`
fn parse_await_expr(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Await {
            expr: Box::new(expr),
        },
        span,
    ))
}

/// Parse bitwise not: `~expr`
fn parse_bitwise_not(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::BitwiseNot,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse spawn expression: `spawn expr`
fn parse_spawn_expr(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Spawn {
            actor: Box::new(expr),
        },
        span,
    ))
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_negate() {
        let code = "-42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Negation should parse");
    }

    #[test]
    fn test_logical_not() {
        let code = "!true";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Logical not should parse");
    }

    #[test]
    fn test_reference() {
        let code = "&x";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Reference should parse");
    }

    #[test]
    fn test_dereference() {
        let code = "*ptr";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Dereference should parse");
    }

    #[test]
    fn test_double_deref() {
        let code = "**ptr_ptr";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Double deref should parse");
    }

    #[test]
    fn test_bitwise_not() {
        let code = "~0xFF";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Bitwise not should parse");
    }

    // Property tests for unary operators
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_negate_parses(n in any::<i32>()) {
                let code = format!("-{n}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Negation -{} should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_not_parses(b in any::<bool>()) {
                let code = format!("!{b}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Logical not !{} should parse", b);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_bitwise_not_parses(n in any::<u32>()) {
                let code = format!("~{n}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Bitwise not ~{} should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_nested_negations_parse(depth in 1..5usize) {
                let negations = "-".repeat(depth);
                let code = format!("{negations}42");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Nested negations {} should parse", code);
            }
        }
    }
}
