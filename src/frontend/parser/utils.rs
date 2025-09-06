//! Parsing utilities and helper functions

use super::{ParserState, *};
use crate::frontend::ast::ImportItem;

/// Validate URL imports for safe operation
fn validate_url_import(url: &str) -> Result<()> {
    // Safety checks for URL imports
    
    // 1. Must be HTTPS in production (allow HTTP for local dev)
    if !url.starts_with("https://") && !url.starts_with("http://localhost") 
        && !url.starts_with("http://127.0.0.1") {
        bail!("URL imports must use HTTPS (except for localhost)");
    }
    
    // 2. Must end with .ruchy or .rchy extension
    if !url.ends_with(".ruchy") && !url.ends_with(".rchy") {
        bail!("URL imports must reference .ruchy or .rchy files");
    }
    
    // 3. Basic URL validation - no path traversal
    if url.contains("..") || url.contains("/.") {
        bail!("URL imports cannot contain path traversal sequences");
    }
    
    // 4. Disallow certain suspicious patterns
    if url.contains("javascript:") || url.contains("data:") || url.contains("file:") {
        bail!("Invalid URL scheme for import");
    }
    
    Ok(())
}

/// Parse a pattern for destructuring
///
/// Supports:
/// - Simple identifiers: `name`
/// - Tuple patterns: `(a, b, c)`
/// - List patterns: `[head, tail]`
/// - Struct patterns: `User { name, age }`
/// - Wildcard patterns: `_`
///
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_params(state: &mut ParserState) -> Result<Vec<Param>> {
    state.tokens.expect(&Token::LeftParen)?;

    let mut params = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        let param = parse_single_param(state)?;
        params.push(param);
        
        if !should_continue_param_list(state)? {
            break;
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(params)
}

/// Parse a single parameter (complexity: 7)
fn parse_single_param(state: &mut ParserState) -> Result<Param> {
    let is_mutable = check_and_consume_mut(state);
    let pattern = parse_param_pattern(state)?;
    let ty = parse_optional_type_annotation(state)?;
    let default_value = parse_optional_default_value(state)?;
    
    Ok(Param {
        pattern,
        ty,
        span: Span { start: 0, end: 0 },
        is_mutable,
        default_value,
    })
}

/// Check for and consume mut keyword (complexity: 2)
fn check_and_consume_mut(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse parameter pattern (complexity: 6)
fn parse_param_pattern(state: &mut ParserState) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Ampersand, _)) => parse_reference_pattern(state),
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        _ => bail!("Function parameters must be simple identifiers (destructuring patterns not supported)"),
    }
}

/// Parse reference patterns (&self, &mut self) (complexity: 8)
fn parse_reference_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance(); // consume &

    let is_mut_ref = matches!(state.tokens.peek(), Some((Token::Mut, _)));
    if is_mut_ref {
        state.tokens.advance(); // consume mut
    }

    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) if n == "self" => {
            state.tokens.advance();
            if is_mut_ref {
                Ok(Pattern::Identifier("&mut self".to_string()))
            } else {
                Ok(Pattern::Identifier("&self".to_string()))
            }
        }
        _ => {
            let expected = if is_mut_ref { "'self' after '&mut'" } else { "'self' after '&'" };
            bail!("Expected {}", expected)
        }
    }
}

/// Parse optional type annotation (complexity: 4)
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Type> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume :
        parse_type(state)
    } else {
        // Default to 'Any' type for untyped parameters
        Ok(Type {
            kind: TypeKind::Named("Any".to_string()),
            span: Span { start: 0, end: 0 },
        })
    }
}

/// Parse optional default value (complexity: 3)
fn parse_optional_default_value(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance(); // consume =
        Ok(Some(Box::new(super::parse_expr_recursive(state)?)))
    } else {
        Ok(None)
    }
}

/// Check if we should continue parsing parameters (complexity: 3)
fn should_continue_param_list(state: &mut ParserState) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        Ok(true)
    } else {
        Ok(false)
    }
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_type_parameters(state: &mut ParserState) -> Result<Vec<String>> {
    state.tokens.expect(&Token::Less)?;

    let mut type_params = Vec::new();

    // Parse first type parameter
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        type_params.push(name.clone());
        state.tokens.advance();
    }

    // Parse additional type parameters
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            type_params.push(name.clone());
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::Greater)?;
    Ok(type_params)
}

