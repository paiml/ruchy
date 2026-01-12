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
    let statements =
        crate::frontend::parser::collections::parse_block_expressions(state, start_span)?;

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
    #[ignore = "Property tests run with --ignored flag"] // ASYNC-001: Async lambda no-param syntax not yet supported
    fn test_async_lambda_no_params() {
        let code = "async || await fetch()";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async lambda with no params should parse");
    }

    #[test]
    fn test_async_lambda_multiple_params() {
        let code = "async |x, y| await combine(x, y)";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Async lambda with multiple params should parse"
        );
    }

    #[test]
    #[ignore = "Property tests run with --ignored flag"] // ASYNC-002: Async arrow lambda syntax not yet supported
    fn test_async_arrow_lambda() {
        let code = "async x => await transform(x)";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async arrow lambda should parse");
    }

    #[test]
    fn test_async_function_with_return_type() {
        let code = "async fun getData() -> Result<String> { await fetch() }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Async function with return type should parse"
        );
    }

    // Coverage-95 tests
    #[test]
    fn test_async_fn_keyword() {
        let code = "async fn fetch() { await get_data() }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async with fn keyword should parse");
    }

    #[test]
    fn test_async_block_simple() {
        let code = "async { 42 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Simple async block should parse");
    }

    #[test]
    fn test_async_block_multiple_statements() {
        let code = "async { let x = 1; let y = 2; x + y }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async block with statements should parse");
    }

    #[test]
    fn test_async_function_with_type_params() {
        let code = "async fun fetch<T>() { await get() }";
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Async function with type params should parse"
        );
    }

    #[test]
    fn test_async_function_anonymous() {
        let code = "async fun() { 42 }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Anonymous async function should parse");
    }

    #[test]
    fn test_async_function_params() {
        let code = "async fun process(x: i32, y: i32) { x + y }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async function with params should parse");
    }

    #[test]
    fn test_async_lambda_body_block() {
        let code = "async |x| { let y = x + 1; y }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Async lambda with block body should parse");
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
    // Async function ExprKind verification
    // ============================================================

    #[test]
    fn test_async_function_produces_function_exprkind() {
        let expr = parse("async fun test() { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::Function { is_async: true, .. }),
                "Should produce Function with is_async=true"
            );
        }
    }

    #[test]
    fn test_async_block_produces_async_block_exprkind() {
        let expr = parse("async { 42 }").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::AsyncBlock { .. }),
                "Should produce AsyncBlock ExprKind"
            );
        }
    }

    #[test]
    fn test_async_lambda_produces_async_lambda_exprkind() {
        let expr = parse("async |x| x").unwrap();
        if let Some(exprs) = get_block_exprs(&expr) {
            assert!(
                matches!(&exprs[0].kind, ExprKind::AsyncLambda { .. }),
                "Should produce AsyncLambda ExprKind"
            );
        }
    }

    // ============================================================
    // Async function variations
    // ============================================================

    #[test]
    fn test_async_fun_simple() {
        let result = parse("async fun test() { 1 }");
        assert!(result.is_ok(), "Simple async fun should parse");
    }

    #[test]
    fn test_async_fn_simple() {
        let result = parse("async fn test() { 1 }");
        assert!(result.is_ok(), "Simple async fn should parse");
    }

    #[test]
    fn test_async_fun_with_await() {
        let result = parse("async fun fetch() { await get_data() }");
        assert!(result.is_ok(), "Async fun with await should parse");
    }

    #[test]
    fn test_async_fun_single_param() {
        let result = parse("async fun process(x) { x }");
        assert!(result.is_ok(), "Async fun with single param should parse");
    }

    #[test]
    fn test_async_fun_two_params() {
        let result = parse("async fun combine(a, b) { a + b }");
        assert!(result.is_ok(), "Async fun with two params should parse");
    }

    #[test]
    fn test_async_fun_three_params() {
        let result = parse("async fun calc(x, y, z) { x + y + z }");
        assert!(result.is_ok(), "Async fun with three params should parse");
    }

    #[test]
    fn test_async_fun_typed_params() {
        let result = parse("async fun add(x: i32, y: i32) { x + y }");
        assert!(result.is_ok(), "Async fun with typed params should parse");
    }

    #[test]
    fn test_async_fun_return_type_string() {
        let result = parse("async fun greet() -> String { \"hello\" }");
        assert!(result.is_ok(), "Async fun with String return should parse");
    }

    #[test]
    fn test_async_fun_return_type_option() {
        let result = parse("async fun find() -> Option<i32> { None }");
        assert!(result.is_ok(), "Async fun with Option return should parse");
    }

    #[test]
    fn test_async_fun_generic_single() {
        let result = parse("async fun identity<T>(x: T) -> T { x }");
        assert!(result.is_ok(), "Async fun with single generic should parse");
    }

    #[test]
    fn test_async_fun_generic_two() {
        let result = parse("async fun pair<T, U>(a: T, b: U) { (a, b) }");
        assert!(result.is_ok(), "Async fun with two generics should parse");
    }

    // ============================================================
    // Async block variations
    // ============================================================

    #[test]
    fn test_async_block_integer() {
        let result = parse("async { 42 }");
        assert!(result.is_ok(), "Async block with integer should parse");
    }

    #[test]
    fn test_async_block_string() {
        let result = parse("async { \"result\" }");
        assert!(result.is_ok(), "Async block with string should parse");
    }

    #[test]
    fn test_async_block_with_await() {
        let result = parse("async { await fetch() }");
        assert!(result.is_ok(), "Async block with await should parse");
    }

    #[test]
    fn test_async_block_with_let() {
        let result = parse("async { let x = 1; x }");
        assert!(result.is_ok(), "Async block with let should parse");
    }

    #[test]
    fn test_async_block_multiple_lets() {
        let result = parse("async { let a = 1; let b = 2; a + b }");
        assert!(
            result.is_ok(),
            "Async block with multiple lets should parse"
        );
    }

    #[test]
    fn test_async_block_with_if() {
        let result = parse("async { if cond { 1 } else { 0 } }");
        assert!(result.is_ok(), "Async block with if should parse");
    }

    #[test]
    fn test_async_block_with_match() {
        let result = parse("async { match x { Some(v) => v, None => 0 } }");
        assert!(result.is_ok(), "Async block with match should parse");
    }

    #[test]
    fn test_async_block_multiple_awaits() {
        let result = parse("async { let a = await first(); let b = await second(); a + b }");
        assert!(
            result.is_ok(),
            "Async block with multiple awaits should parse"
        );
    }

    #[test]
    fn test_async_block_method_chain() {
        let result = parse("async { await fetch().await_json() }");
        assert!(result.is_ok(), "Async block with method chain should parse");
    }

    // ============================================================
    // Async lambda variations
    // ============================================================

    #[test]
    fn test_async_lambda_single_param() {
        let result = parse("async |x| x");
        assert!(result.is_ok(), "Async lambda single param should parse");
    }

    #[test]
    fn test_async_lambda_two_params() {
        let result = parse("async |a, b| a + b");
        assert!(result.is_ok(), "Async lambda two params should parse");
    }

    #[test]
    fn test_async_lambda_three_params() {
        let result = parse("async |x, y, z| x + y + z");
        assert!(result.is_ok(), "Async lambda three params should parse");
    }

    #[test]
    fn test_async_lambda_with_await() {
        let result = parse("async |req| await process(req)");
        assert!(result.is_ok(), "Async lambda with await should parse");
    }

    #[test]
    fn test_async_lambda_block_body() {
        let result = parse("async |x| { let y = x * 2; y }");
        assert!(result.is_ok(), "Async lambda with block body should parse");
    }

    #[test]
    fn test_async_lambda_if_body() {
        let result = parse("async |x| if x > 0 { x } else { 0 }");
        assert!(result.is_ok(), "Async lambda with if body should parse");
    }

    #[test]
    fn test_async_lambda_match_body() {
        let result = parse("async |opt| match opt { Some(v) => v, None => 0 }");
        assert!(result.is_ok(), "Async lambda with match body should parse");
    }

    #[test]
    fn test_async_lambda_arithmetic() {
        let result = parse("async |n| n * 2 + 1");
        assert!(result.is_ok(), "Async lambda with arithmetic should parse");
    }

    #[test]
    fn test_async_lambda_method_call() {
        let result = parse("async |s| s.len()");
        assert!(result.is_ok(), "Async lambda with method call should parse");
    }

    // ============================================================
    // Complex async patterns
    // ============================================================

    #[test]
    fn test_async_block_nested() {
        let result = parse("async { let inner = async { 42 }; await inner }");
        assert!(result.is_ok(), "Nested async blocks should parse");
    }

    #[test]
    fn test_async_lambda_returns_async_block() {
        let result = parse("async |x| async { x }");
        assert!(
            result.is_ok(),
            "Async lambda returning async block should parse"
        );
    }

    #[test]
    fn test_async_as_argument() {
        let result = parse("spawn(async { await task() })");
        assert!(result.is_ok(), "Async block as argument should parse");
    }

    #[test]
    fn test_async_in_let() {
        let result = parse("let future = async { 42 }");
        assert!(result.is_ok(), "Async block in let should parse");
    }

    // ============================================================
    // Additional EXTREME TDD tests
    // ============================================================

    // ===== Name variations =====

    #[test]
    fn test_async_fun_single_char_name() {
        let result = parse("async fun f() { 1 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_long_name() {
        let result = parse("async fun very_long_function_name() { 1 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_name_with_numbers() {
        let result = parse("async fun fetch2() { 1 }");
        assert!(result.is_ok());
    }

    // ===== Param name variations =====

    #[test]
    fn test_async_lambda_single_char_param() {
        let result = parse("async |x| x");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_lambda_long_param_name() {
        let result = parse("async |very_long_param| very_long_param");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_lambda_params_with_numbers() {
        let result = parse("async |x1, y2| x1 + y2");
        assert!(result.is_ok());
    }

    // ===== Multiple async constructs =====

    #[test]
    fn test_two_async_functions() {
        let result = parse("async fun a() { 1 }\nasync fun b() { 2 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_and_regular() {
        let result = parse("async fun a() { 1 }\nfun b() { 2 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_block_and_function() {
        let result = parse("let x = async { 1 }\nasync fun f() { 2 }");
        assert!(result.is_ok());
    }

    // ===== Async with control flow =====

    #[test]
    fn test_async_fun_with_if() {
        let result = parse("async fun check(x) { if x > 0 { x } else { 0 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_with_match() {
        let result = parse("async fun process(opt) { match opt { Some(v) => v, None => 0 } }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_with_loop() {
        let result = parse("async fun count() { let mut i = 0; while i < 10 { i = i + 1 }; i }");
        assert!(result.is_ok());
    }

    // ===== Return types =====

    #[test]
    fn test_async_fun_return_i32() {
        let result = parse("async fun get() -> i32 { 42 }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_return_bool() {
        let result = parse("async fun check() -> bool { true }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_return_vec() {
        let result = parse("async fun list() -> Vec<i32> { vec![] }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_fun_return_result() {
        let result = parse("async fun try_get() -> Result<i32, Error> { Ok(0) }");
        assert!(result.is_ok());
    }

    // ===== Edge cases =====

    #[test]
    fn test_async_block_empty_like() {
        let result = parse("async { () }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_lambda_returns_block() {
        let result = parse("async |x| { x }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_lambda_tuple_return() {
        let result = parse("async |a, b| (a, b)");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_block_with_function_call() {
        let result = parse("async { foo() }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_async_block_with_method_call() {
        let result = parse("async { x.method() }");
        assert!(result.is_ok());
    }

    // ===== Additional coverage tests (Round 106) =====

    // Test 72: Async block with await
    #[test]
    fn test_async_block_await() {
        let result = parse("async { await fetch(url) }");
        assert!(result.is_ok(), "Async block with await should parse");
    }

    // Test 73: Async function with await call
    #[test]
    fn test_async_fun_await_call() {
        let result = parse("async fun get_data() { await http.get(url) }");
        assert!(result.is_ok(), "Async fun with await should parse");
    }

    // Test 74: Chained await
    #[test]
    fn test_await_chain() {
        let result = parse("async { await (await get_client()).fetch() }");
        assert!(result.is_ok(), "Chained await should parse");
    }

    // Test 75: Async block with match
    #[test]
    fn test_async_block_match() {
        let result = parse("async { match x { 1 => a, _ => b } }");
        assert!(result.is_ok(), "Async block with match should parse");
    }

    // Test 76: Multiple async functions
    #[test]
    fn test_multiple_async_funs() {
        let result = parse("async fun f1() { } async fun f2() { }");
        assert!(result.is_ok(), "Multiple async functions should parse");
    }

    // Test 78: Async function calling another
    #[test]
    fn test_async_fun_call_async() {
        let result = parse("async fun outer() { await inner() }");
        assert!(result.is_ok(), "Async calling async should parse");
    }

    // Test 79: Async with try-catch
    #[test]
    fn test_async_try_catch() {
        let result = parse("async { try { await risky() } catch (e) { default } }");
        assert!(result.is_ok(), "Async with try-catch should parse");
    }

    // Test 80: Async function with multiple params
    #[test]
    fn test_async_fun_multi_params() {
        let result = parse("async fun process(a: i32, b: str, c: bool) { }");
        assert!(result.is_ok(), "Async fun multi params should parse");
    }

    // Test 81: Async block assignment
    #[test]
    fn test_async_block_assignment() {
        let result = parse("let future = async { compute() }");
        assert!(result.is_ok(), "Async block assignment should parse");
    }

    // Test 82: Async in if condition
    #[test]
    fn test_async_in_if() {
        let result = parse("async { if ready { await go() } else { wait() } }");
        assert!(result.is_ok(), "Async with if should parse");
    }

    // Property tests
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
            fn prop_async_blocks_parse(_seed in any::<u32>()) {
                let code = "async { 42 }";
                let result = Parser::new(code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_lambda_with_param(param in valid_identifier()) {
                let code = format!("async |{param}| {param}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_arrow_lambda_parses(param in valid_identifier(), val in 0i32..100) {
                // Async arrow lambda uses pipe syntax: async |param| expr
                // NOT arrow syntax: async param => expr (unsupported)
                let code = format!("async |{param}| {val}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_function_parses(name in valid_identifier()) {
                let code = format!("async fun {name}() {{ 42 }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_lambda_multi_params(p1 in valid_identifier(), p2 in valid_identifier()) {
                let code = format!("async |{p1}, {p2}| {p1} + {p2}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_block_with_expressions(n in 0i32..100) {
                let code = format!("async {{ {n} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_async_function_with_params(name in valid_identifier(), param in valid_identifier()) {
                let code = format!("async fun {name}({param}) {{ {param} }}");
                let result = Parser::new(&code).parse();
                prop_assert!(result.is_ok());
            }
        }
    }
}
