//! Variable declaration parsing module  
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, Stmt, StmtKind, Pattern, Type, Span};
use anyhow::bail;

/// Parse let statement/expression
pub fn parse_let(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Let)?;
    
    let is_mut = parse_mutability(state)?;
    let pattern = state.parse_pattern()?;
    let type_annotation = parse_type_annotation(state)?;
    
    // Check for let-in expression
    if state.peek_matches(&Token::Equals) {
        state.advance();
        let value = Box::new(state.parse_expression()?);
        
        if state.peek_matches(&Token::In) {
            parse_let_in_expression(state, pattern, is_mut, type_annotation, value, span_start)
        } else {
            create_let_statement(pattern, is_mut, type_annotation, Some(value), span_start, state)
        }
    } else {
        create_let_statement(pattern, is_mut, type_annotation, None, span_start, state)
    }
}

/// Parse var statement
pub fn parse_var(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Var)?;
    
    let pattern = state.parse_pattern()?;
    let type_annotation = parse_type_annotation(state)?;
    
    let value = if state.peek_matches(&Token::Equals) {
        state.advance();
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    // var is always mutable
    create_let_statement(pattern, true, type_annotation, value, span_start, state)
}

/// Parse mutability modifier
fn parse_mutability(state: &mut ParserState) -> Result<bool> {
    if state.peek_matches(&Token::Mut) {
        state.advance();
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Parse type annotation (: Type)
fn parse_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if state.peek_matches(&Token::Colon) {
        state.advance();
        Ok(Some(state.parse_type()?))
    } else {
        Ok(None)
    }
}

/// Parse let-in expression
fn parse_let_in_expression(
    state: &mut ParserState,
    pattern: Pattern,
    is_mut: bool,
    type_annotation: Option<Type>,
    value: Box<Expr>,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::In)?;
    let body = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::LetIn {
            pattern,
            type_annotation,
            value,
            body,
            is_mut,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Create let statement as expression
fn create_let_statement(
    pattern: Pattern,
    is_mut: bool,
    type_annotation: Option<Type>,
    value: Option<Box<Expr>>,
    span_start: Span,
    state: &ParserState,
) -> Result<Expr> {
    let stmt = Stmt {
        kind: StmtKind::Let {
            pattern,
            type_annotation,
            value,
            is_mut,
        },
        span: span_start.merge(state.current_span()),
    };
    
    Ok(Expr {
        kind: ExprKind::Statement(Box::new(stmt)),
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse const declaration
pub fn parse_const(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Const)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected identifier after 'const'");
    };
    
    let type_annotation = if state.peek_matches(&Token::Colon) {
        state.advance();
        Some(state.parse_type()?)
    } else {
        None
    };
    
    state.expect_token(Token::Equals)?;
    let value = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Const {
            name,
            type_annotation,
            value,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse static declaration
pub fn parse_static(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Static)?;
    
    let is_mut = parse_mutability(state)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected identifier after 'static'");
    };
    
    let type_annotation = if state.peek_matches(&Token::Colon) {
        state.advance();
        Some(state.parse_type()?)
    } else {
        None
    };
    
    state.expect_token(Token::Equals)?;
    let value = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Static {
            name,
            type_annotation,
            value,
            is_mut,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse type alias
pub fn parse_type_alias(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    state.expect_token(Token::Type)?;
    
    let (Token::Identifier(name), _) = state.next_token()? else {
        bail!("Expected identifier after 'type'");
    };
    
    // Parse optional generic parameters
    let generics = if state.peek_matches(&Token::Lt) {
        Some(parse_generic_params(state)?)
    } else {
        None
    };
    
    state.expect_token(Token::Equals)?;
    let target_type = state.parse_type()?;
    
    Ok(Expr {
        kind: ExprKind::TypeAlias {
            name,
            generics,
            target_type,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse generic parameters
fn parse_generic_params(state: &mut ParserState) -> Result<Vec<String>> {
    state.expect_token(Token::Lt)?;
    
    let mut params = Vec::new();
    
    loop {
        let (Token::Identifier(param), _) = state.next_token()? else {
            bail!("Expected generic parameter name");
        };
        params.push(param);
        
        if !state.peek_matches(&Token::Comma) {
            break;
        }
        state.advance();
    }
    
    state.expect_token(Token::Gt)?;
    Ok(params)
}

/// Parse destructuring assignment
pub fn parse_destructuring_assignment(
    state: &mut ParserState,
    pattern: Pattern,
    span_start: Span,
) -> Result<Expr> {
    state.expect_token(Token::Equals)?;
    let value = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::DestructuringAssignment {
            pattern,
            value,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse assignment or augmented assignment
pub fn parse_assignment(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    let (token, _) = state.peek_token()?;
    
    match token {
        Token::Equals => {
            state.advance();
            let right = Box::new(state.parse_expression()?);
            Ok(Expr {
                kind: ExprKind::Assign {
                    target: Box::new(left),
                    value: right,
                },
                span: span_start.merge(state.current_span()),
                attributes: vec![],
            })
        }
        Token::PlusEquals | Token::MinusEquals | Token::StarEquals | 
        Token::SlashEquals | Token::PercentEquals => {
            parse_augmented_assignment(state, left, span_start)
        }
        _ => Ok(left),
    }
}

/// Parse augmented assignment (+=, -=, etc.)
fn parse_augmented_assignment(
    state: &mut ParserState,
    left: Expr,
    span_start: Span,
) -> Result<Expr> {
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::PlusEquals => crate::frontend::ast::BinaryOp::Add,
        Token::MinusEquals => crate::frontend::ast::BinaryOp::Sub,
        Token::StarEquals => crate::frontend::ast::BinaryOp::Mul,
        Token::SlashEquals => crate::frontend::ast::BinaryOp::Div,
        Token::PercentEquals => crate::frontend::ast::BinaryOp::Mod,
        _ => bail!("Expected augmented assignment operator"),
    };
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::AugAssign {
            target: Box::new(left),
            op,
            value: right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}