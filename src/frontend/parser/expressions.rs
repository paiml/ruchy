//! Basic expression parsing - minimal version with only used functions

use super::{ParserState, *};

pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input - expected expression");
    };

    let token_clone = token.clone();
    let span_clone = *span;

    match token_clone {
        // Literal tokens - delegated to focused helper
        Token::Integer(_) | Token::Float(_) | Token::String(_) | 
        Token::FString(_) | Token::Char(_) | Token::Bool(_) => {
            parse_literal_token(state, token_clone, span_clone)
        }
        // Identifier tokens - delegated to focused helper
        Token::Identifier(_) | Token::Underscore => {
            parse_identifier_token(state, token_clone, span_clone)
        }
        // Unary operator tokens - delegated to focused helper
        Token::Minus | Token::Bang => {
            parse_unary_operator_token(state, token_clone, span_clone)
        }
        // Function/block tokens - delegated to focused helper
        Token::Fun | Token::Fn | Token::LeftBrace => {
            parse_function_block_token(state, token_clone)
        }
        // Variable declaration tokens - delegated to focused helper
        Token::Let | Token::Var => {
            parse_variable_declaration_token(state, token_clone)
        }
        // Control flow tokens - delegated to focused helper
        Token::If | Token::Match | Token::While | Token::For => {
            parse_control_flow_token(state, token_clone)
        }
        // Lambda expression tokens - delegated to focused helper
        Token::Pipe | Token::OrOr => {
            parse_lambda_token(state, token_clone)
        }
        // Parentheses tokens - delegated to focused helper (unit, grouping, tuples, lambdas)
        Token::LeftParen => {
            parse_parentheses_token(state, span_clone)
        }
        // Data structure definition tokens - delegated to focused helper
        Token::Struct | Token::Trait | Token::Impl => {
            parse_data_structure_token(state, token_clone)
        }
        // Import/module tokens - delegated to focused helper
        Token::Import | Token::Use => {
            parse_import_token(state, token_clone)
        }
        // Special definition tokens - delegated to focused helper
        Token::DataFrame | Token::Actor => {
            parse_special_definition_token(state, token_clone)
        }
        // Control statement tokens - delegated to focused helper
        Token::Pub | Token::Break | Token::Continue | Token::Return => {
            parse_control_statement_token(state, token_clone, span_clone)
        }
        // Collection/enum definition tokens - delegated to focused helper
        Token::LeftBracket | Token::Enum => {
            parse_collection_enum_token(state, token_clone)
        }
        // Constructor tokens - delegated to focused helper
        Token::Some | Token::None | Token::Ok | Token::Err | Token::Result | Token::Option => {
            parse_constructor_token(state, token_clone, span_clone)
        }
        _ => bail!("Unexpected token: {:?}", token_clone),
    }
}

/// Parse literal tokens (Integer, Float, String, Char, Bool, `FString`)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_literal_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Integer(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Integer(value)), span))
        }
        Token::Float(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Float(value)), span))
        }
        Token::String(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::String(value)), span))
        }
        Token::FString(template) => {
            state.tokens.advance();
            // Parse f-string template into parts with proper interpolation
            let parts = parse_fstring_into_parts(&template)?;
            Ok(Expr::new(ExprKind::StringInterpolation { parts }, span))
        }
        Token::Char(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Char(value)), span))
        }
        Token::Bool(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Bool(value)), span))
        }
        _ => bail!("Expected literal token, got: {:?}", token),
    }
}

/// Parse identifier tokens (Identifier, Underscore, fat arrow lambdas)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_identifier_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
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
                        span,
                    },
                    default_value: None,
                    is_mutable: false,
                    span,
                }];
                Ok(Expr::new(ExprKind::Lambda { params, body }, span))
            // Check for macro syntax: println! etc.
            } else if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
                // This is a macro call like println!
                state.tokens.advance(); // consume !
                
                // Convert macro syntax to regular function call
                // println! -> println, assert! -> assert, etc.
                Ok(Expr::new(ExprKind::Identifier(name), span))
            } else {
                Ok(Expr::new(ExprKind::Identifier(name), span))
            }
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Identifier("_".to_string()), span))
        }
        _ => bail!("Expected identifier token, got: {:?}", token),
    }
}

