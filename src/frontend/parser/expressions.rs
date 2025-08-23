//! Basic expression parsing (literals, identifiers, binary/unary operations)

use super::{ParserState, *};

/// Parse module path segments like `std::fs::read_file`
/// Complexity: 3 (PMAT target <10)
fn parse_module_path_segments(state: &mut ParserState, first_segment: String) -> Result<Vec<String>> {
    let mut path_segments = vec![first_segment];
    
    // Keep consuming :: and identifiers to build the full path
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        // Peek ahead to see if this is a special case
        let is_special_case = should_break_for_special_case(state, &path_segments);
        if is_special_case {
            break;
        }
        
        state.tokens.advance(); // consume ::
        
        // Get the next segment
        let next_segment = match state.tokens.peek() {
            Some((Token::Identifier(next_segment), _)) => next_segment.clone(),
            // Handle special tokens that can be part of module paths
            Some((Token::Result, _)) => "Result".to_string(),
            Some((Token::Option, _)) => "Option".to_string(),
            Some((Token::Ok, _)) => {
                // Only break for Ok/Err/Some/None if they follow certain patterns
                break;
            }
            Some((Token::Err, _)) => {
                break;
            }
            Some((Token::Some, _)) => {
                break;  
            }
            Some((Token::None, _)) => {
                break;
            }
            _ => bail!("Expected identifier after '::'")
        };
        
        path_segments.push(next_segment);
        state.tokens.advance();
    }
    
    Ok(path_segments)
}

/// Check if we should break module path parsing for special cases
/// Complexity: 3 (PMAT target <10)  
fn should_break_for_special_case(state: &mut ParserState, path_segments: &[String]) -> bool {
    // Check if the last segment is "Result" or "Option" and next tokens are special
    let last_segment = path_segments.last().unwrap();
    
    let is_result_ok_err = last_segment == "Result" 
        && matches!(state.tokens.peek_nth(1), Some((Token::Ok | Token::Err, _)));
    let is_option_some_none = last_segment == "Option"
        && matches!(state.tokens.peek_nth(1), Some((Token::Some | Token::None, _)));
        
    is_result_ok_err || is_option_some_none
}

/// Parse `Result::Ok(value)` or `Result::Err(error)` constructs
/// Complexity: 4 (PMAT target <10)
fn parse_result_constructor(state: &mut ParserState, span: Span) -> Result<Option<Expr>> {
    if !matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        return Ok(None);
    }
    
    state.tokens.advance(); // consume ::
    
    match state.tokens.peek() {
        Some((Token::Ok, _)) => {
            state.tokens.advance(); // consume Ok
            state.tokens.expect(&Token::LeftParen)?;
            let value = super::parse_expr_recursive(state)?;
            state.tokens.expect(&Token::RightParen)?;
            Ok(Some(Expr::new(ExprKind::Ok { value: Box::new(value) }, span)))
        }
        Some((Token::Err, _)) => {
            state.tokens.advance(); // consume Err
            state.tokens.expect(&Token::LeftParen)?;
            let value = super::parse_expr_recursive(state)?;
            state.tokens.expect(&Token::RightParen)?;
            Ok(Some(Expr::new(ExprKind::Err { error: Box::new(value) }, span)))
        }
        _ => Ok(None)
    }
}

/// Parse `Option::Some(value)` or `Option::None` constructs
/// Complexity: 4 (PMAT target <10)
fn parse_option_constructor(state: &mut ParserState, span: Span) -> Result<Option<Expr>> {
    if !matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        return Ok(None);
    }
    
    state.tokens.advance(); // consume ::
    
    match state.tokens.peek() {
        Some((Token::Some, _)) => {
            state.tokens.advance(); // consume Some
            state.tokens.expect(&Token::LeftParen)?;
            let value = super::parse_expr_recursive(state)?;
            state.tokens.expect(&Token::RightParen)?;
            Ok(Some(Expr::new(
                ExprKind::Call {
                    func: Box::new(Expr::new(ExprKind::Identifier("Some".to_string()), span)),
                    args: vec![value],
                },
                span,
            )))
        }
        Some((Token::None, _)) => {
            state.tokens.advance(); // consume None
            Ok(Some(Expr::new(
                ExprKind::Call {
                    func: Box::new(Expr::new(ExprKind::Identifier("None".to_string()), span)),
                    args: vec![],
                },
                span,
            )))
        }
        _ => Ok(None)
    }
}

