//! Variable declaration parsing (let and var statements)
//!
//! Handles parsing of variable declarations with support for:
//! - Let bindings: `let x = value` or `let x = value in body`
//! - Mutable bindings: `let mut x = value`
//! - Var statements: `var x = value` (implicitly mutable)
//! - Type annotations: `let x: i32 = 42`
//! - Pattern matching: `let (x, y) = tuple`
//! - Let-else patterns: `let Some(x) = opt else { return }`
//!
//! # Examples
//! ```ruchy
//! // Simple let binding
//! let x = 42
//!
//! // Mutable binding
//! let mut count = 0
//!
//! // Type annotation
//! let name: String = "Alice"
//!
//! // Tuple destructuring
//! let (x, y) = (1, 2)
//!
//! // Let-else pattern
//! let Some(value) = optional else {
//!     return Err("Missing value")
//! }
//!
//! // Let-in expression
//! let x = 10 in x * 2
//!
//! // Var statement (mutable)
//! var counter = 0
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Literal, Pattern, Span, Type};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

// Import pattern parsing from patterns module
use super::patterns::{
    parse_list_pattern, parse_single_pattern, parse_struct_pattern, parse_struct_pattern_with_name,
    parse_tuple_pattern,
};

/// Parse let statement
///
/// Supports let bindings, let-in expressions, and let-else patterns.
pub(in crate::frontend::parser) fn parse_let_statement(state: &mut ParserState) -> Result<Expr> {
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
    let value = Box::new(parse_expr_recursive(state)?);

    // Check for 'else' clause (let-else pattern)
    let else_block = parse_let_else_clause(state)?;

    // Parse optional 'in' clause for let expressions (not compatible with let-else)
    let body = if else_block.is_none() {
        parse_let_in_clause(state, value.span)?
    } else {
        // For let-else, body is unit (the else block handles divergence)
        Box::new(Expr::new(ExprKind::Literal(Literal::Unit), value.span))
    };

    // Create the appropriate expression based on pattern type
    create_let_expression(
        pattern,
        type_annotation,
        value,
        body,
        is_mutable,
        else_block,
        start_span,
    )
}

/// Parse var statement (implicitly mutable)
///
/// Syntax: `var name [: type] = value`
pub(in crate::frontend::parser) fn parse_var_statement(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Var)?;
    // var is always mutable

    let pattern = parse_var_pattern(state)?;
    let type_annotation = parse_optional_type_annotation(state)?;

    state.tokens.expect(&Token::Equal)?;
    let value = Box::new(parse_expr_recursive(state)?);

    create_var_expression(pattern, type_annotation, value, start_span)
}

/// Parse mutability keyword for let statement
fn parse_let_mutability(state: &mut ParserState) -> bool {
    if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance();
        true
    } else {
        false
    }
}

/// Parse pattern for let statement
///
/// Supports identifiers, destructuring, and variant patterns.
fn parse_let_pattern(state: &mut ParserState, is_mutable: bool) -> Result<Pattern> {
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
        Some((Token::Identifier(_) | Token::Result | Token::Var, _)) => {
            // Handle identifier or reserved keywords that can be used as identifiers
            let name = match state.tokens.peek() {
                Some((Token::Identifier(n), _)) => n.clone(),
                Some((Token::Result, _)) => "Result".to_string(),
                Some((Token::Var, _)) => "var".to_string(),
                _ => bail!("Expected identifier in let pattern"),
            };
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

/// Parse variant pattern with name
///
/// Examples: Some(x), Ok(val), Err(e), Color(r, g, b)
fn parse_variant_pattern_with_name(
    state: &mut ParserState,
    variant_name: String,
) -> Result<Pattern> {
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

/// Create pattern for variant
///
/// Special cases for Some/Ok/Err, otherwise `TupleVariant`.
fn create_pattern_for_variant(variant_name: String, patterns: Vec<Pattern>) -> Result<Pattern> {
    // Special case for common Option/Result variants (single element)
    if patterns.len() == 1 {
        match variant_name.as_str() {
            "Some" => {
                return Ok(Pattern::Some(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            "Ok" => {
                return Ok(Pattern::Ok(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            "Err" => {
                return Ok(Pattern::Err(Box::new(
                    patterns
                        .into_iter()
                        .next()
                        .expect("patterns.len() == 1 so next() must return Some"),
                )))
            }
            _ => {}
        }
    }

    // For other variants or multiple elements, use TupleVariant
    Ok(Pattern::TupleVariant {
        path: vec![variant_name],
        patterns,
    })
}

/// Parse optional type annotation
fn parse_let_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume ':'
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Parse optional 'else' clause for let-else patterns
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

/// Create let expression based on pattern type
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

/// Parse variable pattern for var statement
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

/// Parse optional type annotation
fn parse_optional_type_annotation(state: &mut ParserState) -> Result<Option<Type>> {
    if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance();
        Ok(Some(utils::parse_type(state)?))
    } else {
        Ok(None)
    }
}

/// Create variable expression (var is always mutable)
fn create_var_expression(
    pattern: Pattern,
    type_annotation: Option<Type>,
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
                else_block: None,
            },
            start_span.merge(end_span),
        )),
        Pattern::Tuple(_) | Pattern::List(_) | Pattern::Wildcard => Ok(Expr::new(
            ExprKind::LetPattern {
                pattern,
                type_annotation,
                value,
                body,
                is_mutable,
                else_block: None,
            },
            start_span.merge(end_span),
        )),
        _ => bail!("var only supports simple patterns (identifier, tuple, list, wildcard)"),
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_let_simple() {
        let code = "let x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple let binding should parse");
    }

    #[test]
    fn test_let_mut() {
        let code = "let mut x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Mutable let binding should parse");
    }

    #[test]
    fn test_let_with_type() {
        let code = "let x: i32 = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let with type annotation should parse");
    }

    #[test]
    fn test_let_tuple_destructuring() {
        let code = "let (x, y) = (1, 2)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple destructuring should parse");
    }

    #[test]
    fn test_let_in_expression() {
        let code = "let x = 10 in x * 2";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let-in expression should parse");
    }

    #[test]
    fn test_let_else_pattern() {
        let code = "let Some(x) = opt else { return }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Let-else pattern should parse");
    }

    #[test]
    fn test_var_statement() {
        let code = "var x = 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Var statement should parse");
    }

    #[test]
    fn test_var_with_type() {
        let code = "var count: i32 = 0";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Var with type should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_let_with_identifiers(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_mut_parses(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let mut {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_var_parses(name in "[a-z]+", value in 0i32..100) {
                let code = format!("var {name} = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_with_tuple(n1 in 0i32..100, n2 in 0i32..100) {
                let code = format!("let (x, y) = ({n1}, {n2})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_in_expr(name in "[a-z]+", val in 0i32..100, expr in 0i32..100) {
                let code = format!("let {name} = {val} in {expr}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_let_type_annotation(name in "[a-z]+", value in 0i32..100) {
                let code = format!("let {name}: i32 = {value}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_var_tuple_destructuring(n1 in 0i32..100, n2 in 0i32..100) {
                let code = format!("var (a, b) = ({n1}, {n2})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
