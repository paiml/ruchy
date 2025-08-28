//! Parsing utilities and helper functions

use super::{ParserState, *};
use crate::frontend::ast::ImportItem;

/// Validate URL imports for security
fn validate_url_import(url: &str) -> Result<()> {
    // Security checks for URL imports
    
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
        // Check for mut keyword
        let is_mutable = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
            state.tokens.advance(); // consume mut
            true
        } else {
            false
        };

        let pattern = match state.tokens.peek() {
            Some((Token::Ampersand, _)) => {
                // Handle &self or &mut self patterns
                state.tokens.advance(); // consume &

                if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
                    state.tokens.advance(); // consume mut
                    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
                        if n == "self" {
                            state.tokens.advance();
                            Pattern::Identifier("&mut self".to_string())
                        } else {
                            bail!("Expected 'self' after '&mut'");
                        }
                    } else {
                        bail!("Expected 'self' after '&mut'");
                    }
                } else if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
                    if n == "self" {
                        state.tokens.advance();
                        Pattern::Identifier("&self".to_string())
                    } else {
                        bail!("Expected 'self' after '&'");
                    }
                } else {
                    bail!("Expected 'self' after '&'");
                }
            }
            Some((Token::Identifier(name), _)) => {
                // Only accept simple identifier patterns for parameters
                let name = name.clone();
                state.tokens.advance();
                Pattern::Identifier(name)
            }
            _ => bail!("Function parameters must be simple identifiers (destructuring patterns not supported)"),
        };

        // Type annotation is optional for gradual typing
        let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            parse_type(state)?
        } else {
            // Default to 'Any' type for untyped parameters
            Type {
                kind: TypeKind::Named("Any".to_string()),
                span: Span { start: 0, end: 0 },
            }
        };

        // Parse optional default value (only on simple identifiers)
        let default_value = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
            state.tokens.advance(); // consume =
            Some(Box::new(super::parse_expr_recursive(state)?))
        } else {
            None
        };

        params.push(Param {
            pattern,
            ty,
            span: Span { start: 0, end: 0 },
            is_mutable,
            default_value,
        });

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightParen)?;
    Ok(params)
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

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_type(state: &mut ParserState) -> Result<Type> {
    let span = Span { start: 0, end: 0 }; // Simplified for now

    match state.tokens.peek() {
        Some((Token::Fn, _)) => {
            // Function type with fn keyword: fn(T1, T2) -> T3
            state.tokens.advance(); // consume fn
            state.tokens.expect(&Token::LeftParen)?;
            
            let mut param_types = Vec::new();
            if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                param_types.push(parse_type(state)?);
                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                    param_types.push(parse_type(state)?);
                }
            }
            
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
        Some((Token::LeftBracket, _)) => {
            state.tokens.advance(); // consume [
            let inner = parse_type(state)?;
            state.tokens.expect(&Token::RightBracket)?;
            // List type: [T]
            Ok(Type {
                kind: TypeKind::List(Box::new(inner)),
                span,
            })
        }
        Some((Token::LeftParen, _)) => {
            state.tokens.advance(); // consume (
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                // Unit type: ()
                state.tokens.advance();
                Ok(Type {
                    kind: TypeKind::Named("()".to_string()),
                    span,
                })
            } else {
                // Could be tuple type (T1, T2) or function type (T1, T2) -> T3
                let mut param_types = Vec::new();
                param_types.push(parse_type(state)?);

                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                    param_types.push(parse_type(state)?);
                }

                state.tokens.expect(&Token::RightParen)?;
                
                // Check if this is a function type or tuple type
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
        Some((Token::Identifier(name), _)) => {
            let mut name = name.clone();
            state.tokens.advance();

            // Check for qualified type names: std::string::String
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

            // Check for generic types: Vec<T>, Result<T, E>
            if matches!(state.tokens.peek(), Some((Token::Less, _))) {
                state.tokens.advance(); // consume <

                let mut type_params = Vec::new();
                type_params.push(parse_type(state)?);

                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                    type_params.push(parse_type(state)?);
                }

                state.tokens.expect(&Token::Greater)?;

                // Use Generic TypeKind for parameterized types
                Ok(Type {
                    kind: TypeKind::Generic {
                        base: name,
                        params: type_params,
                    },
                    span,
                })
            } else {
                Ok(Type {
                    kind: TypeKind::Named(name),
                    span,
                })
            }
        }
        _ => bail!("Expected type"),
    }
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
pub fn parse_import(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume import/use

    let mut path_parts = Vec::new();

    // Check for URL import (e.g., import "https://example.com/module.ruchy")
    if let Some((Token::String(url), _)) = state.tokens.peek() {
        // URL import - validate it starts with https://
        if !url.starts_with("https://") && !url.starts_with("http://") {
            bail!("URL imports must start with 'https://' or 'http://'");
        }
        
        // Security validation for URL imports
        validate_url_import(url)?;
        
        let url = url.clone();
        state.tokens.advance();
        
        // URL imports are always single module imports (no wildcard or specific items)
        let span = start_span; // simplified for now
        return Ok(Expr::new(
            ExprKind::Import {
                path: url,
                items: vec![ImportItem::Named("*".to_string())], // Import all from URL
            },
            span,
        ));
    }
    
    // Parse regular module path (e.g., std::io::prelude)
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        path_parts.push(name.clone());
        state.tokens.advance();

        while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
            // Check for ::
            state.tokens.advance(); // consume ::

            // Check for wildcard or brace after ::
            if matches!(
                state.tokens.peek(),
                Some((Token::Star | Token::LeftBrace, _))
            ) {
                // This is the start of import items, break out of path parsing
                break;
            }

            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                path_parts.push(name.clone());
                state.tokens.advance();
            } else {
                bail!("Expected identifier, '*', or '{{' after '::'");
            }
        }
    }

    // Check for specific imports like ::{Read, Write} or ::*
    // Note: We may have already consumed the :: in the loop above
    let items = if matches!(state.tokens.peek(), Some((Token::Star, _))) {
        // Wildcard import: import path::*
        state.tokens.advance(); // consume *
        vec![ImportItem::Wildcard]
    } else if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        // Specific imports: import path::{item1, item2, ...}
        state.tokens.expect(&Token::LeftBrace)?; // consume {

        let mut items = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let name = name.clone();
                state.tokens.advance();

                // Check for alias: item as alias
                if matches!(state.tokens.peek(), Some((Token::As, _))) {
                    state.tokens.advance(); // consume as
                    if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
                        let alias = alias.clone();
                        state.tokens.advance();
                        items.push(ImportItem::Aliased { name, alias });
                    } else {
                        bail!("Expected alias name after 'as'");
                    }
                } else {
                    items.push(ImportItem::Named(name));
                }

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                    // After comma, continue to parse next item
                    // Don't break here - continue the loop
                } else if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                    // If not comma and not right brace, error
                    bail!("Expected ',' or '}}' in import list");
                }
            } else if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                bail!("Expected identifier or '}}' in import list");
            }
        }

        state.tokens.expect(&Token::RightBrace)?;
        items
    } else {
        // Simple import: import path or import path as alias
        if matches!(state.tokens.peek(), Some((Token::As, _))) {
            // import path as alias
            state.tokens.advance(); // consume as
            if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
                let alias = alias.clone();
                state.tokens.advance();
                vec![ImportItem::Aliased {
                    name: path_parts.last().unwrap_or(&String::new()).clone(),
                    alias,
                }]
            } else {
                bail!("Expected alias name after 'as'");
            }
        } else {
            // Simple import without alias
            if path_parts.is_empty() {
                Vec::new()
            } else if path_parts.len() == 1 {
                // Single segment like "use math;" - treat as wildcard (use math::*)
                Vec::new() // Empty items = wildcard import in transpiler
            } else {
                // Multi-segment like "use std::collections::HashMap;" - import the last part
                vec![ImportItem::Named(
                    path_parts
                        .last()
                        .expect("checked: !path_parts.is_empty()")
                        .clone(),
                )]
            }
        }
    };

    let path = path_parts.join("::");

    // Validate that we have either a path or items (or both)
    if path.is_empty() && items.is_empty() {
        bail!("Expected import path or items after 'import'");
    }

    let span = start_span; // simplified for now

    Ok(Expr::new(ExprKind::Import { path, items }, span))
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
                // Escaped brace: {{
                chars.next(); // consume second '{'
                current_text.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                // Escaped brace: }}
                chars.next(); // consume second '}'
                current_text.push('}');
            }
            '{' => {
                // Start of interpolation
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }

                // Collect expression until closing '}' with proper string literal handling
                let mut expr_text = String::new();
                let mut brace_count = 1;
                let mut in_string = false;
                let mut in_char = false;
                let mut escaped = false;

                for expr_ch in chars.by_ref() {
                    match expr_ch {
                        '"' if !in_char && !escaped => {
                            in_string = !in_string;
                            expr_text.push(expr_ch);
                        }
                        '\'' if !in_string && !escaped => {
                            in_char = !in_char;
                            expr_text.push(expr_ch);
                        }
                        '{' if !in_string && !in_char => {
                            brace_count += 1;
                            expr_text.push(expr_ch);
                        }
                        '}' if !in_string && !in_char => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                break;
                            }
                            expr_text.push(expr_ch);
                        }
                        '\\' if (in_string || in_char) && !escaped => {
                            escaped = true;
                            expr_text.push(expr_ch);
                        }
                        _ => {
                            escaped = false;
                            expr_text.push(expr_ch);
                        }
                    }

                    // Reset escape flag for non-backslash characters
                    if expr_ch != '\\' {
                        escaped = false;
                    }
                }

                // Check if there's a format specifier (e.g., "score:.2" -> "score" and ":.2")
                let (expr_part, format_spec) = if let Some(colon_pos) = expr_text.find(':') {
                    // Check if the colon is not inside a string or character literal
                    // Simple heuristic: if there are no quotes before the colon, it's likely a format spec
                    let before_colon = &expr_text[..colon_pos];
                    if !before_colon.contains('"') && !before_colon.contains('\'') {
                        (&expr_text[..colon_pos], Some(&expr_text[colon_pos..]))
                    } else {
                        (expr_text.as_str(), None)
                    }
                } else {
                    (expr_text.as_str(), None)
                };
                
                // Parse the expression part (without format specifier)
                let mut expr_parser = super::core::Parser::new(expr_part);
                match expr_parser.parse() {
                    Ok(expr) => {
                        // Store the expression with or without format specifier
                        if let Some(spec) = format_spec {
                            parts.push(StringPart::ExprWithFormat {
                                expr: Box::new(expr),
                                format_spec: spec.to_string(),
                            });
                        } else {
                            parts.push(StringPart::Expr(Box::new(expr)));
                        }
                    }
                    Err(_) => {
                        // Fallback to text if parsing fails
                        parts.push(StringPart::Text(format!("{{{expr_text}}}")));
                    }
                }
            }
            _ => current_text.push(ch),
        }
    }

    // Add remaining text
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }

    parts
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
