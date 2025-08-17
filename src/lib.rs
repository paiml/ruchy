//! Ruchy: A modern systems programming language
//!
//! Ruchy combines functional programming with systems programming capabilities,
//! featuring an ML-style syntax, advanced type inference, and zero-cost abstractions.

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod backend;
pub mod frontend;
pub mod mcp;
pub mod middleend;
pub mod parser;
pub mod runtime;
#[cfg(test)]
pub mod testing;
pub mod transpiler;

pub use backend::transpiler::Transpiler;
pub use frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, UnaryOp};
pub use frontend::lexer::{Token, TokenStream};
pub use frontend::parser::Parser;

use anyhow::Result;

/// Compile Ruchy source code to Rust
///
/// # Examples
///
/// ```
/// use ruchy::compile;
///
/// let rust_code = compile("42").expect("verified by caller");
/// assert!(rust_code.contains("42"));
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The source code cannot be parsed
/// - The transpilation to Rust fails
pub fn compile(source: &str) -> Result<String> {
    let mut parser = Parser::new(source);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    Ok(rust_code.to_string())
}

/// Check if the given source code has valid syntax
#[must_use]
pub fn is_valid_syntax(source: &str) -> bool {
    let mut parser = Parser::new(source);
    parser.parse().is_ok()
}

/// Get parse error details if the source has syntax errors
#[must_use]
pub fn get_parse_error(source: &str) -> Option<String> {
    let mut parser = Parser::new(source);
    parser.parse().err().map(|e| e.to_string())
}

