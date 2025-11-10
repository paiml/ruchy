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

use crate::frontend::ast::Expr;
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse impl block: impl [Trait for] Type { methods }
///
/// Complexity: 1
pub(in crate::frontend::parser) fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    state.tokens.expect(&Token::Impl)?;

    bail!(
        "impl blocks are not supported. \
         In Ruchy, methods should be defined inside the struct body. \
         \n\nExample:\n\
         struct Point {{\n    \
         x: i32,\n    \
         y: i32,\n\n    \
         fun new(x: i32, y: i32) -> Point {{\n        \
         Point {{ x, y }}\n    \
         }}\n\
         }}"
    );
}