/// Create qualified name expression from path segments
/// Complexity: 1 (PMAT target <10)
fn create_qualified_name_expr(path_segments: Vec<String>, span: Span) -> Expr {
    if path_segments.len() > 1 {
        // Join all but the last segment as the module path
        let module_path = path_segments[0..path_segments.len()-1].join("::");
        let name = path_segments.last().unwrap().clone();
        
        Expr::new(
            ExprKind::QualifiedName {
                module: module_path,
                name,
            },
            span,
        )
    } else {
        // Single identifier
        Expr::new(ExprKind::Identifier(path_segments[0].clone()), span)
    }
}

/// Parse unqualified Result constructors Ok(value) or Err(error)
/// Complexity: 4 (PMAT target <10)
fn parse_unqualified_result_constructor(state: &mut ParserState, name: &str, span: Span) -> Result<Option<Expr>> {
    if name != "Ok" && name != "Err" {
        return Ok(None);
    }
    
    // Expect parentheses with value
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        state.tokens.advance(); // consume (
        let value = super::parse_expr_recursive(state)?;
        state.tokens.expect(&Token::RightParen)?;

        if name == "Ok" {
            Ok(Some(Expr::new(ExprKind::Ok { value: Box::new(value) }, span)))
        } else {
            Ok(Some(Expr::new(ExprKind::Err { error: Box::new(value) }, span)))
        }
    } else {
        Ok(None)
    }
}

