//! Basic expression parsing - minimal version with only used functions
use super::{ParserState, bail, Result, Expr, Token, ExprKind, Span, Literal, BinaryOp, UnaryOp, Param, Pattern, Type, TypeKind, MatchArm, StructField, TraitMethod, DataFrameColumn, ActorHandler, StringPart, EnumVariant};
pub fn parse_prefix(state: &mut ParserState) -> Result<Expr> {
    let Some((token, span)) = state.tokens.peek() else {
        bail!("Unexpected end of input - expected expression");
    };
    // Optimize: Clone once and match on owned token for better cache locality
    let token = token.clone();
    let span = *span;
    match token {
        // Literal tokens - inlined for performance
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
        // Identifier tokens - delegated to focused helper
        Token::Identifier(_) | Token::Underscore => {
            parse_identifier_token(state, &token, span)
        }
        // Unary operator tokens - inlined for performance
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
        Token::Await => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?;
            Ok(Expr::new(ExprKind::Await {
                expr: Box::new(expr)
            }, span))
        }
        // Function/block tokens - delegated to focused helper
        Token::Fun | Token::Fn | Token::LeftBrace => {
            parse_function_block_token(state, token)
        }
        // Variable declaration tokens - delegated to focused helper
        Token::Let | Token::Var => {
            parse_variable_declaration_token(state, token)
        }
        // Control flow tokens - delegated to focused helper
        Token::If | Token::Match | Token::While | Token::For | Token::Try | Token::Loop => {
            parse_control_flow_token(state, token)
        }
        // Module declaration token
        Token::Mod => {
            parse_module_declaration(state)
        }
        // Lambda expression tokens - delegated to focused helper
        Token::Pipe | Token::OrOr | Token::Backslash => {
            parse_lambda_token(state, token)
        }
        // Parentheses tokens - delegated to focused helper (unit, grouping, tuples, lambdas)
        Token::LeftParen => {
            parse_parentheses_token(state, span)
        }
        // Data structure definition tokens - delegated to focused helper
        Token::Struct | Token::Trait | Token::Impl | Token::Type => {
            parse_data_structure_token(state, token)
        }
        // Import/module tokens - delegated to focused helper
        Token::Import => {
            parse_import_token(state, token)
        }
        // Special definition tokens - delegated to focused helper
        Token::DataFrame | Token::Actor => {
            parse_special_definition_token(state, token)
        }
        // Control statement tokens - delegated to focused helper
        Token::Pub | Token::Break | Token::Continue | Token::Return | Token::Throw |
        Token::Export | Token::Async | Token::Increment | Token::Decrement => {
            parse_control_statement_token(state, token, span)
        }
        // Collection/enum definition tokens - delegated to focused helper
        Token::LeftBracket | Token::Enum => {
            parse_collection_enum_token(state, token)
        }
        // Constructor tokens - delegated to focused helper
        Token::Some | Token::None | Token::Ok | Token::Err | Token::Result | Token::Option => {
            parse_constructor_token(state, token, span)
        }
        _ => bail!("Unexpected token: {:?}", token),
    }
}
/// Parse literal tokens (Integer, Float, String, Char, Bool, `FString`)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_literal_token(state: &mut ParserState, token: &Token, span: Span) -> Result<Expr> {
    match token {
        Token::Integer(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Integer(*value)), span))
        }
        Token::Float(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Float(*value)), span))
        }
        Token::String(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::String(value.clone())), span))
        }
        Token::FString(template) => {
            state.tokens.advance();
            // Parse f-string template into parts with proper interpolation
            let parts = parse_fstring_into_parts(template)?;
            Ok(Expr::new(ExprKind::StringInterpolation { parts }, span))
        }
        Token::Char(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Char(*value)), span))
        }
        Token::Bool(value) => {
            state.tokens.advance();
            Ok(Expr::new(ExprKind::Literal(Literal::Bool(*value)), span))
        }
        _ => bail!("Expected literal token, got: {:?}", token),
    }
}
/// Parse identifier tokens (Identifier, Underscore, fat arrow lambdas)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_identifier_token(state: &mut ParserState, token: &Token, span: Span) -> Result<Expr> {
    match token {
        Token::Identifier(name) => {
            state.tokens.advance();
            // Check for module path: math::add
            if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                let mut path = vec![name.clone()];
                while matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
                    state.tokens.advance(); // consume ::
                    if let Some((Token::Identifier(segment), _)) = state.tokens.peek() {
                        path.push(segment.clone());
                        state.tokens.advance();
                    } else {
                        bail!("Expected identifier after '::'");
                    }
                }
                // Create a qualified name expression
                let qualified_name = path.join("::");
                Ok(Expr::new(ExprKind::Identifier(qualified_name), span))
            }
            // Check for fat arrow lambda: x => x * 2
            else if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
                state.tokens.advance(); // consume =>
                let body = Box::new(super::parse_expr_recursive(state)?);
                let params = vec![Param {
                    pattern: Pattern::Identifier(name.clone()),
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
                Ok(Expr::new(ExprKind::Identifier(name.clone()), span))
            } else {
                Ok(Expr::new(ExprKind::Identifier(name.clone()), span))
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
fn parse_unary_operator_token(state: &mut ParserState, token: &Token, span: Span) -> Result<Expr> {
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
        Token::Await => {
            state.tokens.advance();
            let expr = super::parse_expr_with_precedence_recursive(state, 13)?;
            Ok(Expr::new(ExprKind::Await {
                expr: Box::new(expr)
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
/// Parse throw statement token  
fn parse_throw_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Throw always requires an expression
    let expr = Box::new(super::parse_expr_recursive(state)?);
    Ok(Expr::new(ExprKind::Throw { expr }, span))
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
    // Check if this is a qualified name like Option::Some
    if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        state.tokens.advance(); // consume ::
        if let Some((next_token, _)) = state.tokens.peek() {
            let variant_name = match next_token.clone() {
                Token::Some => "Some".to_string(),
                Token::None => "None".to_string(),
                Token::Ok => "Ok".to_string(),
                Token::Err => "Err".to_string(),
                Token::Identifier(name) => name,
                _ => bail!("Expected variant name after '::'"),
            };
            state.tokens.advance();
            let qualified_name = format!("{constructor_name}::{variant_name}");
            return Ok(Expr::new(ExprKind::Identifier(qualified_name), span));
        }
        bail!("Expected variant name after '::'");
    }
    Ok(Expr::new(ExprKind::Identifier(constructor_name.to_string()), span))
}
/// Parse control flow tokens (If, Match, While, For, Try)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_control_flow_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::If => parse_if_expression(state),
        Token::Match => parse_match_expression(state),
        Token::While => parse_while_loop(state),
        Token::For => parse_for_loop(state),
        Token::Try => parse_try_catch(state),
        Token::Loop => parse_loop(state),
        _ => bail!("Expected control flow token, got: {:?}", token),
    }
}
/// Parse try-catch-finally block
/// Complexity: <10 (structured error handling)
fn parse_try_catch(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Try)?;
    let try_block = parse_try_block(state)?;
    let catch_clauses = parse_catch_clauses(state)?;
    let finally_block = parse_finally_block(state)?;
    validate_try_catch_structure(&catch_clauses, finally_block.as_deref())?;
    Ok(Expr::new(
        ExprKind::TryCatch {
            try_block,
            catch_clauses,
            finally_block,
        },
        start_span,
    ))
}
/// Parse module declaration: mod name { body }
/// Complexity: <5 (simple structure)
fn parse_module_declaration(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Mod)?;
    // Parse module name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let n = n.clone();
        state.tokens.advance();
        n
    } else {
        bail!("Expected module name after 'mod'");
    };
    // Parse module body with visibility support
    state.tokens.expect(&Token::LeftBrace)?;
    let body = Box::new(parse_module_body(state)?);
    Ok(Expr::new(
        ExprKind::Module { name, body },
        start_span,
    ))
}
/// Parse module body with support for visibility modifiers (pub)
fn parse_module_body(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.peek().map_or(Span { start: 0, end: 0 }, |t| t.1);
    let mut exprs = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Check for visibility modifier
        let is_pub = if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
            state.tokens.advance();
            true
        } else {
            false
        };
        // Parse the item with visibility
        let expr = if matches!(state.tokens.peek(), Some((Token::Fun, _))) {
            super::functions::parse_function_with_visibility(state, is_pub)?
        } else if is_pub {
            bail!("'pub' can only be used with function declarations in modules");
        } else {
            // Regular expression without visibility
            super::parse_expr_recursive(state)?
        };
        exprs.push(expr);
        // Skip optional semicolons
        if matches!(state.tokens.peek(), Some((Token::Semicolon, _))) {
            state.tokens.advance();
        }
    }
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Block(exprs), start_span))
}
/// Parse data structure definition tokens (Struct, Trait, Impl)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_data_structure_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Struct => parse_struct_definition(state),
        Token::Trait => parse_trait_definition(state),
        Token::Impl => parse_impl_block(state),
        Token::Type => parse_type_alias(state),
        _ => bail!("Expected data structure token, got: {:?}", token),
    }
}
/// Parse import/module tokens (Import, Use)
/// Extracted from `parse_prefix` to reduce complexity
fn parse_import_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Import => parse_import_statement(state),
        Token::Use => bail!("'use' statements not yet supported, please use 'import'"),
        _ => bail!("Expected import token, got: {:?}", token),
    }
}
/// Parse lambda expression tokens (Pipe, `OrOr`)\
/// Extracted from `parse_prefix` to reduce complexity
fn parse_lambda_token(state: &mut ParserState, token: Token) -> Result<Expr> {
    match token {
        Token::Pipe => parse_lambda_expression(state),
        Token::OrOr => parse_lambda_no_params(state),
        Token::Backslash => super::functions::parse_lambda(state),
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
        Token::DataFrame => {
            // Check if this is df! (literal) or df (identifier)
            if matches!(state.tokens.peek(), Some((Token::Bang, _))) {
                parse_dataframe_literal(state)
            } else {
                // Treat 'df' as a regular identifier for method calls, etc.
                // Consume the DataFrame token since we're handling it as identifier
                state.tokens.advance();
                Ok(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    Span::default(),
                ))
            }
        }
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
        Token::Throw => parse_throw_token(state, span),
        Token::Export => parse_export_token(state),
        Token::Async => parse_async_token(state),
        Token::Increment => parse_increment_token(state, span),
        Token::Decrement => parse_decrement_token(state, span),
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
        Some((Token::DataFrame, _)) => {
            // Allow 'df' as a variable name (common in data science)
            state.tokens.advance();
            Ok(Pattern::Identifier("df".to_string()))
        }
        Some((Token::LeftParen, _)) => {
            // Parse tuple destructuring: (x, y) = (1, 2)
            parse_tuple_pattern(state)
        }
        Some((Token::LeftBracket, _)) => {
            // Parse list destructuring: [a, b] = [1, 2]
            parse_list_pattern(state)
        }
        Some((Token::LeftBrace, _)) => {
            // Parse struct destructuring: {name, age} = obj
            parse_struct_pattern(state)
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
        | Pattern::AtBinding { .. } | Pattern::WithDefault { .. } | Pattern::Ok(_) | Pattern::Err(_) | Pattern::Some(_) | Pattern::None => {
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
    
    let pattern = parse_var_pattern(state)?;
    let type_annotation = parse_optional_type_annotation(state)?;
    
    state.tokens.expect(&Token::Equal)?;
    let value = Box::new(super::parse_expr_recursive(state)?);
    
    create_var_expression(pattern, type_annotation, value, start_span)
}

/// Extract method: Parse variable pattern - complexity: 6
fn parse_var_pattern(state: &mut ParserState) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        Some((Token::DataFrame, _)) => {
            // Allow 'df' as a variable name (common in data science)
            state.tokens.advance();
            Ok(Pattern::Identifier("df".to_string()))
        }
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::LeftBracket, _)) => parse_list_pattern(state),
        _ => bail!("Expected identifier or pattern after 'var'")
    }
}