/// Parse unary operator tokens (Minus, Bang)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_unary_operator_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Minus => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?; // High precedence for unary
            Ok(Expr::new(ExprKind::Unary { 
                op: UnaryOp::Negate, 
                operand: Box::new(expr) 
            }, span))
        }
        Token::Bang => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?;
            Ok(Expr::new(ExprKind::Unary { 
                op: UnaryOp::Not, 
                operand: Box::new(expr) 
            }, span))
        }
        _ => bail!("Expected unary operator token, got: {:?}", token),
    }
}

/// Parse parentheses tokens - either unit type (), grouped expression (expr), or tuple (a, b, c)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_parentheses_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Check for unit type ()
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        state.tokens.advance();
        Ok(Expr::new(ExprKind::Literal(Literal::Unit), span))
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
            Ok(Expr::new(ExprKind::Tuple(elements), span))
        } else {
            // Just a grouped expression
            state.tokens.expect(&Token::RightParen)?;
            
            // Check if this is a lambda: (x) => expr
            if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                parse_lambda_from_expr(state, first_expr, span)
            } else {
                Ok(first_expr)
            }
        }
    }
}

/// Parse pub token - handles public declarations for functions, structs, traits, impl blocks
/// Extracted from `parse_prefix` to reduce complexity
fn parse_pub_token(state: &mut ParserState) -> Result<Expr> {
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

/// Parse break token with optional label
/// Extracted from `parse_prefix` to reduce complexity
fn parse_break_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Optional label
    let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let label = Some(name.clone());
        state.tokens.advance();
        label
    } else {
        None
    };
    Ok(Expr::new(ExprKind::Break { label }, span))
}

/// Parse continue token with optional label
/// Extracted from `parse_prefix` to reduce complexity
fn parse_continue_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Optional label
    let label = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let label = Some(name.clone());
        state.tokens.advance();
        label
    } else {
        None
    };
    Ok(Expr::new(ExprKind::Continue { label }, span))
}

/// Parse return token with optional expression
/// Extracted from `parse_prefix` to reduce complexity
fn parse_return_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Check if there's an expression to return
    let value = if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) 
        || state.tokens.peek().is_none() {
        // No expression, just return
        None
    } else {
        // Parse the return expression
        Some(Box::new(super::parse_expr_recursive(state)?))
    };
    Ok(Expr::new(ExprKind::Return { value }, span))
}

/// Parse constructor tokens (Some, None, Ok, Err, Result, Option)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_constructor_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    let constructor_name = match token {
        Token::Some => "Some",
        Token::None => "None", 
        Token::Ok => "Ok",
        Token::Err => "Err",
        Token::Result => "Result",
        Token::Option => "Option",
        _ => bail!("Expected constructor token, got: {:?}", token),
    };
    
    state.tokens.advance();
    Ok(Expr::new(ExprKind::Identifier(constructor_name.to_string()), span))
}

/// Parse control flow tokens (If, Match, While, For)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_control_flow_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::If => parse_if_expression(state),
        Token::Match => parse_match_expression(state),
        Token::While => parse_while_loop(state),
        Token::For => parse_for_loop(state),
        _ => bail!("Expected control flow token, got: {:?}", token),
    }
}

/// Parse data structure definition tokens (Struct, Trait, Impl)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_data_structure_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Struct => parse_struct_definition(state),
        Token::Trait => parse_trait_definition(state),
        Token::Impl => parse_impl_block(state),
        _ => bail!("Expected data structure token, got: {:?}", token),
    }
}

/// Parse import/module tokens (Import, Use)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_import_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Import => parse_import_statement(state),
        Token::Use => parse_use_statement(state),
        _ => bail!("Expected import token, got: {:?}", token),
    }
}

/// Parse lambda expression tokens (Pipe, `OrOr`)\
/// Extracted from `parse_prefix` to reduce complexity
fn parse_lambda_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Pipe => parse_lambda_expression(state),
        Token::OrOr => parse_lambda_no_params(state),
        _ => bail!("Expected lambda token, got: {:?}", token),
    }
}

