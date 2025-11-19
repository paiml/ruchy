//! Import and export parsing functions
//!
//! This module contains all import/export parsing logic extracted from utils.rs
//! to reduce file complexity and improve maintainability.

use super::super::{bail, Expr, ExprKind, ImportItem, ParserState, Result, Span, Token};
use super::url_validation::validate_url_import;

/// Parse import statement (legacy import parser)
///
/// NOTE: This is the legacy import parser. New imports are parsed in expressions.rs
#[allow(dead_code)]
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
