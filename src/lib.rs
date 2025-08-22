//! Ruchy: A modern systems programming language
//!
//! Ruchy combines functional programming with systems programming capabilities,
//! featuring an ML-style syntax, advanced type inference, and zero-cost abstractions.

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod actors;
pub mod backend;
pub mod frontend;
pub mod lints;
pub mod lsp;
pub mod mcp;
pub mod middleend;
pub mod optimization;
pub mod parser;
pub mod proving;
pub mod quality;
pub mod runtime;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod transpiler;
pub mod wasm;

pub use actors::{
    Actor, ActorHandle, McpActor, McpMessage, McpResponse, SupervisionStrategy, Supervisor,
};
pub use backend::transpiler::Transpiler;
pub use frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, UnaryOp};
pub use frontend::lexer::{Token, TokenStream};
pub use frontend::parser::Parser;
pub use lsp::{start_server, start_tcp_server, Formatter, RuchyLanguageServer, SemanticAnalyzer};
pub use quality::{
    CiQualityEnforcer, CoverageCollector, CoverageReport, CoverageTool, FileCoverage,
    HtmlReportGenerator, QualityGates, QualityMetrics, QualityReport, QualityThresholds,
};
pub use quality::gates::{QualityGateEnforcer, QualityGateConfig, GateResult};

use anyhow::Result;

/// Compile Ruchy source code to Rust
///
/// # Examples
///
/// ```
/// use ruchy::compile;
///
/// let rust_code = compile("42").expect("Failed to compile");
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
    // Use transpile_to_program to wrap in main() for standalone compilation
    let rust_code = transpiler.transpile_to_program(&ast)?;
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
/// run_repl().expect("Failed to run REPL");
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
mod test_config {
    use std::sync::Once;

    static INIT: Once = Once::new();