/// Parse function/block tokens (Fun, Fn, `LeftBrace`)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_function_block_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Fun | Token::Fn => super::functions::parse_function(state),
        Token::LeftBrace => super::collections::parse_block(state),
        _ => bail!("Expected function/block token, got: {:?}", token),
    }
}

/// Parse variable declaration tokens (Let, Var)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_variable_declaration_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Let => parse_let_statement(state),
        Token::Var => parse_var_statement(state),
        _ => bail!("Expected variable declaration token, got: {:?}", token),
    }
}

/// Parse special definition tokens (`DataFrame`, Actor)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_special_definition_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::DataFrame => parse_dataframe_literal(state),
        Token::Actor => parse_actor_definition(state),
        _ => bail!("Expected special definition token, got: {:?}", token),
    }
}

/// Parse control statement tokens (Pub, Break, Continue, Return)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_control_statement_token(state: &mut ParserState, token: Token, span: Span) -> Result<Expr> {
    match token {
        Token::Pub => parse_pub_token(state),
        Token::Break => parse_break_token(state, span),
        Token::Continue => parse_continue_token(state, span), 
        Token::Return => parse_return_token(state, span),
        _ => bail!("Expected control statement token, got: {:?}", token),
    }
}

/// Parse collection/enum definition tokens (`LeftBracket`, Enum)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_collection_enum_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::LeftBracket => parse_list_literal(state),
        Token::Enum => parse_enum_definition(state),
        _ => bail!("Expected collection/enum token, got: {:?}", token),
    }
}

/// Parse let statement: let [mut] name [: type] = value [in body]
fn parse_let_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Let)?;
    
    // Check for optional 'mut' keyword
    let is_mutable = parse_let_mutability(state);
    
    // Parse variable name or destructuring pattern
    let pattern = parse_let_pattern(state, is_mutable)?;
    
    // Parse optional type annotation
    let type_annotation = parse_let_type_annotation(state)?;
    
    // Parse '=' token
    state.tokens.expect(&Token::Equal)?;
    
    // Parse value expression
    let value = Box::new(super::parse_expr_recursive(state)?);
    
    // Parse optional 'in' clause for let expressions
    let body = parse_let_in_clause(state, value.span)?;
    
    // Create the appropriate expression based on pattern type
    create_let_expression(pattern, type_annotation, value, body, is_mutable, start_span)
}

/// Parse mutability for let statement
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_mutability(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse pattern for let statement (identifier or destructuring)
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_pattern(state: &mut ParserState, is_mutable: bool) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        Some((Token::LeftParen, _)) => {
            // Parse tuple destructuring: (x, y) = (1, 2)
            parse_tuple_pattern(state)
        }
        Some((Token::LeftBracket, _)) => {
            // Parse list destructuring: [a, b] = [1, 2]
            parse_list_pattern(state)
        }
        _ => bail!("Expected identifier or pattern after 'let{}'", 
                   if is_mutable { " mut" } else { "" })
    }
}

/// Parse optional type annotation for let statement
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Ok(Some(super::utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Parse optional 'in' clause for let expressions
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_in_clause(state: &mut ParserState, value_span: Span) -> Result<Box<Expr>> {
    if matches!(state.tokens.peek(), Some((Token::In, _))) {
        state.tokens.advance(); // consume 'in'
        Ok(Box::new(super::parse_expr_recursive(state)?))
    } else {
        // For let statements (no 'in'), body is unit
        Ok(Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value_span)))
    }
}

/// Create the appropriate let expression based on pattern type
/// Extracted from `parse_let_statement` to reduce complexity
fn create_let_expression(
    pattern: Pattern,
    type_annotation: Option<Type>,
    value: Box<Expr>,
    body: Box<Expr>,
    is_mutable: bool,
    start_span: Span,
) -> Result<Expr> {
    let end_span = body.span;
    
    match &pattern {
        Pattern::Identifier(name) => {
            Ok(Expr::new(
                ExprKind::Let {
                    name: name.clone(),
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                },
                start_span.merge(end_span),
            ))
        }
        Pattern::Tuple(_) | Pattern::List(_) => {
            // For destructuring patterns, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                },
                start_span.merge(end_span),
            ))
        }
        Pattern::Wildcard | Pattern::Literal(_) | Pattern::QualifiedName(_) | Pattern::Struct { .. } 
        | Pattern::Range { .. } | Pattern::Or(_) | Pattern::Rest | Pattern::RestNamed(_) 
        | Pattern::Ok(_) | Pattern::Err(_) | Pattern::Some(_) | Pattern::None => {
            // For other pattern types, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                },
                start_span.merge(end_span),
            ))
        }
    }
}

