//! Basic expression parsing (literals, identifiers, binary/unary operations)

use super::{ParserState, *};

#[allow(clippy::too_many_lines)]
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input");
    };

    let token_clone = token.clone();
    let span_clone = *span;

    match token_clone {
        Token::Integer(i) => {
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::Integer(i)),
                span_clone,
            ))
        }
        Token::Float(f) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Float(f)), span_clone))
        }
        Token::String(s) => {
            state.tokens.advance();
            // Check if the string contains interpolation markers
            if s.contains('{') && s.contains('}') {
                let parts = utils::parse_string_interpolation(state, &s);
                Ok(Expr::new(
                    ExprKind::StringInterpolation { parts },
                    span_clone,
                ))
            } else {
                Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span_clone))
            }
        }
        Token::Bool(b) => {
            let value = b;
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::Bool(value)),
                span_clone,
            ))
        }
        Token::Identifier(name) => {
            state.tokens.advance();
            // Only handle postfix operators that can't be confused with binary operators
            Ok(Expr::new(ExprKind::Identifier(name), span_clone))
        }
        Token::LeftParen => {
            state.tokens.advance(); // consume (
                                    // Check for unit literal ()
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                state.tokens.advance(); // consume )
                Ok(Expr::new(ExprKind::Literal(Literal::Unit), span_clone))
            } else {
                let expr = super::parse_expr_recursive(state)?;
                state.tokens.expect(&Token::RightParen)?;
                Ok(expr)
            }
        }
        Token::Async => {
            // Check if it's async function or async block
            if matches!(state.tokens.peek_nth(1), Some((Token::Fun, _))) {
                // async fun - parse as async function
                state.tokens.advance(); // consume async
                let mut func_expr = functions::parse_function(state)?;
                // Mark the function as async
                if let ExprKind::Function { is_async, .. } = &mut func_expr.kind {
                    *is_async = true;
                }
                Ok(func_expr)
            } else {
                // async block
                control_flow::parse_async_block(state)
            }
        }
        Token::If => control_flow::parse_if(state),
        Token::Let => control_flow::parse_let(state),
        Token::Fun => functions::parse_function(state),
        Token::Match => control_flow::parse_match(state),
        Token::For => control_flow::parse_for(state),
        Token::While => control_flow::parse_while(state),
        Token::Break => Ok(control_flow::parse_break(state)),
        Token::Continue => Ok(control_flow::parse_continue(state)),
        Token::Try => control_flow::parse_try_catch(state),
        Token::Await => {
            // Parse as prefix but it will transpile to postfix
            state.tokens.advance(); // consume await
                                    // Parse the full expression including postfix operations like calls
            let expr = super::parse_expr_recursive(state)?;
            Ok(Expr::new(
                ExprKind::Await {
                    expr: Box::new(expr),
                },
                span_clone,
            ))
        }
        Token::LeftBrace => collections::parse_block(state),
        Token::LeftBracket => collections::parse_list(state),
        Token::Struct => types::parse_struct(state),
        Token::Trait => types::parse_trait(state),
        Token::Impl => types::parse_impl(state),
        Token::Actor => actors::parse_actor(state),
        Token::Import | Token::Use => utils::parse_import(state),
        Token::Pipe => functions::parse_lambda(state),
        Token::OrOr => functions::parse_empty_lambda(state),
        Token::DataFrame => collections::parse_dataframe(state),
        Token::Minus | Token::Bang | Token::Tilde => {
            let op_token = state.tokens.advance().expect("checked").0;
            let op = match op_token {
                Token::Minus => UnaryOp::Negate,
                Token::Bang => UnaryOp::Not,
                Token::Tilde => UnaryOp::BitwiseNot,
                _ => unreachable!(),
            };
            let operand = parse_prefix(state)?;
            Ok(Expr::new(
                ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span_clone,
            ))
        }
        _ => bail!("Unexpected token: {:?}", token_clone),
    }
}

pub fn token_to_binary_op(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Plus => Some(BinaryOp::Add),
        Token::Minus => Some(BinaryOp::Subtract),
        Token::Star => Some(BinaryOp::Multiply),
        Token::Slash => Some(BinaryOp::Divide),
        Token::Percent => Some(BinaryOp::Modulo),
        Token::Power => Some(BinaryOp::Power),
        Token::EqualEqual => Some(BinaryOp::Equal),
        Token::NotEqual => Some(BinaryOp::NotEqual),
        Token::Less => Some(BinaryOp::Less),
        Token::LessEqual => Some(BinaryOp::LessEqual),
        Token::Greater => Some(BinaryOp::Greater),
        Token::GreaterEqual => Some(BinaryOp::GreaterEqual),
        Token::AndAnd => Some(BinaryOp::And),
        Token::OrOr => Some(BinaryOp::Or),
        Token::Ampersand => Some(BinaryOp::BitwiseAnd),
        Token::Pipe => Some(BinaryOp::BitwiseOr),
        Token::Caret => Some(BinaryOp::BitwiseXor),
        Token::LeftShift => Some(BinaryOp::LeftShift),
        Token::RightShift => Some(BinaryOp::RightShift),
        _ => None,
    }
}

pub fn get_precedence(op: BinaryOp) -> i32 {
    match op {
        BinaryOp::Or => 1,
        BinaryOp::And => 2,
        BinaryOp::BitwiseOr => 3,
        BinaryOp::BitwiseXor => 4,
        BinaryOp::BitwiseAnd => 5,
        BinaryOp::Equal | BinaryOp::NotEqual => 6,
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => 7,
        BinaryOp::LeftShift | BinaryOp::RightShift => 8,
        BinaryOp::Add | BinaryOp::Subtract => 9,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 10,
        BinaryOp::Power => 11,
    }
}
