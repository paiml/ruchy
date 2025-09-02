//! Basic expression parsing - minimal version with only used functions

use super::{ParserState, *};

pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input - expected expression");
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
        Token::FString(template) => {
            state.tokens.advance();
            // Parse f-string template into parts
            // For now, treat it as a simple string with placeholders
            let parts = vec![StringPart::Text(template)];
            Ok(Expr::new(ExprKind::StringInterpolation {
                parts,
            }, span_clone))
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
            // Check for fat arrow lambda: x => x * 2
            if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                state.tokens.advance(); // consume =>
                let body = Box::new(super::parse_expr_recursive(state)?);
                let params = vec![Param {
                    pattern: Pattern::Identifier(name),
                    ty: Type {
                        kind: TypeKind::Named("_".to_string()),
                        span: span_clone,
                    },
                    default_value: None,
                    is_mutable: false,
                    span: span_clone,
                }];
                Ok(Expr::new(ExprKind::Lambda {
                    params,
                    body,
                }, span_clone))
            } else {
                Ok(Expr::new(ExprKind::Identifier(name), span_clone))
            }
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("_".to_string()), span_clone))
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
        Token::LeftBrace => {
            // Parse block - do NOT advance token, let collections parser handle it
            super::collections::parse_block(state)
        }
        Token::Let => {
            // Parse let statement/expression
            parse_let_statement(state)
        }
        Token::If => {
            // Parse if expression
            parse_if_expression(state)
        }
        Token::Match => {
            // Parse match expression
            parse_match_expression(state)
        }
        Token::While => {
            // Parse while loop
            parse_while_loop(state)
        }
        Token::For => {
            // Parse for loop
            parse_for_loop(state)
        }
        Token::LeftBracket => {
            // Parse list literal
            parse_list_literal(state)
        }
        Token::Pipe => {
            // Parse lambda expression |x| x + 1
            parse_lambda_expression(state)
        }
        Token::OrOr => {
            // Parse lambda with no params: || expr
            parse_lambda_no_params(state)
        }
        Token::LeftParen => {
            state.tokens.advance();
            // Check for unit type ()
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                state.tokens.advance();
                Ok(Expr::new(ExprKind::Literal(Literal::Unit), span_clone))
            } else {
                // Parse first expression
                let first_expr = super::parse_expr_recursive(state)?;
                
                // Check if we have a comma (tuple) or just closing paren (grouped expr)
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    // This is a tuple, parse remaining elements
                    let mut elements = vec![first_expr];
                    
                    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance(); // consume comma
                        
                        // Check for trailing comma before closing paren
                        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                            break;
                        }
                        
                        elements.push(super::parse_expr_recursive(state)?);
                    }
                    
                    state.tokens.expect(&Token::RightParen)?;
                    Ok(Expr::new(ExprKind::Tuple(elements), span_clone))
                } else {
                    // Just a grouped expression
                    state.tokens.expect(&Token::RightParen)?;
                    
                    // Check if this is a lambda: (x) => expr
                    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                        parse_lambda_from_expr(state, first_expr, span_clone)
                    } else {
                        Ok(first_expr)
                    }
                }
            }
        }
        Token::Struct => {
            // Parse struct definition
            parse_struct_definition(state)
        }
        Token::Trait => {
            // Parse trait definition
            parse_trait_definition(state)
        }
        Token::Impl => {
            // Parse impl block
            parse_impl_block(state)
        }
        Token::Import => {
            // Parse import statement
            parse_import_statement(state)
        }
        Token::Use => {
            // Parse use statement
            parse_use_statement(state)
        }
        Token::DataFrame => {
            // Parse dataframe literal df![...]
            parse_dataframe_literal(state)
        }
        Token::Actor => {
            // Parse actor definition
            parse_actor_definition(state)
        }
        Token::Pub => {
            // Parse public declaration (function, struct, etc.)
            state.tokens.advance();
            // Get the next token to determine what follows pub
            let mut expr = parse_prefix(state)?;
            // Mark the expression as public if it supports it
            match &mut expr.kind {
                ExprKind::Function { is_pub, .. } => *is_pub = true,
                ExprKind::Struct { is_pub, .. } => *is_pub = true,
                ExprKind::Trait { is_pub, .. } => *is_pub = true,
                ExprKind::Impl { is_pub, .. } => *is_pub = true,
                _ => {} // Other expressions don't have is_pub
            }
            Ok(expr)
        }
        Token::Break => {
            state.tokens.advance();
            // Optional label
            let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let label = Some(name.clone());
                state.tokens.advance();
                label
            } else {
                None
            };
            Ok(Expr::new(ExprKind::Break { label }, span_clone))
        }
        Token::Continue => {
            state.tokens.advance();
            // Optional label
            let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let label = Some(name.clone());
                state.tokens.advance();
                label
            } else {
                None
            };
            Ok(Expr::new(ExprKind::Continue { label }, span_clone))
        }
        Token::Return => {
            state.tokens.advance();
            // Optional return value
            let value = if matches!(state.tokens.peek(), Some((Token::Semicolon, _)) | None) {
                None
            } else {
                Some(Box::new(super::parse_expr_recursive(state)?))
            };
            Ok(Expr::new(ExprKind::Return { value }, span_clone))
        }
        Token::Enum => {
            // Parse enum definition
            parse_enum_definition(state)
        }
        Token::Some => {
            // Parse Some(..) constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Some".to_string()), span_clone))
        }
        Token::None => {
            // Parse None constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("None".to_string()), span_clone))
        }
        Token::Ok => {
            // Parse Ok(..) constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Ok".to_string()), span_clone))
        }
        Token::Err => {
            // Parse Err(..) constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Err".to_string()), span_clone))
        }
        Token::Result => {
            // Parse Result type constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Result".to_string()), span_clone))
        }
        Token::Option => {
            // Parse Option type constructor
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("Option".to_string()), span_clone))
        }
        _ => bail!("Unexpected token: {:?}", token_clone),
    }
}

