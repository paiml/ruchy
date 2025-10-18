//! Error handling expression parsing
//!
//! Handles parsing of try-catch-finally constructs:
//! - Try blocks: `try { ... }`
//! - Catch clauses: `catch (e) { ... }` or `catch e { ... }`
//! - Finally blocks: `finally { ... }`
//! - Validation: Ensures at least one catch or finally clause
//!
//! # Examples
//! ```ruchy
//! // Basic try-catch
//! try {
//!     risky_operation()
//! } catch (e) {
//!     handle_error(e)
//! }
//!
//! // Try-catch-finally
//! try {
//!     connect_to_database()
//! } catch (e) {
//!     log_error(e)
//! } finally {
//!     cleanup()
//! }
//!
//! // Multiple catch clauses
//! try {
//!     process_data()
//! } catch (NetworkError) {
//!     retry()
//! } catch (e) {
//!     fallback(e)
//! }
//!
//! // Catch without parentheses
//! try {
//!     execute()
//! } catch e {
//!     handle(e)
//! }
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{CatchClause, Expr, ExprKind, Pattern};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, ParserState, Result};

/// Parse try-catch-finally block
///
/// Syntax: `try { ... } catch (e) { ... } finally { ... }`
pub(in crate::frontend::parser) fn parse_try_catch(state: &mut ParserState) -> Result<Expr> {
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

/// Parse try block
///
/// Delegates to collections module for block parsing.
fn parse_try_block(state: &mut ParserState) -> Result<Box<Expr>> {
    // parse_block expects and consumes the left brace
    Ok(Box::new(
        crate::frontend::parser::collections::parse_block(state)?,
    ))
}

/// Parse catch clauses
///
/// Supports multiple catch clauses: `catch (e1) { ... } catch (e2) { ... }`
fn parse_catch_clauses(state: &mut ParserState) -> Result<Vec<CatchClause>> {
    let mut catch_clauses = Vec::new();
    while matches!(state.tokens.peek(), Some((Token::Catch, _))) {
        state.tokens.advance(); // consume 'catch'
        let pattern = parse_catch_pattern(state)?;
        let body = parse_catch_body(state)?;
        catch_clauses.push(CatchClause { pattern, body });
    }
    Ok(catch_clauses)
}

/// Parse catch pattern
///
/// Supports both `catch (e)` and `catch e` syntax
fn parse_catch_pattern(state: &mut ParserState) -> Result<Pattern> {
    // Check if using parentheses syntax: catch (e)
    let has_parens = matches!(state.tokens.peek(), Some((Token::LeftParen, _)));

    if has_parens {
        state.tokens.expect(&Token::LeftParen)?;
    }

    let pattern = if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let name = name.clone();
        state.tokens.advance();
        Pattern::Identifier(name)
    } else {
        bail!("Expected identifier in catch clause");
    };

    if has_parens {
        state.tokens.expect(&Token::RightParen)?;
    }

    Ok(pattern)
}

/// Parse catch body
///
/// Delegates to collections module for block parsing.
fn parse_catch_body(state: &mut ParserState) -> Result<Box<Expr>> {
    // parse_block expects and consumes the left brace
    Ok(Box::new(
        crate::frontend::parser::collections::parse_block(state)?,
    ))
}

/// Parse optional finally block
///
/// Returns Some(block) if finally clause present, None otherwise.
fn parse_finally_block(state: &mut ParserState) -> Result<Option<Box<Expr>>> {
    if matches!(state.tokens.peek(), Some((Token::Finally, _))) {
        state.tokens.advance(); // consume 'finally'
                                // parse_block expects and consumes the left brace
        Ok(Some(Box::new(
            crate::frontend::parser::collections::parse_block(state)?,
        )))
    } else {
        Ok(None)
    }
}

/// Validate try-catch structure
///
/// Ensures at least one catch clause or finally block is present.
fn validate_try_catch_structure(
    catch_clauses: &[CatchClause],
    finally_block: Option<&Expr>,
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

    #[test]
    fn test_try_catch_basic() {
        let code = "try { risky() } catch (e) { handle(e) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Basic try-catch should parse");
    }

    #[test]
    fn test_try_catch_without_parens() {
        let code = "try { risky() } catch e { handle(e) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Try-catch without parens should parse");
    }

    #[test]
    fn test_try_catch_finally() {
        let code = "try { connect() } catch (e) { log(e) } finally { cleanup() }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Try-catch-finally should parse successfully"
        );
    }

    #[test]
    fn test_try_finally_no_catch() {
        let code = "try { operation() } finally { cleanup() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Try-finally without catch should parse");
    }

    #[test]
    fn test_try_multiple_catch() {
        let code = "try { process() } catch (NetworkError) { retry() } catch (e) { fallback(e) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Try with multiple catch clauses should parse");
    }

    #[test]
    fn test_try_without_catch_or_finally() {
        let code = "try { operation() }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_err(),
            "Try without catch or finally should fail validation"
        );
    }

    #[test]
    fn test_nested_try_catch() {
        let code = "try { try { inner() } catch (e) { handle(e) } } catch (e) { outer_handle(e) }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested try-catch should parse");
    }

    #[test]
    fn test_try_catch_with_complex_body() {
        let code = "try { let x = 10; if x > 5 { risky(x) } else { safe(x) } } catch (e) { log(e); recover() }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Try-catch with complex body should parse successfully"
        );
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_try_catch_always_parses(_seed in any::<u32>()) {
                let code = "try { 42 } catch (e) { 0 }";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_try_catch_with_identifier(err_name in "[a-z]+") {
                let code = format!("try {{ 42 }} catch ({}) {{ 0 }}", err_name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_try_finally_parses(val in 0i32..100) {
                let code = format!("try {{ {} }} finally {{ cleanup() }}", val);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_multiple_catch_parses(n in 1usize..5) {
                let mut code = String::from("try { risky() }");
                for i in 0..n {
                    code.push_str(&format!(" catch (e{}) {{ handle{}() }}", i, i));
                }
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_try_without_handlers_fails(_seed in any::<u32>()) {
                let code = "try { operation() }";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_err());
            }

            #[test]
            #[ignore]
            fn prop_nested_try_catch_parses(depth in 1usize..4) {
                let mut code = String::new();
                for _ in 0..depth {
                    code.push_str("try { ");
                }
                code.push_str("42");
                for i in 0..depth {
                    code.push_str(&format!(" }} catch (e{}) {{ 0 }}", i));
                }
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_catch_without_parens_parses(err_name in "[a-z]+") {
                let code = format!("try {{ 42 }} catch {} {{ 0 }}", err_name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
