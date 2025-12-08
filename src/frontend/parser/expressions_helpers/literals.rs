//! Literal expression parsing
//!
//! Handles parsing of primitive literal values:
//! - Integers with optional type suffixes (42, 100i32, 0xFF)
//! - Floats (3.15, 1e-5)
//! - Strings (regular and raw strings)
//! - F-strings with interpolation
//! - Characters ('a', '\n')
//! - Bytes (b'x')
//! - Booleans (true, false)
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
use crate::frontend::error_recovery::ParseError;
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

// Import f-string parsing from string_operations module
use super::string_operations::parse_fstring_into_parts;

/// Parse literal tokens into expressions
///
/// Handles all primitive literal types with proper type suffix parsing for integers.
///
/// # Examples
/// ```ruchy
/// 42          // Integer
/// 3.15        // Float
/// "hello"     // String
/// 'a'         // Char
/// true        // Bool
/// f"x={x}"    // F-string
/// ```
pub(in crate::frontend::parser) fn parse_literal_token(
    state: &mut ParserState,
    token: &Token,
    span: Span,
) -> Result<Expr> {
    match token {
        Token::Integer(value_str) => {
            state.tokens.advance();
            // Parse integer value and optional type suffix
            let (num_part, type_suffix) =
                if let Some(pos) = value_str.find(|c: char| c.is_alphabetic()) {
                    (&value_str[..pos], Some(value_str[pos..].to_string()))
                } else {
                    (value_str.as_str(), None)
                };
            let value = num_part.parse::<i64>().map_err(|_| {
                ParseError::new(format!("Invalid integer literal: {num_part}"), span)
            })?;
            Ok(Expr::new(
                ExprKind::Literal(Literal::Integer(value, type_suffix)),
                span,
            ))
        }
        // Issue #168: Hexadecimal literal support (0xFF, 0x1A2B, etc.)
        Token::HexInteger(value_str) => {
            state.tokens.advance();
            // Parse hex value: strip 0x/0X prefix and optional type suffix
            let without_prefix = &value_str[2..]; // Skip "0x" or "0X"
            let (hex_part, type_suffix) = if let Some(pos) = without_prefix.find(['i', 'u']) {
                (
                    &without_prefix[..pos],
                    Some(without_prefix[pos..].to_string()),
                )
            } else {
                (without_prefix, None)
            };
            let value = i64::from_str_radix(hex_part, 16).map_err(|_| {
                ParseError::new(format!("Invalid hexadecimal literal: {value_str}"), span)
            })?;
            Ok(Expr::new(
                ExprKind::Literal(Literal::Integer(value, type_suffix)),
                span,
            ))
        }
        Token::Float(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Float(*value)), span))
        }
        Token::String(value) => {
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::String(value.clone())),
                span,
            ))
        }
        Token::RawString(value) => {
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::String(value.clone())),
                span,
            ))
        }
        Token::FString(template) => {
            state.tokens.advance();
            // Parse f-string template into parts with proper interpolation
            let parts = parse_fstring_into_parts(template)?;
            Ok(Expr::new(ExprKind::StringInterpolation { parts }, span))
        }
        Token::Char(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Char(*value)), span))
        }
        Token::Byte(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Byte(*value)), span))
        }
        Token::Bool(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Bool(*value)), span))
        }
        Token::Atom(value) => {
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::Atom(value.clone())),
                span,
            ))
        }
        _ => bail!("Expected literal token, got: {token:?}"),
    }
}

/// Parse Null literal
pub(in crate::frontend::parser) fn parse_null(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    Ok(Expr::new(ExprKind::Literal(Literal::Null), span))
}

/// Parse None literal
pub(in crate::frontend::parser) fn parse_none(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    Ok(Expr::new(ExprKind::None, span))
}

/// Parse Some constructor: Some(value)
pub(in crate::frontend::parser) fn parse_some_constructor(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    use crate::frontend::parser::parse_expr_with_precedence_recursive;

    state.tokens.advance();
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        bail!("Expected '(' after Some");
    }
    state.tokens.advance();
    let value = parse_expr_with_precedence_recursive(state, 0)?;
    if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        bail!("Expected ')' after Some value");
    }
    state.tokens.advance();
    Ok(Expr::new(
        ExprKind::Some {
            value: Box::new(value),
        },
        span,
    ))
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_integer_literal() {
        let code = "42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Integer literal should parse");
    }

    #[test]
    fn test_integer_with_type_suffix() {
        let code = "100i32";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Integer with type suffix should parse");
    }

    #[test]
    fn test_float_literal() {
        let code = "3.15";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Float literal should parse");
    }

    #[test]
    fn test_string_literal() {
        let code = "\"hello world\"";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "String literal should parse");
    }

    #[test]
    fn test_char_literal() {
        let code = "'a'";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Char literal should parse");
    }

    #[test]
    fn test_bool_literal() {
        let code = "true";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Bool literal should parse");
    }

    #[test]
    fn test_atom_literal() {
        let code = ":status";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Atom literal should parse");
    }

    #[test]
    fn test_fstring_literal() {
        let code = "f\"x={42}\"";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "F-string should parse");
    }

    // Property tests for literals
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_integers_never_panic(n in any::<i32>()) {
                let code = format!("{n}");
                let _ = Parser::new(&code).parse(); // Should not panic
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_floats_never_panic(f in any::<f64>().prop_filter("finite", |x| x.is_finite())) {
                let code = format!("{f}");
                let _ = Parser::new(&code).parse(); // Should not panic
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_strings_never_panic(s in "\\PC*") {
                let code = format!("\"{}\"", s.replace('"', "\\\""));
                let _ = Parser::new(&code).parse(); // Should not panic
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_bools_always_parse(b in any::<bool>()) {
                let code = format!("{b}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_integer_type_suffixes(n in any::<i32>(),
                                          suffix in prop::sample::select(vec!["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"])) {
                let code = format!("{n}{suffix}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Integer with suffix {} should parse", suffix);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_hex_integers_parse(n in 0u32..=0xFFFF) {
                let code = format!("0x{n:X}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Hex integer 0x{:X} should parse", n);
            }
        }
    }
}