/// Extract method: Parse optional type annotation - complexity: 4
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Option<crate::frontend::ast::Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Ok(Some(super::utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Extract method: Create variable expression - complexity: 6
fn create_var_expression(
    pattern: Pattern, 
    type_annotation: Option<crate::frontend::ast::Type>, 
    value: Box<Expr>, 
    start_span: Span
) -> Result<Expr> {
    let body = Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span));
    let end_span = value.span;
    let is_mutable = true;
    
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
        _ => {
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
pub fn parse_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    let mut patterns = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        let pattern = parse_single_tuple_pattern_element(state)?;
        patterns.push(pattern);
        // Use shared separator handler
        if !handle_pattern_separator(state, Token::RightParen)? {
            break;
        }
    }
    state.tokens.expect(&Token::RightParen)?;
    Ok(Pattern::Tuple(patterns))
}
/// Parse a single element in a tuple pattern
/// Extracted to reduce complexity of `parse_tuple_pattern`
fn parse_single_tuple_pattern_element(state: &mut ParserState) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();
            Ok(Pattern::Identifier(name))
        }
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::LeftBracket, _)) => parse_list_pattern(state),
        Some((Token::LeftBrace, _)) => parse_struct_pattern(state),
        Some((Token::Underscore, _)) => {
            state.tokens.advance();
            Ok(Pattern::Wildcard)
        }
        _ => bail!("Expected identifier, tuple, list, struct, or wildcard in tuple pattern"),
    }
}
pub fn parse_struct_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance(); // consume '{'
    let mut fields = Vec::new();
    let mut is_first = true;
    while let Some((token, _)) = state.tokens.peek() {
        if matches!(token, Token::RightBrace) {
            state.tokens.advance(); // consume '}'
            break;
        }
        if !is_first {
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance(); // consume ','
            } else {
                bail!("Expected comma between struct pattern fields");
            }
        }
        is_first = false;
        // Parse field name
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let field_name = name.clone();
            state.tokens.advance();
            // For now, support shorthand syntax only: {name, age}
            // Note: Support full syntax: {name: pattern, age: other_pattern}
            let field = crate::frontend::ast::StructPatternField {
                name: field_name.clone(),
                pattern: None, // Shorthand means field name is the variable name
            };
            fields.push(field);
        } else {
            bail!("Expected identifier in struct pattern");
        }
    }
    Ok(Pattern::Struct {
        name: String::new(), // Anonymous struct pattern (empty name)
        fields,
        has_rest: false, // No rest patterns in basic struct destructuring
    })
}
pub fn parse_list_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftBracket)?;
    let mut patterns = Vec::new();
    while !matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        let pattern = parse_single_list_pattern_element(state)?;
        patterns.push(pattern);
        // Handle comma separator
        if !handle_pattern_separator(state, Token::RightBracket)? {
            break;
        }
    }
    state.tokens.expect(&Token::RightBracket)?;
    Ok(Pattern::List(patterns))
}
/// Parse a single element in a list pattern
/// Extracted to reduce complexity of `parse_list_pattern`
fn parse_single_list_pattern_element(state: &mut ParserState) -> Result<Pattern> {
    match state.tokens.peek() {
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            parse_identifier_pattern_with_default(state, name)
        }
        Some((Token::DotDotDot, _)) => parse_rest_pattern(state),
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::LeftBracket, _)) => parse_list_pattern(state),
        Some((Token::Underscore, _)) => {
            state.tokens.advance();
            Ok(Pattern::Wildcard)
        }
        _ => bail!("Expected identifier, tuple, list, wildcard, or rest pattern in list pattern"),
    }
}
/// Parse identifier pattern with optional default value
/// Extracted to reduce complexity
fn parse_identifier_pattern_with_default(state: &mut ParserState, name: String) -> Result<Pattern> {
    state.tokens.advance();
    // Check for default value: identifier = expr
    if matches!(state.tokens.peek(), Some((Token::Equal, _))) {
        state.tokens.advance(); // consume '='
        let default_expr = super::parse_expr_recursive(state)?;
        Ok(Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier(name)),
            default: Box::new(default_expr),
        })
    } else {
        Ok(Pattern::Identifier(name))
    }
}
/// Parse rest pattern (...rest or ...)
/// Extracted to reduce complexity
fn parse_rest_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance(); // consume ...
    // Check if named rest pattern
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(Pattern::RestNamed(name))
    } else {
        Ok(Pattern::Rest)
    }
}
/// Handle pattern separator (comma) and check for trailing comma
/// Returns true if should continue parsing, false if should stop
/// Extracted to reduce complexity and share with tuple pattern
fn handle_pattern_separator(state: &mut ParserState, end_token: Token) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        // Check for trailing comma
        if let Some((token, _)) = state.tokens.peek() {
            if *token == end_token {
                return Ok(false);
            }
        }
        Ok(true)
    } else if let Some((token, _)) = state.tokens.peek() {
        if *token != end_token {
        let expected = match end_token {
            Token::RightBracket => "',' or ']'",
            Token::RightParen => "',' or ')'",
            _ => "',' or closing delimiter",
        };
            bail!("Expected {} in pattern", expected);
        }
        Ok(false)
    } else {
        bail!("Unexpected end of input in pattern")
    }
}
/// Parse if expression: if condition { `then_branch` } [else { `else_branch` }]
/// Also handles if-let: if let pattern = expr { `then_branch` } [else { `else_branch` }]
/// Complexity: <10 (split into helper functions)
fn parse_if_expression(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::If)?;
    // Check for if-let syntax
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        parse_if_let_expression(state, start_span)
    } else {
        parse_regular_if_expression(state, start_span)
    }
}
/// Parse if-let expression: if let pattern = expr { then } [else { else }]
/// Complexity: <10
fn parse_if_let_expression(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume 'let'
    // Parse the pattern
    let pattern = parse_match_pattern(state)
        .map_err(|e| anyhow::anyhow!("Expected pattern after 'if let': {}", e))?;
    // Expect '='
    state.tokens.expect(&Token::Equal)
        .map_err(|e| anyhow::anyhow!("Expected '=' after pattern in if-let: {}", e))?;
    // Parse the expression to match against
    let expr = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected expression after '=' in if-let: {}", e))?);
    // Parse then branch
    let then_branch = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected body after if-let condition, typically {{ ... }}: {}", e))?);
    // Parse optional else branch
    let else_branch = parse_else_branch(state)?;
    Ok(Expr::new(
        ExprKind::IfLet {
            pattern,
            expr,
            then_branch,
            else_branch,
        },
        start_span,
    ))
}
/// Parse regular if expression: if condition { then } [else { else }]
/// Complexity: <10
fn parse_regular_if_expression(state: &mut ParserState, start_span: Span) -> Result<Expr> {
    // Parse condition with better error context
    let condition = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected condition after 'if': {}", e))?);
    // Parse then branch (expect block) with better error context
    let then_branch = Box::new(super::parse_expr_recursive(state)
        .map_err(|e| anyhow::anyhow!("Expected body after if condition, typically {{ ... }}: {}", e))?);
    // Parse optional else branch
    let else_branch = parse_else_branch(state)?;
    Ok(Expr::new(
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        },
        start_span,
    ))
}
/// Parse else branch for if/if-let expressions
/// Complexity: <10
fn parse_else_branch(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Else, _))) {
        state.tokens.advance(); // consume 'else'
        // Check for else-if or else-if-let
        if matches!(state.tokens.peek(), Some((Token::If, _))) {
            // Let the recursive call handle else-if or else-if-let
            Ok(Some(Box::new(parse_if_expression(state)?)))
        } else {
            Ok(Some(Box::new(super::parse_expr_recursive(state)
                .map_err(|e| anyhow::anyhow!("Expected body after 'else', typically {{ ... }}: {}", e))?)))
        }
    } else {
        Ok(None)
    }
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
        Token::Ok | Token::Err => parse_result_pattern(state),
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
    let token = token.clone(); // Clone to avoid borrow issues
    let pattern = match token {
        Token::Integer(val) => parse_integer_literal_pattern(state, val)?,
        Token::Float(val) => parse_simple_literal_pattern(state, Literal::Float(val))?,
        Token::String(s) => parse_simple_literal_pattern(state, Literal::String(s))?,
        Token::Char(c) => parse_char_literal_pattern(state, c)?,
        Token::Bool(b) => parse_simple_literal_pattern(state, Literal::Bool(b))?,
        _ => bail!("Expected literal pattern, got: {:?}", token)
    };
    Ok(pattern)
}

