//! Parser for converting tokens into AST

#![allow(clippy::expect_used)] // Parser needs expect for checked conditions

use crate::frontend::ast::{ActorHandler, Attribute, BinaryOp, Expr, ExprKind, ImplMethod, Literal, MatchArm, Param, Pattern, PipelineStage, Span, StringPart, StructField, TraitMethod, Type, TypeKind, UnaryOp};
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
            let attributes = self.parse_attributes()?;
            let mut expr = self.parse_expr()?;
            expr.attributes = attributes;
            exprs.push(expr);
            
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
                attributes: Vec::new(),
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

        // Check for postfix operators
        loop {
            match self.tokens.peek() {
                Some((Token::Question, _)) => {
                    let span = self.tokens.advance().expect("checked: Question token exists").1;
                    let full_span = left.span.merge(span);
                    left = Expr::new(
                        ExprKind::Try {
                            expr: Box::new(left),
                        },
                        full_span,
                    );
                }
                Some((Token::Pipeline, _)) => {
                    left = self.parse_pipeline(left)?;
                }
                Some((Token::DotDot | Token::DotDotEqual, _)) => {
                    left = self.parse_range_from(left)?;
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn handle_postfix_operators(&mut self, mut expr: Expr) -> Result<Expr> {
        loop {
            match self.tokens.peek() {
                Some((Token::Dot, _)) => {
                    self.tokens.advance();
                    expr = self.parse_method_call(expr)?;
                }
                Some((Token::LeftParen, _)) => {
                    expr = self.parse_call(expr)?;
                }
                Some((Token::Question, _)) => {
                    let span = self.tokens.advance().expect("checked: Question token exists").1;
                    // Check if next token is a parenthesis (ask operation)
                    if let Some((Token::LeftParen, _)) = self.tokens.peek() {
                        self.tokens.advance();
                        let message = Box::new(self.parse_expr()?);
                        
                        // Optional timeout after comma
                        let timeout = if let Some((Token::Comma, _)) = self.tokens.peek() {
                            self.tokens.advance();
                            Some(Box::new(self.parse_expr()?))
                        } else {
                            None
                        };
                        
                        self.tokens.expect(Token::RightParen)?;
                        let full_span = expr.span.merge(span);
                        expr = Expr::new(
                            ExprKind::Ask {
                                actor: Box::new(expr),
                                message,
                                timeout,
                            },
                            full_span,
                        );
                    } else {
                        // Regular try operator
                        let full_span = expr.span.merge(span);
                        expr = Expr::new(
                            ExprKind::Try {
                                expr: Box::new(expr),
                            },
                            full_span,
                        );
                    }
                }
                Some((Token::Bang, _)) => {
                    let span = self.tokens.advance().expect("checked: Bang token exists").1;
                    // Check if next token is a parenthesis (send operation)  
                    if let Some((Token::LeftParen, _)) = self.tokens.peek() {
                        self.tokens.advance();
                        let message = Box::new(self.parse_expr()?);
                        self.tokens.expect(Token::RightParen)?;
                        let full_span = expr.span.merge(span);
                        expr = Expr::new(
                            ExprKind::Send {
                                actor: Box::new(expr),
                                message,
                            },
                            full_span,
                        );
                    } else {
                        // Not a send operation, stop parsing
                        break;
                    }
                }
                Some((Token::LeftBrace, _)) => {
                    // Check if this could be a struct literal
                    // Only parse as struct literal if the expression is a simple identifier
                    // and it starts with an uppercase letter (convention for types)
                    if let ExprKind::Identifier(name) = &expr.kind {
                        if name.chars().next().is_some_and(char::is_uppercase) {
                            let span = expr.span;
                            return self.parse_struct_literal(name.clone(), span);
                        }
                    }
                    // Otherwise, not a struct literal, stop parsing postfix
                    break;
                }
                _ => break,
            }
        }
        Ok(expr)
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
                
                // Check if the string contains interpolation markers
                if s.contains('{') && s.contains('}') {
                    let parts = self.parse_string_interpolation(&s)?;
                    Ok(Expr::new(ExprKind::StringInterpolation { parts }, span))
                } else {
                    Ok(Expr::new(ExprKind::Literal(Literal::String(s)), span))
                }
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

                let expr = Expr::new(ExprKind::Identifier(name), span);
                self.handle_postfix_operators(expr)
            }
            Some((Token::If, _)) => self.parse_if(),
            Some((Token::Let, _)) => self.parse_let(),
            Some((Token::Async, _)) => {
                // Could be async function or async block
                let next = self.tokens.peek_nth(1);
                if matches!(next, Some((Token::Fun, _))) {
                    self.parse_function()
                } else {
                    self.parse_async_block()
                }
            }
            Some((Token::Await, _)) => self.parse_await(),
            Some((Token::Fun, _)) => self.parse_function(),
            Some((Token::Match, _)) => self.parse_match(),
            Some((Token::For, _)) => self.parse_for(),
            Some((Token::While, _)) => self.parse_while(),
            Some((Token::Try, _)) => self.parse_try_catch(),
            Some((Token::Struct, _)) => self.parse_struct(),
            Some((Token::Trait, _)) => self.parse_trait(),
            Some((Token::Impl, _)) => self.parse_impl(),
            Some((Token::Actor, _)) => self.parse_actor(),
            Some((Token::Import | Token::Use, _)) => self.parse_import(),
            Some((Token::Break, _)) => self.parse_break(),
            Some((Token::Continue, _)) => self.parse_continue(),
            Some((Token::LeftBracket, _)) => self.parse_list(),
            Some((Token::DataFrame, _)) => self.parse_dataframe(),
            Some((Token::Pipe | Token::OrOr, _)) => self.parse_lambda(),
            Some((Token::LeftParen, _span)) => {
                self.tokens.advance();
                let expr = self.parse_expr()?;
                self.tokens.expect(Token::RightParen)?;
                self.handle_postfix_operators(expr)
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
        // Check for async modifier
        let is_async = if let Some((Token::Async, _)) = self.tokens.peek() {
            self.tokens.advance();
            true
        } else {
            false
        };
        
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
                is_async,
            },
            span,
        ))
    }

    fn parse_lambda(&mut self) -> Result<Expr> {
        // Handle || as a special case for empty parameter lambdas
        if let Some((Token::OrOr, span)) = self.tokens.peek() {
            let start_span = *span;
            self.tokens.advance();
            
            // Parse the body
            let body = Box::new(self.parse_expr()?);
            let end_span = body.span;
            
            return Ok(Expr::new(
                ExprKind::Lambda { 
                    params: Vec::new(), 
                    body 
                },
                start_span.merge(end_span),
            ));
        }
        
        let start_span = self.tokens.expect(Token::Pipe)?;
        
        // Parse parameters between pipes: |x, y|
        let mut params = Vec::new();
        
        // Check for empty params with single |
        if let Some((Token::Pipe, _)) = self.tokens.peek() {
            self.tokens.advance();
        } else {
            loop {
                let (name, name_span) = match self.tokens.advance() {
                    Some((Token::Identifier(name), span)) => (name, span),
                    _ => bail!("Expected parameter name in lambda"),
                };
                
                // Type annotation is optional for lambdas
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
                    Some((Token::Pipe, _)) => {
                        self.tokens.advance();
                        break;
                    }
                    _ => bail!("Expected ',' or '|' in lambda parameter list"),
                }
            }
        }
        
        // Parse the body
        let body = Box::new(self.parse_expr()?);
        let end_span = body.span;
        
        Ok(Expr::new(
            ExprKind::Lambda { params, body },
            start_span.merge(end_span),
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
        
        // Check for empty list
        if matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
            let end_span = self.tokens.expect(Token::RightBracket)?;
            let span = start_span.merge(end_span);
            let list_expr = Expr::new(ExprKind::List(Vec::new()), span);
            return self.handle_postfix_operators(list_expr);
        }
        
        // Parse the first element
        let first_element = self.parse_expr()?;
        
        // Check if this is a list comprehension by looking for 'for'
        if matches!(self.tokens.peek(), Some((Token::For, _))) {
            return self.parse_list_comprehension(start_span, first_element);
        }
        
        // Regular list - continue parsing elements
        let mut elements = vec![first_element];
        
        while let Some((Token::Comma, _)) = self.tokens.peek() {
            self.tokens.advance(); // consume comma
            
            if matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
                break; // trailing comma
            }
            
            elements.push(self.parse_expr()?);
        }

        let end_span = self.tokens.expect(Token::RightBracket)?;
        let span = start_span.merge(end_span);

        let list_expr = Expr::new(ExprKind::List(elements), span);
        self.handle_postfix_operators(list_expr)
    }

    fn parse_list_comprehension(&mut self, start_span: Span, element: Expr) -> Result<Expr> {
        // We've already parsed the element expression
        // Now expect: for variable in iterable [if condition]
        
        self.tokens.expect(Token::For)?;
        
        // Parse variable name
        let variable = if let Some((Token::Identifier(name), _)) = self.tokens.advance() {
            name
        } else {
            bail!("Expected variable name after 'for' in list comprehension");
        };
        
        self.tokens.expect(Token::In)?;
        
        // Parse iterable expression
        let iterable = self.parse_expr()?;
        
        // Check for optional if condition
        let condition = if matches!(self.tokens.peek(), Some((Token::If, _))) {
            self.tokens.advance(); // consume 'if'
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        
        let end_span = self.tokens.expect(Token::RightBracket)?;
        let span = start_span.merge(end_span);
        
        let comprehension_expr = Expr::new(
            ExprKind::ListComprehension {
                element: Box::new(element),
                variable,
                iterable: Box::new(iterable),
                condition,
            },
            span,
        );
        
        self.handle_postfix_operators(comprehension_expr)
    }

    fn parse_dataframe(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::DataFrame)?;
        self.tokens.expect(Token::Bang)?;
        self.tokens.expect(Token::LeftBracket)?;
        
        let mut columns = Vec::new();
        let mut rows = Vec::new();
        
        // Parse column names (first row should be column identifiers)
        let mut first_row = true;
        
        while !matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
            if !first_row {
                // Expect semicolon between rows
                if let Some((Token::Semicolon, _)) = self.tokens.peek() {
                    self.tokens.advance();
                } else if !matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
                    bail!("Expected ';' or ']' in DataFrame literal");
                }
            }
            
            if matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
                break;
            }
            
            let mut row = Vec::new();
            loop {
                if first_row {
                    // Parse column names
                    match self.tokens.advance() {
                        Some((Token::Identifier(name), _)) => columns.push(name),
                        _ => bail!("Expected column name in DataFrame literal"),
                    }
                } else {
                    // Parse data values
                    row.push(self.parse_expr()?);
                }
                
                // Check for comma or end of row
                match self.tokens.peek() {
                    Some((Token::Comma, _)) => {
                        self.tokens.advance();
                    }
                    Some((Token::Semicolon | Token::RightBracket, _)) => break,
                    _ => bail!("Expected ',' ';' or ']' in DataFrame literal"),
                }
            }
            
            if !first_row {
                if row.len() != columns.len() {
                    bail!("DataFrame row has {} values but {} columns were defined", row.len(), columns.len());
                }
                rows.push(row);
            }
            first_row = false;
        }
        
        let end_span = self.tokens.expect(Token::RightBracket)?;
        
        let dataframe_expr = Expr::new(
            ExprKind::DataFrame { columns, rows },
            start_span.merge(end_span),
        );
        self.handle_postfix_operators(dataframe_expr)
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

    fn parse_method_call(&mut self, receiver: Expr) -> Result<Expr> {
        let method = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected method name after '.'"),
        };

        // Check if it's a method call (with parentheses) or field access
        if let Some((Token::LeftParen, _)) = self.tokens.peek() {
            self.tokens.advance();
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
            let span = receiver.span.merge(end_span);

            Ok(Expr::new(
                ExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method,
                    args,
                },
                span,
            ))
        } else {
            // Field access
            let span = receiver.span;
            Ok(Expr::new(
                ExprKind::FieldAccess {
                    object: Box::new(receiver),
                    field: method,
                },
                span,
            ))
        }
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

    fn parse_while(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::While)?;
        
        // Parse the condition
        let condition = Box::new(self.parse_expr()?);
        
        // Parse the body block
        self.tokens.expect(Token::LeftBrace)?;
        let body = Box::new(self.parse_block()?);
        
        let span = start_span.merge(body.span);
        Ok(Expr::new(ExprKind::While { condition, body }, span))
    }
    
    fn parse_break(&mut self) -> Result<Expr> {
        let span = self.tokens.expect(Token::Break)?;
        
        // Check for optional label
        let label = if let Some((Token::Identifier(name), _)) = self.tokens.peek() {
            let name = name.clone();
            self.tokens.advance();
            Some(name)
        } else {
            None
        };
        
        Ok(Expr::new(ExprKind::Break { label }, span))
    }
    
    fn parse_continue(&mut self) -> Result<Expr> {
        let span = self.tokens.expect(Token::Continue)?;
        
        // Check for optional label
        let label = if let Some((Token::Identifier(name), _)) = self.tokens.peek() {
            let name = name.clone();
            self.tokens.advance();
            Some(name)
        } else {
            None
        };
        
        Ok(Expr::new(ExprKind::Continue { label }, span))
    }

    fn parse_try_catch(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Try)?;
        
        // Parse the try block
        self.tokens.expect(Token::LeftBrace)?;
        let try_block = Box::new(self.parse_block()?);
        
        // Expect catch keyword
        self.tokens.expect(Token::Catch)?;
        
        // Parse catch variable (error binding)
        self.tokens.expect(Token::LeftParen)?;
        let catch_var = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected identifier for catch variable"),
        };
        self.tokens.expect(Token::RightParen)?;
        
        // Parse the catch block
        self.tokens.expect(Token::LeftBrace)?;
        let catch_block = Box::new(self.parse_block()?);
        
        let span = start_span.merge(catch_block.span);
        Ok(Expr::new(
            ExprKind::TryCatch {
                try_block,
                catch_var,
                catch_block,
            },
            span,
        ))
    }

    fn parse_async_block(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Async)?;
        self.tokens.expect(Token::LeftBrace)?;
        let block = self.parse_block()?;
        let span = start_span.merge(block.span);
        
        // For now, wrap the async block in a lambda that returns a future
        // In a full implementation, we'd have a dedicated AsyncBlock AST node
        Ok(Expr::new(
            ExprKind::Lambda {
                params: Vec::new(),
                body: Box::new(block),
            },
            span,
        ))
    }

    fn parse_await(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Await)?;
        let expr = Box::new(self.parse_prefix()?);
        let span = start_span.merge(expr.span);
        Ok(Expr::new(ExprKind::Await { expr }, span))
    }

    fn parse_struct(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Struct)?;
        
        // Parse struct name
        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected struct name"),
        };
        
        // Parse struct fields
        self.tokens.expect(Token::LeftBrace)?;
        let mut fields = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            // Parse field visibility (pub is optional)
            let is_pub = if let Some((Token::Pub, _)) = self.tokens.peek() {
                self.tokens.advance();
                true
            } else {
                false
            };
            
            // Parse field name
            let field_name = match self.tokens.advance() {
                Some((Token::Identifier(name), _)) => name,
                _ => bail!("Expected field name in struct"),
            };
            
            // Parse type annotation
            self.tokens.expect(Token::Colon)?;
            let field_type = self.parse_type()?;
            
            fields.push(StructField {
                name: field_name,
                ty: field_type,
                is_pub,
            });
            
            // Handle comma or end of struct
            match self.tokens.peek() {
                Some((Token::Comma, _)) => {
                    self.tokens.advance();
                }
                Some((Token::RightBrace, _)) => break,
                _ => bail!("Expected ',' or '}}' in struct definition"),
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(ExprKind::Struct { name, fields }, span))
    }

    fn parse_struct_literal(&mut self, name: String, start_span: Span) -> Result<Expr> {
        self.tokens.expect(Token::LeftBrace)?;
        
        let mut fields = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            // Parse field name
            let field_name = match self.tokens.advance() {
                Some((Token::Identifier(name), _)) => name,
                _ => bail!("Expected field name in struct literal"),
            };
            
            // Parse colon and value
            self.tokens.expect(Token::Colon)?;
            let value = self.parse_expr()?;
            
            fields.push((field_name, value));
            
            // Handle comma or end of struct literal
            match self.tokens.peek() {
                Some((Token::Comma, _)) => {
                    self.tokens.advance();
                }
                Some((Token::RightBrace, _)) => break,
                _ => bail!("Expected ',' or '}}' in struct literal"),
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(ExprKind::StructLiteral { name, fields }, span))
    }

    fn parse_trait(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Trait)?;
        
        // Parse trait name
        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected trait name"),
        };
        
        // Parse trait body
        self.tokens.expect(Token::LeftBrace)?;
        let mut methods = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            // Parse method
            let method = self.parse_trait_method()?;
            methods.push(method);
            
            // Handle semicolon or comma
            if let Some((Token::Semicolon | Token::Comma, _)) = self.tokens.peek() {
                self.tokens.advance();
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(ExprKind::Trait { name, methods }, span))
    }

    fn parse_trait_method(&mut self) -> Result<TraitMethod> {
        // Parse fn keyword
        self.tokens.expect(Token::Fun)?;
        
        // Parse method name
        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected method name"),
        };
        
        // Parse parameters
        self.tokens.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.tokens.expect(Token::RightParen)?;
        
        // Parse return type if present
        let return_type = if let Some((Token::Arrow, _)) = self.tokens.peek() {
            self.tokens.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Check for method body (default implementation) or just signature
        let body = if let Some((Token::LeftBrace, _)) = self.tokens.peek() {
            self.tokens.expect(Token::LeftBrace)?;
            let body_expr = self.parse_block()?;
            Some(Box::new(body_expr))
        } else {
            None
        };
        
        Ok(TraitMethod {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_impl(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Impl)?;
        
        // Parse trait name (optional) and for_type
        let (trait_name, for_type) = if let Some((Token::Identifier(name), _)) = self.tokens.peek() {
            let first_name = name.clone();
            self.tokens.advance();
            
            if let Some((Token::For, _)) = self.tokens.peek() {
                // impl TraitName for TypeName
                self.tokens.advance();
                let type_name = match self.tokens.advance() {
                    Some((Token::Identifier(name), _)) => name,
                    _ => bail!("Expected type name after 'for'"),
                };
                (Some(first_name), type_name)
            } else {
                // impl TypeName (inherent impl)
                (None, first_name)
            }
        } else {
            bail!("Expected trait or type name after 'impl'")
        };
        
        // Parse impl body
        self.tokens.expect(Token::LeftBrace)?;
        let mut methods = Vec::new();
        
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            // Parse method implementation
            let method = self.parse_impl_method()?;
            methods.push(method);
            
            // Skip optional semicolons
            if let Some((Token::Semicolon, _)) = self.tokens.peek() {
                self.tokens.advance();
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(
            ExprKind::Impl {
                trait_name,
                for_type,
                methods,
            },
            span,
        ))
    }

    fn parse_actor(&mut self) -> Result<Expr> {
        let start_span = self.tokens.expect(Token::Actor)?;
        
        // Parse actor name
        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected actor name after 'actor'"),
        };
        
        self.tokens.expect(Token::LeftBrace)?;
        
        let mut state = Vec::new();
        let mut handlers = Vec::new();
        
        // Parse actor body (state fields and message handlers)
        while !matches!(self.tokens.peek(), Some((Token::RightBrace, _))) {
            // Check if it's a state field or handler
            if let Some((Token::Identifier(_), _)) = self.tokens.peek() {
                // Could be a field (name: Type) or handler (on MessageType)
                let ident_name = if let Some((Token::Identifier(n), _)) = self.tokens.advance() {
                    n
                } else {
                    bail!("Expected identifier");
                };
                
                if let Some((Token::Colon, _)) = self.tokens.peek() {
                    // It's a state field
                    self.tokens.advance();
                    let ty = self.parse_type()?;
                    state.push(StructField {
                        name: ident_name,
                        ty,
                        is_pub: false,
                    });
                    
                    // Optional comma or semicolon
                    if matches!(self.tokens.peek(), Some((Token::Comma | Token::Semicolon, _))) {
                        self.tokens.advance();
                    }
                } else if ident_name == "on" {
                    // It's a message handler
                    let message_type = match self.tokens.advance() {
                        Some((Token::Identifier(name), _)) => name,
                        _ => bail!("Expected message type after 'on'"),
                    };
                    
                    // Parse optional parameters
                    let params = if let Some((Token::LeftParen, _)) = self.tokens.peek() {
                        self.tokens.advance();
                        let p = self.parse_params()?;
                        self.tokens.expect(Token::RightParen)?;
                        p
                    } else {
                        Vec::new()
                    };
                    
                    // Parse handler body
                    self.tokens.expect(Token::LeftBrace)?;
                    let body = Box::new(self.parse_block()?);
                    
                    handlers.push(ActorHandler {
                        message_type,
                        params,
                        body,
                    });
                } else {
                    // Must be an 'on' handler without the 'on' keyword being recognized
                    // Try to recover by treating it as a potential handler
                    bail!("Expected ':' for field or 'on' for handler, got '{}'", ident_name);
                }
            } else {
                bail!("Expected field or handler in actor body");
            }
        }
        
        let end_span = self.tokens.expect(Token::RightBrace)?;
        let span = start_span.merge(end_span);
        
        Ok(Expr::new(
            ExprKind::Actor {
                name,
                state,
                handlers,
            },
            span,
        ))
    }
    
    fn parse_impl_method(&mut self) -> Result<ImplMethod> {
        // Parse fn keyword
        self.tokens.expect(Token::Fun)?;
        
        // Parse method name
        let name = match self.tokens.advance() {
            Some((Token::Identifier(name), _)) => name,
            _ => bail!("Expected method name"),
        };
        
        // Parse parameters
        self.tokens.expect(Token::LeftParen)?;
        let params = self.parse_params()?;
        self.tokens.expect(Token::RightParen)?;
        
        // Parse return type if present
        let return_type = if let Some((Token::Arrow, _)) = self.tokens.peek() {
            self.tokens.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        // Parse method body (required for impl)
        self.tokens.expect(Token::LeftBrace)?;
        let body = Box::new(self.parse_block()?);
        
        Ok(ImplMethod {
            name,
            params,
            return_type,
            body,
        })
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
        while let Some((Token::Identifier(part), _)) = self.tokens.peek() {
            path_parts.push(part.clone());
            self.tokens.advance();

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

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>> {
        let mut attributes = Vec::new();
        
        while let Some((Token::Hash, start_span)) = self.tokens.peek().cloned() {
            self.tokens.advance(); // consume #
            
            if !matches!(self.tokens.peek(), Some((Token::LeftBracket, _))) {
                bail!("Expected '[' after '#'");
            }
            self.tokens.advance(); // consume [
            
            let name = if let Some((Token::Identifier(name), _)) = self.tokens.advance() {
                name
            } else {
                bail!("Expected attribute name");
            };
            
            let mut args = Vec::new();
            if matches!(self.tokens.peek(), Some((Token::LeftParen, _))) {
                self.tokens.advance(); // consume (
                
                while !matches!(self.tokens.peek(), Some((Token::RightParen, _))) {
                    if let Some((Token::Identifier(arg), _)) = self.tokens.advance() {
                        args.push(arg);
                    }
                    
                    if matches!(self.tokens.peek(), Some((Token::Comma, _))) {
                        self.tokens.advance();
                    }
                }
                
                if !matches!(self.tokens.peek(), Some((Token::RightParen, _))) {
                    bail!("Expected ')' to close attribute arguments");
                }
                self.tokens.advance(); // consume )
            }
            
            if !matches!(self.tokens.peek(), Some((Token::RightBracket, _))) {
                bail!("Expected ']' to close attribute");
            }
            let end_span = self.tokens.advance().expect("Expected ']' token").1; // consume ]
            
            attributes.push(Attribute {
                name,
                args,
                span: start_span.merge(end_span),
            });
        }
        
        Ok(attributes)
    }

    /// Parse string interpolation from a string containing {expr} patterns
    fn parse_string_interpolation(&mut self, s: &str) -> Result<Vec<StringPart>> {
        let mut parts = Vec::new();
        let mut current_text = String::new();
        let mut chars = s.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                // Escaped brace: {{
                chars.next(); // consume second '{'
                current_text.push('{');
            } else if ch == '}' && chars.peek() == Some(&'}') {
                // Escaped brace: }}
                chars.next(); // consume second '}'
                current_text.push('}');
            } else if ch == '{' {
                // Start of interpolation
                if !current_text.is_empty() {
                    parts.push(StringPart::Text(current_text.clone()));
                    current_text.clear();
                }
                
                // Collect expression until closing '}'
                let mut expr_text = String::new();
                let mut brace_count = 1;
                
                #[allow(clippy::while_let_on_iterator)]
                while let Some(ch) = chars.next() {
                    if ch == '{' {
                        brace_count += 1;
                        expr_text.push(ch);
                    } else if ch == '}' {
                        brace_count -= 1;
                        if brace_count == 0 {
                            break;
                        }
                        expr_text.push(ch);
                    } else {
                        expr_text.push(ch);
                    }
                }
                
                if brace_count > 0 {
                    bail!("Unclosed interpolation expression in string");
                }
                
                // Parse the expression
                let mut expr_parser = Parser::new(&expr_text);
                let expr = expr_parser.parse_expr()?;
                parts.push(StringPart::Expr(Box::new(expr)));
            } else {
                current_text.push(ch);
            }
        }
        
        // Add remaining text
        if !current_text.is_empty() {
            parts.push(StringPart::Text(current_text));
        }
        
        Ok(parts)
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

    #[test]
    fn test_parse_method_call() {
        let mut parser = Parser::new("x.foo()");
        let expr = parser.parse().expect("Failed to parse method call");
        match expr.kind {
            ExprKind::MethodCall { method, args, .. } => {
                assert_eq!(method, "foo");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected method call"),
        }

        let mut parser = Parser::new("list.push(42)");
        let expr = parser.parse().expect("Failed to parse method call with args");
        match expr.kind {
            ExprKind::MethodCall { method, args, .. } => {
                assert_eq!(method, "push");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected method call"),
        }

        let mut parser = Parser::new("obj.method(1, 2, 3)");
        let expr = parser.parse().expect("Failed to parse method call with multiple args");
        match expr.kind {
            ExprKind::MethodCall { method, args, .. } => {
                assert_eq!(method, "method");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("Expected method call"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        // Test simple lambda
        let mut parser = Parser::new("|x| x + 1");
        let expr = parser.parse().expect("Failed to parse simple lambda");
        match expr.kind {
            ExprKind::Lambda { params, body } => {
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name, "x");
                // Check body is x + 1
                match body.kind {
                    ExprKind::Binary { op, .. } => {
                        assert_eq!(op, BinaryOp::Add);
                    }
                    _ => panic!("Expected binary expression in lambda body"),
                }
            }
            _ => panic!("Expected lambda expression"),
        }

        // Test lambda with multiple parameters
        let mut parser = Parser::new("|x, y| x * y");
        let expr = parser.parse().expect("Failed to parse multi-param lambda");
        match expr.kind {
            ExprKind::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert_eq!(params[1].name, "y");
            }
            _ => panic!("Expected lambda expression"),
        }

        // Test lambda with no parameters
        let mut parser = Parser::new("|| 42");
        let expr = parser.parse().expect("Failed to parse no-param lambda");
        match expr.kind {
            ExprKind::Lambda { params, body } => {
                assert_eq!(params.len(), 0);
                match body.kind {
                    ExprKind::Literal(Literal::Integer(i)) => {
                        assert_eq!(i, 42);
                    }
                    _ => panic!("Expected literal in lambda body"),
                }
            }
            _ => panic!("Expected lambda expression"),
        }

        // Test lambda with typed parameters
        let mut parser = Parser::new("|x: i32, y: f64| x + y");
        let expr = parser.parse().expect("Failed to parse typed lambda");
        match expr.kind {
            ExprKind::Lambda { params, .. } => {
                assert_eq!(params.len(), 2);
                assert_eq!(params[0].name, "x");
                assert!(matches!(params[0].ty.kind, TypeKind::Named(ref s) if s == "i32"));
                assert_eq!(params[1].name, "y");
                assert!(matches!(params[1].ty.kind, TypeKind::Named(ref s) if s == "f64"));
            }
            _ => panic!("Expected lambda expression"),
        }
    }

    #[test]
    fn test_parse_dataframe() {
        // Test simple DataFrame
        let mut parser = Parser::new("df![name, age; \"Alice\", 30; \"Bob\", 25]");
        let expr = parser.parse().expect("Failed to parse DataFrame");
        match expr.kind {
            ExprKind::DataFrame { columns, rows } => {
                assert_eq!(columns.len(), 2);
                assert_eq!(columns[0], "name");
                assert_eq!(columns[1], "age");
                assert_eq!(rows.len(), 2);
                assert_eq!(rows[0].len(), 2);
                assert_eq!(rows[1].len(), 2);
            }
            _ => panic!("Expected DataFrame expression"),
        }

        // Test empty DataFrame
        let mut parser = Parser::new("df![]");
        let expr = parser.parse().expect("Failed to parse empty DataFrame");
        match expr.kind {
            ExprKind::DataFrame { columns, rows } => {
                assert_eq!(columns.len(), 0);
                assert_eq!(rows.len(), 0);
            }
            _ => panic!("Expected DataFrame expression"),
        }

        // Test DataFrame with single column
        let mut parser = Parser::new("df![values; 1; 2; 3]");
        let expr = parser.parse().expect("Failed to parse single column DataFrame");
        match expr.kind {
            ExprKind::DataFrame { columns, rows } => {
                assert_eq!(columns.len(), 1);
                assert_eq!(columns[0], "values");
                assert_eq!(rows.len(), 3);
            }
            _ => panic!("Expected DataFrame expression"),
        }
    }

    #[test]
    fn test_parse_try_operator() {
        // Test simple try
        let mut parser = Parser::new("foo()?");
        let expr = parser.parse().expect("Failed to parse try operator");
        match expr.kind {
            ExprKind::Try { expr } => {
                match expr.kind {
                    ExprKind::Call { .. } => {},
                    _ => panic!("Expected call expression inside try"),
                }
            }
            _ => panic!("Expected try expression"),
        }

        // Test chained try (use parentheses to avoid SafeNav token)
        let mut parser = Parser::new("(foo()?).bar()?");
        let expr = parser.parse().expect("Failed to parse chained try");
        match expr.kind {
            ExprKind::Try { .. } => {},
            _ => panic!("Expected try expression"),
        }

        // Test try with method call
        let mut parser = Parser::new("x.method()?");
        let expr = parser.parse().expect("Failed to parse try with method");
        match expr.kind {
            ExprKind::Try { expr } => {
                match expr.kind {
                    ExprKind::MethodCall { .. } => {},
                    _ => panic!("Expected method call inside try"),
                }
            }
            _ => panic!("Expected try expression"),
        }
    }

    #[test]
    fn test_parse_while() {
        // Test simple while loop
        let mut parser = Parser::new("while x < 10 { println(x) }");
        let expr = parser.parse().expect("Failed to parse while loop");
        match expr.kind {
            ExprKind::While { condition, body } => {
                // Check condition is comparison
                match condition.kind {
                    ExprKind::Binary { op, .. } => {
                        assert_eq!(op, BinaryOp::Less);
                    }
                    _ => panic!("Expected binary comparison in while condition"),
                }
                // Check body is block
                match body.kind {
                    ExprKind::Block(_) => {},
                    _ => panic!("Expected block in while body"),
                }
            }
            _ => panic!("Expected while expression"),
        }

        // Test while with boolean literal
        let mut parser = Parser::new("while true { println(\"loop\") }");
        let expr = parser.parse().expect("Failed to parse infinite while");
        match expr.kind {
            ExprKind::While { condition, .. } => {
                match condition.kind {
                    ExprKind::Literal(Literal::Bool(true)) => {},
                    _ => panic!("Expected true literal in condition"),
                }
            }
            _ => panic!("Expected while expression"),
        }
    }

    #[test]
    fn test_parse_struct_definition() {
        // Simple struct
        let mut parser = Parser::new("struct Point { x: i32, y: i32 }");
        let expr = parser.parse().expect("Failed to parse struct");
        match expr.kind {
            ExprKind::Struct { name, fields } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "x");
                assert_eq!(fields[1].name, "y");
                assert!(!fields[0].is_pub);
                assert!(!fields[1].is_pub);
            }
            _ => panic!("Expected struct definition"),
        }

        // Struct with public fields
        let mut parser = Parser::new("struct Person { pub name: String, pub age: i32 }");
        let expr = parser.parse().expect("Failed to parse struct with pub fields");
        match expr.kind {
            ExprKind::Struct { name, fields } => {
                assert_eq!(name, "Person");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name, "name");
                assert!(fields[0].is_pub);
                assert_eq!(fields[1].name, "age");
                assert!(fields[1].is_pub);
            }
            _ => panic!("Expected struct definition"),
        }

        // Empty struct
        let mut parser = Parser::new("struct Empty { }");
        let expr = parser.parse().expect("Failed to parse empty struct");
        match expr.kind {
            ExprKind::Struct { name, fields } => {
                assert_eq!(name, "Empty");
                assert_eq!(fields.len(), 0);
            }
            _ => panic!("Expected struct definition"),
        }
    }

    #[test]
    fn test_parse_struct_literal() {
        // Simple struct instantiation
        let mut parser = Parser::new("Point { x: 10, y: 20 }");
        let expr = parser.parse().expect("Failed to parse struct literal");
        match expr.kind {
            ExprKind::StructLiteral { name, fields } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert_eq!(fields[1].0, "y");
                match &fields[0].1.kind {
                    ExprKind::Literal(Literal::Integer(10)) => {},
                    _ => panic!("Expected integer literal for x"),
                }
                match &fields[1].1.kind {
                    ExprKind::Literal(Literal::Integer(20)) => {},
                    _ => panic!("Expected integer literal for y"),
                }
            }
            _ => panic!("Expected struct literal"),
        }

        // Struct literal with expressions
        let mut parser = Parser::new("Person { name: \"Alice\", age: 25 + 5 }");
        let expr = parser.parse().expect("Failed to parse struct literal with expressions");
        match expr.kind {
            ExprKind::StructLiteral { name, fields } => {
                assert_eq!(name, "Person");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "name");
                match &fields[0].1.kind {
                    ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "Alice"),
                    _ => panic!("Expected string literal for name"),
                }
                assert_eq!(fields[1].0, "age");
                match &fields[1].1.kind {
                    ExprKind::Binary { op, .. } => assert_eq!(*op, BinaryOp::Add),
                    _ => panic!("Expected binary expression for age"),
                }
            }
            _ => panic!("Expected struct literal"),
        }
    }

    #[test]
    fn test_parse_trait() {
        // Simple trait
        let mut parser = Parser::new("trait Display { fun show(self) -> String }");
        let expr = parser.parse().expect("Failed to parse trait");
        match expr.kind {
            ExprKind::Trait { name, methods } => {
                assert_eq!(name, "Display");
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "show");
                assert!(methods[0].body.is_none()); // No default implementation
            }
            _ => panic!("Expected trait definition"),
        }

        // Trait with default implementation
        let mut parser = Parser::new("trait Greet { fun hello(self) { println(\"Hello\") } }");
        let expr = parser.parse().expect("Failed to parse trait with default");
        match expr.kind {
            ExprKind::Trait { name, methods } => {
                assert_eq!(name, "Greet");
                assert!(methods[0].body.is_some()); // Has default implementation
            }
            _ => panic!("Expected trait definition"),
        }

        // Trait with multiple methods
        let mut parser = Parser::new("trait Math { fun add(self, x: i32) -> i32; fun sub(self, x: i32) -> i32 }");
        let expr = parser.parse().expect("Failed to parse trait with multiple methods");
        match expr.kind {
            ExprKind::Trait { methods, .. } => {
                assert_eq!(methods.len(), 2);
                assert_eq!(methods[0].name, "add");
                assert_eq!(methods[1].name, "sub");
            }
            _ => panic!("Expected trait definition"),
        }
    }

    #[test]
    fn test_parse_impl() {
        // Inherent impl
        let mut parser = Parser::new("impl Point { fun distance(self) -> f64 { 0.0 } }");
        let expr = parser.parse().expect("Failed to parse inherent impl");
        match expr.kind {
            ExprKind::Impl { trait_name, for_type, methods } => {
                assert!(trait_name.is_none());
                assert_eq!(for_type, "Point");
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "distance");
            }
            _ => panic!("Expected impl block"),
        }

        // Trait impl
        let mut parser = Parser::new("impl Display for Point { fun show(self) -> String { \"Point\" } }");
        let expr = parser.parse().expect("Failed to parse trait impl");
        match expr.kind {
            ExprKind::Impl { trait_name, for_type, methods } => {
                assert_eq!(trait_name, Some("Display".to_string()));
                assert_eq!(for_type, "Point");
                assert_eq!(methods.len(), 1);
                assert_eq!(methods[0].name, "show");
            }
            _ => panic!("Expected impl block"),
        }

        // Multiple methods in impl
        let mut parser = Parser::new("impl Math for Calculator { fun add(self, x: i32) -> i32 { x } fun sub(self, x: i32) -> i32 { x } }");
        let expr = parser.parse().expect("Failed to parse impl with multiple methods");
        match expr.kind {
            ExprKind::Impl { methods, .. } => {
                assert_eq!(methods.len(), 2);
                assert_eq!(methods[0].name, "add");
                assert_eq!(methods[1].name, "sub");
            }
            _ => panic!("Expected impl block"),
        }
    }

    #[test]
    fn test_parse_async() {
        // Async function
        let mut parser = Parser::new("async fun fetch() -> String { \"data\" }");
        let expr = parser.parse().expect("Failed to parse async function");
        match expr.kind {
            ExprKind::Function { name, is_async, .. } => {
                assert_eq!(name, "fetch");
                assert!(is_async);
            }
            _ => panic!("Expected async function"),
        }

        // Await expression
        let mut parser = Parser::new("await fetch()");
        let expr = parser.parse().expect("Failed to parse await");
        match expr.kind {
            ExprKind::Await { expr } => {
                match expr.kind {
                    ExprKind::Call { .. } => {},
                    _ => panic!("Expected call in await"),
                }
            }
            _ => panic!("Expected await expression"),
        }

        // Async block
        let mut parser = Parser::new("async { fetch() }");
        let expr = parser.parse().expect("Failed to parse async block");
        // Async blocks are parsed as lambdas for now
        match expr.kind {
            ExprKind::Lambda { params, .. } => {
                assert_eq!(params.len(), 0);
            }
            _ => panic!("Expected lambda (async block)"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        // Simple field access
        let mut parser = Parser::new("point.x");
        let expr = parser.parse().expect("Failed to parse field access");
        match expr.kind {
            ExprKind::FieldAccess { object, field } => {
                match object.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "point"),
                    _ => panic!("Expected identifier as object"),
                }
                assert_eq!(field, "x");
            }
            _ => panic!("Expected field access"),
        }

        // Chained field access
        let mut parser = Parser::new("obj.field1.field2");
        let expr = parser.parse().expect("Failed to parse chained field access");
        match expr.kind {
            ExprKind::FieldAccess { object, field } => {
                assert_eq!(field, "field2");
                match object.kind {
                    ExprKind::FieldAccess { object: inner_obj, field: inner_field } => {
                        assert_eq!(inner_field, "field1");
                        match inner_obj.kind {
                            ExprKind::Identifier(name) => assert_eq!(name, "obj"),
                            _ => panic!("Expected identifier at base"),
                        }
                    }
                    _ => panic!("Expected nested field access"),
                }
            }
            _ => panic!("Expected field access"),
        }

        // Field access with method call
        let mut parser = Parser::new("point.x.abs()");
        let expr = parser.parse().expect("Failed to parse field access with method");
        match expr.kind {
            ExprKind::MethodCall { receiver, method, args } => {
                assert_eq!(method, "abs");
                assert_eq!(args.len(), 0);
                match receiver.kind {
                    ExprKind::FieldAccess { object, field } => {
                        assert_eq!(field, "x");
                        match object.kind {
                            ExprKind::Identifier(name) => assert_eq!(name, "point"),
                            _ => panic!("Expected identifier"),
                        }
                    }
                    _ => panic!("Expected field access as receiver"),
                }
            }
            _ => panic!("Expected method call"),
        }
    }

    #[test]
    fn test_parse_list_comprehension_basic() {
        let mut parser = Parser::new("[x * 2 for x in numbers]");
        let expr = parser.parse_expr().unwrap();

        match expr.kind {
            ExprKind::ListComprehension { element, variable, iterable, condition } => {
                assert_eq!(variable, "x");
                assert!(condition.is_none());
                
                match element.kind {
                    ExprKind::Binary { left, op, right } => {
                        assert_eq!(op, BinaryOp::Multiply);
                        match left.kind {
                            ExprKind::Identifier(name) => assert_eq!(name, "x"),
                            _ => panic!("Expected identifier in element"),
                        }
                        match right.kind {
                            ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 2),
                            _ => panic!("Expected integer literal"),
                        }
                    }
                    _ => panic!("Expected binary expression in element"),
                }

                match iterable.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "numbers"),
                    _ => panic!("Expected identifier in iterable"),
                }
            }
            _ => panic!("Expected list comprehension"),
        }
    }

    #[test]
    fn test_parse_list_comprehension_with_condition() {
        let mut parser = Parser::new("[x for x in numbers if x > 0]");
        let expr = parser.parse_expr().unwrap();

        match expr.kind {
            ExprKind::ListComprehension { element, variable, iterable, condition } => {
                assert_eq!(variable, "x");
                assert!(condition.is_some());
                
                match element.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "x"),
                    _ => panic!("Expected identifier in element"),
                }

                match iterable.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "numbers"),
                    _ => panic!("Expected identifier in iterable"),
                }

                if let Some(cond) = condition {
                    match cond.kind {
                        ExprKind::Binary { left, op, right } => {
                            assert_eq!(op, BinaryOp::Greater);
                            match left.kind {
                                ExprKind::Identifier(name) => assert_eq!(name, "x"),
                                _ => panic!("Expected identifier in condition"),
                            }
                            match right.kind {
                                ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 0),
                                _ => panic!("Expected integer literal"),
                            }
                        }
                        _ => panic!("Expected binary expression in condition"),
                    }
                }
            }
            _ => panic!("Expected list comprehension"),
        }
    }

    #[test]
    fn test_parse_list_comprehension_complex() {
        let mut parser = Parser::new("[x.value * y for x in items if x.active]");
        let expr = parser.parse_expr().unwrap();

        match expr.kind {
            ExprKind::ListComprehension { element, variable, iterable, condition } => {
                assert_eq!(variable, "x");
                assert!(condition.is_some());

                match iterable.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "items"),
                    _ => panic!("Expected identifier in iterable"),
                }
                
                match element.kind {
                    ExprKind::Binary { left, op, right } => {
                        assert_eq!(op, BinaryOp::Multiply);
                        match left.kind {
                            ExprKind::FieldAccess { object, field } => {
                                assert_eq!(field, "value");
                                match object.kind {
                                    ExprKind::Identifier(name) => assert_eq!(name, "x"),
                                    _ => panic!("Expected identifier in field access"),
                                }
                            }
                            _ => panic!("Expected field access in left side"),
                        }
                        match right.kind {
                            ExprKind::Identifier(name) => assert_eq!(name, "y"),
                            _ => panic!("Expected identifier in right side"),
                        }
                    }
                    _ => panic!("Expected binary expression in element"),
                }

                if let Some(cond) = condition {
                    match cond.kind {
                        ExprKind::FieldAccess { object, field } => {
                            assert_eq!(field, "active");
                            match object.kind {
                                ExprKind::Identifier(name) => assert_eq!(name, "x"),
                                _ => panic!("Expected identifier in condition field access"),
                            }
                        }
                        _ => panic!("Expected field access in condition"),
                    }
                }
            }
            _ => panic!("Expected list comprehension"),
        }
    }

    #[test]
    fn test_parse_regular_list_vs_comprehension() {
        // Test that regular lists still parse correctly
        let mut parser = Parser::new("[1, 2, 3]");
        let expr = parser.parse_expr().unwrap();

        match expr.kind {
            ExprKind::List(items) => {
                assert_eq!(items.len(), 3);
                for (i, item) in items.iter().enumerate() {
                    match item.kind {
                        #[allow(clippy::cast_possible_wrap)] // Test data small enough for cast
                        ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, (i + 1) as i64),
                        _ => panic!("Expected integer literal"),
                    }
                }
            }
            _ => panic!("Expected regular list"),
        }

        // Test that list with 'for' name gets parsed correctly with proper spacing  
        let mut parser = Parser::new("[x + 1, y * 2]");
        let expr = parser.parse_expr().unwrap();

        match expr.kind {
            ExprKind::List(items) => {
                assert_eq!(items.len(), 2);
                // Should parse arithmetic expressions correctly
                for item in &items {
                    match item.kind {
                        ExprKind::Binary { .. } => {},
                        _ => panic!("Expected binary expression"),
                    }
                }
            }
            _ => panic!("Expected regular list"),
        }
    }

    #[test]
    fn test_parse_string_interpolation_simple() {
        let mut parser = Parser::new("\"Hello, {name}!\"");
        let expr = parser.parse().expect("Failed to parse string interpolation");
        match expr.kind {
            ExprKind::StringInterpolation { parts } => {
                assert_eq!(parts.len(), 3);
                match &parts[0] {
                    StringPart::Text(text) => assert_eq!(text, "Hello, "),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
                match &parts[1] {
                    StringPart::Expr(expr) => match &expr.kind {
                        ExprKind::Identifier(name) => assert_eq!(name, "name"),
                        _ => panic!("Expected identifier"),
                    },
                    StringPart::Text(_) => panic!("Expected expression part"),
                }
                match &parts[2] {
                    StringPart::Text(text) => assert_eq!(text, "!"),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected string interpolation"),
        }
    }

    #[test]
    fn test_parse_string_interpolation_complex() {
        let mut parser = Parser::new("\"Result: {x + y} (calculated at {time})\"");
        let expr = parser.parse().expect("Failed to parse complex interpolation");
        match expr.kind {
            ExprKind::StringInterpolation { parts } => {
                assert_eq!(parts.len(), 5);
                match &parts[0] {
                    StringPart::Text(text) => assert_eq!(text, "Result: "),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
                match &parts[1] {
                    StringPart::Expr(expr) => match &expr.kind {
                        ExprKind::Binary { .. } => {}, // x + y
                        _ => panic!("Expected binary expression"),
                    },
                    StringPart::Text(_) => panic!("Expected expression part"),
                }
                match &parts[2] {
                    StringPart::Text(text) => assert_eq!(text, " (calculated at "),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
                match &parts[3] {
                    StringPart::Expr(expr) => match &expr.kind {
                        ExprKind::Identifier(name) => assert_eq!(name, "time"),
                        _ => panic!("Expected identifier"),
                    },
                    StringPart::Text(_) => panic!("Expected expression part"),
                }
                match &parts[4] {
                    StringPart::Text(text) => assert_eq!(text, ")"),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
            }
            _ => panic!("Expected string interpolation"),
        }
    }

    #[test]
    fn test_parse_string_interpolation_escaped_braces() {
        let mut parser = Parser::new("\"Value: {{static}} and {dynamic}\"");
        let expr = parser.parse().expect("Failed to parse escaped braces");
        match expr.kind {
            ExprKind::StringInterpolation { parts } => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    StringPart::Text(text) => assert_eq!(text, "Value: {static} and "),
                    StringPart::Expr(_) => panic!("Expected text part"),
                }
                match &parts[1] {
                    StringPart::Expr(expr) => match &expr.kind {
                        ExprKind::Identifier(name) => assert_eq!(name, "dynamic"),
                        _ => panic!("Expected identifier"),
                    },
                    StringPart::Text(_) => panic!("Expected expression part"),
                }
            }
            _ => panic!("Expected string interpolation"),
        }
    }

    #[test]
    fn test_parse_string_no_interpolation() {
        let mut parser = Parser::new("\"Simple string\"");
        let expr = parser.parse().expect("Failed to parse simple string");
        match expr.kind {
            ExprKind::Literal(Literal::String(s)) => {
                assert_eq!(s, "Simple string");
            }
            _ => panic!("Expected string literal"),
        }
    }
}
