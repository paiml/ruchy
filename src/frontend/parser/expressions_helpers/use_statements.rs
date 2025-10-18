//! Use statement parsing
//!
//! Handles parsing of Rust-style use statements with support for:
//! - Simple imports: `use std::collections::HashMap`
//! - Wildcard imports: `use std::collections::*`
//! - Aliased imports: `use std::collections::HashMap as Map`
//! - Grouped imports: `use std::{collections, io}`
//! - Nested grouped imports: `use std::collections::{HashMap, BTreeMap}`
//!
//! # Examples
//! ```ruchy
//! // Simple import
//! use std::collections::HashMap
//!
//! // Wildcard
//! use std::collections::*
//!
//! // Aliased
//! use std::collections::HashMap as Map
//!
//! // Grouped
//! use std::{collections::HashMap, io::Read}
//!
//! // Nested grouped
//! use std::collections::{HashMap, BTreeMap, HashSet}
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse use statement
///
/// Entry point for parsing `use` statements.
pub(in crate::frontend::parser) fn parse_use_statement(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'use'
    let start_span = Span { start: 0, end: 0 };

    // Parse the use statement recursively to handle nested grouped imports
    parse_use_path(state, start_span)
}

/// Recursively parse use statement paths with support for nested grouped imports
///
/// Handles: `std::collections::{HashMap, BTreeMap}`
/// Handles: `std::{collections::{HashMap, HashSet}, io::{Read, Write}}`
pub(in crate::frontend::parser) fn parse_use_path(
    state: &mut ParserState,
    start_span: Span,
) -> Result<Expr> {
    // Parse initial module path
    let mut path_parts = vec![];
    parse_use_first_segment(state, &mut path_parts)?;

    // Additional components separated by ::
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::

        // Check for {Item1, Item2} syntax
        if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            return parse_nested_grouped_imports(state, path_parts, start_span);
        } else if matches!(state.tokens.peek(), Some((Token::Star, _))) {
            // Handle wildcard import: use std::collections::*
            state.tokens.advance(); // consume *
            let module_path = path_parts.join("::");
            return Ok(Expr::new(
                ExprKind::ImportAll {
                    module: module_path,
                    alias: "*".to_string(), // Use "*" to indicate wildcard import
                },
                start_span,
            ));
        }
        // After :: we can have identifier, super, self, or any keyword
        parse_use_segment_after_colon(state, &mut path_parts)?;
    }

    let module_path = path_parts.join("::");

    // Check for 'as' alias
    if matches!(state.tokens.peek(), Some((Token::As, _))) {
        state.tokens.advance(); // consume 'as'
        if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
            let alias = alias.clone();
            state.tokens.advance();
            // For aliased imports, we use ImportAll with the alias
            Ok(Expr::new(
                ExprKind::ImportAll {
                    module: module_path,
                    alias,
                },
                start_span,
            ))
        } else {
            bail!("Expected alias name after 'as'");
        }
    } else {
        // Create simple import expression
        Ok(Expr::new(
            ExprKind::Import {
                module: module_path,
                items: None,
            },
            start_span,
        ))
    }
}

/// Parse first segment in use path (identifier or keyword)
fn parse_use_first_segment(state: &mut ParserState, path_parts: &mut Vec<String>) -> Result<()> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            path_parts.push(name.clone());
            state.tokens.advance();
            Ok(())
        }
        Some((token, _)) => {
            let keyword_str = super::identifiers::token_to_keyword_string(token);
            if !keyword_str.is_empty() {
                path_parts.push(keyword_str);
                state.tokens.advance();
                Ok(())
            } else {
                bail!("Expected module path after 'use'")
            }
        }
        None => bail!("Expected module path after 'use'"),
    }
}

/// Parse segment after :: in use path (identifier or keyword)
fn parse_use_segment_after_colon(state: &mut ParserState, path_parts: &mut Vec<String>) -> Result<()> {
    match state.tokens.peek() {
        Some((Token::Identifier(segment), _)) => {
            path_parts.push(segment.clone());
            state.tokens.advance();
            Ok(())
        }
        Some((token, _)) => {
            let keyword_str = super::identifiers::token_to_keyword_string(token);
            if !keyword_str.is_empty() {
                path_parts.push(keyword_str);
                state.tokens.advance();
                Ok(())
            } else {
                bail!("Expected identifier or keyword after '::'")
            }
        }
        None => bail!("Expected identifier after '::'"),
    }
}

/// Parse nested grouped imports: use std::{collections, io}
fn parse_nested_grouped_imports(
    state: &mut ParserState,
    base_path: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    state.tokens.advance(); // consume {

    let mut imports = Vec::new();

    loop {
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            state.tokens.advance(); // consume }
            break;
        }

        // Parse grouped import item
        let item_imports = parse_grouped_import_item(state, &base_path, start_span)?;
        imports.extend(item_imports);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            state.tokens.advance();
            break;
        } else {
            bail!("Expected ',' or '}}' in grouped import");
        }
    }

    // Return a block containing all imports
    Ok(Expr::new(ExprKind::Block(imports), start_span))
}

