//! Actor system parsing

use super::{ParserState, *};

    /// # Errors
    ///
    /// Returns an error if the operation fails
pub fn parse_actor(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume actor

    // Parse actor name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected actor name");
    };

    state.tokens.expect(&Token::LeftBrace)?;

    let mut state_fields = Vec::new();
    let mut handlers = Vec::new();

    // Parse actor body (state fields and receive block)
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        if matches!(state.tokens.peek(), Some((Token::Receive, _))) {
            // Parse receive block
            state.tokens.advance(); // consume 'receive'
            state.tokens.expect(&Token::LeftBrace)?;

            // Parse message handlers
            while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                // Parse message pattern (e.g., MessageType(params))
                let message_type = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    let name = name.clone();
                    state.tokens.advance();
                    name
                } else {
                    bail!("Expected message type");
                };

                // Parse optional parameters
                let params = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                    utils::parse_params(state)?
                } else {
                    Vec::new()
                };

                // Expect => or ->
                if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                    state.tokens.advance();
                } else {
                    state.tokens.expect(&Token::Arrow)?;
                }

                // Parse handler body (either block or expression)
                let body = if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                    state.tokens.advance(); // Consume the LeftBrace
                    Box::new(collections::parse_block(state)?) // parse_block consumes the closing brace
                } else {
                    Box::new(super::parse_expr_recursive(state)?)
                };

                handlers.push(ActorHandler {
                    message_type,
                    params,
                    body,
                });

                // Optional comma or semicolon between handlers
                if matches!(
                    state.tokens.peek(),
                    Some((Token::Comma | Token::Semicolon, _))
                ) {
                    state.tokens.advance();
                }
            }

            state.tokens.expect(&Token::RightBrace)?; // Close receive block
        } else {
            // State field
            let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let name = name.clone();
                state.tokens.advance();
                name
            } else {
                bail!("Expected field name");
            };

            state.tokens.expect(&Token::Colon)?;
            let ty = utils::parse_type(state)?;

            // Parse optional default value
            if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
                // Skip the default value for now (not stored in AST yet)
                state.tokens.advance(); // consume =
                let _default_value = super::parse_expr_recursive(state)?;
            }

            state_fields.push(StructField {
                name: field_name,
                ty,
                is_pub: false,
            });

            // Optional comma or semicolon
            if matches!(
                state.tokens.peek(),
                Some((Token::Comma | Token::Semicolon, _))
            ) {
                state.tokens.advance();
            }
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Actor {
            name,
            state: state_fields,
            handlers,
        },
        start_span,
    ))
}
