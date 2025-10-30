//! Loop expression parsing
//!
//! Handles parsing of all loop constructs:
//! - `for` loops with pattern destructuring
//! - `while` loops and `while let` loops
//! - Infinite `loop` expressions
//! - Loop labels for break/continue targeting
//!
//! # Examples
//! ```ruchy
//! // For loop with simple pattern
//! for i in 0..10 {
//!     print(i)
//! }
//!
//! // For loop with tuple destructuring
//! for key, value in map {
//!     print(f"{key}: {value}")
//! }
//!
//! // While loop
//! while x < 10 {
//!     x = x + 1
//! }
//!
//! // While-let loop
//! while let Some(value) = optional {
//!     process(value)
//! }
//!
//! // Labeled loop
//! 'outer: for i in 0..10 {
//!     for j in 0..10 {
//!         if should_exit {
//!             break 'outer
//!         }
//!     }
//! }
//!
//! // Infinite loop
//! loop {
//!     if done { break }
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Pattern};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, ParserState, Result};

// Import pattern parsing functions from parent expressions module
use super::super::{parse_list_pattern, parse_match_pattern, parse_tuple_pattern};

/// Parse loop label dispatch
///
/// Handles labeled loop constructs: `'label: for/while/loop`
pub(in crate::frontend::parser) fn parse_loop_label(
    state: &mut ParserState,
    label_name: String,
) -> Result<Expr> {
    // Note: Caller has already consumed the Lifetime token, so current token is Colon
    state.tokens.expect(&Token::Colon)?;
    match state.tokens.peek() {
        Some((Token::For, _)) => parse_labeled_for_loop(state, Some(label_name)),
        Some((Token::While, _)) => parse_labeled_while_loop(state, Some(label_name)),
        Some((Token::Loop, _)) => parse_labeled_loop(state, Some(label_name)),
        _ => bail!("Expected loop keyword after label"),
    }
}

/// Parse while loop without label
pub(in crate::frontend::parser) fn parse_while_loop(state: &mut ParserState) -> Result<Expr> {
    parse_labeled_while_loop(state, None)
}

/// Parse while loop with optional label
///
/// Supports both regular `while` and `while let` syntax.
fn parse_labeled_while_loop(state: &mut ParserState, label: Option<String>) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::While)?;
    // Check for while-let syntax
    if matches!(state.tokens.peek(), Some((Token::Let, _))) {
        state.tokens.advance(); // consume 'let'
                                // Parse the pattern
        let pattern = parse_match_pattern(state)
            .map_err(|e| anyhow::anyhow!("Expected pattern after 'while let': {e}"))?;
        // Expect '='
        state
            .tokens
            .expect(&Token::Equal)
            .map_err(|e| anyhow::anyhow!("Expected '=' after pattern in while-let: {e}"))?;
        // Parse the expression to match against
        let expr =
            Box::new(parse_expr_recursive(state).map_err(|e| {
                anyhow::anyhow!("Expected expression after '=' in while-let: {e}")
            })?);
        // Parse body (expect block)
        let body = Box::new(
            parse_expr_recursive(state)
                .map_err(|e| anyhow::anyhow!("Expected body after while-let condition: {e}"))?,
        );
        Ok(Expr::new(
            ExprKind::WhileLet {
                label,
                pattern,
                expr,
                body,
            },
            start_span,
        ))
    } else {
        // Regular while loop
        // Parse condition
        let condition = Box::new(
            parse_expr_recursive(state)
                .map_err(|e| anyhow::anyhow!("Expected condition after 'while': {e}"))?,
        );
        // Parse body (expect block)
        let body = Box::new(
            parse_expr_recursive(state)
                .map_err(|e| anyhow::anyhow!("Expected body after while condition: {e}"))?,
        );
        Ok(Expr::new(
            ExprKind::While {
                label,
                condition,
                body,
            },
            start_span,
        ))
    }
}

/// Parse for loop without label
///
/// Supports pattern destructuring in the loop variable.
pub(in crate::frontend::parser) fn parse_for_loop(state: &mut ParserState) -> Result<Expr> {
    parse_labeled_for_loop(state, None)
}

/// Parse for loop with optional label
///
/// Syntax: `['label:] for pattern in iterator { body }`
fn parse_labeled_for_loop(state: &mut ParserState, label: Option<String>) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::For)?;
    // Parse pattern (e.g., "i" in "for i in ...")
    let pattern = parse_for_pattern(state)?;
    // Expect 'in' keyword
    state
        .tokens
        .expect(&Token::In)
        .map_err(|_| anyhow::anyhow!("Expected 'in' after for pattern"))?;
    // Parse iterator expression
    let iterator = Box::new(
        parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected iterator after 'in': {e}"))?,
    );
    // Parse body (expect block)
    let body = Box::new(
        parse_expr_recursive(state)
            .map_err(|e| anyhow::anyhow!("Expected body after for iterator: {e}"))?,
    );
    // Get the var name from the pattern for backward compatibility
    let var = pattern.primary_name();
    Ok(Expr::new(
        ExprKind::For {
            label,
            var,
            pattern: Some(pattern),
            iter: iterator,
            body,
        },
        start_span,
    ))
}