/// Parse let statement: let [mut] name [: type] = value [in body]
fn parse_let_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Let)?;
    
    // Check for optional 'mut' keyword
    let is_mutable = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    };
    
    // Parse variable name or destructuring pattern
    let pattern = match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Pattern::Identifier(name)
        }
        Some((Token::LeftParen, _)) => {
            // Parse tuple destructuring: (x, y) = (1, 2)
            parse_tuple_pattern(state)?
        }
        Some((Token::LeftBracket, _)) => {
            // Parse list destructuring: [a, b] = [1, 2]
            parse_list_pattern(state)?
        }
        _ => bail!("Expected identifier or pattern after 'let{}'", if is_mutable { " mut" } else { "" })
    };
    
    // Parse optional type annotation
    let type_annotation = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Some(super::utils::parse_type(state)?)
    } else {
        None
    };
    
    // Parse '=' token
    state.tokens.expect(&Token::Equal)?;
    
    // Parse value expression
    let value = Box::new(super::parse_expr_recursive(state)?);
    
    // Parse optional 'in' clause for let expressions
    let body = if matches!(state.tokens.peek(), Some((Token::In, _))) {
        state.tokens.advance(); // consume 'in'
        Box::new(super::parse_expr_recursive(state)?)
    } else {
        // For let statements (no 'in'), body is unit
        Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
    };
    
    // Convert pattern to name for simple cases
    let _name = match &pattern {
        Pattern::Identifier(name) => name.clone(),
        Pattern::Tuple(patterns) => {
            // For now, use the first identifier from tuple destructuring
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        Pattern::List(patterns) => {
            // For now, use the first identifier from list destructuring
            patterns.first()
                .and_then(|p| match p {
                    Pattern::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "__destructured".to_string())
        }
        _ => "__destructured".to_string(), // Placeholder for complex patterns
    };

    let end_span = body.span;
    Ok(Expr::new(
        ExprKind::Let {
            name: match &pattern {
                Pattern::Identifier(name) => name.clone(),
                Pattern::Tuple(_patterns) => {
                    // For tuple destructuring, use LetPattern variant instead
                    return Ok(Expr::new(
                        ExprKind::LetPattern {
                            pattern,
                            type_annotation,
                            value,
                            body,
                            is_mutable,
                        },
                        start_span.merge(end_span),
                    ));
                }
                Pattern::List(_patterns) => {
                    // For list destructuring, use LetPattern variant instead
                    return Ok(Expr::new(
                        ExprKind::LetPattern {
                            pattern,
                            type_annotation,
                            value,
                            body,
                            is_mutable,
                        },
                        start_span.merge(end_span),
                    ));
                }
                _ => {
                    // For complex patterns, use LetPattern variant instead
                    return Ok(Expr::new(
                        ExprKind::LetPattern {
                            pattern,
                            type_annotation,
                            value,
                            body,
                            is_mutable,
                        },
                        start_span.merge(end_span),
                    ));
                }
            },
            type_annotation,
            value,
            body,
            is_mutable,
        },
        start_span.merge(end_span),
    ))
}

fn parse_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut patterns = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            patterns.push(Pattern::Identifier(name.clone()));
            state.tokens.advance();
        } else {
            bail!("Expected identifier in tuple pattern");
        }
        
        // Only consume comma if not at end
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
            // If we hit RightParen after comma, break (trailing comma case)
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                break;
            }
        } else if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            bail!("Expected ',' or ')' in tuple pattern");
        }
    }
    
    state.tokens.expect(&Token::RightParen)?;
    Ok(Pattern::Tuple(patterns))
}

