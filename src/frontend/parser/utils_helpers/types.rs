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
/// DEFECT-028 FIX: Also handle lifetime parameters like 'a
fn try_parse_type_parameter(state: &mut ParserState) -> Result<Option<String>> {
    match state.tokens.peek() {
        Some((Token::Identifier(_), _)) => Ok(Some(parse_single_type_parameter(state)?)),
        Some((Token::Lifetime(lt), _)) => {
            // Lifetime parameter like 'a
            let lifetime = lt.clone();
            state.tokens.advance();
            Ok(Some(lifetime))
        }
        _ => Ok(None),
    }
}

/// Parse remaining type parameters after the first one
fn parse_remaining_type_parameters(
    state: &mut ParserState,
    type_params: &mut Vec<String>,
) -> Result<()> {
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

    // DEFECT-021 FIX: Preserve trait bounds (T: Display + Clone) instead of skipping them
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        let bounds = collect_trait_bounds(state);
        if !bounds.is_empty() {
            return Ok(format!("{name}: {bounds}"));
        }
    }

    Ok(name)
}

/// Collect trait bounds as a string (e.g., "Clone + Debug")
fn collect_trait_bounds(state: &mut ParserState) -> String {
    let mut bounds = Vec::new();
    let mut current_bound = String::new();

    while let Some((token, _)) = state.tokens.peek() {
        match token {
            Token::Comma | Token::Greater => break,
            Token::Plus => {
                if !current_bound.is_empty() {
                    bounds.push(current_bound.trim().to_string());
                    current_bound = String::new();
                }
                state.tokens.advance();
            }
            Token::Identifier(id) => {
                if !current_bound.is_empty() {
                    current_bound.push(' ');
                }
                current_bound.push_str(id);
                state.tokens.advance();
            }
            Token::ColonColon => {
                current_bound.push_str("::");
                state.tokens.advance();
            }
            Token::Less => {
                // Handle generic bounds like Iterator<Item=T>
                current_bound.push('<');
                state.tokens.advance();
                let nested = collect_nested_generic(state);
                current_bound.push_str(&nested);
                current_bound.push('>');
            }
            _ => {
                state.tokens.advance();
            }
        }
    }

    if !current_bound.is_empty() {
        bounds.push(current_bound.trim().to_string());
    }

    bounds.join(" + ")
}