/// Main coordinator function for parsing identifier tokens
/// Complexity: 7 (PMAT target <10) 
fn parse_identifier_token(state: &mut ParserState, name: String, span: Span) -> Result<Expr> {
    // Parse the full module path (e.g., std::fs::read_file)
    let path_segments = parse_module_path_segments(state, name)?;
    
    // Handle special cases based on the last segment
    let last_segment = path_segments.last().unwrap();
    
    // Try Result::Ok/Err constructors (works for any path length)
    if last_segment == "Result" {
        if let Some(expr) = parse_result_constructor(state, span)? {
            return Ok(expr);
        }
    }
    
    // Try Option::Some/None constructors (works for any path length)
    if last_segment == "Option" {
        if let Some(expr) = parse_option_constructor(state, span)? {
            return Ok(expr);
        }
    }
    
    // Try unqualified Ok/Err constructors (only for single segments)
    if path_segments.len() == 1 {
        if let Some(expr) = parse_unqualified_result_constructor(state, last_segment, span)? {
            return Ok(expr);
        }
    }
    
    // Create the appropriate expression based on path length
    Ok(create_qualified_name_expr(path_segments, span))
}

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
            
            // Check for macro call (identifier!) first
            if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
                state.tokens.advance(); // consume !
                
                // Determine if using parentheses or square brackets
                let (open_token, close_token) = match state.tokens.peek() {
                    Some((Token::LeftParen, _)) => (Token::LeftParen, Token::RightParen),
                    Some((Token::LeftBracket, _)) => (Token::LeftBracket, Token::RightBracket),
                    _ => bail!("Expected '(' or '[' after macro name"),
                };
                
                state.tokens.expect(&open_token)?;
                let mut args = Vec::new();
                
                // Parse macro arguments
                if !matches!(state.tokens.peek(), Some((token, _)) if *token == close_token) {
                    loop {
                        args.push(super::parse_expr_recursive(state)?);
                        
                        match state.tokens.peek() {
                            Some((Token::Comma, _)) => {
                                state.tokens.advance(); // consume comma
                                // Allow trailing comma
                                if matches!(state.tokens.peek(), Some((token, _)) if *token == close_token) {
                                    break;
                                }
                            }
                            Some((token, _)) if *token == close_token => break,
                            _ => bail!("Expected ',' or closing delimiter in macro arguments"),
                        }
                    }
                }
                
                state.tokens.expect(&close_token)?;
                
                return Ok(Expr::new(
                    ExprKind::Macro { name, args },
                    span_clone,
                ));
            }

            // Parse module paths and special constructors
            parse_identifier_token(state, name, span_clone)
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
            
            // Check what follows pub
            match state.tokens.peek() {
                Some((Token::Fun | Token::Fn, _)) => {
                    functions::parse_function_with_visibility(state, true)
                }
                Some((Token::Struct, _)) => {
                    types::parse_struct_with_visibility(state, true)
                }
                Some((Token::Enum, _)) => {
                    types::parse_enum_with_visibility(state, true)
                }
                Some((Token::Trait, _)) => {
                    types::parse_trait_with_visibility(state, true)
                }
                Some((Token::Impl, _)) => {
                    types::parse_impl_with_visibility(state, true)
                }
                Some((Token::Mod, _)) => {
                    // For now, treat pub mod the same as mod (visibility not yet fully implemented)
                    parse_module(state)
                }
                _ => bail!("Expected 'fn', 'struct', 'enum', 'trait', 'impl', or 'mod' after 'pub'")
            }
        }
        Token::Fun | Token::Fn => functions::parse_function(state),
        Token::Backslash | Token::Pipe => functions::parse_lambda(state),
        Token::Match => control_flow::parse_match(state),
        Token::For => control_flow::parse_for(state),
        Token::While => control_flow::parse_while(state),
        Token::Loop => control_flow::parse_loop(state),
        Token::Mod => parse_module(state),
        Token::Break => Ok(control_flow::parse_break(state)),
        Token::Continue => Ok(control_flow::parse_continue(state)),
        Token::Return => control_flow::parse_return(state),
        Token::Command => parse_command(state),
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
        BinaryOp::LeftShift => 8,
        BinaryOp::Add | BinaryOp::Subtract => 9,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 10,
        BinaryOp::Power => 11,
    }
}

/// Parse command expression: command `program` with optional args
fn parse_command(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'command'
    
    // Parse program name (string literal)
    let program = if let Some((Token::String(prog), _)) = state.tokens.peek() {
        let prog = prog.clone();
        state.tokens.advance();
        prog
    } else {
        bail!("Expected string literal for command program");
    };
    
    // Parse optional arguments list
    let args = if matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
        state.tokens.advance(); // consume '['
        let mut args = Vec::new();
        
        while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            if let Some((Token::String(arg), _)) = state.tokens.peek() {
                args.push(arg.clone());
                state.tokens.advance();
                
                // Handle comma
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                }
            } else {
                bail!("Expected string literal in command arguments");
            }
        }
        
        state.tokens.expect(&Token::RightBracket)?;
        args
    } else {
        Vec::new()
    };
    
    Ok(Expr {
        kind: ExprKind::Command {
            program,
            args,
            env: Vec::new(),
            working_dir: None,
        },
        span: Span { start: 0, end: 0 },
        attributes: Vec::new(),
    })
}

/// Parse a module definition
pub fn parse_module(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume mod
    
    // Parse module name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected module name after 'mod'");
    };
    
    // Parse module body
    state.tokens.expect(&Token::LeftBrace)?;
    
    // Parse module contents as a block
    let mut items = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        items.push(super::parse_expr_recursive(state)?);
        
        // Optional semicolon between items
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    let body = if items.len() == 1 {
        items.into_iter().next().expect("checked that len == 1")
    } else {
        Expr::new(
            ExprKind::Block(items),
            start_span,
        )
    };
    
    Ok(Expr::new(
        ExprKind::Module {
            name,
            body: Box::new(body),
        },
        start_span,
    ))
}
