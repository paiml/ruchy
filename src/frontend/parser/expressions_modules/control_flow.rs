//! Control flow parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, Pattern, Span};
use crate::frontend::ast::MatchArm;
use anyhow::bail;

/// Parse if expression
pub fn parse_if(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::If)?;
    
    // Check for if-let pattern
    if state.peek_matches(&Token::Let) {
        parse_if_let(state, span_start)
    } else {
        parse_regular_if(state, span_start)
    }
}

/// Parse regular if expression
fn parse_regular_if(state: &mut ParserState, span_start: Span) -> Result<Expr> {
    let condition = Box::new(state.parse_expression()?);
    let then_expr = parse_if_block(state)?;
    let else_expr = parse_else_clause(state)?;
    
    Ok(Expr {
        kind: ExprKind::If {
            condition,
            then_expr,
            else_expr,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse if-let expression
fn parse_if_let(state: &mut ParserState, span_start: Span) -> Result<Expr> {
    state.expect_token(Token::Let)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::Equals)?;
    let expr = Box::new(state.parse_expression()?);
    
    let then_block = parse_if_block(state)?;
    let else_block = parse_else_clause(state)?;
    
    Ok(Expr {
        kind: ExprKind::IfLet {
            pattern,
            expr,
            then_block,
            else_block,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse if/then block
fn parse_if_block(state: &mut ParserState) -> Result<Box<Expr>> {
    if state.peek_matches(&Token::LeftBrace) {
        Ok(Box::new(state.parse_block()?))
    } else {
        state.expect_token(Token::Then)?;
        Ok(Box::new(state.parse_expression()?))
    }
}

/// Parse else clause
fn parse_else_clause(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if state.peek_matches(&Token::Else) {
        state.advance();
        
        if state.peek_matches(&Token::If) {
            Ok(Some(Box::new(parse_if(state)?)))
        } else if state.peek_matches(&Token::LeftBrace) {
            Ok(Some(Box::new(state.parse_block()?)))
        } else {
            Ok(Some(Box::new(state.parse_expression()?)))
        }
    } else {
        Ok(None)
    }
}

/// Parse match expression
pub fn parse_match(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Match)?;
    
    let expr = Box::new(state.parse_expression()?);
    state.expect_token(Token::LeftBrace)?;
    
    let mut arms = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        arms.push(parse_match_arm(state)?);
        
        // Optional comma
        if state.peek_matches(&Token::Comma) {
            state.advance();
        }
    }
    
    state.expect_token(Token::RightBrace)?;
    
    if arms.is_empty() {
        bail!("Match expression must have at least one arm");
    }
    
    Ok(Expr {
        kind: ExprKind::Match { expr, arms },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse match arm
fn parse_match_arm(state: &mut ParserState) -> Result<MatchArm> {
    let pattern = state.parse_pattern()?;
    let guard = parse_match_guard(state)?;
    
    state.expect_token(Token::FatArrow)?;
    let body = state.parse_expression()?;
    
    Ok(MatchArm {
        pattern,
        guard,
        body,
    })
}

/// Parse match guard
fn parse_match_guard(state: &mut ParserState) -> Result<Option<Expr>> {
    if state.peek_matches(&Token::If) {
        state.advance();
        Ok(Some(state.parse_expression()?))
    } else {
        Ok(None)
    }
}

/// Parse while loop
pub fn parse_while(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::While)?;
    
    // Check for while-let
    if state.peek_matches(&Token::Let) {
        parse_while_let(state, span_start)
    } else {
        parse_regular_while(state, span_start)
    }
}

/// Parse regular while loop
fn parse_regular_while(state: &mut ParserState, span_start: Span) -> Result<Expr> {
    let condition = Box::new(state.parse_expression()?);
    let body = Box::new(parse_loop_body(state)?);
    
    Ok(Expr {
        kind: ExprKind::While { condition, body },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse while-let loop
fn parse_while_let(state: &mut ParserState, span_start: Span) -> Result<Expr> {
    state.expect_token(Token::Let)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::Equals)?;
    let expr = Box::new(state.parse_expression()?);
    let body = Box::new(parse_loop_body(state)?);
    
    Ok(Expr {
        kind: ExprKind::WhileLet {
            pattern,
            expr,
            body,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse for loop
pub fn parse_for(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::For)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::In)?;
    let iterable = Box::new(state.parse_expression()?);
    let body = Box::new(parse_loop_body(state)?);
    
    Ok(Expr {
        kind: ExprKind::For {
            pattern,
            iterable,
            body,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse loop expression
pub fn parse_loop(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Loop)?;
    
    let body = Box::new(parse_loop_body(state)?);
    
    Ok(Expr {
        kind: ExprKind::Loop { body },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse loop body (block or single expression)
fn parse_loop_body(state: &mut ParserState) -> Result<Expr> {
    if state.peek_matches(&Token::LeftBrace) {
        state.parse_block()
    } else {
        state.parse_expression()
    }
}

/// Parse break expression
pub fn parse_break(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Break)?;
    
    let value = if is_expression_start(state) {
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Expr {
        kind: ExprKind::Break { value },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse continue expression
pub fn parse_continue(state: &mut ParserState) -> Result<Expr> {
    let span = state.current_span();
    state.expect_token(Token::Continue)?;
    
    Ok(Expr {
        kind: ExprKind::Continue,
        span,
        attributes: vec![],
    })
}

/// Parse return expression
pub fn parse_return(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Return)?;
    
    let value = if is_expression_start(state) {
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Expr {
        kind: ExprKind::Return { value },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Check if next token starts an expression
fn is_expression_start(state: &ParserState) -> bool {
    if let Ok((token, _)) = state.peek_token() {
        !matches!(token, 
            Token::Semicolon | Token::RightBrace | Token::RightParen |
            Token::Comma | Token::EOF
        )
    } else {
        false
    }
}