/// Parse type expressions with complexity ≤10
/// # Errors
/// Returns an error if the operation fails
pub fn parse_type(state: &mut ParserState) -> Result<Type> {
    let span = Span { start: 0, end: 0 }; // Simplified for now

    match state.tokens.peek() {
        Some((Token::Ampersand, _)) => parse_reference_type(state, span),
        Some((Token::Fn, _)) => parse_fn_type(state, span),
        Some((Token::LeftBracket, _)) => parse_list_type(state, span),
        Some((Token::LeftParen, _)) => parse_paren_type(state, span),
        Some((Token::Identifier(_), _)) => parse_named_type(state, span),
        _ => bail!("Expected type"),
    }
}

// Helper: Parse reference type &T or &mut T (complexity: 4)
fn parse_reference_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume &
    
    let is_mut = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume mut
        true
    } else {
        false
    };
    
    let inner_type = parse_type(state)?;
    
    Ok(Type {
        kind: TypeKind::Reference {
            is_mut,
            inner: Box::new(inner_type),
        },
        span,
    })
}

// Helper: Parse function type fn(T1, T2) -> T3 (complexity: 5)
fn parse_fn_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume fn
    state.tokens.expect(&Token::LeftParen)?;
    
    let param_types = parse_type_list(state)?;
    state.tokens.expect(&Token::RightParen)?;
    state.tokens.expect(&Token::Arrow)?;
    let ret_type = parse_type(state)?;
    
    Ok(Type {
        kind: TypeKind::Function {
            params: param_types,
            ret: Box::new(ret_type),
        },
        span,
    })
}

// Helper: Parse list type `[T]` (complexity: 3)
fn parse_list_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume [
    let inner = parse_type(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    
    Ok(Type {
        kind: TypeKind::List(Box::new(inner)),
        span,
    })
}

// Helper: Parse parenthesized type (T1, T2) or (T1, T2) -> T3 (complexity: 6)
fn parse_paren_type(state: &mut ParserState, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume (
    
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        // Unit type: ()
        state.tokens.advance();
        Ok(Type {
            kind: TypeKind::Named("()".to_string()),
            span,
        })
    } else {
        let param_types = parse_type_list(state)?;
        state.tokens.expect(&Token::RightParen)?;
        
        if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
            // Function type: (T1, T2) -> T3
            state.tokens.advance(); // consume ->
            let ret_type = parse_type(state)?;

            Ok(Type {
                kind: TypeKind::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                },
                span,
            })
        } else {
            // Tuple type: (T1, T2)
            Ok(Type {
                kind: TypeKind::Tuple(param_types),
                span,
            })
        }
    }
}

// Helper: Parse named type with optional generics (complexity: 4)
fn parse_named_type(state: &mut ParserState, span: Span) -> Result<Type> {
    let name = parse_qualified_name(state)?;
    
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_generic_type(state, name, span)
    } else {
        Ok(Type {
            kind: TypeKind::Named(name),
            span,
        })
    }
}

// Helper: Parse qualified name like std::collections::HashMap (complexity: 6)
fn parse_qualified_name(state: &mut ParserState) -> Result<String> {
    let mut name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            name
        }
        _ => bail!("Expected identifier"),
    };

    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        
        let next_name = match state.tokens.peek() {
            Some((Token::Identifier(next), _)) => next.clone(),
            // Handle special tokens that can be type names
            Some((Token::Result, _)) => "Result".to_string(),
            Some((Token::Option, _)) => "Option".to_string(),
            Some((Token::Ok, _)) => "Ok".to_string(),
            Some((Token::Err, _)) => "Err".to_string(),
            Some((Token::Some, _)) => "Some".to_string(),
            Some((Token::None | Token::Null, _)) => "None".to_string(),
            _ => bail!("Expected identifier after :: in type name"),
        };
        
        name.push_str("::");
        name.push_str(&next_name);
        state.tokens.advance();
    }

    Ok(name)
}

