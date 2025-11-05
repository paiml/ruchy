//! Impl block parsing
//!
//! Handles parsing of implementation blocks:
//! - Type implementations: `impl TypeName { methods }`
//! - Trait implementations: `impl TraitName for TypeName { methods }`
//! - Generic implementations: `impl<T> TraitName for TypeName<T> { methods }`
//! - Method definitions within impl blocks
//!
//! # Examples
//! ```ruchy
//! // Type implementation
//! impl Point {
//!     fun new(x: f64, y: f64) -> Point {
//!         Point { x, y }
//!     }
//! }
//!
//! // Trait implementation
//! impl Display for Point {
//!     fun fmt(&self) -> String {
//!         f"Point({self.x}, {self.y})"
//!     }
//! }
//!
//! // Generic implementation
//! impl<T> From<T> for Wrapper<T> {
//!     fun from(value: T) -> Wrapper<T> {
//!         Wrapper { value }
//!     }
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, ImplMethod};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

/// Parse impl block: impl [Trait for] Type { methods }
pub(in crate::frontend::parser) fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Impl)?;

    // Parse optional generic parameters: impl<T> or impl<T: Display>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        super::type_aliases::parse_generic_params(state)?
    } else {
        vec![]
    };

    // Parse impl header (trait and type names)
    let (trait_name, type_name) = parse_impl_header(state)?;

    // Parse impl body (methods)
    state.tokens.expect(&Token::LeftBrace)?;
    let methods = parse_impl_methods(state)?;
    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Impl {
            type_params,
            trait_name,
            for_type: type_name,
            methods,
            is_pub: false,
        },
        start_span,
    ))
}

/// Parse impl header to get trait and type names (complexity: 8)
fn parse_impl_header(state: &mut ParserState) -> Result<(Option<String>, String)> {
    // Parse first identifier (trait or type name)
    let first_name = parse_optional_identifier(state);

    // Check for "for" keyword to determine if first was trait
    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        // DEFECT-PARSER-014 FIX: Use parse_optional_identifier to handle generics
        // on target type: impl<T> Trait for Type<T>
        let type_name = parse_optional_identifier(state)
            .ok_or_else(|| anyhow::anyhow!("Expected type name after 'for' in impl"))?;
        Ok((first_name, type_name))
    } else if let Some(type_name) = first_name {
        // impl Type { ... } case
        Ok((None, type_name))
    } else {
        bail!("Expected type or trait name in impl");
    }
}

/// Parse optional identifier with generic params: Point or Point<T> (complexity: 7)
/// Also accepts keywords that can be trait/type names: From, Default, Option, Result, etc.
fn parse_optional_identifier(state: &mut ParserState) -> Option<String> {
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        // Allow keywords that are valid trait/type names in impl blocks
        Some((Token::From, _)) => { state.tokens.advance(); "From".to_string() }
        Some((Token::Default, _)) => { state.tokens.advance(); "Default".to_string() }
        Some((Token::Option, _)) => { state.tokens.advance(); "Option".to_string() }
        Some((Token::Result, _)) => { state.tokens.advance(); "Result".to_string() }
        Some((Token::Some, _)) => { state.tokens.advance(); "Some".to_string() }
        Some((Token::None, _)) => { state.tokens.advance(); "None".to_string() }
        Some((Token::Ok, _)) => { state.tokens.advance(); "Ok".to_string() }
        Some((Token::Err, _)) => { state.tokens.advance(); "Err".to_string() }
        _ => return None,
    };

    // Check for generic parameters: Point<T>
    let final_name = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        // Parse and append generics to name
        parse_identifier_with_generics(state, name).ok()?
    } else {
        name
    };

    Some(final_name)
}

