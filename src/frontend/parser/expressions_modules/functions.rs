//! Function parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, StmtKind, Type, Param, Span};
use anyhow::bail;

/// Parse function definition
pub fn parse_function(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    // Parse visibility and modifiers
    let is_pub = parse_visibility(state)?;
    let is_async = parse_async_modifier(state)?;
    
    // Expect fn or fun keyword
    if !state.peek_matches(&Token::Fn) && !state.peek_matches(&Token::Fun) {
        bail!("Expected 'fn' or 'fun' keyword");
    }
    state.advance();
    
    // Parse function name
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected function name");
    };
    
    // Parse generic parameters
    let generics = parse_generics(state)?;
    
    // Parse parameters
    let params = parse_function_params(state)?;
    
    // Parse return type
    let return_type = parse_return_type(state)?;
    
    // Parse where clause
    let where_clause = parse_where_clause(state)?;
    
    // Parse body
    let body = parse_function_body(state)?;
    
    create_function_expr(
        name, params, return_type, body, generics, 
        is_pub, is_async, where_clause, span_start, state
    )
}

/// Parse visibility modifier
fn parse_visibility(state: &mut ParserState) -> Result<bool> {
    if state.peek_matches(&Token::Pub) {
        state.advance();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parse async modifier
fn parse_async_modifier(state: &mut ParserState) -> Result<bool> {
    if state.peek_matches(&Token::Async) {
        state.advance();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parse generic parameters
fn parse_generics(state: &mut ParserState) -> Result<Option<Vec<String>>> {
    if state.peek_matches(&Token::Lt) {
        state.advance();
        let mut params = Vec::new();
        
        loop {
            let (Token::Identifier(param), _) = state.next_token()? else {
                bail!("Expected generic parameter");
            };
            params.push(param);
            
            if !state.peek_matches(&Token::Comma) {
                break;
            }
            state.advance();
        }
        
        state.expect_token(Token::Gt)?;
        Ok(Some(params))
    } else {
        Ok(None)
    }
}

/// Parse function parameters
fn parse_function_params(state: &mut ParserState) -> Result<Vec<Param>> {
    state.expect_token(Token::LeftParen)?;
    
    let mut params = Vec::new();
    
    while !state.peek_matches(&Token::RightParen) {
        params.push(parse_single_param(state)?);
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::RightParen)?;
    Ok(params)
}

/// Parse single parameter
fn parse_single_param(state: &mut ParserState) -> Result<Param> {
    // Check for self parameter
    if state.peek_matches(&Token::SelfParam) {
        state.advance();
        return Ok(Param {
            pattern: crate::frontend::ast::Pattern::Identifier("self".to_string()),
            type_annotation: None,
            default: None,
        });
    }
    
    // Parse pattern
    let pattern = state.parse_pattern()?;
    
    // Parse type annotation
    let type_annotation = if state.peek_matches(&Token::Colon) {
        state.advance();
        Some(state.parse_type()?)
    } else {
        None
    };
    
    // Parse default value
    let default = if state.peek_matches(&Token::Equals) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Param {
        pattern,
        type_annotation,
        default,
    })
}

/// Parse return type annotation
fn parse_return_type(state: &mut ParserState) -> Result<Option<Type>> {
    if state.peek_matches(&Token::Arrow) {
        state.advance();
        Ok(Some(state.parse_type()?))
    } else {
        Ok(None)
    }
}

/// Parse where clause
fn parse_where_clause(state: &mut ParserState) -> Result<Option<Vec<String>>> {
    if state.peek_matches(&Token::Where) {
        state.advance();
        let mut clauses = Vec::new();
        
        // Simplified: just collect clause strings for now
        while !state.peek_matches(&Token::LeftBrace) {
            let clause = collect_where_clause(state)?;
            clauses.push(clause);
            
            if !state.peek_matches(&Token::Comma) {
                break;
            }
            state.advance();
        }
        
        Ok(Some(clauses))
    } else {
        Ok(None)
    }
}

/// Collect single where clause as string
fn collect_where_clause(state: &mut ParserState) -> Result<String> {
    // Simplified implementation - collect tokens until comma or {
    let mut clause = String::new();
    
    while !state.peek_matches(&Token::Comma) && !state.peek_matches(&Token::LeftBrace) {
        let (token, _) = state.next_token()?;
        clause.push_str(&format!("{:?} ", token));
    }
    
    Ok(clause)
}

/// Parse function body
fn parse_function_body(state: &mut ParserState) -> Result<Box<Expr>> {
    if state.peek_matches(&Token::LeftBrace) {
        Ok(Box::new(state.parse_block()?))
    } else if state.peek_matches(&Token::Equals) {
        state.advance();
        Ok(Box::new(state.parse_expression()?))
    } else {
        bail!("Expected function body (block or = expression)");
    }
}

/// Create function expression
fn create_function_expr(
    name: String,
    params: Vec<Param>,
    return_type: Option<Type>,
    body: Box<Expr>,
    generics: Option<Vec<String>>,
    is_pub: bool,
    is_async: bool,
    where_clause: Option<Vec<String>>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Function {
            name,
            params,
            return_type,
            body,
            generics,
            is_async,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: if is_pub { vec!["pub".to_string()] } else { vec![] },
    })
}

/// Parse lambda expression
pub fn parse_lambda(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    
    // Parse parameters
    let params = if state.peek_matches(&Token::Pipe) {
        parse_lambda_params_pipes(state)?
    } else if state.peek_matches(&Token::OrOr) {
        // Empty parameter list ||
        state.advance();
        vec![]
    } else {
        bail!("Expected lambda parameters");
    };
    
    // Parse arrow if present
    if state.peek_matches(&Token::Arrow) || state.peek_matches(&Token::FatArrow) {
        state.advance();
    }
    
    // Parse body
    let body = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Lambda {
            params: params.iter().map(|p| p.to_string()).collect(),
            body,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse lambda parameters between pipes
fn parse_lambda_params_pipes(state: &mut ParserState) -> Result<Vec<String>> {
    state.expect_token(Token::Pipe)?;
    
    let mut params = Vec::new();
    
    while !state.peek_matches(&Token::Pipe) {
        let (Token::Identifier(param), _) = state.next_token()? else {
            bail!("Expected parameter name in lambda");
        };
        params.push(param);
        
        if state.peek_matches(&Token::Comma) {
            state.advance();
        }
    }
    
    state.expect_token(Token::Pipe)?;
    Ok(params)
}

/// Parse closure expression
pub fn parse_closure(state: &mut ParserState) -> Result<Expr> {
    // Closures are parsed similarly to lambdas but with move semantics
    let span_start = state.current_span();
    
    let is_move = if state.peek_matches(&Token::Move) {
        state.advance();
        true
    } else {
        false
    };
    
    let lambda = parse_lambda(state)?;
    
    // Add move attribute if needed
    let mut expr = lambda;
    if is_move {
        expr.attributes.push("move".to_string());
    }
    
    Ok(expr)
}

/// Parse method definition (for impl blocks)
pub fn parse_method(state: &mut ParserState) -> Result<Expr> {
    // Methods are parsed like functions but in impl context
    parse_function(state)
}

/// Parse associated function
pub fn parse_associated_function(state: &mut ParserState) -> Result<Expr> {
    // Associated functions are like methods but without self
    parse_function(state)
}