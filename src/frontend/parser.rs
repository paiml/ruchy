//! Parser for converting tokens into AST

#![allow(clippy::expect_used)] // Parser needs expect for checked conditions

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, MatchArm, Param, Pattern, PipelineStage, Span, Type, TypeKind, UnaryOp};
use crate::frontend::lexer::{Token, TokenStream};
use anyhow::{bail, Result};

pub struct Parser<'a> {
    tokens: TokenStream<'a>,
}

impl<'a> Parser<'a> {
    #[must_use] pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
        }
    }

    /// Parse the input into an expression or block of expressions
    /// 
    /// # Panics
    /// 
    /// Should not panic in normal operation. Uses `expect` on verified conditions.
    pub fn parse(&mut self) -> Result<Expr> {
        // Parse multiple top-level expressions/statements as a block
        let mut exprs = Vec::new();
        
        while self.tokens.peek().is_some() {
            exprs.push(self.parse_expr()?);
            
            // Skip optional semicolons
            if let Some((Token::Semicolon, _)) = self.tokens.peek() {
                self.tokens.advance();
            }
        }
        
        if exprs.is_empty() {
            bail!("Empty program");
        } else if exprs.len() == 1 {
            Ok(exprs.into_iter().next().expect("checked: non-empty vec"))
        } else {
            Ok(Expr {
                kind: ExprKind::Block(exprs),
                span: Span { start: 0, end: 0 }, // Simplified span for now
            })
        }
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
            let prec = Self::precedence(&token_clone);
            if prec < min_prec {
                break;
            }

            let (op_token, _op_span) = self.tokens.advance().expect("Token disappeared after peek");
            let op = Self::token_to_binary_op(&op_token)?;

            let right = if Self::is_right_associative(op) {
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

        // Check for range operator
        if let Some((Token::DotDot | Token::DotDotEqual, _)) = self.tokens.peek() {
            left = self.parse_range_from(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr> {
        match self.tokens.peek() {
            Some((Token::Integer(n), span)) => {
                let n = *n;
                let span = *span;
                self.tokens.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Integer(n)), span))
            }
            Some((Token::Float(f), span)) => {
                let f = *f;
                let span = *span;
                self.tokens.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Float(f)), span))
            }
            Some((Token::String(s), span)) => {
                let s = s.clone();
                let span = *span;
                self.tokens.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span))
            }
            Some((Token::Bool(b), span)) => {
                let b = *b;
                let span = *span;
                self.tokens.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Bool(b)), span))
            }
            Some((Token::Identifier(name), span)) => {
                let name = name.clone();
                let span = *span;
                self.tokens.advance();

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
            Some((Token::For, _)) => self.parse_for(),
            Some((Token::Import | Token::Use, _)) => self.parse_import(),
            Some((Token::LeftBracket, _)) => self.parse_list(),
            Some((Token::LeftParen, _span)) => {
                self.tokens.advance();
                let expr = self.parse_expr()?;
                self.tokens.expect(Token::RightParen)?;
                Ok(expr)
            }
            Some((token, span)) if token.is_unary_op() => {
                let token_clone = token.clone();
                let span = *span;
                self.tokens.advance();
                let op = Self::token_to_unary_op(&token_clone)?;
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
            self.tokens.advance();
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

        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected identifier after 'let'"),
        };

        self.tokens.expect(Token::Equal)?;
        let value = Box::new(self.parse_expr()?);

        // For now, let's parse the body as the rest of the expression
        // In a real implementation, we'd handle this more carefully
        let body = if let Some((Token::In, _)) = self.tokens.peek() {
            self.tokens.advance();
            Box::new(self.parse_expr()?)
        } else {
            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
        };

        let span = start_span.merge(body.span);
        Ok(Expr::new(ExprKind::Let { name, value, body }, span))
    }

    fn parse_function(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Fun)?;

        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected function name"),
        };

        self.tokens.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.tokens.expect(Token::RightParen)?;

        let return_type = if let Some((Token::Arrow, _)) = self.tokens.peek() {
            self.tokens.advance();
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
            let (name, name_span) = match self.tokens.advance() {
                Some((Token::Identifier(name), span)) => (name, span),
                _ => bail!("Expected parameter name"),
            };

            // Type annotation is optional for gradual typing
            let ty = if let Some((Token::Colon, _)) = self.tokens.peek() {
                self.tokens.advance();
                self.parse_type()?
            } else {
                // Default to 'Any' type for untyped parameters
                Type {
                    kind: TypeKind::Named("Any".to_string()),
                    span: name_span,
                }
            };

            params.push(Param {
                name,
                ty,
                span: name_span,
            });

            match self.tokens.peek() {
                Some((Token::Comma, _)) => {
                    self.tokens.advance();
                }
                Some((Token::RightParen, _)) => break,
                _ => bail!("Expected ',' or ')' in parameter list"),
            }
        }

        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type> {
        let (mut base_type, span) = match self.tokens.peek() {
            Some((Token::LeftBracket, _)) => {
                // List type: [T]
                let start_span = self.tokens.advance().expect("checked: peeked token exists").1;
                let inner = self.parse_type()?;
                self.tokens.expect(Token::RightBracket)?;
                (TypeKind::List(Box::new(inner)), start_span)
            }
            Some((Token::LeftParen, _)) => {
                // Function type: (T1, T2) -> T3
                let start = self.tokens.advance().expect("checked: peeked token exists").1;
                let mut params = Vec::new();
                
                while !matches!(self.tokens.peek(), Some((Token::RightParen, _))) {
                    params.push(self.parse_type()?);
                    if let Some((Token::Comma, _)) = self.tokens.peek() {
                        self.tokens.advance();
                    }
                }
                self.tokens.expect(Token::RightParen)?;
                self.tokens.expect(Token::Arrow)?;
                let ret = Box::new(self.parse_type()?);
                
                (TypeKind::Function { params, ret }, start)
            }
            Some((Token::Identifier(name), span)) => {
                let name = name.clone();
                let span = *span;
                self.tokens.advance();
                
                // Check for generic types: Vec<T>, Result<T, E>
                if let Some((Token::Less, _)) = self.tokens.peek() {
                    self.tokens.advance();
                    let mut type_args = vec![self.parse_type()?];
                    
                    while let Some((Token::Comma, _)) = self.tokens.peek() {
                        self.tokens.advance();
                        type_args.push(self.parse_type()?);
                    }
                    
                    self.tokens.expect(Token::Greater)?;
                    
                    // For now, represent generics as Named with special formatting
                    let generic_name = if type_args.len() == 1 {
                        format!("{name}<{:?}>", type_args[0])
                    } else {
                        format!("{name}<{type_args:?}>")
                    };
                    (TypeKind::Named(generic_name), span)
                } else {
                    (TypeKind::Named(name), span)
                }
            }
            _ => bail!("Expected type"),
        };

        // Check for optional type suffix
        if let Some((Token::Question, _)) = self.tokens.peek() {
            self.tokens.advance();
            base_type = TypeKind::Optional(Box::new(Type {
                kind: base_type,
                span,
            }));
        }

        Ok(Type { kind: base_type, span })
    }

    fn parse_block(&mut self) -> Result<Expr> {
        let mut exprs = Vec::new();
        let start_span = self
            .tokens
            .peek()
            .map_or(Span::new(0, 0), |(_, span)| *span);

        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _)) | None) {
            exprs.push(self.parse_expr()?);

            // Optional semicolon
            if let Some((Token::Semicolon, _)) = self.tokens.peek() {
                self.tokens.advance();
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
                self.tokens.advance();
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
                self.tokens.advance();
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
            let pipe_span = self.tokens.advance().expect("checked by parser logic").1;
            let stage_expr = self.parse_prefix()?;
            let stage_span = pipe_span.merge(stage_expr.span);

            stages.push(PipelineStage {
                op: Box::new(stage_expr),
                span: stage_span,
            });
        }

        let span = expr.span.merge(stages.last().expect("checked by parser logic").span);
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
                self.tokens.advance();
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
                self.tokens.advance();
            }
        }

        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);

        Ok(Expr::new(ExprKind::Match { expr, arms }, span))
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.tokens.peek() {
            Some((Token::Underscore, _)) => {
                self.tokens.advance();
                Ok(Pattern::Wildcard)
            }
            Some((Token::Integer(n), _)) => {
                let n = *n;
                self.tokens.advance();
                Ok(Pattern::Literal(Literal::Integer(n)))
            }
            Some((Token::String(s), _)) => {
                let s = s.clone();
                self.tokens.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            Some((Token::Bool(b), _)) => {
                let b = *b;
                self.tokens.advance();
                Ok(Pattern::Literal(Literal::Bool(b)))
            }
            Some((Token::Identifier(name), _)) => {
                let name = name.clone();
                self.tokens.advance();
                Ok(Pattern::Identifier(name))
            }
            _ => bail!("Expected pattern"),
        }
    }

    fn parse_for(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::For)?;

        let var = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected variable name after 'for'"),
        };

        self.tokens.expect(Token::In)?;
        let iter = Box::new(self.parse_expr()?);

        self.tokens.expect(Token::LeftBrace)?;
        let body = Box::new(self.parse_block()?);

        let span = start_span.merge(body.span);
        Ok(Expr::new(ExprKind::For { var, iter, body }, span))
    }

    fn parse_range_from(&mut self, start: Expr) -> Result<Expr> {
        let (op_token, _op_span) = self.tokens.advance().expect("checked by parser logic");
        let inclusive = matches!(op_token, Token::DotDotEqual);

        let end = Box::new(self.parse_prefix()?);
        let span = start.span.merge(end.span);

        Ok(Expr::new(
            ExprKind::Range {
                start: Box::new(start),
                end,
                inclusive,
            },
            span,
        ))
    }

    fn parse_import(&mut self) -> Result<Expr> {
        let start_span = self.tokens.advance().expect("checked by parser logic").1; // consume import/use

        let mut path_parts = Vec::new();

        // Parse the path (e.g., std::io::prelude)
        loop {
            match self.tokens.peek() {
                Some((Token::Identifier(part), _)) => {
                    path_parts.push(part.clone());
                    self.tokens.advance();
                }
                _ => break,
            }

            // Check for ::
            if !matches!(self.tokens.peek(), Some((Token::ColonColon, _))) {
                break;
            }
            self.tokens.advance(); // consume ::
        }

        let path = path_parts.join("::");

        // Check for specific imports like ::{Read, Write}
        let items = if matches!(self.tokens.peek(), Some((Token::LeftBrace, _))) {
            self.tokens.advance(); // consume {
            let mut items = Vec::new();

            loop {
                match self.tokens.advance() {
                    Some((Token::Identifier(item), _)) => {
                        items.push(item);
                    }
                    Some((Token::RightBrace, _)) => break,
                    _ => {}
                }

                if matches!(self.tokens.peek(), Some((Token::Comma, _))) {
                    self.tokens.advance();
                }
            }

            items
        } else {
            Vec::new()
        };

        let span = start_span; // simplified for now
        Ok(Expr::new(ExprKind::Import { path, items }, span))
    }

    fn precedence(token: &Token) -> i32 {
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

    fn is_right_associative(op: BinaryOp) -> bool {
        matches!(op, BinaryOp::Power)
    }

    fn token_to_binary_op(token: &Token) -> Result<BinaryOp> {
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

    fn token_to_unary_op(token: &Token) -> Result<UnaryOp> {
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
    #[allow(clippy::approx_constant)] // Intentional literal for test
    fn test_parse_literals() {
        let mut parser = Parser::new("42");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal"),
        }

        let mut parser = Parser::new("3.14");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < 0.001), // Intentional literal for test
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_binary_ops() {
        let mut parser = Parser::new("1 + 2 * 3");
        let expr = parser.parse().expect("checked by parser logic");
        // Should parse as 1 + (2 * 3) due to precedence
        match expr.kind {
            ExprKind::Binary {
                op: BinaryOp::Add, ..
            } => {}
            _ => panic!("Expected addition at top level"),
        }
    }

    #[test]
    fn test_parse_function() {
        let mut parser = Parser::new("fun add(x: i32, y: i32) -> i32 { x + y }");
        let expr = parser.parse().expect("checked by parser logic");
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
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Pipeline { stages, .. } => {
                assert_eq!(stages.len(), 2);
            }
            _ => panic!("Expected pipeline"),
        }
    }

    #[test]
    fn test_parse_match() {
        let mut parser = Parser::new(r#"match x { 1 => "one", 2 => "two", _ => "other" }"#);
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Match { arms, .. } => {
                assert_eq!(arms.len(), 3);
            }
            _ => panic!("Expected match expression"),
        }
    }

    #[test]
    fn test_parse_let() {
        let mut parser = Parser::new("let x = 42 in x + 1");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Let { name, .. } => {
                assert_eq!(name, "x");
            }
            _ => panic!("Expected let expression"),
        }
    }

    #[test]
    fn test_parse_for() {
        let mut parser = Parser::new("for i in 1..10 { print(i) }");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::For { var, .. } => {
                assert_eq!(var, "i");
            }
            _ => panic!("Expected for expression"),
        }
    }

    #[test]
    fn test_parse_range() {
        let mut parser = Parser::new("1..10");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Range { inclusive, .. } => {
                assert!(!inclusive);
            }
            _ => panic!("Expected range expression"),
        }

        let mut parser = Parser::new("1..=10");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Range { inclusive, .. } => {
                assert!(inclusive);
            }
            _ => panic!("Expected inclusive range expression"),
        }
    }

    #[test]
    fn test_parse_list() {
        let mut parser = Parser::new("[]");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::List(ref elements) => {
                assert_eq!(elements.len(), 0);
            }
            _ => panic!("Expected empty list"),
        }

        let mut parser = Parser::new("[1, 2, 3]");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::List(ref elements) => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected list with 3 elements"),
        }
    }

    #[test]
    fn test_parse_import() {
        let mut parser = Parser::new("import std::io");
        let expr = parser.parse().expect("checked by parser logic");
        match expr.kind {
            ExprKind::Import { path, items } => {
                assert_eq!(path, "std::io");
                assert!(items.is_empty());
            }
            _ => panic!("Expected import"),
        }
    }
}