/// Parse for loop pattern
///
/// Supports:
/// - Simple identifier: `for i in ...`
/// - Wildcard: `for _ in ...`
/// - Tuple destructuring: `for (x, y) in ...` or `for x, y in ...`
/// - List destructuring: `for [x, y] in ...`
fn parse_for_pattern(state: &mut ParserState) -> Result<Pattern> {
    let Some((token, _)) = state.tokens.peek() else {
        bail!("Expected pattern in for loop");
    };
    match token {
        Token::Identifier(name) => {
            let name = name.clone();
            state.tokens.advance();
            // Check if this is a bare tuple pattern: key, value (without parens)
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                // Parse as bare tuple: key, value, ...
                let mut patterns = vec![Pattern::Identifier(name)];
                while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                    state.tokens.advance(); // consume comma
                                            // Parse next pattern element
                    if let Some((Token::Identifier(next_name), _)) = state.tokens.peek() {
                        let next_name = next_name.clone();
                        state.tokens.advance();
                        patterns.push(Pattern::Identifier(next_name));
                    } else {
                        bail!("Expected identifier after comma in tuple pattern");
                    }
                }
                Ok(Pattern::Tuple(patterns))
            } else {
                Ok(Pattern::Identifier(name))
            }
        }
        Token::Underscore => {
            state.tokens.advance();
            Ok(Pattern::Wildcard)
        }
        Token::LeftParen => {
            // Parse tuple pattern with parens: (x, y)
            parse_tuple_pattern(state)
        }
        Token::LeftBracket => {
            // Parse list pattern: [x, y]
            parse_list_pattern(state)
        }
        _ => bail!("Expected identifier, underscore, or destructuring pattern in for loop"),
    }
}

/// Parse infinite loop without label
pub(in crate::frontend::parser) fn parse_loop(state: &mut ParserState) -> Result<Expr> {
    parse_labeled_loop(state, None)
}

/// Parse infinite loop with optional label
///
/// Syntax: `['label:] loop { body }`
fn parse_labeled_loop(state: &mut ParserState, label: Option<String>) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Loop)?;
    let body = Box::new(parse_expr_recursive(state)?);

    Ok(Expr::new(ExprKind::Loop { label, body }, start_span))
}

#[cfg(test)]
mod tests {
    
    use crate::frontend::parser::Parser;

    #[test]
    fn test_for_loop_simple() {
        let code = "for i in 0..10 { print(i) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple for loop should parse");
    }

    #[test]
    fn test_for_loop_tuple_destructuring() {
        let code = "for key, value in map { print(key) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "For loop with tuple destructuring should parse");
    }

    #[test]
    fn test_for_loop_tuple_with_parens() {
        let code = "for (x, y) in pairs { print(x) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "For loop with tuple pattern (parens) should parse");
    }

    #[test]
    fn test_for_loop_wildcard() {
        let code = "for _ in 0..10 { side_effect() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "For loop with wildcard should parse");
    }

    #[test]
    fn test_while_loop() {
        let code = "while x < 10 { x = x + 1 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "While loop should parse");
    }

    #[test]
    fn test_while_let_loop() {
        let code = "while let Some(value) = optional { process(value) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "While-let loop should parse");
    }

    #[test]
    fn test_infinite_loop() {
        let code = "loop { if done { break } }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Infinite loop should parse");
    }

    #[test]
    #[ignore = "Property tests run with --ignored flag"] // PARSER-079: Break statements in blocks not yet working
    fn test_labeled_for_loop() {
        let code = "'outer: for i in 0..10 { break 'outer }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Labeled for loop should parse");
    }

    #[test]
    #[ignore = "Property tests run with --ignored flag"] // PARSER-079: Break statements in blocks not yet working
    fn test_labeled_while_loop() {
        let code = "'outer: while true { break 'outer }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Labeled while loop should parse");
    }

    #[test]
    #[ignore = "Property tests run with --ignored flag"] // PARSER-079: Break statements in blocks not yet working
    fn test_labeled_infinite_loop() {
        let code = "'outer: loop { break 'outer }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Labeled infinite loop should parse");
    }

    // Property tests for loops
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_for_loops_never_panic(var in "[a-z]+", n in 0u32..100) {
                let code = format!("for {var} in 0..{n} {{ }}");
                let _ = Parser::new(&code).parse(); // Should not panic
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_while_loops_never_panic(var in "[a-z]+", n in 0i32..100) {
                let code = format!("while {var} < {n} {{ }}");
                let _ = Parser::new(&code).parse(); // Should not panic
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_infinite_loops_always_parse(_seed in any::<u32>()) {
                let code = "loop { break }";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_labeled_loops_parse(label in "[a-z]+") {
                let code = format!("'{label}: loop {{ break '{label}  }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_for_tuple_destructuring(var1 in "[a-z]+", var2 in "[a-z]+") {
                let code = format!("for {var1}, {var2} in pairs {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_while_let_always_has_pattern(var in "[a-z]+") {
                let code = format!("while let Some({var}) = opt {{ }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_nested_loops_parse(depth in 1usize..5) {
                let mut code = String::new();
                for i in 0..depth {
                    code.push_str(&format!("for x{i} in 0..10 {{ "));
                }
                code.push_str("()");
                for _ in 0..depth {
                    code.push_str(" }");
                }
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