fn parse_list_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftBracket)?;
    let mut patterns = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            patterns.push(Pattern::Identifier(name.clone()));
            state.tokens.advance();
        } else {
            bail!("Expected identifier in list pattern");
        }
        
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Pattern::List(patterns))
}

/// Parse if expression: if condition { `then_branch` } [else { `else_branch` }]
fn parse_if_expression(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::If)?;
    
    // Parse condition with better error context
    let condition = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected condition after 'if': {}", e))?);
    
    // Parse then branch (expect block) with better error context
    let then_branch = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected body after if condition, typically {{ ... }}: {}", e))?);
    
    // Parse optional else branch
    let else_branch = if matches!(state.tokens.peek(), Some((Token::Else, _))) {
        state.tokens.advance(); // consume 'else'
        Some(Box::new(super::parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected body after 'else', typically {{ ... }}: {}", e))?))
    } else {
        None
    };
    
    Ok(Expr::new(
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        },
        start_span,
    ))
}

/// Parse match expression: match expr { pattern => result, ... }
/// Complexity target: <10 (using helper functions for TDG compliance)
fn parse_match_expression(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Match)?;
    
    // Parse the expression to match on
    let expr = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected expression after 'match': {}", e))?);
    
    // Expect opening brace for match arms
    state.tokens.expect(&Token::LeftBrace)
        .map_err(|_| anyhow::anyhow!("Expected '{{' after match expression"))?;
    
    // Parse match arms
    let arms = parse_match_arms(state)?;
    
    // Expect closing brace
    state.tokens.expect(&Token::RightBrace)
        .map_err(|_| anyhow::anyhow!("Expected '}}' after match arms"))?;
    
    Ok(Expr::new(
        ExprKind::Match { expr, arms },
        start_span,
    ))
}

/// Parse match arms with low complexity (helper function for TDG compliance)
fn parse_match_arms(state: &mut ParserState) -> Result<Vec<MatchArm>> {
    let mut arms = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _)) | None) {
        // Parse single arm
        let arm = parse_single_match_arm(state)?;
        arms.push(arm);
        
        // Optional comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
        
        // Check if we're done
        if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            break;
        }
    }
    
    if arms.is_empty() {
        bail!("Match expression must have at least one arm");
    }
    
    Ok(arms)
}

/// Parse a single match arm: pattern [if guard] => expr
/// Complexity: <5 (simple sequential parsing)
fn parse_single_match_arm(state: &mut ParserState) -> Result<MatchArm> {
    let start_span = state.tokens.peek().map(|(_, s)| *s)
        .unwrap_or_default();
    
    // Parse pattern
    let pattern = parse_match_pattern(state)?;
    
    // Parse optional guard (if condition)
    let guard = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance(); // consume 'if'
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };
    
    // Expect => token
    state.tokens.expect(&Token::FatArrow)
        .map_err(|_| anyhow::anyhow!("Expected '=>' in match arm"))?;
    
    // Parse result expression
    let body = Box::new(super::parse_expr_recursive(state)?);
    
    let end_span = body.span;
    
    Ok(MatchArm {
        pattern,
        guard,
        body,
        span: start_span.merge(end_span),
    })
}

/// Parse match pattern with low complexity
/// Complexity: <5 (simple pattern matching)
fn parse_match_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected pattern in match arm");
    };
    
    let pattern = match token {
        Token::Underscore => {
            state.tokens.advance();
            Pattern::Wildcard
        }
        Token::Integer(val) => {
            let val = *val;
            state.tokens.advance();
            Pattern::Literal(Literal::Integer(val))
        }
        Token::String(s) => {
            let s = s.clone();
            state.tokens.advance();
            Pattern::Literal(Literal::String(s))
        }
        Token::Bool(b) => {
            let b = *b;
            state.tokens.advance();
            Pattern::Literal(Literal::Bool(b))
        }
        Token::Some => {
            state.tokens.advance();
            // Some can be used alone or with arguments
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "Some".to_string())?
            } else {
                Pattern::Identifier("Some".to_string())
            }
        }
        Token::None => {
            state.tokens.advance();
            // None is typically used alone but can have empty parens
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "None".to_string())?
            } else {
                Pattern::Identifier("None".to_string())
            }
        }
        Token::Identifier(name) => {
            let name = name.clone();
            state.tokens.advance();
            
            // Check for enum-like patterns: Ok(x), Err(e), etc.
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, name)?
            } else {
                Pattern::Identifier(name)
            }
        }
        _ => bail!("Unexpected token in pattern: {:?}", token)
    };
    
    // Handle multiple patterns with | (or)
    if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        parse_or_pattern(state, pattern)
    } else {
        Ok(pattern)
    }
}

