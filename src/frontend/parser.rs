use crate::frontend::ast::*;
use crate::frontend::lexer::{Token, TokenStream};
use anyhow::{Result, bail};

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
        }
    }
    
    pub fn parse(&mut self) -> Result<Expr> {
        self.parse_expr()
    }
    
    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_expr_with_precedence(0)
    }
    
    fn parse_expr_with_precedence(&mut self, min_prec: i32) -> Result<Expr> {
        let mut left = self.parse_prefix()?;
        
        loop {
            let Some((token, _)) = self.tokens.peek() else {
                break;
            };
            
            if !token.is_binary_op() {
                break;
            }
            
            let token_clone = token.clone();
            let prec = self.precedence(&token_clone);
            if prec < min_prec {
                break;
            }
            
            let (op_token, _op_span) = self.tokens.next().unwrap();
            let op = self.token_to_binary_op(&op_token)?;
            
            let right = if self.is_right_associative(&op) {
                self.parse_expr_with_precedence(prec)?
            } else {
                self.parse_expr_with_precedence(prec + 1)?
            };
            
            let span = left.span.merge(right.span);
            left = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                span,
            );
        }
        
        // Check for pipeline operator
        if let Some((Token::Pipeline, _)) = self.tokens.peek() {
            left = self.parse_pipeline(left)?;
        }
        
        Ok(left)
    }
    
    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.tokens.peek() {
            Some((Token::Integer(n), span)) => {
                let n = *n;
                let span = *span;
                self.tokens.next();
                Ok(Expr::new(ExprKind::Literal(Literal::Integer(n)), span))
            }
            Some((Token::Float(f), span)) => {
                let f = *f;
                let span = *span;
                self.tokens.next();
                Ok(Expr::new(ExprKind::Literal(Literal::Float(f)), span))
            }
            Some((Token::String(s), span)) => {
                let s = s.clone();
                let span = *span;
                self.tokens.next();
                Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span))
            }
            Some((Token::Bool(b), span)) => {
                let b = *b;
                let span = *span;
                self.tokens.next();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(b)), span))
            }
            Some((Token::Identifier(name), span)) => {
                let name = name.clone();
                let span = *span;
                self.tokens.next();
                
                // Check for function call
                if let Some((Token::LeftParen, _)) = self.tokens.peek() {
                    self.parse_call(Expr::new(ExprKind::Identifier(name), span))
                } else {
                    Ok(Expr::new(ExprKind::Identifier(name), span))
                }
            }
            Some((Token::If, _)) => self.parse_if(),
            Some((Token::Let, _)) => self.parse_let(),
            Some((Token::Fun, _)) => self.parse_function(),
            Some((Token::Match, _)) => self.parse_match(),
            Some((Token::LeftBracket, _)) => self.parse_list(),
            Some((Token::LeftParen, _span)) => {
                self.tokens.next();
                let expr = self.parse_expr()?;
                self.tokens.expect(Token::RightParen)?;
                Ok(expr)
            }
            Some((token, span)) if token.is_unary_op() => {
                let token_clone = token.clone();
                let span = *span;
                self.tokens.next();
                let op = self.token_to_unary_op(&token_clone)?;
                let operand = self.parse_prefix()?;
                let full_span = span.merge(operand.span);
                Ok(Expr::new(
                    ExprKind::Unary {
                        op,
                        operand: Box::new(operand),
                    },
                    full_span,
                ))
            }
            Some((token, _)) => bail!("Unexpected token: {:?}", token),
            None => bail!("Unexpected end of input"),
        }
    }
    
    fn parse_if(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::If)?;
        let condition = Box::new(self.parse_expr()?);
        self.tokens.expect(Token::LeftBrace)?;
        let then_branch = Box::new(self.parse_block()?);
        
        let else_branch = if let Some((Token::Else, _)) = self.tokens.peek() {
            self.tokens.next();
            self.tokens.expect(Token::LeftBrace)?;
            Some(Box::new(self.parse_block()?))
        } else {
            None
        };
        
        let span = if let Some(ref else_br) = else_branch {
            start_span.merge(else_br.span)
        } else {
            start_span.merge(then_branch.span)
        };
        
        Ok(Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            span,
        ))
    }
    
    fn parse_let(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Let)?;
        
        let name = match self.tokens.next() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected identifier after 'let'"),
        };
        
        self.tokens.expect(Token::Equal)?;
        let value = Box::new(self.parse_expr()?);
        
        // For now, let's parse the body as the rest of the expression
        // In a real implementation, we'd handle this more carefully
        let body = if let Some((Token::In, _)) = self.tokens.peek() {
            self.tokens.next();
            Box::new(self.parse_expr()?)
        } else {
            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
        };
        
        let span = start_span.merge(body.span);
        Ok(Expr::new(
            ExprKind::Let { name, value, body },
            span,
        ))
    }
    
    fn parse_function(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Fun)?;
        
        let name = match self.tokens.next() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected function name"),
        };
        
        self.tokens.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.tokens.expect(Token::RightParen)?;
        
        let return_type = if let Some((Token::Arrow, _)) = self.tokens.peek() {
            self.tokens.next();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.tokens.expect(Token::LeftBrace)?;
        let body = Box::new(self.parse_block()?);
        
        let span = start_span.merge(body.span);
        Ok(Expr::new(
            ExprKind::Function {
                name,
                params,
                return_type,
                body,
            },
            span,
        ))
    }
    
    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        
        if let Some((Token::RightParen, _)) = self.tokens.peek() {
            return Ok(params);
        }
        
        loop {
            let (name, name_span) = match self.tokens.next() {
                Some((Token::Identifier(name), span)) => (name, span),
                _ => bail!("Expected parameter name"),
            };
            
            self.tokens.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            
            params.push(Param {
                name,
                ty,
                span: name_span,
            });
            
            match self.tokens.peek() {
                Some((Token::Comma, _)) => {
                    self.tokens.next();
                }
                Some((Token::RightParen, _)) => break,
                _ => bail!("Expected ',' or ')' in parameter list"),
            }
        }
        
        Ok(params)
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        let (base_type, span) = match self.tokens.next() {
            Some((Token::Identifier(name), span)) => {
                (TypeKind::Named(name), span)
            }
            _ => bail!("Expected type"),
        };
        
        // Check for optional type
        let kind = if let Some((Token::Question, _)) = self.tokens.peek() {
            self.tokens.next();
            TypeKind::Optional(Box::new(Type {
                kind: base_type,
                span,
            }))
        } else {
            base_type
        };
        
        Ok(Type { kind, span })
    }
    
    fn parse_block(&mut self) -> Result<Expr> {
        let mut exprs = Vec::new();
        let start_span = self.tokens.peek()
            .map(|(_, span)| *span)
            .unwrap_or(Span::new(0, 0));
        
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _)) | None) {
            exprs.push(self.parse_expr()?);
            
            // Optional semicolon
            if let Some((Token::Semicolon, _)) = self.tokens.peek() {
                self.tokens.next();
            }
        }
        
        self.tokens.expect(Token::RightBrace)?;
        
        let span = if let Some(last) = exprs.last() {
            start_span.merge(last.span)
        } else {
            start_span
        };
        
        Ok(Expr::new(ExprKind::Block(exprs), span))
    }
    
    fn parse_list(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::LeftBracket)?;
        let mut elements = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
            elements.push(self.parse_expr()?);
            
            if let Some((Token::Comma, _)) = self.tokens.peek() {
                self.tokens.next();
            } else {
                break;
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBracket)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(ExprKind::List(elements), span))
    }
    
    fn parse_call(&mut self, func: Expr) -> Result<Expr> {
        self.tokens.expect(Token::LeftParen)?;
        let mut args = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightParen, _))) {
            args.push(self.parse_expr()?);
            
            if let Some((Token::Comma, _)) = self.tokens.peek() {
                self.tokens.next();
            } else {
                break;
            }
        }
        
        let end_span = self.tokens.expect(Token::RightParen)?;
        let span = func.span.merge(end_span);
        
        Ok(Expr::new(
            ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span,
        ))
    }
    
    fn parse_pipeline(&mut self, expr: Expr) -> Result<Expr> {
        let mut stages = Vec::new();
        
        while let Some((Token::Pipeline, _)) = self.tokens.peek() {
            let pipe_span = self.tokens.next().unwrap().1;
            let stage_expr = self.parse_prefix()?;
            let stage_span = pipe_span.merge(stage_expr.span);
            
            stages.push(PipelineStage {
                op: Box::new(stage_expr),
                span: stage_span,
            });
        }
        
        let span = expr.span.merge(stages.last().unwrap().span);
        Ok(Expr::new(
            ExprKind::Pipeline {
                expr: Box::new(expr),
                stages,
            },
            span,
        ))
    }
    
    fn parse_match(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Match)?;
        let expr = Box::new(self.parse_expr()?);
        self.tokens.expect(Token::LeftBrace)?;
        
        let mut arms = Vec::new();
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            let pattern = self.parse_pattern()?;
            
            let guard = if let Some((Token::If, _)) = self.tokens.peek() {
                self.tokens.next();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            
            self.tokens.expect(Token::FatArrow)?;
            let body = Box::new(self.parse_expr()?);
            
            let arm_span = body.span; // Simplified for now
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_span,
            });
            
            if let Some((Token::Comma, _)) = self.tokens.peek() {
                self.tokens.next();
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(ExprKind::Match { expr, arms }, span))
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.tokens.peek() {
            Some((Token::Underscore, _)) => {
                self.tokens.next();
                Ok(Pattern::Wildcard)
            }
            Some((Token::Integer(n), _)) => {
                let n = *n;
                self.tokens.next();
                Ok(Pattern::Literal(Literal::Integer(n)))
            }
            Some((Token::Identifier(name), _)) => {
                let name = name.clone();
                self.tokens.next();
                Ok(Pattern::Identifier(name))
            }
            _ => bail!("Expected pattern"),
        }
    }
    
    fn precedence(&self, token: &Token) -> i32 {
        match token {
            Token::OrOr => 1,
            Token::AndAnd => 2,
            Token::Pipe => 3,
            Token::Caret => 4,
            Token::Ampersand => 5,
            Token::EqualEqual | Token::NotEqual => 6,
            Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual => 7,
            Token::LeftShift | Token::RightShift => 8,
            Token::Plus | Token::Minus => 9,
            Token::Star | Token::Slash | Token::Percent => 10,
            Token::Power => 11,
            _ => 0,
        }
    }
    
    fn is_right_associative(&self, op: &BinaryOp) -> bool {
        matches!(op, BinaryOp::Power)
    }
    
    fn token_to_binary_op(&self, token: &Token) -> Result<BinaryOp> {
        Ok(match token {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Subtract,
            Token::Star => BinaryOp::Multiply,
            Token::Slash => BinaryOp::Divide,
            Token::Percent => BinaryOp::Modulo,
            Token::Power => BinaryOp::Power,
            Token::EqualEqual => BinaryOp::Equal,
            Token::NotEqual => BinaryOp::NotEqual,
            Token::Less => BinaryOp::Less,
            Token::LessEqual => BinaryOp::LessEqual,
            Token::Greater => BinaryOp::Greater,
            Token::GreaterEqual => BinaryOp::GreaterEqual,
            Token::AndAnd => BinaryOp::And,
            Token::OrOr => BinaryOp::Or,
            Token::Ampersand => BinaryOp::BitwiseAnd,
            Token::Pipe => BinaryOp::BitwiseOr,
            Token::Caret => BinaryOp::BitwiseXor,
            Token::LeftShift => BinaryOp::LeftShift,
            Token::RightShift => BinaryOp::RightShift,
            _ => bail!("Not a binary operator: {:?}", token),
        })
    }
    
    fn token_to_unary_op(&self, token: &Token) -> Result<UnaryOp> {
        Ok(match token {
            Token::Bang => UnaryOp::Not,
            Token::Minus => UnaryOp::Negate,
            Token::Tilde => UnaryOp::BitwiseNot,
            _ => bail!("Not a unary operator: {:?}", token),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_literals() {
        let mut parser = Parser::new("42");
        let expr = parser.parse().unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal"),
        }
        
        let mut parser = Parser::new("3.14");
        let expr = parser.parse().unwrap();
        match expr.kind {
            ExprKind::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float literal"),
        }
    }
    
    #[test]
    fn test_parse_binary_ops() {
        let mut parser = Parser::new("1 + 2 * 3");
        let expr = parser.parse().unwrap();
        // Should parse as 1 + (2 * 3) due to precedence
        match expr.kind {
            ExprKind::Binary { op: BinaryOp::Add, .. } => {}
            _ => panic!("Expected addition at top level"),
        }
    }
    
    #[test]
    fn test_parse_function() {
        let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
        let expr = parser.parse().unwrap();
        match expr.kind {
            ExprKind::Function { name, params, .. } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
            }
            _ => panic!("Expected function"),
        }
    }
    
    #[test]
    fn test_parse_pipeline() {
        let mut parser = Parser::new("[1, 2, 3] |> map(double) |> filter(even)");
        let expr = parser.parse().unwrap();
        match expr.kind {
            ExprKind::Pipeline { stages, .. } => {
                assert_eq!(stages.len(), 2);
            }
            _ => panic!("Expected pipeline"),
        }
    }
}