//! Data structure parsing module (struct, enum, trait, impl)
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, StmtKind, Type, Pattern, Span};
use crate::frontend::ast::{Field, Variant, ImplItem};
use anyhow::bail;

/// Parse struct definition
pub fn parse_struct(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    // Parse visibility
    let is_pub = if state.peek_matches(&Token::Pub) {
        state.advance();
        true
    } else {
        false
    };
    
    state.expect_token(Token::Struct)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected struct name");
    };
    
    // Parse generic parameters
    let generics = parse_generics(state)?;
    
    // Parse struct body or tuple struct
    let (fields, is_tuple) = if state.peek_matches(&Token::LeftParen) {
        // Tuple struct
        (parse_tuple_struct_fields(state)?, true)
    } else if state.peek_matches(&Token::LeftBrace) {
        // Regular struct
        (parse_struct_fields(state)?, false)
    } else {
        // Unit struct
        (vec![], false)
    };
    
    create_struct_expr(name, generics, fields, is_tuple, is_pub, span_start, state)
}

/// Parse generic parameters
fn parse_generics(state: &mut ParserState) -> Result<Option<Vec<String>>> {
    if state.peek_matches(&Token::Lt) {
        state.advance();
        let mut params = Vec::new();
        
        loop {
            let (Token::Identifier(param), _) = state.next_token()? else {
                bail!("Expected generic parameter");
            };
            params.push(param);
            
            if !state.peek_matches(&Token::Comma) {
                break;
            }
            state.advance();
        }
        
        state.expect_token(Token::Gt)?;
        Ok(Some(params))
    } else {
        Ok(None)
    }
}

/// Parse tuple struct fields
fn parse_tuple_struct_fields(state: &mut ParserState) -> Result<Vec<Field>> {
    state.expect_token(Token::LeftParen)?;
    let mut fields = Vec::new();
    let mut index = 0;
    
    while !state.peek_matches(&Token::RightParen) {
        let ty = state.parse_type()?;
        fields.push(Field {
            name: index.to_string(),
            ty,
            is_pub: false,
            default: None,
        });
        index += 1;
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightParen)?;
    Ok(fields)
}

/// Parse struct fields
fn parse_struct_fields(state: &mut ParserState) -> Result<Vec<Field>> {
    state.expect_token(Token::LeftBrace)?;
    let mut fields = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        fields.push(parse_single_field(state)?);
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightBrace)?;
    Ok(fields)
}

/// Parse single field
fn parse_single_field(state: &mut ParserState) -> Result<Field> {
    let is_pub = if state.peek_matches(&Token::Pub) {
        state.advance();
        true
    } else {
        false
    };
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected field name");
    };
    
    state.expect_token(Token::Colon)?;
    let ty = state.parse_type()?;
    
    // Parse default value
    let default = if state.peek_matches(&Token::Equals) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Field {
        name,
        ty,
        is_pub,
        default,
    })
}

/// Create struct expression
fn create_struct_expr(
    name: String,
    generics: Option<Vec<String>>,
    fields: Vec<Field>,
    is_tuple: bool,
    is_pub: bool,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Struct {
            name,
            generics,
            fields,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: if is_pub { vec!["pub".to_string()] } else { vec![] },
    })
}

/// Parse enum definition
pub fn parse_enum(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    let is_pub = if state.peek_matches(&Token::Pub) {
        state.advance();
        true
    } else {
        false
    };
    
    state.expect_token(Token::Enum)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected enum name");
    };
    
    let generics = parse_generics(state)?;
    
    state.expect_token(Token::LeftBrace)?;
    let variants = parse_enum_variants(state)?;
    state.expect_token(Token::RightBrace)?;
    
    create_enum_expr(name, generics, variants, is_pub, span_start, state)
}

/// Parse enum variants
fn parse_enum_variants(state: &mut ParserState) -> Result<Vec<Variant>> {
    let mut variants = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        variants.push(parse_single_variant(state)?);
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    Ok(variants)
}

