//! Type-related parsing - minimal version with only used functions
use super::{bail, Expr, ExprKind, ParserState, Result, Span, Token};
pub fn parse_struct_literal(
    state: &mut ParserState,
    name: String,
    start_span: Span,
) -> Result<Expr> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();
    let mut base = None;

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Check for update syntax ..expr
        if matches!(state.tokens.peek(), Some((Token::DotDot, _))) {
            state.tokens.advance(); // consume ..
            base = Some(Box::new(super::parse_expr_recursive(state)?));
            // After base expression, we should only have optional comma and closing brace
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            }
            break;
        }

        // Parse field name
        let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let name = name.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name");
        };
        // Parse colon and value, or use field shorthand
        let value = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
            state.tokens.advance(); // consume :
            super::parse_expr_recursive(state)?
        } else {
            // Field shorthand: use field name as identifier expression
            Expr::new(ExprKind::Identifier(field_name.clone()), start_span)
        };
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
        ExprKind::StructLiteral { name, fields, base },
        start_span,
    ))
}
