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
                // Only break if this is after Result/Option (handled by should_break_for_special_case)
                "Ok".to_string()
            }
            Some((Token::Err, _)) => {
                "Err".to_string()
            }
            Some((Token::Some, _)) => {
                "Some".to_string()
            }
            Some((Token::None, _)) => {
                "None".to_string()
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
/// Complexity: 8 (PMAT target <10) 
fn parse_identifier_token(state: &mut ParserState, name: String, span: Span) -> Result<Expr> {
    // Check for fat arrow lambda syntax: x => x * 2 (before parsing module paths)
    if !matches!(state.tokens.peek(), Some((Token::ColonColon, _))) 
        && matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        state.tokens.advance(); // consume =>
        
        // Create parameter
        let param = Param {
            pattern: Pattern::Identifier(name),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span { start: 0, end: 0 },
            },
            span,
            is_mutable: false,
            default_value: None,
        };
        
        // Parse body
        let body = super::parse_expr_recursive(state)?;
        
        return Ok(Expr::new(
            ExprKind::Lambda {
                params: vec![param],
                body: Box::new(body),
            },
            span,
        ));
    }
    
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
#[allow(clippy::cognitive_complexity)]
/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
/// Parses literal tokens (integers, floats, chars, booleans)
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("42");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_literal_prefix(&mut state).unwrap();
/// // Should parse as integer literal 42
/// ```
/// 
/// # Errors
/// Returns error if token is not a recognized literal type
fn parse_literal_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Option<Expr>> {
    match token {
        Token::Integer(i) => {
            state.tokens.advance();
            Ok(Some(Expr::new(
                ExprKind::Literal(Literal::Integer(i)),
                span,
            )))
        }
        Token::Float(f) => {
            state.tokens.advance();
            Ok(Some(Expr::new(ExprKind::Literal(Literal::Float(f)), span)))
        }
        Token::Char(c) => {
            state.tokens.advance();
            Ok(Some(Expr::new(
                ExprKind::Literal(Literal::Char(c)),
                span,
            )))
        }
        Token::Bool(b) => {
            state.tokens.advance();
            Ok(Some(Expr::new(
                ExprKind::Literal(Literal::Bool(b)),
                span,
            )))
        }
        _ => Ok(None), // Not a simple literal
    }
}

/// Parses string tokens with interpolation support
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("\"hello world\"");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_string_prefix(&mut state).unwrap();
/// // Should parse as string literal
/// ```
/// 
/// # Errors
/// Returns error if string interpolation parsing fails
fn parse_string_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Option<Expr>> {
    match token {
        Token::String(s) => {
            state.tokens.advance();
            // Check if the string contains interpolation markers
            if s.contains('{') && s.contains('}') {
                let parts = utils::parse_string_interpolation(state, &s);
                Ok(Some(Expr::new(
                    ExprKind::StringInterpolation { parts },
                    span,
                )))
            } else {
                Ok(Some(Expr::new(ExprKind::Literal(Literal::String(s)), span)))
            }
        }
        Token::RawString(s) => {
            state.tokens.advance();
            // Raw strings don't support interpolation
            Ok(Some(Expr::new(ExprKind::Literal(Literal::String(s)), span)))
        }
        Token::FString(s) => {
            state.tokens.advance();
            // F-strings always have string interpolation
            let parts = utils::parse_string_interpolation(state, &s);
            Ok(Some(Expr::new(
                ExprKind::StringInterpolation { parts },
                span,
            )))
        }
        _ => Ok(None), // Not a string token
    }
}

/// Parses identifier tokens and macro calls
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("println!(\"hello\")");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_identifier_prefix(&mut state).unwrap();
/// // Should parse as macro call
/// ```
/// 
/// # Errors
/// Returns error if macro syntax is malformed
fn parse_identifier_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Option<Expr>> {
    if let Token::Identifier(name) = token {
        state.tokens.advance();
        
        // Check for macro call (identifier!) first
        if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
            state.tokens.advance(); // consume !
            
            // Determine if using parentheses, square brackets, or braces
            let (open_token, close_token) = match state.tokens.peek() {
                Some((Token::LeftParen, _)) => (Token::LeftParen, Token::RightParen),
                Some((Token::LeftBracket, _)) => (Token::LeftBracket, Token::RightBracket),
                Some((Token::LeftBrace, _)) => (Token::LeftBrace, Token::RightBrace),
                _ => bail!("Expected '(', '[', or '{{' after macro name"),
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
            
            return Ok(Some(Expr::new(
                ExprKind::Macro { name, args },
                span,
            )));
        }

        // Parse module paths and special constructors
        Ok(Some(parse_identifier_token(state, name, span)?))
    } else {
        Ok(None) // Not an identifier
    }
}

