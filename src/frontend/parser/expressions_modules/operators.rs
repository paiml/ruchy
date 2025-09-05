//! Operator parsing module
//! Extracted from expressions.rs for modularity (complexity: ≤10 per function)

use crate::frontend::parser::{ParserState, Result, Token, Expr, ExprKind, BinaryOp, UnaryOp, Span};
use anyhow::bail;

/// Parse binary operator expression
pub fn parse_binary_op(
    state: &mut ParserState,
    left: Expr,
    op: BinaryOp,
) -> Result<Expr> {
    let span_start = left.span;
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse unary operator expression  
pub fn parse_unary_op(state: &mut ParserState) -> Result<Expr> {
    let span_start = state.current_span();
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::Plus => UnaryOp::Pos,
        Token::Minus => UnaryOp::Neg,
        Token::Not | Token::Bang => UnaryOp::Not,
        Token::Tilde => UnaryOp::BitNot,
        _ => bail!("Expected unary operator"),
    };
    
    let operand = Box::new(state.parse_prefix()?);
    
    Ok(Expr {
        kind: ExprKind::Unary { op, operand },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse comparison operator
pub fn parse_comparison(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::Eq => BinaryOp::Eq,
        Token::NotEq => BinaryOp::NotEq,
        Token::Lt => BinaryOp::Lt,
        Token::LtEq => BinaryOp::LtEq,
        Token::Gt => BinaryOp::Gt,
        Token::GtEq => BinaryOp::GtEq,
        _ => bail!("Expected comparison operator"),
    };
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse logical operator (and/or)
pub fn parse_logical(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::And | Token::AndAnd => BinaryOp::And,
        Token::Or | Token::OrOr => BinaryOp::Or,
        _ => bail!("Expected logical operator"),
    };
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse arithmetic operator
pub fn parse_arithmetic(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::Plus => BinaryOp::Add,
        Token::Minus => BinaryOp::Sub,
        Token::Star => BinaryOp::Mul,
        Token::Slash => BinaryOp::Div,
        Token::Percent => BinaryOp::Mod,
        Token::StarStar => BinaryOp::Pow,
        _ => bail!("Expected arithmetic operator"),
    };
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse bitwise operator
pub fn parse_bitwise(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    let (token, _) = state.next_token()?;
    
    let op = match token {
        Token::Ampersand => BinaryOp::BitAnd,
        Token::Pipe => BinaryOp::BitOr,
        Token::Caret => BinaryOp::BitXor,
        Token::LtLt => BinaryOp::Shl,
        Token::GtGt => BinaryOp::Shr,
        _ => bail!("Expected bitwise operator"),
    };
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse range operator (..)
pub fn parse_range(
    state: &mut ParserState,
    left: Option<Expr>,
) -> Result<Expr> {
    let span_start = left.as_ref()
        .map(|e| e.span)
        .unwrap_or_else(|| state.current_span());
    
    let inclusive = if state.peek_matches(&Token::DotDotEquals) {
        state.advance();
        true
    } else if state.peek_matches(&Token::DotDot) {
        state.advance();
        false
    } else {
        bail!("Expected range operator");
    };
    
    let end = if is_expression_start(state) {
        Some(Box::new(state.parse_expression()?))
    } else {
        None
    };
    
    Ok(Expr {
        kind: ExprKind::Range {
            start: left.map(Box::new),
            end,
            inclusive,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse pipeline operator (|>)
pub fn parse_pipeline(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    state.expect_token(Token::PipeGt)?;
    
    let right = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Pipeline {
            expr: Box::new(left),
            func: right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse in operator (for membership test)
pub fn parse_in_operator(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    state.expect_token(Token::In)?;
    
    let container = Box::new(state.parse_expression()?);
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::In,
            left: Box::new(left),
            right: container,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse is operator (for identity test)
pub fn parse_is_operator(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    state.expect_token(Token::Is)?;
    
    // Check for 'is not'
    let negate = if state.peek_matches(&Token::Not) {
        state.advance();
        true
    } else {
        false
    };
    
    let right = Box::new(state.parse_expression()?);
    
    let op = if negate {
        BinaryOp::IsNot
    } else {
        BinaryOp::Is
    };
    
    Ok(Expr {
        kind: ExprKind::Binary {
            op,
            left: Box::new(left),
            right,
        },
        span: span_start.merge(state.current_span()),
        attributes: vec![],
    })
}

/// Parse as operator (type cast)
pub fn parse_as_operator(
    state: &mut ParserState,
    left: Expr,
) -> Result<Expr> {
    let span_start = left.span;
    state.expect_token(Token::As)?;
    
    let target_type = state.parse_type()?;
    
    Ok(Expr {
        kind: ExprKind::Cast {
            expr: Box::new(left),
            target_type,
        },
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