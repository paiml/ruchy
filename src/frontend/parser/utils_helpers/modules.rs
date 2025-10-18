//! Module declaration parsing functions
//!
//! This module contains module parsing logic extracted from utils.rs
//! to reduce file complexity and improve maintainability.

use super::super::{bail, Expr, ExprKind, Literal, ParserState, Result, Span, Token};

/// Parse module declarations
///
/// Supports:
/// - Empty modules: `module MyModule {}`
/// - Single expression modules: `module Math { sqrt(x) }`
/// - Multi-expression modules: `module Utils { fn helper() {...}; const PI = 3.14 }`
///
/// # Examples
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::{ExprKind, Literal};
///
/// // Empty module
/// let mut parser = Parser::new("42");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Literal(Literal::Integer(n, None)) => {
///         assert_eq!(*n, 42);
///     }
///     _ => panic!("Expected literal expression"),
/// }
/// ```
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::{ExprKind, Literal};
///
/// // Module with content
/// let mut parser = Parser::new("42");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Literal(Literal::Integer(n, None)) => {
///         assert_eq!(*n, 42);
///     }
///     _ => panic!("Expected literal expression"),
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No identifier follows the module keyword
/// - Missing opening or closing braces
/// - Invalid syntax in module body
pub fn parse_module(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1;
    let name = parse_module_name(state)?;
    state.tokens.expect(&Token::LeftBrace)?;
    let body = parse_module_body(state)?;
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Module { name, body }, start_span))
}

/// Parse module name after 'module' keyword
fn parse_module_name(state: &mut ParserState) -> Result<String> {
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        bail!("Expected module name after 'module'");
    };

    let name = name.clone();
    state.tokens.advance();
    Ok(name)
}

/// Parse module body (empty, single expr, or block)
fn parse_module_body(state: &mut ParserState) -> Result<Box<Expr>> {
    if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        return Ok(Box::new(Expr::new(
            ExprKind::Literal(Literal::Unit),
            Span { start: 0, end: 0 },
        )));
    }

    let exprs = parse_module_expressions(state)?;
    Ok(create_module_body_expr(exprs))
}

/// Parse expressions inside module body
fn parse_module_expressions(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut exprs = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        exprs.push(crate::frontend::parser::parse_expr_recursive(state)?);
        consume_optional_separator(state);
    }

    Ok(exprs)
}

/// Consume optional semicolon or comma separator
fn consume_optional_separator(state: &mut ParserState) {
    if matches!(
        state.tokens.peek(),
        Some((Token::Semicolon | Token::Comma, _))
    ) {
        state.tokens.advance();
    }
}

/// Create module body expression (single or block)
fn create_module_body_expr(exprs: Vec<Expr>) -> Box<Expr> {
    if exprs.len() == 1 {
        Box::new(exprs.into_iter().next().expect("checked: exprs.len() == 1"))
    } else {
        Box::new(Expr::new(ExprKind::Block(exprs), Span { start: 0, end: 0 }))
    }
}
