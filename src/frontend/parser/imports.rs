//! Import statement parsing with comprehensive support for various import syntax
//!
//! Supports:
//! - `import std`
//! - `import std.collections.HashMap`
//! - `from std import println`
//! - `from std.collections import HashMap, HashSet`
//! - `import std.collections.HashMap as Map`
//! - `from std.collections import *`
//! - `import { readFile, writeFile } from fs`

use super::{bail, Expr, ExprKind, ParserState, Result, Token};

/// Parse import statement with dot notation support
/// Handles: `import std`, `import std.collections.HashMap`, `import foo as bar`
pub fn parse_import_statement(state: &mut ParserState) -> Result<Expr> {
    // Import token has already been consumed by the caller
    // Use a default span since we don't have access to the import token span
    let start_span = crate::frontend::ast::Span { start: 0, end: 0 };

    // Parse the module path (dot-separated identifiers)
    let module = parse_module_path(state)?;

    // Check for 'as' alias
    let (final_module, items) = if matches!(state.tokens.peek(), Some((Token::As, _))) {
        state.tokens.advance(); // consume 'as'
        if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
            let alias = alias.clone();
            state.tokens.advance();

            // For aliased imports, we need to handle the path correctly
            // "import std.collections.HashMap as Map" should become "use std::collections::HashMap as Map"
            // The alias applies to the entire import path

            // Split the module path to separate the parent module from the item
            let parts: Vec<&str> = module.split('.').collect();
            if parts.len() > 1 {
                // Has a parent module and an item
                let parent_module = parts[..parts.len() - 1].join(".");
                let item_name = parts[parts.len() - 1];
                // Return the parent module and the aliased item
                (
                    parent_module,
                    Some(vec![format!("{} as {}", item_name, alias)]),
                )
            } else {
                // No parent module, the entire thing is aliased
                (module.clone(), Some(vec![format!("self as {}", alias)]))
            }
        } else {
            bail!("Expected identifier after 'as'");
        }
    } else {
        // No alias
        (module, None)
    };

    Ok(Expr::new(
        ExprKind::Import {
            module: final_module,
            items,
        },
        start_span,
    ))
}

/// Parse from...import statement
/// Handles: `from std import println`, `from std.collections import HashMap, HashSet`
pub fn parse_from_import_statement(state: &mut ParserState) -> Result<Expr> {
    // From token has already been consumed by the caller
    let start_span = crate::frontend::ast::Span { start: 0, end: 0 };

    // Parse the module path
    let module = parse_module_path(state)?;

    // Expect 'import'
    state.tokens.expect(&Token::Import)?;

    // Parse the import items
    let items = if matches!(state.tokens.peek(), Some((Token::Star, _))) {
        parse_wildcard_import_items(state)?
    } else {
        parse_named_import_items(state)?
    };

    Ok(Expr::new(ExprKind::Import { module, items }, start_span))
}

// Helper: Parse wildcard import (from module import *)
fn parse_wildcard_import_items(state: &mut ParserState) -> Result<Option<Vec<String>>> {
    state.tokens.advance(); // consume *
                            // Use an empty vector to indicate wildcard import
    Ok(Some(vec![]))
}

// Helper: Parse named import items (from module import item1, item2)
fn parse_named_import_items(state: &mut ParserState) -> Result<Option<Vec<String>>> {
    let mut import_items = Vec::new();

    loop {
        import_items.push(parse_import_item(state)?);

        // Check for more items
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    Ok(Some(import_items))
}

// Helper: Parse single import item with optional alias (complexity: 3)
fn parse_import_item(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let mut item = name.clone();
        state.tokens.advance();

        // Check for 'as' alias
        if matches!(state.tokens.peek(), Some((Token::As, _))) {
            state.tokens.advance();
            if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
                item = format!("{item} as {alias}");
                state.tokens.advance();
            } else {
                bail!("Expected identifier after 'as'");
            }
        }

        Ok(item)
    } else {
        bail!("Expected identifier in import list");
    }
}

// Helper: Consume optional trailing comma (complexity: 2)
fn consume_import_comma(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
    }
}

// Helper: Parse module source (string or path) (complexity: 2)
fn parse_module_source(state: &mut ParserState) -> Result<String> {
    if let Some((Token::String(path), _)) = state.tokens.peek() {
        let path = path.clone();
        state.tokens.advance();
        Ok(path)
    } else {
        parse_module_path(state)
    }
}