/// Parse var statement: var name [: type] = value
/// var is implicitly mutable (like let mut)
fn parse_var_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Var)?;
    
    // var is always mutable
    let is_mutable = true;
    
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
        _ => bail!("Expected identifier or pattern after 'var'")
    };
    
    // Parse optional type annotation
    let type_annotation = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Some(super::utils::parse_type(state)?)
    } else {
        None
    };
    
    // Parse '=' token
    state.tokens.expect(&Token::Equal)?;
    
    // Parse value expression
    let value = Box::new(super::parse_expr_recursive(state)?);
    
    // var doesn't support 'in' syntax, just creates a mutable binding
    let body = Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span));
    
    let end_span = value.span;
    
    // Handle different pattern types
    match &pattern {
        Pattern::Identifier(name) => {
            // Simple identifier binding
            Ok(Expr::new(
                ExprKind::Let {
                    name: name.clone(),
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                },
                start_span.merge(end_span),
            ))
        }
        _ => {
            // For all other patterns (tuple, list, etc), use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                },
                start_span.merge(end_span),
            ))
        }
    }
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
    if state.tokens.peek().is_none() {
        bail!("Expected pattern in match arm");
    }
    
    // Delegate to focused helper functions
    let pattern = parse_single_pattern(state)?;
    
    // Handle multiple patterns with | (or)
    if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        parse_or_pattern(state, pattern)
    } else {
        Ok(pattern)
    }
}

/// Parse a single pattern (delegates to specific pattern parsers)
/// Complexity: <8
fn parse_single_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected pattern");
    };
    
    match token {
        Token::Underscore => parse_wildcard_pattern(state),
        Token::Integer(_) | Token::Float(_) | Token::String(_) | 
        Token::Char(_) | Token::Bool(_) => parse_literal_pattern(state),
        Token::Some | Token::None => parse_option_pattern(state),
        Token::Identifier(_) => parse_identifier_or_constructor_pattern(state),
        Token::LeftParen => parse_match_tuple_pattern(state),
        Token::LeftBracket => parse_match_list_pattern(state),
        _ => bail!("Unexpected token in pattern: {:?}", token)
    }
}

/// Parse wildcard pattern: _
/// Complexity: 1
fn parse_wildcard_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance();
    Ok(Pattern::Wildcard)
}

/// Parse literal patterns: integers, floats, strings, chars, booleans
/// Complexity: <5
fn parse_literal_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected literal pattern");
    };
    
    let pattern = match token {
        Token::Integer(val) => {
            let val = *val;
            state.tokens.advance();
            Pattern::Literal(Literal::Integer(val))
        }
        Token::Float(val) => {
            let val = *val;
            state.tokens.advance();
            Pattern::Literal(Literal::Float(val))
        }
        Token::String(s) => {
            let s = s.clone();
            state.tokens.advance();
            Pattern::Literal(Literal::String(s))
        }
        Token::Char(c) => {
            let c = *c;
            state.tokens.advance();
            Pattern::Literal(Literal::Char(c))
        }
        Token::Bool(b) => {
            let b = *b;
            state.tokens.advance();
            Pattern::Literal(Literal::Bool(b))
        }
        _ => bail!("Expected literal pattern, got: {:?}", token)
    };
    
    Ok(pattern)
}

/// Parse Option patterns: Some, None
/// Complexity: <5
fn parse_option_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected Option pattern");
    };
    
    match token {
        Token::Some => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "Some".to_string())
            } else {
                Ok(Pattern::Identifier("Some".to_string()))
            }
        }
        Token::None => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "None".to_string())
            } else {
                Ok(Pattern::Identifier("None".to_string()))
            }
        }
        _ => bail!("Expected Some or None pattern")
    }
}