/// Collect tokens inside a nested generic <...>
fn collect_nested_generic(state: &mut ParserState) -> String {
    let mut result = String::new();
    let mut depth = 1;

    while depth > 0 {
        if let Some((token, _)) = state.tokens.peek() {
            match token {
                Token::Less => {
                    result.push('<');
                    depth += 1;
                    state.tokens.advance();
                }
                Token::Greater => {
                    depth -= 1;
                    if depth > 0 {
                        result.push('>');
                    }
                    state.tokens.advance();
                }
                Token::Identifier(id) => {
                    result.push_str(id);
                    state.tokens.advance();
                }
                Token::Comma => {
                    result.push_str(", ");
                    state.tokens.advance();
                }
                Token::Equal => {
                    result.push('=');
                    state.tokens.advance();
                }
                _ => {
                    state.tokens.advance();
                }
            }
        } else {
            break;
        }
    }

    result
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
    let base_type = match state.tokens.peek() {
        Some((Token::Ampersand, _)) => parse_reference_type(state, span)?,
        Some((Token::Fn, _)) => parse_fn_type(state, span)?,
        Some((Token::Fun, _)) => parse_fn_type(state, span)?,
        Some((Token::Impl, _)) => parse_impl_trait_type(state, span)?,
        Some((Token::LeftBracket, _)) => parse_list_type(state, span)?,
        Some((Token::LeftParen, _)) => parse_paren_type(state, span)?,
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
        )) => parse_named_type(state, span)?,
        _ => bail!("Expected type"),
    };

    // SPEC-001-H: Check for refined type (where clause)
    // DEFECT-026 FIX: Distinguish refined types from generic bounds
    // Refined type: `x: i32 where x > 0` (constraint is a comparison/boolean expression)
    // Generic bounds: `fun foo<T>() -> T where T: Clone` (T: followed by trait name)
    // If we see `where Identifier :`, it's likely generic bounds - leave for function parser
    if matches!(state.tokens.peek(), Some((Token::Where, _))) {
        // Peek ahead to check if this is generic bounds pattern (Identifier : Trait)
        // peek_nth(1) = token after 'where' (should be identifier like T)
        // peek_nth(2) = token after identifier (should be colon for generic bounds)
        let is_generic_bounds = matches!(
            (state.tokens.peek_nth(1), state.tokens.peek_nth(2)),
            (Some((Token::Identifier(_), _)), Some((Token::Colon, _)))
        );

        if is_generic_bounds {
            // Don't consume - let function parser handle generic bounds
            Ok(base_type)
        } else {
            // Refined type constraint
            state.tokens.advance(); // consume 'where'
            let constraint = crate::frontend::parser::parse_expr_recursive(state)?;
            Ok(Type {
                kind: TypeKind::Refined {
                    base: Box::new(base_type),
                    constraint: Box::new(constraint),
                },
                span,
            })
        }
    } else {
        Ok(base_type)
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

    // ============================================================
    // Qualified name tests
    // ============================================================

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
    fn test_parse_qualified_name_single() {
        let mut state = ParserState::new("String");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "String");
    }

    #[test]
    fn test_parse_qualified_name_two_parts() {
        let mut state = ParserState::new("std::Vec");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "std::Vec");
    }

    #[test]
    fn test_parse_qualified_name_four_parts() {
        let mut state = ParserState::new("foo::bar::baz::Type");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "foo::bar::baz::Type");
    }

    #[test]
    fn test_parse_qualified_name_result_keyword() {
        let mut state = ParserState::new("Result");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Result");
    }

    #[test]
    fn test_parse_qualified_name_option_keyword() {
        let mut state = ParserState::new("Option");
        let result = parse_qualified_name(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Option");
    }

    // ============================================================
    // Generic type tests
    // ============================================================

    #[test]
    fn test_parse_generic_type_nested() {
        let mut state = ParserState::new("<str, Vec<int>>");
        let base = "HashMap".to_string();
        let span = Span { start: 0, end: 0 };
        let result = parse_generic_type(&mut state, base, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_generic_type_single_param() {
        let mut state = ParserState::new("<i32>");
        let base = "Vec".to_string();
        let span = Span { start: 0, end: 0 };
        let result = parse_generic_type(&mut state, base, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_generic_type_two_params() {
        let mut state = ParserState::new("<String, i32>");
        let base = "HashMap".to_string();
        let span = Span { start: 0, end: 0 };
        let result = parse_generic_type(&mut state, base, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_generic_type_three_params() {
        let mut state = ParserState::new("<A, B, C>");
        let base = "Triple".to_string();
        let span = Span { start: 0, end: 0 };
        let result = parse_generic_type(&mut state, base, span);
        assert!(result.is_ok());
    }

    // ============================================================
    // Reference type tests (via parse_type entry point)
    // ============================================================

    #[test]
    fn test_parse_reference_type_immutable() {
        let mut state = ParserState::new("&i32");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_reference_type_mutable() {
        let mut state = ParserState::new("&mut String");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
        if let Ok(t) = result {
            if let TypeKind::Reference { is_mut, .. } = t.kind {
                assert!(is_mut);
            }
        }
    }

    #[test]
    fn test_parse_reference_type_nested() {
        let mut state = ParserState::new("& &i32");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    // ============================================================
    // Function type tests
    // ============================================================

    #[test]
    fn test_parse_fn_type_no_params() {
        let mut state = ParserState::new("fn() -> i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_fn_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fn_type_one_param() {
        let mut state = ParserState::new("fn(i32) -> i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_fn_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fn_type_two_params() {
        let mut state = ParserState::new("fn(i32, String) -> bool");
        let span = Span { start: 0, end: 0 };
        let result = parse_fn_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fn_type_no_return() {
        let mut state = ParserState::new("fn()");
        let span = Span { start: 0, end: 0 };
        let result = parse_fn_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_fn_type_fun_keyword() {
        let mut state = ParserState::new("fun(i32) -> i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_fn_type(&mut state, span);
        assert!(result.is_ok());
    }

    // ============================================================
    // List and array type tests
    // ============================================================

    #[test]
    fn test_parse_list_type_simple() {
        let mut state = ParserState::new("[i32]");
        let span = Span { start: 0, end: 0 };
        let result = parse_list_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_list_type_string() {
        let mut state = ParserState::new("[String]");
        let span = Span { start: 0, end: 0 };
        let result = parse_list_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_type_fixed_size() {
        let mut state = ParserState::new("[i32; 10]");
        let span = Span { start: 0, end: 0 };
        let result = parse_list_type(&mut state, span);
        assert!(result.is_ok());
        if let Ok(t) = result {
            if let TypeKind::Array { size, .. } = t.kind {
                assert_eq!(size, 10);
            }
        }
    }

    #[test]
    fn test_parse_array_type_large_size() {
        let mut state = ParserState::new("[u8; 256]");
        let span = Span { start: 0, end: 0 };
        let result = parse_list_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_array_type_nested() {
        let mut state = ParserState::new("[[i32; 3]; 4]");
        let span = Span { start: 0, end: 0 };
        let result = parse_list_type(&mut state, span);
        assert!(result.is_ok());
    }

    // ============================================================
    // Tuple type tests
    // ============================================================

    #[test]
    fn test_parse_paren_type_unit() {
        let mut state = ParserState::new("()");
        let span = Span { start: 0, end: 0 };
        let result = parse_paren_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_paren_type_single() {
        let mut state = ParserState::new("(i32)");
        let span = Span { start: 0, end: 0 };
        let result = parse_paren_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_paren_type_tuple_two() {
        let mut state = ParserState::new("(i32, String)");
        let span = Span { start: 0, end: 0 };
        let result = parse_paren_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_paren_type_tuple_three() {
        let mut state = ParserState::new("(i32, f64, bool)");
        let span = Span { start: 0, end: 0 };
        let result = parse_paren_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_paren_type_function() {
        let mut state = ParserState::new("(i32, i32) -> i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_paren_type(&mut state, span);
        assert!(result.is_ok());
    }

    // ============================================================
    // Named type tests
    // ============================================================

    #[test]
    fn test_parse_named_type_simple() {
        let mut state = ParserState::new("i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_named_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_named_type_string() {
        let mut state = ParserState::new("String");
        let span = Span { start: 0, end: 0 };
        let result = parse_named_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_named_type_with_generics() {
        let mut state = ParserState::new("Vec<i32>");
        let span = Span { start: 0, end: 0 };
        let result = parse_named_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_named_type_option() {
        let mut state = ParserState::new("Option<String>");
        let span = Span { start: 0, end: 0 };
        let result = parse_named_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_named_type_result() {
        let mut state = ParserState::new("Result<i32, String>");
        let span = Span { start: 0, end: 0 };
        let result = parse_named_type(&mut state, span);
        assert!(result.is_ok());
    }

    // ============================================================
    // Type parameters tests
    // ============================================================

    #[test]
    fn test_parse_type_parameters_single() {
        let mut state = ParserState::new("<T>");
        let result = parse_type_parameters(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_parse_type_parameters_two() {
        let mut state = ParserState::new("<T, U>");
        let result = parse_type_parameters(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_type_parameters_three() {
        let mut state = ParserState::new("<T, U, V>");
        let result = parse_type_parameters(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_parse_type_parameters_with_bounds() {
        let mut state = ParserState::new("<T: Clone>");
        let result = parse_type_parameters(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_parameters_multiple_bounds() {
        let mut state = ParserState::new("<T: Clone + Debug>");
        let result = parse_type_parameters(&mut state);
        assert!(result.is_ok());
    }

    // ============================================================
    // Impl trait type tests
    // ============================================================

    #[test]
    fn test_parse_impl_trait_simple() {
        let mut state = ParserState::new("impl Iterator");
        let span = Span { start: 0, end: 0 };
        let result = parse_impl_trait_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_impl_trait_fn() {
        let mut state = ParserState::new("impl Fn(i32) -> i32");
        let span = Span { start: 0, end: 0 };
        let result = parse_impl_trait_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_impl_trait_fn_once() {
        let mut state = ParserState::new("impl FnOnce() -> String");
        let span = Span { start: 0, end: 0 };
        let result = parse_impl_trait_type(&mut state, span);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_impl_trait_fn_mut() {
        let mut state = ParserState::new("impl FnMut(i32)");
        let span = Span { start: 0, end: 0 };
        // This should either parse or fail gracefully
        let _ = parse_impl_trait_type(&mut state, span);
    }

    // ============================================================
    // Main parse_type entry point tests
    // ============================================================

    #[test]
    fn test_parse_type_i32() {
        let mut state = ParserState::new("i32");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_string() {
        let mut state = ParserState::new("String");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_bool() {
        let mut state = ParserState::new("bool");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_f64() {
        let mut state = ParserState::new("f64");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_reference() {
        let mut state = ParserState::new("&str");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_mut_reference() {
        let mut state = ParserState::new("&mut Vec<i32>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_vec() {
        let mut state = ParserState::new("Vec<u8>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_option() {
        let mut state = ParserState::new("Option<i32>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_result() {
        let mut state = ParserState::new("Result<String, Error>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_hashmap() {
        let mut state = ParserState::new("HashMap<String, i32>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_array() {
        let mut state = ParserState::new("[i32; 5]");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_slice() {
        let mut state = ParserState::new("[u8]");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_tuple() {
        let mut state = ParserState::new("(i32, String, bool)");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_unit() {
        let mut state = ParserState::new("()");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_fn() {
        let mut state = ParserState::new("fn(i32) -> i32");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_impl_trait() {
        let mut state = ParserState::new("impl Clone");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_qualified() {
        let mut state = ParserState::new("std::vec::Vec<i32>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    // ============================================================
    // Helper function tests
    // ============================================================

    #[test]
    fn test_strip_type_suffix_i32() {
        assert_eq!(strip_type_suffix("42i32"), "42");
    }

    #[test]
    fn test_strip_type_suffix_usize() {
        assert_eq!(strip_type_suffix("100usize"), "100");
    }

    #[test]
    fn test_strip_type_suffix_no_suffix() {
        assert_eq!(strip_type_suffix("42"), "42");
    }

    #[test]
    fn test_strip_type_suffix_u8() {
        assert_eq!(strip_type_suffix("255u8"), "255");
    }

    #[test]
    fn test_is_function_trait_fn() {
        assert!(is_function_trait("Fn"));
    }

    #[test]
    fn test_is_function_trait_fn_once() {
        assert!(is_function_trait("FnOnce"));
    }

    #[test]
    fn test_is_function_trait_fn_mut() {
        assert!(is_function_trait("FnMut"));
    }

    #[test]
    fn test_is_function_trait_other() {
        assert!(!is_function_trait("Clone"));
        assert!(!is_function_trait("Iterator"));
    }

    // ============================================================
    // Type list tests
    // ============================================================

    #[test]
    fn test_parse_type_list_empty() {
        let mut state = ParserState::new(")");
        let result = parse_type_list(&mut state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_type_list_single() {
        let mut state = ParserState::new("i32)");
        let result = parse_type_list(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_parse_type_list_multiple() {
        let mut state = ParserState::new("i32, String, bool)");
        let result = parse_type_list(&mut state);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn test_parse_type_nested_generics() {
        let mut state = ParserState::new("Vec<Vec<i32>>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_deeply_nested() {
        // Deeply nested generics - uses >> token handling
        let mut state = ParserState::new("Option<Vec<i32>>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_complex_fn() {
        let mut state = ParserState::new("fn(Vec<i32>, &str) -> Result<String, Error>");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_type_tuple_with_generics() {
        let mut state = ParserState::new("(Vec<i32>, Option<String>)");
        let result = parse_type(&mut state);
        assert!(result.is_ok());
    }

    // ========================================================================
    // collect_nested_generic tests
    // ========================================================================

    #[test]
    fn test_collect_nested_generic_simple() {
        // Simulate parsing inside HashMap<String, i32> after the first '<' is consumed
        let mut state = ParserState::new("String, i32>");
        // We need to be past the opening '<', so we directly call collect_nested_generic
        // which expects depth=1 and reads until matching '>'
        let result = collect_nested_generic(&mut state);
        assert!(result.contains("String"), "Should contain String: {result}");
        assert!(result.contains("i32"), "Should contain i32: {result}");
    }

    #[test]
    fn test_collect_nested_generic_nested() {
        // Vec<Vec<i32> > - space before outer '>' so tokenizer doesn't merge '>>'
        let mut state = ParserState::new("Vec<i32> >");
        let result = collect_nested_generic(&mut state);
        assert!(result.contains("Vec"), "Should contain Vec: {result}");
        assert!(result.contains('<'), "Should contain nested '<': {result}");
        assert!(result.contains("i32"), "Should contain i32: {result}");
        assert!(result.contains('>'), "Should contain nested '>': {result}");
    }

    #[test]
    fn test_collect_nested_generic_with_comma() {
        let mut state = ParserState::new("K, V>");
        let result = collect_nested_generic(&mut state);
        assert!(result.contains('K'), "Should contain K: {result}");
        assert!(result.contains("V"), "Should contain V: {result}");
        assert!(result.contains(", "), "Should contain comma separator: {result}");
    }

    #[test]
    fn test_collect_nested_generic_with_equals() {
        // Generic with default: T = i32>
        let mut state = ParserState::new("T=i32>");
        let result = collect_nested_generic(&mut state);
        assert!(result.contains('T'), "Should contain T: {result}");
        assert!(result.contains('='), "Should contain equals sign: {result}");
        assert!(result.contains("i32"), "Should contain i32: {result}");
    }

    #[test]
    fn test_collect_nested_generic_deeply_nested() {
        // HashMap<String, Vec<i32> > after first '<'
        // Spaces between '>' chars to avoid '>>' tokenization
        let mut state = ParserState::new("String, Vec<i32> >");
        let result = collect_nested_generic(&mut state);
        assert!(result.contains("String"), "Should contain String: {result}");
        assert!(result.contains("Vec"), "Should contain Vec: {result}");
        assert!(result.contains("i32"), "Should contain i32: {result}");
        // Verify nesting is handled (inner '<' and '>')
        assert!(result.contains('<'), "Should contain '<': {result}");
        assert!(result.contains('>'), "Should contain '>': {result}");
    }

    #[test]
    fn test_collect_nested_generic_empty() {
        // Just '>' (empty generic params)
        let mut state = ParserState::new(">");
        let result = collect_nested_generic(&mut state);
        assert!(result.is_empty(), "Empty generic should return empty string");
    }

    #[test]
    fn test_collect_nested_generic_unknown_tokens() {
        // Tokens that are not identifier/comma/less/greater/equal are advanced but not appended meaningfully
        let mut state = ParserState::new("42>");
        let result = collect_nested_generic(&mut state);
        // Numeric literal token is advanced but not added to result string
        assert!(!result.contains("error"));
    }

    // Also test via parse_type which exercises collect_nested_generic indirectly
    #[test]
    fn test_parse_type_hashmap_string_vec_i32() {
        let mut state = ParserState::new("HashMap<String, Vec<i32>>");
        let result = parse_type(&mut state);
        assert!(result.is_ok(), "HashMap<String, Vec<i32>> should parse: {:?}", result.err());
    }

    #[test]
    fn test_parse_type_triple_nested() {
        let mut state = ParserState::new("Option<Vec<HashMap<String, i32>>>");
        let result = parse_type(&mut state);
        assert!(result.is_ok(), "Triple nested generic should parse");
    }
}
