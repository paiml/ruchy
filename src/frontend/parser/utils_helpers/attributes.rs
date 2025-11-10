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
    let span = state.tokens.peek().unwrap().1;
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
