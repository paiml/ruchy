//! Trait definition parsing
//!
//! Handles parsing of trait (interface) definitions:
//! - Trait declarations: `trait Name { ... }`
//! - Interface declarations: `interface Name { ... }` (alias for trait)
//! - Associated types: `type Item`
//! - Method signatures: `fun method(&self)`
//! - Generic traits: `trait Iterator<T> { ... }`
//!
//! # Examples
//! ```ruchy
//! // Basic trait
//! trait Display {
//!     fun fmt(&self) -> String
//! }
//!
//! // Trait with associated type
//! trait Iterator {
//!     type Item
//!     fun next(&mut self) -> Option<Item>
//! }
//!
//! // Generic trait
//! trait From<T> {
//!     fun from(value: T) -> Self
//! }
//!
//! // Interface (alias for trait)
//! interface Comparable {
//!     fun compare(&self, other: &Self) -> i32
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span, TraitMethod};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse trait definition: trait Name { methods }
/// Complexity: 5 (Toyota Way: <10 ✓)
pub(in crate::frontend::parser) fn parse_trait_definition(state: &mut ParserState) -> Result<Expr> {
    // Parse trait/interface keyword
    let start_span = parse_trait_keyword(state)?;
    let name = parse_trait_name(state)?;
    let type_params = parse_optional_trait_generics(state)?;

    // Parse { associated types and methods }
    state.tokens.expect(&Token::LeftBrace)?;
    let (associated_types, methods) = parse_trait_body_items(state)?;
    state.tokens.expect(&Token::RightBrace)?;

    let trait_methods = convert_to_trait_methods(methods);

    Ok(Expr::new(
        ExprKind::Trait {
            name,
            type_params,
            associated_types,
            methods: trait_methods,
            is_pub: false,
        },
        start_span,
    ))
}

fn parse_trait_keyword(state: &mut ParserState) -> Result<Span> {
    match state.tokens.peek() {
        Some((Token::Trait | Token::Interface, span)) => {
            let span = *span;
            state.tokens.advance();
            Ok(span)
        }
        _ => bail!("Expected 'trait' or 'interface' keyword"),
    }
}

/// Parse trait name after 'trait' keyword
fn parse_trait_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected trait name after 'trait'"),
    }
}

/// Parse optional generic parameters
fn parse_optional_trait_generics(state: &mut ParserState) -> Result<Vec<String>> {
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        super::type_aliases::parse_generic_params(state)
    } else {
        Ok(vec![])
    }
}

/// Parse trait body items (associated types and methods)
fn parse_trait_body_items(state: &mut ParserState) -> Result<(Vec<String>, Vec<String>)> {
    let mut associated_types = Vec::new();
    let mut methods = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        match state.tokens.peek() {
            Some((Token::Type, _)) => {
                associated_types.push(parse_trait_associated_type(state)?);
            }
            Some((Token::Fun | Token::Fn, _)) => {
                methods.push(parse_trait_method(state)?);
            }
            _ => bail!("Expected 'type' or method in trait body"),
        }
    }

    Ok((associated_types, methods))
}

/// Parse single trait method signature (with optional default implementation)
/// Complexity: 8 (Toyota Way: <10 ✓)
fn parse_trait_method(state: &mut ParserState) -> Result<String> {
    // Expect 'fun' or 'fn' keyword
    match state.tokens.peek() {
        Some((Token::Fun | Token::Fn, _)) => {
            state.tokens.advance();
        }
        _ => bail!("Expected 'fun' or 'fn' keyword in trait/interface"),
    }

    // Parse method name (can be identifier or reserved keyword like 'from')
    let method_name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        Some((Token::From, _)) => {
            state.tokens.advance();
            "from".to_string()
        }
        _ => bail!("Expected method name in trait"),
    };

    // Skip method signature (params and return type) and optional body
    let mut depth = 0;
    while state.tokens.peek().is_some() {
        match state.tokens.peek() {
            Some((Token::LeftBrace, _)) => {
                depth += 1;
                state.tokens.advance();
            }
            Some((Token::RightBrace, _)) if depth > 0 => {
                depth -= 1;
                state.tokens.advance();
                if depth == 0 {
                    break; // End of method body
                }
            }
            Some((Token::Type | Token::Fun | Token::Fn | Token::RightBrace, _)) if depth == 0 => {
                break; // Next trait item or end of trait
            }
            _ => {
                state.tokens.advance();
            }
        }
    }

    Ok(method_name)
}

/// Parse trait associated type: type Item
/// Complexity: <5 (Toyota Way: <10 ✓)
fn parse_trait_associated_type(state: &mut ParserState) -> Result<String> {
    // Expect 'type' keyword
    state.tokens.expect(&Token::Type)?;

    // Parse type name (can be identifier or reserved keyword like 'Error', 'Result', 'Item')
    let type_name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        Some((Token::Result, _)) => {
            state.tokens.advance();
            "Result".to_string()
        }
        Some((Token::Err, _)) => {
            state.tokens.advance();
            "Err".to_string()
        }
        _ => bail!("Expected type name after 'type' keyword in trait"),
    };

    // Associated types can optionally have bounds or default: type Item: Display = String
    // For now, skip to next trait item (type or fn) or right brace
    while !matches!(
        state.tokens.peek(),
        Some((Token::Type | Token::Fun | Token::Fn | Token::RightBrace, _))
    ) && state.tokens.peek().is_some()
    {
        state.tokens.advance();
    }

    Ok(type_name)
}

/// Convert method names to `TraitMethod` structs
fn convert_to_trait_methods(methods: Vec<String>) -> Vec<TraitMethod> {
    methods
        .into_iter()
        .map(|name| TraitMethod {
            name,
            params: vec![],
            return_type: None,
            body: None,
            is_pub: true,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_basic_trait() {
        let code = "trait Display { fun fmt(&self) -> String }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic trait should parse");
    }

    #[test]
    fn test_trait_with_associated_type() {
        let code = "trait Iterator { type Item }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Trait with associated type should parse");
    }

    #[test]
    fn test_generic_trait() {
        let code = "trait From<T> { fun from(value: T) -> Self }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic trait should parse");
    }

    #[test]
    fn test_interface_keyword() {
        let code = "interface Comparable { fun compare(&self, other: &Self) -> i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Interface keyword should parse");
    }

    #[test]
    fn test_trait_with_multiple_methods() {
        let code = "trait Animal { fun name(&self) -> String\n fun age(&self) -> i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Trait with multiple methods should parse");
    }

    #[test]
    fn test_trait_with_keyword_method_name() {
        let code = "trait Convertible { fun from(value: String) -> Self }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Trait with 'from' keyword method should parse");
    }

    #[test]
    fn test_empty_trait() {
        let code = "trait Marker { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty trait should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_basic_traits_parse(name in "[A-Z][a-z]+", method in "[a-z]+") {
                let code = format!("trait {name} {{ fun {method}() }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_generic_traits_parse(name in "[A-Z][a-z]+", param in "[A-Z]") {
                let code = format!("trait {name}<{param}> {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_traits_with_associated_types(name in "[A-Z][a-z]+", type_name in "[A-Z][a-z]+") {
                let code = format!("trait {name} {{ type {type_name} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_interface_keyword_parses(name in "[A-Z][a-z]+") {
                let code = format!("interface {name} {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_empty_traits_parse(name in "[A-Z][a-z]+") {
                let code = format!("trait {name} {{}}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
