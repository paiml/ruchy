//! Type alias and generic parameter parsing
//!
//! Handles parsing of:
//! - Type aliases: `type Name = TargetType`
//! - Generic parameters: `<T, U>`, `<T: Trait>`, `<T: Trait1 + Trait2>`
//! - Type bounds parsing for trait constraints
//! - Optional generic parameter detection
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, utils, ParserState, Result};

/// Parse type alias: type Name = Type
/// Complexity: <5
pub(in crate::frontend::parser) fn parse_type_alias(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Type)?;

    // Parse the alias name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier after 'type'");
    };

    // Expect =
    state.tokens.expect(&Token::Equal)?;

    // Parse the target type
    let target_type = utils::parse_type(state)?;

    let end_span = target_type.span;
    Ok(Expr::new(
        ExprKind::TypeAlias { name, target_type },
        start_span.merge(end_span),
    ))
}

/// Parse optional generic parameters: <T, U, ...>
pub(in crate::frontend::parser) fn parse_optional_generics(
    state: &mut ParserState,
) -> Result<Vec<String>> {
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_generic_params(state)
    } else {
        Ok(vec![])
    }
}

/// Parse generic parameters: <T, U, ...> or <T: Display, U: Debug + Clone>
/// Made public for use by impls and traits modules
pub(in crate::frontend::parser) fn parse_generic_params(
    state: &mut ParserState,
) -> Result<Vec<String>> {
    // Parse <T, U, ...> or <T: Display, U: Debug + Clone>
    state.tokens.expect(&Token::Less)?;
    let mut params = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::Greater, _))) {
        match state.tokens.peek() {
            Some((Token::Lifetime(lt), _)) => {
                params.push(lt.clone());
                state.tokens.advance();
            }
            Some((Token::Identifier(name), _)) => {
                let param_name = name.clone();
                state.tokens.advance();

                // Check for constraints with ':'
                if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                    state.tokens.advance();
                    // Parse bounds: Trait1 + Trait2 + ...
                    parse_type_bounds(state)?;
                }

                params.push(param_name);
            }
            Some((Token::Char(_), _)) => {
                // Legacy handling for char literals as lifetimes
                state.tokens.advance();
            }
            _ => bail!("Expected type parameter or lifetime"),
        }
        // Check for comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::Greater)?;
    Ok(params)
}

/// Parse type bounds: Trait1 + Trait2 + ...
fn parse_type_bounds(state: &mut ParserState) -> Result<Vec<String>> {
    let mut bounds = Vec::new();

    // Parse first bound
    if let Some((Token::Identifier(bound), _)) = state.tokens.peek() {
        bounds.push(bound.clone());
        state.tokens.advance();
    }

    // Parse additional bounds with '+'
    while matches!(state.tokens.peek(), Some((Token::Plus, _))) {
        state.tokens.advance();
        if let Some((Token::Identifier(bound), _)) = state.tokens.peek() {
            bounds.push(bound.clone());
            state.tokens.advance();
        }
    }

    Ok(bounds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_basic_type_alias() {
        let code = "type MyInt = i32";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic type alias should parse");
    }

    #[test]
    fn test_type_alias_with_generic_target() {
        let code = "type StringResult = Result<String>";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Type alias with generic target should parse: {:?}", result.err());
    }

    #[test]
    fn test_generic_with_bounds() {
        let code = "struct Container<T: Display> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic with bounds should parse");
    }

    #[test]
    fn test_generic_multiple_bounds() {
        let code = "struct Container<T: Display + Clone> { value: T }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic with multiple bounds should parse");
    }

    #[test]
    fn test_multiple_generic_params() {
        let code = "struct Pair<T, U> { first: T, second: U }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Multiple generic params should parse");
    }

    #[test]
    fn test_no_generics() {
        let code = "struct Point { x: i32, y: i32 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Non-generic struct should parse");
    }

    #[test]
    fn test_lifetime_parameter() {
        let code = "struct Container<'a> { value: &'a str }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Lifetime parameter should parse");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::frontend::parser::Parser;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_type_alias_never_panics(name in "[A-Z][a-zA-Z0-9]{0,10}", target in "[a-z][a-z0-9]{0,10}") {
            let code = format!("type {} = {}", name, target);
            let _ = Parser::new(&code).parse();
        }

        #[test]
        fn test_generic_params_never_panic(param in "[A-Z]") {
            let code = format!("struct Container<{}> {{ }}", param);
            let _ = Parser::new(&code).parse();
        }

        #[test]
        fn test_bounded_generics_never_panic(param in "[A-Z]", bound in "[A-Z][a-z]{2,8}") {
            let code = format!("struct Container<{}: {}> {{ }}", param, bound);
            let _ = Parser::new(&code).parse();
        }

        #[test]
        fn test_multiple_bounds_never_panic(param in "[A-Z]", bound1 in "[A-Z][a-z]{2,6}", bound2 in "[A-Z][a-z]{2,6}") {
            let code = format!("struct Container<{}: {} + {}> {{ }}", param, bound1, bound2);
            let _ = Parser::new(&code).parse();
        }
    }
}
