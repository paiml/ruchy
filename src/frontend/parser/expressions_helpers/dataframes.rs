//! `DataFrame` literal parsing
//!
//! Handles parsing of `DataFrame` literals using the `df!` macro syntax.
//! `DataFrames` are delegated to the collections module for full parsing,
//! while this module handles the dispatch between:
//! - `df![...]` - `DataFrame` literal (delegated to collections)
//! - `df` - `DataFrame` identifier (for method calls, etc.)
//!
//! # Examples
//! ```ruchy
//! df![name => ["Alice", "Bob"], age => [30, 25]]  // DataFrame literal
//! df.filter(...)                                   // DataFrame identifier
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import DataFrame parser from collections module
use crate::frontend::parser::collections;

/// Parse `DataFrame` token: either `df![...]` literal or `df` identifier
///
/// Dispatches based on whether the next token is `!`:
/// - `df!` → Delegate to `collections::parse_dataframe` for literal parsing
/// - `df` → Treat as identifier for method calls
///
/// # Examples
/// ```ruchy
/// df![x => [1, 2, 3]]          // Literal
/// df.select("column")           // Identifier
/// let my_df = df               // Identifier
/// ```
pub(in crate::frontend::parser) fn parse_dataframe_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    // Check if this is df! (literal) or df (identifier)
    if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
        // DataFrame literal: df![...]
        // Delegate to collections module which handles the full syntax
        // Note: collections::parse_dataframe will consume the DataFrame token
        collections::parse_dataframe(state)
    } else {
        // DataFrame identifier: df.method() or df variable reference
        // Consume the DataFrame token since we're handling it as identifier
        state.tokens.advance();
        Ok(Expr::new(
            ExprKind::Identifier("df".to_string()),
            span,
        ))
    }
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_dataframe_literal_empty() {
        let code = "df![]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty DataFrame should parse");
    }

    #[test]
    fn test_dataframe_literal_single_column() {
        let code = r#"df![name => ["Alice", "Bob"]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Single column DataFrame should parse");
    }

    #[test]
    fn test_dataframe_literal_multiple_columns() {
        let code = r#"df![name => ["Alice", "Bob"], age => [30, 25]]"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Multiple column DataFrame should parse");
    }

    #[test]
    fn test_dataframe_identifier() {
        let code = "df";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame identifier should parse");
    }

    #[test]
    fn test_dataframe_method_call() {
        let code = r#"df.select("column")"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame method call should parse");
    }

    #[test]
    fn test_dataframe_assignment() {
        let code = "let my_df = df";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame assignment should parse");
    }

    #[test]
    fn test_dataframe_literal_with_integers() {
        let code = "df![x => [1, 2, 3], y => [4, 5, 6]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "DataFrame with integers should parse");
    }

    // Property tests for DataFrames
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_dataframe_identifier_always_parses(_suffix in "[a-z]{0,10}") {
                let code = "df";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "DataFrame identifier should always parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_empty_dataframe_literal_parses(_n in 0..100usize) {
                let code = "df![]";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "Empty DataFrame literal should parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_single_column_integers_parse(values in prop::collection::vec(any::<i32>(), 1..10)) {
                let values_str = values.iter()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                let code = format!("df![x => [{values_str}]]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Single column with integers {} should parse", code);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_dataframe_method_chain_parses(depth in 1..5usize) {
                let mut code = "df".to_string();
                for _ in 0..depth {
                    code.push_str(".select(\"x\")");
                }
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame method chain {} should parse", code);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_dataframe_column_names_parse(name in "[a-z][a-z0-9_]{0,10}") {
                let code = format!("df![{name} => [1, 2, 3]]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame with column {} should parse", name);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multiple_columns_parse(num_cols in 1..5usize) {
                let columns = (0..num_cols)
                    .map(|i| format!("col{i} => [1, 2]"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let code = format!("df![{columns}]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "DataFrame with {} columns should parse", num_cols);
            }
        }
    }
}
