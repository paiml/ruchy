//! Enum definition parsing
//!
//! Handles parsing of enum (algebraic data type) definitions:
//! - Unit variants: `enum Status { Active, Inactive }`
//! - Tuple variants: `enum Message { Write(String), Move(i32, i32) }`
//! - Struct variants: `enum Shape { Circle { radius: f64 }, Rectangle { width: f64, height: f64 } }`
//! - Discriminants: `enum Color { Red = 1, Green = 2, Blue = 3 }`
//! - Generic enums: `enum Option<T> { Some(T), None }`
//!
//! # Examples
//! ```ruchy
//! // Unit variants
//! enum Status {
//!     Active,
//!     Inactive,
//!     Pending
//! }
//!
//! // Tuple variants
//! enum Message {
//!     Quit,
//!     Write(String),
//!     Move(i32, i32)
//! }
//!
//! // Struct variants
//! enum Shape {
//!     Circle { radius: f64 },
//!     Rectangle { width: f64, height: f64 }
//! }
//!
//! // With discriminants
//! enum Priority {
//!     Low = 1,
//!     Medium = 5,
//!     High = 10
//! }
//!
//! // Generic enum
//! enum Result<T, E> {
//!     Ok(T),
//!     Err(E)
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{EnumVariant, EnumVariantKind, Expr, ExprKind, StructField, Type};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, utils, ParserState, Result};

pub(in crate::frontend::parser) fn parse_enum_definition(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Enum)?;
    let name = parse_enum_name(state)?;
    let type_params = super::super::parse_optional_generics(state)?;
    let variants = parse_enum_variants(state)?;
    Ok(Expr::new(
        ExprKind::Enum {
            name,
            type_params,
            variants,
            is_pub: false,
        },
        start_span,
    ))
}
fn parse_enum_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Option, _)) => {
            state.tokens.advance();
            Ok("Option".to_string())
        }
        Some((Token::Result, _)) => {
            state.tokens.advance();
            Ok("Result".to_string())
        }
        _ => bail!("Expected enum name after 'enum'"),
    }
}

fn parse_enum_variants(state: &mut ParserState) -> Result<Vec<EnumVariant>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut variants = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        variants.push(parse_single_variant(state)?);

        // Skip any inline comments after variant definition
        while matches!(state.tokens.peek(), Some((Token::LineComment(_) | Token::BlockComment(_) | Token::DocComment(_) | Token::HashComment(_), _))) {
            state.tokens.advance();
        }

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();

            // Skip comments after comma
            while matches!(state.tokens.peek(), Some((Token::LineComment(_) | Token::BlockComment(_) | Token::DocComment(_) | Token::HashComment(_), _))) {
                state.tokens.advance();
            }
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(variants)
}
fn parse_single_variant(state: &mut ParserState) -> Result<EnumVariant> {
    let variant_name = parse_variant_name(state)?;

    // Determine variant kind based on next token
    let (kind, discriminant) = match state.tokens.peek() {
        // Struct variant: Move { x: i32, y: i32 }
        Some((Token::LeftBrace, _)) => {
            let fields = parse_variant_struct_fields(state)?;
            (EnumVariantKind::Struct(fields), None)
        }
        // Tuple variant: Write(String)
        Some((Token::LeftParen, _)) => {
            let types = parse_variant_tuple_fields(state)?;
            (EnumVariantKind::Tuple(types), None)
        }
        // Discriminant: Quit = 0
        Some((Token::Equal, _)) => {
            state.tokens.advance(); // consume =
            let disc = parse_variant_discriminant(state)?;
            (EnumVariantKind::Unit, disc)
        }
        // Unit variant: Quit
        _ => (EnumVariantKind::Unit, None),
    };

    Ok(EnumVariant {
        name: variant_name,
        kind,
        discriminant,
    })
}
/// Parse discriminant value for enum variant
/// Complexity: <5
fn parse_variant_discriminant(state: &mut ParserState) -> Result<Option<i64>> {
    match state.tokens.peek() {
        Some((Token::Integer(val_str), _)) => {
            let val_str = val_str.clone();
            state.tokens.advance();
            // Parse the integer value
            let (num_part, _type_suffix) =
                if let Some(pos) = val_str.find(|c: char| c.is_alphabetic()) {
                    (&val_str[..pos], Some(val_str[pos..].to_string()))
                } else {
                    (val_str.as_str(), None)
                };
            let value = num_part.parse::<i64>().map_err(|_| {
                anyhow::anyhow!("Invalid integer literal: {num_part}")
            })?;
            Ok(Some(value))
        }
        Some((Token::Minus, _)) => {
            state.tokens.advance(); // consume -
            match state.tokens.peek() {
                Some((Token::Integer(val_str), _)) => {
                    let val_str = val_str.clone();
                    state.tokens.advance();
                    // Parse the integer value
                    let (num_part, _type_suffix) =
                        if let Some(pos) = val_str.find(|c: char| c.is_alphabetic()) {
                            (&val_str[..pos], Some(val_str[pos..].to_string()))
                        } else {
                            (val_str.as_str(), None)
                        };
                    let value = num_part.parse::<i64>().map_err(|_| {
                        anyhow::anyhow!("Invalid integer literal: {num_part}")
                    })?;
                    Ok(Some(-value))
                }
                _ => bail!("Expected integer after - in enum discriminant"),
            }
        }
        _ => bail!("Expected integer value for enum discriminant"),
    }
}
fn parse_variant_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Some, _)) => {
            state.tokens.advance();
            Ok("Some".to_string())
        }
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok("None".to_string())
        }
        Some((Token::Ok, _)) => {
            state.tokens.advance();
            Ok("Ok".to_string())
        }
        Some((Token::Err, _)) => {
            state.tokens.advance();
            Ok("Err".to_string())
        }
        _ => bail!("Expected variant name in enum"),
    }
}
/// Parse tuple variant fields: (String, i32)
fn parse_variant_tuple_fields(state: &mut ParserState) -> Result<Vec<Type>> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut field_types = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        field_types.push(utils::parse_type(state)?);
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::RightParen)?;
    Ok(field_types)
}

