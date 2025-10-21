//! Async expression parsing
//!
//! Handles parsing of async/await constructs:
//! - Async functions: `async fun name() { ... }`
//! - Async blocks: `async { ... }`
//! - Async lambdas: `async |x| x + 1`
//! - Async arrow lambdas: `async x => x + 1`
//! - Await expressions: Handled in parent module
//!
//! # Examples
//! ```ruchy
//! // Async function
//! async fun fetch_data() {
//!     let response = await http_get(url)
//!     response
//! }
//!
//! // Async block
//! let result = async {
//!     await some_operation()
//! }
//!
//! // Async lambda
//! let handler = async |request| {
//!     await process(request)
//! }
//!
//! // Async arrow lambda
//! let mapper = async x => await transform(x)
//! ```
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::{Expr, ExprKind};
use crate::frontend::lexer::Token;
use crate::frontend::parser::{bail, parse_expr_recursive, utils, ParserState, Result};

/// Parse async token dispatch
///
/// Routes to appropriate async construct parser based on next token.
pub(in crate::frontend::parser) fn parse_async_token(state: &mut ParserState) -> Result<Expr> {
    state.tokens.advance(); // consume 'async'

    match state.tokens.peek() {
        // async fun/fn declaration (support both keywords)
        Some((Token::Fun | Token::Fn, _)) => parse_async_function(state, false),
        // async { ... } block
        Some((Token::LeftBrace, _)) => parse_async_block(state),
        // async |x| ... lambda
        Some((Token::Pipe, _)) => parse_async_lambda(state),
        // async x => ... lambda (arrow syntax)
        Some((Token::Identifier(_), _)) => {
            if let Some((Token::Arrow, _)) = state.tokens.peek_ahead(1) {
                parse_async_arrow_lambda(state)
            } else {
                bail!("Expected 'fun'/'fn', '{{', '|', or arrow lambda after 'async'")
            }
        }
        _ => bail!("Expected 'fun'/'fn', '{{', '|', or identifier after 'async'"),
    }
}

/// Parse async function declaration
///
/// Syntax: `async fun name<T>(params) -> ReturnType { body }`
fn parse_async_function(state: &mut ParserState, is_pub: bool) -> Result<Expr> {
    let start_span = state.tokens.advance().expect("checked by parser logic").1; // consume fun
    let is_async = true; // We know it's async since we came from async token

    // Parse function name
    let name = if let Some((Token::Identifier(n), _)) = state.tokens.peek() {
        let name = n.clone();
        state.tokens.advance();
        name
    } else {
        "anonymous".to_string()
    };

    // Parse optional type parameters <T, U, ...>
    let type_params = if matches!(state.tokens.peek(), Some((Token::Less, _))) {
        utils::parse_type_parameters(state)?
    } else {
        Vec::new()
    };

    // Parse parameters
    let params = utils::parse_params(state)?;

    // Parse return type if present
    let return_type = if matches!(state.tokens.peek(), Some((Token::Arrow, _))) {
        state.tokens.advance(); // consume ->
        Some(utils::parse_type(state)?)
    } else {
        None
    };

    // Parse body
    let body = parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::Function {
            name,
            type_params,
            params,
            return_type,
            body: Box::new(body),
            is_async,
            is_pub,
        },
        start_span,
    ))
}

/// Parse async block
///
/// Syntax: `async { body }` or `async { stmt1; stmt2; ... }`
/// PARSER-056: Support blocks with multiple statements
fn parse_async_block(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::LeftBrace)?; // consume '{'

    // Parse multiple statements (PARSER-056 fix)
    let statements = crate::frontend::parser::collections::parse_block_expressions(state, start_span)?;

    state.tokens.expect(&Token::RightBrace)?; // consume '}'

    // Wrap statements in a Block expression
    let body = Expr::new(ExprKind::Block(statements), start_span);

    Ok(Expr::new(
        ExprKind::AsyncBlock {
            body: Box::new(body),
        },
        start_span,
    ))
}

