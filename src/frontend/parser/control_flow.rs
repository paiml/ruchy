//! Control flow parsing (if/else, match, loops, try/catch)

use super::{ParserState, *};
use crate::frontend::ast::{CatchClause, StructPatternField};

/// # Errors
///
/// Returns an error if the operation fails
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_if(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume if

    // Parse the condition
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
        false
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

        // Optional guard
        let guard = if matches!(state.tokens.peek(), Some((Token::If, _))) {
            state.tokens.advance(); // consume if
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

    // Parse variable name
    let var = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
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

    // Parse the condition
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
/// # Errors
///
/// Returns an error if the operation fails
pub fn parse_try_catch(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume try

    // Parse the try block
    let try_block = super::parse_expr_recursive(state)?;

    // Parse catch clauses
    let mut catch_clauses = Vec::new();

    while matches!(state.tokens.peek(), Some((Token::Catch, _))) {
        catch_clauses.push(parse_catch_clause(state)?);
    }

    // Parse optional finally block
    let finally_block = if matches!(state.tokens.peek(), Some((Token::Finally, _))) {
        state.tokens.advance(); // consume finally
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };

    // Must have at least one catch clause or a finally block
    if catch_clauses.is_empty() && finally_block.is_none() {
        bail!("Expected 'catch' or 'finally' after 'try'");
    }

    Ok(Expr::new(
        ExprKind::TryCatch {
            try_block: Box::new(try_block),
            catch_clauses,
            finally_block,
        },
        start_span,
    ))
}

/// Parse a single catch clause
fn parse_catch_clause(state: &mut ParserState) -> Result<CatchClause> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume catch

    // Parse catch signature: catch (ExceptionType variable) or catch variable
    let has_parens = matches!(state.tokens.peek(), Some((Token::LeftParen, _)));
    if has_parens {
        state.tokens.advance(); // consume (
    }

    // Parse exception type and variable name
    let (exception_type, variable) =
        if let Some((Token::Identifier(first_name), _)) = state.tokens.peek() {
            let first_name = first_name.clone();
            state.tokens.advance();

            // Check if there's a second identifier (TypeName variable pattern)
            if let Some((Token::Identifier(second_name), _)) = state.tokens.peek() {
                let second_name = second_name.clone();
                state.tokens.advance();
                (Some(first_name), second_name) // first is type, second is variable
            } else {
                (None, first_name) // first is variable, no type
            }
        } else {
            bail!("Expected variable name in catch clause");
        };

    if has_parens {
        state.tokens.expect(&Token::RightParen)?; // consume )
    }

    // Parse optional guard condition: if condition
    let condition = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance(); // consume if
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };

    // Parse catch body
    let body = Box::new(super::parse_expr_recursive(state)?);

    Ok(CatchClause {
        exception_type,
        variable,
        condition,
        body,
        span: start_span,
    })
}

/// # Errors
///
/// Returns an error if the operation fails
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