/// Parse a single item in grouped imports
fn parse_grouped_import_item(
    state: &mut ParserState,
    base_path: &[String],
    start_span: Span,
) -> Result<Vec<Expr>> {
    let identifier = parse_import_identifier(state)?;

    if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::

        if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            parse_nested_grouped_import(state, base_path, identifier, start_span)
        } else {
            parse_path_extension_import(state, base_path, identifier, start_span)
        }
    } else {
        parse_simple_import_with_alias(state, base_path, identifier, start_span)
    }
}

fn parse_import_identifier(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let identifier = name.clone();
        state.tokens.advance();
        Ok(identifier)
    } else {
        bail!("Expected identifier in import list");
    }
}

fn parse_nested_grouped_import(
    state: &mut ParserState,
    base_path: &[String],
    identifier: String,
    start_span: Span,
) -> Result<Vec<Expr>> {
    state.tokens.advance(); // consume {
    let items = parse_nested_import_items(state)?;
    state.tokens.expect(&Token::RightBrace)?;

    let full_module_path = [base_path, &[identifier]].concat().join("::");
    Ok(vec![Expr::new(
        ExprKind::Import {
            module: full_module_path,
            items: Some(items),
        },
        start_span,
    )])
}

fn parse_nested_import_items(state: &mut ParserState) -> Result<Vec<String>> {
    let mut items = Vec::new();

    loop {
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }

        let item_name = parse_import_item_with_alias(state)?;
        items.push(item_name);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        } else {
            bail!("Expected ',' or '}}' in nested import list");
        }
    }

    Ok(items)
}

fn parse_import_item_with_alias(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(item), _)) = state.tokens.peek() {
        let mut item_name = item.clone();
        state.tokens.advance();

        if matches!(state.tokens.peek(), Some((Token::As, _))) {
            state.tokens.advance(); // consume 'as'
            if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
                item_name = format!("{item_name} as {alias}");
                state.tokens.advance();
            }
        }

        Ok(item_name)
    } else {
        bail!("Expected identifier in nested import list");
    }
}

fn parse_path_extension_import(
    state: &mut ParserState,
    base_path: &[String],
    identifier: String,
    start_span: Span,
) -> Result<Vec<Expr>> {
    let mut path_parts = vec![identifier];

    while matches!(state.tokens.peek(), Some((Token::Identifier(_), _))) {
        if let Some((Token::Identifier(segment), _)) = state.tokens.peek() {
            path_parts.push(segment.clone());
            state.tokens.advance();

            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                state.tokens.advance();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    let full_path = [base_path, &path_parts].concat().join("::");
    Ok(vec![Expr::new(
        ExprKind::Import {
            module: full_path,
            items: None,
        },
        start_span,
    )])
}

fn parse_simple_import_with_alias(
    state: &mut ParserState,
    base_path: &[String],
    identifier: String,
    start_span: Span,
) -> Result<Vec<Expr>> {
    let full_module_path = [base_path, &[identifier.clone()]].concat().join("::");

    if matches!(state.tokens.peek(), Some((Token::As, _))) {
        state.tokens.advance(); // consume 'as'
        if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
            let alias = alias.clone();
            state.tokens.advance();
            Ok(vec![Expr::new(
                ExprKind::ImportAll {
                    module: full_module_path,
                    alias,
                },
                start_span,
            )])
        } else {
            bail!("Expected alias after 'as'");
        }
    } else {
        Ok(vec![Expr::new(
            ExprKind::Import {
                module: full_module_path,
                items: None,
            },
            start_span,
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_simple_use() {
        let code = "use std::collections::HashMap";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple use should parse");
    }

    #[test]
    fn test_wildcard_use() {
        let code = "use std::collections::*";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Wildcard use should parse");
    }

    #[test]
    fn test_aliased_use() {
        let code = "use std::collections::HashMap as Map";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Aliased use should parse");
    }

    #[test]
    fn test_grouped_use() {
        let code = "use std::{collections, io}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Grouped use should parse");
    }

    #[test]
    fn test_nested_grouped_use() {
        let code = "use std::collections::{HashMap, BTreeMap}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested grouped use should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_simple_use_parses(module in "[a-z]+", item in "[a-z]+") {
                let code = format!("use {}::{}", module, item);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_wildcard_use_parses(module in "[a-z]+") {
                let code = format!("use {}::*", module);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_aliased_use_parses(module in "[a-z]+", item in "[a-z]+", alias in "[A-Z][a-z]+") {
                let code = format!("use {}::{} as {}", module, item, alias);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_grouped_use_parses(module in "[a-z]+", item1 in "[a-z]+", item2 in "[a-z]+") {
                let code = format!("use {}::{{{}, {}}}", module, item1, item2);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_nested_path_parses(m1 in "[a-z]+", m2 in "[a-z]+", item in "[a-z]+") {
                let code = format!("use {}::{}::{}", m1, m2, item);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
