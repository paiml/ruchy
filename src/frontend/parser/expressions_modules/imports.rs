//! Import/Export parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, StmtKind, Span};
use anyhow::bail;

/// Parse import statement
pub fn parse_import(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Import)?;
    
    // Parse import path
    let path = parse_import_path(state)?;
    
    // Parse import items or wildcard
    let items = if state.peek_matches(&Token::Star) {
        state.advance();
        vec!["*".to_string()]
    } else if state.peek_matches(&Token::LeftBrace) {
        parse_import_items(state)?
    } else {
        // Single item import
        vec![path.last().unwrap_or(&String::new()).clone()]
    };
    
    // Parse optional alias
    let alias = parse_import_alias(state)?;
    
    create_import_expr(path, items, alias, span_start, state)
}

/// Parse from-import statement
pub fn parse_from_import(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::From)?;
    
    let module_path = parse_import_path(state)?;
    state.expect_token(Token::Import)?;
    
    let items = if state.peek_matches(&Token::Star) {
        state.advance();
        vec!["*".to_string()]
    } else if state.peek_matches(&Token::LeftBrace) {
        parse_import_items(state)?
    } else {
        parse_single_import_item(state)?
    };
    
    create_from_import_expr(module_path, items, span_start, state)
}

/// Parse import path (module.submodule.item)
fn parse_import_path(state: &mut ParserState) -> Result<Vec<String>> {
    let mut path = Vec::new();
    
    // Handle relative imports (.module or ..module)
    while state.peek_matches(&Token::Dot) {
        state.advance();
        path.push(".".to_string());
    }
    
    // Parse module path
    loop {
        let (Token::Identifier(part), _) = state.next_token()? else {
            bail!("Expected module name in import path");
        };
        path.push(part);
        
        if !state.peek_matches(&Token::Dot) {
            break;
        }
        state.advance();
    }
    
    Ok(path)
}

/// Parse import items in braces
fn parse_import_items(state: &mut ParserState) -> Result<Vec<String>> {
    state.expect_token(Token::LeftBrace)?;
    let mut items = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        let (Token::Identifier(item), _) = state.next_token()? else {
            bail!("Expected import item name");
        };
        
        // Check for alias
        if state.peek_matches(&Token::As) {
            state.advance();
            let (Token::Identifier(alias), _) = state.next_token()? else {
                bail!("Expected alias name after 'as'");
            };
            items.push(format!("{} as {}", item, alias));
        } else {
            items.push(item);
        }
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightBrace)?;
    Ok(items)
}

/// Parse single import item
fn parse_single_import_item(state: &mut ParserState) -> Result<Vec<String>> {
    let (Token::Identifier(item), _) = state.next_token()? else {
        bail!("Expected import item");
    };
    
    if state.peek_matches(&Token::As) {
        state.advance();
        let (Token::Identifier(alias), _) = state.next_token()? else {
            bail!("Expected alias name");
        };
        Ok(vec![format!("{} as {}", item, alias)])
    } else {
        Ok(vec![item])
    }
}

/// Parse import alias
fn parse_import_alias(state: &mut ParserState) -> Result<Option<String>> {
    if state.peek_matches(&Token::As) {
        state.advance();
        let (Token::Identifier(alias), _) = state.next_token()? else {
            bail!("Expected alias name after 'as'");
        };
        Ok(Some(alias))
    } else {
        Ok(None)
    }
}