/// Parse struct variant fields: { x: i32, y: i32 }
fn parse_variant_struct_fields(state: &mut ParserState) -> Result<Vec<StructField>> {
    use crate::frontend::ast::{StructField, Visibility};

    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field name
        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name in struct variant")
        };

        // Expect colon
        state.tokens.expect(&Token::Colon)?;

        // Parse field type
        let ty = utils::parse_type(state)?;

        fields.push(StructField {
            name,
            ty,
            visibility: Visibility::Public, // Enum variant fields are public
            is_mut: false,
            default_value: None,
            decorators: vec![],
        });

        // Handle comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(fields)
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_unit_enum() {
        let code = "enum Status { Active, Inactive, Pending }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit enum should parse");
    }

    #[test]
    fn test_tuple_variant_enum() {
        let code = "enum Message { Quit, Write(String), Move(i32, i32) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple variant enum should parse");
    }

    #[test]
    fn test_struct_variant_enum() {
        let code = "enum Shape { Circle { radius: f64 }, Rectangle { width: f64, height: f64 } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct variant enum should parse");
    }

    #[test]
    fn test_enum_with_discriminants() {
        let code = "enum Priority { Low = 1, Medium = 5, High = 10 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Enum with discriminants should parse");
    }

    #[test]
    fn test_generic_enum() {
        let code = "enum Option<T> { Some(T), None }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic enum should parse");
    }

    #[test]
    fn test_result_enum() {
        let code = "enum Result<T, E> { Ok(T), Err(E) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Result enum should parse");
    }

    #[test]
    fn test_enum_with_type_bounds() {
        let code = "enum Container<T: Clone> { Value(T), Empty }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Enum with type bounds should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_unit_enums_parse(name in "[A-Z][a-z]+", v1 in "[A-Z][a-z]+", v2 in "[A-Z][a-z]+") {
                let code = format!("enum {} {{ {}, {} }}", name, v1, v2);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_tuple_variant_parses(name in "[A-Z][a-z]+", variant in "[A-Z][a-z]+") {
                let code = format!("enum {} {{ {}(String) }}", name, variant);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_discriminant_enums_parse(name in "[A-Z][a-z]+", v1 in "[A-Z][a-z]+", n1 in 0i32..100) {
                let code = format!("enum {} {{ {} = {} }}", name, v1, n1);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_generic_enums_parse(name in "[A-Z][a-z]+", param in "[A-Z]") {
                let code = format!("enum {}<{}> {{ Some({}), None }}", name, param, param);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_mixed_variant_enums_parse(name in "[A-Z][a-z]+") {
                let code = format!("enum {} {{ Unit, Tuple(i32), Struct {{ x: i32 }} }}", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