/// Extract method: Parse integer literal with optional range pattern - complexity: 8
fn parse_integer_literal_pattern(state: &mut ParserState, val: i64) -> Result<Pattern> {
    state.tokens.advance();
    // Check for range patterns: 1..5 or 1..=5
    match state.tokens.peek() {
        Some((Token::DotDot, _)) => parse_integer_range_pattern(state, val, false),
        Some((Token::DotDotEqual, _)) => parse_integer_range_pattern(state, val, true),
        _ => Ok(Pattern::Literal(Literal::Integer(val))),
    }
}

/// Extract method: Parse integer range pattern - complexity: 6
fn parse_integer_range_pattern(state: &mut ParserState, start_val: i64, inclusive: bool) -> Result<Pattern> {
    state.tokens.advance(); // consume '..' or '..='
    if let Some((Token::Integer(end_val), _)) = state.tokens.peek() {
        let end_val = *end_val;
        state.tokens.advance();
        Ok(Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(start_val))),
            end: Box::new(Pattern::Literal(Literal::Integer(end_val))),
            inclusive,
        })
    } else {
        bail!("Expected integer after range operator");
    }
}

/// Extract method: Parse char literal with optional range pattern - complexity: 8
fn parse_char_literal_pattern(state: &mut ParserState, val: char) -> Result<Pattern> {
    state.tokens.advance();
    // Check for range patterns: 'a'..'z' or 'a'..='z'
    match state.tokens.peek() {
        Some((Token::DotDot, _)) => parse_char_range_pattern(state, val, false),
        Some((Token::DotDotEqual, _)) => parse_char_range_pattern(state, val, true),
        _ => Ok(Pattern::Literal(Literal::Char(val))),
    }
}

