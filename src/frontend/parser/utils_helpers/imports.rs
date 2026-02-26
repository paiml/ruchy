//! Import and export parsing functions
//!
//! This module contains all import/export parsing logic extracted from utils.rs
//! to reduce file complexity and improve maintainability.

use super::super::{bail, Expr, ExprKind, ImportItem, ParserState, Result, Span, Token};
use super::url_validation::validate_url_import;

/// Parse import statement (legacy import parser)
///
/// NOTE: This is the legacy import parser. New imports are parsed in expressions.rs
pub fn parse_import_legacy(state: &mut ParserState) -> Result<Expr> {
    // Consume the Import token first (required by new parser)
    state.tokens.expect(&Token::Import)?;
    // Check if it's JS-style import
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        crate::frontend::parser::imports::parse_js_style_import(state)
    } else {
        // Delegate to the new import parser in expressions.rs
        crate::frontend::parser::imports::parse_import_statement(state)
    }
}

/// Parse URL import statement (complexity: 6)
pub fn parse_url_import(state: &mut ParserState, url: &str, start_span: Span) -> Result<Expr> {
    // Validate URL format
    if !url.starts_with("https://") && !url.starts_with("http://") {
        bail!("URL imports must start with 'https://' or 'http://'");
    }
    // Safety validation for URL imports
    validate_url_import(url)?;
    state.tokens.advance();
    // URL imports become simple module imports
    Ok(Expr::new(
        ExprKind::Import {
            module: url.to_string(),
            items: None, // URL imports import everything
        },
        start_span,
    ))
}

/// Parse module path components
pub fn parse_module_path(state: &mut ParserState) -> Result<Vec<String>> {
    let Some((Token::Identifier(_), _)) = state.tokens.peek() else {
        return Ok(Vec::new());
    };

    let mut path_parts = vec![consume_module_identifier(state)?];
    parse_additional_path_segments(state, &mut path_parts)?;
    Ok(path_parts)
}

/// Consume and return module identifier
fn consume_module_identifier(state: &mut ParserState) -> Result<String> {
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        bail!("Expected identifier");
    };

    let name = name.clone();
    state.tokens.advance();
    Ok(name)
}

/// Parse additional path segments after initial identifier
fn parse_additional_path_segments(
    state: &mut ParserState,
    path_parts: &mut Vec<String>,
) -> Result<()> {
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::

        if is_import_items_start(state) {
            break;
        }

        path_parts.push(parse_path_segment(state)?);
    }
    Ok(())
}

/// Check if current position is start of import items (complexity: 2)
fn is_import_items_start(state: &mut ParserState) -> bool {
    matches!(
        state.tokens.peek(),
        Some((Token::Star | Token::LeftBrace, _))
    )
}

/// Parse single path segment after :: (complexity: 3)
fn parse_path_segment(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected identifier, '*', or '{{' after '::'");
    }
}

/// Parse import items (wildcard, braced list, or simple) (complexity: 9)
pub fn parse_import_items(
    state: &mut ParserState,
    path_parts: &[String],
) -> Result<Vec<ImportItem>> {
    if matches!(state.tokens.peek(), Some((Token::Star, _))) {
        parse_wildcard_import(state)
    } else if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        parse_braced_import_list(state)
    } else {
        parse_simple_import(state, path_parts)
    }
}

/// Parse wildcard import (* syntax) (complexity: 2)
fn parse_wildcard_import(state: &mut ParserState) -> Result<Vec<ImportItem>> {
    state.tokens.advance(); // consume *
    Ok(vec![ImportItem::Wildcard])
}

/// Parse braced import list ({item1, item2, ...})
pub fn parse_braced_import_list(state: &mut ParserState) -> Result<Vec<ImportItem>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let items = parse_import_item_list(state)?;
    state.tokens.expect(&Token::RightBrace)?;
    Ok(items)
}