/// Parse constructor pattern: Some(x), Ok(value), etc.
/// Complexity: <5
fn parse_constructor_pattern(state: &mut ParserState, name: String) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    
    // Check for empty tuple (e.g., None())
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        state.tokens.advance();
        return Ok(Pattern::Identifier(name));
    }
    
    // Parse inner patterns as tuple
    let mut patterns = vec![parse_match_pattern(state)?];
    
    // Parse additional patterns if comma-separated
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            break; // trailing comma
        }
        patterns.push(parse_match_pattern(state)?);
    }
    
    state.tokens.expect(&Token::RightParen)?;
    
    // Handle constructor patterns (Some(x), Ok(val), etc.)
    // These should ideally use a proper constructor pattern type
    // For now, we'll use the appropriate pattern based on the constructor name
    
    if name == "Some" && patterns.len() == 1 {
        // Some(pattern) - use Ok variant to represent Option::Some
        Ok(Pattern::Ok(Box::new(patterns.into_iter().next().unwrap())))
    } else if name == "None" && patterns.is_empty() {
        // None - just an identifier
        Ok(Pattern::Identifier("None".to_string()))
    } else if patterns.len() == 1 {
        // Single argument constructor - for simplicity, use the inner pattern
        Ok(patterns.into_iter().next().unwrap())
    } else {
        // Multiple arguments - use tuple pattern
        Ok(Pattern::Tuple(patterns))
    }
}

/// Parse or-pattern: pattern | pattern | ...
/// Complexity: <5
fn parse_or_pattern(state: &mut ParserState, first: Pattern) -> Result<Pattern> {
    let mut patterns = vec![first];
    
    while matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        state.tokens.advance(); // consume '|'
        
        // Need to parse the next pattern without recursing into or-patterns again
        let next = parse_single_pattern(state)?;
        patterns.push(next);
    }
    
    // Use the Or pattern variant for multiple alternatives
    if patterns.len() == 1 {
        Ok(patterns.into_iter().next().unwrap())
    } else {
        Ok(Pattern::Or(patterns))
    }
}

/// Parse a single pattern without checking for | (helper to avoid recursion)
/// Complexity: <5
fn parse_single_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected pattern");
    };
    
    match token {
        Token::Underscore => {
            state.tokens.advance();
            Ok(Pattern::Wildcard)
        }
        Token::Integer(val) => {
            let val = *val;
            state.tokens.advance();
            Ok(Pattern::Literal(Literal::Integer(val)))
        }
        Token::String(s) => {
            let s = s.clone();
            state.tokens.advance();
            Ok(Pattern::Literal(Literal::String(s)))
        }
        Token::Identifier(name) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        _ => bail!("Unexpected token in pattern: {:?}", token)
    }
}

/// Parse while loop: while condition { body }
/// Complexity: <5 (simple structure)
fn parse_while_loop(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::While)?;
    
    // Parse condition
    let condition = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected condition after 'while': {}", e))?);
    
    // Parse body (expect block)
    let body = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected body after while condition: {}", e))?);
    
    Ok(Expr::new(
        ExprKind::While { condition, body },
        start_span,
    ))
}

/// Parse for loop: for pattern in iterator { body }
/// Complexity: <5 (simple structure)
fn parse_for_loop(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::For)?;
    
    // Parse pattern (e.g., "i" in "for i in ...")
    let pattern = parse_for_pattern(state)?;
    
    // Expect 'in' keyword
    state.tokens.expect(&Token::In)
        .map_err(|_| anyhow::anyhow!("Expected 'in' after for pattern"))?;
    
    // Parse iterator expression
    let iterator = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected iterator after 'in': {}", e))?);
    
    // Parse body (expect block)
    let body = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected body after for iterator: {}", e))?);
    
    // Get the var name from the pattern for backward compatibility
    let var = pattern.primary_name();
    
    Ok(Expr::new(
        ExprKind::For { 
            var,
            pattern: Some(pattern), 
            iter: iterator, 
            body 
        },
        start_span,
    ))
}

