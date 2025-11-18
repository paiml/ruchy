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
use crate::frontend::parser::{ParserState, Result};
use crate::frontend::parser::utils::{parse_params, parse_type, parse_type_parameters};
use crate::frontend::parser::collections::parse_block;

/// Parse trait and type names: impl [Trait for] Type
fn parse_impl_target(state: &mut ParserState) -> Result<(Option<String>, String)> {
    let first_ident = expect_identifier(state)?;

    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance(); // consume 'for'
        let type_name = expect_identifier(state)?;
        Ok((Some(first_ident), type_name))
    } else {
        Ok((None, first_ident))
    }
}

/// Parse impl block: impl [Trait for] Type { methods }
pub(in crate::frontend::parser) fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    let start = state.tokens.expect(&Token::Impl)?.start;

    // Parse optional type parameters: impl<T, U>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_type_parameters(state)?
    } else {
        vec![]
    };

    let (trait_name, for_type) = parse_impl_target(state)?;

    // Parse methods block
    state.tokens.expect(&Token::LeftBrace)?;

    let mut methods = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _)) | None) {
        methods.push(parse_impl_method(state)?);
    }

    let end = state.tokens.expect(&Token::RightBrace)?.end;

    Ok(Expr::new(
        ExprKind::Impl {
            type_params,
            trait_name,
            for_type,
            methods,
            is_pub: false,
        },
        crate::frontend::ast::Span::new(start, end),
    ))
}

/// Helper: Expect identifier token
fn expect_identifier(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        use crate::frontend::parser::bail;
        bail!("Expected identifier")
    }
}

/// Parse single method in impl block
///
/// Complexity: 6 (within Toyota Way limits)
fn parse_impl_method(state: &mut ParserState) -> Result<ImplMethod> {
    // Check for pub visibility
    let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
        state.tokens.advance();
        true
    } else {
        false
    };

    // Expect fun keyword
    state.tokens.expect(&Token::Fun)?;

    // Method name
    let name = expect_identifier(state)?;

    // Parameters (parse_params handles both parens)
    let params = parse_params(state)?;

    // Return type
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(parse_type(state)?)
    } else {
        None
    };

    // Body
    let body = Box::new(parse_block(state)?);

    Ok(ImplMethod {
        name,
        params,
        return_type,
        body,
        is_pub,
    })
}
