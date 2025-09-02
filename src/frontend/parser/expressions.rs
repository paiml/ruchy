//! Basic expression parsing - minimal version with only used functions

use super::{ParserState, *};

pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input");
    };

    let token_clone = token.clone();
    let span_clone = *span;

    match token_clone {
        Token::Integer(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Integer(value)), span_clone))
        }
        Token::Float(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Float(value)), span_clone))
        }
        Token::String(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::String(value)), span_clone))
        }
        Token::Char(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Char(value)), span_clone))
        }
        Token::Bool(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Bool(value)), span_clone))
        }
        Token::Identifier(name) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier(name), span_clone))
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("_".to_string()), span_clone))
        }
        Token::LeftParen => {
            state.tokens.advance();
            let expr = super::parse_expr_recursive(state)?;
            state.tokens.expect(&Token::RightParen)?;
            Ok(expr)
        }
        Token::Minus => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?; // High precedence for unary
            Ok(Expr::new(ExprKind::Unary { 
                op: UnaryOp::Negate, 
                operand: Box::new(expr) 
            }, span_clone))
        }
        Token::Bang => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?;
            Ok(Expr::new(ExprKind::Unary { 
                op: UnaryOp::Not, 
                operand: Box::new(expr) 
            }, span_clone))
        }
        Token::Fun | Token::Fn => {
            // Parse function definition - do NOT advance token, let function parser handle it
            super::functions::parse_function(state)
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
        Token::NullCoalesce => Some(BinaryOp::NullCoalesce),
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
        BinaryOp::NullCoalesce => 2,
        BinaryOp::And => 3,
        BinaryOp::BitwiseOr => 4,
        BinaryOp::BitwiseXor => 5,
        BinaryOp::BitwiseAnd => 6,
        BinaryOp::Equal | BinaryOp::NotEqual => 7,
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => 8,
        BinaryOp::LeftShift => 9,
        BinaryOp::Add | BinaryOp::Subtract => 10,
        BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => 11,
        BinaryOp::Power => 12,
    }
}