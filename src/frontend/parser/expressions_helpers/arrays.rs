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

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::{Expr, ExprKind};
    use crate::frontend::parser::Result;

    fn parse(code: &str) -> Result<Expr> {
        Parser::new(code).parse()
    }

    fn get_block_exprs(expr: &Expr) -> Option<&Vec<Expr>> {
        match &expr.kind {
            ExprKind::Block(exprs) => Some(exprs),
            _ => None,
        }
    }

    // ============================================================
    // List produces List ExprKind
    // ============================================================

    #[test]
    fn test_list_produces_list_exprkind() {
        let expr = parse("[1, 2, 3]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::List(_)),
                "Should produce List ExprKind"
            );
        }
    }

    #[test]
    fn test_empty_list_produces_list_exprkind() {
        let expr = parse("[]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::List(_)),
                "Empty list should produce List"
            );
        }
    }

    #[test]
    fn test_array_init_produces_array_init_exprkind() {
        let expr = parse("[0; 5]").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::ArrayInit { .. }),
                "Should produce ArrayInit ExprKind"
            );
        }
    }

    // ============================================================
    // Integer lists
    // ============================================================

    #[test]
    fn test_list_single_int() {
        let result = parse("[42]");
        assert!(result.is_ok(), "Single int list should parse");
    }

    #[test]
    fn test_list_two_ints() {
        let result = parse("[1, 2]");
        assert!(result.is_ok(), "Two int list should parse");
    }

    #[test]
    fn test_list_five_ints() {
        let result = parse("[1, 2, 3, 4, 5]");
        assert!(result.is_ok(), "Five int list should parse");
    }

    #[test]
    fn test_list_negative_ints() {
        let result = parse("[-1, -2, -3]");
        assert!(result.is_ok(), "Negative int list should parse");
    }

    #[test]
    fn test_list_large_ints() {
        let result = parse("[1000000, 2000000, 3000000]");
        assert!(result.is_ok(), "Large int list should parse");
    }

    // ============================================================
    // Float lists
    // ============================================================

    #[test]
    fn test_list_floats() {
        let result = parse("[1.0, 2.5, 3.14]");
        assert!(result.is_ok(), "Float list should parse");
    }

    #[test]
    fn test_list_negative_floats() {
        let result = parse("[-1.0, -2.5]");
        assert!(result.is_ok(), "Negative float list should parse");
    }

    // ============================================================
    // String lists
    // ============================================================

    #[test]
    fn test_list_strings() {
        let result = parse("[\"a\", \"b\", \"c\"]");
        assert!(result.is_ok(), "String list should parse");
    }

    #[test]
    fn test_list_empty_strings() {
        let result = parse("[\"\", \"\"]");
        assert!(result.is_ok(), "Empty string list should parse");
    }

    #[test]
    fn test_list_multiword_strings() {
        let result = parse("[\"hello world\", \"foo bar\"]");
        assert!(result.is_ok(), "Multiword string list should parse");
    }

    // ============================================================
    // Boolean lists
    // ============================================================

    #[test]
    fn test_list_booleans() {
        let result = parse("[true, false, true]");
        assert!(result.is_ok(), "Boolean list should parse");
    }

    #[test]
    fn test_list_all_true() {
        let result = parse("[true, true, true]");
        assert!(result.is_ok(), "All true list should parse");
    }

    #[test]
    fn test_list_all_false() {
        let result = parse("[false, false]");
        assert!(result.is_ok(), "All false list should parse");
    }

    // ============================================================
    // Mixed type lists
    // ============================================================

    #[test]
    fn test_list_mixed_int_float() {
        let result = parse("[1, 2.5, 3]");
        assert!(result.is_ok(), "Mixed int/float list should parse");
    }

    #[test]
    fn test_list_mixed_various() {
        let result = parse("[1, \"hello\", true]");
        assert!(result.is_ok(), "Mixed various types should parse");
    }

    // ============================================================
    // Nested lists
    // ============================================================

    #[test]
    fn test_nested_empty_lists() {
        let result = parse("[[], []]");
        assert!(result.is_ok(), "Nested empty lists should parse");
    }

    #[test]
    fn test_nested_two_level() {
        let result = parse("[[1, 2], [3, 4]]");
        assert!(result.is_ok(), "Two level nested should parse");
    }

    #[test]
    fn test_nested_three_level() {
        let result = parse("[[[1]]]");
        assert!(result.is_ok(), "Three level nested should parse");
    }

    #[test]
    fn test_nested_mixed_depths() {
        let result = parse("[1, [2, 3], [[4]]]");
        assert!(result.is_ok(), "Mixed depth nested should parse");
    }

    // ============================================================
    // Array initialization
    // ============================================================

    #[test]
    fn test_array_init_zero() {
        let result = parse("[0; 10]");
        assert!(result.is_ok(), "Zero array init should parse");
    }

    #[test]
    fn test_array_init_one() {
        let result = parse("[1; 5]");
        assert!(result.is_ok(), "One array init should parse");
    }

    #[test]
    fn test_array_init_string() {
        let result = parse("[\"x\"; 3]");
        assert!(result.is_ok(), "String array init should parse");
    }

    #[test]
    fn test_array_init_boolean() {
        let result = parse("[false; 4]");
        assert!(result.is_ok(), "Boolean array init should parse");
    }

    #[test]
    fn test_array_init_empty_list() {
        let result = parse("[[]; 3]");
        assert!(result.is_ok(), "Empty list array init should parse");
    }

    #[test]
    fn test_array_init_expression_size() {
        let result = parse("[0; n]");
        assert!(result.is_ok(), "Expression size array init should parse");
    }

    #[test]
    fn test_array_init_expression_value() {
        let result = parse("[default_value; 10]");
        assert!(result.is_ok(), "Expression value array init should parse");
    }

    // ============================================================
    // Spread expressions
    // ============================================================

    #[test]
    fn test_spread_only() {
        let result = parse("[...items]");
        assert!(result.is_ok(), "Spread only should parse");
    }

    #[test]
    fn test_spread_at_start() {
        let result = parse("[...first, 1, 2]");
        assert!(result.is_ok(), "Spread at start should parse");
    }

    #[test]
    fn test_spread_at_end() {
        let result = parse("[1, 2, ...rest]");
        assert!(result.is_ok(), "Spread at end should parse");
    }

    #[test]
    fn test_spread_in_middle() {
        let result = parse("[1, ...middle, 2]");
        assert!(result.is_ok(), "Spread in middle should parse");
    }

    #[test]
    fn test_multiple_spreads() {
        let result = parse("[...a, ...b]");
        assert!(result.is_ok(), "Multiple spreads should parse");
    }

    #[test]
    fn test_spread_function_call() {
        let result = parse("[...get_items()]");
        assert!(result.is_ok(), "Spread function call should parse");
    }

    // ============================================================
    // Trailing commas
    // ============================================================

    #[test]
    fn test_trailing_comma_single() {
        let result = parse("[1,]");
        assert!(result.is_ok(), "Single element trailing comma should parse");
    }

    #[test]
    fn test_trailing_comma_multiple() {
        let result = parse("[1, 2, 3,]");
        assert!(result.is_ok(), "Multiple element trailing comma should parse");
    }

    #[test]
    fn test_trailing_comma_nested() {
        let result = parse("[[1, 2,], [3, 4,],]");
        assert!(result.is_ok(), "Nested trailing commas should parse");
    }

    // ============================================================
    // List comprehensions
    // ============================================================

    #[test]
    fn test_list_comprehension_simple() {
        let result = parse("[x for x in items]");
        assert!(result.is_ok(), "Simple list comprehension should parse");
    }

    #[test]
    fn test_list_comprehension_expression() {
        let result = parse("[x * 2 for x in items]");
        assert!(result.is_ok(), "Expression list comprehension should parse");
    }

    #[test]
    fn test_list_comprehension_with_filter() {
        let result = parse("[x for x in items if x > 0]");
        assert!(result.is_ok(), "Filtered list comprehension should parse");
    }

    #[test]
    fn test_list_comprehension_range() {
        let result = parse("[x for x in 0..10]");
        assert!(result.is_ok(), "Range list comprehension should parse");
    }

    #[test]
    fn test_list_comprehension_method_call() {
        let result = parse("[s.len() for s in strings]");
        assert!(result.is_ok(), "Method call list comprehension should parse");
    }

    // ============================================================
    // Expression elements
    // ============================================================

    #[test]
    fn test_list_with_arithmetic() {
        let result = parse("[1 + 2, 3 * 4, 5 - 6]");
        assert!(result.is_ok(), "Arithmetic expression list should parse");
    }

    #[test]
    fn test_list_with_function_calls() {
        let result = parse("[foo(), bar(), baz()]");
        assert!(result.is_ok(), "Function call list should parse");
    }

    #[test]
    fn test_list_with_method_calls() {
        let result = parse("[a.len(), b.size(), c.count()]");
        assert!(result.is_ok(), "Method call list should parse");
    }

    #[test]
    fn test_list_with_conditionals() {
        let result = parse("[if a { 1 } else { 0 }, if b { 2 } else { 0 }]");
        assert!(result.is_ok(), "Conditional expression list should parse");
    }

    // Property tests for arrays
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_empty_list_always_parses(_seed in any::<u32>()) {
                let code = "[]";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "Empty list should always parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_single_element_lists_parse(n in any::<i32>()) {
                let code = format!("[{n}]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Single element list [{}] should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multi_element_lists_parse(
                a in any::<i32>(),
                b in any::<i32>(),
                c in any::<i32>()
            ) {
                let code = format!("[{a}, {b}, {c}]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Multi-element list [{}, {}, {}] should parse", a, b, c);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_array_init_parses(value in any::<i32>(), size in 1..100usize) {
                let code = format!("[{value}; {size}]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Array init [{}; {}] should parse", value, size);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_trailing_commas_parse(n in any::<i32>()) {
                let code = format!("[{n},]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Trailing comma [{}] should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_nested_lists_parse(
                inner1 in prop::collection::vec(any::<i32>(), 0..5),
                inner2 in prop::collection::vec(any::<i32>(), 0..5)
            ) {
                let inner1_str = inner1.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ");
                let inner2_str = inner2.iter().map(std::string::ToString::to_string).collect::<Vec<_>>().join(", ");
                let code = format!("[[{inner1_str}], [{inner2_str}]]");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Nested list [[...], [...]] should parse");
            }
        }
    }
}