/// Parse identifier or constructor patterns
/// Complexity: <5
fn parse_identifier_or_constructor_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((Token::Identifier(name), _span)) = state.tokens.peek() else {
        bail!("Expected identifier pattern");
    };
    
    let name = name.clone();
    state.tokens.advance();
    
    // Check for enum-like patterns: Ok(x), Err(e), etc.
    if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
        parse_constructor_pattern(state, name)
    } else {
        Ok(Pattern::Identifier(name))
    }
}

/// Parse match tuple pattern: (a, b, c)
/// Complexity: <7
fn parse_match_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    
    // Check for empty tuple ()
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        state.tokens.advance();
        return Ok(Pattern::Tuple(vec![]));
    }
    
    // Parse pattern elements
    let mut patterns = vec![parse_match_pattern(state)?];
    
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            break; // trailing comma
        }
        patterns.push(parse_match_pattern(state)?);
    }
    
    state.tokens.expect(&Token::RightParen)?;
    Ok(Pattern::Tuple(patterns))
}

/// Parse list pattern in match: [], [a], [a, b], [head, ...tail]
/// Complexity: <8
fn parse_match_list_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftBracket)?;
    
    // Check for empty list []
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance();
        return Ok(Pattern::List(vec![]));
    }
    
    // Parse pattern elements
    let mut patterns = vec![];
    
    loop {
        // Check for rest pattern ...tail
        if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
            state.tokens.advance();
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let name = name.clone();
                state.tokens.advance();
                patterns.push(Pattern::RestNamed(name));
                break;
            }
            bail!("Expected identifier after ... in list pattern");
        }
        
        patterns.push(parse_match_pattern(state)?);
        
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
                break; // trailing comma
            }
        } else {
            break;
        }
    }
    
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Pattern::List(patterns))
}

/// Parse constructor pattern: Some(x), Ok(value), etc.
/// Complexity: <5
fn parse_constructor_pattern(state: &mut ParserState, name: String) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    
    // Parse the pattern arguments
    let patterns = parse_constructor_arguments(state)?;
    
    state.tokens.expect(&Token::RightParen)?;
    
    // Delegate pattern creation to helper
    create_constructor_pattern(name, patterns)
}

/// Parse constructor arguments (complexity: 6)
fn parse_constructor_arguments(state: &mut ParserState) -> Result<Vec<Pattern>> {
    // Check for empty tuple
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        return Ok(vec![]);
    }
    
    let mut patterns = vec![parse_match_pattern(state)?];
    
    // Parse additional patterns if comma-separated
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma
        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            break; // trailing comma
        }
        patterns.push(parse_match_pattern(state)?);
    }
    
    Ok(patterns)
}

/// Create appropriate pattern based on constructor name (complexity: 5)
fn create_constructor_pattern(name: String, patterns: Vec<Pattern>) -> Result<Pattern> {
    match (name.as_str(), patterns.len()) {
        ("Some", 1) => {
            // Some(pattern) - use Ok variant to represent Option::Some
            Ok(Pattern::Ok(Box::new(patterns.into_iter().next().unwrap())))
        }
        ("None", 0) => {
            // None - just an identifier
            Ok(Pattern::Identifier("None".to_string()))
        }
        (_, 1) => {
            // Single argument constructor - for simplicity, use the inner pattern
            Ok(patterns.into_iter().next().unwrap())
        }
        (name, 0) => {
            // Empty constructor - return as identifier
            Ok(Pattern::Identifier(name.to_string()))
        }
        (_, _) => {
            // Multiple arguments - use tuple pattern
            Ok(Pattern::Tuple(patterns))
        }
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
    let name = parse_struct_name(state)?;
    let type_params = parse_optional_generics(state)?;
    let struct_fields = parse_struct_fields(state)?;
    
    Ok(Expr::new(ExprKind::Struct {
        name,
        type_params,
        fields: struct_fields,
        is_pub: false,
    }, start_span))
}

/// Parse struct name identifier - complexity: 4
fn parse_struct_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected struct name after 'struct'");
    }
}

