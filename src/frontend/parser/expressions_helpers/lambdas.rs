//! Lambda expression parsing
//!
//! Handles parsing of lambda/anonymous function expressions:
//! - No-parameter lambdas: `|| expr`
//! - Pipe-delimited parameters: `|x| expr` or `|x, y| expr`
//! - Arrow syntax conversion: `x => expr` or `(x, y) => expr`
//!
//! # Examples
//! ```ruchy
//! // No parameters
//! let f = || 42;
//!
//! // Single parameter
//! let double = |x| x * 2;
//!
//! // Multiple parameters
//! let add = |a, b| a + b;
//!
//! // Arrow syntax (tuple converted to params)
//! let square = x => x * x;
//! let sum = (a, b) => a + b;
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind, Param, Pattern, Span, Type, TypeKind};
use crate::frontend::lexer::Token;
use crate::frontend::parser::utils::parse_type;
use crate::frontend::parser::{bail, parse_expr_recursive, ParserState, Result};

/// Parse no-parameter lambda: `|| body`
///
/// Syntax: `|| expr`
pub(in crate::frontend::parser) fn parse_lambda_no_params(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::OrOr)?;
    // Parse the body
    let body = Box::new(parse_expr_recursive(state)?);
    Ok(Expr::new(
        ExprKind::Lambda {
            params: vec![],
            body,
        },
        start_span,
    ))
}

/// Parse lambda from arrow expression: `x => body` or `(x, y) => body`
///
/// Converts identifier or tuple expression into lambda parameters.
pub(in crate::frontend::parser) fn parse_lambda_from_expr(
    state: &mut ParserState,
    expr: Expr,
    start_span: Span,
) -> Result<Expr> {
    // Convert (x) => expr or (x, y) => expr syntax
    state.tokens.advance(); // consume =>

    // Convert the expression to parameters
    let params = match &expr.kind {
        ExprKind::Identifier(name) => vec![Param {
            pattern: Pattern::Identifier(name.clone()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: expr.span,
            },
            default_value: None,
            is_mutable: false,
            span: expr.span,
        }],
        ExprKind::Tuple(elements) => {
            // Convert tuple elements to parameters
            elements
                .iter()
                .map(|elem| match &elem.kind {
                    ExprKind::Identifier(name) => Ok(Param {
                        pattern: Pattern::Identifier(name.clone()),
                        ty: Type {
                            kind: TypeKind::Named("_".to_string()),
                            span: elem.span,
                        },
                        default_value: None,
                        is_mutable: false,
                        span: elem.span,
                    }),
                    _ => bail!("Expected identifier in lambda parameter"),
                })
                .collect::<Result<Vec<_>>>()?
        }
        _ => bail!("Expected identifier or tuple in lambda parameter"),
    };

    // Parse the body
    let body = Box::new(parse_expr_recursive(state)?);
    Ok(Expr::new(ExprKind::Lambda { params, body }, start_span))
}

/// Parse a single lambda parameter with optional type annotation
fn parse_lambda_param(state: &mut ParserState, default_span: Span) -> Result<(Pattern, Type)> {
    let Some((Token::Identifier(name), _)) = state.tokens.peek() else {
        bail!("Expected parameter name in lambda");
    };
    let param_name = name.clone();
    state.tokens.advance();

    // PARSER-077: Check for type annotation (|x: i32| support)
    let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
        state.tokens.advance(); // consume :
        parse_type(state)?
    } else {
        Type {
            kind: TypeKind::Named("_".to_string()),
            span: default_span,
        }
    };

    Ok((Pattern::Identifier(param_name), ty))
}

/// Parse pipe-delimited lambda: `|param, param| body`
///
/// Supports zero or more parameters separated by commas.
pub(in crate::frontend::parser) fn parse_lambda_expression(
    state: &mut ParserState,
) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Pipe)?;
    let mut params = Vec::new();

    // Parse parameters
    while !matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        params.push(parse_lambda_param(state, start_span)?);

        if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
            state.tokens.advance();
        }
    }

    state
        .tokens
        .expect(&Token::Pipe)
        .map_err(|_| anyhow::anyhow!("Expected '|' after lambda parameters"))?;

    // SPEC-001-A: Parse optional return type annotation
    if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance();
        let _return_type = parse_type(state)?;
    }

    // Parse body and convert params to Param structs
    let body = Box::new(parse_expr_recursive(state)?);
    let params = params
        .into_iter()
        .map(|(pattern, ty)| Param {
            pattern,
            ty,
            span: start_span,
            is_mutable: false,
            default_value: None,
        })
        .collect();

    Ok(Expr::new(ExprKind::Lambda { params, body }, start_span))
}

