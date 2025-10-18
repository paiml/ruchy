//! Struct definition parsing
//!
//! Handles parsing of struct (record type) definitions:
//! - Named structs: `struct Point { x: f64, y: f64 }`
//! - Tuple structs: `struct Color(u8, u8, u8)`
//! - Unit structs: `struct Marker`
//! - Generic structs: `struct Container<T> { value: T }`
//! - Field visibility: `pub`, `pub(crate)`, `private`
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span, StructField, Type, Visibility};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_struct_variant(
    state: &mut ParserState,
    name: String,
    type_params: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    match state.tokens.peek() {
        Some((Token::LeftParen, _)) => {
            let fields = parse_tuple_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::TupleStruct {
                    name,
                    type_params,
                    fields,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        Some((Token::LeftBrace, _)) => {
            let fields = parse_struct_fields(state)?;
            Ok(Expr::new(
                ExprKind::Struct {
                    name,
                    type_params,
                    fields,
                    derives: Vec::new(),
                    is_pub: false,
                },
                start_span,
            ))
        }
        _ => Ok(Expr::new(
            ExprKind::Struct {
                name,
                type_params,
                fields: Vec::new(),
                derives: Vec::new(),
                is_pub: false,
            },
            start_span,
        )),
    }
}

pub(in crate::frontend::parser) fn parse_struct_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected struct name after 'struct'");
    }
}

fn parse_tuple_struct_fields(state: &mut ParserState) -> Result<Vec<Type>> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        fields.push(utils::parse_type(state)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(fields)
}

fn parse_struct_fields(state: &mut ParserState) -> Result<Vec<StructField>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let (visibility, is_mut) = parse_struct_field_modifiers(state)?;
        let (field_name, field_type, default_value) = parse_single_struct_field(state)?;

        fields.push(StructField {
            name: field_name,
            ty: field_type,
            visibility,
            is_mut,
            default_value,
            decorators: vec![],
        });

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(fields)
}

fn parse_struct_field_modifiers(state: &mut ParserState) -> Result<(Visibility, bool)> {
    let visibility = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        parse_pub_visibility(state)?
    } else if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        parse_private_keyword(state);
        Visibility::Private
    } else {
        Visibility::Private
    };

    let is_mut = parse_mut_modifier(state);
    Ok((visibility, is_mut))
}

fn parse_pub_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::Pub)?;

    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_scoped_visibility(state)
    } else {
        Ok(Visibility::Public)
    }
}

fn parse_scoped_visibility(state: &mut ParserState) -> Result<Visibility> {
    state.tokens.expect(&Token::LeftParen)?;

    let visibility = match state.tokens.peek() {
        Some((Token::Identifier(id), _)) if id == "crate" => {
            state.tokens.advance();
            Visibility::PubCrate
        }
        Some((Token::Identifier(id), _)) if id == "super" => {
            state.tokens.advance();
            Visibility::PubSuper
        }
        _ => Visibility::Public,
    };

    state.tokens.expect(&Token::RightParen)?;
    Ok(visibility)
}

fn parse_mut_modifier(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

fn parse_private_keyword(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Private, _))) {
        state.tokens.advance();
    }
}

pub(in crate::frontend::parser) fn parse_single_struct_field(state: &mut ParserState) -> Result<(String, Type, Option<Expr>)> {
    let field_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected field name");
    };

    state.tokens.expect(&Token::Colon)?;
    let field_type = utils::parse_type(state)?;

    let default_value = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance();
        Some(parse_expr_recursive(state)?)
    } else {
        None
    };

    Ok((field_name, field_type, default_value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_named_struct() {
        let code = "struct Point { x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Named struct should parse");
    }

    #[test]
    fn test_tuple_struct() {
        let code = "struct Color(u8, u8, u8)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple struct should parse");
    }

    #[test]
    fn test_unit_struct() {
        let code = "struct Marker";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit struct should parse");
    }

    #[test]
    fn test_generic_struct() {
        let code = "struct Container<T> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic struct should parse");
    }

    #[test]
    fn test_pub_field() {
        let code = "struct Point { pub x: f64, y: f64 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with pub field should parse");
    }

    #[test]
    fn test_mut_field() {
        let code = "struct Counter { mut count: i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with mut field should parse");
    }

    #[test]
    fn test_field_with_default() {
        let code = "struct Config { timeout: i32 = 30 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct with default value should parse");
    }
}
