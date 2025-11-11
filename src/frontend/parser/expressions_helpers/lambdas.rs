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
use crate::frontend::parser::{bail, parse_expr_recursive, ParserState, Result};
use crate::frontend::parser::utils::parse_type;

/// Parse no-parameter lambda: `|| body`
///
/// Syntax: `|| expr`
pub(in crate::frontend::parser) fn parse_lambda_no_params(
    state: &mut ParserState,
) -> Result<Expr> {
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

/// Parse pipe-delimited lambda: `|param, param| body`
///
/// Supports zero or more parameters separated by commas.
pub(in crate::frontend::parser) fn parse_lambda_expression(
    state: &mut ParserState,
) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Pipe)?;
    let mut params = Vec::new();

    // Parse parameters (with optional type annotations)
    while !matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
            let param_name = name.clone();
            state.tokens.advance();

            // PARSER-077: Check for type annotation (|x: i32| support)
            let ty = if matches!(state.tokens.peek(), Some((Token::Colon, _))) {
                state.tokens.advance(); // consume :
                parse_type(state)?
            } else {
                // Inferred type
                Type {
                    kind: TypeKind::Named("_".to_string()),
                    span: start_span,
                }
            };

            params.push((Pattern::Identifier(param_name), ty));

            // Check for comma
            if matches!(state.tokens.peek(), Some((Token::Comma, _))) {
                state.tokens.advance();
            }
        } else {
            bail!("Expected parameter name in lambda");
        }
    }

    state
        .tokens
        .expect(&Token::Pipe)
        .map_err(|_| anyhow::anyhow!("Expected '|' after lambda parameters"))?;

    // SPEC-001-A: Parse optional return type annotation (|x: i32| -> i32 support)
    let _return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        Some(parse_type(state)?)
    } else {
        None
    };
    // Note: return_type is parsed but not yet stored in Lambda AST node
    // This is intentional - keeps AST simple while allowing syntax to parse

    // Parse body
    let body = Box::new(parse_expr_recursive(state)?);

    // Convert (Pattern, Type) pairs to Param structs
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
            "[a-z]+"
                .prop_filter("Must not be a keyword", |s| {
                    !matches!(
                        s.as_str(),
                        "fn" | "fun" | "let" | "var" | "if" | "else" | "for" | "while"
                            | "loop" | "match" | "break" | "continue" | "return" | "async"
                            | "await" | "try" | "catch" | "throw" | "in" | "as" | "is"
                            | "self" | "super" | "mod" | "use" | "pub" | "const" | "static"
                            | "mut" | "ref" | "type" | "struct" | "enum" | "trait" | "impl"
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
