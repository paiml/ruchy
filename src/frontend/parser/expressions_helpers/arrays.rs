//! Array and list literal parsing
//!
//! Handles parsing of:
//! - Array literals: `[1, 2, 3]`
//! - Empty arrays: `[]`
//! - Array initialization: `[value; size]` (e.g., `[0; 10]`)
//! - Spread expressions: `[...items]`
//! - Trailing commas: `[1, 2, 3,]`
//! - List comprehensions: Delegated to collections module
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import functions from parent parser module
use crate::frontend::parser::collections;
use crate::frontend::parser::parse_expr_recursive;

/// Parse list literal: `[...]`
///
/// Determines list type based on syntax:
/// - `[]` - empty list
/// - `[value; size]` - array initialization
/// - `[expr for item in iter]` - list comprehension (delegated)
/// - `[expr, expr, ...]` - regular list
///
/// # Examples
/// ```ruchy
/// []              // Empty list
/// [1, 2, 3]       // Regular list
/// [0; 10]         // Array init (10 zeros)
/// [...items]      // Spread expression
/// [1, 2, 3,]      // Trailing comma allowed
/// ```
pub(in crate::frontend::parser) fn parse_list_literal(state: &mut ParserState) -> Result<Expr> {
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

/// Parse array element (might be spread expression)
///
/// # Examples
/// ```ruchy
/// expr        // Regular expression
/// ...expr     // Spread expression
/// ```
pub(in crate::frontend::parser) fn parse_array_element(state: &mut ParserState) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::DotDotDot, _))) {
        let start_span = state.tokens.expect(&Token::DotDotDot)?; // consume ...
        let expr = parse_expr_recursive(state)?;
        Ok(Expr::new(
            ExprKind::Spread {
                expr: Box::new(expr),
            },
            start_span,
        ))
    } else {
        parse_expr_recursive(state)
    }
}

/// Parse array initialization: `[value; size]`
///
/// Creates an array with `size` copies of `value`.
///
/// # Examples
/// ```ruchy
/// [0; 10]         // [0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
/// [false; 5]      // [false, false, false, false, false]
/// [[]; 3]         // [[], [], []]
/// ```
fn parse_array_init(state: &mut ParserState, value_expr: Expr, start_span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume ;
    let size_expr = parse_expr_recursive(state)?;
    state
        .tokens
        .expect(&Token::RightBracket)
        .map_err(|_| anyhow::anyhow!("Expected ']' after array initialization"))?;
    Ok(Expr::new(
        ExprKind::ArrayInit {
            value: Box::new(value_expr),
            size: Box::new(size_expr),
        },
        start_span,
    ))
}

/// Parse regular list: `[expr, expr, ...]`
///
/// Supports trailing commas and spread expressions.
///
/// # Examples
/// ```ruchy
/// [1, 2, 3]       // Regular list
/// [1, 2, 3,]      // Trailing comma
/// [1, ...rest, 4] // With spread
/// ```
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

    state
        .tokens
        .expect(&Token::RightBracket)
        .map_err(|_| anyhow::anyhow!("Expected ']' to close list literal"))?;
    Ok(Expr::new(ExprKind::List(elements), start_span))
}

/// Parse list comprehension body
///
/// Delegates to collections module for proper nested comprehension handling.
///
/// # Examples
/// ```ruchy
/// [x for x in items]
/// [x * 2 for x in range(10)]
/// [x for x in items if x > 0]
/// ```
fn parse_list_comprehension_body(
    state: &mut ParserState,
    expr: Expr,
    start_span: Span,
) -> Result<Expr> {
    // Delegate to the collections module which handles nested comprehensions properly
    collections::parse_list_comprehension(state, start_span, expr)
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_empty_list() {
        let code = "[]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Empty list should parse");
    }

    #[test]
    fn test_simple_list() {
        let code = "[1, 2, 3]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple list should parse");
    }

    #[test]
    fn test_list_trailing_comma() {
        let code = "[1, 2, 3,]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "List with trailing comma should parse");
    }

    #[test]
    fn test_array_init() {
        let code = "[0; 10]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Array initialization should parse");
    }

    #[test]
    fn test_spread_expression() {
        let code = "[...items]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Spread expression should parse");
    }

    #[test]
    fn test_nested_list() {
        let code = "[[1, 2], [3, 4]]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested list should parse");
    }

    #[test]
    fn test_mixed_spread() {
        let code = "[1, ...rest, 2]";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Mixed spread expression should parse");
    }

    // Property tests for arrays
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_empty_list_always_parses(_seed in any::<u32>()) {
                let code = "[]";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "Empty list should always parse");
            }

            #[test]
            #[ignore]
            fn prop_single_element_lists_parse(n in any::<i32>()) {
                let code = format!("[{}]", n);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Single element list [{}] should parse", n);
            }

            #[test]
            #[ignore]
            fn prop_multi_element_lists_parse(
                a in any::<i32>(),
                b in any::<i32>(),
                c in any::<i32>()
            ) {
                let code = format!("[{}, {}, {}]", a, b, c);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Multi-element list [{}, {}, {}] should parse", a, b, c);
            }

            #[test]
            #[ignore]
            fn prop_array_init_parses(value in any::<i32>(), size in 1..100usize) {
                let code = format!("[{}; {}]", value, size);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Array init [{}; {}] should parse", value, size);
            }

            #[test]
            #[ignore]
            fn prop_trailing_commas_parse(n in any::<i32>()) {
                let code = format!("[{},]", n);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Trailing comma [{}] should parse", n);
            }

            #[test]
            #[ignore]
            fn prop_nested_lists_parse(
                inner1 in prop::collection::vec(any::<i32>(), 0..5),
                inner2 in prop::collection::vec(any::<i32>(), 0..5)
            ) {
                let inner1_str = inner1.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
                let inner2_str = inner2.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
                let code = format!("[[{}], [{}]]", inner1_str, inner2_str);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Nested list [[...], [...]] should parse");
            }
        }
    }
}