/// Parse struct field definitions - complexity: 6
fn parse_struct_fields(state: &mut ParserState) -> Result<Vec<StructField>> {
    state.tokens.expect(&Token::LeftBrace)?;
    let mut fields = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        let (field_name, field_type) = parse_single_struct_field(state)?;
        fields.push((field_name, field_type));
        
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    state.tokens.expect(&Token::RightBrace)?;
    
    // Convert to proper Struct variant with StructField
    Ok(fields.into_iter().map(|(name, ty)| StructField {
        name,
        ty,
        is_pub: false,
    }).collect())
}

/// Parse a single struct field (name: Type) - complexity: 5
fn parse_single_struct_field(state: &mut ParserState) -> Result<(String, Type)> {
    let field_name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected field name in struct");
    };
    
    state.tokens.expect(&Token::Colon)?;
    let field_type = super::utils::parse_type(state)?;
    
    Ok((field_name, field_type))
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
    let start_span = state.tokens.expect(&Token::Impl)?;
    
    // Parse impl header (trait and type names)
    let (trait_name, type_name) = parse_impl_header(state)?;
    
    // Parse impl body (methods)
    state.tokens.expect(&Token::LeftBrace)?;
    skip_impl_body(state)?;
    state.tokens.expect(&Token::RightBrace)?;
    
    Ok(Expr::new(ExprKind::Impl {
        type_params: vec![],
        trait_name,
        for_type: type_name,
        methods: vec![],
        is_pub: false,
    }, start_span))
}

/// Parse impl header to get trait and type names (complexity: 8)
fn parse_impl_header(state: &mut ParserState) -> Result<(Option<String>, String)> {
    // Parse first identifier (trait or type name)
    let first_name = parse_optional_identifier(state);
    
    // Check for "for" keyword to determine if first was trait
    if matches!(state.tokens.peek(), Some((Token::For, _))) {
        state.tokens.advance();
        let type_name = parse_required_identifier(state, "type name after 'for' in impl")?;
        Ok((first_name, type_name))
    } else if let Some(type_name) = first_name {
        // impl Type { ... } case
        Ok((None, type_name))
    } else {
        bail!("Expected type or trait name in impl");
    }
}

/// Parse optional identifier (complexity: 3)
fn parse_optional_identifier(state: &mut ParserState) -> Option<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Some(name)
    } else {
        None
    }
}

/// Parse required identifier with error message (complexity: 3)
fn parse_required_identifier(state: &mut ParserState, context: &str) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected {}", context)
    }
}

/// Skip impl body by tracking brace depth (complexity: 5)
fn skip_impl_body(state: &mut ParserState) -> Result<()> {
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
    Ok(())
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
    let start_span = parse_dataframe_header(state)?;
    let columns = parse_dataframe_columns(state)?;
    state.tokens.expect(&Token::RightBracket)?;
    
    // Convert to DataFrame expression
    let df_columns = create_dataframe_columns(columns);
    Ok(Expr::new(ExprKind::DataFrame { columns: df_columns }, start_span))
}

/// Parse dataframe header: df![
/// Complexity: 3
fn parse_dataframe_header(state: &mut ParserState) -> Result<Span> {
    let start_span = state.tokens.expect(&Token::DataFrame)?;
    state.tokens.expect(&Token::Bang)?;
    state.tokens.expect(&Token::LeftBracket)?;
    Ok(start_span)
}

/// Parse all dataframe columns
/// Complexity: <5
fn parse_dataframe_columns(state: &mut ParserState) -> Result<Vec<(String, Expr)>> {
    let mut columns = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        let column = parse_single_dataframe_column(state)?;
        columns.push(column);
        
        // Check for comma separator
        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }
    
    Ok(columns)
}

/// Parse a single dataframe column: "name" => [values]
/// Complexity: <5
fn parse_single_dataframe_column(state: &mut ParserState) -> Result<(String, Expr)> {
    let col_name = parse_dataframe_column_name(state)?;
    state.tokens.expect(&Token::FatArrow)?;
    let values = parse_dataframe_column_values(state)?;
    Ok((col_name, values))
}