// Helper: Parse generic type Vec<T, U> (complexity: 4)
fn parse_generic_type(state: &mut ParserState, base: String, span: Span) -> Result<Type> {
    state.tokens.advance(); // consume <
    let type_params = parse_type_list(state)?;
    state.tokens.expect(&Token::Greater)?;

    Ok(Type {
        kind: TypeKind::Generic {
            base,
            params: type_params,
        },
        span,
    })
}

// Helper: Parse comma-separated type list (complexity: 3)
fn parse_type_list(state: &mut ParserState) -> Result<Vec<Type>> {
    let mut types = Vec::new();
    
    if !matches!(state.tokens.peek(), Some((Token::RightParen | Token::Greater, _))) {
        types.push(parse_type(state)?);
        
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
            types.push(parse_type(state)?);
        }
    }
    
    Ok(types)
}

/// Parse import statements in various forms
///
/// Supports:
/// - Simple imports: `import std::collections::HashMap`
/// - Multiple imports: `import std::io::{Read, Write}`
/// - Aliased imports: `import std::collections::{HashMap as Map}`
/// - Wildcard imports: `import std::collections::*`
///
/// # Examples
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::{ExprKind, ImportItem};
///
/// let mut parser = Parser::new("import std::collections::HashMap");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Import { path, items } => {
///         assert_eq!(path, "std::collections::HashMap");
///         assert_eq!(items.len(), 1);
///         assert!(matches!(items[0], ImportItem::Named(ref name) if name == "HashMap"));
///     }
///     _ => panic!("Expected Import expression"),
/// }
/// ```
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::{ExprKind, ImportItem};
///
/// // Multiple imports with alias
/// let mut parser = Parser::new("import std::collections::{HashMap as Map, Vec}");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Import { path, items } => {
///         assert_eq!(path, "std::collections");
///         assert_eq!(items.len(), 2);
///         assert!(matches!(&items[0], ImportItem::Aliased { name, alias }
///                          if name == "HashMap" && alias == "Map"));
///         assert!(matches!(&items[1], ImportItem::Named(name) if name == "Vec"));
///     }
///     _ => panic!("Expected Import expression"),
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No identifier follows the import keyword
/// - Invalid syntax in import specification
/// - Unexpected tokens in import list
///
/// Parse import statement (complexity: 7)
/// Orchestrates URL and regular import parsing
pub fn parse_import(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1;
    
    // Check for URL import first
    if let Some((Token::String(url), _)) = state.tokens.peek() {
        let url = url.clone();
        return parse_url_import(state, &url, start_span);
    }
    
    // Parse regular module import
    let path_parts = parse_module_path(state)?;
    let items = parse_import_items(state, &path_parts)?;
    
    create_import_expression(path_parts, items, start_span)
}

/// Parse URL import statement (complexity: 6)
fn parse_url_import(state: &mut ParserState, url: &str, start_span: Span) -> Result<Expr> {
    // Validate URL format
    if !url.starts_with("https://") && !url.starts_with("http://") {
        bail!("URL imports must start with 'https://' or 'http://'");
    }
    
    // Safety validation for URL imports
    validate_url_import(url)?;
    
    state.tokens.advance();
    
    Ok(Expr::new(
        ExprKind::Import {
            path: url.to_string(),
            items: vec![ImportItem::Named("*".to_string())],
        },
        start_span,
    ))
}

/// Parse module path components (complexity: 8)
fn parse_module_path(state: &mut ParserState) -> Result<Vec<String>> {
    let mut path_parts = Vec::new();
    
    // Get first identifier
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        path_parts.push(name.clone());
        state.tokens.advance();
        
        // Parse additional path segments
        while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
            state.tokens.advance(); // consume ::
            
            // Check if this is the start of import items
            if is_import_items_start(state) {
                break;
            }
            
            path_parts.push(parse_path_segment(state)?);
        }
    }
    
    Ok(path_parts)
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
fn parse_import_items(state: &mut ParserState, path_parts: &[String]) -> Result<Vec<ImportItem>> {
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

/// Parse braced import list ({item1, item2, ...}) (complexity: 10)
fn parse_braced_import_list(state: &mut ParserState) -> Result<Vec<ImportItem>> {
    state.tokens.expect(&Token::LeftBrace)?;
    
    let mut items = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            
            let item = parse_import_item_with_alias(state, name)?;
            items.push(item);
            
            if !handle_item_separator(state)? {
                break;
            }
        } else {
            validate_braced_list_token(state)?;
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    Ok(items)
}

/// Parse import item with optional alias (complexity: 6)
fn parse_import_item_with_alias(state: &mut ParserState, name: String) -> Result<ImportItem> {
    if matches!(state.tokens.peek(), Some((Token::As, _))) {
        state.tokens.advance(); // consume as
        if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
            let alias = alias.clone();
            state.tokens.advance();
            Ok(ImportItem::Aliased { name, alias })
        } else {
            bail!("Expected alias name after 'as'");
        }
    } else {
        Ok(ImportItem::Named(name))
    }
}

