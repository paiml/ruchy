//! Pattern matching and destructuring parsing
//!
//! Handles parsing of all pattern constructs in Ruchy:
//! - Identifier patterns: `x`, `_`
//! - Tuple patterns: `(a, b, c)`
//! - List patterns: `[first, ...rest]`
//! - Struct patterns: `Point { x, y }`
//! - Variant patterns: `Some(x)`, `Ok(val)`, `Err(e)`
//! - Or patterns: `Some(x) | None`
//! - Literal patterns: `42`, `"hello"`, `true`
//! - Range patterns: `1..10`, `1..=100`
//!
//! # Examples
//! ```ruchy
//! // Tuple destructuring
//! let (x, y, z) = (1, 2, 3)
//!
//! // List destructuring with rest
//! let [first, ...rest] = [1, 2, 3, 4]
//!
//! // Struct destructuring
//! let Point { x, y } = point
//!
//! // Variant patterns in match
//! match result {
//!     Ok(value) => handle(value),
//!     Err(e) => log(e)
//! }
//!
//! // Or patterns
//! match value {
//!     Some(x) | None => process(x)
//! }
//! ```
//!
//! This is the largest extraction from expressions.rs (1,130+ lines),
//! representing Phase 17 of the modularization effort.

use crate::frontend::ast::{Expr, ExprKind, Literal, MatchArm, Pattern, Span, Type};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

fn parse_variant_pattern_with_name(state: &mut ParserState, variant_name: String) -> Result<Pattern> {
    // At this point, we've consumed the variant name and peeked '('
    state.tokens.expect(&Token::LeftParen)?;

    // Parse patterns (could be single or multiple comma-separated)
    let mut patterns = vec![];

    // Parse first pattern
    if !matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        patterns.push(parse_single_pattern(state)?);

        // Parse additional patterns separated by commas
        while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance(); // consume comma

            // Check for trailing comma
            if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
                break;
            }

            patterns.push(parse_single_pattern(state)?);
        }
    }

    state.tokens.expect(&Token::RightParen)?;

    // Try to create special pattern for common variants
    create_pattern_for_variant(variant_name, patterns)
}

/// Create pattern for variant (special cases for Some/Ok/Err, otherwise `TupleVariant`)
fn create_pattern_for_variant(variant_name: String, patterns: Vec<Pattern>) -> Result<Pattern> {
    // Special case for common Option/Result variants (single element)
    if patterns.len() == 1 {
        match variant_name.as_str() {
            "Some" => return Ok(Pattern::Some(Box::new(patterns.into_iter().next().unwrap()))),
            "Ok" => return Ok(Pattern::Ok(Box::new(patterns.into_iter().next().unwrap()))),
            "Err" => return Ok(Pattern::Err(Box::new(patterns.into_iter().next().unwrap()))),
            _ => {}
        }
    }

    // For other variants or multiple elements, use TupleVariant
    Ok(Pattern::TupleVariant {
        path: vec![variant_name],
        patterns,
    })
}

