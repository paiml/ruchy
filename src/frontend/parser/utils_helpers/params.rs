//! Parameter parsing helpers

use crate::frontend::parser::{
    bail, expressions, parse_expr_recursive, Expr, Param, ParserState, Pattern, Result, Span,
    Token, Type, TypeKind,
};

/// Parse function parameters
pub fn parse_params(state: &mut ParserState) -> Result<Vec<Param>> {
    state.tokens.expect(&Token::LeftParen)?;
    let params = parse_param_list(state)?;
    state.tokens.expect(&Token::RightParen)?;
    Ok(params)
}

/// Parse list of parameters (extracted to reduce nesting)
fn parse_param_list(state: &mut ParserState) -> Result<Vec<Param>> {
    let mut params = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        params.push(parse_single_param(state)?);

        if !should_continue_param_list(state)? {
            break;
        }
    }

    Ok(params)
}

/// Parse a single parameter (complexity: 9)
fn parse_single_param(state: &mut ParserState) -> Result<Param> {
    let is_mutable = check_and_consume_mut(state);
    let (pattern, (is_reference, is_ref_mut)) = parse_param_pattern(state)?;
    let mut ty = parse_optional_type_annotation(state)?;

    // For self parameters with references, create the appropriate reference type
    if is_reference {
        if let Pattern::Identifier(name) = &pattern {
            if name == "self" {
                // Create reference type for self parameter
                ty = Type {
                    kind: TypeKind::Reference {
                        is_mut: is_ref_mut,
                        lifetime: None, // Lifetimes on self references not yet supported
                        inner: Box::new(Type {
                            kind: TypeKind::Named("Self".to_string()),
                            span: Span { start: 0, end: 0 },
                        }),
                    },
                    span: ty.span,
                };
            }
        }
    }

    let default_value = parse_optional_default_value(state)?;
    Ok(Param {
        pattern,
        ty,
        span: Span { start: 0, end: 0 },
        is_mutable,
        default_value,
    })
}

/// Check for and consume mut keyword (complexity: 2)
fn check_and_consume_mut(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse parameter pattern (complexity: 8 - increased to support destructuring)
/// Returns (pattern, `reference_info`) where `reference_info` is (`is_reference`, `is_mut`)
fn parse_param_pattern(state: &mut ParserState) -> Result<(Pattern, (bool, bool))> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            // PARSER-087: Check if this is a typed parameter (name: Type)
            // If so, parse as identifier pattern - the & is part of the type annotation, not the pattern
            let name = name.clone();
            state.tokens.advance();
            Ok((Pattern::Identifier(name), (false, false)))
        }
        Some((Token::Ampersand, _)) => {
            // This must be &self or &mut self
            // We don't support other reference patterns in function parameters
            parse_reference_pattern(state)
        }
        Some((Token::DataFrame, _)) => {
            // Handle "df" parameter name (tokenized as DataFrame)
            state.tokens.advance();
            Ok((Pattern::Identifier("df".to_string()), (false, false)))
        }
        Some((Token::Self_, _)) => {
            // Handle "self" parameter name
            state.tokens.advance();
            Ok((Pattern::Identifier("self".to_string()), (false, false)))
        }
        Some((Token::LeftParen, _)) => {
            // Parse tuple destructuring: fun f((x, y)) {}
            let pattern = expressions::parse_tuple_pattern(state)?;
            Ok((pattern, (false, false)))
        }
        Some((Token::LeftBracket, _)) => {
            // Parse list destructuring: fun f([x, y]) {}
            let pattern = expressions::parse_list_pattern(state)?;
            Ok((pattern, (false, false)))
        }
        Some((Token::LeftBrace, _)) => {
            // Parse struct destructuring: fun f({x, y}) {}
            let pattern = expressions::parse_struct_pattern(state)?;
            Ok((pattern, (false, false)))
        }
        Some((Token::Default, _)) => {
            // PARSER-087: Allow 'default' as parameter name (common pattern: default values)
            state.tokens.advance();
            Ok((Pattern::Identifier("default".to_string()), (false, false)))
        }
        Some((Token::From, _)) => {
            bail!(
                "'from' is a reserved keyword (for future import syntax).\n\
                 Suggestion: Use 'from_vertex', 'source', 'start_node', or similar instead.\n\
                 \n\
                 Example:\n\
                 ✗ fun shortest_path(from, to) {{ ... }}  // Error\n\
                 ✓ fun shortest_path(source, target) {{ ... }}  // OK\n\
                 \n\
                 See: https://github.com/paiml/ruchy/issues/23"
            )
        }
        _ => bail!("Function parameters must be simple identifiers or destructuring patterns"),
    }
}

/// Parse reference patterns (&self, &mut self) (complexity: 8)
/// Returns (pattern, `reference_info`) where `reference_info` is (`is_reference`, `is_mut`)
fn parse_reference_pattern(state: &mut ParserState) -> Result<(Pattern, (bool, bool))> {
    state.tokens.advance(); // consume &
    let is_mut_ref = matches!(state.tokens.peek(), Some((Token::Mut, _)));
    if is_mut_ref {
        state.tokens.advance(); // consume mut
    }

    match state.tokens.peek() {
        Some((Token::Self_, _)) => {
            state.tokens.advance();
            // Return "self" as pattern with reference info
            Ok((Pattern::Identifier("self".to_string()), (true, is_mut_ref)))
        }
        Some((Token::Identifier(n), _)) => {
            // For regular identifiers after &, we need to handle them differently
            // This is for parameters like "other: &Type"
            // The & is part of the type, not the parameter pattern
            // So we should not have consumed the & yet
            // This is a design issue - we need to refactor
            let expected = if is_mut_ref {
                "'self' after '&mut'"
            } else {
                "'self' after '&'"
            };
            bail!("Expected {expected} (got identifier '{n}')")
        }
        _ => {
            let expected = if is_mut_ref {
                "'self' after '&mut'"
            } else {
                "'self' after '&'"
            };
            bail!("Expected {expected}")
        }
    }
}