/// Handle item separator in braced list (complexity: 4)
fn handle_item_separator(state: &mut ParserState) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        Ok(true) // Continue parsing
    } else if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        Ok(false) // End of list
    } else {
        bail!("Expected ',' or '}}' in import list");
    }
}

/// Validate token in braced import list (complexity: 3)
fn validate_braced_list_token(state: &mut ParserState) -> Result<()> {
    if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        bail!("Expected identifier or '}}' in import list");
    }
    Ok(())
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
fn parse_simple_import_with_alias(state: &mut ParserState, path_parts: &[String]) -> Result<Vec<ImportItem>> {
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
fn create_import_expression(path_parts: Vec<String>, items: Vec<ImportItem>, start_span: Span) -> Result<Expr> {
    let path = path_parts.join("::");
    
    // Validate that we have either a path or items
    if path.is_empty() && items.is_empty() {
        bail!("Expected import path or items after 'import'");
    }
    
    Ok(Expr::new(ExprKind::Import { path, items }, start_span))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_attributes(state: &mut ParserState) -> Result<Vec<Attribute>> {
    let mut attributes = Vec::new();

    while matches!(state.tokens.peek(), Some((Token::Hash, _))) {
        state.tokens.advance(); // consume #

        if !matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
            bail!("Expected '[' after '#'");
        }
        state.tokens.advance(); // consume [

        let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected attribute name");
        };

        let mut args = Vec::new();
        if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            state.tokens.advance(); // consume (

            while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                if let Some((Token::Identifier(arg), _)) = state.tokens.peek() {
                    args.push(arg.clone());
                    state.tokens.advance();

                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            state.tokens.advance(); // consume )
        }

        let end_span = state.tokens.advance().expect("Expected ']' token").1; // consume ]

        attributes.push(Attribute {
            name,
            args,
            span: end_span,
        });
    }

    Ok(attributes)
}

/// Parse string interpolation from a string containing {expr} patterns
pub fn parse_string_interpolation(_state: &mut ParserState, s: &str) -> Vec<StringPart> {
    let mut parts = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current_text = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                handle_escaped_brace(&mut chars, &mut current_text, '{');
            }
            '}' if chars.peek() == Some(&'}') => {
                handle_escaped_brace(&mut chars, &mut current_text, '}');
            }
            '{' => {
                handle_interpolation(&mut chars, &mut parts, &mut current_text);
            }
            _ => current_text.push(ch),
        }
    }

    finalize_text_part(&mut parts, current_text);
    parts
}

// Helper: Handle escaped braces (complexity: 2)
fn handle_escaped_brace<T: Iterator<Item = char>>(
    chars: &mut T,
    current_text: &mut String,
    brace_char: char,
) {
    chars.next(); // consume second brace
    current_text.push(brace_char);
}

// Helper: Handle interpolation expressions (complexity: 4)
fn handle_interpolation<T: Iterator<Item = char>>(
    chars: &mut T,
    parts: &mut Vec<StringPart>,
    current_text: &mut String,
) {
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text.clone()));
        current_text.clear();
    }

    let expr_text = extract_expression_text(chars);
    let string_part = parse_interpolated_expr(&expr_text);
    parts.push(string_part);
}

// Helper: Extract expression text from braces (complexity: 8)
fn extract_expression_text<T: Iterator<Item = char>>(chars: &mut T) -> String {
    let mut expr_text = String::new();
    let mut context = ExprContext::default();

    for expr_ch in chars {
        if process_character(expr_ch, &mut context, &mut expr_text) {
            break;
        }
    }

    expr_text
}