/// Parse pattern for let statement (identifier or destructuring)
/// Extracted from `parse_let_statement` to reduce complexity
pub(in crate::frontend::parser) fn parse_let_pattern(state: &mut ParserState, is_mutable: bool) -> Result<Pattern> {
    match state.tokens.peek() {
        // Handle Option::Some pattern
        Some((Token::Some, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Some".to_string())
            } else {
                bail!("Some must be followed by parentheses in patterns: Some(value)")
            }
        }
        // Handle Result::Ok pattern
        Some((Token::Ok, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Ok".to_string())
            } else {
                bail!("Ok must be followed by parentheses in patterns: Ok(value)")
            }
        }
        // Handle Result::Err pattern
        Some((Token::Err, _)) => {
            state.tokens.advance();
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                parse_variant_pattern_with_name(state, "Err".to_string())
            } else {
                bail!("Err must be followed by parentheses in patterns: Err(value)")
            }
        }
        // Handle Option::None pattern
        Some((Token::None, _)) => {
            state.tokens.advance();
            Ok(Pattern::None)
        }
        Some((Token::Identifier(name), _)) => {
            let name = name.clone();
            state.tokens.advance();

            // Check if this is a variant pattern with custom variants
            if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
                // Parse enum variant pattern with tuple destructuring
                parse_variant_pattern_with_name(state, name)
            }
            // Check if this is a struct pattern: Name { ... }
            else if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
                parse_struct_pattern_with_name(state, name)
            } else {
                Ok(Pattern::Identifier(name))
            }
        }
        Some((Token::DataFrame, _)) => {
            // Allow 'df' as a variable name (common in data science)
            state.tokens.advance();
            Ok(Pattern::Identifier("df".to_string()))
        }
        Some((Token::Default, _)) => {
            // Allow 'default' as a variable name (common in configurations)
            state.tokens.advance();
            Ok(Pattern::Identifier("default".to_string()))
        }
        Some((Token::Final, _)) => {
            // Allow 'final' as a variable name (Rust keyword, needs r# prefix in transpiler)
            state.tokens.advance();
            Ok(Pattern::Identifier("final".to_string()))
        }
        Some((Token::Underscore, _)) => {
            // Allow wildcard pattern
            state.tokens.advance();
            Ok(Pattern::Identifier("_".to_string()))
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
        _ => bail!(
            "Expected identifier or pattern after 'let{}'",
            if is_mutable { " mut" } else { "" }
        ),
    }
}
/// Parse optional type annotation for let statement
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}
/// Parse optional 'else' clause for let-else patterns
/// Extracted to reduce complexity
fn parse_let_else_clause(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Else, _))) {
        state.tokens.advance(); // consume 'else'
        // Must be followed by a block (diverging expression)
        if !matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            bail!("let-else requires a block after 'else'");
        }
        let block = parse_expr_recursive(state)?;
        Ok(Some(Box::new(block)))
    } else {
        Ok(None)
    }
}

/// Parse optional 'in' clause for let expressions
/// Extracted from `parse_let_statement` to reduce complexity
fn parse_let_in_clause(state: &mut ParserState, value_span: Span) -> Result<Box<Expr>> {
    if matches!(state.tokens.peek(), Some((Token::In, _))) {
        state.tokens.advance(); // consume 'in'
        Ok(Box::new(parse_expr_recursive(state)?))
    } else {
        // For let statements (no 'in'), body is unit
        Ok(Box::new(Expr::new(
            ExprKind::Literal(Literal::Unit),
            value_span,
        )))
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
    else_block: Option<Box<Expr>>,
    start_span: Span,
) -> Result<Expr> {
    let end_span = body.span;
    match &pattern {
        Pattern::Identifier(name) => Ok(Expr::new(
            ExprKind::Let {
                name: name.clone(),
                type_annotation,
                value,
                body,
                is_mutable,
                else_block,
            },
            start_span.merge(end_span),
        )),
        Pattern::Tuple(_) | Pattern::List(_) => {
            // For destructuring patterns, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                    else_block,
                },
                start_span.merge(end_span),
            ))
        }
        Pattern::Wildcard
        | Pattern::Literal(_)
        | Pattern::QualifiedName(_)
        | Pattern::Struct { .. }
        | Pattern::TupleVariant { .. }
        | Pattern::Range { .. }
        | Pattern::Or(_)
        | Pattern::Rest
        | Pattern::RestNamed(_)
        | Pattern::AtBinding { .. }
        | Pattern::WithDefault { .. }
        | Pattern::Ok(_)
        | Pattern::Err(_)
        | Pattern::Some(_)
        | Pattern::None
        | Pattern::Mut(_) => {
            // For other pattern types, use LetPattern variant
            Ok(Expr::new(
                ExprKind::LetPattern {
                    pattern,
                    type_annotation,
                    value,
                    body,
                    is_mutable,
                    else_block,
                },
                start_span.merge(end_span),
            ))
        }
    }
}
// Var statement parsing moved to expressions_helpers/variable_declarations.rs module
pub(in crate::frontend::parser) fn parse_var_statement(state: &mut ParserState) -> Result<Expr> {
    super::variable_declarations::parse_var_statement(state)
}

/// Extract method: Parse variable pattern - complexity: 6
pub(in crate::frontend::parser) fn parse_var_pattern(state: &mut ParserState) -> Result<Pattern> {
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
        Some((Token::Underscore, _)) => {
            // Allow wildcard pattern in var statements too
            state.tokens.advance();
            Ok(Pattern::Identifier("_".to_string()))
        }
        Some((Token::LeftParen, _)) => parse_tuple_pattern(state),
        Some((Token::LeftBracket, _)) => parse_list_pattern(state),
        _ => bail!("Expected identifier or pattern after 'var'"),
    }
}