    /// Initialize test configuration once per test run
    pub fn init() {
        INIT.call_once(|| {
            // Limit proptest for development (CI uses different settings)
            if std::env::var("CI").is_err() {
                std::env::set_var("PROPTEST_CASES", "10");
                std::env::set_var("PROPTEST_MAX_SHRINK_ITERS", "50");
            }
            // Limit test threads if not already set
            if std::env::var("RUST_TEST_THREADS").is_err() {
                std::env::set_var("RUST_TEST_THREADS", "4");
            }
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::single_char_pattern)]
mod tests {
    use super::test_config;
    use super::*;

    #[test]
    fn test_compile_simple() {
        test_config::init();
        let result = compile("42").unwrap();
        assert!(result.contains("42"));
    }

    #[test]
    fn test_compile_let() {
        let result = compile("let x = 10 in x + 1").unwrap();
        assert!(result.contains("let"));
        assert!(result.contains("10"));
    }

    #[test]
    fn test_compile_function() {
        let result = compile("fun add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        assert!(result.contains("fn"));
        assert!(result.contains("add"));
        assert!(result.contains("i32"));
    }

    #[test]
    fn test_compile_if() {
        let result = compile("if true { 1 } else { 0 }").unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("else"));
    }

    #[test]
    fn test_compile_match() {
        let result = compile("match x { 0 => \"zero\", _ => \"other\" }").unwrap();
        assert!(result.contains("match"));
    }

    #[test]
    fn test_compile_list() {
        let result = compile("[1, 2, 3]").unwrap();
        assert!(result.contains("vec") && result.contains("!"));
    }

    #[test]
    fn test_compile_lambda() {
        let result = compile("|x| x * 2").unwrap();
        assert!(result.contains("|"));
    }

    #[test]
    fn test_compile_struct() {
        let result = compile("struct Point { x: f64, y: f64 }").unwrap();
        assert!(result.contains("struct"));
        assert!(result.contains("Point"));
    }

    #[test]
    fn test_compile_impl() {
        let result =
            compile("impl Point { fun new() -> Point { Point { x: 0.0, y: 0.0 } } }").unwrap();
        assert!(result.contains("impl"));
    }

    #[test]
    fn test_compile_trait() {
        let result = compile("trait Show { fun show(&self) -> String }").unwrap();
        assert!(result.contains("trait"));
    }

    #[test]
    fn test_compile_for_loop() {
        let result = compile("for x in [1, 2, 3] { print(x) }").unwrap();
        assert!(result.contains("for"));
    }

    #[test]
    fn test_compile_binary_ops() {
        let result = compile("1 + 2 * 3 - 4 / 2").unwrap();
        assert!(result.contains("+"));
        assert!(result.contains("*"));
        assert!(result.contains("-"));
        assert!(result.contains("/"));
    }

    #[test]
    fn test_compile_comparison_ops() {
        let result = compile("x < y && y <= z").unwrap();
        assert!(result.contains("<"));
        assert!(result.contains("<="));
        assert!(result.contains("&&"));
    }

    #[test]
    fn test_compile_unary_ops() {
        let result = compile("-x").unwrap();
        assert!(result.contains("-"));

        let result = compile("!flag").unwrap();
        assert!(result.contains("!"));
    }

    #[test]
    fn test_compile_call() {
        let result = compile("func(1, 2, 3)").unwrap();
        assert!(result.contains("func"));
        assert!(result.contains("("));
        assert!(result.contains(")"));
    }

    #[test]
    fn test_compile_method_call() {
        let result = compile("obj.method()").unwrap();
        assert!(result.contains("."));
        assert!(result.contains("method"));
    }

    #[test]
    fn test_compile_block() {
        let result = compile("{ let x = 1; x + 1 }").unwrap();
        assert!(result.contains("{"));
        assert!(result.contains("}"));
    }

    #[test]
    fn test_compile_string() {
        let result = compile("\"hello world\"").unwrap();
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_compile_bool() {
        let result = compile("true && false").unwrap();
        assert!(result.contains("true"));
        assert!(result.contains("false"));
    }

    #[test]
    fn test_compile_unit() {
        let result = compile("()").unwrap();
        assert!(result.contains("()"));
    }

    #[test]
    fn test_compile_nested_let() {
        let result = compile("let x = 1 in let y = 2 in x + y").unwrap();
        assert!(result.contains("let"));
    }

    #[test]
    fn test_compile_nested_if() {
        let result = compile("if x { if y { 1 } else { 2 } } else { 3 }").unwrap();
        assert!(result.contains("if"));
    }

    #[test]
    fn test_compile_empty_list() {
        let result = compile("[]").unwrap();
        assert!(result.contains("vec") && result.contains("!"));
    }

    #[test]
    fn test_compile_empty_block() {
        let result = compile("{ }").unwrap();
        assert!(result.contains("()"));
    }

    #[test]
    fn test_compile_float() {
        let result = compile("3.14159").unwrap();
        assert!(result.contains("3.14159"));
    }

    #[test]
    fn test_compile_large_int() {
        let result = compile("999999999").unwrap();
        assert!(result.contains("999999999"));
    }

    #[test]
    fn test_compile_string_escape() {
        let result = compile(r#""hello\nworld""#).unwrap();
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_compile_power_op() {
        let result = compile("2 ** 8").unwrap();
        assert!(result.contains("pow"));
    }

    #[test]
    fn test_compile_modulo() {
        let result = compile("10 % 3").unwrap();
        assert!(result.contains("%"));
    }

    #[test]
    fn test_compile_bitwise_ops() {
        let result = compile("a & b | c ^ d").unwrap();
        assert!(result.contains("&"));
        assert!(result.contains("|"));
        assert!(result.contains("^"));
    }

    #[test]
    fn test_compile_left_shift() {
        let result = compile("x << 2").unwrap();
        assert!(result.contains("<<"));
    }

    #[test]
    fn test_compile_not_equal() {
        let result = compile("x != y").unwrap();
        assert!(result.contains("!="));
    }

    #[test]
    fn test_compile_greater_ops() {
        let result = compile("x > y && x >= z").unwrap();
        assert!(result.contains(">"));
        assert!(result.contains(">="));
    }

    #[test]
    fn test_compile_or_op() {
        let result = compile("x || y").unwrap();
        assert!(result.contains("||"));
    }

    #[test]
    fn test_compile_complex_expression() {
        let result = compile("(x + y) * (z - w) / 2").unwrap();
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
        let error = get_parse_error("fun (");
        assert!(error.is_some());
        assert!(error.unwrap().contains("Expected"));
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
        let result = compile("fun id<T>(x: T) -> T { x }").unwrap();
        assert!(result.contains("fn"));
        assert!(result.contains("id"));
    }

    #[test]
    fn test_compile_generic_struct() {
        let result = compile("struct Box<T> { value: T }").unwrap();
        assert!(result.contains("struct"));
        assert!(result.contains("Box"));
    }

    #[test]
    fn test_compile_multiple_statements() {
        let result = compile("let x = 1 in let y = 2 in x + y").unwrap();
        assert!(result.contains("let"));
    }

    #[test]
    fn test_compile_pattern_matching() {
        let result = compile("match x { 0 => \"zero\", _ => \"other\" }").unwrap();
        assert!(result.contains("match"));
    }

    #[test]
    fn test_compile_struct_literal() {
        let result = compile("Point { x: 10, y: 20 }").unwrap();
        assert!(result.contains("Point"));
    }

    #[test]
    fn test_compile_try_operator() {
        let result = compile("func()?").unwrap();
        assert!(result.contains("?"));
    }

    #[test]
    fn test_compile_await_expression() {
        let result = compile("async_func().await").unwrap();
        assert!(result.contains("await"));
    }

    #[test]
    fn test_compile_import() {
        let result = compile("import std.collections.HashMap").unwrap();
        assert!(result.contains("use"));
    }

    #[test]
    fn test_compile_while_loop() {
        let result = compile("while x < 10 { x + 1 }").unwrap();
        assert!(result.contains("while"));
    }

    #[test]
    fn test_compile_range() {
        let result = compile("1..10").unwrap();
        assert!(result.contains(".."));
    }

    #[test]
    fn test_compile_pipeline() {
        let result = compile("data >> filter >> map").unwrap();
        assert!(result.contains("("));
    }

    #[test]
    fn test_compile_send_operation() {
        let result = compile("myactor <- message").unwrap();
        assert!(result.contains(". send (")); // Formatted with spaces
        assert!(result.contains(". await")); // Formatted with spaces
    }

    #[test]
    fn test_compile_ask_operation() {
        let result = compile("myactor ? request").unwrap();
        assert!(result.contains(". ask (")); // Formatted with spaces
        assert!(result.contains(". await")); // Formatted with spaces
    }

    #[test]
    fn test_compile_list_comprehension() {
        let result = compile("[x * 2 for x in range(10)]").unwrap();
        assert!(result.contains("map"));
    }

    #[test]
    fn test_compile_actor() {
        let result = compile(
            r"
            actor Counter {
                count: i32,
                
                receive {
                    Inc => 1,
                    Get => 0
                }
            }
        ",
        )
        .unwrap();
        assert!(result.contains("struct Counter"));
        assert!(result.contains("enum CounterMessage"));
    }
}
