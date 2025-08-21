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
        Token::RawString(s) => {
            state.tokens.advance();
            // Raw strings don't support interpolation
            Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span_clone))
        }
        Token::FString(s) => {
            state.tokens.advance();
            // F-strings always have string interpolation
            let parts = utils::parse_string_interpolation(state, &s);
            Ok(Expr::new(
                ExprKind::StringInterpolation { parts },
                span_clone,
            ))
        }
        Token::Char(c) => {
            let value = c;
            state.tokens.advance();
            Ok(Expr::new(
                ExprKind::Literal(Literal::Char(value)),
                span_clone,
            ))
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

            // Check for qualified name (module::name)
            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                state.tokens.advance(); // consume ::
                
                // Handle special tokens after ::
                match state.tokens.peek() {
                    Some((Token::Ok, _)) if name == "Result" => {
                        state.tokens.advance(); // consume Ok
                        state.tokens.expect(&Token::LeftParen)?;
                        let value = super::parse_expr_recursive(state)?;
                        state.tokens.expect(&Token::RightParen)?;
                        
                        return Ok(Expr::new(
                            ExprKind::Ok {
                                value: Box::new(value),
                            },
                            span_clone,
                        ));
                    }
                    Some((Token::Err, _)) if name == "Result" => {
                        state.tokens.advance(); // consume Err
                        state.tokens.expect(&Token::LeftParen)?;
                        let value = super::parse_expr_recursive(state)?;
                        state.tokens.expect(&Token::RightParen)?;
                        
                        return Ok(Expr::new(
                            ExprKind::Err {
                                error: Box::new(value),
                            },
                            span_clone,
                        ));
                    }
                    Some((Token::Some, _)) if name == "Option" => {
                        state.tokens.advance(); // consume Some
                        state.tokens.expect(&Token::LeftParen)?;
                        let value = super::parse_expr_recursive(state)?;
                        state.tokens.expect(&Token::RightParen)?;
                        
                        return Ok(Expr::new(
                            ExprKind::Call {
                                func: Box::new(Expr::new(
                                    ExprKind::Identifier("Some".to_string()),
                                    span_clone,
                                )),
                                args: vec![value],
                            },
                            span_clone,
                        ));
                    }
                    Some((Token::None, _)) if name == "Option" => {
                        state.tokens.advance(); // consume None
                        
                        return Ok(Expr::new(
                            ExprKind::Call {
                                func: Box::new(Expr::new(
                                    ExprKind::Identifier("None".to_string()),
                                    span_clone,
                                )),
                                args: vec![],
                            },
                            span_clone,
                        ));
                    }
                    Some((Token::Identifier(qualified_name), _)) => {
                        let qualified_name = qualified_name.clone();
                        state.tokens.advance();
                        
                        // Check if qualified name is a Result constructor
                        if name == "Result" && (qualified_name == "Ok" || qualified_name == "Err")
                            && matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                                state.tokens.advance(); // consume (
                                let value = super::parse_expr_recursive(state)?;
                                state.tokens.expect(&Token::RightParen)?;
                                
                                if qualified_name == "Ok" {
                                    return Ok(Expr::new(
                                        ExprKind::Ok {
                                            value: Box::new(value),
                                        },
                                        span_clone,
                                    ));
                                }
                                return Ok(Expr::new(
                                    ExprKind::Err {
                                        error: Box::new(value),
                                    },
                                    span_clone,
                                ));
                            }
                        
                        return Ok(Expr::new(
                            ExprKind::QualifiedName {
                                module: name,
                                name: qualified_name,
                            },
                            span_clone,
                        ));
                    }
                    _ => bail!("Expected identifier after '::'")
                }
            }

            // Check for Result constructors Ok and Err (unqualified)
            if name == "Ok" || name == "Err" {
                // Expect parentheses with value
                if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                    state.tokens.advance(); // consume (
                    let value = super::parse_expr_recursive(state)?;
                    state.tokens.expect(&Token::RightParen)?;

                    if name == "Ok" {
                        return Ok(Expr::new(
                            ExprKind::Ok {
                                value: Box::new(value),
                            },
                            span_clone,
                        ));
                    }
                    return Ok(Expr::new(
                        ExprKind::Err {
                            error: Box::new(value),
                        },
                        span_clone,
                    ));
                }
            }

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
                let first_expr = super::parse_expr_recursive(state)?;

                // Check if this is a tuple or just a grouped expression
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    // It's a tuple!
                    let mut elements = vec![first_expr];

                    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance(); // consume comma

                        // Allow trailing comma
                        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                            break;
                        }

                        elements.push(super::parse_expr_recursive(state)?);
                    }

                    state.tokens.expect(&Token::RightParen)?;
                    Ok(Expr::new(ExprKind::Tuple(elements), span_clone))
                } else {
                    // Just a grouped expression
                    state.tokens.expect(&Token::RightParen)?;
                    Ok(first_expr)
                }
            }
        }
        Token::Async => {
            // Check if it's async function or async block
            if matches!(state.tokens.peek_nth(1), Some((Token::Fun | Token::Fn, _))) {
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
        Token::Pub => {
            state.tokens.advance(); // consume pub
            
            // For now, we only support pub fn/fun
            if matches!(state.tokens.peek(), Some((Token::Fun | Token::Fn, _))) {
                let func_expr = functions::parse_function(state)?;
                // Mark the function as public (would need to add is_pub field to Function)
                // For now, just return the function as-is since we don't have visibility tracking yet
                Ok(func_expr)
            } else {
                bail!("Expected 'fn' or 'fun' after 'pub'")
            }
        }
        Token::Fun | Token::Fn => functions::parse_function(state),
        Token::Backslash | Token::Pipe => functions::parse_lambda(state),
        Token::Match => control_flow::parse_match(state),
        Token::For => control_flow::parse_for(state),
        Token::While => control_flow::parse_while(state),
        Token::Loop => control_flow::parse_loop(state),
        Token::Break => Ok(control_flow::parse_break(state)),
        Token::Continue => Ok(control_flow::parse_continue(state)),
        Token::Try => control_flow::parse_try_catch(state),
        Token::Return => control_flow::parse_return(state),
        Token::Result => {
            state.tokens.advance(); // consume Result
            
            // Check for qualified Result::Ok or Result::Err
            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                state.tokens.advance(); // consume ::
                
                match state.tokens.peek() {
                    Some((Token::Ok, _)) => {
                        state.tokens.advance(); // consume Ok
                        state.tokens.expect(&Token::LeftParen)?;
                        let value = Box::new(super::parse_expr_recursive(state)?);
                        state.tokens.expect(&Token::RightParen)?;
                        
                        Ok(Expr::new(ExprKind::Ok { value }, span_clone))
                    }
                    Some((Token::Err, _)) => {
                        state.tokens.advance(); // consume Err
                        state.tokens.expect(&Token::LeftParen)?;
                        let error = Box::new(super::parse_expr_recursive(state)?);
                        state.tokens.expect(&Token::RightParen)?;
                        
                        Ok(Expr::new(ExprKind::Err { error }, span_clone))
                    }
                    _ => bail!("Expected Ok or Err after Result::")
                }
            } else {
                // Just Result as an identifier
                Ok(Expr::new(ExprKind::Identifier("Result".to_string()), span_clone))
            }
        }
        Token::Option => {
            state.tokens.advance(); // consume Option
            
            // Check for qualified Option::Some or Option::None
            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                state.tokens.advance(); // consume ::
                
                match state.tokens.peek() {
                    Some((Token::Some, _)) => {
                        state.tokens.advance(); // consume Some
                        state.tokens.expect(&Token::LeftParen)?;
                        let value = Box::new(super::parse_expr_recursive(state)?);
                        state.tokens.expect(&Token::RightParen)?;
                        
                        Ok(Expr::new(ExprKind::Some { value }, span_clone))
                    }
                    Some((Token::None, _)) => {
                        state.tokens.advance(); // consume None
                        Ok(Expr::new(ExprKind::None, span_clone))
                    }
                    _ => bail!("Expected Some or None after Option::")
                }
            } else {
                // Just Option as an identifier
                Ok(Expr::new(ExprKind::Identifier("Option".to_string()), span_clone))
            }
        }
        Token::Ok => {
            state.tokens.advance(); // consume Ok
            state.tokens.expect(&Token::LeftParen)?;
            let value = Box::new(super::parse_expr_recursive(state)?);
            state.tokens.expect(&Token::RightParen)?;
            Ok(Expr::new(ExprKind::Ok { value }, span_clone))
        }
        Token::Err => {
            state.tokens.advance(); // consume Err
            state.tokens.expect(&Token::LeftParen)?;
            let error = Box::new(super::parse_expr_recursive(state)?);
            state.tokens.expect(&Token::RightParen)?;
            Ok(Expr::new(ExprKind::Err { error }, span_clone))
        }
        Token::Some => {
            state.tokens.advance(); // consume Some
            state.tokens.expect(&Token::LeftParen)?;
            let value = Box::new(super::parse_expr_recursive(state)?);
            state.tokens.expect(&Token::RightParen)?;
            Ok(Expr::new(ExprKind::Some { value }, span_clone))
        }
        Token::None => {
            state.tokens.advance(); // consume None
            Ok(Expr::new(ExprKind::None, span_clone))
        }
        Token::Throw => {
            state.tokens.advance(); // consume throw
            let expr = super::parse_expr_recursive(state)?;
            Ok(Expr::new(
                ExprKind::Throw {
                    expr: Box::new(expr),
                },
                span_clone,
            ))
        }
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
        Token::Enum => types::parse_enum(state),
        Token::Trait => types::parse_trait(state),
        Token::Impl => types::parse_impl(state),
        Token::Extend => types::parse_extend(state),
        Token::Actor => actors::parse_actor(state),
        Token::Import | Token::Use => utils::parse_import(state),
        Token::Module => utils::parse_module(state),
        Token::Export => utils::parse_export(state),
        Token::OrOr => functions::parse_empty_lambda(state),
        Token::DataFrame => collections::parse_dataframe(state),
        Token::Minus | Token::Bang | Token::Tilde | Token::Ampersand => {
            let op_token = state.tokens.advance().expect("checked").0;
            let op = match op_token {
                Token::Minus => UnaryOp::Negate,
                Token::Bang => UnaryOp::Not,
                Token::Tilde => UnaryOp::BitwiseNot,
                Token::Ampersand => UnaryOp::Reference,
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
        Token::Increment => {
            state.tokens.advance(); // consume ++
            let target = parse_prefix(state)?;
            Ok(Expr::new(
                ExprKind::PreIncrement {
                    target: Box::new(target),
                },
                span_clone,
            ))
        }
        Token::Decrement => {
            state.tokens.advance(); // consume --
            let target = parse_prefix(state)?;
            Ok(Expr::new(
                ExprKind::PreDecrement {
                    target: Box::new(target),
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
