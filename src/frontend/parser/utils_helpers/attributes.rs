//! Attribute and decorator parsing functions
//!
//! This module contains all attribute/decorator parsing logic extracted from utils.rs
//! to reduce file complexity and improve maintainability.

use super::super::{bail, Attribute, ParserState, Result, Token};

/// Parse attributes (@-style decorators and #[...] attributes)
///
/// # Errors
///
/// Returns an error if attribute syntax is malformed or contains invalid tokens.
pub fn parse_attributes(state: &mut ParserState) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();
    parse_at_style_decorators(state, &mut attributes)?;
    parse_rust_style_attributes(state, &mut attributes)?;
    Ok(attributes)
}

/// Parse @-style decorators
fn parse_at_style_decorators(
    state: &mut ParserState,
    attributes: &mut Vec<Attribute>,
) -> Result<()> {
    while matches!(state.tokens.peek(), Some((Token::At, _))) {
        let decorator = parse_single_at_decorator(state)?;
        attributes.push(decorator);
    }
    Ok(())
}

/// Parse single @-style decorator
fn parse_single_at_decorator(state: &mut ParserState) -> Result<Attribute> {
    let span = state
        .tokens
        .peek()
        .expect("peek() should return Some after matches! check in caller")
        .1;
    state.tokens.advance(); // consume @

    let name = parse_decorator_name(state)?;
    let args = parse_decorator_arguments(state)?;

    Ok(Attribute { name, args, span })
}

/// Parse decorator name
fn parse_decorator_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected identifier after '@'"),
    }
}

/// Parse decorator arguments
fn parse_decorator_arguments(state: &mut ParserState) -> Result<Vec<String>> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        return Ok(Vec::new());
    }

    state.tokens.advance(); // consume (
    let mut args = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        let arg = parse_single_decorator_argument(state)?;
        args.push(arg);
        consume_argument_separator(state)?;
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(args)
}

/// Parse single decorator argument
fn parse_single_decorator_argument(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::String(s), _)) => {
            let arg = s.clone();
            state.tokens.advance();
            Ok(arg)
        }
        Some((Token::Identifier(id), _)) => {
            let arg = id.clone();
            state.tokens.advance();
            Ok(arg)
        }
        _ => bail!("Expected string or identifier in decorator arguments"),
    }
}

/// Consume argument separator (comma or end of list)
fn consume_argument_separator(state: &mut ParserState) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        Ok(())
    } else if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        Ok(())
    } else {
        bail!("Expected ',' or ')' in decorator arguments")
    }
}

/// Parse Rust-style attributes (#[...])
///
/// Complexity: 2
fn parse_rust_style_attributes(
    state: &mut ParserState,
    _attributes: &mut Vec<Attribute>,
) -> Result<()> {
    if matches!(state.tokens.peek(), Some((Token::AttributeStart, _))) {
        bail!(
            "Attributes are not supported. \
             Ruchy does not use Rust-style attributes like #[derive]. \
             Use @decorator syntax instead."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn test_parse_at_decorator_simple() {
        let mut parser = Parser::new("@deprecated fn foo() { 42 }");
        let expr = parser.parse().unwrap();
        // Decorator should be attached to function - attrs are on expr, not inside kind
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { name, .. } => {
                assert_eq!(name, "foo");
                assert_eq!(expr.attributes.len(), 1);
                assert_eq!(expr.attributes[0].name, "deprecated");
            }
            _ => panic!("Expected function definition with decorator"),
        }
    }

    #[test]
    fn test_parse_at_decorator_with_args() {
        let mut parser = Parser::new("@test(\"unit\") fn bar() { 1 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes.len(), 1);
                assert_eq!(expr.attributes[0].name, "test");
                assert_eq!(expr.attributes[0].args, vec!["unit".to_string()]);
            }
            _ => panic!("Expected function definition with decorator"),
        }
    }

    #[test]
    fn test_parse_at_decorator_multiple_args() {
        let mut parser = Parser::new("@config(\"key\", \"value\") fn baz() { 0 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes.len(), 1);
                assert_eq!(expr.attributes[0].args.len(), 2);
                assert_eq!(expr.attributes[0].args[0], "key");
                assert_eq!(expr.attributes[0].args[1], "value");
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_multiple_decorators() {
        let mut parser = Parser::new("@deprecated @inline fn qux() { 42 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes.len(), 2);
                assert_eq!(expr.attributes[0].name, "deprecated");
                assert_eq!(expr.attributes[1].name, "inline");
            }
            _ => panic!("Expected function definition with decorators"),
        }
    }

    #[test]
    fn test_parse_decorator_with_identifier_arg() {
        let mut parser = Parser::new("@route(my_method) fn process_request() { 1 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes[0].name, "route");
                assert_eq!(expr.attributes[0].args, vec!["my_method".to_string()]);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_no_decorators() {
        let mut parser = Parser::new("fn simple() { 42 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert!(expr.attributes.is_empty());
            }
            _ => panic!("Expected function definition"),
        }
    }

    // === EXTREME TDD Round 18 tests ===

    #[test]
    fn test_parse_at_decorator_empty_args() {
        let mut parser = Parser::new("@test() fn empty_args() { 1 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes.len(), 1);
                assert_eq!(expr.attributes[0].name, "test");
                assert!(expr.attributes[0].args.is_empty());
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parse_decorator_on_let() {
        // Decorators can be on various statements
        let mut parser = Parser::new("@memoize let x = 42");
        let expr = parser.parse().unwrap();
        // Should have decorator attached
        assert_eq!(expr.attributes.len(), 1);
        assert_eq!(expr.attributes[0].name, "memoize");
    }

    #[test]
    fn test_decorator_span_preserved() {
        let mut parser = Parser::new("@test fn foo() { 1 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                // Span should have valid position (non-zero indicates parsed correctly)
                // Position depends on tokenizer implementation
                assert!(expr.attributes[0].span.start < 20);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_multiple_decorators_with_args() {
        let mut parser = Parser::new("@route(\"GET\") @auth(\"admin\") fn api() { 0 }");
        let expr = parser.parse().unwrap();
        match &expr.kind {
            crate::frontend::ast::ExprKind::Function { .. } => {
                assert_eq!(expr.attributes.len(), 2);
                assert_eq!(expr.attributes[0].name, "route");
                assert_eq!(expr.attributes[0].args, vec!["GET"]);
                assert_eq!(expr.attributes[1].name, "auth");
                assert_eq!(expr.attributes[1].args, vec!["admin"]);
            }
            _ => panic!("Expected function definition"),
        }
    }
}