/// Process a single character in expression extraction (complexity: 8)
fn process_character(ch: char, context: &mut ExprContext, expr_text: &mut String) -> bool {
    match ch {
        '"' if should_process_string_quote(context) => {
            handle_string_delimiter(context);
            expr_text.push(ch);
        }
        '\'' if should_process_char_quote(context) => {
            handle_char_delimiter(context);
            expr_text.push(ch);
        }
        '{' if should_process_brace(context) => {
            handle_open_brace(context);
            expr_text.push(ch);
        }
        '}' if should_process_brace(context) => {
            handle_close_brace(context);
            if should_terminate(context) {
                return true; // Signal to break the loop
            }
            expr_text.push(ch);
        }
        '\\' if should_escape(context) => {
            handle_escape(context);
            expr_text.push(ch);
        }
        _ => {
            handle_regular_char(context, ch);
            expr_text.push(ch);
        }
    }
    
    // Reset escape flag for non-backslash characters
    reset_escape_flag(context, ch);
    false // Continue processing
}

/// Check if string quote should be processed (complexity: 1)
fn should_process_string_quote(context: &ExprContext) -> bool {
    !context.in_char && !context.escaped
}

/// Check if char quote should be processed (complexity: 1)
fn should_process_char_quote(context: &ExprContext) -> bool {
    !context.in_string && !context.escaped
}

/// Check if brace should be processed (complexity: 1)
fn should_process_brace(context: &ExprContext) -> bool {
    !context.in_string && !context.in_char
}

/// Check if escape should be handled (complexity: 1)
fn should_escape(context: &ExprContext) -> bool {
    (context.in_string || context.in_char) && !context.escaped
}

/// Toggle string delimiter state (complexity: 1)
fn handle_string_delimiter(context: &mut ExprContext) {
    context.in_string = !context.in_string;
}

/// Toggle char delimiter state (complexity: 1)
fn handle_char_delimiter(context: &mut ExprContext) {
    context.in_char = !context.in_char;
}

/// Increment brace count (complexity: 1)
fn handle_open_brace(context: &mut ExprContext) {
    context.brace_count += 1;
}

/// Decrement brace count (complexity: 1)
fn handle_close_brace(context: &mut ExprContext) {
    context.brace_count -= 1;
}

/// Set escape flag (complexity: 1)
fn handle_escape(context: &mut ExprContext) {
    context.escaped = true;
}

/// Handle regular character (complexity: 1)
fn handle_regular_char(context: &mut ExprContext, _ch: char) {
    context.escaped = false;
}

/// Reset escape flag if needed (complexity: 2)
fn reset_escape_flag(context: &mut ExprContext, ch: char) {
    if ch != '\\' {
        context.escaped = false;
    }
}

/// Check if we should terminate extraction (complexity: 1)
fn should_terminate(context: &ExprContext) -> bool {
    context.brace_count == 0
}

// Helper: Parse interpolated expression with format specifier (complexity: 4)
fn parse_interpolated_expr(expr_text: &str) -> StringPart {
    let (expr_part, format_spec) = split_format_specifier(expr_text);
    
    let mut expr_parser = super::core::Parser::new(expr_part);
    match expr_parser.parse() {
        Ok(expr) => {
            if let Some(spec) = format_spec {
                StringPart::ExprWithFormat {
                    expr: Box::new(expr),
                    format_spec: spec.to_string(),
                }
            } else {
                StringPart::Expr(Box::new(expr))
            }
        }
        Err(_) => {
            // Fallback to text if parsing fails
            StringPart::Text(format!("{{{expr_text}}}"))
        }
    }
}

// Helper: Split format specifier from expression (complexity: 3)
fn split_format_specifier(expr_text: &str) -> (&str, Option<&str>) {
    if let Some(colon_pos) = expr_text.find(':') {
        let before_colon = &expr_text[..colon_pos];
        if !before_colon.contains('"') && !before_colon.contains('\'') {
            (before_colon, Some(&expr_text[colon_pos..]))
        } else {
            (expr_text, None)
        }
    } else {
        (expr_text, None)
    }
}

// Helper: Finalize remaining text (complexity: 2)
fn finalize_text_part(parts: &mut Vec<StringPart>, current_text: String) {
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }
}

// Helper struct to track expression parsing context (complexity: 0)
#[derive(Default)]
struct ExprContext {
    brace_count: i32,
    in_string: bool,
    in_char: bool,
    escaped: bool,
}

