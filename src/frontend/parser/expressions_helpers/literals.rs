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

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::{Expr, ExprKind, Literal};
    use crate::frontend::parser::Result;

    fn parse(code: &str) -> Result<Expr> {
        Parser::new(code).parse()
    }

    fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        match &expr.kind {
            ExprKind::Block(exprs) => Some(exprs),
            _ => None,
        }
    }

    // ============================================================
    // Integer literals
    // ============================================================

    #[test]
    fn test_integer_zero() {
        let result = parse("0");
        assert!(result.is_ok(), "Zero should parse");
    }

    #[test]
    fn test_integer_one() {
        let result = parse("1");
        assert!(result.is_ok(), "One should parse");
    }

    #[test]
    fn test_integer_large() {
        let result = parse("999999999");
        assert!(result.is_ok(), "Large integer should parse");
    }

    #[test]
    fn test_integer_negative() {
        let result = parse("-42");
        assert!(result.is_ok(), "Negative integer should parse");
    }

    #[test]
    fn test_integer_suffix_i8() {
        let result = parse("42i8");
        assert!(result.is_ok(), "i8 suffix should parse");
    }

    #[test]
    fn test_integer_suffix_i16() {
        let result = parse("42i16");
        assert!(result.is_ok(), "i16 suffix should parse");
    }

    #[test]
    fn test_integer_suffix_i64() {
        let result = parse("42i64");
        assert!(result.is_ok(), "i64 suffix should parse");
    }

    #[test]
    fn test_integer_suffix_u8() {
        let result = parse("42u8");
        assert!(result.is_ok(), "u8 suffix should parse");
    }

    #[test]
    fn test_integer_suffix_u32() {
        let result = parse("42u32");
        assert!(result.is_ok(), "u32 suffix should parse");
    }

    #[test]
    fn test_integer_suffix_u64() {
        let result = parse("42u64");
        assert!(result.is_ok(), "u64 suffix should parse");
    }

    // ============================================================
    // Hex literals
    // ============================================================

    #[test]
    fn test_hex_zero() {
        let result = parse("0x0");
        assert!(result.is_ok(), "Hex zero should parse");
    }

    #[test]
    fn test_hex_lowercase() {
        let result = parse("0xff");
        assert!(result.is_ok(), "Hex lowercase should parse");
    }

    #[test]
    fn test_hex_uppercase() {
        let result = parse("0xFF");
        assert!(result.is_ok(), "Hex uppercase should parse");
    }

    #[test]
    fn test_hex_mixed_case() {
        let result = parse("0xAbCd");
        assert!(result.is_ok(), "Hex mixed case should parse");
    }

    #[test]
    fn test_hex_long() {
        let result = parse("0x123456");
        assert!(result.is_ok(), "Hex long should parse");
    }

    // ============================================================
    // Float literals
    // ============================================================

    #[test]
    fn test_float_zero() {
        let result = parse("0.0");
        assert!(result.is_ok(), "Float zero should parse");
    }

    #[test]
    fn test_float_one() {
        let result = parse("1.0");
        assert!(result.is_ok(), "Float one should parse");
    }

    #[test]
    fn test_float_pi() {
        let result = parse("3.14159");
        assert!(result.is_ok(), "Float pi should parse");
    }

    #[test]
    fn test_float_small() {
        let result = parse("0.001");
        assert!(result.is_ok(), "Small float should parse");
    }

    #[test]
    fn test_float_negative() {
        let result = parse("-3.14");
        assert!(result.is_ok(), "Negative float should parse");
    }

    // ============================================================
    // String literals
    // ============================================================

    #[test]
    fn test_string_empty() {
        let result = parse("\"\"");
        assert!(result.is_ok(), "Empty string should parse");
    }

    #[test]
    fn test_string_single_char() {
        let result = parse("\"a\"");
        assert!(result.is_ok(), "Single char string should parse");
    }

    #[test]
    fn test_string_with_spaces() {
        let result = parse("\"hello world\"");
        assert!(result.is_ok(), "String with spaces should parse");
    }

    #[test]
    fn test_string_with_numbers() {
        let result = parse("\"test123\"");
        assert!(result.is_ok(), "String with numbers should parse");
    }

    // ============================================================
    // Char literals
    // ============================================================

    #[test]
    fn test_char_letter() {
        let result = parse("'x'");
        assert!(result.is_ok(), "Char letter should parse");
    }

    #[test]
    fn test_char_digit() {
        let result = parse("'5'");
        assert!(result.is_ok(), "Char digit should parse");
    }

    #[test]
    fn test_char_space() {
        let result = parse("' '");
        assert!(result.is_ok(), "Char space should parse");
    }

    // ============================================================
    // Bool literals
    // ============================================================

    #[test]
    fn test_bool_true() {
        let result = parse("true");
        assert!(result.is_ok(), "True should parse");
    }

    #[test]
    fn test_bool_false() {
        let result = parse("false");
        assert!(result.is_ok(), "False should parse");
    }

    // ============================================================
    // None and Some
    // ============================================================

    #[test]
    fn test_none_literal() {
        let result = parse("None");
        assert!(result.is_ok(), "None should parse");
    }

    #[test]
    fn test_some_integer() {
        let result = parse("Some(42)");
        assert!(result.is_ok(), "Some with integer should parse");
    }

    #[test]
    fn test_some_string() {
        let result = parse("Some(\"hello\")");
        assert!(result.is_ok(), "Some with string should parse");
    }

    #[test]
    fn test_some_variable() {
        let result = parse("Some(x)");
        assert!(result.is_ok(), "Some with variable should parse");
    }

    // ============================================================
    // Atom literals
    // ============================================================

    #[test]
    fn test_atom_short() {
        let result = parse(":ok");
        assert!(result.is_ok(), "Short atom should parse");
    }

    #[test]
    fn test_atom_long() {
        let result = parse(":my_atom_name");
        assert!(result.is_ok(), "Long atom should parse");
    }

    #[test]
    fn test_atom_with_numbers() {
        let result = parse(":status_200");
        assert!(result.is_ok(), "Atom with numbers should parse");
    }

    // ============================================================
    // F-string literals
    // ============================================================

    #[test]
    fn test_fstring_simple() {
        let result = parse("f\"hello\"");
        assert!(result.is_ok(), "Simple f-string should parse");
    }

    #[test]
    fn test_fstring_with_var() {
        let result = parse("f\"hello {name}\"");
        assert!(result.is_ok(), "F-string with var should parse");
    }

    #[test]
    fn test_fstring_with_expr() {
        let result = parse("f\"sum = {a + b}\"");
        assert!(result.is_ok(), "F-string with expr should parse");
    }

    #[test]
    fn test_fstring_multiple() {
        let result = parse("f\"{x} + {y} = {z}\"");
        assert!(result.is_ok(), "F-string multiple should parse");
    }

    // ============================================================
    // Literal produces ExprKind::Literal
    // ============================================================

    #[test]
    fn test_integer_produces_literal_exprkind() {
        let expr = parse("42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Literal(Literal::Integer(42, _))),
                "Should produce Literal Integer"
            );
        }
    }

    #[test]
    fn test_bool_produces_literal_exprkind() {
        let expr = parse("true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Literal(Literal::Bool(true))),
                "Should produce Literal Bool"
            );
        }
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