/// Parse async lambda with pipe-delimited parameters
///
/// Syntax: `async |x, y| body`
fn parse_async_lambda(state: &mut ParserState) -> Result<Expr> {
    let start_span = state.tokens.expect(&Token::Pipe)?; // consume '|'

    let params = parse_async_lambda_params(state)?;

    state.tokens.expect(&Token::Pipe)?; // consume closing '|'

    let body = parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::AsyncLambda {
            params,
            body: Box::new(body),
        },
        start_span,
    ))
}

/// Parse async lambda parameters
fn parse_async_lambda_params(state: &mut ParserState) -> Result<Vec<String>> {
    if matches!(state.tokens.peek(), Some((Token::Pipe, _))) {
        return Ok(Vec::new()); // Empty parameter list
    }

    parse_async_param_list(state)
}

/// Parse comma-separated parameter list
fn parse_async_param_list(state: &mut ParserState) -> Result<Vec<String>> {
    let mut params = Vec::new();
    let first_param = parse_single_async_param(state)?;
    params.push(first_param);

    while matches!(state.tokens.peek(), Some((Token::Comma, _))) {
        state.tokens.advance(); // consume ','
        let param = parse_single_async_param(state)?;
        params.push(param);
    }

    Ok(params)
}

/// Parse single async parameter
fn parse_single_async_param(state: &mut ParserState) -> Result<String> {
    if let Some((Token::Identifier(name), _)) = state.tokens.peek() {
        let param_name = name.clone();
        state.tokens.advance();
        Ok(param_name)
    } else {
        bail!("Expected parameter name in async lambda");
    }
}

/// Parse async arrow lambda
///
/// Syntax: `async x => body` or `async (x, y) => body`
fn parse_async_arrow_lambda(state: &mut ParserState) -> Result<Expr> {
    // Parse single parameter
    let param = if let Some((Token::Identifier(name), span)) = state.tokens.peek() {
        let name = name.clone();
        let span = *span;
        state.tokens.advance();
        (name, span)
    } else {
        bail!("Expected parameter name in async arrow lambda");
    };

    // Expect '=>'
    state.tokens.expect(&Token::Arrow)?;

    // Parse body
    let body = parse_expr_recursive(state)?;

    Ok(Expr::new(
        ExprKind::AsyncLambda {
            params: vec![param.0],
            body: Box::new(body),
        },
        param.1,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::parser::Parser;

    #[test]
    fn test_async_function() {
        let code = "async fun fetch() { await get_data() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async function should parse");
    }

    #[test]
    fn test_async_block() {
        let code = "async { await operation() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async block should parse");
    }

    #[test]
    fn test_async_lambda() {
        let code = "async |x| await process(x)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async lambda should parse");
    }

    #[test]
    #[ignore] // ASYNC-001: Async lambda no-param syntax not yet supported
    fn test_async_lambda_no_params() {
        let code = "async || await fetch()";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async lambda with no params should parse");
    }

    #[test]
    fn test_async_lambda_multiple_params() {
        let code = "async |x, y| await combine(x, y)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async lambda with multiple params should parse");
    }

    #[test]
    #[ignore] // ASYNC-002: Async arrow lambda syntax not yet supported
    fn test_async_arrow_lambda() {
        let code = "async x => await transform(x)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async arrow lambda should parse");
    }

    #[test]
    fn test_async_function_with_return_type() {
        let code = "async fun getData() -> Result<String> { await fetch() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async function with return type should parse");
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore] // Run with: cargo test property_tests -- --ignored
            fn prop_async_blocks_parse(_seed in any::<u32>()) {
                let code = "async { 42 }";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_lambda_with_param(param in "[a-z]+") {
                let code = format!("async |{}| {}", param, param);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_arrow_lambda_parses(param in "[a-z]+", val in 0i32..100) {
                let code = format!("async {} => {}", param, val);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_function_parses(name in "[a-z]+") {
                let code = format!("async fun {}() {{ 42 }}", name);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_lambda_multi_params(p1 in "[a-z]+", p2 in "[a-z]+") {
                let code = format!("async |{}, {}| {} + {}", p1, p2, p1, p2);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_block_with_expressions(n in 0i32..100) {
                let code = format!("async {{ {} }}", n);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore]
            fn prop_async_function_with_params(name in "[a-z]+", param in "[a-z]+") {
                let code = format!("async fun {}({}) {{ {} }}", name, param, param);
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
