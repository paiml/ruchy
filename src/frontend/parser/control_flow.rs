//! Control flow parsing (if/else, match, loops, try/catch)

use super::{ParserState, *};

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

    // Parse variable name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier after 'let'");
    };

    // Expect =
    state.tokens.expect(&Token::Equal)?;

    // Parse value
    let value = super::parse_expr_recursive(state)?;

    // Expect 'in'
    state.tokens.expect(&Token::In)?;

    // For now, let's parse the body as the rest of the expression
    // In a real implementation, we'd handle this more carefully
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Let {
            name,
            value: Box::new(value),
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

pub fn parse_pattern(state: &mut ParserState) -> Pattern {
    match state.tokens.peek() {
        Some((Token::Underscore, _)) => {
            state.tokens.advance();
            Pattern::Wildcard
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();

            // Check for Ok/Err patterns
            if name == "Ok" || name == "Err" {
                if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                    state.tokens.advance(); // consume (
                    let inner = parse_pattern(state);
                    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                        state.tokens.advance(); // consume )
                    }
                    if name == "Ok" {
                        return Pattern::Ok(Box::new(inner));
                    } else {
                        return Pattern::Err(Box::new(inner));
                    }
                }
            }

            Pattern::Identifier(name)
        }
        Some((Token::Integer(i), _)) => {
            let i = *i;
            state.tokens.advance();
            Pattern::Literal(Literal::Integer(i))
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

    // Expect catch keyword
    state.tokens.expect(&Token::Catch)?;

    // Parse catch variable (error binding), with optional parentheses
    let has_parens = matches!(state.tokens.peek(), Some((Token::LeftParen, _)));
    if has_parens {
        state.tokens.advance(); // consume (
    }

    let catch_var = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier after 'catch'");
    };

    if has_parens {
        state.tokens.expect(&Token::RightParen)?; // consume )
    }

    // Parse the catch block
    let catch_block = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::TryCatch {
            try_block: Box::new(try_block),
            catch_var,
            catch_block: Box::new(catch_block),
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
pub fn parse_async_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume async

    // Check what follows async
    let body = super::parse_expr_recursive(state)?;

    // For now, wrap the async block in a lambda that returns a future
    // In a full implementation, we'd have a dedicated AsyncBlock AST node
    Ok(Expr::new(
        ExprKind::Lambda {
            params: Vec::new(),
            body: Box::new(body),
        },
        start_span,
    ))
}

// Note: await is now handled as a postfix operator in parse_method_call
