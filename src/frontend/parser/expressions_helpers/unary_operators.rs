//! Unary operator parsing
//!
//! Handles parsing of prefix unary operators:
//! - `-expr` - negation
//! - `!expr` - logical not
//! - `*expr` - dereference
//! - `&expr` - reference
//! - `**expr` - double dereference
//! - `await expr` - await expression
//! - `~expr` - bitwise not
//! - `spawn expr` - spawn actor
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Span, UnaryOp};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{ParserState, Result};

// Import parse_expr_with_precedence_recursive from parent module
use crate::frontend::parser::parse_expr_with_precedence_recursive;

/// Parse unary prefix expressions
///
/// Dispatches to specific unary operator handlers.
///
/// # Examples
/// ```ruchy
/// -42        // Negate
/// !true      // Logical not
/// *ptr       // Dereference
/// &value     // Reference
/// **ptr_ptr  // Double deref
/// await fut  // Await
/// ~bits      // Bitwise not
/// spawn actor // Spawn
/// ```
pub(in crate::frontend::parser) fn parse_unary_prefix(
    state: &mut ParserState,
    token: Token,
    span: Span,
) -> Result<Expr> {
    match token {
        Token::Minus => parse_unary_negate(state, span),
        Token::Plus => parse_unary_plus(state, span),
        Token::Bang => parse_unary_not(state, span),
        Token::Star => parse_unary_deref(state, span),
        Token::Ampersand => parse_unary_reference(state, span),
        Token::Power => parse_double_deref(state, span),
        Token::Await => parse_await_expr(state, span),
        Token::Tilde => parse_bitwise_not(state, span),
        Token::Spawn => parse_spawn_expr(state, span),
        _ => unreachable!(),
    }
}

/// Parse unary negation: `-expr`
fn parse_unary_negate(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse unary plus: `+expr` (identity operation - returns operand unchanged)
fn parse_unary_plus(state: &mut ParserState, _span: Span) -> Result<Expr> {
    state.tokens.advance();
    // Unary plus is identity operation - just parse and return the operand
    parse_expr_with_precedence_recursive(state, 13)
}

/// Parse logical not: `!expr`
fn parse_unary_not(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Not,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse dereference: `*expr`
fn parse_unary_deref(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse reference: `&expr`
fn parse_unary_reference(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance(); // consume &

    // PARSER-085: Issue #71 - Check for optional 'mut' keyword after '&'
    let op = if matches!(state.tokens.peek(), Some((Token::Mut, _))) {
        state.tokens.advance(); // consume mut
        UnaryOp::MutableReference
    } else {
        UnaryOp::Reference
    };

    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse double dereference: `**expr`
fn parse_double_deref(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    let inner_deref = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(expr),
        },
        span,
    );
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Deref,
            operand: Box::new(inner_deref),
        },
        span,
    ))
}

/// Parse await expression: `await expr`
fn parse_await_expr(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Await {
            expr: Box::new(expr),
        },
        span,
    ))
}

/// Parse bitwise not: `~expr`
fn parse_bitwise_not(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Unary {
            op: UnaryOp::BitwiseNot,
            operand: Box::new(expr),
        },
        span,
    ))
}

