//! Error recovery parser for better error messages and IDE support

#![allow(clippy::items_after_statements)] // Recovery parser needs constants in local scopes

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Param, Span, Type, TypeKind};
use crate::frontend::lexer::{Token, TokenStream};
use anyhow::Result;
use std::fmt;

/// Parse error with recovery information
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
    pub recovery_hint: Option<String>,
    pub expected: Vec<Token>,
    pub found: Option<Token>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {:?}", self.message, self.span)?;
        if let Some(ref hint) = self.recovery_hint {
            write!(f, " (hint: {hint})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

/// Result of parsing with error recovery
#[derive(Debug)]
pub struct ParseResult {
    pub ast: Option<Expr>,
    pub errors: Vec<ParseError>,
    pub partial_ast: bool,
}

/// Parser with error recovery capabilities
pub struct RecoveryParser<'a> {
    tokens: TokenStream<'a>,
    errors: Vec<ParseError>,
    recovery_mode: bool,
    ghost_node_count: usize,
    recursion_depth: usize,
}

impl<'a> RecoveryParser<'a> {
    #[must_use] pub fn new(input: &'a str) -> Self {
        Self {
            tokens: TokenStream::new(input),
            errors: Vec::new(),
            recovery_mode: false,
            ghost_node_count: 0,
            recursion_depth: 0,
        }
    }

    /// Parse with error recovery, always producing some AST
    pub fn parse_with_recovery(&mut self) -> ParseResult {
        match self.parse_expr_recovery() {
            Ok(ast) => ParseResult {
                ast: Some(ast),
                errors: self.errors.clone(),
                partial_ast: !self.errors.is_empty(),
            },
            Err(_) if !self.errors.is_empty() => {
                // We have errors but can produce a partial AST
                let ghost = self.create_ghost_node("Failed to parse expression");
                ParseResult {
                    ast: Some(ghost),
                    errors: self.errors.clone(),
                    partial_ast: true,
                }
            }
            Err(e) => {
                // Fatal error, no recovery possible
                self.errors.push(ParseError {
                    message: e.to_string(),
                    span: Span::new(0, 0),
                    recovery_hint: None,
                    expected: vec![],
                    found: None,
                });
                ParseResult {
                    ast: None,
                    errors: self.errors.clone(),
                    partial_ast: false,
                }
            }
        }
    }

    fn parse_expr_recovery(&mut self) -> Result<Expr> {
        self.parse_expr_with_precedence_recovery(0)
    }

    fn parse_expr_with_precedence_recovery(&mut self, min_prec: i32) -> Result<Expr> {
        let mut left = self.parse_prefix_recovery()?;
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        let mut iteration_count = 0;

        loop {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                self.record_error(
                    "Expression too complex or malformed".to_string(),
                    Some("Simplify the expression".to_string()),
                );
                break;
            }

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

            match self.tokens.advance() {
                Some((op_token, _op_span)) => {
                    let op = match self.token_to_binary_op(&op_token) {
                        Ok(op) => op,
                        Err(e) => {
                            self.record_error(format!("Invalid operator: {e}"), None);
                            // Skip the invalid operator and continue
                            continue;
                        }
                    };

                    let right = if let Ok(expr) = self.parse_expr_with_precedence_recovery(prec + 1) { expr } else {
                        // Create ghost node for missing right operand
                        let ghost = self.create_ghost_node("Missing right operand");
                        self.record_error(
                            format!("Expected expression after '{op_token:?}'"),
                            Some("Add the right side of the operation".to_string()),
                        );
                        ghost
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
                None => break,
            }
        }

        Ok(left)
    }

    fn parse_prefix_recovery(&mut self) -> Result<Expr> {
        const MAX_RECURSION_DEPTH: usize = 100;

        self.recursion_depth += 1;
        if self.recursion_depth > MAX_RECURSION_DEPTH {
            self.recursion_depth -= 1;
            self.record_error(
                "Expression too deeply nested".to_string(),
                Some("Simplify the expression".to_string()),
            );
            return Ok(self.create_ghost_node("Max recursion depth"));
        }

        let result = match self.tokens.peek() {
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
                Ok(Expr::new(ExprKind::Identifier(name), span))
            }
            Some((Token::If, _)) => self.parse_if_recovery(),
            Some((Token::Let, _)) => self.parse_let_recovery(),
            Some((Token::Fun, _)) => self.parse_function_recovery(),
            Some((Token::LeftBracket, _)) => self.parse_list_recovery(),
            Some((Token::LeftParen, _)) => self.parse_paren_recovery(),
            Some((token, _span)) => {
                let token_clone = token.clone();

                // Special handling for binary operators in prefix position
                if token.is_binary_op() {
                    self.tokens.advance(); // Consume the misplaced operator
                    self.record_error(
                        format!("Unexpected operator: {token_clone:?}"),
                        Some("An expression was expected here, not an operator".to_string()),
                    );
                    // Try to parse what comes after the misplaced operator
                    return self.parse_prefix_recovery();
                }

                self.tokens.advance(); // Advance before recording error
                self.record_error(
                    format!("Unexpected token: {token_clone:?}"),
                    Some("Expected an expression".to_string()),
                );
                // Try to recover
                self.synchronize();
                Ok(self.create_ghost_node("Unexpected token"))
            }
            None => {
                self.record_error(
                    "Unexpected end of input".to_string(),
                    Some("Add more code to complete the expression".to_string()),
                );
                Ok(self.create_ghost_node("Unexpected EOF"))
            }
        };

        self.recursion_depth -= 1;
        result
    }

    fn parse_if_recovery(&mut self) -> Result<Expr> {
        let start_span = self.expect_or_recover(&Token::If);

        let condition = if let Ok(expr) = self.parse_expr_recovery() { Box::new(expr) } else {
            self.record_error(
                "Missing condition in if expression".to_string(),
                Some("Add a condition after 'if'".to_string()),
            );
            Box::new(self.create_ghost_node("Missing condition"))
        };

        let _ = self.expect_or_recover(&Token::LeftBrace);
        let then_branch = Box::new(self.parse_block_recovery());

        let else_branch = if matches!(self.tokens.peek(), Some((Token::Else, _))) {
            self.tokens.advance();
            let _ = self.expect_or_recover(&Token::LeftBrace);
            Some(Box::new(self.parse_block_recovery()))
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

    fn parse_let_recovery(&mut self) -> Result<Expr> {
        let start_span = self.expect_or_recover(&Token::Let);

        let name = if let Some((Token::Identifier(name), _)) = self.tokens.advance() { name } else {
            self.record_error(
                "Expected identifier after 'let'".to_string(),
                Some("Add a variable name".to_string()),
            );
            format!("_ghost_{}", self.ghost_node_count)
        };

        let _ = self.expect_or_recover(&Token::Equal);

        let value = if let Ok(expr) = self.parse_expr_recovery() { Box::new(expr) } else {
            self.record_error(
                "Missing value in let binding".to_string(),
                Some("Add a value after '='".to_string()),
            );
            Box::new(self.create_ghost_node("Missing value"))
        };

        let body = if matches!(self.tokens.peek(), Some((Token::In, _))) {
            self.tokens.advance();
            match self.parse_expr_recovery() {
                Ok(expr) => Box::new(expr),
                Err(_) => Box::new(self.create_ghost_node("Missing body")),
            }
        } else {
            Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
        };

        let span = start_span.merge(body.span);
        Ok(Expr::new(ExprKind::Let { name, value, body }, span))
    }

    fn parse_function_recovery(&mut self) -> Result<Expr> {
        let start_span = self.expect_or_recover(&Token::Fun);

        let name = if let Some((Token::Identifier(name), _)) = self.tokens.advance() { name } else {
            self.record_error(
                "Expected function name".to_string(),
                Some("Add a function name after 'fun'".to_string()),
            );
            format!("_ghost_fn_{}", self.ghost_node_count)
        };

        let _ = self.expect_or_recover(&Token::LeftParen);
        let params = self.parse_params_recovery().unwrap_or_default();
        let _ = self.expect_or_recover(&Token::RightParen);

        let return_type = if matches!(self.tokens.peek(), Some((Token::Arrow, _))) {
            self.tokens.advance();
            Some(self.parse_type_recovery())
        } else {
            None
        };

        let _ = self.expect_or_recover(&Token::LeftBrace);
        let body = Box::new(self.parse_block_recovery());

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

    fn parse_block_recovery(&mut self) -> Expr {
        let mut exprs = Vec::new();
        let start_span = self
            .tokens
            .peek()
            .map_or(Span::new(0, 0), |(_, span)| *span);

        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _)) | None) {
            if let Ok(expr) = self.parse_expr_recovery() { exprs.push(expr) } else {
                // Try to recover by finding next statement
                self.synchronize();
                if self.recovery_mode {
                    exprs.push(self.create_ghost_node("Recovery statement"));
                }
            }

            // Optional semicolon
            if matches!(self.tokens.peek(), Some((Token::Semicolon, _))) {
                self.tokens.advance();
            }
        }

        let _ = self.expect_or_recover(&Token::RightBrace);

        let span = if let Some(last) = exprs.last() {
            start_span.merge(last.span)
        } else {
            start_span
        };

        Expr::new(ExprKind::Block(exprs), span)
    }

    fn parse_list_recovery(&mut self) -> Result<Expr> {
        let start_span = self.expect_or_recover(&Token::LeftBracket);
        let mut elements = Vec::new();

        while !matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
            match self.parse_expr_recovery() {
                Ok(expr) => elements.push(expr),
                Err(_) => {
                    self.synchronize_to(&[Token::Comma, Token::RightBracket]);
                }
            }

            if matches!(self.tokens.peek(), Some((Token::Comma, _))) {
                self.tokens.advance();
            } else {
                break;
            }
        }

        let end_span = self.expect_or_recover(&Token::RightBracket);
        let span = start_span.merge(end_span);

        Ok(Expr::new(ExprKind::List(elements), span))
    }

    fn parse_paren_recovery(&mut self) -> Result<Expr> {
        self.tokens.advance(); // consume (
        let expr = self.parse_expr_recovery()?;
        let _ = self.expect_or_recover(&Token::RightParen);
        Ok(expr)
    }

    fn parse_params_recovery(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();

        if matches!(self.tokens.peek(), Some((Token::RightParen, _))) {
            return Ok(params);
        }

        loop {
            let Some((Token::Identifier(name), name_span)) = self.tokens.advance() else {
                self.record_error(
                    "Expected parameter name".to_string(),
                    Some("Add a parameter name".to_string()),
                );
                self.synchronize_to(&[Token::Comma, Token::RightParen]);
                continue;
            };

            let ty = if matches!(self.tokens.peek(), Some((Token::Colon, _))) {
                self.tokens.advance();
                self.parse_type_recovery()
            } else {
                // Default to inferred type
                Type {
                    kind: TypeKind::Named("_".to_string()),
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
                _ => {
                    self.record_error("Expected ',' or ')' in parameter list".to_string(), None);
                    break;
                }
            }
        }

        Ok(params)
    }

    fn parse_type_recovery(&mut self) -> Type {
        let (base_type, span) = if let Some((Token::Identifier(name), span)) = self.tokens.advance() { (TypeKind::Named(name), span) } else {
            self.record_error(
                "Expected type".to_string(),
                Some("Add a type annotation".to_string()),
            );
            (TypeKind::Named("_".to_string()), Span::new(0, 0))
        };

        let kind = if matches!(self.tokens.peek(), Some((Token::Question, _))) {
            self.tokens.advance();
            TypeKind::Optional(Box::new(Type {
                kind: base_type,
                span,
            }))
        } else {
            base_type
        };

        Type { kind, span }
    }

    /// Create a ghost node for error recovery
    fn create_ghost_node(&mut self, reason: &str) -> Expr {
        self.ghost_node_count += 1;
        Expr::new(
            ExprKind::Identifier(format!(
                "_ghost_{}_{}",
                self.ghost_node_count,
                reason.replace(' ', "_")
            )),
            Span::new(0, 0),
        )
    }

    /// Record an error for later reporting
    fn record_error(&mut self, message: String, hint: Option<String>) {
        let span = self
            .tokens
            .peek()
            .map_or(Span::new(0, 0), |(_, s)| *s);

        self.errors.push(ParseError {
            message,
            span,
            recovery_hint: hint,
            expected: vec![],
            found: self.tokens.peek().map(|(t, _)| t.clone()),
        });
    }

    /// Expect a token or record error and try to recover
    fn expect_or_recover(&mut self, expected: &Token) -> Span {
        match self.tokens.peek() {
            Some((token, span)) if token == expected => {
                let span = *span;
                self.tokens.advance();
                span
            }
            _ => {
                self.record_error(
                    format!("Expected {expected:?}"),
                    Some(format!("Add '{expected:?}' here")),
                );
                self.recovery_mode = true;
                Span::new(0, 0)
            }
        }
    }

    /// Synchronize to a known recovery point
    fn synchronize(&mut self) {
        self.recovery_mode = true;

        // Synchronization tokens - statement boundaries
        let sync_tokens = [
            Token::Semicolon,
            Token::RightBrace,
            Token::Fun,
            Token::Let,
            Token::If,
            Token::For,
            Token::Match,
        ];

        let mut tokens_consumed = 0;
        const MAX_SYNC_TOKENS: usize = 100; // Prevent infinite loops

        while let Some((token, _)) = self.tokens.peek() {
            if tokens_consumed >= MAX_SYNC_TOKENS {
                // Force exit to prevent infinite loop
                break;
            }

            if sync_tokens.iter().any(|t| t == token) {
                if matches!(token, Token::Semicolon) {
                    self.tokens.advance(); // consume semicolon
                }
                break;
            }
            self.tokens.advance();
            tokens_consumed += 1;
        }

        self.recovery_mode = false;
    }

    /// Synchronize to specific tokens
    fn synchronize_to(&mut self, targets: &[Token]) {
        let mut tokens_consumed = 0;
        const MAX_SYNC_TOKENS: usize = 100; // Prevent infinite loops

        while let Some((token, _)) = self.tokens.peek() {
            if tokens_consumed >= MAX_SYNC_TOKENS {
                // Force exit to prevent infinite loop
                break;
            }

            if targets.iter().any(|t| t == token) {
                break;
            }
            self.tokens.advance();
            tokens_consumed += 1;
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
            _ => anyhow::bail!("Not a binary operator: {:?}", token),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_missing_operand() {
        let mut parser = RecoveryParser::new("1 +");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(!result.errors.is_empty());
        assert!(result.partial_ast);
    }

    #[test]
    fn test_recovery_missing_paren() {
        let mut parser = RecoveryParser::new("(1 + 2");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_recovery_invalid_token() {
        let mut parser = RecoveryParser::new("let x = @ + 1");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_recovery_incomplete_if() {
        let mut parser = RecoveryParser::new("if x > 0 { print(x)");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(!result.errors.is_empty());
        assert!(result.partial_ast);
    }

    #[test]
    fn test_recovery_missing_function_body() {
        let mut parser = RecoveryParser::new("fun foo(x: i32)");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_no_errors_on_valid_code() {
        let mut parser = RecoveryParser::new("1 + 2 * 3");
        let result = parser.parse_with_recovery();

        assert!(result.ast.is_some());
        assert!(result.errors.is_empty());
        assert!(!result.partial_ast);
    }
}