#[cfg(test)]
mod tests {

    use crate::frontend::parser::Parser;

    #[test]
    fn test_lambda_no_params() {
        let code = "|| 42";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "No-parameter lambda should parse");
    }

    #[test]
    fn test_lambda_single_param() {
        let code = "|x| x * 2";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Single-parameter lambda should parse");
    }

    #[test]
    fn test_lambda_multiple_params() {
        let code = "|a, b| a + b";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Multi-parameter lambda should parse");
    }

    #[test]
    fn test_arrow_single_param() {
        let code = "x => x * 2";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Arrow syntax single param should parse");
    }

    #[test]
    fn test_arrow_tuple_params() {
        let code = "(a, b) => a + b";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Arrow syntax tuple params should parse");
    }

    #[test]
    fn test_lambda_with_block() {
        let code = "|x| { let y = x * 2; y }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Lambda with block should parse");
    }

    #[test]
    fn test_nested_lambda() {
        let code = "|x| |y| x + y";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Nested lambda should parse");
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
    // Lambda produces Lambda ExprKind
    // ============================================================

    #[test]
    fn test_lambda_produces_lambda_exprkind() {
        let expr = parse("|| 1").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Lambda { .. }),
                "Should produce Lambda ExprKind"
            );
        }
    }

    #[test]
    fn test_lambda_with_params_produces_lambda() {
        let expr = parse("|x| x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Lambda { .. }),
                "Param lambda should produce Lambda"
            );
        }
    }

    // ============================================================
    // No-parameter lambda variations
    // ============================================================

    #[test]
    fn test_lambda_no_params_simple() {
        let result = parse("|| 42");
        assert!(result.is_ok(), "Simple no-param lambda should parse");
    }

    #[test]
    fn test_lambda_no_params_string() {
        let result = parse("|| \"hello\"");
        assert!(result.is_ok(), "No-param lambda returning string should parse");
    }

    #[test]
    fn test_lambda_no_params_boolean() {
        let result = parse("|| true");
        assert!(result.is_ok(), "No-param lambda returning boolean should parse");
    }

    #[test]
    fn test_lambda_no_params_block() {
        let result = parse("|| { 42 }");
        assert!(result.is_ok(), "No-param lambda with block should parse");
    }

    #[test]
    fn test_lambda_no_params_call() {
        let result = parse("|| foo()");
        assert!(result.is_ok(), "No-param lambda with call should parse");
    }

    // ============================================================
    // Single parameter variations
    // ============================================================

    #[test]
    fn test_lambda_single_multiplication() {
        let result = parse("|x| x * 2");
        assert!(result.is_ok(), "Lambda multiplication should parse");
    }

    #[test]
    fn test_lambda_single_addition() {
        let result = parse("|n| n + 1");
        assert!(result.is_ok(), "Lambda addition should parse");
    }

    #[test]
    fn test_lambda_single_subtraction() {
        let result = parse("|v| v - 10");
        assert!(result.is_ok(), "Lambda subtraction should parse");
    }

    #[test]
    fn test_lambda_single_division() {
        let result = parse("|d| d / 2");
        assert!(result.is_ok(), "Lambda division should parse");
    }

    #[test]
    fn test_lambda_single_comparison() {
        let result = parse("|x| x > 0");
        assert!(result.is_ok(), "Lambda comparison should parse");
    }

    #[test]
    fn test_lambda_single_negation() {
        let result = parse("|b| !b");
        assert!(result.is_ok(), "Lambda negation should parse");
    }

    // ============================================================
    // Multiple parameter variations
    // ============================================================

    #[test]
    fn test_lambda_two_params_add() {
        let result = parse("|a, b| a + b");
        assert!(result.is_ok(), "Two-param add should parse");
    }

    #[test]
    fn test_lambda_two_params_multiply() {
        let result = parse("|x, y| x * y");
        assert!(result.is_ok(), "Two-param multiply should parse");
    }

    #[test]
    fn test_lambda_three_params() {
        let result = parse("|a, b, c| a + b + c");
        assert!(result.is_ok(), "Three-param lambda should parse");
    }

    #[test]
    fn test_lambda_four_params() {
        let result = parse("|a, b, c, d| a + b + c + d");
        assert!(result.is_ok(), "Four-param lambda should parse");
    }

    #[test]
    fn test_lambda_params_with_expression() {
        let result = parse("|x, y| (x + y) * 2");
        assert!(result.is_ok(), "Lambda with grouped expression should parse");
    }

    // ============================================================
    // Type annotations
    // ============================================================

    #[test]
    fn test_lambda_param_type_i32() {
        let result = parse("|x: i32| x");
        assert!(result.is_ok(), "Lambda with i32 type should parse");
    }

    #[test]
    fn test_lambda_param_type_string() {
        let result = parse("|s: String| s");
        assert!(result.is_ok(), "Lambda with String type should parse");
    }

    #[test]
    fn test_lambda_param_type_bool() {
        let result = parse("|b: bool| b");
        assert!(result.is_ok(), "Lambda with bool type should parse");
    }

    #[test]
    fn test_lambda_multiple_typed_params() {
        let result = parse("|x: i32, y: i32| x + y");
        assert!(result.is_ok(), "Lambda with multiple typed params should parse");
    }

    #[test]
    fn test_lambda_mixed_typed_params() {
        let result = parse("|x: i32, y| x + y");
        assert!(result.is_ok(), "Lambda with mixed typed params should parse");
    }

    // ============================================================
    // Arrow syntax variations
    // ============================================================

    #[test]
    fn test_arrow_simple() {
        let result = parse("x => x");
        assert!(result.is_ok(), "Simple arrow lambda should parse");
    }

    #[test]
    fn test_arrow_expression() {
        let result = parse("x => x * 2 + 1");
        assert!(result.is_ok(), "Arrow with expression should parse");
    }

    #[test]
    fn test_arrow_tuple_two() {
        let result = parse("(a, b) => a + b");
        assert!(result.is_ok(), "Arrow tuple two should parse");
    }

    #[test]
    fn test_arrow_tuple_three() {
        let result = parse("(a, b, c) => a + b + c");
        assert!(result.is_ok(), "Arrow tuple three should parse");
    }

    #[test]
    fn test_arrow_with_block() {
        let result = parse("x => { x * 2 }");
        assert!(result.is_ok(), "Arrow with block should parse");
    }

    // ============================================================
    // Complex body expressions
    // ============================================================

    #[test]
    fn test_lambda_with_if() {
        let result = parse("|x| if x > 0 { x } else { 0 }");
        assert!(result.is_ok(), "Lambda with if should parse");
    }

    #[test]
    fn test_lambda_with_match() {
        let result = parse("|opt| match opt { Some(v) => v, None => 0 }");
        assert!(result.is_ok(), "Lambda with match should parse");
    }

    #[test]
    fn test_lambda_with_let_block() {
        let result = parse("|x| { let y = x * 2; y + 1 }");
        assert!(result.is_ok(), "Lambda with let block should parse");
    }

    #[test]
    fn test_lambda_with_multiple_stmts() {
        let result = parse("|x| { let a = x; let b = a + 1; b }");
        assert!(result.is_ok(), "Lambda with multiple stmts should parse");
    }

    #[test]
    fn test_lambda_with_function_call() {
        let result = parse("|x| compute(x)");
        assert!(result.is_ok(), "Lambda with function call should parse");
    }

    #[test]
    fn test_lambda_with_method_call() {
        let result = parse("|s| s.len()");
        assert!(result.is_ok(), "Lambda with method call should parse");
    }

    // ============================================================
    // Nested lambdas
    // ============================================================

    #[test]
    fn test_nested_two_deep() {
        let result = parse("|x| |y| x + y");
        assert!(result.is_ok(), "Two-deep nested lambda should parse");
    }

    #[test]
    fn test_nested_three_deep() {
        let result = parse("|x| |y| |z| x + y + z");
        assert!(result.is_ok(), "Three-deep nested lambda should parse");
    }

    #[test]
    fn test_nested_arrow_in_pipe() {
        let result = parse("|f| x => f(x)");
        assert!(result.is_ok(), "Nested arrow in pipe should parse");
    }

    // ============================================================
    // Lambda as argument
    // ============================================================

    #[test]
    fn test_lambda_arg_to_map() {
        let result = parse("list.map(|x| x * 2)");
        assert!(result.is_ok(), "Lambda argument to map should parse");
    }

    #[test]
    fn test_lambda_arg_to_filter() {
        let result = parse("items.filter(|x| x > 0)");
        assert!(result.is_ok(), "Lambda argument to filter should parse");
    }

    #[test]
    fn test_lambda_arg_to_fold() {
        let result = parse("nums.fold(0, |acc, x| acc + x)");
        assert!(result.is_ok(), "Lambda argument to fold should parse");
    }

    #[test]
    fn test_lambda_arg_to_sort_by() {
        let result = parse("items.sort_by(|a, b| a.value - b.value)");
        assert!(result.is_ok(), "Lambda argument to sort_by should parse");
    }

    // ============================================================
    // Return type annotation
    // ============================================================

    #[test]
    fn test_lambda_return_type() {
        let result = parse("|x| -> i32 x * 2");
        assert!(result.is_ok(), "Lambda with return type should parse");
    }

    #[test]
    fn test_lambda_return_type_string() {
        let result = parse("|x| -> String format(x)");
        assert!(result.is_ok(), "Lambda with String return type should parse");
    }

    // ============================================================
    // Edge cases
    // ============================================================

    #[test]
    fn test_lambda_identity() {
        let result = parse("|x| x");
        assert!(result.is_ok(), "Identity lambda should parse");
    }

    #[test]
    fn test_lambda_constant() {
        // Note: underscore alone |_| not supported - use ignored param name
        let result = parse("|unused| 42");
        assert!(result.is_ok(), "Constant lambda should parse");
    }

    #[test]
    fn test_lambda_long_param_name() {
        let result = parse("|very_long_parameter_name| very_long_parameter_name");
        assert!(result.is_ok(), "Lambda with long param name should parse");
    }

    #[test]
    fn test_lambda_ignored_params() {
        // Note: underscore alone not supported - use prefixed names
        let result = parse("|unused1, unused2| 42");
        assert!(result.is_ok(), "Lambda with ignored params should parse");
    }

    #[test]
    fn test_lambda_single_char_params() {
        let result = parse("|a, b, c, d, e| a + b + c + d + e");
        assert!(result.is_ok(), "Lambda with single char params should parse");
    }

    // Property tests for lambdas
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        /// Helper: Generate valid identifiers (not keywords)
        ///
        /// Keywords like "fn", "if", "let" would cause parser failures.
        /// This strategy filters them out for property test validity.
        fn valid_identifier() -> impl Strategy<Value = String> {
            "[a-z]+".prop_filter("Must not be a keyword", |s| {
                !matches!(
                    s.as_str(),
                    "fn" | "fun"
                        | "let"
                        | "var"
                        | "if"
                        | "else"
                        | "for"
                        | "while"
                        | "loop"
                        | "match"
                        | "break"
                        | "continue"
                        | "return"
                        | "async"
                        | "await"
                        | "try"
                        | "catch"
                        | "throw"
                        | "in"
                        | "as"
                        | "is"
                        | "self"
                        | "super"
                        | "mod"
                        | "use"
                        | "pub"
                        | "const"
                        | "static"
                        | "mut"
                        | "ref"
                        | "type"
                        | "struct"
                        | "enum"
                        | "trait"
                        | "impl"
                )
            })
        }

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_no_param_lambdas_parse(_seed in any::<u32>()) {
                let code = "|| 42";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_single_param_lambdas_parse(param in valid_identifier()) {
                let code = format!("|{param}| {param}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multi_param_lambdas_parse(p1 in valid_identifier(), p2 in valid_identifier()) {
                let code = format!("|{p1}, {p2}| {p1} + {p2}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_arrow_syntax_parses(param in valid_identifier()) {
                let code = format!("{param} => {param} * 2");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_arrow_tuple_syntax_parses(p1 in valid_identifier(), p2 in valid_identifier()) {
                let code = format!("({p1}, {p2}) => {p1} + {p2}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_lambda_with_numbers(n in 0i32..100) {
                let code = format!("|x| x + {n}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_nested_lambdas_parse(p1 in valid_identifier(), p2 in valid_identifier()) {
                let code = format!("|{p1}| |{p2}| {p1} + {p2}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