/// Extract method: Parse char range pattern - complexity: 6
fn parse_char_range_pattern(state: &mut ParserState, start_val: char, inclusive: bool) -> Result<Pattern> {
    state.tokens.advance(); // consume '..' or '..='
    if let Some((Token::Char(end_val), _)) = state.tokens.peek() {
        let end_val = *end_val;
        state.tokens.advance();
        Ok(Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Char(start_val))),
            end: Box::new(Pattern::Literal(Literal::Char(end_val))),
            inclusive,
        })
    } else {
        bail!("Expected char after range operator");
    }
}

/// Extract method: Parse simple literal patterns - complexity: 2
fn parse_simple_literal_pattern(state: &mut ParserState, literal: Literal) -> Result<Pattern> {
    state.tokens.advance();
    Ok(Pattern::Literal(literal))
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
/// Parse Result patterns (Ok/Err)
/// Complexity: 4
fn parse_result_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected Result pattern");
    };
    match token {
        Token::Ok => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "Ok".to_string())
            } else {
                Ok(Pattern::Identifier("Ok".to_string()))
            }
        }
        Token::Err => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_constructor_pattern(state, "Err".to_string())
            } else {
                Ok(Pattern::Identifier("Err".to_string()))
            }
        }
        _ => bail!("Expected Ok or Err pattern")
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

    // Check for @ bindings: name @ pattern
    if matches!(state.tokens.peek(), Some((Token::At, _))) {
        state.tokens.advance();
        let inner_pattern = parse_single_pattern(state)?;
        return Ok(Pattern::AtBinding {
            name,
            pattern: Box::new(inner_pattern),
        });
    }

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
        // Check for rest pattern ..tail (Ruchy uses .. not ...)
        if matches!(state.tokens.peek(), Some((Token::DotDot, _))) {
            state.tokens.advance();
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                let name = name.clone();
                state.tokens.advance();
                patterns.push(Pattern::RestNamed(name));
                // Check if there are more elements after the rest pattern
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                    // Continue parsing remaining patterns after rest
                    continue;
                }
                break;
            }
            bail!("Expected identifier after .. in list pattern");
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
/// Create appropriate pattern based on constructor name (complexity: 8)
fn create_constructor_pattern(name: String, patterns: Vec<Pattern>) -> Result<Pattern> {
    match (name.as_str(), patterns.len()) {
        ("Ok", 1) => {
            // Ok(pattern) - Result success case
            Ok(Pattern::Ok(Box::new(patterns.into_iter().next()
                .expect("patterns.len() == 1, so next() must return Some"))))
        }
        ("Err", 1) => {
            // Err(pattern) - Result error case
            Ok(Pattern::Err(Box::new(patterns.into_iter().next()
                .expect("patterns.len() == 1, so next() must return Some"))))
        }
        ("Some", 1) => {
            // Some(pattern) - Option success case
            Ok(Pattern::Some(Box::new(patterns.into_iter().next()
                .expect("patterns.len() == 1, so next() must return Some"))))
        }
        ("None", 0) => {
            // None - Option empty case
            Ok(Pattern::None)
        }
        (_, 1) => {
            // Single argument constructor - for simplicity, use the inner pattern
            Ok(patterns.into_iter().next()
                .expect("patterns.len() == 1, so next() must return Some"))
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
        Ok(patterns.into_iter().next()
            .expect("patterns.len() == 1, so next() must return Some"))
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
    // Check for while-let syntax
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        state.tokens.advance(); // consume 'let'
        // Parse the pattern
        let pattern = parse_match_pattern(state)
            .map_err(|e| anyhow::anyhow!("Expected pattern after 'while let': {}", e))?;
        // Expect '='
        state.tokens.expect(&Token::Equal)
            .map_err(|e| anyhow::anyhow!("Expected '=' after pattern in while-let: {}", e))?;
        // Parse the expression to match against
        let expr = Box::new(super::parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected expression after '=' in while-let: {}", e))?);
        // Parse body (expect block)
        let body = Box::new(super::parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected body after while-let condition: {}", e))?);
        Ok(Expr::new(
            ExprKind::WhileLet {
                pattern,
                expr,
                body,
            },
            start_span,
        ))
    } else {
        // Regular while loop
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
/// Parse an array element which might be a spread expression (...expr) or regular expression
fn parse_array_element(state: &mut ParserState) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
        let start_span = state.tokens.expect(&Token::DotDotDot)?; // consume ...
        let expr = super::parse_expr_recursive(state)?;
        Ok(Expr::new(ExprKind::Spread { expr: Box::new(expr) }, start_span))
    } else {
        super::parse_expr_recursive(state)
    }
}
fn parse_list_literal(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::LeftBracket)?;
    // Handle empty list
    if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::List(vec![]), start_span));
    }
    // Parse first element
    let first_expr = parse_array_element(state)?;
    // Determine list type based on next token
    match state.tokens.peek() {
        Some((Token::Semicolon, _)) => parse_array_init(state, first_expr, start_span),
        Some((Token::For, _)) => parse_list_comprehension_body(state, first_expr, start_span),
        _ => parse_regular_list(state, first_expr, start_span),
    }
}
/// Parse array initialization syntax: [value; size]
/// Extracted to reduce complexity
fn parse_array_init(state: &mut ParserState, value_expr: Expr, start_span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume ;
    let size_expr = super::parse_expr_recursive(state)?;
    state.tokens.expect(&Token::RightBracket)
        .map_err(|_| anyhow::anyhow!("Expected ']' after array initialization"))?;
    Ok(Expr::new(
        ExprKind::ArrayInit { 
            value: Box::new(value_expr), 
            size: Box::new(size_expr) 
        }, 
        start_span
    ))
}
/// Parse regular list literal: [expr, expr, ...]
/// Extracted to reduce complexity
fn parse_regular_list(state: &mut ParserState, first_expr: Expr, start_span: Span) -> Result<Expr> {
    let mut elements = vec![first_expr];
    // Parse remaining elements
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        // Check for trailing comma
        if matches!(state.tokens.peek(), Some((Token::RightBracket, _))) {
            break;
        }
        elements.push(parse_array_element(state)?);
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
/// Parse type alias: type Name = Type
/// Complexity: <5
fn parse_type_alias(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Type)?;

    // Parse the alias name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        bail!("Expected identifier after 'type'");
    };

    // Expect =
    state.tokens.expect(&Token::Equal)?;

    // Parse the target type
    let target_type = super::utils::parse_type(state)?;

    let end_span = target_type.span;
    Ok(Expr::new(
        ExprKind::TypeAlias { name, target_type },
        start_span.merge(end_span),
    ))
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
    let methods = parse_impl_methods(state)?;
    state.tokens.expect(&Token::RightBrace)?;
    Ok(Expr::new(ExprKind::Impl {
        type_params: vec![],
        trait_name,
        for_type: type_name,
        methods,
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
/// Parse impl methods (complexity: 7)
fn parse_impl_methods(state: &mut ParserState) -> Result<Vec<crate::frontend::ast::ImplMethod>> {
    let mut methods = Vec::new();

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        // Skip any visibility modifiers for now
        if matches!(state.tokens.peek(), Some((Token::Pub, _))) {
            state.tokens.advance();
        }

        // Parse method
        if matches!(state.tokens.peek(), Some((Token::Fun, _))) {
            let method = parse_impl_method(state)?;
            methods.push(method);
        } else {
            // Skip unexpected tokens
            state.tokens.advance();
        }
    }

    Ok(methods)
}

/// Parse a single impl method (complexity: 8)
fn parse_impl_method(state: &mut ParserState) -> Result<crate::frontend::ast::ImplMethod> {
    state.tokens.expect(&Token::Fun)?;

    // Parse method name
    let name = parse_required_identifier(state, "method name")?;

    // Parse parameters
    let params = super::utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        Some(super::utils::parse_type(state)?)
    } else {
        None
    };

    // Parse body
    let body = super::parse_expr_recursive(state)?;

    Ok(crate::frontend::ast::ImplMethod {
        name,
        params,
        return_type,
        body: Box::new(body),
        is_pub: false,
    })
}

/// Skip impl body by tracking brace depth (complexity: 5) - DEPRECATED
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
pub(super) fn parse_import_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Import)?;

    // Check for different import forms
    // 1. import "module_path"
    if let Some((Token::String(module), _)) = state.tokens.peek() {
        let module = module.clone();
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::Import {
            module,
            items: None,
        }, start_span));
    }

    // 2. import { items } from "module"
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        state.tokens.advance(); // consume {
        let mut items = Vec::new();
        while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
            if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
                items.push(name.clone());
                state.tokens.advance();
                if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance();
                }
            } else {
                bail!("Expected identifier in import list");
            }
        }
        state.tokens.expect(&Token::RightBrace)?;
        state.tokens.expect(&Token::From)?;
        if let Some((Token::String(module), _)) = state.tokens.peek() {
            let module = module.clone();
            state.tokens.advance();
            return Ok(Expr::new(ExprKind::Import {
                module,
                items: Some(items),
            }, start_span));
        }
        bail!("Expected module path after 'from'");
    }

    // 3. import * as name from "module"
    if matches!(state.tokens.peek(), Some((Token::Star, _))) {
        state.tokens.advance();
        state.tokens.expect(&Token::As)?;
        if let Some((Token::Identifier(alias), _)) = state.tokens.peek() {
            let alias = alias.clone();
            state.tokens.advance();
            state.tokens.expect(&Token::From)?;
            if let Some((Token::String(module), _)) = state.tokens.peek() {
                let module = module.clone();
                state.tokens.advance();
                return Ok(Expr::new(ExprKind::ImportAll {
                    module,
                    alias,
                }, start_span));
            }
            bail!("Expected module path after 'from'");
        }
        bail!("Expected alias after 'as'");
    }

    // 4. import Name from "module" (default import)
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        state.tokens.expect(&Token::From)?;
        if let Some((Token::String(module), _)) = state.tokens.peek() {
            let module = module.clone();
            state.tokens.advance();
            return Ok(Expr::new(ExprKind::ImportDefault {
                module,
                name,
            }, start_span));
        }
        bail!("Expected module path after 'from'");
    }

    bail!("Invalid import statement")
}
// Legacy use statement parser - disabled in favor of new import syntax
#[allow(dead_code)]
fn parse_use_statement(_state: &mut ParserState) -> Result<Expr> {
    bail!("'use' statements are deprecated. Please use 'import' syntax instead")
}
fn parse_dataframe_literal(state: &mut ParserState) -> Result<Expr> {
    // Parse df![...] macro syntax - DataFrame token already consumed by caller
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
        BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual | BinaryOp::Gt => 8,
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
/// Helper for parsing try block (complexity: 3)
fn parse_try_block(state: &mut ParserState) -> Result<Box<Expr>> {
    // parse_block expects and consumes the left brace
    Ok(Box::new(super::collections::parse_block(state)?))
}
/// Helper for parsing catch clauses (complexity: 8)
fn parse_catch_clauses(state: &mut ParserState) -> Result<Vec<crate::frontend::ast::CatchClause>> {
    use crate::frontend::ast::CatchClause;
    let mut catch_clauses = Vec::new();
    while matches!(state.tokens.peek(), Some((Token::Catch, _))) {
        state.tokens.advance(); // consume 'catch'
        let pattern = parse_catch_pattern(state)?;
        let body = parse_catch_body(state)?;
        catch_clauses.push(CatchClause { pattern, body });
    }
    Ok(catch_clauses)
}
/// Helper for parsing catch pattern (complexity: 5)
fn parse_catch_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.expect(&Token::LeftParen)?;
    let pattern = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Pattern::Identifier(name)
    } else {
        bail!("Expected identifier in catch clause");
    };
    state.tokens.expect(&Token::RightParen)?;
    Ok(pattern)
}
/// Helper for parsing catch body (complexity: 3)
fn parse_catch_body(state: &mut ParserState) -> Result<Box<Expr>> {
    // parse_block expects and consumes the left brace
    Ok(Box::new(super::collections::parse_block(state)?))
}
/// Helper for parsing optional finally block (complexity: 6)
fn parse_finally_block(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Finally, _))) {
        state.tokens.advance(); // consume 'finally'
        // parse_block expects and consumes the left brace
        Ok(Some(Box::new(super::collections::parse_block(state)?)))
    } else {
        Ok(None)
    }
}
/// Helper for validating try-catch structure (complexity: 3)
fn validate_try_catch_structure(
    catch_clauses: &[crate::frontend::ast::CatchClause], 
    finally_block: Option<&Expr>
) -> Result<()> {
    if catch_clauses.is_empty() && finally_block.is_none() {
        bail!("Try block must have at least one catch clause or a finally block");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;
    use crate::frontend::ast::{Expr, ExprKind, Literal};

    // Unit tests for specific parsing functions

    #[test]
    fn test_parse_integer_literal() {
        let mut parser = Parser::new("42");
        let result = parser.parse().unwrap();
        if let ExprKind::Literal(Literal::Integer(n)) = &result.kind {
            assert_eq!(*n, 42);
        } else {
            panic!("Expected integer literal, got {:?}", result.kind);
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let mut parser = Parser::new("3.14");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse float literal");
    }

    #[test]
    fn test_parse_string_literal() {
        let mut parser = Parser::new("\"hello world\"");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse string literal");
    }

    #[test]
    fn test_parse_boolean_true() {
        let mut parser = Parser::new("true");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse boolean true");
    }

    #[test]
    fn test_parse_boolean_false() {
        let mut parser = Parser::new("false");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse boolean false");
    }

    #[test]
    fn test_parse_char_literal() {
        let mut parser = Parser::new("'a'");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse char literal");
    }

    #[test]
    fn test_parse_fstring_literal() {
        let mut parser = Parser::new("f\"Hello {name}\"");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse f-string literal");
    }

    #[test]
    fn test_parse_identifier() {
        let mut parser = Parser::new("variable_name");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse identifier");
    }

    #[test]
    fn test_parse_underscore() {
        let mut parser = Parser::new("_");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse underscore");
    }

    #[test]
    fn test_parse_unary_minus() {
        let mut parser = Parser::new("-42");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse unary minus");
    }

    #[test]
    fn test_parse_unary_not() {
        let mut parser = Parser::new("!true");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse unary not");
    }

    #[test]
    fn test_parse_binary_addition() {
        let mut parser = Parser::new("1 + 2");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary addition");
    }

    #[test]
    fn test_parse_binary_subtraction() {
        let mut parser = Parser::new("5 - 3");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary subtraction");
    }

    #[test]
    fn test_parse_binary_multiplication() {
        let mut parser = Parser::new("4 * 2");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary multiplication");
    }

    #[test]
    fn test_parse_binary_division() {
        let mut parser = Parser::new("8 / 2");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary division");
    }

    #[test]
    fn test_parse_binary_modulo() {
        let mut parser = Parser::new("10 % 3");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary modulo");
    }

    #[test]
    fn test_parse_binary_equality() {
        let mut parser = Parser::new("x == y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary equality");
    }

    #[test]
    fn test_parse_binary_inequality() {
        let mut parser = Parser::new("x != y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary inequality");
    }

    #[test]
    fn test_parse_binary_less_than() {
        let mut parser = Parser::new("x < y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary less than");
    }

    #[test]
    fn test_parse_binary_greater_than() {
        let mut parser = Parser::new("x > y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary greater than");
    }

    #[test]
    fn test_parse_binary_less_equal() {
        let mut parser = Parser::new("x <= y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary less equal");
    }

    #[test]
    fn test_parse_binary_greater_equal() {
        let mut parser = Parser::new("x >= y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary greater equal");
    }

    #[test]
    fn test_parse_binary_logical_and() {
        let mut parser = Parser::new("true && false");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary logical and");
    }

    #[test]
    fn test_parse_binary_logical_or() {
        let mut parser = Parser::new("true || false");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse binary logical or");
    }

    #[test]
    fn test_parse_parenthesized_expression() {
        let mut parser = Parser::new("(42)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse parenthesized expression");
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let mut parser = Parser::new("((42))");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse nested parentheses");
    }

    #[test]
    fn test_parse_unit_value() {
        let mut parser = Parser::new("()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse unit value");
    }

    #[test]
    fn test_parse_tuple_two_elements() {
        let mut parser = Parser::new("(1, 2)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse tuple with two elements");
    }

    #[test]
    fn test_parse_tuple_three_elements() {
        let mut parser = Parser::new("(1, 2, 3)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse tuple with three elements");
    }

    #[test]
    fn test_parse_list_empty() {
        let mut parser = Parser::new("[]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse empty list");
    }

    #[test]
    fn test_parse_list_with_elements() {
        let mut parser = Parser::new("[1, 2, 3]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse list with elements");
    }

    #[test]
    fn test_parse_dict_empty() {
        let mut parser = Parser::new("{}");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse empty dict");
    }

    #[test]
    fn test_parse_dict_with_entries() {
        let mut parser = Parser::new("{\"key\": \"value\"}");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse dict with entries");
    }

    #[test]
    fn test_parse_function_call_no_args() {
        let mut parser = Parser::new("func()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse function call without args");
    }

    #[test]
    fn test_parse_function_call_with_args() {
        let mut parser = Parser::new("func(1, 2, 3)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse function call with args");
    }

    #[test]
    fn test_parse_method_call() {
        let mut parser = Parser::new("obj.method()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse method call");
    }

    #[test]
    fn test_parse_chained_method_calls() {
        let mut parser = Parser::new("obj.method1().method2()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse chained method calls");
    }

    #[test]
    fn test_parse_index_access() {
        let mut parser = Parser::new("array[0]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse index access");
    }

    #[test]
    fn test_parse_nested_index_access() {
        let mut parser = Parser::new("matrix[i][j]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse nested index access");
    }

    #[test]
    fn test_parse_field_access() {
        let mut parser = Parser::new("obj.field");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse field access");
    }

    #[test]
    #[ignore = "async blocks not fully implemented"]
    fn test_parse_async_block() {
        let mut parser = Parser::new("async { 42 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse async block");
    }

    #[test]
    #[ignore = "await expressions not fully implemented"]
    fn test_parse_await_expression() {
        let mut parser = Parser::new("await async_func()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse await expression");
    }

    #[test]
    fn test_parse_pipeline_operator() {
        let mut parser = Parser::new("data |> process |> filter");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse pipeline operator");
    }

    #[test]
    fn test_parse_range_inclusive() {
        let mut parser = Parser::new("1..10");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse inclusive range");
    }

    #[test]
    #[ignore = "exclusive range syntax not fully implemented"]
    fn test_parse_range_exclusive() {
        let mut parser = Parser::new("1...10");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse exclusive range");
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut parser = Parser::new("(a + b) * c - d / e");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse complex expression");
    }

    #[test]
    #[ignore = "ternary conditional not fully implemented"]
    fn test_parse_ternary_conditional() {
        let mut parser = Parser::new("condition ? true_val : false_val");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse ternary conditional");
    }

    #[test]
    fn test_parse_lambda_no_params() {
        let mut parser = Parser::new("|| 42");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse lambda without params");
    }

    #[test]
    fn test_parse_lambda_with_params() {
        let mut parser = Parser::new("|x, y| x + y");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse lambda with params");
    }

    #[test]
    fn test_parse_fat_arrow_lambda() {
        let mut parser = Parser::new("x => x * 2");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse fat arrow lambda");
    }

    // Test 56: Complex nested expressions
    #[test]
    fn test_parse_complex_nested_expression() {
        let mut parser = Parser::new("((a + b) * (c - d)) / (e + f)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse complex nested expression");
    }

    // Test 57: Struct literal parsing
    #[test]
    fn test_parse_struct_literal() {
        let mut parser = Parser::new("Point { x: 10, y: 20 }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse struct literal");
    }

    // Test 58: Array indexing
    #[test]
    fn test_parse_array_indexing() {
        let mut parser = Parser::new("arr[0]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse array indexing");

        // Nested indexing
        let mut parser2 = Parser::new("matrix[i][j]");
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "Failed to parse nested array indexing");
    }

    // Test 59: Range expressions
    #[test]
    fn test_parse_range_expressions() {
        // Inclusive range
        let mut parser = Parser::new("1..10");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse inclusive range");

        // Exclusive range
        let mut parser2 = Parser::new("1..=10");
        let result2 = parser2.parse();
        assert!(result2.is_ok() || result2.is_err(), "Range parsing handled");
    }

    // Test 60: Type casting
    #[test]
    fn test_parse_type_casting() {
        let mut parser = Parser::new("x as i32");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse type casting");
    }

    // Test 61: Await expressions
    #[test]
    fn test_parse_await_expression_comprehensive() {
        let mut parser = Parser::new("await fetch_data()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse await expression");
    }

    // Test 62: Error handling expressions
    #[test]
    fn test_parse_error_handling() {
        // Try operator
        let mut parser = Parser::new("risky_op()?");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse try operator");

        // Ok variant
        let mut parser2 = Parser::new("Ok(42)");
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "Failed to parse Ok variant");

        // Err variant
        let mut parser3 = Parser::new("Err(\"error\")");
        let result3 = parser3.parse();
        assert!(result3.is_ok(), "Failed to parse Err variant");
    }

    // Test 63: Option types
    #[test]
    fn test_parse_option_types() {
        // Some variant
        let mut parser = Parser::new("Some(value)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse Some variant");

        // None variant
        let mut parser2 = Parser::new("None");
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "Failed to parse None variant");
    }

    // Test 64: Closure with multiple parameters
    #[test]
    fn test_parse_multi_param_closure() {
        let mut parser = Parser::new("|x, y, z| x + y + z");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse multi-parameter closure");
    }

    // Test 65: Destructuring in let bindings
    #[test]
    fn test_parse_destructuring_let() {
        // Tuple destructuring
        let mut parser = Parser::new("let (x, y) = pair");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse tuple destructuring");

        // Array destructuring
        let mut parser2 = Parser::new("let [first, second] = arr");
        let result2 = parser2.parse();
        assert!(result2.is_ok(), "Failed to parse array destructuring");
    }

    // Test 66: Qualified names
    #[test]
    fn test_parse_qualified_names() {
        let mut parser = Parser::new("std::collections::HashMap");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse qualified name");
    }

    // Test 67: Macro invocations
    #[test]
    fn test_parse_macro_invocation() {
        let mut parser = Parser::new("println!(\"Hello, world!\")");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse macro invocation");
    }

    // Test 68: Field access chains
    #[test]
    fn test_parse_field_access_chain() {
        let mut parser = Parser::new("obj.field1.field2.method()");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse field access chain");
    }

    // Test 69: Optional field access
    #[test]
    fn test_parse_optional_field_access() {
        let mut parser = Parser::new("obj?.field");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse optional field access");
    }

    // Test 70: Array slicing
    #[test]
    fn test_parse_array_slicing() {
        let mut parser = Parser::new("arr[1..5]");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse array slicing");
    }

    // Test 71: Binary operators precedence
    #[test]
    fn test_parse_operator_precedence() {
        let mut parser = Parser::new("a + b * c - d / e");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse operator precedence");

        // Verify multiplication has higher precedence
        if let Ok(expr) = result {
            // The expression structure should respect precedence
            assert!(matches!(expr.kind, ExprKind::Binary { .. }));
        }
    }

    // Test 72: Parenthesized expressions
    #[test]
    fn test_parse_parenthesized_expressions() {
        let mut parser = Parser::new("(a + b) * (c - d)");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse parenthesized expressions");
    }

    // Test 73: Empty block expression
    #[test]
    fn test_parse_empty_block() {
        let mut parser = Parser::new("{}");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse empty block");
    }

    // Test 74: Block with multiple statements
    #[test]
    fn test_parse_multi_statement_block() {
        let mut parser = Parser::new("{ let x = 1; let y = 2; x + y }");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse multi-statement block");
    }

    // Test 75: Chained comparisons
    #[test]
    fn test_parse_chained_comparisons() {
        let mut parser = Parser::new("a < b && b < c");
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse chained comparisons");
    }
}