/// Parse identifier with generic params: Point<T> or Vec<Vec<T> > (complexity: 5)
/// NOTE: Nested generics without spaces like Vec<Vec<T>> are not supported due to
/// lexer tokenizing >> as `RightShift`. Use Vec<Vec<T> > with a space instead.
fn parse_identifier_with_generics(state: &mut ParserState, base_name: String) -> Result<String> {
    state.tokens.expect(&Token::Less)?;
    let mut result = format!("{base_name}<");
    let mut first = true;

    while !matches!(state.tokens.peek(), Some((Token::Greater, _))) {
        if !first {
            result.push_str(", ");
        }
        first = false;

        // Parse type parameter (can be nested like Vec<T>)
        if let Some((Token::Identifier(param), _)) = state.tokens.peek() {
            let param_name = param.clone();
            state.tokens.advance();

            // Check for nested generics: Vec<T>
            if matches!(state.tokens.peek(), Some((Token::Less, _))) {
                let nested = parse_identifier_with_generics(state, param_name)?;
                result.push_str(&nested);
            } else {
                result.push_str(&param_name);
            }
        } else {
            bail!("Expected type parameter in generic list")
        }

        // Handle comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::Greater)?;
    result.push('>');
    Ok(result)
}

/// Parse impl methods (complexity: 7)
fn parse_impl_methods(state: &mut ParserState) -> Result<Vec<ImplMethod>> {
    let mut methods = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Skip any visibility modifiers for now
        // PARSER-008 FIX: Check for pub keyword and capture the flag
        let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
            state.tokens.advance();
            true
        } else {
            false
        };

        // Parse method
        if matches!(state.tokens.peek(), Some((Token::Fun, _)))
            || matches!(state.tokens.peek(), Some((Token::Fn, _)))
        {
            // PARSER-008 FIX: Pass is_pub flag to parse_impl_method
            let method = parse_impl_method(state, is_pub)?;
            methods.push(method);
        } else {
            // Skip unexpected tokens
            state.tokens.advance();
        }
    }

    Ok(methods)
}

/// Parse a single impl method (complexity: 8)
/// PARSER-008 FIX: Accept is_pub parameter to preserve visibility
fn parse_impl_method(state: &mut ParserState, is_pub: bool) -> Result<ImplMethod> {
    // Accept both 'fun' and 'fn' for method definitions
    if matches!(state.tokens.peek(), Some((Token::Fun, _))) {
        state.tokens.expect(&Token::Fun)?;
    } else {
        state.tokens.expect(&Token::Fn)?;
    }

    // Parse method name (accept keywords that can be method names)
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let n = n.clone();
            state.tokens.advance();
            n
        }
        Some((Token::From, _)) => {
            state.tokens.advance();
            "from".to_string()
        }
        Some((Token::Default, _)) => {
            state.tokens.advance();
            "default".to_string()
        }
        _ => bail!("Expected method name after 'fn' in impl block"),
    };

    // Parse parameters
    let params = utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse body
    let body = parse_expr_recursive(state)?;

    // PARSER-008 FIX: Use passed is_pub parameter instead of hardcoded false
    Ok(ImplMethod {
        name,
        params,
        return_type,
        body: Box::new(body),
        is_pub,
    })
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_basic_impl() {
        let code = "impl Point { fun new() -> Point { Point { x: 0, y: 0 } } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic impl should parse");
    }

    #[test]
    fn test_trait_impl() {
        let code = "impl Display for Point { fun fmt(&self) -> String { \"Point\" } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Trait impl should parse");
    }

    #[test]
    fn test_generic_impl() {
        let code = "impl<T> From<T> for Wrapper { fun from(value: T) -> Wrapper { Wrapper { value } } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Generic impl should parse");
    }

    #[test]
    fn test_impl_with_keyword_trait_name() {
        let code = "impl From for MyType { fun from(value: i32) -> MyType { MyType { value } } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Impl with keyword trait name should parse");
    }

    #[test]
    fn test_impl_with_multiple_methods() {
        let code = "impl Point { fun x(&self) -> f64 { self.x } fun y(&self) -> f64 { self.y } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Impl with multiple methods should parse");
    }

    #[test]
    fn test_impl_with_generic_type() {
        let code = "impl Default for Vec<i32> { fun default() -> Vec<i32> { Vec::new() } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Impl with generic type should parse");
    }

    #[test]
    fn test_empty_impl() {
        let code = "impl MyType { }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty impl should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_basic_impls_parse(type_name in "[A-Z][a-z]+", method in "[a-z]+") {
                let code = format!("impl {type_name} {{ fun {method}() {{ 42 }} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_trait_impls_parse(trait_name in "[A-Z][a-z]+", type_name in "[A-Z][a-z]+") {
                let code = format!("impl {trait_name} for {type_name} {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_generic_impls_parse(type_name in "[A-Z][a-z]+", param in "[A-Z]") {
                let code = format!("impl<{param}> {type_name} {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_empty_impls_parse(type_name in "[A-Z][a-z]+") {
                let code = format!("impl {type_name} {{}}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
