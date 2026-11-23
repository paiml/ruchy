#![allow(missing_docs)]
// BUG-004 (#123): recursion depth 50 must not overflow a 2MB test-thread stack,
// and recursion past the limit must return an error, not abort.

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

fn run_on_small_stack(depth: i64) -> Result<String, String> {
    let code = format!(
        "fun count_down(n: i64) -> i64 {{ if n <= 0 {{ 0 }} else {{ count_down(n - 1) }} }}\ncount_down({depth})"
    );
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let ast = Parser::new(&code).parse().expect("parse");
            let mut interp = Interpreter::new();
            interp
                .eval_expr(&ast)
                .map(|v| v.to_string())
                .map_err(|e| e.to_string())
        })
        .expect("spawn")
        .join()
        .expect("thread must not overflow its stack")
}

#[test]
fn test_bug004_depth_50_on_2mb_stack() {
    assert_eq!(run_on_small_stack(50), Ok("0".to_string()));
}

#[test]
fn test_bug004_depth_over_limit_is_graceful_error() {
    let err = run_on_small_stack(100_000).expect_err("must error");
    assert!(err.contains("Recursion limit"), "got: {err}");
}

#[test]
fn test_bug004_depth_50_with_arithmetic_on_2mb_stack() {
    let code = "fun count(n: i64) -> i64 { if n == 0 { 0 } else { 1 + count(n - 1) } }\ncount(50)";
    let out = std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(move || {
            let ast = Parser::new(code).parse().expect("parse");
            Interpreter::new()
                .eval_expr(&ast)
                .map(|v| v.to_string())
                .map_err(|e| e.to_string())
        })
        .expect("spawn")
        .join()
        .expect("no stack overflow");
    assert_eq!(out, Ok("50".to_string()));
}