/// Parse dataframe column name (string or identifier)
/// Complexity: 3
fn parse_dataframe_column_name(state: &mut ParserState) -> Result<String> {
    match state.tokens.peek() {
        Some((Token::String(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(name)
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(name)
        }
        _ => bail!("Expected column name (string or identifier) in dataframe")
    }
}

/// Parse dataframe column values (must be a list)
/// Complexity: 2
fn parse_dataframe_column_values(state: &mut ParserState) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::LeftBracket, _))) {
        parse_list_literal(state)
    } else {
        bail!("Expected list of values after => in dataframe column")
    }
}

/// Convert parsed columns to `DataFrameColumn` structs
/// Complexity: <5
fn create_dataframe_columns(columns: Vec<(String, Expr)>) -> Vec<DataFrameColumn> {
    columns.into_iter().map(|(name, values)| {
        let value_exprs = match values.kind {
            ExprKind::List(exprs) => exprs,
            _ => vec![values], // Fallback for non-list
        };
        DataFrameColumn {
            name,
            values: value_exprs,
        }
    }).collect()
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
    
    // Check for discriminant value: = <integer>
    let discriminant = if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance(); // consume =
        parse_variant_discriminant(state)?
    } else {
        None
    };
    
    // Check for fields (tuple variants)
    let fields = if discriminant.is_none() {
        parse_variant_fields(state)?
    } else {
        None // Can't have both discriminant and fields
    };
    
    Ok(EnumVariant {
        name: variant_name,
        fields,
        discriminant,
    })
}

/// Parse discriminant value for enum variant
/// Complexity: <5
fn parse_variant_discriminant(state: &mut ParserState) -> Result<Option<i64>> {
    match state.tokens.peek() {
        Some((Token::Integer(val), _)) => {
            let value = *val;
            state.tokens.advance();
            Ok(Some(value))
        }
        Some((Token::Minus, _)) => {
            state.tokens.advance(); // consume -
            match state.tokens.peek() {
                Some((Token::Integer(val), _)) => {
                    let value = -(*val);
                    state.tokens.advance();
                    Ok(Some(value))
                }
                _ => bail!("Expected integer after - in enum discriminant")
            }
        }
        _ => bail!("Expected integer value for enum discriminant")
    }
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
    let name = parse_actor_name(state)?;
    
    // Parse { body }
    state.tokens.expect(&Token::LeftBrace)?;
    
    // Parse actor body components
    let (state_fields, handlers) = parse_actor_body(state)?;
    
    state.tokens.expect(&Token::RightBrace)?;
    
    // Create the actor expression
    create_actor_expression(name, state_fields, handlers, start_span)
}

/// Parse actor name
/// Extracted from `parse_actor_definition` to reduce complexity
fn parse_actor_name(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        Ok(name)
    } else {
        bail!("Expected actor name after 'actor'");
    }
}

/// Parse actor body including state fields and handlers
/// Extracted from `parse_actor_definition` to reduce complexity
fn parse_actor_body(state: &mut ParserState) -> Result<(Vec<(String, Type, Option<Box<Expr>>)>, Vec<String>)> {
    let mut state_fields = Vec::new();
    let mut handlers = Vec::new();
    
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        match state.tokens.peek() {
            Some((Token::State, _)) => {
                let field = parse_actor_state_field(state)?;
                state_fields.push(field);
            }
            Some((Token::Receive, _)) => {
                let new_handlers = parse_actor_receive_block(state)?;
                handlers.extend(new_handlers);
            }
            Some((Token::Identifier(_), _)) => {
                let field = parse_actor_bare_field(state)?;
                state_fields.push(field);
            }
            _ => {
                // Skip unknown tokens
                state.tokens.advance();
            }
        }
    }
    
    Ok((state_fields, handlers))
}

/// Parse state field with 'state' keyword
/// Extracted from `parse_actor_body` to reduce complexity
fn parse_actor_state_field(state: &mut ParserState) -> Result<(String, Type, Option<Box<Expr>>)> {
    state.tokens.advance(); // consume 'state'
    
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
        
        Ok((field, field_type, initial_value))
    } else {
        bail!("Expected field name after 'state'");
    }
}