/// Parse for loop pattern (simple version)
/// Complexity: <3
fn parse_for_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _)) = state.tokens.peek() else {
        bail!("Expected pattern in for loop");
    };
    
    match token {
        Token::Identifier(name) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Pattern::Wildcard)
        }
        Token::LeftParen => {
            // Parse tuple pattern: (x, y)
            parse_tuple_pattern(state)
        }
        Token::LeftBracket => {
            // Parse list pattern: [x, y]
            parse_list_pattern(state)
        }
        _ => bail!("Expected identifier, underscore, or destructuring pattern in for loop")
    }
}

fn parse_list_literal(state: &mut ParserState) -> Result<Expr> {
    // Parse [ expr, expr, ... ] or [expr for var in iter if cond]
    let start_span = state.tokens.expect(&Token::LeftBracket)?;
    
    // Handle empty list
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::List(vec![]), start_span));
    }
    
    // Parse first element/expression
    let first_expr = super::parse_expr_recursive(state)?;
    
    // Check if this is a list comprehension
    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        return parse_list_comprehension_body(state, first_expr, start_span);
    }
    
    // Regular list literal
    let mut elements = vec![first_expr];
    
    // Parse remaining elements
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        
        // Check for trailing comma
        if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            break;
        }
        
        elements.push(super::parse_expr_recursive(state)?);
    }
    
    state.tokens.expect(&Token::RightBracket)
        .map_err(|_| anyhow::anyhow!("Expected ']' to close list literal"))?;
    
    Ok(Expr::new(ExprKind::List(elements), start_span))
}

fn parse_list_comprehension_body(
    state: &mut ParserState,
    expr: Expr,
    start_span: Span,
) -> Result<Expr> {
    // Parse: for var in iter [if cond]
    state.tokens.expect(&Token::For)?;
    
    // Parse variable
    let var = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected variable name in list comprehension");
    };
    
    state.tokens.expect(&Token::In)?;
    
    // Parse iterator
    let iter = super::parse_expr_recursive(state)?;
    
    // Parse optional condition
    let condition = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance();
        Some(Box::new(super::parse_expr_recursive(state)?))
    } else {
        None
    };
    
    state.tokens.expect(&Token::RightBracket)?;
    
    Ok(Expr::new(
        ExprKind::ListComprehension {
            element: Box::new(expr),
            variable: var,
            iterable: Box::new(iter),
            condition,
        },
        start_span,
    ))
}

fn parse_lambda_no_params(state: &mut ParserState) -> Result<Expr> {
    // Parse || body
    let start_span = state.tokens.expect(&Token::OrOr)?;
    
    // Parse the body
    let body = Box::new(super::parse_expr_recursive(state)?);
    
    Ok(Expr::new(ExprKind::Lambda { 
        params: vec![], 
        body 
    }, start_span))
}

fn parse_lambda_from_expr(state: &mut ParserState, expr: Expr, start_span: Span) -> Result<Expr> {
    // Convert (x) => expr syntax
    state.tokens.advance(); // consume =>
    
    // Convert the expression to a parameter
    let param = match &expr.kind {
        ExprKind::Identifier(name) => Param {
            pattern: Pattern::Identifier(name.clone()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: expr.span,
            },
            default_value: None,
            is_mutable: false,
            span: expr.span,
        },
        _ => bail!("Expected identifier in lambda parameter")
    };
    
    // Parse the body
    let body = Box::new(super::parse_expr_recursive(state)?);
    
    Ok(Expr::new(ExprKind::Lambda {
        params: vec![param],
        body,
    }, start_span))
}

fn parse_lambda_expression(state: &mut ParserState) -> Result<Expr> {
    // Parse |param, param| body or |param| body
    let start_span = state.tokens.expect(&Token::Pipe)?;
    
    let mut params = Vec::new();
    
    // Parse parameters
    while !matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            params.push(Pattern::Identifier(name.clone()));
            state.tokens.advance();
            
            // Check for comma
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            }
        } else {
            bail!("Expected parameter name in lambda");
        }
    }
    
    state.tokens.expect(&Token::Pipe)
        .map_err(|_| anyhow::anyhow!("Expected '|' after lambda parameters"))?;
    
    // Parse body
    let body = Box::new(super::parse_expr_recursive(state)?);
    
    // Convert Pattern to Param for Lambda
    let params = params.into_iter().map(|p| Param {
        pattern: p,
        ty: Type {
            kind: TypeKind::Named("_".to_string()),
            span: start_span,
        },
        span: start_span,
        is_mutable: false,
        default_value: None,
    }).collect();
    
    Ok(Expr::new(ExprKind::Lambda { params, body }, start_span))
}