/// Parse JS-style import statement (complexity: 3)
/// Handles: `import { readFile, writeFile } from fs`
pub fn parse_js_style_import(state: &mut ParserState) -> Result<Expr> {
    // Import token has already been consumed by the caller
    let start_span = crate::frontend::ast::Span { start: 0, end: 0 };

    // Expect '{'
    state.tokens.expect(&Token::LeftBrace)?;

    // Parse import items
    let mut items = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let item = parse_import_item(state)?;
        items.push(item);
        consume_import_comma(state);
    }

    state.tokens.expect(&Token::RightBrace)?;
    state.tokens.expect(&Token::From)?;

    // Parse module source
    let module = parse_module_source(state)?;

    Ok(Expr::new(
        ExprKind::Import {
            module,
            items: Some(items),
        },
        start_span,
    ))
}

/// Parse a dot-separated module path
/// Handles: `std`, `std.collections`, `std.collections.HashMap`
fn parse_module_path(state: &mut ParserState) -> Result<String> {
    // Check for string literal first (for compatibility)
    if let Some((Token::String(path), _)) = state.tokens.peek() {
        let path = path.clone();
        state.tokens.advance();
        return Ok(path);
    }

    // Parse dot-separated identifiers
    let mut parts = Vec::new();

    // Special handling for keywords that can be module names
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            parts.push(name.clone());
            state.tokens.advance();
        }
        // Allow some keywords as module names
        Some((token @ (Token::Self_ | Token::Super | Token::Crate), _)) => {
            let name = match token {
                Token::Self_ => "self",
                Token::Super => "super",
                Token::Crate => "crate",
                _ => bail!("Unexpected token in module path"),
            };
            parts.push(name.to_string());
            state.tokens.advance();
        }
        _ => bail!("Expected module path"),
    }

    // Parse additional dot-separated or :: -separated parts
    // DEFECT-PARSER-013 FIX: Accept both . and :: as separators
    while matches!(state.tokens.peek(), Some((Token::Dot, _)))
        || matches!(state.tokens.peek(), Some((Token::ColonColon, _)))
    {
        state.tokens.advance(); // consume dot or ::

        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            parts.push(name.clone());
            state.tokens.advance();
        } else {
            bail!("Expected identifier after '.' or '::' in module path");
        }
    }

    // Join with :: (Rust-style) regardless of input separator
    Ok(parts.join("::"))
}

#[cfg(test)]
mod tests {
    use super::super::Parser;

    // Sprint 8 Phase 1: Mutation test gap coverage for imports.rs
    // Target: 1 MISSED → 0 MISSED (mutation coverage improvement)

    #[test]
    fn test_import_with_crate_keyword() {
        // Test gap: verify Token::Crate match arm (line 222)
        let mut parser = Parser::new("import crate");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse 'import crate' statement");
    }

    #[test]
    fn test_import_with_self_keyword() {
        // Test gap: verify Token::Self_ match arm (line 220)
        let mut parser = Parser::new("import self");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse 'import self' statement");
    }

    #[test]
    fn test_import_with_super_keyword() {
        // Test gap: verify Token::Super match arm (line 221)
        let mut parser = Parser::new("import super");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse 'import super' statement");
    }

    #[test]
    fn test_from_crate_import() {
        // Test gap: verify crate keyword in from...import
        let mut parser = Parser::new("from crate import utils");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse 'from crate import' statement");
    }

    #[test]
    fn test_import_crate_with_path() {
        // Test gap: verify crate keyword with dot notation
        let mut parser = Parser::new("import crate.utils");
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Should parse 'import crate.utils' statement"
        );
    }

    #[test]
    fn test_from_super_import() {
        // Test gap: verify super keyword in from...import
        let mut parser = Parser::new("from super import foo");
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse 'from super import' statement");
    }
}

#[cfg(test)]
mod mutation_tests {
    use super::super::Parser;

    #[test]
    fn test_crate_keyword_deletion() {
        // MISSED: delete match arm Token::Crate in parse_module_path (line 222)
        // NOTE: Existing test_import_with_crate_keyword already tests this,
        // but mutation testing still marks it as MISSED. This may be a limitation
        // of the mutation testing tool or the test needs to verify output more specifically.

        // Attempting to verify the match arm is actually used
        let mut parser = Parser::new("import crate");
        let result = parser.parse();

        // This should succeed with the Token::Crate arm present
        assert!(
            result.is_ok(),
            "Should parse 'import crate' using Token::Crate match arm"
        );
    }
}