/// Run the REPL
///
/// # Examples
///
/// ```no_run
/// use ruchy::run_repl;
///
/// run_repl().expect("verified by caller");
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The REPL cannot be initialized
/// - User interaction fails
pub fn run_repl() -> Result<()> {
    let mut repl = runtime::repl::Repl::new()?;
    repl.run()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::single_char_pattern)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let result = compile("42").expect("verified by caller");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_compile_let() {
        let result = compile("let x = 10 in x + 1").expect("verified by caller");
        assert!(result.contains("let"));
        assert!(result.contains("10"));
    }

    #[test]
    fn test_compile_function() {
        let result =
            compile("fun add(x: i32, y: i32) -> i32 { x + y }").expect("verified by caller");
        assert!(result.contains("fn"));
        assert!(result.contains("add"));
        assert!(result.contains("i32"));
    }

    #[test]
    fn test_compile_if() {
        let result = compile("if true { 1 } else { 0 }").expect("verified by caller");
        assert!(result.contains("if"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_compile_match() {
        let result =
            compile("match x { 0 => \"zero\", _ => \"other\" }").expect("verified by caller");
        assert!(result.contains("match"));
    }

    #[test]
    fn test_compile_list() {
        let result = compile("[1, 2, 3]").expect("verified by caller");
        assert!(result.contains("vec!"));
    }

    #[test]
    fn test_compile_lambda() {
        let result = compile("fun (x) { x * 2 }").expect("verified by caller");
        assert!(result.contains("|"));
    }

    #[test]
    fn test_compile_struct() {
        let result = compile("struct Point { x: f64, y: f64 }").expect("verified by caller");
        assert!(result.contains("struct"));
        assert!(result.contains("Point"));
    }

    #[test]
    fn test_compile_impl() {
        let result = compile("impl Point { fun new() -> Point { Point { x: 0.0, y: 0.0 } } }")
            .expect("verified by caller");
        assert!(result.contains("impl"));
    }

    #[test]
    fn test_compile_trait() {
        let result =
            compile("trait Show { fun show(&self) -> String }").expect("verified by caller");
        assert!(result.contains("trait"));
    }

    #[test]
    fn test_compile_for_loop() {
        let result = compile("for x in [1, 2, 3] { print(x) }").expect("verified by caller");
        assert!(result.contains("for"));
    }

    #[test]
    fn test_compile_binary_ops() {
        let result = compile("1 + 2 * 3 - 4 / 2").expect("verified by caller");
        assert!(result.contains("+"));
        assert!(result.contains("*"));
        assert!(result.contains("-"));
        assert!(result.contains("/"));
    }

    #[test]
    fn test_compile_comparison_ops() {
        let result = compile("x < y && y <= z").expect("verified by caller");
        assert!(result.contains("<"));
        assert!(result.contains("<="));
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_compile_unary_ops() {
        let result = compile("-x").expect("verified by caller");
        assert!(result.contains("-"));

        let result = compile("!flag").expect("verified by caller");
        assert!(result.contains("!"));
    }

    #[test]
    fn test_compile_call() {
        let result = compile("func(1, 2, 3)").expect("verified by caller");
        assert!(result.contains("func"));
        assert!(result.contains("("));
        assert!(result.contains(")"));
    }

    #[test]
    fn test_compile_method_call() {
        let result = compile("obj.method()").expect("verified by caller");
        assert!(result.contains("."));
        assert!(result.contains("method"));
    }

    #[test]
    fn test_compile_block() {
        let result = compile("{ let x = 1; x + 1 }").expect("verified by caller");
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_compile_string() {
        let result = compile("\"hello world\"").expect("verified by caller");
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_compile_bool() {
        let result = compile("true && false").expect("verified by caller");
        assert!(result.contains("true"));
        assert!(result.contains("false"));
    }

    #[test]
    fn test_compile_unit() {
        let result = compile("()").expect("verified by caller");
        assert!(result.contains("()"));
    }

    #[test]
    fn test_compile_nested_let() {
        let result = compile("let x = 1 in let y = 2 in x + y").expect("verified by caller");
        assert!(result.contains("let"));
    }

    #[test]
    fn test_compile_nested_if() {
        let result =
            compile("if x { if y { 1 } else { 2 } } else { 3 }").expect("verified by caller");
        assert!(result.contains("if"));
    }

    #[test]
    fn test_compile_empty_list() {
        let result = compile("[]").expect("verified by caller");
        assert!(result.contains("vec!"));
    }

    #[test]
    fn test_compile_empty_block() {
        let result = compile("{ }").expect("verified by caller");
        assert!(result.contains("()"));
    }

    #[test]
    fn test_compile_float() {
        let result = compile("3.14159").expect("verified by caller");
        assert!(result.contains("3.14159"));
    }

    #[test]
    fn test_compile_large_int() {
        let result = compile("999999999").expect("verified by caller");
        assert!(result.contains("999999999"));
    }

    #[test]
    fn test_compile_string_escape() {
        let result = compile(r#""hello\nworld""#).expect("verified by caller");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_compile_power_op() {
        let result = compile("2 ** 8").expect("verified by caller");
        assert!(result.contains("pow"));
    }

    #[test]
    fn test_compile_modulo() {
        let result = compile("10 % 3").expect("verified by caller");
        assert!(result.contains("%"));
    }

    #[test]
    fn test_compile_bitwise_ops() {
        let result = compile("a & b | c ^ d").expect("verified by caller");
        assert!(result.contains("&"));
        assert!(result.contains("|"));
        assert!(result.contains("^"));
    }

    #[test]
    fn test_compile_shift_ops() {
        let result = compile("x << 2 >> 1").expect("verified by caller");
        assert!(result.contains("<<"));
        assert!(result.contains(">>"));
    }

    #[test]
    fn test_compile_not_equal() {
        let result = compile("x != y").expect("verified by caller");
        assert!(result.contains("!="));
    }

    #[test]
    fn test_compile_greater_ops() {
        let result = compile("x > y && x >= z").expect("verified by caller");
        assert!(result.contains(">"));
        assert!(result.contains(">="));
    }

    #[test]
    fn test_compile_or_op() {
        let result = compile("x || y").expect("verified by caller");
        assert!(result.contains("||"));
    }

    #[test]
    fn test_compile_complex_expression() {
        let result = compile("(x + y) * (z - w) / 2").expect("verified by caller");
        assert!(result.contains("+"));
        assert!(result.contains("-"));
        assert!(result.contains("*"));
        assert!(result.contains("/"));
    }

    #[test]
    fn test_compile_errors() {
        assert!(compile("").is_err());
        assert!(compile("   ").is_err());
        assert!(compile("let x =").is_err());
        assert!(compile("if").is_err());
        assert!(compile("match").is_err());
    }

    #[test]
    fn test_is_valid_syntax_valid_cases() {
        assert!(is_valid_syntax("42"));
        assert!(is_valid_syntax("3.14"));
        assert!(is_valid_syntax("true"));
        assert!(is_valid_syntax("false"));
        assert!(is_valid_syntax("\"hello\""));
        assert!(is_valid_syntax("x + y"));
        assert!(is_valid_syntax("[1, 2, 3]"));
        assert!(is_valid_syntax("if true { 1 } else { 2 }"));
    }

    #[test]
    fn test_is_valid_syntax_invalid_cases() {
        assert!(!is_valid_syntax(""));
        assert!(!is_valid_syntax("   "));
        assert!(!is_valid_syntax("let x ="));
        assert!(!is_valid_syntax("if { }"));
        assert!(!is_valid_syntax("[1, 2,"));
        assert!(!is_valid_syntax("match"));
        assert!(!is_valid_syntax("struct"));
    }

    #[test]
    fn test_get_parse_error_with_errors() {
        let error = get_parse_error("let x =");
        assert!(error.is_some());
        assert!(error.expect("verified by caller").contains("Expected"));
    }

    #[test]
    fn test_get_parse_error_without_errors() {
        let error = get_parse_error("42");
        assert!(error.is_none());
    }

    #[test]
    fn test_get_parse_error_detailed() {
        let error = get_parse_error("if");
        assert!(error.is_some());

        let error = get_parse_error("match");
        assert!(error.is_some());

        let error = get_parse_error("[1, 2,");
        assert!(error.is_some());
    }

    #[test]
    fn test_compile_generic_function() {
        let result = compile("fun id<T>(x: T) -> T { x }").expect("verified by caller");
        assert!(result.contains("fn"));
        assert!(result.contains("id"));
    }

    #[test]
    fn test_compile_generic_struct() {
        let result = compile("struct Box<T> { value: T }").expect("verified by caller");
        assert!(result.contains("struct"));
        assert!(result.contains("Box"));
    }

    #[test]
    fn test_compile_multiple_statements() {
        let result = compile("let x = 1; let y = 2; x + y").expect("verified by caller");
        assert!(result.contains("let"));
    }

    #[test]
    fn test_compile_pattern_matching() {
        let result = compile("match x { [] => 0, [h, ...t] => 1 }").expect("verified by caller");
        assert!(result.contains("match"));
    }

    #[test]
    fn test_compile_struct_literal() {
        let result = compile("Point { x: 10, y: 20 }").expect("verified by caller");
        assert!(result.contains("Point"));
    }

    #[test]
    fn test_compile_try_operator() {
        let result = compile("func()?").expect("verified by caller");
        assert!(result.contains("?"));
    }

    #[test]
    fn test_compile_await_expression() {
        let result = compile("async_func().await").expect("verified by caller");
        assert!(result.contains("await"));
    }

    #[test]
    fn test_compile_import() {
        let result = compile("import std.collections.HashMap").expect("verified by caller");
        assert!(result.contains("use"));
    }

    #[test]
    fn test_compile_while_loop() {
        let result = compile("while x < 10 { x = x + 1 }").expect("verified by caller");
        assert!(result.contains("while"));
    }

    #[test]
    fn test_compile_range() {
        let result = compile("1..10").expect("verified by caller");
        assert!(result.contains(".."));
    }

    #[test]
    fn test_compile_pipeline() {
        let result = compile("data |> filter |> map").expect("verified by caller");
        assert!(result.contains("("));
    }

    #[test]
    fn test_compile_send_operation() {
        let result = compile("actor ! message").expect("verified by caller");
        assert!(result.contains("send"));
    }

    #[test]
    fn test_compile_ask_operation() {
        let result = compile("actor ? request").expect("verified by caller");
        assert!(result.contains("ask"));
    }

    #[test]
    fn test_compile_list_comprehension() {
        let result = compile("[x * 2 for x in range(10)]").expect("verified by caller");
        assert!(result.contains("map"));
    }

    #[test]
    fn test_compile_actor() {
        let result = compile(
            r"
            actor Counter {
                state { count: i32 }
                receive {
                    Inc => count + 1,
                    Get => count
                }
            }
        ",
        )
        .expect("verified by caller");
        assert!(result.contains("actor"));
    }
}