fn parse_struct_definition(state: &mut ParserState) -> Result<Expr> {
    // Parse struct Name<T> { field: Type, ... }
    let start_span = state.tokens.expect(&Token::Struct)?;
    
    // Get struct name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected struct name after 'struct'");
    };
    
    // Parse optional generic parameters
    let type_params = parse_optional_generics(state)?;
    
    // Parse { fields }
    state.tokens.expect(&Token::LeftBrace)?;
    
    let mut fields = Vec::new();
    
    // Parse fields
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Parse field name
        let field_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected field name in struct");
        };
        
        // Parse : Type
        state.tokens.expect(&Token::Colon)?;
        let field_type = super::utils::parse_type(state)?;
        
        fields.push((field_name, field_type));
        
        // Check for comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    // Convert to proper Struct variant with StructField
    let struct_fields = fields.into_iter().map(|(name, ty)| StructField {
        name,
        ty,
        is_pub: false,
    }).collect();
    
    Ok(Expr::new(ExprKind::Struct {
        name,
        type_params,
        fields: struct_fields,
        is_pub: false,
    }, start_span))
}

fn parse_trait_definition(state: &mut ParserState) -> Result<Expr> {
    // Parse trait Name { fun method(self) -> Type ... }
    let start_span = state.tokens.expect(&Token::Trait)?;
    
    // Get trait name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected trait name after 'trait'");
    };
    
    // Parse { methods }
    state.tokens.expect(&Token::LeftBrace)?;
    
    let mut methods = Vec::new();
    
    // Parse methods
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Expect 'fun' keyword
        state.tokens.expect(&Token::Fun)?;
        
        // Parse method name
        let method_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected method name in trait");
        };
        
        // For now, skip the rest of the method signature
        // This is a simplified implementation
        methods.push(method_name);
        
        // Skip to end of line or next method
        while !matches!(state.tokens.peek(), Some((Token::Fun | Token::RightBrace, _))) 
              && state.tokens.peek().is_some() {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    // Convert to proper Trait variant with TraitMethod
    let trait_methods = methods.into_iter().map(|name| TraitMethod {
        name,
        params: vec![],
        return_type: None,
        body: None,
        is_pub: true,
    }).collect();
    
    Ok(Expr::new(ExprKind::Trait {
        name,
        type_params: vec![],
        methods: trait_methods,
        is_pub: false,
    }, start_span))
}

fn parse_impl_block(state: &mut ParserState) -> Result<Expr> {
    // Parse impl Trait for Type { ... } or impl Type { ... }
    let start_span = state.tokens.expect(&Token::Impl)?;
    
    // Parse trait or type name
    let trait_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Some(name)
    } else {
        None
    };
    
    // Check for "for" keyword
    let type_name = if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
            let name = n.clone();
            state.tokens.advance();
            name
        } else {
            bail!("Expected type name after 'for' in impl");
        }
    } else if let Some(t) = trait_name {
        // impl Type { ... } case
        t
    } else {
        bail!("Expected type or trait name in impl");
    };
    
    // Parse { methods }
    state.tokens.expect(&Token::LeftBrace)?;
    
    // For now, parse until closing brace
    let mut depth = 1;
    while depth > 0 && state.tokens.peek().is_some() {
        match state.tokens.peek() {
            Some((Token::LeftBrace, _)) => depth += 1,
            Some((Token::RightBrace, _)) => depth -= 1,
            _ => {}
        }
        if depth > 0 {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    Ok(Expr::new(ExprKind::Impl {
        type_params: vec![],
        trait_name: None, // Simplified implementation for now
        for_type: type_name,
        methods: vec![],
        is_pub: false,
    }, start_span))
}

fn parse_import_statement(state: &mut ParserState) -> Result<Expr> {
    // Parse import path::to::module
    let start_span = state.tokens.expect(&Token::Import)?;
    
    // Parse module path
    let mut path_parts = Vec::new();
    
    // Get first identifier
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        path_parts.push(name.clone());
        state.tokens.advance();
    } else {
        bail!("Expected module path after 'import'");
    }
    
    // Parse additional path segments
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            path_parts.push(name.clone());
            state.tokens.advance();
        } else {
            bail!("Expected identifier after '::'");
        }
    }
    
    let path = path_parts.join("::");
    
    Ok(Expr::new(ExprKind::Import {
        path,
        items: vec![],
    }, start_span))
}

