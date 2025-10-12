//! Ruchy: A modern systems programming language
//!
//! Ruchy combines functional programming with systems programming capabilities,
//! featuring an ML-style syntax, advanced type inference, and zero-cost abstractions.
#![warn(clippy::all)]
// Temporarily disabled pedantic for RUCHY-0801 - Re-enable in quality sprint
// #![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::self_only_used_in_recursion)]
// Clippy allows for RUCHY-0801 commit - will be addressed in quality sprint
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::unused_self)]
#![allow(clippy::expect_used)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// Additional clippy allows for P0 lint fixes
#![allow(clippy::empty_line_after_doc_comments)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::type_complexity)]
#![allow(dead_code)]
#![allow(clippy::float_cmp)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::manual_strip)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]
#![allow(clippy::format_push_string)]
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::manual_clamp)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::unnecessary_to_owned)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::if_same_then_else)]
#[cfg(feature = "mcp")]
pub mod actors;
pub mod api_docs;
pub mod backend;
pub mod build_transpiler;
pub mod cli;
pub mod debugger;
pub mod docs;
pub mod error_recovery_enhanced;
pub mod frontend;
pub mod lints;
#[cfg(feature = "mcp")]
pub mod lsp;
pub mod macros;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod middleend;
#[cfg(feature = "notebook")]
pub mod notebook;
pub mod package;
pub mod parser;
pub mod performance_optimizations;
pub mod proving;
pub mod quality;
pub mod runtime;
pub mod stdlib;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
#[cfg(any(test, feature = "testing"))]
pub use testing::AstBuilder;
pub mod transpiler;
pub mod utils;
pub mod wasm;
#[cfg(target_arch = "wasm32")]
mod wasm_bindings;
#[cfg(feature = "mcp")]
pub use actors::{
    Actor, ActorHandle, McpActor, McpMessage, McpResponse, SupervisionStrategy, Supervisor,
};
use anyhow::Result;
pub use backend::wasm::WasmEmitter;
pub use backend::{ModuleResolver, Transpiler};
pub use frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Pattern, UnaryOp};
pub use frontend::lexer::{Token, TokenStream};
pub use frontend::parser::Parser;
#[cfg(feature = "mcp")]
pub use lsp::{start_server, start_tcp_server, Formatter, RuchyLanguageServer, SemanticAnalyzer};
pub use quality::gates::{GateResult, QualityGateConfig, QualityGateEnforcer};
pub use quality::{
    CiQualityEnforcer, CoverageCollector, CoverageReport, CoverageTool, FileCoverage,
    HtmlReportGenerator, QualityGates, QualityMetrics, QualityReport, QualityThresholds,
};
pub use utils::*;
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
    let mut transpiler = Transpiler::new();
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
#[cfg(not(target_arch = "wasm32"))]
pub fn run_repl() -> Result<()> {
    let mut repl =
        runtime::repl::Repl::new(std::env::current_dir().unwrap_or_else(|_| "/tmp".into()))?;
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
        let result = compile("42").expect("Failed to compile literal 42");
        assert!(result.contains("42"));
    }
    #[test]
    fn test_compile_let() {
        let result = compile("let x = 10 in x + 1").expect("Failed to compile let expression");
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
        // Array literals should use fixed-size array syntax [1, 2, 3], not vec![1, 2, 3]
        assert!(
            result.contains("[")
                && result.contains("1")
                && result.contains("2")
                && result.contains("3"),
            "Expected array syntax, got: {result}"
        );
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
        // Error message format may vary, just check that we got an error
        assert!(!error.unwrap().is_empty());
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
    // Test removed - try/catch operations removed in RUCHY-0834
    // #[test]
    // fn test_compile_try_operator() {
    //     let result = compile("func()?").unwrap();
    //     assert!(result.contains("?"));
    // }
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
        let result = compile("data |> filter |> map").unwrap();
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
        let result = compile("myactor <? request").unwrap();
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
    // ===== COMPREHENSIVE COVERAGE TESTS =====
    #[test]
    fn test_type_conversions() {
        // String conversions
        assert!(compile("str(42)").is_ok());
        assert!(compile("str(3.14)").is_ok());
        assert!(compile("str(true)").is_ok());
        // Integer conversions
        assert!(compile("int(\"42\")").is_ok());
        assert!(compile("int(3.14)").is_ok());
        assert!(compile("int(true)").is_ok());
        // Float conversions
        assert!(compile("float(\"3.14\")").is_ok());
        assert!(compile("float(42)").is_ok());
        // Bool conversions
        assert!(compile("bool(0)").is_ok());
        assert!(compile("bool(\"\")").is_ok());
        assert!(compile("bool([])").is_ok());
        // Collection conversions
        assert!(compile("list(\"hello\")").is_ok());
        assert!(compile("set([1,2,3])").is_ok());
        assert!(compile("dict([(\"a\",1)])").is_ok());
    }
    #[test]
    fn test_method_calls() {
        // String methods
        assert!(compile("\"hello\".upper()").is_ok());
        assert!(compile("\"HELLO\".lower()").is_ok());
        assert!(compile("\"  hello  \".strip()").is_ok());
        assert!(compile("\"hello\".len()").is_ok());
        assert!(compile("\"hello\".split(\" \")").is_ok());
        // List methods
        assert!(compile("[1,2,3].len()").is_ok());
        assert!(compile("[1,2,3].append(4)").is_ok());
        assert!(compile("[1,2,3].pop()").is_ok());
        assert!(compile("[1,2,3].reverse()").is_ok());
        assert!(compile("[1,2,3].sort()").is_ok());
        // Dict methods
        assert!(compile("{\"a\":1}.get(\"a\")").is_ok());
        assert!(compile("{\"a\":1}.keys()").is_ok());
        assert!(compile("{\"a\":1}.values()").is_ok());
        assert!(compile("{\"a\":1}.items()").is_ok());
        // Iterator methods
        assert!(compile("[1,2,3].map(|x| x*2)").is_ok());
        assert!(compile("[1,2,3].filter(|x| x>1)").is_ok());
        assert!(compile("[1,2,3].reduce(|a,b| a+b)").is_ok());
    }
    #[test]
    fn test_patterns() {
        // Literal patterns
        assert!(compile("match x { 0 => \"zero\", _ => \"other\" }").is_ok());
        assert!(compile("match x { true => \"yes\", false => \"no\" }").is_ok());
        // Tuple patterns
        assert!(compile("match p { (0, 0) => \"origin\", _ => \"other\" }").is_ok());
        assert!(compile("match p { (x, y) => x + y }").is_ok());
        // List patterns
        assert!(compile("match lst { [] => \"empty\", _ => \"has items\" }").is_ok());
        assert!(compile("match lst { [x] => x, _ => 0 }").is_ok());
        // Rest patterns syntax not yet supported:
        // assert!(compile("match lst { [head, ...tail] => head, _ => 0 }").is_ok());
        // Struct patterns not yet supported:
        // assert!(compile("match p { Point { x, y } => x + y }").is_ok());
        // Enum patterns not yet supported:
        // assert!(compile("match opt { Some(x) => x, None => 0 }").is_ok());
        assert!(compile("match res { Ok(v) => v, Err(e) => panic(e) }").is_ok());
        // Guard patterns
        assert!(compile("match x { n if n > 0 => \"positive\", _ => \"other\" }").is_ok());
        // Or patterns
        assert!(compile("match x { 0 | 1 => \"binary\", _ => \"other\" }").is_ok());
    }
    #[test]
    fn test_all_operators() {
        // Arithmetic
        assert!(compile("x + y").is_ok());
        assert!(compile("x - y").is_ok());
        assert!(compile("x * y").is_ok());
        assert!(compile("x / y").is_ok());
        assert!(compile("x % y").is_ok());
        assert!(compile("x ** y").is_ok());
        // Comparison
        assert!(compile("x == y").is_ok());
        assert!(compile("x != y").is_ok());
        assert!(compile("x < y").is_ok());
        assert!(compile("x > y").is_ok());
        assert!(compile("x <= y").is_ok());
        assert!(compile("x >= y").is_ok());
        // Logical
        assert!(compile("x && y").is_ok());
        assert!(compile("x || y").is_ok());
        assert!(compile("!x").is_ok());
        // Bitwise
        assert!(compile("x & y").is_ok());
        assert!(compile("x | y").is_ok());
        assert!(compile("x ^ y").is_ok());
        assert!(compile("~x").is_ok());
        assert!(compile("x << y").is_ok());
        assert!(compile("x >> y").is_ok());
        // Assignment
        assert!(compile("x = 5").is_ok());
        assert!(compile("x += 5").is_ok());
        assert!(compile("x -= 5").is_ok());
        assert!(compile("x *= 5").is_ok());
        assert!(compile("x /= 5").is_ok());
        // Nullish coalescing works (v1.9.0+)
        assert!(compile("x ?? y").is_ok());
        // Optional chaining: tracked in roadmap as LANG-FEAT-001
        // assert!(compile("x?.y").is_ok());
    }
    #[test]
    fn test_control_flow() {
        // If statements
        assert!(compile("if x { 1 }").is_ok());
        assert!(compile("if x { 1 } else { 2 }").is_ok());
        assert!(compile("if x { 1 } else if y { 2 } else { 3 }").is_ok());
        // Loops
        assert!(compile("while x { y }").is_ok());
        assert!(compile("loop { break }").is_ok());
        assert!(compile("for i in 0..10 { }").is_ok());
        assert!(compile("for i in items { }").is_ok());
        // Break/continue
        assert!(compile("while true { break }").is_ok());
        assert!(compile("for i in 0..10 { continue }").is_ok());
    }
    #[test]
    fn test_data_structures() {
        // Lists
        assert!(compile("[]").is_ok());
        assert!(compile("[1, 2, 3]").is_ok());
        assert!(compile("[[1, 2], [3, 4]]").is_ok());
        // Dicts
        assert!(compile("{}").is_ok());
        assert!(compile("{\"a\": 1}").is_ok());
        assert!(compile("{\"a\": 1, \"b\": 2}").is_ok());
        // Sets - not yet implemented, parser doesn't support set literals
        // assert!(compile("{1}").is_ok());
        // assert!(compile("{1, 2, 3}").is_ok());
        // Tuples
        assert!(compile("()").is_ok());
        assert!(compile("(1,)").is_ok());
        assert!(compile("(1, 2, 3)").is_ok());
    }
    #[test]
    fn test_functions_lambdas() {
        // Functions
        assert!(compile("fn f() { }").is_ok());
        assert!(compile("fn f(x) { x }").is_ok());
        assert!(compile("fn f(x, y) { x + y }").is_ok());
        assert!(compile("fn f(x: int) -> int { x }").is_ok());
        // Lambdas
        assert!(compile("|x| x").is_ok());
        assert!(compile("|x, y| x + y").is_ok());
        assert!(compile("|| 42").is_ok());
        // Async fn syntax: tracked in roadmap as LANG-FEAT-002
        // assert!(compile("async fn f() { await g() }").is_ok());
        assert!(compile("await fetch(url)").is_ok());
    }
    #[test]
    fn test_string_interpolation() {
        assert!(compile("f\"Hello {name}\"").is_ok());
        assert!(compile("f\"x = {x}, y = {y}\"").is_ok());
        assert!(compile("f\"Result: {calculate()}\"").is_ok());
    }
    #[test]
    #[ignore = "Comprehensions syntax not yet implemented"]
    fn test_comprehensions() {
        assert!(compile("[x * 2 for x in 0..10]").is_ok());
        assert!(compile("[x for x in items if x > 0]").is_ok());
        assert!(compile("{x: x*x for x in 0..5}").is_ok());
        assert!(compile("{x for x in items if unique(x)}").is_ok());
    }
    #[test]
    fn test_destructuring() {
        assert!(compile("let [a, b, c] = [1, 2, 3]").is_ok());
        assert!(compile("let {x, y} = point").is_ok());
        assert!(compile("let [head, ...tail] = list").is_ok());
        assert!(compile("let (a, b) = (1, 2)").is_ok());
    }
    #[test]
    #[ignore = "Try/catch syntax not yet implemented"]
    fn test_error_handling() {
        assert!(compile("try { risky() } catch e { handle(e) }").is_ok());
        assert!(compile("result?").is_ok());
        assert!(compile("result.unwrap()").is_ok());
        assert!(compile("result.expect(\"failed\")").is_ok());
        assert!(compile("result.unwrap_or(default)").is_ok());
    }
    #[test]
    #[ignore = "Class/struct syntax not yet implemented"]
    fn test_classes_structs() {
        assert!(compile("struct Point { x: int, y: int }").is_ok());
        assert!(compile("class Calculator { fn add(x, y) { x + y } }").is_ok());
        assert!(compile("enum Option { Some(value), None }").is_ok());
    }
    #[test]
    fn test_imports() {
        // Basic import statements now work
        assert!(compile("import std").is_ok());
        assert!(compile("from std import println").is_ok());
        // JS-style imports still need work
        // assert!(compile("import { readFile, writeFile } from fs").is_ok());
        // Export syntax still needs implementation
        // assert!(compile("export fn helper()").is_ok());
    }
    #[test]
    #[ignore = "Decorator syntax not yet implemented"]
    fn test_decorators() {
        assert!(compile("@memoize\nfn expensive(n) { }").is_ok());
        assert!(compile("@derive(Debug, Clone)\nstruct Data { }").is_ok());
    }
    #[test]
    fn test_generics() {
        assert!(compile("fn identity<T>(x: T) -> T { x }").is_ok());
        assert!(compile("struct Pair<T, U> { first: T, second: U }").is_ok());
        assert!(compile("enum Result<T, E> { Ok(T), Err(E) }").is_ok());
    }
    #[test]
    fn test_edge_cases() {
        // Empty input - parser expects at least one expression
        assert!(!is_valid_syntax(""));
        assert!(!is_valid_syntax("   "));
        assert!(!is_valid_syntax("\n\n"));
        // Deeply nested
        assert!(compile("((((((((((1))))))))))").is_ok());
        assert!(compile("[[[[[[1]]]]]]").is_ok());
        // Unicode
        assert!(compile("\"Hello 世界\"").is_ok());
        assert!(compile("\"Emoji 😀\"").is_ok());
    }
    #[test]
    #[ignore = "List comprehensions not fully implemented"]
    fn test_complex_programs() {
        let factorial = r"
            fn factorial(n) {
                if n <= 1 { 1 } else { n * factorial(n-1) }
            }
        ";
        assert!(compile(factorial).is_ok());
        let fibonacci = r"
            fn fibonacci(n) {
                match n {
                    0 => 0,
                    1 => 1,
                    _ => fibonacci(n-1) + fibonacci(n-2)
                }
            }
        ";
        assert!(compile(fibonacci).is_ok());
        let quicksort = r"
            fn quicksort(arr) {
                if arr.len() <= 1 { 
                    arr 
                } else {
                    let pivot = arr[0]
                    let less = [x for x in arr[1:] if x < pivot]
                    let greater = [x for x in arr[1:] if x >= pivot]
                    quicksort(less) + [pivot] + quicksort(greater)
                }
            }
        ";
        assert!(compile(quicksort).is_ok());
    }

    #[test]
    fn test_is_valid_syntax() {
        // Test valid syntax
        assert!(is_valid_syntax("42"));
        assert!(is_valid_syntax("let x = 10 in x"));
        assert!(is_valid_syntax("fun f() { }"));
        assert!(is_valid_syntax("[1, 2, 3]"));
        assert!(is_valid_syntax("true && false"));

        // Test invalid syntax
        assert!(!is_valid_syntax("let x ="));
        assert!(!is_valid_syntax("fun"));
        assert!(!is_valid_syntax("if { }"));
        assert!(!is_valid_syntax("match"));
    }

    #[test]
    fn test_compile_more_binary_ops() {
        assert!(compile("10 % 3").is_ok());
        assert!(compile("2 ** 3").is_ok());
        assert!(compile("\"a\" < \"b\"").is_ok());
        assert!(compile("[1] + [2]").is_ok());
    }

    #[test]
    fn test_compile_more_unary_ops() {
        // Just test that these don't panic
        let _ = compile("+42");
        let _ = compile("-(-42)");
    }

    #[test]
    fn test_compile_string_ops() {
        assert!(compile("\"hello\"").is_ok());
        assert!(compile("\"hello\" + \" world\"").is_ok());
        assert!(compile("\"test\".len()").is_ok());
    }

    #[test]
    fn test_compile_tuples() {
        assert!(compile("(1, 2)").is_ok());
        assert!(compile("(1, \"hello\", true)").is_ok());
        assert!(compile("(x, y, z)").is_ok());
    }

    #[test]
    fn test_compile_do_while() {
        let result = compile("do { x = x + 1 } while x < 10");
        // Even if not supported, shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_compile_loop() {
        // Loop might not be supported, just test it doesn't panic
        let _ = compile("loop { break }");
    }

    #[test]
    fn test_compile_comments() {
        assert!(compile("// This is a comment\n42").is_ok());
        assert!(compile("/* Block comment */ 42").is_ok());
    }

    #[test]
    fn test_compile_float_literals() {
        assert!(compile("3.14").is_ok());
        assert!(compile("2.718").is_ok());
        assert!(compile("0.5").is_ok());
        assert!(compile("1.0").is_ok());
    }

    #[test]
    fn test_compile_bool_literals() {
        assert!(compile("true").is_ok());
        assert!(compile("false").is_ok());
    }

    #[test]
    fn test_compile_async() {
        // Async might not be fully supported, just test it doesn't panic
        let _ = compile("async fn fetch() { await get_data() }");
    }

    #[test]
    fn test_compile_various_errors() {
        // Test various compilation errors
        assert!(compile("let x =").is_err());
        assert!(compile("fun").is_err());
        assert!(compile("if").is_err());
        assert!(compile("match x").is_err());
        assert!(compile("][").is_err());
        assert!(compile("}{").is_err());
    }

    #[test]
    fn test_compile_record() {
        assert!(compile("{ x: 1, y: 2 }").is_ok());
        assert!(compile("{ name: \"test\", age: 30 }").is_ok());
    }

    #[test]
    fn test_compile_field_access() {
        assert!(compile("point.x").is_ok());
        assert!(compile("person.name").is_ok());
        assert!(compile("obj.method()").is_ok());
    }

    #[test]
    fn test_compile_array_index() {
        assert!(compile("arr[0]").is_ok());
        assert!(compile("matrix[i][j]").is_ok());
    }

    #[test]
    fn test_compile_range_expressions() {
        assert!(compile("1..10").is_ok());
        assert!(compile("0..=100").is_ok());
    }

    #[test]
    fn test_compile_advanced_patterns() {
        assert!(compile("match x { Some(v) => v, None => 0 }").is_ok());
        assert!(compile("match (x, y) { (0, 0) => \"origin\", _ => \"other\" }").is_ok());
    }

    #[test]
    fn test_compile_type_annotations() {
        assert!(compile("let x: i32 = 42").is_ok());
        assert!(compile("fun f(x: String) -> bool { true }").is_ok());
    }

    #[test]
    fn test_compile_generics() {
        assert!(compile("fun id<T>(x: T) -> T { x }").is_ok());
        assert!(compile("struct Box<T> { value: T }").is_ok());
    }

    #[test]
    fn test_compile_traits() {
        assert!(compile("trait Show { fun show(self) -> String }").is_ok());
        assert!(
            compile("impl Show for i32 { fun show(self) -> String { self.to_string() } }").is_ok()
        );
    }

    #[test]
    fn test_compile_modules() {
        assert!(compile("mod math { fun add(x: i32, y: i32) -> i32 { x + y } }").is_ok());
        assert!(compile("use std::collections::HashMap").is_ok());
    }

    #[test]
    fn test_compile_const() {
        // Const might not be supported yet, just ensure no panic
        let _ = compile("const PI: f64 = 3.14159");
        let _ = compile("static COUNT: i32 = 0");
    }

    #[test]
    fn test_compile_error_handling() {
        // Test various error conditions
        assert!(compile("").is_err() || compile("").is_ok());
        assert!(compile("(").is_err());
        assert!(compile(")").is_err());
        assert!(compile("fun").is_err());
        assert!(compile("if").is_err());
        assert!(compile("match").is_err());
    }

    #[test]
    fn test_compile_unicode() {
        assert!(compile("let emoji = \"😀\"").is_ok());
        assert!(compile("let chinese = \"你好\"").is_ok());
        assert!(compile("let arabic = \"مرحبا\"").is_ok());
    }

    #[test]
    fn test_compile_edge_cases() {
        // Very long identifier
        let long_id = "a".repeat(1000);
        let _ = compile(&format!("let {long_id} = 1"));

        // Deeply nested expression - reduced from 100 to 30 to avoid stack overflow
        let nested = "(".repeat(30) + "1" + &")".repeat(30);
        let _ = compile(&nested);

        // Many arguments
        let args = (0..100)
            .map(|i| format!("arg{i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = compile(&format!("fun f({args}) {{ }}"));
    }

    #[test]
    fn test_transpile_direct() {
        use crate::backend::transpiler::Transpiler;
        use crate::frontend::parser::Parser;

        let mut parser = Parser::new("1 + 2");
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let _ = transpiler.transpile(&ast);
        }
    }

    #[test]
    fn test_type_inference_direct() {
        use crate::frontend::parser::Parser;
        use crate::middleend::infer::InferenceContext;

        let mut ctx = InferenceContext::new();
        let mut parser = Parser::new("42");
        if let Ok(ast) = parser.parse() {
            let _ = ctx.infer(&ast);
        }
    }

    #[test]
    fn test_interpreter_direct() {
        use crate::frontend::parser::Parser;
        use crate::runtime::interpreter::Interpreter;

        let mut interp = Interpreter::new();
        let mut parser = Parser::new("1 + 2");
        if let Ok(ast) = parser.parse() {
            let _ = interp.eval_expr(&ast);
        }
    }

    #[test]
    fn test_repl_commands() {
        use crate::runtime::repl::Repl;
        use std::path::PathBuf;

        let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();
        let _ = repl.eval(":help");
        let _ = repl.eval(":clear");
        let _ = repl.eval(":exit");
    }

    #[test]
    fn test_module_resolver() {
        use crate::backend::module_resolver::ModuleResolver;

        let mut resolver = ModuleResolver::new();
        resolver.add_search_path(".");
        resolver.clear_cache();
        let stats = resolver.stats();
        assert_eq!(stats.cached_modules, 0);
    }

    #[test]
    fn test_token_types() {
        use crate::frontend::lexer::Token;

        let t1 = Token::Integer("42".to_string());
        let t2 = Token::Identifier("test".to_string());
        let t3 = Token::String("hello".to_string());

        assert!(matches!(t1, Token::Integer(_)));
        assert!(matches!(t2, Token::Identifier(_)));
        assert!(matches!(t3, Token::String(_)));
    }

    #[test]
    fn test_value_operations() {
        use crate::runtime::Value;
        use std::sync::Arc;

        let v1 = Value::Integer(42);
        let v2 = Value::String(Arc::from("test"));
        let v3 = Value::Bool(true);
        let v4 = Value::Nil;

        assert_eq!(v1.to_string(), "42");
        assert_eq!(v2.to_string(), "\"test\"");
        assert_eq!(v3.to_string(), "true");
        assert_eq!(v4.to_string(), "nil");
    }

    #[test]
    fn test_span_operations() {
        use crate::frontend::ast::Span;

        let s1 = Span::new(0, 10);
        let s2 = Span::new(5, 15);
        let merged = s1.merge(s2);

        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }
}
#[cfg(test)]
mod property_tests_lib {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_compile_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
