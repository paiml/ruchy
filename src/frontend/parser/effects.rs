//! Effect system parsing - SPEC-001-I
use super::{bail, Expr, ExprKind, ParserState, Result, Token, utils};
use crate::frontend::ast::{EffectOperation};

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
