//! Actor system parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, StmtKind, Pattern, Span};
use anyhow::bail;

/// Parse actor definition
pub fn parse_actor(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    let is_pub = if state.peek_matches(&Token::Pub) {
        state.advance();
        true
    } else {
        false
    };
    
    state.expect_token(Token::Actor)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected actor name");
    };
    
    // Parse actor state (optional)
    let state_fields = if state.peek_matches(&Token::LeftParen) {
        parse_actor_state(state)?
    } else {
        vec![]
    };
    
    state.expect_token(Token::LeftBrace)?;
    let handlers = parse_actor_handlers(state)?;
    state.expect_token(Token::RightBrace)?;
    
    create_actor_expr(name, state_fields, handlers, is_pub, span_start, state)
}

/// Parse actor state fields
fn parse_actor_state(state: &mut ParserState) -> Result<Vec<(String, Expr)>> {
    state.expect_token(Token::LeftParen)?;
    let mut fields = Vec::new();
    
    while !state.peek_matches(&Token::RightParen) {
        let (Token::Identifier(field_name), _) = state.next_token()? else {
            bail!("Expected field name in actor state");
        };
        
        state.expect_token(Token::Colon)?;
        let field_type = state.parse_expression()?;
        
        fields.push((field_name, field_type));
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightParen)?;
    Ok(fields)
}

/// Parse actor message handlers
fn parse_actor_handlers(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut handlers = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        if state.peek_matches(&Token::Receive) {
            handlers.push(parse_receive_handler(state)?);
        } else if state.peek_matches(&Token::On) {
            handlers.push(parse_on_handler(state)?);
        } else {
            // Regular method
            handlers.push(state.parse_expression()?);
        }
        
        // Skip optional semicolon
        if state.peek_matches(&Token::Semicolon) {
            state.advance();
        }
    }
    
    Ok(handlers)
}

/// Parse receive handler
fn parse_receive_handler(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Receive)?;
    
    state.expect_token(Token::LeftBrace)?;
    let cases = parse_message_cases(state)?;
    state.expect_token(Token::RightBrace)?;
    
    Ok(Expr {
        kind: ExprKind::Receive { cases },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse on handler (event-based)
fn parse_on_handler(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::On)?;
    
    let pattern = state.parse_pattern()?;
    state.expect_token(Token::FatArrow)?;
    let handler = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::OnHandler {
            pattern,
            handler,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse message cases for receive block
fn parse_message_cases(state: &mut ParserState) -> Result<Vec<(Pattern, Expr)>> {
    let mut cases = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        let pattern = state.parse_pattern()?;
        state.expect_token(Token::FatArrow)?;
        let handler = state.parse_expression()?;
        
        cases.push((pattern, handler));
        
        // Optional comma
        if state.peek_matches(&Token::Comma) {
            state.advance();
        }
    }
    
    Ok(cases)
}

/// Create actor expression
fn create_actor_expr(
    name: String,
    state_fields: Vec<(String, Expr)>,
    handlers: Vec<Expr>,
    is_pub: bool,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Actor {
            name,
            state: state_fields,
            handlers,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: if is_pub { vec!["pub".to_string()] } else { vec![] },
    })
}

/// Parse spawn expression
pub fn parse_spawn(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Spawn)?;
    
    // Parse actor constructor call or async block
    let expr = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Spawn { expr },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse send expression (!)
pub fn parse_send(state: &mut ParserState, actor: Expr) -> Result<Expr> {
    let span_start = actor.span;
    state.expect_token(Token::Bang)?;
    
    let message = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Send {
            actor: Box::new(actor),
            message,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse select expression (for multiple receives)
pub fn parse_select(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Select)?;
    
    state.expect_token(Token::LeftBrace)?;
    let branches = parse_select_branches(state)?;
    state.expect_token(Token::RightBrace)?;
    
    Ok(Expr {
        kind: ExprKind::Select { branches },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse select branches
fn parse_select_branches(state: &mut ParserState) -> Result<Vec<Expr>> {
    let mut branches = Vec::new();
    
    while !state.peek_matches(&Token::RightBrace) {
        if state.peek_matches(&Token::Receive) {
            branches.push(parse_select_receive(state)?);
        } else if state.peek_matches(&Token::Default) {
            branches.push(parse_select_default(state)?);
        } else {
            bail!("Expected 'receive' or 'default' in select block");
        }
        
        // Optional comma
        if state.peek_matches(&Token::Comma) {
            state.advance();
        }
    }
    
    Ok(branches)
}

/// Parse select receive branch
fn parse_select_receive(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Receive)?;
    
    let pattern = state.parse_pattern()?;
    
    // Optional from clause
    let from_actor = if state.peek_matches(&Token::From) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    state.expect_token(Token::FatArrow)?;
    let handler = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::SelectReceive {
            pattern,
            from_actor,
            handler,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse select default branch
fn parse_select_default(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Default)?;
    
    state.expect_token(Token::FatArrow)?;
    let handler = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::SelectDefault { handler },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse async expression
pub fn parse_async(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Async)?;
    
    // Check for async block
    if state.peek_matches(&Token::LeftBrace) {
        let block = Box::new(state.parse_block()?);
        Ok(Expr {
            kind: ExprKind::AsyncBlock { block },
            span: span_start.merge(state.current_span()),
            attributes: vec![],
        })
    } else {
        // Async function handled elsewhere
        bail!("Expected async block");
    }
}

/// Parse await expression
pub fn parse_await(state: &mut ParserState, expr: Expr) -> Result<Expr> {
    let span_start = expr.span;
    state.expect_token(Token::Dot)?;
    state.expect_token(Token::Await)?;
    
    Ok(Expr {
        kind: ExprKind::Await {
            future: Box::new(expr),
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse yield expression
pub fn parse_yield(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Yield)?;
    
    let value = if is_expression_start(state) {
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Expr {
        kind: ExprKind::Yield { value },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Check if next token starts an expression
fn is_expression_start(state: &ParserState) -> bool {
    if let Ok((token, _)) = state.peek_token() {
        !matches!(token,
            Token::Semicolon | Token::RightBrace | Token::RightParen |
            Token::Comma | Token::EOF
        )
    } else {
        false
    }
}