/// Extract method: Parse optional type annotation - complexity: 4
fn parse_optional_type_annotation(
    state: &mut ParserState,
) -> Result<Option<crate::frontend::ast::Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Extract method: Create variable expression - complexity: 6
fn create_var_expression(
    pattern: Pattern,
    type_annotation: Option<crate::frontend::ast::Type>,
    value: Box<Expr>,
    start_span: Span,
) -> Result<Expr> {
    let body = Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span));
    let end_span = value.span;
    let is_mutable = true;

    match &pattern {
        Pattern::Identifier(name) => Ok(Expr::new(
            ExprKind::Let {
                name: name.clone(),
                type_annotation,
                value,
                body,
                is_mutable,
                else_block: None,  // var doesn't support let-else
            },
            start_span.merge(end_span),
        )),
        _ => Ok(Expr::new(
            ExprKind::LetPattern {
                pattern,
                type_annotation,
                value,
                body,
                is_mutable,
                else_block: None,  // var doesn't support let-else
            },
            start_span.merge(end_span),
        )),
    }
}
pub(in crate::frontend::parser) fn parse_tuple_pattern(state: &mut ParserState) -> Result<Pattern> {
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
/// Complexity: 7 (within Toyota Way limits)
fn parse_single_tuple_pattern_element(state: &mut ParserState) -> Result<Pattern> {
    // Check for 'mut' modifier
    let is_mut = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume 'mut'
        true
    } else {
        false
    };

    // Parse the pattern element
    let pattern = match state.tokens.peek() {
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
    }?;

    // Wrap in Mut pattern if mut modifier was present
    if is_mut {
        Ok(Pattern::Mut(Box::new(pattern)))
    } else {
        Ok(pattern)
    }
}
pub(in crate::frontend::parser) fn parse_struct_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance(); // consume '{'
    parse_struct_pattern_fields(state, String::new())
}

/// Parse struct pattern with a specific name: Point { x, y }
pub(in crate::frontend::parser) fn parse_struct_pattern_with_name(state: &mut ParserState, name: String) -> Result<Pattern> {
    state.tokens.advance(); // consume '{'
    parse_struct_pattern_fields(state, name)
}

/// Parse struct pattern fields (shared logic for both named and anonymous patterns)
fn parse_struct_pattern_fields(state: &mut ParserState, name: String) -> Result<Pattern> {
    let mut fields = Vec::new();
    let mut has_rest = false;

    while !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        if matches!(state.tokens.peek(), Some((Token::DotDot, _))) {
            has_rest = parse_struct_rest_pattern(state)?;
            break;
        }

        fields.push(parse_struct_field_pattern(state)?);

        if !handle_struct_field_separator(state)? {
            break;
        }
    }

    state.tokens.expect(&Token::RightBrace)?;

    Ok(Pattern::Struct {
        name,
        fields,
        has_rest,
    })
}

fn parse_struct_rest_pattern(state: &mut ParserState) -> Result<bool> {
    state.tokens.advance(); // consume '..'

    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
    }

    if !matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        bail!("Rest pattern (..) must be the last field in struct pattern");
    }

    Ok(true)
}

fn parse_struct_field_pattern(
    state: &mut ParserState,
) -> Result<crate::frontend::ast::StructPatternField> {
    let field_name = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        name.clone()
    } else {
        bail!("Expected identifier or '..' in struct pattern")
    };
    state.tokens.advance();

    let pattern = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Some(parse_match_pattern(state)?)
    } else {
        None
    };

    Ok(crate::frontend::ast::StructPatternField {
        name: field_name,
        pattern,
    })
}

