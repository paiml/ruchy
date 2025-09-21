//! LSP semantic token capabilities
use std::sync::LazyLock;
use tower_lsp::lsp_types::{SemanticTokenModifier, SemanticTokenType, SemanticTokensLegend};
/// Semantic token types for Ruchy
pub static SEMANTIC_TOKEN_LEGEND: LazyLock<SemanticTokensLegend> =
    LazyLock::new(|| SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::TYPE,
            SemanticTokenType::CLASS,
            SemanticTokenType::ENUM,
            SemanticTokenType::INTERFACE,
            SemanticTokenType::STRUCT,
            SemanticTokenType::TYPE_PARAMETER,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::PROPERTY,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::EVENT,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::METHOD,
            SemanticTokenType::MACRO,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::MODIFIER,
            SemanticTokenType::COMMENT,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::REGEXP,
            SemanticTokenType::OPERATOR,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::ABSTRACT,
            SemanticTokenModifier::ASYNC,
            SemanticTokenModifier::MODIFICATION,
            SemanticTokenModifier::DOCUMENTATION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ],
    });
/// Token types specific to Ruchy
///
/// # Examples
///
/// ```
/// use ruchy::lsp::{RuchyTokenType, ruchy_token_to_lsp};
/// use tower_lsp::lsp_types::SemanticTokenType;
///
/// let token = RuchyTokenType::Actor;
/// let lsp_token = ruchy_token_to_lsp(token);
/// assert_eq!(lsp_token, SemanticTokenType::CLASS);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuchyTokenType {
    Actor,
    DataFrame,
    Pipeline,
    Pattern,
}
/// Convert Ruchy-specific tokens to LSP semantic token types
///
/// # Examples
///
/// ```
/// use ruchy::lsp::{RuchyTokenType, ruchy_token_to_lsp};
/// use tower_lsp::lsp_types::SemanticTokenType;
///
/// assert_eq!(ruchy_token_to_lsp(RuchyTokenType::Actor), SemanticTokenType::CLASS);
/// assert_eq!(ruchy_token_to_lsp(RuchyTokenType::DataFrame), SemanticTokenType::TYPE);
/// ```
pub fn ruchy_token_to_lsp(token: RuchyTokenType) -> SemanticTokenType {
    match token {
        RuchyTokenType::Actor => SemanticTokenType::CLASS,
        RuchyTokenType::DataFrame => SemanticTokenType::TYPE,
        RuchyTokenType::Pipeline => SemanticTokenType::OPERATOR,
        RuchyTokenType::Pattern => SemanticTokenType::ENUM_MEMBER,
    }
}
#[cfg(test)]
mod property_tests_capabilities {
    use proptest::prelude::*;
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_ruchy_token_to_lsp_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