/// Parses parentheses expressions (unit literals, lambdas, tuples, grouped expressions)
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("(x, y)");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_parentheses_prefix(&mut state).unwrap();
/// // Should parse as tuple
/// ```
/// 
/// # Errors
/// Returns error if parentheses expression is malformed
fn parse_parentheses_prefix(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume (
    
    // Check for unit literal ()
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        state.tokens.advance(); // consume )
        
        // Check for fat arrow lambda: () => expr
        if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
            state.tokens.advance(); // consume =>
            let body = super::parse_expr_recursive(state)?;
            return Ok(Expr::new(
                ExprKind::Lambda {
                    params: vec![],
                    body: Box::new(body),
                },
                span,
            ));
        }
        
        Ok(Expr::new(ExprKind::Literal(Literal::Unit), span))
    } else {
        // Try to parse as lambda parameters for fat arrow syntax
        let saved_pos = state.tokens.position();
        
        // Try to parse lambda params: (x, y) => x + y
        let mut could_be_lambda = true;
        let mut param_names = Vec::new();
        
        // Check first item
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            param_names.push(name.clone());
            state.tokens.advance();
            
            // Check for more parameters
            while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance(); // consume comma
                if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    param_names.push(name.clone());
                    state.tokens.advance();
                } else {
                    could_be_lambda = false;
                    break;
                }
            }
            
            // Check for closing paren and fat arrow
            if could_be_lambda 
                && matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                state.tokens.advance(); // consume )
                
                if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                    // It's a fat arrow lambda!
                    state.tokens.advance(); // consume =>
                    
                    // Create parameters
                    let params = param_names.into_iter().map(|name| Param {
                        pattern: Pattern::Identifier(name),
                        ty: Type {
                            kind: TypeKind::Named("_".to_string()),
                            span: Span { start: 0, end: 0 },
                        },
                        span,
                        is_mutable: false,
                        default_value: None,
                    }).collect();
                    
                    // Parse body
                    let body = super::parse_expr_recursive(state)?;
                    
                    return Ok(Expr::new(
                        ExprKind::Lambda {
                            params,
                            body: Box::new(body),
                        },
                        span,
                    ));
                }
            }
        }
        
        // Not a lambda, restore position and parse as normal expression/tuple
        state.tokens.set_position(saved_pos);
        
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
            Ok(Expr::new(ExprKind::Tuple(elements), span))
        } else {
            // Just a grouped expression
            state.tokens.expect(&Token::RightParen)?;
            Ok(first_expr)
        }
    }
}

/// Parses control flow and declaration keywords
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("if true { 42 }");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_control_flow_prefix(&mut state).unwrap();
/// // Should parse as if expression
/// ```
/// 
/// # Errors
/// Returns error if control flow syntax is malformed
fn parse_control_flow_prefix(state: &mut ParserState, token: Token, _span: Span) -> Result<Option<Expr>> {
    match token {
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
                Ok(Some(func_expr))
            } else {
                // async block
                Ok(Some(control_flow::parse_async_block(state)?))
            }
        }
        Token::If => Ok(Some(control_flow::parse_if(state)?)),
        Token::Let => Ok(Some(control_flow::parse_let(state)?)),
        Token::Pub => {
            state.tokens.advance(); // consume pub
            
            // Check what follows pub
            match state.tokens.peek() {
                Some((Token::Fun | Token::Fn, _)) => {
                    Ok(Some(functions::parse_function_with_visibility(state, true)?))
                }
                Some((Token::Struct, _)) => {
                    Ok(Some(types::parse_struct_with_visibility(state, true)?))
                }
                Some((Token::Enum, _)) => {
                    Ok(Some(types::parse_enum_with_visibility(state, true)?))
                }
                Some((Token::Trait, _)) => {
                    Ok(Some(types::parse_trait_with_visibility(state, true)?))
                }
                Some((Token::Impl, _)) => {
                    Ok(Some(types::parse_impl_with_visibility(state, true)?))
                }
                Some((Token::Mod, _)) => {
                    // For now, treat pub mod the same as mod (visibility not yet fully implemented)
                    Ok(Some(parse_module(state)?))
                }
                _ => bail!("Expected 'fn', 'struct', 'enum', 'trait', 'impl', or 'mod' after 'pub'")
            }
        }
        Token::Fun | Token::Fn => Ok(Some(functions::parse_function(state)?)),
        Token::Backslash | Token::Pipe => Ok(Some(functions::parse_lambda(state)?)),
        Token::Match => Ok(Some(control_flow::parse_match(state)?)),
        Token::For => Ok(Some(control_flow::parse_for(state)?)),
        Token::While => Ok(Some(control_flow::parse_while(state)?)),
        Token::Loop => Ok(Some(control_flow::parse_loop(state)?)),
        Token::Mod => Ok(Some(parse_module(state)?)),
        Token::Break => Ok(Some(control_flow::parse_break(state))),
        Token::Continue => Ok(Some(control_flow::parse_continue(state))),
        Token::Return => Ok(Some(control_flow::parse_return(state)?)),
        Token::Command => Ok(Some(parse_command(state)?)),
        _ => Ok(None), // Not a control flow token
    }
}

