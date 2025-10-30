//! Tuple parsing
//!
//! Handles parsing of:
//! - Tuple expressions: `(1, 2, 3)`
//! - Unit type: `()`
//! - Grouped expressions: `(expr)`
//! - Trailing commas: `(1, 2,)`
//! - Tuple-to-lambda conversion: `(x, y) => x + y`
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Literal, Span};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import functions from parent parser module
use crate::frontend::parser::parse_expr_recursive;

/// Parse parenthesized expression, tuple, or lambda
///
/// Determines type based on syntax:
/// - `()` - unit type
/// - `(expr)` - grouped expression
/// - `(expr,)` or `(expr, expr)` - tuple
/// - `(x) => ...` or `(x, y) => ...` - lambda
///
/// # Examples
/// ```ruchy
/// ()              // Unit type
/// (42)            // Grouped expression
/// (1, 2)          // 2-tuple
/// (1, 2, 3,)      // 3-tuple with trailing comma
/// (x, y) => x + y // Lambda from tuple
/// ```
pub(in crate::frontend::parser) fn parse_parentheses_token(
    state: &mut ParserState,
    span: Span,
) -> Result<Expr> {
    state.tokens.advance();

    // Check for unit type ()
    if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
        state.tokens.advance();
        return Ok(Expr::new(ExprKind::Literal(Literal::Unit), span));
    }

    // Parse first expression
    let first_expr = parse_expr_recursive(state)?;

    // Check if we have a comma (tuple) or just closing paren (grouped expr)
    if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        // This is a tuple, parse remaining elements
        let elements = parse_tuple_elements(state, first_expr)?;
        state.tokens.expect(&Token::RightParen)?;
        let tuple_expr = Expr::new(ExprKind::Tuple(elements), span);
        maybe_parse_lambda(state, tuple_expr, span)
    } else {
        // Just a grouped expression
        state.tokens.expect(&Token::RightParen)?;
        maybe_parse_lambda(state, first_expr, span)
    }
}

/// Parse tuple elements after first element
///
/// Supports trailing commas.
///
/// # Examples
/// ```ruchy
/// 1, 2       // (1, 2) - two elements
/// 1, 2, 3    // (1, 2, 3) - three elements
/// 1, 2,      // (1, 2) - trailing comma allowed
/// ```
fn parse_tuple_elements(state: &mut ParserState, first_expr: Expr) -> Result<Vec<Expr>> {
    let mut elements = vec![first_expr];
    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume comma

        // Check for trailing comma before closing paren
        if matches!(state.tokens.peek(), Some((Token::RightParen, _))) {
            break;
        }

        elements.push(parse_expr_recursive(state)?);
    }
    Ok(elements)
}

/// Check if expression should be converted to lambda
///
/// Converts `(x) => expr` or `(x, y) => expr` into lambda expressions.
///
/// # Examples
/// ```ruchy
/// (x) => x + 1        // Single parameter lambda
/// (x, y) => x + y     // Multiple parameter lambda
/// ```
fn maybe_parse_lambda(state: &mut ParserState, expr: Expr, span: Span) -> Result<Expr> {
    if matches!(state.tokens.peek(), Some((Token::FatArrow, _))) {
        super::super::parse_lambda_from_expr(state, expr, span)
    } else {
        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_unit_type() {
        let code = "()";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Unit type should parse");
    }

    #[test]
    fn test_grouped_expression() {
        let code = "(42)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Grouped expression should parse");
    }

    #[test]
    fn test_simple_tuple() {
        let code = "(1, 2)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple tuple should parse");
    }

    #[test]
    fn test_triple_tuple() {
        let code = "(1, 2, 3)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Triple tuple should parse");
    }

    #[test]
    fn test_trailing_comma() {
        let code = "(1, 2,)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple with trailing comma should parse");
    }

    #[test]
    fn test_nested_tuple() {
        let code = "((1, 2), (3, 4))";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested tuple should parse");
    }

    #[test]
    fn test_tuple_lambda() {
        let code = "(x, y) => x + y";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Tuple lambda should parse");
    }

    // Property tests for tuples
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_unit_type_always_parses(_seed in any::<u32>()) {
                let code = "()";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok(), "Unit type should always parse");
            }

            #[test]
            #[ignore]
            fn prop_grouped_expressions_parse(n in any::<i32>()) {
                let code = format!("({n})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Grouped ({}) should parse", n);
            }

            #[test]
            #[ignore]
            fn prop_two_tuples_parse(a in any::<i32>(), b in any::<i32>()) {
                let code = format!("({a}, {b})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Tuple ({}, {}) should parse", a, b);
            }

            #[test]
            #[ignore]
            fn prop_three_tuples_parse(
                a in any::<i32>(),
                b in any::<i32>(),
                c in any::<i32>()
            ) {
                let code = format!("({a}, {b}, {c})");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Tuple ({}, {}, {}) should parse", a, b, c);
            }

            #[test]
            #[ignore]
            fn prop_trailing_commas_parse(a in any::<i32>(), b in any::<i32>()) {
                let code = format!("({a}, {b},)");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Trailing comma ({}, {}) should parse", a, b);
            }

            #[test]
            #[ignore]
            fn prop_nested_tuples_parse(
                a in any::<i32>(),
                b in any::<i32>(),
                c in any::<i32>(),
                d in any::<i32>()
            ) {
                let code = format!("(({a}, {b}), ({c}, {d}))");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Nested tuple should parse");
            }
        }
    }
}