/// Parse single variant
fn parse_single_variant(state: &mut ParserState) -> Result<Variant> {
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected variant name");
    };
    
    // Check variant type
    if state.peek_matches(&Token::LeftParen) {
        // Tuple variant
        state.advance();
        let mut fields = Vec::new();
        
        while !state.peek_matches(&Token::RightParen) {
            fields.push(state.parse_type()?);
            
            if !state.peek_matches(&Token::Comma) {
                break;
            }
            state.advance();
        }
        
        state.expect_token(Token::RightParen)?;
        Ok(Variant::Tuple(name, fields))
    } else if state.peek_matches(&Token::LeftBrace) {
        // Struct variant
        let fields = parse_struct_fields(state)?;
        Ok(Variant::Struct(name, fields))
    } else {
        // Unit variant
        Ok(Variant::Unit(name))
    }
}

/// Create enum expression
fn create_enum_expr(
    name: String,
    generics: Option<Vec<String>>,
    variants: Vec<Variant>,
    is_pub: bool,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Enum {
            name,
            generics,
            variants,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: if is_pub { vec!["pub".to_string()] } else { vec![] },
    })
}

/// Parse trait definition
pub fn parse_trait(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    let is_pub = if state.peek_matches(&Token::Pub) {
        state.advance();
        true
    } else {
        false
    };
    
    state.expect_token(Token::Trait)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected trait name");
    };
    
    let generics = parse_generics(state)?;
    
    // Parse trait bounds
    let bounds = if state.peek_matches(&Token::Colon) {
        state.advance();
        Some(parse_trait_bounds(state)?)
    } else {
        None
    };
    
    state.expect_token(Token::LeftBrace)?;
    let items = parse_trait_items(state)?;
    state.expect_token(Token::RightBrace)?;
    
    create_trait_expr(name, generics, bounds, items, is_pub, span_start, state)
}

/// Parse trait bounds
fn parse_trait_bounds(state: &mut ParserState) -> Result<Vec<String>> {
    let mut bounds = Vec::new();
    
    loop {
        let (Token::Identifier(bound), _) = state.next_token()? else {
            bail!("Expected trait bound");
        };
        bounds.push(bound);
        
        if !state.peek_matches(&Token::Plus) {
            break;
        }
        state.advance();
    }
    
    Ok(bounds)
}

/// Parse trait items (simplified - just collect expressions)
fn parse_trait_items(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut items = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        items.push(state.parse_expression()?);
        
        // Skip optional semicolon
        if state.peek_matches(&Token::Semicolon) {
            state.advance();
        }
    }
    
    Ok(items)
}

/// Create trait expression
fn create_trait_expr(
    name: String,
    generics: Option<Vec<String>>,
    bounds: Option<Vec<String>>,
    items: Vec<Expr>,
    is_pub: bool,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Trait {
            name,
            generics,
            bounds,
            items,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: if is_pub { vec!["pub".to_string()] } else { vec![] },
    })
}

/// Parse impl block
pub fn parse_impl(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Impl)?;
    
    let generics = parse_generics(state)?;
    
    // Parse trait (if trait impl)
    let (trait_name, for_type) = parse_impl_target(state)?;
    
    state.expect_token(Token::LeftBrace)?;
    let items = parse_impl_items(state)?;
    state.expect_token(Token::RightBrace)?;
    
    create_impl_expr(trait_name, for_type, generics, items, span_start, state)
}

/// Parse impl target (trait for type, or just type)
fn parse_impl_target(state: &mut ParserState) -> Result<(Option<Type>, Type)> {
    let first_type = state.parse_type()?;
    
    if state.peek_matches(&Token::For) {
        // Trait implementation
        state.advance();
        let for_type = state.parse_type()?;
        Ok((Some(first_type), for_type))
    } else {
        // Inherent implementation
        Ok((None, first_type))
    }
}

/// Parse impl items
fn parse_impl_items(state: &mut ParserState) -> Result<Vec<ImplItem>> {
    let mut items = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        let item = state.parse_expression()?;
        items.push(ImplItem::Method(item));
        
        // Skip optional semicolon
        if state.peek_matches(&Token::Semicolon) {
            state.advance();
        }
    }
    
    Ok(items)
}

/// Create impl expression
fn create_impl_expr(
    trait_name: Option<Type>,
    for_type: Type,
    generics: Option<Vec<String>>,
    items: Vec<ImplItem>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Impl {
            trait_name,
            for_type,
            generics,
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