fn handle_struct_field_separator(state: &mut ParserState) -> Result<bool> {
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance();
        // Trailing comma before closing brace is ok
        Ok(!matches!(state.tokens.peek(), Some((Token::RightBrace, _))))
    } else if matches!(state.tokens.peek(), Some((Token::RightBrace, _))) {
        Ok(false)
    } else {
        bail!("Expected comma or closing brace after struct pattern field")
    }
}
pub(in crate::frontend::parser) fn parse_list_pattern(state: &mut ParserState) -> Result<Pattern> {
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
        let default_expr = parse_expr_recursive(state)?;
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
            bail!("Expected {expected} in pattern");
        }
        Ok(false)
    } else {
        bail!("Unexpected end of input in pattern")
    }
}
/// Parse if expression: if condition { `then_branch` } [else { `else_branch` }]
/// Also handles if-let: if let pattern = expr { `then_branch` } [else { `else_branch` }]
/// Complexity: <10 (split into helper functions)
pub(in crate::frontend::parser) fn parse_if_expression(state: &mut ParserState) -> Result<Expr> {
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
        .map_err(|e| anyhow::anyhow!("Expected pattern after 'if let': {e}"))?;
    // Expect '='
    state
        .tokens
        .expect(&Token::Equal)
        .map_err(|e| anyhow::anyhow!("Expected '=' after pattern in if-let: {e}"))?;
    // Parse the expression to match against
    let expr = Box::new(
        parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected expression after '=' in if-let: {e}"))?,
    );
    // Parse then branch
    let then_branch = Box::new(parse_expr_recursive(state).map_err(|e| {
        anyhow::anyhow!(
            "Expected body after if-let condition, typically {{ ... }}: {e}"
        )
    })?);
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
    let condition = Box::new(
        parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected condition after 'if': {e}"))?,
    );
    // Parse then branch (expect block) with better error context
    let then_branch = Box::new(parse_expr_recursive(state).map_err(|e| {
        anyhow::anyhow!(
            "Expected body after if condition, typically {{ ... }}: {e}"
        )
    })?);
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
            Ok(Some(Box::new(parse_expr_recursive(state).map_err(
                |e| anyhow::anyhow!("Expected body after 'else', typically {{ ... }}: {e}"),
            )?)))
        }
    } else {
        Ok(None)
    }
}
/// Parse match expression: match expr { pattern => result, ... }
/// Complexity target: <10 (using helper functions for TDG compliance)
pub(in crate::frontend::parser) fn parse_match_expression(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Match)?;
    // Parse the expression to match on
    let expr = Box::new(
        parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected expression after 'match': {e}"))?,
    );
    // Expect opening brace for match arms
    state
        .tokens
        .expect(&Token::LeftBrace)
        .map_err(|_| anyhow::anyhow!("Expected '{{' after match expression"))?;
    // Parse match arms
    let arms = parse_match_arms(state)?;
    // Expect closing brace
    state
        .tokens
        .expect(&Token::RightBrace)
        .map_err(|_| anyhow::anyhow!("Expected '}}' after match arms"))?;
    Ok(Expr::new(ExprKind::Match { expr, arms }, start_span))
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
    let start_span = state.tokens.peek().map(|(_, s)| *s).unwrap_or_default();
    // Parse pattern
    let pattern = parse_match_pattern(state)?;
    // Parse optional guard (if condition)
    let guard = if matches!(state.tokens.peek(), Some((Token::If, _))) {
        state.tokens.advance(); // consume 'if'
        Some(Box::new(parse_expr_recursive(state)?))
    } else {
        None
    };
    // Expect => token
    state
        .tokens
        .expect(&Token::FatArrow)
        .map_err(|_| anyhow::anyhow!("Expected '=>' in match arm"))?;
    // Parse result expression
    let body = Box::new(parse_expr_recursive(state)?);
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
pub(in crate::frontend::parser) fn parse_match_pattern(state: &mut ParserState) -> Result<Pattern> {
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
pub(in crate::frontend::parser) fn parse_single_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _span)) = state.tokens.peek() else {
        bail!("Expected pattern");
    };
    match token {
        Token::Underscore => parse_wildcard_pattern(state),
        Token::Integer(_)
        | Token::Float(_)
        | Token::String(_)
        | Token::RawString(_)
        | Token::Char(_)
        | Token::Bool(_) => parse_literal_pattern(state),
        Token::Some | Token::None => parse_option_pattern(state),
        Token::Ok | Token::Err => parse_result_pattern(state),
        Token::Identifier(_) => parse_identifier_or_constructor_pattern(state),
        Token::LeftParen => parse_match_tuple_pattern(state),
        Token::LeftBracket => parse_match_list_pattern(state),
        _ => bail!("Unexpected token in pattern: {token:?}"),
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
        Token::RawString(s) => parse_simple_literal_pattern(state, Literal::String(s))?,
        Token::Char(c) => parse_char_literal_pattern(state, c)?,
        Token::Byte(b) => parse_simple_literal_pattern(state, Literal::Byte(b))?,
        Token::Bool(b) => parse_simple_literal_pattern(state, Literal::Bool(b))?,
        _ => bail!("Expected literal pattern, got: {token:?}"),
    };
    Ok(pattern)
}