/// Parses Result/Option constructors and special enum variants
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("Some(42)");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_result_option_prefix(&mut state).unwrap();
/// // Should parse as Some constructor
/// ```
/// 
/// # Errors
/// Returns error if constructor syntax is malformed
fn parse_result_option_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Option<Expr>> {
    match token {
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
                        
                        Ok(Some(Expr::new(ExprKind::Ok { value }, span)))
                    }
                    Some((Token::Err, _)) => {
                        state.tokens.advance(); // consume Err
                        state.tokens.expect(&Token::LeftParen)?;
                        let error = Box::new(super::parse_expr_recursive(state)?);
                        state.tokens.expect(&Token::RightParen)?;
                        
                        Ok(Some(Expr::new(ExprKind::Err { error }, span)))
                    }
                    _ => bail!("Expected Ok or Err after Result::")
                }
            } else {
                // Just Result as an identifier
                Ok(Some(Expr::new(ExprKind::Identifier("Result".to_string()), span)))
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
                        
                        Ok(Some(Expr::new(ExprKind::Some { value }, span)))
                    }
                    Some((Token::None, _)) => {
                        state.tokens.advance(); // consume None
                        Ok(Some(Expr::new(ExprKind::None, span)))
                    }
                    _ => bail!("Expected Some or None after Option::")
                }
            } else {
                // Just Option as an identifier
                Ok(Some(Expr::new(ExprKind::Identifier("Option".to_string()), span)))
            }
        }
        Token::Ok => {
            state.tokens.advance(); // consume Ok
            state.tokens.expect(&Token::LeftParen)?;
            let value = Box::new(super::parse_expr_recursive(state)?);
            state.tokens.expect(&Token::RightParen)?;
            Ok(Some(Expr::new(ExprKind::Ok { value }, span)))
        }
        Token::Err => {
            state.tokens.advance(); // consume Err
            state.tokens.expect(&Token::LeftParen)?;
            let error = Box::new(super::parse_expr_recursive(state)?);
            state.tokens.expect(&Token::RightParen)?;
            Ok(Some(Expr::new(ExprKind::Err { error }, span)))
        }
        Token::Some => {
            state.tokens.advance(); // consume Some first
            
            // Check for qualified path like Some::Value
            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                // We're in Some::Something case, delegate to qualified name parsing
                Ok(Some(parse_identifier_token(state, "Some".to_string(), span)?))
            } else {
                // Standard Some(value) constructor (already consumed Some above)
                state.tokens.expect(&Token::LeftParen)?;
                let value = Box::new(super::parse_expr_recursive(state)?);
                state.tokens.expect(&Token::RightParen)?;
                Ok(Some(Expr::new(ExprKind::Some { value }, span)))
            }
        }
        Token::None => {
            state.tokens.advance(); // consume None
            Ok(Some(Expr::new(ExprKind::None, span)))
        }
        Token::Throw => {
            state.tokens.advance(); // consume throw
            let expr = super::parse_expr_recursive(state)?;
            Ok(Some(Expr::new(
                ExprKind::Throw {
                    expr: Box::new(expr),
                },
                span,
            )))
        }
        Token::Await => {
            // Parse as prefix but it will transpile to postfix
            state.tokens.advance(); // consume await
            // Parse the full expression including postfix operations like calls
            let expr = super::parse_expr_recursive(state)?;
            Ok(Some(Expr::new(
                ExprKind::Await {
                    expr: Box::new(expr),
                },
                span,
            )))
        }
        _ => Ok(None), // Not a Result/Option token
    }
}