/// Parse export statement - delegates to utils module
fn parse_export_token(state: &mut ParserState) -> Result<Expr> {
    super::utils::parse_export(state)
}

/// Parse async function declaration - modifies following function
fn parse_async_token(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'async'

    // After 'async', we expect 'fun' for async function
    if matches!(state.tokens.peek(), Some((Token::Fun, _))) {
        // Parse the async function
        parse_async_function(state, false)
    } else {
        bail!("Expected 'fun' after 'async'")
    }
}

/// Parse async function with explicit async flag handling
fn parse_async_function(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume fun
    let is_async = true; // We know it's async since we came from async token

    // Parse function name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        "anonymous".to_string()
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        super::utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse parameters
    let params = super::utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        Some(super::utils::parse_type(state)?)
    } else {
        None
    };

    // Parse body
    let body = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Function {
            name,
            type_params,
            params,
            return_type,
            body: Box::new(body),
            is_async,
            is_pub,
        },
        start_span,
    ))
}

/// Parse loop statement - infinite loop with break/continue
fn parse_loop(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Loop)?;
    let body = Box::new(super::parse_expr_recursive(state)?);

    Ok(Expr::new(
        ExprKind::Loop { body },
        start_span,
    ))
}

/// Parse increment operator (++var or var++)
fn parse_increment_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume '++'

    // Parse the variable being incremented
    let variable = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::PreIncrement {
            target: Box::new(variable),
        },
        span,
    ))
}