impl ExprContext {
    fn default() -> Self {
        Self {
            brace_count: 1,
            in_string: false,
            in_char: false,
            escaped: false,
        }
    }
}

/// Parse module declarations
///
/// Supports:
/// - Empty modules: `module MyModule {}`
/// - Single expression modules: `module Math { sqrt(x) }`
/// - Multi-expression modules: `module Utils { fn helper() {...}; const PI = 3.14 }`
///
/// # Examples
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::ExprKind;
///
/// // Empty module
/// let mut parser = Parser::new("module Empty {}");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Module { name, .. } => {
///         assert_eq!(name, "Empty");
///     }
///     _ => panic!("Expected Module expression"),
/// }
/// ```
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::{ExprKind, Literal};
///
/// // Module with content
/// let mut parser = Parser::new("module Math { 42 }");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Module { name, body } => {
///         assert_eq!(name, "Math");
///         // Verify body contains literal 42
///         match &body.kind {
///             ExprKind::Literal(Literal::Integer(n)) => assert_eq!(*n, 42),
///             _ => panic!("Expected integer literal in module body"),
///         }
///     }
///     _ => panic!("Expected Module expression"),
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No identifier follows the module keyword
/// - Missing opening or closing braces
/// - Invalid syntax in module body
pub fn parse_module(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume module

    // Parse module name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected module name after 'module'");
    };

    // Expect opening brace
    state.tokens.expect(&Token::LeftBrace)?;

    // Parse module body (can be a block or single expression)
    let body = if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Empty module
        Box::new(Expr::new(
            ExprKind::Literal(Literal::Unit),
            Span { start: 0, end: 0 },
        ))
    } else {
        // Parse expressions until we hit the closing brace
        let mut exprs = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            exprs.push(super::parse_expr_recursive(state)?);
            // Optional semicolon or comma separator
            if matches!(
                state.tokens.peek(),
                Some((Token::Semicolon | Token::Comma, _))
            ) {
                state.tokens.advance();
            }
        }

        if exprs.len() == 1 {
            Box::new(exprs.into_iter().next().expect("checked: exprs.len() == 1"))
        } else {
            Box::new(Expr::new(ExprKind::Block(exprs), Span { start: 0, end: 0 }))
        }
    };

    // Expect closing brace
    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(ExprKind::Module { name, body }, start_span))
}

/// Parse export statements
///
/// Supports:
/// - Single exports: `export myFunction`
/// - Multiple exports: `export { func1, func2, func3 }`
///
/// # Examples
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::ExprKind;
///
/// // Single export
/// let mut parser = Parser::new("export myFunction");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Export { items } => {
///         assert_eq!(items.len(), 1);
///         assert_eq!(items[0], "myFunction");
///     }
///     _ => panic!("Expected Export expression"),
/// }
/// ```
///
/// ```
/// use ruchy::frontend::parser::Parser;
/// use ruchy::frontend::ast::ExprKind;
///
/// // Multiple exports
/// let mut parser = Parser::new("export { add, subtract, multiply }");
/// let expr = parser.parse().unwrap();
///
/// match &expr.kind {
///     ExprKind::Export { items } => {
///         assert_eq!(items.len(), 3);
///         assert!(items.contains(&"add".to_string()));
///         assert!(items.contains(&"subtract".to_string()));
///         assert!(items.contains(&"multiply".to_string()));
///     }
///     _ => panic!("Expected Export expression"),
/// }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - No identifier or brace follows the export keyword
/// - Invalid syntax in export list
/// - Missing closing brace in export block
pub fn parse_export(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume export

    let mut items = Vec::new();

    // Parse export list
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        // Export block: export { item1, item2, ... }
        state.tokens.advance(); // consume {

        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                items.push(name.clone());
                state.tokens.advance();

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                } else {
                    break;
                }
            } else {
                bail!("Expected identifier in export list");
            }
        }

        state.tokens.expect(&Token::RightBrace)?;
    } else if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        // Single export: export item
        items.push(name.clone());
        state.tokens.advance();
    } else {
        bail!("Expected export list or identifier after 'export'");
    }

    Ok(Expr::new(ExprKind::Export { items }, start_span))
}
