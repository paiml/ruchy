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
use crate::frontend::parser::collections::parse_block;
use crate::frontend::parser::utils::{parse_params, parse_type, parse_type_parameters};
use crate::frontend::parser::{ParserState, Result};

/// Parse trait and type names: impl [Trait for] Type
/// Handles generic types like Container<T> or Result<T, E>
fn parse_impl_target(state: &mut ParserState) -> Result<(Option<String>, String)> {
    let first_type = expect_type_name_with_generics(state)?;

    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance(); // consume 'for'
        let type_name = expect_type_name_with_generics(state)?;
        Ok((Some(first_type), type_name))
    } else {
        Ok((None, first_type))
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
        // PARSER-XXX: Skip comment tokens in impl blocks
        while matches!(
            state.tokens.peek(),
            Some((
                Token::LineComment(_)
                    | Token::BlockComment(_)
                    | Token::DocComment(_)
                    | Token::HashComment(_),
                _
            ))
        ) {
            state.tokens.advance();
        }
        // Check again after skipping comments
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _)) | None) {
            break;
        }
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

/// Helper: Parse type name including optional generic arguments
/// E.g., "Container" or "Container<T>" or "Result<T, E>"
fn expect_type_name_with_generics(state: &mut ParserState) -> Result<String> {
    let base = expect_identifier(state)?;

    // Check for generic type arguments
    if !matches!(state.tokens.peek(), Some((Token::Less, _))) {
        return Ok(base);
    }

    // Consume the generic arguments: <T, U, ...>
    let mut result = base;
    result.push('<');
    state.tokens.advance(); // consume <

    let mut depth = 1;
    let mut first = true;
    while depth > 0 {
        match state.tokens.peek() {
            Some((Token::Less, _)) => {
                result.push('<');
                depth += 1;
                state.tokens.advance();
            }
            Some((Token::Greater, _)) => {
                result.push('>');
                depth -= 1;
                state.tokens.advance();
            }
            Some((Token::RightShift, _)) => {
                // >> is two > tokens
                result.push_str(">>");
                depth -= 2;
                state.tokens.advance();
                if depth < 0 {
                    use crate::frontend::parser::bail;
                    bail!("Mismatched >> in generic type");
                }
            }
            Some((Token::Comma, _)) => {
                result.push_str(", ");
                state.tokens.advance();
                first = true;
            }
            Some((Token::Identifier(name), _)) => {
                if !first {
                    result.push(' ');
                }
                result.push_str(name);
                state.tokens.advance();
                first = false;
            }
            Some((Token::Colon, _)) => {
                result.push(':');
                state.tokens.advance();
            }
            Some((Token::ColonColon, _)) => {
                result.push_str("::");
                state.tokens.advance();
            }
            Some((_, _)) => {
                // Skip other tokens in generics (like &, mut, etc.)
                state.tokens.advance();
            }
            None => {
                use crate::frontend::parser::bail;
                bail!("Unexpected end of file in generic type arguments");
            }
        }
    }

    Ok(result)
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

    // PARSER-XXX: Accept both 'fun' and 'fn' in impl blocks for consistency
    if !matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
        use crate::frontend::parser::bail;
        bail!("Expected 'fun' or 'fn' keyword in impl method");
    }
    state.tokens.advance();

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
