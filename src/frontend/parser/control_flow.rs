//! Control flow parsing (if/else, match, loops, try/catch)

use super::{ParserState, *};
use crate::frontend::ast::StructPatternField;

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_if(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume if

    // Check if this is an if-let expression
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        state.tokens.advance(); // consume let
        
        // Parse pattern
        let pattern = parse_pattern(state);
        
        // Expect '=' 
        state.tokens.expect(&Token::Equal)?;
        
        // Parse expression to match against
        let expr = Box::new(super::parse_expr_recursive(state)?);
        
        // Parse the then branch
        let then_branch = Box::new(super::parse_expr_recursive(state)?);
        
        // Check for else branch
        let else_branch = if matches!(state.tokens.peek(), Some((Token::Else, _))) {
            state.tokens.advance(); // consume else
            Some(Box::new(super::parse_expr_recursive(state)?))
        } else {
            None
        };
        
        return Ok(Expr::new(
            ExprKind::IfLet {
                pattern,
                expr,
                then_branch,
                else_branch,
            },
            start_span,
        ));
    }

    // Parse the condition for regular if
    let condition = super::parse_expr_recursive(state)?;

    // Parse the then branch
    let then_branch = super::parse_expr_recursive(state)?;

    // Check for else branch
    let else_branch = if matches!(state.tokens.peek(), Some((Token::Else, _))) {
        state.tokens.advance(); // consume else
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };

    Ok(Expr::new(
        ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_let(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume let

    // Check for mut keyword
    let is_mutable = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume mut
        true
    } else {
        true  // Variables are mutable by default in Ruchy
    };

    // Parse variable name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier after 'let' or 'let mut'");
    };

    // Optional type annotation
    let type_annotation = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume :
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Expect =
    state.tokens.expect(&Token::Equal)?;

    // Parse value
    let value = super::parse_expr_recursive(state)?;

    // Check if 'in' keyword is present (optional for REPL-style let statements)
    let body = if matches!(state.tokens.peek(), Some((Token::In, _))) {
        state.tokens.advance(); // consume 'in'
                                // Parse the body expression after 'in'
        super::parse_expr_recursive(state)?
    } else {
        // REPL-style let statement without 'in' - create a unit body
        // This allows statements like "let x = 5" to work in REPL
        use crate::frontend::ast::{ExprKind, Literal, Span};
        Expr::new(ExprKind::Literal(Literal::Unit), Span { start: 0, end: 0 })
    };

    Ok(Expr::new(
        ExprKind::Let {
            name,
            type_annotation,
            value: Box::new(value),
            body: Box::new(body),
            is_mutable,
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_match(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume match

    let expr = super::parse_expr_recursive(state)?;

    state.tokens.expect(&Token::LeftBrace)?;

    let mut arms = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let pattern = parse_pattern(state);

        // Check for pattern guard: if condition
        let guard = if matches!(state.tokens.peek(), Some((Token::If, _))) {
            state.tokens.advance(); // consume 'if'
            Some(Box::new(super::parse_expr_recursive(state)?))
        } else {
            None
        };

        // Expect => or ->
        if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
            state.tokens.advance();
        } else {
            state.tokens.expect(&Token::FatArrow)?;
        }

        let body = super::parse_expr_recursive(state)?;
        let arm_span = body.span; // Simplified for now

        arms.push(MatchArm {
            pattern,
            guard,
            body: Box::new(body),
            span: arm_span,
        });

        // Optional comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Expr::new(
        ExprKind::Match {
            expr: Box::new(expr),
            arms,
        },
        start_span,
    ))
}

/// Parse a pattern with OR support (lowest precedence)
pub fn parse_pattern(state: &mut ParserState) -> Pattern {
    let mut left = parse_pattern_base(state);

    // Handle OR patterns (x | y | z)
    while matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        state.tokens.advance(); // consume |
        let right = parse_pattern_base(state);

        // Combine into OR pattern
        left = match left {
            Pattern::Or(mut patterns) => {
                patterns.push(right);
                Pattern::Or(patterns)
            }
            _ => Pattern::Or(vec![left, right]),
        };
    }

    left
}