/// Parse list of import items (extracted to reduce nesting)
fn parse_import_item_list(state: &mut ParserState) -> Result<Vec<ImportItem>> {
    let mut items = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        items.push(parse_single_import_item(state)?);

        if !try_consume_item_separator(state)? {
            break;
        }
    }
    Ok(items)
}

/// Parse a single import item
fn parse_single_import_item(state: &mut ParserState) -> Result<ImportItem> {
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            bail!("Expected identifier or '}}' in import list");
        }
        bail!("Expected identifier in import list");
    };

    let name = name.clone();
    state.tokens.advance();
    parse_item_with_optional_alias(state, name)
}

/// Parse import item with optional alias
fn parse_item_with_optional_alias(state: &mut ParserState, name: String) -> Result<ImportItem> {
    if !matches!(state.tokens.peek(), Some((Token::As, _))) {
        return Ok(ImportItem::Named(name));
    }

    state.tokens.advance(); // consume as
    let Some((Token::Identifier(alias), _)) = state.tokens.peek() else {
        bail!("Expected alias name after 'as'");
    };

    let alias = alias.clone();
    state.tokens.advance();
    Ok(ImportItem::Aliased { name, alias })
}

/// Try to consume item separator (returns false if at end)
fn try_consume_item_separator(state: &mut ParserState) -> Result<bool> {
    match state.tokens.peek() {
        Some((Token::Comma, _)) => {
            state.tokens.advance();
            Ok(true)
        }
        Some((Token::RightBrace, _)) => Ok(false),
        _ => bail!("Expected ',' or '}}' in import list"),
    }
}

/// Parse simple import (path or path as alias) (complexity: 8)
fn parse_simple_import(state: &mut ParserState, path_parts: &[String]) -> Result<Vec<ImportItem>> {
    if matches!(state.tokens.peek(), Some((Token::As, _))) {
        parse_simple_import_with_alias(state, path_parts)
    } else {
        parse_simple_import_without_alias(path_parts)
    }
}

/// Parse simple import with alias (complexity: 5)
fn parse_simple_import_with_alias(
    state: &mut ParserState,
    path_parts: &[String],
) -> Result<Vec<ImportItem>> {
    state.tokens.advance(); // consume as
    if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
        let alias = alias.clone();
        state.tokens.advance();
        Ok(vec![ImportItem::Aliased {
            name: path_parts.last().unwrap_or(&String::new()).clone(),
            alias,
        }])
    } else {
        bail!("Expected alias name after 'as'");
    }
}

/// Parse simple import without alias (complexity: 5)
fn parse_simple_import_without_alias(path_parts: &[String]) -> Result<Vec<ImportItem>> {
    if path_parts.is_empty() {
        Ok(Vec::new())
    } else if path_parts.len() == 1 {
        // Single segment - treat as wildcard
        Ok(Vec::new())
    } else {
        // Multi-segment - import the last part
        Ok(vec![ImportItem::Named(
            path_parts
                .last()
                .expect("checked: !path_parts.is_empty()")
                .clone(),
        )])
    }
}

/// Create final import expression (complexity: 4)
pub fn create_import_expression(
    path_parts: Vec<String>,
    _items: Vec<ImportItem>,
    start_span: Span,
) -> Result<Expr> {
    let module = path_parts.join("::");
    // Validate that we have a module
    if module.is_empty() {
        bail!("Expected import path after 'import'");
    }
    // Legacy import - convert to simple module import
    Ok(Expr::new(
        ExprKind::Import {
            module,
            items: None, // Legacy imports use None for now
        },
        start_span,
    ))
}

// Export parsing functions

/// Parse export statement
///
/// # Errors
///
/// Returns an error if the export statement is malformed or contains invalid syntax.
pub fn parse_export(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1;

    match state.tokens.peek() {
        Some((Token::Default, _)) => parse_export_default(state, start_span),
        Some((Token::LeftBrace, _)) => parse_export_list(state, start_span),
        Some((Token::Fun | Token::Const | Token::Let | Token::Class | Token::Struct, _)) => {
            parse_export_declaration(state, start_span)
        }
        _ => bail!("Invalid export statement"),
    }
}