/// Parse optional type annotation (complexity: 4)
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Type> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume :
        crate::frontend::parser::utils::parse_type(state)
    } else {
        // Default to 'Any' type for untyped parameters
        Ok(Type {
            kind: TypeKind::Named("Any".to_string()),
            span: Span { start: 0, end: 0 },
        })
    }
}

/// Parse optional default value (complexity: 3)
fn parse_optional_default_value(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance(); // consume =
        Ok(Some(Box::new(parse_expr_recursive(state)?)))
    } else {
        Ok(None)
    }
}

/// Check if we should continue parsing parameters (complexity: 3)
fn should_continue_param_list(state: &mut ParserState) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_state(source: &str) -> ParserState<'_> {
        ParserState::new(source)
    }

    // Test 1: Parse empty params
    #[test]
    fn test_parse_params_empty() {
        let mut state = create_state("()");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // Test 2: Parse single identifier param
    #[test]
    fn test_parse_params_single_identifier() {
        let mut state = create_state("(x)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].pattern, Pattern::Identifier(n) if n == "x"));
    }

    // Test 3: Parse typed param
    #[test]
    fn test_parse_params_with_type() {
        let mut state = create_state("(x: i32)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].ty.kind, TypeKind::Named(n) if n == "i32"));
    }

    // Test 4: Parse multiple params
    #[test]
    fn test_parse_params_multiple() {
        let mut state = create_state("(x, y, z)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 3);
    }

    // Test 5: Parse mutable param
    #[test]
    fn test_parse_params_mutable() {
        let mut state = create_state("(mut x)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(params[0].is_mutable);
    }

    // Test 6: Parse self param
    #[test]
    fn test_parse_params_self() {
        let mut state = create_state("(self)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].pattern, Pattern::Identifier(n) if n == "self"));
    }

    // Test 7: Parse &self param
    #[test]
    fn test_parse_params_ref_self() {
        let mut state = create_state("(&self)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].ty.kind, TypeKind::Reference { is_mut: false, .. }));
    }

    // Test 8: Parse &mut self param
    #[test]
    fn test_parse_params_ref_mut_self() {
        let mut state = create_state("(&mut self)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].ty.kind, TypeKind::Reference { is_mut: true, .. }));
    }

    // Test 9: Parse df param (DataFrame keyword)
    #[test]
    fn test_parse_params_df_identifier() {
        let mut state = create_state("(df)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].pattern, Pattern::Identifier(n) if n == "df"));
    }

    // Test 10: Parse default as identifier
    #[test]
    fn test_parse_params_default_identifier() {
        let mut state = create_state("(default)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].pattern, Pattern::Identifier(n) if n == "default"));
    }

    // Test 11: Parse param with default value
    #[test]
    fn test_parse_params_with_default_value() {
        let mut state = create_state("(x = 42)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(params[0].default_value.is_some());
    }

    // Test 12: Parse typed param with default value
    #[test]
    fn test_parse_params_typed_with_default() {
        let mut state = create_state("(x: i32 = 0)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].ty.kind, TypeKind::Named(n) if n == "i32"));
        assert!(params[0].default_value.is_some());
    }

    // Test 13: Parse mixed params
    #[test]
    fn test_parse_params_mixed() {
        let mut state = create_state("(x: i32, mut y, z = 5)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 3);
        assert!(!params[0].is_mutable);
        assert!(params[1].is_mutable);
        assert!(params[2].default_value.is_some());
    }

    // Test 14: Error on from keyword
    #[test]
    fn test_parse_params_from_keyword_error() {
        let mut state = create_state("(from)");
        let result = parse_params(&mut state);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("reserved keyword"));
    }

    // Test 15: Error on invalid & pattern (not self)
    #[test]
    fn test_parse_params_ref_non_self_error() {
        let mut state = create_state("(&x)");
        let result = parse_params(&mut state);
        assert!(result.is_err());
    }

    // Test 16: Parse reference type param (not &self)
    #[test]
    fn test_parse_params_reference_type() {
        let mut state = create_state("(x: &str)");
        let result = parse_params(&mut state);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.len(), 1);
        assert!(matches!(&params[0].ty.kind, TypeKind::Reference { .. }));
    }

    // Test 17: check_and_consume_mut with mut
    #[test]
    fn test_check_and_consume_mut_true() {
        let mut state = create_state("mut x");
        let is_mut = check_and_consume_mut(&mut state);
        assert!(is_mut);
    }

    // Test 18: check_and_consume_mut without mut
    #[test]
    fn test_check_and_consume_mut_false() {
        let mut state = create_state("x");
        let is_mut = check_and_consume_mut(&mut state);
        assert!(!is_mut);
    }

    // Test 19: should_continue_param_list with comma
    #[test]
    fn test_should_continue_with_comma() {
        let mut state = create_state(", y");
        let result = should_continue_param_list(&mut state);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // Test 20: should_continue_param_list without comma
    #[test]
    fn test_should_continue_without_comma() {
        let mut state = create_state(")");
        let result = should_continue_param_list(&mut state);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
