//! Actor system parsing

use super::{ParserState, *};
use crate::frontend::ast::{ActorHandler, StructField};

/// # Errors
///
/// Returns an error if the operation fails
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
        if matches!(state.tokens.peek(), Some((Token::State, _))) {
            // Parse state block
            state.tokens.advance(); // consume 'state'
            state.tokens.expect(&Token::LeftBrace)?;

            // Parse state fields
            while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    let name = name.clone();
                    state.tokens.advance();
                    name
                } else {
                    bail!("Expected field name in state block");
                };

                state.tokens.expect(&Token::Colon)?;
                let ty = utils::parse_type(state)?;

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

            state.tokens.expect(&Token::RightBrace)?; // Close state block
        } else if matches!(state.tokens.peek(), Some((Token::Receive, _))) {
            state.tokens.advance(); // consume 'receive'

            // Check if it's a receive block or individual handler
            if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                // Parse receive block with multiple handlers
                state.tokens.advance(); // consume {

                while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                    // Parse message pattern
                    let message_type =
                        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                            let name = name.clone();
                            state.tokens.advance();
                            name
                        } else {
                            bail!("Expected message type in receive block");
                        };

                    // Parse optional parameters
                    let params = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                        utils::parse_params(state)?
                    } else {
                        Vec::new()
                    };

                    // Expect => for handler body
                    state.tokens.expect(&Token::FatArrow)?;

                    // Parse handler body
                    let body = if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                        Box::new(collections::parse_block(state)?)
                    } else {
                        // Single expression, not a block
                        Box::new(super::parse_expr_recursive(state)?)
                    };

                    handlers.push(ActorHandler {
                        message_type,
                        params,
                        body,
                    });

                    // Optional separator (comma or newline already consumed)
                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    }
                }

                state.tokens.expect(&Token::RightBrace)?; // Close receive block
            } else {
                // Parse individual receive handler
                // Parse message pattern (e.g., MessageType(params))
                let message_type = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    let name = name.clone();
                    state.tokens.advance();
                    name
                } else {
                    bail!("Expected message type after receive");
                };

                // Parse optional parameters
                let params = if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                    utils::parse_params(state)?
                } else {
                    Vec::new()
                };

                // Parse optional return type
                let _return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
                    state.tokens.advance(); // consume ->
                    Some(utils::parse_type(state)?)
                } else {
                    None
                };

                // Parse handler body - expect block
                let body = Box::new(collections::parse_block(state)?);

                handlers.push(ActorHandler {
                    message_type,
                    params,
                    body,
                });
            }
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
