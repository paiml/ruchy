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
fn parse_rust_style_attributes(
    state: &mut ParserState,
    attributes: &mut Vec<Attribute>,
) -> Result<()> {
    while matches!(state.tokens.peek(), Some((Token::AttributeStart, _))) {
        let attribute = parse_single_rust_attribute(state)?;
        attributes.push(attribute);
    }
    Ok(())
}

/// Parse single Rust-style attribute
fn parse_single_rust_attribute(state: &mut ParserState) -> Result<Attribute> {
    state.tokens.advance(); // consume #[ (AttributeStart token)

    let name = parse_rust_attribute_name(state)?;
    let args = parse_rust_attribute_arguments(state)?;

    let end_span = state.tokens.advance().expect("Expected ']' token").1; // consume ]

    Ok(Attribute {
        name,
        args,
        span: end_span,
    })
}

/// Parse Rust attribute name
fn parse_rust_attribute_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Crate, _)) => {
            state.tokens.advance();
            Ok("crate".to_string())
        }
        _ => bail!("Expected attribute name"),
    }
}

/// Parse Rust attribute arguments
fn parse_rust_attribute_arguments(state: &mut ParserState) -> Result<Vec<String>> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        return Ok(Vec::new());
    }

    state.tokens.advance();
    parse_argument_list(state)
}

/// Parse the argument list (extracted to reduce nesting)
fn parse_argument_list(state: &mut ParserState) -> Result<Vec<String>> {
    let mut args = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        args.push(parse_single_argument(state)?);

        if !try_consume_separator(state)? {
            break;
        }
    }

    state.tokens.advance(); // consume )
    Ok(args)
}

/// Parse a single argument (identifier or string)
fn parse_single_argument(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(_), _)) => parse_identifier_with_optional_value(state),
        Some((Token::String(s), _)) => {
            let result = format!("\"{s}\"");
            state.tokens.advance();
            Ok(result)
        }
        _ => bail!("Expected identifier or string in attribute arguments"),
    }
}

/// Parse identifier argument with optional = value
fn parse_identifier_with_optional_value(state: &mut ParserState) -> Result<String> {
    let Some((Token::Identifier(id), _)) = state.tokens.peek() else {
        bail!("Expected identifier");
    };
    let id = id.clone();
    state.tokens.advance();

    if !matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        return Ok(id);
    }

    state.tokens.advance(); // consume =
    let value = parse_simple_value(state)?;
    Ok(format!("{id} = {value}"))
}

/// Parse a simple value (identifier, integer, float, string, bool)
fn parse_simple_value(state: &mut ParserState) -> Result<String> {
    let Some((token, _)) = state.tokens.peek() else {
        bail!("Expected attribute value");
    };

    let value = match token {
        Token::Identifier(v) => v.clone(),
        Token::Integer(v) => v.clone(),
        Token::Float(v) => v.to_string(),
        Token::String(v) => format!("\"{v}\""),
        Token::Bool(v) => v.to_string(),
        _ => bail!("Unsupported attribute value type: {:?}", token),
    };
    state.tokens.advance();
    Ok(value)
}

/// Try to consume a comma separator (returns false if at end of list)
fn try_consume_separator(state: &mut ParserState) -> Result<bool> {
    match state.tokens.peek() {
        Some((Token::Comma, _)) => {
            state.tokens.advance();
            Ok(true)
        }
        Some((Token::RightParen, _)) => Ok(false),
        _ => bail!("Expected ',' or ')' after attribute argument"),
    }
}