/// Parse base patterns without OR handling
#[allow(clippy::cognitive_complexity)] // Pattern matching has inherent complexity from many variants
#[allow(clippy::too_many_lines)] // All pattern types must be handled in one place for consistency
pub fn parse_pattern_base(state: &mut ParserState) -> Pattern {
    match state.tokens.peek() {
        Some((Token::Underscore, _)) => {
            state.tokens.advance();
            Pattern::Wildcard
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();

            // Check for qualified name patterns like Ordering::Less
            let mut path = vec![name.clone()];
            while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                state.tokens.advance(); // consume ::
                if let Some((Token::Identifier(segment), _)) = state.tokens.peek() {
                    path.push(segment.clone());
                    state.tokens.advance();
                } else {
                    break;
                }
            }

            // If we have a qualified path, return it
            if path.len() > 1 {
                return Pattern::QualifiedName(path);
            }

            // Check for Ok/Err patterns
            if (name == "Ok" || name == "Err")
                && matches!(state.tokens.peek(), Some((Token::LeftParen, _)))
            {
                state.tokens.advance(); // consume (
                let inner = parse_pattern_base(state);
                if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                    state.tokens.advance(); // consume )
                }
                if name == "Ok" {
                    return Pattern::Ok(Box::new(inner));
                }
                return Pattern::Err(Box::new(inner));
            }

            // Check for struct patterns Point { x, y }
            if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                state.tokens.advance(); // consume {
                let mut fields = Vec::new();

                while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                    if let Some((Token::Identifier(field_name), _)) = state.tokens.peek() {
                        let field_name = field_name.clone();
                        state.tokens.advance();

                        let pattern = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                            state.tokens.advance(); // consume :
                            Some(parse_pattern_base(state))
                        } else {
                            None // Shorthand like { x } instead of { x: x }
                        };

                        fields.push(StructPatternField {
                            name: field_name,
                            pattern,
                        });

                        // Consume comma if present
                        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                            state.tokens.advance();
                        }
                    } else {
                        break;
                    }
                }

                if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                    state.tokens.advance(); // consume }
                }

                return Pattern::Struct { name, fields };
            }

            Pattern::Identifier(name)
        }
        Some((Token::LeftParen, _)) => {
            state.tokens.advance(); // consume (

            // Check for unit pattern ()
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                state.tokens.advance(); // consume )
                return Pattern::Literal(Literal::Unit);
            }

            // Parse tuple pattern (x, y, z)
            let mut patterns = Vec::new();

            patterns.push(parse_pattern_base(state));

            // Check if it's a single-element parenthesized pattern or a tuple
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance(); // consume comma

                // Parse remaining patterns
                while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                    patterns.push(parse_pattern_base(state));

                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    } else {
                        break;
                    }
                }
            }

            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                state.tokens.advance(); // consume )
            }

            // If only one pattern and no comma, it's not a tuple
            if patterns.len() == 1 {
                patterns
                    .into_iter()
                    .next()
                    .expect("checked: patterns.len() == 1")
            } else {
                Pattern::Tuple(patterns)
            }
        }
        Some((Token::LeftBracket, _)) => {
            state.tokens.advance(); // consume [
            let mut patterns = Vec::new();

            while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
                // Check for rest pattern ...
                if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
                    state.tokens.advance();
                    patterns.push(Pattern::Rest);
                } else {
                    patterns.push(parse_pattern_base(state));
                }

                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                } else {
                    break;
                }
            }

            if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
                state.tokens.advance(); // consume ]
            }

            Pattern::List(patterns)
        }
        Some((Token::Integer(i), _)) => {
            let i = *i;
            state.tokens.advance();

            // Check for range pattern 1..=10 or 1..10
            if matches!(
                state.tokens.peek(),
                Some((Token::DotDot | Token::DotDotEqual, _))
            ) {
                let inclusive = matches!(state.tokens.peek(), Some((Token::DotDotEqual, _)));
                state.tokens.advance(); // consume .. or ..=

                let end_pattern = parse_pattern_base(state);
                return Pattern::Range {
                    start: Box::new(Pattern::Literal(Literal::Integer(i))),
                    end: Box::new(end_pattern),
                    inclusive,
                };
            }

            Pattern::Literal(Literal::Integer(i))
        }
        Some((Token::Float(f), _)) => {
            let f = *f;
            state.tokens.advance();
            Pattern::Literal(Literal::Float(f))
        }
        Some((Token::String(s) | Token::RawString(s), _)) => {
            let s = s.clone();
            state.tokens.advance();
            Pattern::Literal(Literal::String(s))
        }
        Some((Token::Bool(b), _)) => {
            let b = *b;
            state.tokens.advance();
            Pattern::Literal(Literal::Bool(b))
        }
        Some((Token::DotDotDot, _)) => {
            state.tokens.advance();
            Pattern::Rest
        }
        Some((Token::Some, _)) => {
            state.tokens.advance(); // consume Some
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                state.tokens.advance(); // consume (
                let inner = parse_pattern_base(state);
                if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                    state.tokens.advance(); // consume )
                }
                Pattern::Some(Box::new(inner))
            } else {
                // Some without parentheses is just an identifier
                Pattern::Identifier("Some".to_string())
            }
        }
        Some((Token::None, _)) => {
            state.tokens.advance(); // consume None
            Pattern::None
        }
        Some((Token::Ok, _)) => {
            state.tokens.advance(); // consume Ok
            
            // Check for Ok(pattern)
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                state.tokens.advance(); // consume (
                let inner = parse_pattern_base(state);
                if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                    state.tokens.advance(); // consume )
                }
                Pattern::Ok(Box::new(inner))
            } else {
                // Just Ok without parentheses - treat as constructor pattern
                Pattern::Ok(Box::new(Pattern::Wildcard))
            }
        }
        Some((Token::Err, _)) => {
            state.tokens.advance(); // consume Err
            
            // Check for Err(pattern)
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                state.tokens.advance(); // consume (
                let inner = parse_pattern_base(state);
                if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                    state.tokens.advance(); // consume )
                }
                Pattern::Err(Box::new(inner))
            } else {
                // Just Err without parentheses - treat as constructor pattern
                Pattern::Err(Box::new(Pattern::Wildcard))
            }
        }
        _ => Pattern::Wildcard, // Default fallback
    }
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_for(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume for

    // Check if we have a tuple pattern (multiple identifiers separated by commas)
    let mut identifiers = Vec::new();
    let mut pattern = None;
    
    // Parse first identifier
    let var = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        identifiers.push(name);
        state.tokens.advance();
        
        // Check for comma (indicating tuple pattern)
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma
            
            if let Some((Token::Identifier(next_name), _)) = state.tokens.peek() {
                identifiers.push(next_name.clone());
                state.tokens.advance();
            } else {
                bail!("Expected identifier after comma in for loop pattern");
            }
        }
        
        // If we have multiple identifiers, create a tuple pattern
        if identifiers.len() > 1 {
            pattern = Some(Pattern::Tuple(
                identifiers.iter().map(|id| Pattern::Identifier(id.clone())).collect()
            ));
        }
        
        identifiers[0].clone() // Return first identifier as var for compatibility
    } else {
        bail!("Expected identifier after 'for'");
    };

    // Expect 'in'
    state.tokens.expect(&Token::In)?;

    // Parse iterable expression
    let iter = super::parse_expr_recursive(state)?;

    // Parse the body block
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::For {
            var,
            pattern,
            iter: Box::new(iter),
            body: Box::new(body),
        },
        start_span,
    ))
}

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_while(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume while

    // Check if this is a while-let expression
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        state.tokens.advance(); // consume let
        
        // Parse pattern
        let pattern = parse_pattern(state);
        
        // Expect '=' 
        state.tokens.expect(&Token::Equal)?;
        
        // Parse expression to match against
        let expr = Box::new(super::parse_expr_recursive(state)?);
        
        // Parse the body
        let body = Box::new(super::parse_expr_recursive(state)?);
        
        return Ok(Expr::new(
            ExprKind::WhileLet {
                pattern,
                expr,
                body,
            },
            start_span,
        ));
    }

    // Parse the condition for regular while
    let condition = super::parse_expr_recursive(state)?;

    // Parse the body block
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
        },
        start_span,
    ))
}

pub fn parse_loop(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume loop

    // Parse the body block
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Loop {
            body: Box::new(body),
        },
        start_span,
    ))
}

pub fn parse_break(state: &mut ParserState) -> Expr {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume break

    // Check for optional label
    let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Some(name)
    } else {
        None
    };

    Expr::new(ExprKind::Break { label }, start_span)
}

pub fn parse_continue(state: &mut ParserState) -> Expr {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume continue

    // Check for optional label
    let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Some(name)
    } else {
        None
    };

    Expr::new(ExprKind::Continue { label }, start_span)
}

/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_async_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume async

    // Parse the async block body
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::AsyncBlock {
            body: Box::new(body),
        },
        start_span,
    ))
}

// Note: await is now handled as a postfix operator in parse_method_call

/// Parse return statement
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_return(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume return
    
    // Check if there's a value to return
    let value = if matches!(state.tokens.peek(), Some((Token::RightBrace | Token::Semicolon, _))) 
        || state.tokens.peek().is_none() {
        // No value, return unit
        None
    } else {
        // Parse the return value
        Some(Box::new(super::parse_expr_recursive(state)?))
    };
    
    Ok(Expr::new(
        ExprKind::Return { value },
        start_span,
    ))
}