fn parse_use_statement(state: &mut ParserState) -> Result<Expr> {
    // Parse use path::to::Type or use path::to::{Type1, Type2}
    let start_span = state.tokens.expect(&Token::Use)?;
    
    // Parse module path
    let mut path_parts = Vec::new();
    
    // Get first identifier
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        path_parts.push(name.clone());
        state.tokens.advance();
    } else {
        bail!("Expected module path after 'use'");
    }
    
    // Parse additional path segments
    while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        
        // Check for { imports }
        if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            state.tokens.advance();
            let mut items = Vec::new();
            
            // Parse imported items
            while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                    items.push(ImportItem::Named(name.clone()));
                    state.tokens.advance();
                    
                    // Check for comma
                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    }
                } else {
                    bail!("Expected identifier in import list");
                }
            }
            
            state.tokens.expect(&Token::RightBrace)?;
            
            let path = path_parts.join("::");
            return Ok(Expr::new(ExprKind::Import { path, items }, start_span));
        } else if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            path_parts.push(name.clone());
            state.tokens.advance();
        } else {
            bail!("Expected identifier or '{{' after '::'");
        }
    }
    
    // Simple use statement (use fully::qualified::Name)
    let path = path_parts.join("::");
    let last_part = path_parts.last().unwrap().clone();
    
    Ok(Expr::new(ExprKind::Import {
        path,
        items: vec![ImportItem::Named(last_part)],
    }, start_span))
}

fn parse_dataframe_literal(state: &mut ParserState) -> Result<Expr> {
    // Parse df![...] macro syntax
    let start_span = state.tokens.expect(&Token::DataFrame)?;
    
    // Expect ! after df
    state.tokens.expect(&Token::Bang)?;
    
    // Expect [
    state.tokens.expect(&Token::LeftBracket)?;
    
    let mut columns = Vec::new();
    
    // Parse column definitions: "name" => [values]
    while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        // Parse column name (string literal or identifier)
        let col_name = match state.tokens.peek() {
            Some((Token::String(name), _)) => {
                let name = name.clone();
                state.tokens.advance();
                name
            }
            Some((Token::Identifier(name), _)) => {
                let name = name.clone();
                state.tokens.advance();
                name
            }
            _ => bail!("Expected column name (string or identifier) in dataframe")
        };
        
        // Expect =>
        state.tokens.expect(&Token::FatArrow)?;
        
        // Parse column values (list)
        let values = if matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
            parse_list_literal(state)?
        } else {
            bail!("Expected list of values after => in dataframe column");
        };
        
        columns.push((col_name, values));
        
        // Check for comma
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBracket)?;
    
    // Create a DataFrame expression with proper DataFrameColumn structs
    let df_columns = columns.into_iter().map(|(name, values)| {
        // Extract expressions from the list literal
        let value_exprs = match values.kind {
            ExprKind::List(exprs) => exprs,
            _ => vec![values], // Fallback for non-list
        };
        DataFrameColumn {
            name,
            values: value_exprs,
        }
    }).collect();
    
    Ok(Expr::new(ExprKind::DataFrame { columns: df_columns }, start_span))
}

fn parse_enum_definition(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Enum)?;
    let name = parse_enum_name(state)?;
    let type_params = parse_optional_generics(state)?;
    let variants = parse_enum_variants(state)?;
    
    Ok(Expr::new(ExprKind::Enum {
        name,
        type_params,
        variants,
        is_pub: false,
    }, start_span))
}

fn parse_enum_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Option, _)) => {
            state.tokens.advance();
            Ok("Option".to_string())
        }
        Some((Token::Result, _)) => {
            state.tokens.advance();
            Ok("Result".to_string())
        }
        _ => bail!("Expected enum name after 'enum'")
    }
}

fn parse_optional_generics(state: &mut ParserState) -> Result<Vec<String>> {
    if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        parse_generic_params(state)
    } else {
        Ok(vec![])
    }
}

fn parse_enum_variants(state: &mut ParserState) -> Result<Vec<EnumVariant>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut variants = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        variants.push(parse_single_variant(state)?);
        
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    Ok(variants)
}

fn parse_single_variant(state: &mut ParserState) -> Result<EnumVariant> {
    let variant_name = parse_variant_name(state)?;
    let fields = parse_variant_fields(state)?;
    
    Ok(EnumVariant {
        name: variant_name,
        fields,
    })
}