/// Create import expression
fn create_import_expr(
    path: Vec<String>,
    items: Vec<String>,
    alias: Option<String>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Import {
            module_path: path,
            items,
            alias,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Create from-import expression
fn create_from_import_expr(
    module_path: Vec<String>,
    items: Vec<String>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::FromImport {
            module_path,
            items,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse export statement
pub fn parse_export(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Export)?;
    
    // Parse what to export
    if state.peek_matches(&Token::LeftBrace) {
        // Export specific items
        let items = parse_export_items(state)?;
        create_export_expr(items, None, span_start, state)
    } else if state.peek_matches(&Token::Star) {
        // Export all
        state.advance();
        
        if state.peek_matches(&Token::From) {
            state.advance();
            let module = parse_import_path(state)?;
            create_reexport_expr(vec!["*".to_string()], module, span_start, state)
        } else {
            create_export_expr(vec!["*".to_string()], None, span_start, state)
        }
    } else {
        // Export declaration (export fn foo() {})
        let decl = state.parse_expression()?;
        create_export_decl_expr(decl, span_start, state)
    }
}

/// Parse export items in braces
fn parse_export_items(state: &mut ParserState) -> Result<Vec<String>> {
    state.expect_token(Token::LeftBrace)?;
    let mut items = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        let (Token::Identifier(item), _) = state.next_token()? else {
            bail!("Expected export item name");
        };
        
        // Check for alias
        if state.peek_matches(&Token::As) {
            state.advance();
            let (Token::Identifier(alias), _) = state.next_token()? else {
                bail!("Expected alias name after 'as'");
            };
            items.push(format!("{} as {}", item, alias));
        } else {
            items.push(item);
        }
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightBrace)?;
    
    // Check for re-export
    if state.peek_matches(&Token::From) {
        state.advance();
        let module = parse_import_path(state)?;
        return create_reexport_expr(items, module, span_start, state);
    }
    
    Ok(items)
}

/// Create export expression
fn create_export_expr(
    items: Vec<String>,
    from_module: Option<Vec<String>>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Export {
            items,
            from_module,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Create re-export expression
fn create_reexport_expr(
    items: Vec<String>,
    from_module: Vec<String>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Export {
            items,
            from_module: Some(from_module),
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Create export declaration expression
fn create_export_decl_expr(
    decl: Expr,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    // Add export attribute to the declaration
    let mut exported_decl = decl;
    exported_decl.attributes.push("export".to_string());
    Ok(exported_decl)
}

/// Parse use statement (Rust-style imports)
pub fn parse_use(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Use)?;
    
    let path = parse_use_path(state)?;
    
    // Parse optional alias
    let alias = if state.peek_matches(&Token::As) {
        state.advance();
        let (Token::Identifier(alias), _) = state.next_token()? else {
            bail!("Expected alias name after 'as'");
        };
        Some(alias)
    } else {
        None
    };
    
    state.expect_token(Token::Semicolon)?;
    
    create_use_expr(path, alias, span_start, state)
}

/// Parse use path (can include wildcards and nested imports)
fn parse_use_path(state: &mut ParserState) -> Result<Vec<String>> {
    let mut path = Vec::new();
    
    // Handle path prefix if present
    if let Some(prefix) = parse_path_prefix(state)? {
        path.push(prefix);
    }
    
    // Parse path segments
    parse_path_segments(state, &mut path)?;
    
    Ok(path)
}

/// Parse path prefix (crate::, super::, self::) (complexity: 4)
fn parse_path_prefix(state: &mut ParserState) -> Result<Option<String>> {
    let prefix = if state.peek_matches(&Token::Crate) {
        Some("crate")
    } else if state.peek_matches(&Token::Super) {
        Some("super")
    } else if state.peek_matches(&Token::SelfType) {
        Some("self")
    } else {
        None
    };
    
    if let Some(prefix_str) = prefix {
        state.advance();
        state.expect_token(Token::ColonColon)?;
        Ok(Some(prefix_str.to_string()))
    } else {
        Ok(None)
    }
}

/// Parse path segments (complexity: 5)
fn parse_path_segments(state: &mut ParserState, path: &mut Vec<String>) -> Result<()> {
    loop {
        let segment = parse_single_segment(state)?;
        path.push(segment.clone());
        
        // Check if this is a terminal segment
        if is_terminal_segment(&segment) || !state.peek_matches(&Token::ColonColon) {
            break;
        }
        
        state.advance(); // consume ::
    }
    Ok(())
}

/// Parse a single path segment (complexity: 5)
fn parse_single_segment(state: &mut ParserState) -> Result<String> {
    if state.peek_matches(&Token::Star) {
        state.advance();
        Ok("*".to_string())
    } else if state.peek_matches(&Token::LeftBrace) {
        let nested = parse_nested_use(state)?;
        Ok(format!("{{{}}}", nested.join(", ")))
    } else {
        parse_identifier_segment(state)
    }
}

/// Parse identifier segment (complexity: 2)
fn parse_identifier_segment(state: &mut ParserState) -> Result<String> {
    let (Token::Identifier(segment), _) = state.next_token()? else {
        bail!("Expected path segment in use statement");
    };
    Ok(segment)
}

/// Check if segment is terminal (complexity: 2)
fn is_terminal_segment(segment: &str) -> bool {
    segment == "*" || segment.starts_with('{')
}

/// Parse nested use items
fn parse_nested_use(state: &mut ParserState) -> Result<Vec<String>> {
    state.expect_token(Token::LeftBrace)?;
    let mut items = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        if state.peek_matches(&Token::Star) {
            state.advance();
            items.push("*".to_string());
        } else {
            let (Token::Identifier(item), _) = state.next_token()? else {
                bail!("Expected item in nested use");
            };
            items.push(item);
        }
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightBrace)?;
    Ok(items)
}

/// Create use expression
fn create_use_expr(
    path: Vec<String>,
    alias: Option<String>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Use {
            path,
            alias,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}