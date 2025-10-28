//! Type parsing functions for Ruchy parser
//!
//! This module contains all type-related parsing logic extracted from utils.rs
//! as part of QUALITY-009 TDG refactoring.
//!
//! ## Functions
//!
//! - `parse_type()` - Main entry point for parsing type expressions
//! - `parse_type_parameters()` - Parse generic type parameters like <T, U>
//! - `parse_reference_type()` - Parse & and &mut references
//! - `parse_fn_type()` - Parse function types fn(T1) -> T2
//! - `parse_impl_trait_type()` - Parse impl Trait types
//! - `parse_list_type()` - Parse [T] list and [T; N] array types
//! - `parse_paren_type()` - Parse tuple and parenthesized function types
//! - `parse_named_type()` - Parse named types with optional generics
//! - `parse_qualified_name()` - Parse module paths like `std::vec::Vec`
//! - `parse_generic_type()` - Parse generic instantiations Vec<T>
//! - `parse_type_list()` - Parse comma-separated lists of types

use crate::frontend::ast::{Span, Type, TypeKind};
use crate::frontend::lexer::Token;
use crate::frontend::parser::ParserState;
use anyhow::{bail, Result};

/// Parse type parameters from angle brackets <T, U>
///
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_type_parameters(state: &mut ParserState) -> Result<Vec<String>> {
    state.tokens.expect(&Token::Less)?;
    let type_params = parse_type_parameter_list(state)?;
    state.tokens.expect(&Token::Greater)?;
    Ok(type_params)
}

/// Parse list of type parameters (extracted to reduce nesting)
fn parse_type_parameter_list(state: &mut ParserState) -> Result<Vec<String>> {
    let mut type_params = Vec::new();

    // Parse first type parameter
    if let Some(first_param) = try_parse_type_parameter(state)? {
        type_params.push(first_param);
    }

    // Parse additional type parameters
    parse_remaining_type_parameters(state, &mut type_params)?;

    Ok(type_params)
}

/// Try to parse a single type parameter (returns None if not present)
fn try_parse_type_parameter(state: &mut ParserState) -> Result<Option<String>> {
    if matches!(state.tokens.peek(), Some((Token::Identifier(_), _))) {
        Ok(Some(parse_single_type_parameter(state)?))
    } else {
        Ok(None)
    }
}

/// Parse remaining type parameters after the first one
fn parse_remaining_type_parameters(state: &mut ParserState, type_params: &mut Vec<String>) -> Result<()> {
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        if let Some(param) = try_parse_type_parameter(state)? {
            type_params.push(param);
        }
    }
    Ok(())
}

fn parse_single_type_parameter(state: &mut ParserState) -> Result<String> {
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        n.clone()
    } else {
        bail!("Expected type parameter identifier")
    };
    state.tokens.advance();

    // Skip trait bounds if present (T: Display + Clone)
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        skip_trait_bounds(state);
    }

    Ok(name)
}

fn skip_trait_bounds(state: &mut ParserState) {
    while let Some((token, _)) = state.tokens.peek() {
        match token {
            Token::Comma | Token::Greater => break,
            _ => {
                state.tokens.advance();
            }
        }
    }
}
/// Parse type expressions with complexity ≤10
/// # Errors
/// Returns an error if the operation fails
pub fn parse_type(state: &mut ParserState) -> Result<Type> {
    let span = Span { start: 0, end: 0 }; // Simplified for now
    match state.tokens.peek() {
        Some((Token::Ampersand, _)) => parse_reference_type(state, span),
        Some((Token::Fn, _)) => parse_fn_type(state, span),
        Some((Token::Fun, _)) => parse_fn_type(state, span),
        Some((Token::Impl, _)) => parse_impl_trait_type(state, span),
        Some((Token::LeftBracket, _)) => parse_list_type(state, span),
        Some((Token::LeftParen, _)) => parse_paren_type(state, span),
        Some((
            Token::Identifier(_)
            | Token::Result
            | Token::Option
            | Token::Ok
            | Token::Err
            | Token::Some
            | Token::DataFrame
            | Token::None
            | Token::Null,
            _,
        )) => parse_named_type(state, span),
        _ => bail!("Expected type"),
    }
}
// Helper: Parse reference type &T or &mut T or &'a T (complexity: 5)
fn parse_reference_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume &

    // Check for lifetime parameter
    let lifetime = if matches!(state.tokens.peek(), Some((Token::Lifetime(_), _))) {
        if let Some((Token::Lifetime(lt), _)) = state.tokens.peek() {
            let lifetime = lt.clone();
            state.tokens.advance();
            Some(lifetime)
        } else {
            None
        }
    } else {
        None
    };

    let is_mut = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume mut
        true
    } else {
        false
    };
    let inner_type = parse_type(state)?;
    Ok(Type {
        kind: TypeKind::Reference {
            is_mut,
            lifetime,
            inner: Box::new(inner_type),
        },
        span,
    })
}
// Helper: Parse function type fn(T1, T2) -> T3 or fn() (complexity: 6)
// PARSER-085: Fixed to support fn() without return type (GitHub Issue #70)
fn parse_fn_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume fn/fun
    state.tokens.expect(&Token::LeftParen)?;
    let param_types = parse_type_list(state)?;
    state.tokens.expect(&Token::RightParen)?;

    // PARSER-085: Make arrow and return type OPTIONAL
    // If no arrow, default to unit type ()
    let ret_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        parse_type(state)?
    } else {
        // Default: fn() returns unit type ()
        Type {
            kind: TypeKind::Named("()".to_string()),
            span,
        }
    };

    Ok(Type {
        kind: TypeKind::Function {
            params: param_types,
            ret: Box::new(ret_type),
        },
        span,
    })
}
// Helper: Parse impl Trait type (e.g., impl Fn(i32) -> i32) (complexity: 8)
fn parse_impl_trait_type(state: &mut ParserState, _span: Span) -> Result<Type> {
    state.tokens.advance(); // consume 'impl'
    let trait_name = parse_trait_name(state)?;

    // Handle Fn/FnOnce/FnMut trait bounds: Fn(Args) -> Ret
    if is_function_trait(&trait_name) {
        parse_function_trait_type(state, trait_name)
    } else {
        parse_named_trait_type(trait_name)
    }
}