/// Parses unary operators and increment/decrement expressions
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("-42");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_unary_prefix(&mut state).unwrap();
/// // Should parse as negation of 42
/// ```
/// 
/// # Errors
/// Returns error if unary expression is malformed
fn parse_unary_prefix(state: &mut ParserState, token: Token, span: Span) -> Result<Option<Expr>> {
    match token {
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
            Ok(Some(Expr::new(
                ExprKind::Unary {
                    op,
                    operand: Box::new(operand),
                },
                span,
            )))
        }
        Token::Increment => {
            state.tokens.advance(); // consume ++
            let target = parse_prefix(state)?;
            Ok(Some(Expr::new(
                ExprKind::PreIncrement {
                    target: Box::new(target),
                },
                span,
            )))
        }
        Token::Decrement => {
            state.tokens.advance(); // consume --
            let target = parse_prefix(state)?;
            Ok(Some(Expr::new(
                ExprKind::PreDecrement {
                    target: Box::new(target),
                },
                span,
            )))
        }
        _ => Ok(None), // Not a unary operator
    }
}

/// Parses collection and type declaration tokens
/// 
/// # Examples
/// ```
/// use ruchy::frontend::lexer::Lexer;
/// use ruchy::frontend::parser::ParserState;
/// 
/// let mut lexer = Lexer::new("[1, 2, 3]");
/// let tokens = lexer.tokenize().unwrap();
/// let mut state = ParserState::new(tokens);
/// let expr = parse_collections_types_prefix(&mut state).unwrap();
/// // Should parse as list literal
/// ```
/// 
/// # Errors
/// Returns error if collection/type syntax is malformed
fn parse_collections_types_prefix(state: &mut ParserState, token: Token, _span: Span) -> Result<Option<Expr>> {
    match token {
        Token::LeftBrace => Ok(Some(collections::parse_block(state)?)),
        Token::LeftBracket => Ok(Some(collections::parse_list(state)?)),
        Token::Struct => Ok(Some(types::parse_struct(state)?)),
        Token::Enum => Ok(Some(types::parse_enum(state)?)),
        Token::Trait => Ok(Some(types::parse_trait(state)?)),
        Token::Impl => Ok(Some(types::parse_impl(state)?)),
        Token::Extend => Ok(Some(types::parse_extend(state)?)),
        Token::Actor => Ok(Some(actors::parse_actor(state)?)),
        Token::Import | Token::Use => Ok(Some(utils::parse_import(state)?)),
        Token::Module => Ok(Some(utils::parse_module(state)?)),
        Token::Export => Ok(Some(utils::parse_export(state)?)),
        Token::OrOr => Ok(Some(functions::parse_empty_lambda(state)?)),
        Token::DataFrame => Ok(Some(collections::parse_dataframe(state)?)),
        _ => Ok(None), // Not a collection/type token
    }
}

pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input");
    };

    let token_clone = token.clone();
    let span_clone = *span;

    // Try literal parsing first
    if let Some(expr) = parse_literal_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try string parsing
    if let Some(expr) = parse_string_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try identifier/macro parsing
    if let Some(expr) = parse_identifier_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try control flow/declaration parsing
    if let Some(expr) = parse_control_flow_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try Result/Option constructor parsing
    if let Some(expr) = parse_result_option_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try unary operator parsing
    if let Some(expr) = parse_unary_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Try collections/types parsing
    if let Some(expr) = parse_collections_types_prefix(state, token_clone.clone(), span_clone)? {
        return Ok(expr);
    }

    // Handle parentheses expressions specifically
    if matches!(token_clone, Token::LeftParen) {
        return parse_parentheses_prefix(state, span_clone);
    }

    // If we reach here, no helper function handled the token
    bail!("Unexpected token: {:?}", token_clone)
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