/// Parse receive block with handlers
/// Extracted from `parse_actor_body` to reduce complexity
fn parse_actor_receive_block(state: &mut ParserState) -> Result<Vec<String>> {
    state.tokens.advance(); // consume 'receive'
    state.tokens.expect(&Token::LeftBrace)?;
    
    let mut handlers = Vec::new();
    
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
    Ok(handlers)
}

/// Parse bare field definition
/// Extracted from `parse_actor_body` to reduce complexity
fn parse_actor_bare_field(state: &mut ParserState) -> Result<(String, Type, Option<Box<Expr>>)> {
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
        
        Ok((field, field_type, None))
    } else {
        bail!("Expected field name in actor");
    }
}

/// Create the final actor expression
/// Extracted from `parse_actor_definition` to reduce complexity
fn create_actor_expression(
    name: String,
    state_fields: Vec<(String, Type, Option<Box<Expr>>)>,
    handlers: Vec<String>,
    start_span: Span,
) -> Result<Expr> {
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
    // Try each category of operators
    map_arithmetic_operator(token)
        .or_else(|| map_comparison_operator(token))
        .or_else(|| map_logical_operator(token))
        .or_else(|| map_bitwise_operator(token))
}

/// Map arithmetic tokens to binary operators
/// Extracted from `token_to_binary_op` to reduce complexity
fn map_arithmetic_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::Plus => Some(BinaryOp::Add),
        Token::Minus => Some(BinaryOp::Subtract),
        Token::Star => Some(BinaryOp::Multiply),
        Token::Slash => Some(BinaryOp::Divide),
        Token::Percent => Some(BinaryOp::Modulo),
        Token::Power => Some(BinaryOp::Power),
        _ => None,
    }
}

/// Map comparison tokens to binary operators
/// Extracted from `token_to_binary_op` to reduce complexity
fn map_comparison_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::EqualEqual => Some(BinaryOp::Equal),
        Token::NotEqual => Some(BinaryOp::NotEqual),
        Token::Less => Some(BinaryOp::Less),
        Token::LessEqual => Some(BinaryOp::LessEqual),
        Token::Greater => Some(BinaryOp::Greater),
        Token::GreaterEqual => Some(BinaryOp::GreaterEqual),
        _ => None,
    }
}

/// Map logical tokens to binary operators
/// Extracted from `token_to_binary_op` to reduce complexity
fn map_logical_operator(token: &Token) -> Option<BinaryOp> {
    match token {
        Token::AndAnd => Some(BinaryOp::And),
        Token::OrOr => Some(BinaryOp::Or),
        Token::NullCoalesce => Some(BinaryOp::NullCoalesce),
        _ => None,
    }
}

/// Map bitwise tokens to binary operators
/// Extracted from `token_to_binary_op` to reduce complexity
fn map_bitwise_operator(token: &Token) -> Option<BinaryOp> {
    match token {
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

/// Parse f-string content into interpolation parts
fn parse_fstring_into_parts(input: &str) -> Result<Vec<StringPart>> {
    use crate::frontend::parser::Parser;
    
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                // Escaped brace
                chars.next();
                current.push('{');
            } else {
                // Save text part if any
                if !current.is_empty() {
                    parts.push(StringPart::Text(current.clone()));
                    current.clear();
                }
                
                // Extract and parse expression
                let expr_str = extract_fstring_expr(&mut chars)?;
                let mut parser = Parser::new(&expr_str);
                let expr = parser.parse_expr()?;
                parts.push(StringPart::Expr(Box::new(expr)));
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                // Escaped brace
                chars.next();
                current.push('}');
            } else {
                bail!("Unmatched '}}' in f-string");
            }
        } else {
            current.push(ch);
        }
    }
    
    // Add remaining text
    if !current.is_empty() {
        parts.push(StringPart::Text(current));
    }
    
    Ok(parts)
}

/// Extract expression from f-string between braces
fn extract_fstring_expr(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
    let mut expr = String::new();
    let mut depth = 1;
    
    for ch in chars.by_ref() {
        if ch == '{' {
            depth += 1;
            expr.push(ch);
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(expr);
            }
            expr.push(ch);
        } else {
            expr.push(ch);
        }
    }
    
    bail!("Unclosed interpolation in f-string")
}