fn parse_variant_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::Identifier(n), _)) => {
            let name = n.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Some, _)) => {
            state.tokens.advance();
            Ok("Some".to_string())
        }
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok("None".to_string())
        }
        Some((Token::Ok, _)) => {
            state.tokens.advance();
            Ok("Ok".to_string())
        }
        Some((Token::Err, _)) => {
            state.tokens.advance();
            Ok("Err".to_string())
        }
        _ => bail!("Expected variant name in enum")
    }
}

fn parse_variant_fields(state: &mut ParserState) -> Result<Option<Vec<Type>>> {
    if !matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        return Ok(None);
    }
    
    state.tokens.advance();
    let mut field_types = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        field_types.push(super::utils::parse_type(state)?);
        
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightParen)?;
    Ok(Some(field_types))
}

fn parse_generic_params(state: &mut ParserState) -> Result<Vec<String>> {
    // Parse <T, U, ...>
    state.tokens.expect(&Token::Less)?;
    let mut params = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::Greater, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            params.push(name.clone());
            state.tokens.advance();
            
            // Check for comma
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            }
        } else {
            bail!("Expected type parameter name");
        }
    }
    
    state.tokens.expect(&Token::Greater)?;
    Ok(params)
}

fn parse_actor_definition(state: &mut ParserState) -> Result<Expr> {
    // Parse actor Name { state: fields, receive handlers }
    let start_span = state.tokens.expect(&Token::Actor)?;
    
    // Get actor name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected actor name after 'actor'");
    };
    
    // Parse { body }
    state.tokens.expect(&Token::LeftBrace)?;
    
    let mut state_fields = Vec::new();
    let mut handlers = Vec::new();
    
    // Parse actor body
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        match state.tokens.peek() {
            Some((Token::State, _)) => {
                // Parse state field with 'state' keyword
                state.tokens.advance();
                if let Some((Token::Identifier(field_name), _)) = state.tokens.peek() {
                    let field = field_name.clone();
                    state.tokens.advance();
                    
                    // Parse : Type
                    state.tokens.expect(&Token::Colon)?;
                    let field_type = super::utils::parse_type(state)?;
                    
                    // Optional = initial_value
                    let initial_value = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
                        state.tokens.advance();
                        Some(Box::new(super::parse_expr_recursive(state)?))
                    } else {
                        None
                    };
                    
                    state_fields.push((field, field_type, initial_value));
                }
            }
            Some((Token::Receive, _)) => {
                // Parse receive block: receive { handler => value, ... }
                state.tokens.advance();
                state.tokens.expect(&Token::LeftBrace)?;
                
                // Parse handler mappings
                while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
                    if let Some((Token::Identifier(handler_name), _)) = state.tokens.peek() {
                        handlers.push(handler_name.clone());
                        state.tokens.advance();
                        
                        // Skip => value for now
                        state.tokens.expect(&Token::FatArrow)?;
                        super::parse_expr_recursive(state)?; // Skip the value
                        
                        // Optional comma
                        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                            state.tokens.advance();
                        }
                    } else {
                        bail!("Expected handler name in receive block");
                    }
                }
                
                state.tokens.expect(&Token::RightBrace)?;
            }
            Some((Token::Identifier(_), _)) => {
                // Parse bare field definition: field: Type,
                if let Some((Token::Identifier(field_name), _)) = state.tokens.peek() {
                    let field = field_name.clone();
                    state.tokens.advance();
                    
                    // Parse : Type
                    state.tokens.expect(&Token::Colon)?;
                    let field_type = super::utils::parse_type(state)?;
                    
                    // Optional comma
                    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                        state.tokens.advance();
                    }
                    
                    state_fields.push((field, field_type, None));
                } else {
                    bail!("Expected field name in actor");
                }
            }
            _ => {
                // Skip unknown tokens
                state.tokens.advance();
            }
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    // Create an Actor expression with proper types
    let actor_state = state_fields.into_iter().map(|(name, ty, _init)| StructField {
        name,
        ty,
        is_pub: false,
    }).collect();
    
    // For now, create simple handlers
    let actor_handlers = handlers.into_iter().map(|name| ActorHandler {
        message_type: name,
        params: vec![],
        body: Box::new(Expr::new(ExprKind::Block(vec![]), start_span)),
    }).collect();
    
    Ok(Expr::new(ExprKind::Actor { 
        name, 
        state: actor_state,
        handlers: actor_handlers,
    }, start_span))
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