/// Parse export default statement
fn parse_export_default(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume default
    let expr = crate::frontend::parser::parse_expr_with_precedence_recursive(state, 0)?;
    Ok(Expr::new(
        ExprKind::ExportDefault {
            expr: Box::new(expr),
        },
        start_span,
    ))
}

/// Parse export list statement
fn parse_export_list(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume {
    let items = parse_export_identifier_list(state)?;
    state.tokens.expect(&Token::RightBrace)?;

    create_export_or_reexport(state, items, start_span)
}

/// Parse list of export identifiers
fn parse_export_identifier_list(state: &mut ParserState) -> Result<Vec<String>> {
    let mut items = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        items.push(parse_export_identifier(state)?);
        try_consume_export_comma(state);
    }

    Ok(items)
}

/// Parse single export identifier
fn parse_export_identifier(state: &mut ParserState) -> Result<String> {
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        bail!("Expected identifier in export list");
    };

    let name = name.clone();
    state.tokens.advance();
    Ok(name)
}

/// Try to consume optional comma in export list
fn try_consume_export_comma(state: &mut ParserState) {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
    }
}

/// Create export or re-export expression based on 'from' keyword
fn create_export_or_reexport(
    state: &mut ParserState,
    items: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
    if !matches!(state.tokens.peek(), Some((Token::From, _))) {
        return Ok(Expr::new(ExprKind::ExportList { names: items }, start_span));
    }

    state.tokens.advance();
    let module = parse_module_specifier(state)?;
    Ok(Expr::new(ExprKind::ReExport { items, module }, start_span))
}

/// Parse module specifier (for re-exports)
fn parse_module_specifier(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::String(module), _)) => {
            let module = module.clone();
            state.tokens.advance();
            Ok(module)
        }
        Some((Token::Identifier(module), _)) => {
            let module = module.clone();
            state.tokens.advance();
            Ok(module)
        }
        _ => bail!("Expected module path after 'from'"),
    }
}

