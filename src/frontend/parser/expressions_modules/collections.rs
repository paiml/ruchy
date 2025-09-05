//! Collection literal parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Span};
use anyhow::bail;

/// Parse list literal or list comprehension
pub fn parse_list(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::LeftBracket)?;
    
    // Check for empty list
    if state.peek_matches(&Token::RightBracket) {
        state.advance();
        return Ok(create_list_expr(vec![], span_start, state));
    }
    
    // Parse first element to check for comprehension
    let first = state.parse_expression()?;
    
    // Check for list comprehension
    if state.peek_matches(&Token::For) {
        parse_list_comprehension(state, first, span_start)
    } else {
        parse_list_elements(state, first, span_start)
    }
}

/// Parse list elements
fn parse_list_elements(
    state: &mut ParserState,
    first: Expr,
    span_start: Span,
) -> Result<Expr> {
    let mut elements = vec![first];
    
    while state.peek_matches(&Token::Comma) {
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightBracket) {
            break;
        }
        
        elements.push(state.parse_expression()?);
    }
    
    state.expect_token(Token::RightBracket)?;
    Ok(create_list_expr(elements, span_start, state))
}

/// Create list expression
fn create_list_expr(elements: Vec<Expr>, span_start: Span, state: &ParserState) -> Expr {
    Expr {
        kind: ExprKind::List { elements },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    }
}

/// Parse list comprehension
fn parse_list_comprehension(
    state: &mut ParserState,
    expr: Expr,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::For)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::In)?;
    let iterable = Box::new(state.parse_expression()?);
    
    // Parse optional filter
    let filter = if state.peek_matches(&Token::If) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    state.expect_token(Token::RightBracket)?;
    
    Ok(Expr {
        kind: ExprKind::ListComp {
            expr: Box::new(expr),
            pattern,
            iterable,
            filter,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse tuple literal
pub fn parse_tuple(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::LeftParen)?;
    
    // Check for unit tuple ()
    if state.peek_matches(&Token::RightParen) {
        state.advance();
        return Ok(create_tuple_expr(vec![], span_start, state));
    }
    
    let mut elements = vec![state.parse_expression()?];
    
    // Parse remaining elements
    while state.peek_matches(&Token::Comma) {
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightParen) {
            break;
        }
        
        elements.push(state.parse_expression()?);
    }
    
    state.expect_token(Token::RightParen)?;
    
    // Single element without comma is just a grouped expression
    if elements.len() == 1 && !state.last_was_comma() {
        Ok(elements.into_iter().next().unwrap())
    } else {
        Ok(create_tuple_expr(elements, span_start, state))
    }
}

/// Create tuple expression
fn create_tuple_expr(elements: Vec<Expr>, span_start: Span, state: &ParserState) -> Expr {
    Expr {
        kind: ExprKind::Tuple { elements },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    }
}

/// Parse set literal
pub fn parse_set(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::LeftBrace)?;
    
    // Empty {} is a dict, not a set
    if state.peek_matches(&Token::RightBrace) {
        state.advance();
        return Ok(create_dict_expr(vec![], vec![], span_start, state));
    }
    
    // Parse first element to determine if it's a set or dict
    let first = state.parse_expression()?;
    
    // Check for dict (key: value)
    if state.peek_matches(&Token::Colon) {
        parse_dict_elements(state, first, span_start)
    } else if state.peek_matches(&Token::For) {
        // Set comprehension
        parse_set_comprehension(state, first, span_start)
    } else {
        // Regular set
        parse_set_elements(state, first, span_start)
    }
}

/// Parse set elements
fn parse_set_elements(
    state: &mut ParserState,
    first: Expr,
    span_start: Span,
) -> Result<Expr> {
    let mut elements = vec![first];
    
    while state.peek_matches(&Token::Comma) {
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightBrace) {
            break;
        }
        
        elements.push(state.parse_expression()?);
    }
    
    state.expect_token(Token::RightBrace)?;
    
    Ok(Expr {
        kind: ExprKind::Set { elements },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse set comprehension
fn parse_set_comprehension(
    state: &mut ParserState,
    expr: Expr,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::For)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::In)?;
    let iterable = Box::new(state.parse_expression()?);
    
    // Parse optional filter
    let filter = if state.peek_matches(&Token::If) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    state.expect_token(Token::RightBrace)?;
    
    Ok(Expr {
        kind: ExprKind::SetComp {
            expr: Box::new(expr),
            pattern,
            iterable,
            filter,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse dictionary literal
pub fn parse_dict(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::LeftBrace)?;
    
    // Empty dict
    if state.peek_matches(&Token::RightBrace) {
        state.advance();
        return Ok(create_dict_expr(vec![], vec![], span_start, state));
    }
    
    let first_key = state.parse_expression()?;
    
    // Check for dict comprehension
    if state.peek_matches(&Token::Colon) {
        state.advance();
        let first_value = state.parse_expression()?;
        
        if state.peek_matches(&Token::For) {
            parse_dict_comprehension(state, first_key, first_value, span_start)
        } else {
            parse_dict_elements_cont(state, first_key, first_value, span_start)
        }
    } else {
        bail!("Expected ':' after dict key");
    }
}

/// Parse dict elements (after first pair)
fn parse_dict_elements(
    state: &mut ParserState,
    first_key: Expr,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::Colon)?;
    let first_value = state.parse_expression()?;
    parse_dict_elements_cont(state, first_key, first_value, span_start)
}

/// Continue parsing dict elements
fn parse_dict_elements_cont(
    state: &mut ParserState,
    first_key: Expr,
    first_value: Expr,
    span_start: Span,
) -> Result<Expr> {
    let mut keys = vec![first_key];
    let mut values = vec![first_value];
    
    while state.peek_matches(&Token::Comma) {
        state.advance();
        
        // Allow trailing comma
        if state.peek_matches(&Token::RightBrace) {
            break;
        }
        
        keys.push(state.parse_expression()?);
        state.expect_token(Token::Colon)?;
        values.push(state.parse_expression()?);
    }
    
    state.expect_token(Token::RightBrace)?;
    Ok(create_dict_expr(keys, values, span_start, state))
}

/// Create dict expression
fn create_dict_expr(
    keys: Vec<Expr>,
    values: Vec<Expr>,
    span_start: Span,
    state: &ParserState,
) -> Expr {
    Expr {
        kind: ExprKind::Dict { keys, values },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    }
}

/// Parse dict comprehension
fn parse_dict_comprehension(
    state: &mut ParserState,
    key_expr: Expr,
    value_expr: Expr,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::For)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::In)?;
    let iterable = Box::new(state.parse_expression()?);
    
    // Parse optional filter
    let filter = if state.peek_matches(&Token::If) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    state.expect_token(Token::RightBrace)?;
    
    Ok(Expr {
        kind: ExprKind::DictComp {
            key: Box::new(key_expr),
            value: Box::new(value_expr),
            pattern,
            iterable,
            filter,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}