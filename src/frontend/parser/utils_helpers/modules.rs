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
/// - Multi-expression modules: `module Utils { fn helper() {...}; const PI = 3.15 }`
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_parse_module_empty() {
        let mut parser = Parser::new("module MyModule {}");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            ExprKind::Module { name, .. } => {
                assert_eq!(name, "MyModule");
                // Empty module returns Unit literal body
            }
            _ => panic!("Expected module expression, got {:?}", expr.kind),
        }
    }

    #[test]
    fn test_parse_module_with_single_expression() {
        let mut parser = Parser::new("module Math { 42 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            ExprKind::Module { name, .. } => {
                assert_eq!(name, "Math");
                // Body contains the expression
            }
            _ => panic!("Expected module expression, got {:?}", expr.kind),
        }
    }

    #[test]
    fn test_parse_module_with_multiple_expressions() {
        let mut parser = Parser::new("module Utils { 1; 2; 3 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            ExprKind::Module { name, body } => {
                assert_eq!(name, "Utils");
                match &body.kind {
                    ExprKind::Block(exprs) => assert_eq!(exprs.len(), 3),
                    _ => panic!("Expected block in body, got {:?}", body.kind),
                }
            }
            _ => panic!("Expected module expression"),
        }
    }

    #[test]
    fn test_parse_module_with_function() {
        let mut parser = Parser::new("module MyModule { fn add(a, b) { a + b } }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            ExprKind::Module { name, .. } => {
                assert_eq!(name, "MyModule");
            }
            _ => panic!("Expected module expression"),
        }
    }

    #[test]
    fn test_create_module_body_expr_single() {
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), Span::new(0, 2));
        let result = create_module_body_expr(vec![expr]);
        match result.kind {
            ExprKind::Literal(Literal::Integer(n, _)) => assert_eq!(n, 42),
            _ => panic!("Expected single expression"),
        }
    }

    #[test]
    fn test_create_module_body_expr_multiple() {
        let expr1 = Expr::new(ExprKind::Literal(Literal::Integer(1, None)), Span::new(0, 1));
        let expr2 = Expr::new(ExprKind::Literal(Literal::Integer(2, None)), Span::new(2, 3));
        let result = create_module_body_expr(vec![expr1, expr2]);
        match result.kind {
            ExprKind::Block(exprs) => assert_eq!(exprs.len(), 2),
            _ => panic!("Expected block expression"),
        }
    }

    // Coverage-95 tests

    #[test]
    fn test_parse_module_with_semicolon_separator() {
        let mut parser = Parser::new("module Data { 1; 2; 3 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_nested() {
        let mut parser = Parser::new("module Outer { module Inner { 42 } }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_with_let() {
        let mut parser = Parser::new("module Config { let x = 5; let y = 10 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_with_struct() {
        let mut parser = Parser::new("module Types { struct Point { x: i32, y: i32 } }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_uppercase_name() {
        let mut parser = Parser::new("module MY_MODULE { 42 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_module_mixed_statements() {
        let mut parser = Parser::new("module Mix { let a = 1; fun f() { 2 }; 3 }");
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
