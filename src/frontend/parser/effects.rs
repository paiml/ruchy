//! Effect system parsing - SPEC-001-I, SPEC-001-J
use super::{bail, Expr, ExprKind, ParserState, Result, Token, utils};
use crate::frontend::ast::{EffectOperation, EffectHandler, Pattern};

pub fn parse_effect(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked").1;
    let name = match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => { let n = n.clone(); state.tokens.advance(); n }
        _ => bail!("Expected effect name"),
    };
    state.tokens.expect(&Token::LeftBrace)?;
    let mut operations = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let op_name = match state.tokens.peek() {
            Some((Token::Identifier(n), _)) => { let n = n.clone(); state.tokens.advance(); n }
            _ => bail!("Expected operation name"),
        };
        let params = utils::parse_params(state)?;
        let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
            state.tokens.advance();
            Some(utils::parse_type(state)?)
        } else { None };
        operations.push(EffectOperation { name: op_name, params, return_type });
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) { state.tokens.advance(); }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Effect { name, operations }, start_span))
}

/// SPEC-001-J: Parse effect handler expression
/// Syntax: handle expr with { operation => body, operation(params) => body }
pub fn parse_handler(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked").1;
    let expr = Box::new(super::parse_expr_recursive(state)?);
    state.tokens.expect(&Token::With)?;
    state.tokens.expect(&Token::LeftBrace)?;

    let mut handlers = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let operation = match state.tokens.peek() {
            Some((Token::Identifier(n), _)) => { let n = n.clone(); state.tokens.advance(); n }
            _ => bail!("Expected operation name in handler"),
        };
        let params = parse_handler_params(state)?;
        state.tokens.expect(&Token::FatArrow)?;
        let body = Box::new(super::parse_expr_recursive(state)?);
        handlers.push(EffectHandler { operation, params, body });
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) { state.tokens.advance(); }
    }

    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Handle { expr, handlers }, start_span))
}

fn parse_handler_params(state: &mut ParserState) -> Result<Vec<Pattern>> {
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        state.tokens.advance();
        let mut params = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            let param_name = match state.tokens.peek() {
                Some((Token::Identifier(n), _)) => { let n = n.clone(); state.tokens.advance(); n }
                _ => bail!("Expected parameter name"),
            };
            params.push(Pattern::Identifier(param_name));
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) { state.tokens.advance(); }
        }
        state.tokens.expect(&Token::RightParen)?;
        Ok(params)
    } else {
        Ok(Vec::new())
    }
}
