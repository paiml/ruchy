//! Type-related parsing - minimal version with only used functions

use super::{ParserState, *};

pub fn parse_struct_literal(
    state: &mut ParserState,
    name: String,
    start_span: Span,
) -> Result<Expr> {
    state.tokens.expect(&Token::LeftBrace)?;

    let mut fields = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field name
        let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name");
        };

        // Parse colon and value
        state.tokens.expect(&Token::Colon)?;
        let value = super::parse_expr_recursive(state)?;

        fields.push((field_name, value));

        // Handle comma or end of struct literal
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        } else {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::StructLiteral { name, fields },
        start_span,
    ))
}