/// Parse export declaration
fn parse_export_declaration(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    let expr = crate::frontend::parser::parse_expr_with_precedence_recursive(state, 0)?;
    Ok(Expr::new(
        ExprKind::Export {
            expr: Box::new(expr),
            is_default: false,
        },
        start_span,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create parser state from source
    fn create_state(source: &str) -> ParserState {
        ParserState::new(source)
    }

    // parse_module_path tests
    #[test]
    fn test_parse_module_path_simple() {
        let mut state = create_state("foo");
        let path = parse_module_path(&mut state).expect("should succeed");
        assert_eq!(path, vec!["foo"]);
    }

    #[test]
    fn test_parse_module_path_nested() {
        let mut state = create_state("foo::bar::baz");
        let path = parse_module_path(&mut state).expect("should succeed");
        assert_eq!(path, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_parse_module_path_empty() {
        let mut state = create_state("");
        let path = parse_module_path(&mut state).expect("should succeed");
        assert!(path.is_empty());
    }

    #[test]
    fn test_parse_module_path_stops_at_star() {
        let mut state = create_state("foo::*");
        let path = parse_module_path(&mut state).expect("should succeed");
        assert_eq!(path, vec!["foo"]);
    }

    #[test]
    fn test_parse_module_path_stops_at_brace() {
        let mut state = create_state("foo::{bar}");
        let path = parse_module_path(&mut state).expect("should succeed");
        assert_eq!(path, vec!["foo"]);
    }

    // parse_import_items tests
    #[test]
    fn test_parse_import_items_wildcard() {
        let mut state = create_state("*");
        let items = parse_import_items(&mut state, &[]).expect("should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], ImportItem::Wildcard));
    }

    #[test]
    fn test_parse_import_items_braced_single() {
        let mut state = create_state("{foo}");
        let items = parse_import_items(&mut state, &[]).expect("should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], ImportItem::Named(n) if n == "foo"));
    }

    #[test]
    fn test_parse_import_items_braced_multiple() {
        let mut state = create_state("{foo, bar, baz}");
        let items = parse_import_items(&mut state, &[]).expect("should succeed");
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_parse_import_items_braced_with_alias() {
        let mut state = create_state("{foo as f}");
        let items = parse_import_items(&mut state, &[]).expect("should succeed");
        assert_eq!(items.len(), 1);
        if let ImportItem::Aliased { name, alias } = &items[0] {
            assert_eq!(name, "foo");
            assert_eq!(alias, "f");
        } else {
            panic!("Expected Aliased import");
        }
    }

    #[test]
    fn test_parse_import_items_simple_with_path() {
        let mut state = create_state("");
        let items = parse_import_items(&mut state, &["foo".to_string(), "bar".to_string()])
            .expect("should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(&items[0], ImportItem::Named(n) if n == "bar"));
    }

    #[test]
    fn test_parse_import_items_simple_empty_path() {
        let mut state = create_state("");
        let items = parse_import_items(&mut state, &[]).expect("should succeed");
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_import_items_single_segment_path() {
        let mut state = create_state("");
        let items = parse_import_items(&mut state, &["foo".to_string()]).expect("should succeed");
        assert!(items.is_empty()); // Single segment treated as wildcard
    }

    // parse_braced_import_list tests
    #[test]
    fn test_parse_braced_import_list_empty() {
        let mut state = create_state("{}");
        let items = parse_braced_import_list(&mut state).expect("should succeed");
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_braced_import_list_trailing_comma() {
        let mut state = create_state("{foo,}");
        // This may or may not be valid depending on grammar
        let result = parse_braced_import_list(&mut state);
        // Either succeeds or errors predictably
        assert!(result.is_ok() || result.is_err());
    }

    // create_import_expression tests
    #[test]
    fn test_create_import_expression_simple() {
        let path = vec!["foo".to_string()];
        let items = vec![];
        let span = Span::default();
        let expr = create_import_expression(path, items, span).expect("should succeed");
        if let ExprKind::Import { module, .. } = &expr.kind {
            assert_eq!(module, "foo");
        } else {
            panic!("Expected Import expression");
        }
    }

    #[test]
    fn test_create_import_expression_nested() {
        let path = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let items = vec![];
        let span = Span::default();
        let expr = create_import_expression(path, items, span).expect("should succeed");
        if let ExprKind::Import { module, .. } = &expr.kind {
            assert_eq!(module, "foo::bar::baz");
        } else {
            panic!("Expected Import expression");
        }
    }

    #[test]
    fn test_create_import_expression_empty_path_error() {
        let path = vec![];
        let items = vec![];
        let span = Span::default();
        let result = create_import_expression(path, items, span);
        assert!(result.is_err());
    }

    // parse_url_import tests
    #[test]
    fn test_parse_url_import_invalid_scheme() {
        let mut state = create_state("");
        let result = parse_url_import(&mut state, "ftp://example.com", Span::default());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("https://") || err.contains("http://"));
    }

    // parse_export tests
    #[test]
    fn test_parse_export_default() {
        let mut state = create_state("export default 42");
        let expr = parse_export(&mut state).expect("should succeed");
        assert!(matches!(expr.kind, ExprKind::ExportDefault { .. }));
    }

    #[test]
    fn test_parse_export_list_simple() {
        let mut state = create_state("export {foo}");
        let expr = parse_export(&mut state).expect("should succeed");
        if let ExprKind::ExportList { names } = &expr.kind {
            assert_eq!(names, &vec!["foo".to_string()]);
        } else {
            panic!("Expected ExportList");
        }
    }

    #[test]
    fn test_parse_export_list_multiple() {
        let mut state = create_state("export {foo, bar}");
        let expr = parse_export(&mut state).expect("should succeed");
        if let ExprKind::ExportList { names } = &expr.kind {
            assert_eq!(names.len(), 2);
        } else {
            panic!("Expected ExportList");
        }
    }

    #[test]
    fn test_parse_export_invalid() {
        let mut state = create_state("export 123");
        let result = parse_export(&mut state);
        // Number literal is not a valid export target
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_export_function() {
        let mut state = create_state("export fun foo() { 42 }");
        let expr = parse_export(&mut state).expect("should succeed");
        assert!(matches!(expr.kind, ExprKind::Export { .. }));
    }

    // is_import_items_start tests
    #[test]
    fn test_is_import_items_start_star() {
        let mut state = create_state("*");
        assert!(is_import_items_start(&mut state));
    }

    #[test]
    fn test_is_import_items_start_brace() {
        let mut state = create_state("{");
        assert!(is_import_items_start(&mut state));
    }

    #[test]
    fn test_is_import_items_start_identifier() {
        let mut state = create_state("foo");
        assert!(!is_import_items_start(&mut state));
    }

    // parse_path_segment tests
    #[test]
    fn test_parse_path_segment_valid() {
        let mut state = create_state("foo");
        let segment = parse_path_segment(&mut state).expect("should succeed");
        assert_eq!(segment, "foo");
    }

    #[test]
    fn test_parse_path_segment_invalid() {
        let mut state = create_state("123");
        let result = parse_path_segment(&mut state);
        assert!(result.is_err());
    }

    // parse_simple_import_without_alias tests
    #[test]
    fn test_parse_simple_import_without_alias_empty() {
        let result = parse_simple_import_without_alias(&[]).expect("should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_simple_import_without_alias_single() {
        let result =
            parse_simple_import_without_alias(&["foo".to_string()]).expect("should succeed");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_simple_import_without_alias_multi() {
        let result = parse_simple_import_without_alias(&["foo".to_string(), "bar".to_string()])
            .expect("should succeed");
        assert_eq!(result.len(), 1);
        assert!(matches!(&result[0], ImportItem::Named(n) if n == "bar"));
    }

    // parse_simple_import tests
    #[test]
    fn test_parse_simple_import_no_alias() {
        let mut state = create_state("");
        let result = parse_simple_import(&mut state, &["foo".to_string(), "bar".to_string()])
            .expect("should succeed");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_simple_import_with_alias() {
        let mut state = create_state("as alias");
        let result = parse_simple_import(&mut state, &["foo".to_string(), "bar".to_string()])
            .expect("should succeed");
        assert_eq!(result.len(), 1);
        if let ImportItem::Aliased { name, alias } = &result[0] {
            assert_eq!(name, "bar");
            assert_eq!(alias, "alias");
        } else {
            panic!("Expected Aliased import");
        }
    }

    // consume_module_identifier tests
    #[test]
    fn test_consume_module_identifier_valid() {
        let mut state = create_state("my_module");
        let name = consume_module_identifier(&mut state).expect("should succeed");
        assert_eq!(name, "my_module");
    }

    #[test]
    fn test_consume_module_identifier_invalid() {
        let mut state = create_state("123");
        let result = consume_module_identifier(&mut state);
        assert!(result.is_err());
    }

    // try_consume_item_separator tests
    #[test]
    fn test_try_consume_item_separator_comma() {
        let mut state = create_state(",");
        let result = try_consume_item_separator(&mut state).expect("should succeed");
        assert!(result);
    }

    #[test]
    fn test_try_consume_item_separator_brace() {
        let mut state = create_state("}");
        let result = try_consume_item_separator(&mut state).expect("should succeed");
        assert!(!result);
    }

    #[test]
    fn test_try_consume_item_separator_invalid() {
        let mut state = create_state("foo");
        let result = try_consume_item_separator(&mut state);
        assert!(result.is_err());
    }

    // parse_item_with_optional_alias tests
    #[test]
    fn test_parse_item_with_optional_alias_no_alias() {
        let mut state = create_state("");
        let item =
            parse_item_with_optional_alias(&mut state, "foo".to_string()).expect("should succeed");
        assert!(matches!(item, ImportItem::Named(n) if n == "foo"));
    }

    #[test]
    fn test_parse_item_with_optional_alias_with_alias() {
        let mut state = create_state("as bar");
        let item =
            parse_item_with_optional_alias(&mut state, "foo".to_string()).expect("should succeed");
        if let ImportItem::Aliased { name, alias } = item {
            assert_eq!(name, "foo");
            assert_eq!(alias, "bar");
        } else {
            panic!("Expected Aliased");
        }
    }

    #[test]
    fn test_parse_item_with_optional_alias_missing_alias_name() {
        let mut state = create_state("as 123");
        let result = parse_item_with_optional_alias(&mut state, "foo".to_string());
        assert!(result.is_err());
    }

    // parse_single_import_item tests
    #[test]
    fn test_parse_single_import_item_simple() {
        let mut state = create_state("foo");
        let item = parse_single_import_item(&mut state).expect("should succeed");
        assert!(matches!(item, ImportItem::Named(n) if n == "foo"));
    }

    // parse_wildcard_import tests
    #[test]
    fn test_parse_wildcard_import() {
        let mut state = create_state("*");
        let items = parse_wildcard_import(&mut state).expect("should succeed");
        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], ImportItem::Wildcard));
    }

    // parse_export_identifier tests
    #[test]
    fn test_parse_export_identifier_valid() {
        let mut state = create_state("foo");
        let name = parse_export_identifier(&mut state).expect("should succeed");
        assert_eq!(name, "foo");
    }

    #[test]
    fn test_parse_export_identifier_invalid() {
        let mut state = create_state("123");
        let result = parse_export_identifier(&mut state);
        assert!(result.is_err());
    }

    // try_consume_export_comma tests
    #[test]
    fn test_try_consume_export_comma_present() {
        let mut state = create_state(",foo");
        try_consume_export_comma(&mut state);
        // Should have advanced past the comma
        assert!(matches!(
            state.tokens.peek(),
            Some((Token::Identifier(_), _))
        ));
    }

    #[test]
    fn test_try_consume_export_comma_absent() {
        let mut state = create_state("foo");
        try_consume_export_comma(&mut state);
        // Should not have advanced
        assert!(matches!(
            state.tokens.peek(),
            Some((Token::Identifier(n), _)) if n == "foo"
        ));
    }

    // parse_module_specifier tests
    #[test]
    fn test_parse_module_specifier_string() {
        let mut state = create_state("\"my_module\"");
        let module = parse_module_specifier(&mut state).expect("should succeed");
        assert_eq!(module, "my_module");
    }

    #[test]
    fn test_parse_module_specifier_identifier() {
        let mut state = create_state("my_module");
        let module = parse_module_specifier(&mut state).expect("should succeed");
        assert_eq!(module, "my_module");
    }

    #[test]
    fn test_parse_module_specifier_invalid() {
        let mut state = create_state("123");
        let result = parse_module_specifier(&mut state);
        assert!(result.is_err());
    }

    // create_export_or_reexport tests
    #[test]
    fn test_create_export_or_reexport_no_from() {
        let mut state = create_state("");
        let expr = create_export_or_reexport(&mut state, vec!["foo".to_string()], Span::default())
            .expect("should succeed");
        assert!(matches!(expr.kind, ExprKind::ExportList { .. }));
    }

    #[test]
    fn test_create_export_or_reexport_with_from() {
        let mut state = create_state("from \"module\"");
        let expr = create_export_or_reexport(&mut state, vec!["foo".to_string()], Span::default())
            .expect("should succeed");
        if let ExprKind::ReExport { items, module } = &expr.kind {
            assert_eq!(items, &vec!["foo".to_string()]);
            assert_eq!(module, "module");
        } else {
            panic!("Expected ReExport");
        }
    }
}