/// Extract method: Parse integer literal with optional range pattern - complexity: 8
fn parse_integer_literal_pattern(state: &mut ParserState, val: String) -> Result<Pattern> {
    state.tokens.advance();
    // Parse the integer value from string (ignore type suffix for pattern matching)
    let (num_part, type_suffix) = if let Some(pos) = val.find(|c: char| c.is_alphabetic()) {
        (&val[..pos], Some(val[pos..].to_string()))
    } else {
        (val.as_str(), None)
    };
    let parsed_val = num_part.parse::<i64>().map_err(|_| {
        anyhow::anyhow!("Invalid integer literal: {num_part}")
    })?;

    // Check for range patterns: 1..5 or 1..=5
    match state.tokens.peek() {
        Some((Token::DotDot, _)) => parse_integer_range_pattern(state, parsed_val, false),
        Some((Token::DotDotEqual, _)) => parse_integer_range_pattern(state, parsed_val, true),
        _ => Ok(Pattern::Literal(Literal::Integer(parsed_val, type_suffix))),
    }
}

/// Extract method: Parse integer range pattern - complexity: 6
fn parse_integer_range_pattern(
    state: &mut ParserState,
    start_val: i64,
    inclusive: bool,
) -> Result<Pattern> {
    state.tokens.advance(); // consume '..' or '..='
    if let Some((Token::Integer(end_val_str), _)) = state.tokens.peek() {
        let end_val_str = end_val_str.clone();
        state.tokens.advance();
        // Parse the end value
        let (num_part, _type_suffix) =
            if let Some(pos) = end_val_str.find(|c: char| c.is_alphabetic()) {
                (&end_val_str[..pos], Some(end_val_str[pos..].to_string()))
            } else {
                (end_val_str.as_str(), None)
            };
        let end_val = num_part.parse::<i64>().map_err(|_| {
            anyhow::anyhow!("Invalid integer literal: {num_part}")
        })?;
        Ok(Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(start_val, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(end_val, None))),
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
fn parse_char_range_pattern(
    state: &mut ParserState,
    start_val: char,
    inclusive: bool,
) -> Result<Pattern> {
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
        _ => bail!("Expected Some or None pattern"),
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
        _ => bail!("Expected Ok or Err pattern"),
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

    // Check for enum variant paths: Color::Red, Option::Some, etc.
    if matches!(state.tokens.peek(), Some((Token::ColonColon, _))) {
        let full_path = super::identifiers::parse_module_path_segments(state, name)?;
        // Check if followed by struct fields or tuple args
        return if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
            parse_struct_pattern_with_name(state, full_path)
        } else if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
            parse_constructor_pattern(state, full_path)
        } else {
            // Unit enum variant like Color::Red - use QualifiedName
            let path_segments: Vec<String> =
                full_path.split("::").map(ToString::to_string).collect();
            Ok(Pattern::QualifiedName(path_segments))
        };
    }

    // Check for struct patterns: Point { x, y }
    if matches!(state.tokens.peek(), Some((Token::LeftBrace, _))) {
        parse_struct_pattern_with_name(state, name)
    }
    // Check for enum-like patterns: Ok(x), Err(e), etc.
    else if matches!(state.tokens.peek(), Some((Token::LeftParen, _))) {
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
/// Parse rest pattern in list with .. (two dots): ..tail
/// Complexity: 3 (Toyota Way: <10 ✓)
fn parse_list_rest_pattern(state: &mut ParserState) -> Result<Pattern> {
    state.tokens.advance(); // consume ..
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Ok(Pattern::RestNamed(name))
    } else {
        bail!("Expected identifier after .. in list pattern")
    }
}

/// Parse single list pattern element (regular or rest)
/// Complexity: 4 (Toyota Way: <10 ✓)
fn parse_list_pattern_element(state: &mut ParserState) -> Result<Pattern> {
    if matches!(state.tokens.peek(), Some((Token::DotDot, _))) {
        parse_list_rest_pattern(state)
    } else {
        parse_match_pattern(state)
    }
}

/// Parse list pattern: [a, b, ..rest, c]
/// Complexity: 4 (Toyota Way: <10 ✓) [Reduced from 11]
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
        patterns.push(parse_list_pattern_element(state)?);

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
            Ok(Pattern::Ok(Box::new(patterns.into_iter().next().expect(
                "patterns.len() == 1, so next() must return Some",
            ))))
        }
        ("Err", 1) => {
            // Err(pattern) - Result error case
            Ok(Pattern::Err(Box::new(patterns.into_iter().next().expect(
                "patterns.len() == 1, so next() must return Some",
            ))))
        }
        ("Some", 1) => {
            // Some(pattern) - Option success case
            Ok(Pattern::Some(Box::new(patterns.into_iter().next().expect(
                "patterns.len() == 1, so next() must return Some",
            ))))
        }
        ("None", 0) => {
            // None - Option empty case
            Ok(Pattern::None)
        }
        (name, 0) => {
            // Empty constructor - check if qualified path or simple identifier
            if name.contains("::") {
                let path: Vec<String> = name.split("::").map(ToString::to_string).collect();
                Ok(Pattern::QualifiedName(path))
            } else {
                Ok(Pattern::Identifier(name.to_string()))
            }
        }
        (name, _) => {
            // Constructor with arguments - use TupleVariant for custom enums
            let path: Vec<String> = name.split("::").map(ToString::to_string).collect();
            Ok(Pattern::TupleVariant { path, patterns })
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
        Ok(patterns
            .into_iter()
            .next()
            .expect("patterns.len() == 1, so next() must return Some"))
    } else {
        Ok(Pattern::Or(patterns))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_identifier_pattern() {
        let code = "let x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Identifier pattern should parse");
    }

    #[test]
    fn test_tuple_pattern() {
        let code = "let (x, y, z) = (1, 2, 3)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple pattern should parse");
    }

    #[test]
    fn test_list_pattern_with_rest() {
        let code = "let [first, ...rest] = [1, 2, 3, 4]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "List pattern with rest should parse");
    }

    #[test]
    fn test_struct_pattern() {
        let code = "let Point { x, y } = point";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Struct pattern should parse");
    }

    #[test]
    fn test_some_pattern() {
        let code = "let Some(x) = maybe_value";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Some pattern should parse");
    }

    #[test]
    fn test_ok_pattern() {
        let code = "let Ok(val) = result";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Ok pattern should parse");
    }

    #[test]
    fn test_err_pattern() {
        let code = "let Err(e) = result";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Err pattern should parse");
    }

    #[test]
    fn test_none_pattern() {
        let code = "let None = maybe_value";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "None pattern should parse");
    }

    #[test]
    fn test_wildcard_pattern() {
        let code = "let _ = value";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Wildcard pattern should parse");
    }

    #[test]
    fn test_literal_pattern() {
        let code = "match x { 42 => true, _ => false }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Literal pattern in match should parse");
    }

    #[test]
    fn test_range_pattern() {
        let code = "match x { 1..10 => \"low\", _ => \"high\" }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Range pattern should parse");
    }

    #[test]
    fn test_or_pattern() {
        let code = "match x { Some(1) | Some(2) => true, _ => false }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Or pattern should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_identifier_patterns_parse(name in "[a-z][a-z0-9_]*") {
                let code = format!("let {} = 42", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_tuple_patterns_parse(a in "[a-z]+", b in "[a-z]+") {
                let code = format!("let ({}, {}) = (1, 2)", a, b);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_list_patterns_parse(name in "[a-z]+") {
                let code = format!("let [{}, ...rest] = [1, 2, 3]", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_some_patterns_parse(inner in "[a-z]+") {
                let code = format!("let Some({}) = value", inner);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_wildcard_always_parses(_seed in any::<u32>()) {
                let code = "let _ = 42";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_literal_patterns_parse(n in 0i32..1000) {
                let code = format!("match x {{ {} => true, _ => false }}", n);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_struct_patterns_parse(field in "[a-z]+") {
                let code = format!("let Point {{ {} }} = p", field);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