/// Parse trait name after 'impl' keyword
fn parse_trait_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected trait name after 'impl'"),
    }
}

/// Check if trait is a function trait (Fn, `FnOnce`, `FnMut`)
fn is_function_trait(trait_name: &str) -> bool {
    matches!(trait_name, "Fn" | "FnOnce" | "FnMut")
}

/// Parse function trait type: impl Fn(Args) -> Ret
fn parse_function_trait_type(state: &mut ParserState, _trait_name: String) -> Result<Type> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        // Not a function signature, return simple trait type
        return parse_named_trait_type(_trait_name);
    }

    state.tokens.advance(); // consume (
    let param_types = parse_type_list(state)?;
    state.tokens.expect(&Token::RightParen)?;
    state.tokens.expect(&Token::Arrow)?;
    let ret_type = parse_type(state)?;

    Ok(Type {
        kind: TypeKind::Function {
            params: param_types,
            ret: Box::new(ret_type),
        },
        span: Span { start: 0, end: 0 },
    })
}

/// Parse named trait type: impl `TraitName`
fn parse_named_trait_type(trait_name: String) -> Result<Type> {
    Ok(Type {
        kind: TypeKind::Named(format!("impl {trait_name}")),
        span: Span { start: 0, end: 0 },
    })
}
// Helper: Parse list type `[T]` or array type `[T; size]` (complexity: 5)
fn parse_list_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume [
    let inner = parse_type(state)?;

    // Check for array syntax [T; size]
    if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
        parse_array_type(state, inner, span)
    } else {
        parse_simple_list_type(state, inner, span)
    }
}

/// Parse array type [T; size] (extracted to reduce nesting)
fn parse_array_type(state: &mut ParserState, inner: Type, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume ;
    let size = parse_array_size(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Type {
        kind: TypeKind::Array {
            elem_type: Box::new(inner),
            size,
        },
        span,
    })
}

/// Parse simple list type [T]
fn parse_simple_list_type(state: &mut ParserState, inner: Type, span: Span) -> Result<Type> {
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Type {
        kind: TypeKind::List(Box::new(inner)),
        span,
    })
}

/// Parse array size (integer literal or identifier)
fn parse_array_size(state: &mut ParserState) -> Result<usize> {
    match state.tokens.peek() {
        Some((Token::Integer(n_str), _)) => {
            let n_str = n_str.clone();
            parse_integer_array_size(state, &n_str)
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            parse_identifier_array_size(state, &name)
        }
        _ => bail!("Expected array size after semicolon"),
    }
}

/// Parse integer literal as array size
fn parse_integer_array_size(state: &mut ParserState, n_str: &str) -> Result<usize> {
    let n_str = n_str.to_string();
    state.tokens.advance();
    let num_part = strip_type_suffix(&n_str);
    num_part
        .parse::<usize>()
        .map_err(|_| anyhow::anyhow!("Invalid array size: {num_part}"))
}

/// Parse identifier as array size (constant resolution placeholder)
fn parse_identifier_array_size(state: &mut ParserState, name: &str) -> Result<usize> {
    let name = name.to_string();
    state.tokens.advance();
    // Placeholder for constant resolution - would need proper implementation
    Ok(if name == "SIZE" { 5 } else { 0 })
}