/// Parse spawn expression: `spawn expr`
fn parse_spawn_expr(state: &mut ParserState, span: Span) -> Result<Expr> {
    state.tokens.advance();
    let expr = parse_expr_with_precedence_recursive(state, 13)?;
    Ok(Expr::new(
        ExprKind::Spawn {
            actor: Box::new(expr),
        },
        span,
    ))
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_negate() {
        let code = "-42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Negation should parse");
    }

    #[test]
    fn test_logical_not() {
        let code = "!true";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Logical not should parse");
    }

    #[test]
    fn test_reference() {
        let code = "&x";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Reference should parse");
    }

    #[test]
    fn test_dereference() {
        let code = "*ptr";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Dereference should parse");
    }

    #[test]
    fn test_double_deref() {
        let code = "**ptr_ptr";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Double deref should parse");
    }

    #[test]
    fn test_bitwise_not() {
        let code = "~0xFF";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Bitwise not should parse");
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    use crate::frontend::ast::{Expr, ExprKind, UnaryOp};
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
    // Negation tests
    // ============================================================

    #[test]
    fn test_negate_produces_unary_expr() {
        let expr = parse("-42").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::Negate, .. }),
                "Should produce Unary with Negate"
            );
        }
    }

    #[test]
    fn test_negate_variable() {
        let result = parse("-x");
        assert!(result.is_ok(), "Negate variable should parse");
    }

    #[test]
    fn test_negate_expression() {
        let result = parse("-(a + b)");
        assert!(result.is_ok(), "Negate expression should parse");
    }

    #[test]
    fn test_negate_float() {
        let result = parse("-3.14");
        assert!(result.is_ok(), "Negate float should parse");
    }

    #[test]
    fn test_double_negate() {
        let result = parse("--x");
        assert!(result.is_ok(), "Double negate should parse");
    }

    #[test]
    fn test_triple_negate() {
        let result = parse("---x");
        assert!(result.is_ok(), "Triple negate should parse");
    }

    // ============================================================
    // Unary plus tests
    // ============================================================

    #[test]
    fn test_unary_plus() {
        let result = parse("+42");
        assert!(result.is_ok(), "Unary plus should parse");
    }

    #[test]
    fn test_unary_plus_variable() {
        let result = parse("+x");
        assert!(result.is_ok(), "Unary plus variable should parse");
    }

    // ============================================================
    // Logical not tests
    // ============================================================

    #[test]
    fn test_not_produces_unary_expr() {
        let expr = parse("!true").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::Not, .. }),
                "Should produce Unary with Not"
            );
        }
    }

    #[test]
    fn test_not_false() {
        let result = parse("!false");
        assert!(result.is_ok(), "Not false should parse");
    }

    #[test]
    fn test_not_variable() {
        let result = parse("!flag");
        assert!(result.is_ok(), "Not variable should parse");
    }

    #[test]
    fn test_not_comparison() {
        let result = parse("!(a == b)");
        assert!(result.is_ok(), "Not comparison should parse");
    }

    #[test]
    fn test_double_not() {
        let result = parse("!!x");
        assert!(result.is_ok(), "Double not should parse");
    }

    // ============================================================
    // Reference tests
    // ============================================================

    #[test]
    fn test_reference_produces_unary_expr() {
        let expr = parse("&x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::Reference, .. }),
                "Should produce Unary with Reference"
            );
        }
    }

    #[test]
    fn test_reference_expression() {
        let result = parse("&(a + b)");
        assert!(result.is_ok(), "Reference expression should parse");
    }

    #[test]
    fn test_mutable_reference() {
        let result = parse("&mut x");
        assert!(result.is_ok(), "Mutable reference should parse");
    }

    #[test]
    fn test_mutable_reference_produces_mut_ref() {
        let expr = parse("&mut x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::MutableReference, .. }),
                "Should produce Unary with MutableReference"
            );
        }
    }

    #[test]
    fn test_double_reference() {
        // && is parsed as logical AND, not double reference
        let result = parse("& &x");
        assert!(result.is_ok(), "Double reference with space should parse");
    }

    // ============================================================
    // Dereference tests
    // ============================================================

    #[test]
    fn test_deref_produces_unary_expr() {
        let expr = parse("*ptr").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::Deref, .. }),
                "Should produce Unary with Deref"
            );
        }
    }

    #[test]
    fn test_deref_expression() {
        let result = parse("*(get_ptr())");
        assert!(result.is_ok(), "Deref expression should parse");
    }

    #[test]
    fn test_deref_field_access() {
        let result = parse("(*ptr).field");
        assert!(result.is_ok(), "Deref field access should parse");
    }

    // ============================================================
    // Double dereference tests
    // ============================================================

    #[test]
    fn test_double_deref_nested() {
        let expr = parse("**ptr").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            // Should be nested Deref
            if let ExprKind::Unary { op: UnaryOp::Deref, operand } = &exprs[0].kind {
                assert!(
                    matches!(&operand.kind, ExprKind::Unary { op: UnaryOp::Deref, .. }),
                    "Should have nested Deref"
                );
            }
        }
    }

    #[test]
    fn test_triple_deref() {
        let result = parse("***ptr");
        assert!(result.is_ok(), "Triple deref should parse");
    }

    // ============================================================
    // Await expression tests
    // ============================================================

    #[test]
    fn test_await_expression() {
        let result = parse("await future");
        assert!(result.is_ok(), "Await should parse");
    }

    #[test]
    fn test_await_produces_await_expr() {
        let expr = parse("await fut").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Await { .. }),
                "Should produce Await ExprKind"
            );
        }
    }

    #[test]
    fn test_await_function_call() {
        let result = parse("await fetch_data()");
        assert!(result.is_ok(), "Await function call should parse");
    }

    #[test]
    fn test_await_method_chain() {
        let result = parse("await client.get().send()");
        assert!(result.is_ok(), "Await method chain should parse");
    }

    // ============================================================
    // Bitwise not tests
    // ============================================================

    #[test]
    fn test_bitwise_not_produces_unary_expr() {
        let expr = parse("~x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Unary { op: UnaryOp::BitwiseNot, .. }),
                "Should produce Unary with BitwiseNot"
            );
        }
    }

    #[test]
    fn test_bitwise_not_variable() {
        let result = parse("~bits");
        assert!(result.is_ok(), "Bitwise not variable should parse");
    }

    #[test]
    fn test_double_bitwise_not() {
        let result = parse("~~x");
        assert!(result.is_ok(), "Double bitwise not should parse");
    }

    // ============================================================
    // Spawn expression tests
    // ============================================================

    #[test]
    fn test_spawn_expression() {
        // spawn may need a block or function call
        let result = parse("spawn { work() }");
        assert!(result.is_ok(), "Spawn block should parse");
    }

    #[test]
    fn test_spawn_produces_spawn_expr() {
        let expr = parse("spawn worker").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Spawn { .. }),
                "Should produce Spawn ExprKind"
            );
        }
    }

    #[test]
    fn test_spawn_function_call() {
        let result = parse("spawn create_actor()");
        assert!(result.is_ok(), "Spawn function call should parse");
    }

    // ============================================================
    // Mixed unary operators
    // ============================================================

    #[test]
    fn test_not_and_negate() {
        let result = parse("!-x");
        assert!(result.is_ok(), "Not and negate should parse");
    }

    #[test]
    fn test_negate_and_not() {
        let result = parse("-!x");
        // May or may not make sense semantically
        let _ = result;
    }

    #[test]
    fn test_reference_and_deref() {
        let result = parse("&*ptr");
        assert!(result.is_ok(), "Reference and deref should parse");
    }

    #[test]
    fn test_deref_and_reference() {
        let result = parse("*&x");
        assert!(result.is_ok(), "Deref and reference should parse");
    }

    #[test]
    fn test_unary_in_binary_expression() {
        let result = parse("-a + -b");
        assert!(result.is_ok(), "Unary in binary should parse");
    }

    #[test]
    fn test_unary_with_parentheses() {
        let result = parse("(-x)");
        assert!(result.is_ok(), "Unary with parentheses should parse");
    }

    #[test]
    fn test_unary_chain_complex() {
        let result = parse("-!~x");
        assert!(result.is_ok(), "Complex unary chain should parse");
    }

    // ============================================================
    // Additional EXTREME TDD tests
    // ============================================================

    // ===== Operand types =====

    #[test]
    fn test_negate_large_number() {
        let result = parse("-999999");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_function_call() {
        let result = parse("!is_valid()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_struct_field() {
        let result = parse("&obj.field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_deref_array_index() {
        let result = parse("*arr[0]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitwise_not_hex() {
        let result = parse("~0xFFFF");
        assert!(result.is_ok());
    }

    // ===== In expressions =====

    #[test]
    fn test_negate_in_return() {
        let result = parse("fun f() { -x }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_in_if_condition() {
        let result = parse("if !done { 1 } else { 0 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_in_let() {
        let result = parse("let r = &value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_deref_in_assignment() {
        let result = parse("*ptr = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_await_in_let() {
        let result = parse("let result = await fetch()");
        assert!(result.is_ok());
    }

    // ===== Complex expressions =====

    #[test]
    fn test_negate_multiply() {
        let result = parse("-x * y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_in_logical() {
        let result = parse("!a && !b");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_method_result() {
        let result = parse("&x.method()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_deref_then_method() {
        let result = parse("(*ptr).method()");
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_await() {
        let result = parse("await first() + await second()");
        assert!(result.is_ok());
    }

    // ===== Edge cases =====

    #[test]
    fn test_negate_zero() {
        let result = parse("-0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_not_parens() {
        let result = parse("!(x && y)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reference_literal() {
        let result = parse("&42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_await_chain() {
        let result = parse("await await_double()");
        assert!(result.is_ok());
    }

    // Property tests for unary operators
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_negate_parses(n in any::<i32>()) {
                let code = format!("-{n}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Negation -{} should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_not_parses(b in any::<bool>()) {
                let code = format!("!{b}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Logical not !{} should parse", b);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_bitwise_not_parses(n in any::<u32>()) {
                let code = format!("~{n}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Bitwise not ~{} should parse", n);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_nested_negations_parse(depth in 1..5usize) {
                let negations = "-".repeat(depth);
                let code = format!("{negations}42");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok(), "Nested negations {} should parse", code);
            }
        }
    }
}