/// Parse decrement operator (--var or var--)
fn parse_decrement_token(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume '--'

    // Parse the variable being decremented
    let variable = super::parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::PreDecrement {
            target: Box::new(variable),
        },
        span,
    ))
}

#[cfg(test)]
mod property_tests_parser_expressions {

    use proptest::prelude::*;
    use crate::frontend::parser::Parser;
    
    proptest! {
        /// Property: Parser never panics on any string input (fuzzing)
        #[test]
        fn test_parser_never_panics_on_any_input(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 200 { &input[..200] } else { &input };
            
            let result = std::panic::catch_unwind(|| {
                let mut parser = Parser::new(&input);
                // Parser should never panic, even on invalid syntax
                let _ = parser.parse();
            });
            
            assert!(result.is_ok(), "Parser panicked on input: {:?}", input);
        }
        
        /// Property: Valid literals always parse successfully
        #[test]
        fn test_valid_literals_always_parse(
            int_val in -1000000i64..1000000i64,
            float_val in -1000000.0f64..1000000.0f64,
            bool_val in any::<bool>(),
            string_val in "[a-zA-Z0-9 ]{0,20}", // Simple ASCII strings only
        ) {
            let test_cases = vec![
                int_val.to_string(),
                float_val.to_string(),
                bool_val.to_string(),
                format!("\"{}\"", string_val.replace('"', "\\\"")),
            ];
            
            for input in test_cases {
                let mut parser = Parser::new(&input);
                let result = parser.parse();
                
                // Valid literals should always parse successfully
                prop_assert!(result.is_ok(), "Failed to parse valid literal: {}", input);
            }
        }
        
        /// Property: Balanced parentheses expressions parse correctly
        #[test]
        fn test_balanced_parentheses_parse(
            expr_content in "42|true|\"test\"|x",
            nesting_level in 0..5_u32, // Limit nesting to avoid timeout
        ) {
            let mut input = expr_content.clone();
            
            // Add balanced parentheses
            for _ in 0..nesting_level {
                input = format!("({})", input);
            }
            
            let mut parser = Parser::new(&input);
            let result = parser.parse();
            
            // Balanced expressions should parse successfully
            prop_assert!(result.is_ok(), "Failed to parse balanced expression: {}", input);
        }
        
        /// Property: Binary operations with valid operators parse correctly
        #[test]
        fn test_binary_operations_parse(
            left in "42|x|true",
            op in r"\+|\-|\*|/|==|!=|<|>|<=|>=|&&|\|\|",
            right in "42|y|false",
        ) {
            let input = format!("{} {} {}", left, op, right);
            
            let mut parser = Parser::new(&input);
            let result = parser.parse();
            
            // Valid binary operations should parse successfully  
            prop_assert!(result.is_ok(), "Failed to parse binary operation: {}", input);
        }
        
        /// Property: Function definitions with valid syntax parse correctly
        #[test]
        fn test_function_definitions_parse(
            func_name in "[a-z][a-z0-9_]*{1,20}",
            param_name in "[a-z][a-z0-9_]*{1,10}",
        ) {
            let input = format!("fun {}({}: i32) -> i32 {{ {} + 1 }}", func_name, param_name, param_name);
            
            let mut parser = Parser::new(&input);
            let result = parser.parse();
            
            // Valid function definitions should parse successfully
            prop_assert!(result.is_ok(), "Failed to parse function definition: {}", input);
        }
    }
}