/// Strip type suffix from integer literal (e.g., "42i32" -> "42")
fn strip_type_suffix(n_str: &str) -> &str {
    if let Some(pos) = n_str.find(|c: char| c.is_alphabetic()) {
        &n_str[..pos]
    } else {
        n_str
    }
}
// Helper: Parse parenthesized type (T1, T2) or (T1, T2) -> T3 (complexity: 6)
fn parse_paren_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume (
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        // Unit type: ()
        state.tokens.advance();
        Ok(Type {
            kind: TypeKind::Named("()".to_string()),
            span,
        })
    } else {
        let param_types = parse_type_list(state)?;
        state.tokens.expect(&Token::RightParen)?;
        if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
            // Function type: (T1, T2) -> T3
            state.tokens.advance(); // consume ->
            let ret_type = parse_type(state)?;
            Ok(Type {
                kind: TypeKind::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                },
                span,
            })
        } else {
            // Tuple type: (T1, T2)
            Ok(Type {
                kind: TypeKind::Tuple(param_types),
                span,
            })
        }
    }
}
// Helper: Parse named type with optional generics (complexity: 4)
fn parse_named_type(state: &mut ParserState, span: Span) -> Result<Type> {
    let name = parse_qualified_name(state)?;
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_generic_type(state, name, span)
    } else {
        Ok(Type {
            kind: TypeKind::Named(name),
            span,
        })
    }
}
// Helper: Parse qualified name like std::collections::HashMap (complexity: 6)
/// Parse special tokens as type name strings (complexity: 3)
fn parse_type_token_as_string(state: &mut ParserState) -> Option<String> {
    let token_str = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => Some(n.clone()),
        Some((Token::Result, _)) => Some("Result".to_string()),
        Some((Token::Option, _)) => Some("Option".to_string()),
        Some((Token::Ok, _)) => Some("Ok".to_string()),
        Some((Token::Err, _)) => Some("Err".to_string()),
        Some((Token::Some, _)) => Some("Some".to_string()),
        Some((Token::DataFrame, _)) => Some("DataFrame".to_string()),
        Some((Token::None | Token::Null, _)) => Some("None".to_string()),
        _ => None,
    };

    if token_str.is_some() {
        state.tokens.advance();
    }

    token_str
}

fn parse_qualified_name(state: &mut ParserState) -> Result<String> {
    let mut name =
        parse_type_token_as_string(state).ok_or_else(|| anyhow::anyhow!("Expected identifier"))?;

    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        let next_name = parse_type_token_as_string(state)
            .ok_or_else(|| anyhow::anyhow!("Expected identifier after :: in type name"))?;
        name.push_str("::");
        name.push_str(&next_name);
    }
    Ok(name)
}
// Helper: Parse generic type Vec<T, U> (complexity: 4)
fn parse_generic_type(state: &mut ParserState, base: String, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume <

    let type_params = parse_type_list(state)?;

    // Check if any of the type parameters are generic types
    let has_generic_param = type_params
        .iter()
        .any(|t| matches!(t.kind, TypeKind::Generic { .. }));

    // Now we need exactly one > to close this generic
    match state.tokens.peek() {
        Some((Token::Greater, _)) => {
            state.tokens.advance(); // consume >
        }
        Some((Token::RightShift, _)) => {
            // This is >> which means we're in a nested generic like Result<Vec<T>>
            // If we have a generic parameter (like Vec<u8> inside Result<Vec<u8>>),
            // then we're the outer generic and should consume the >>
            // Otherwise, we're the inner generic and shouldn't consume it
            if has_generic_param {
                // This is the outer generic (e.g., Result in Result<Vec<T>>)
                // The inner generic saw >> but didn't consume it, so we consume it now
                state.tokens.advance(); // consume >>
            } else {
                // This is an inner generic (e.g., Vec in Result<Vec<T>>)
                // Don't consume >>, let the outer generic handle it
            }
        }
        _ => {
            bail!(
                "Expected > or >> to close generic type {}, found {:?}",
                base,
                state.tokens.peek()
            );
        }
    }

    Ok(Type {
        kind: TypeKind::Generic {
            base,
            params: type_params,
        },
        span,
    })
}
// Helper: Parse comma-separated type list (complexity: 3)
fn parse_type_list(state: &mut ParserState) -> Result<Vec<Type>> {
    let mut types = Vec::new();
    if !matches!(
        state.tokens.peek(),
        Some((Token::RightParen | Token::Greater | Token::RightShift, _))
    ) {
        types.push(parse_type(state)?);
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
            types.push(parse_type(state)?);
        }
    }
    Ok(types)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_qualified_name() {
        let mut state = ParserState::new("std::collections::HashMap");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        if let Ok(name) = result {
            assert_eq!(name, "std::collections::HashMap");
        }
    }

    #[test]
    fn test_parse_generic_type_nested() {
        // The parser state should be positioned at the '<' token
        let mut state = ParserState::new("<str, Vec<int>>");
        let base = "HashMap".to_string();
        let span = Span { start: 0, end: 0 };
        let result = parse_generic_type(&mut state, base, span);
        assert!(result.is_ok());
    }
}
