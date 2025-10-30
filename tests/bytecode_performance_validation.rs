//! Bytecode VM Performance Validation
//!
//! OPT-021: Bytecode VM Performance Validation
//!
//! Simple test-based performance measurement to validate the 98-99% speedup claims

#![allow(clippy::ignore_without_reason)] // Performance tests run with --ignored flag
//! by comparing AST interpreter vs bytecode VM execution.
//!
//! This is a fallback approach that avoids criterion/linker complexity.
//!
//! Run with: cargo test --release --test `bytecode_performance_validation` -- --nocapture --ignored

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;
use std::time::Instant;

/// Helper to measure execution time for a code snippet
fn measure_ast_execution(code: &str, iterations: u32) -> f64 {
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let start = Instant::now();
    for _ in 0..iterations {
        let mut interpreter = Interpreter::new();
        interpreter.eval_expr(&ast).expect("AST eval failed");
    }
    start.elapsed().as_secs_f64()
}

/// Benchmark helper that reports comparative performance
fn benchmark_code(name: &str, code: &str, iterations: u32) {
    println!("\n{}", "=".repeat(60));
    println!("Benchmark: {name}");
    println!("{}", "=".repeat(60));

    // Measure AST interpreter
    let ast_time = measure_ast_execution(code, iterations);
    let ast_ms = ast_time * 1000.0;
    let ast_per_iter = (ast_ms / f64::from(iterations)) * 1000.0; // microseconds

    println!("AST Interpreter:");
    println!("  Total: {ast_ms:.2}ms");
    println!("  Per iteration: {ast_per_iter:.2}µs");

    // NOTE: Bytecode VM comparison would go here once VM is stable
    // For now, we document AST baseline performance

    println!();
}

#[test]
#[ignore] // Run with --ignored --nocapture
fn test_opt_021_basic_arithmetic() {
    benchmark_code(
        "Basic Arithmetic (OPT-001)",
        "2 + 2",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_complex_arithmetic() {
    benchmark_code(
        "Complex Arithmetic (OPT-001)",
        "((10 + 5) * 2 - 3) / 4",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_variables() {
    benchmark_code(
        "Variable Access (OPT-002)",
        "{ let x = 10; let y = 20; x + y }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_comparisons() {
    benchmark_code(
        "Comparisons (OPT-003)",
        "{ let x = 10; let y = 20; x < y }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_logical_ops() {
    benchmark_code(
        "Logical Operations (OPT-004)",
        "{ let a = true; let b = false; a && !b }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_while_loop() {
    benchmark_code(
        "While Loop (OPT-006)",
        r"{
            let mut sum = 0;
            let mut i = 0;
            while i < 10 {
                sum = sum + i;
                i = i + 1;
            }
            sum
        }",
        1_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_assignments() {
    benchmark_code(
        "Assignments (OPT-008)",
        r"{
            let mut x = 0;
            x = 10;
            x = x + 5;
            x = x * 2;
            x
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_for_loop() {
    benchmark_code(
        "For Loop (OPT-012)",
        r"{
            let mut sum = 0;
            for i in [1, 2, 3, 4, 5] {
                sum = sum + i;
            }
            sum
        }",
        1_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_array_indexing() {
    benchmark_code(
        "Array Indexing (OPT-013)",
        r"{
            let arr = [10, 20, 30, 40, 50];
            arr[0] + arr[2] + arr[4]
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_string_methods() {
    benchmark_code(
        "String Methods (OPT-014)",
        r#"{
            let s = "hello";
            s.len()
        }"#,
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_object_field_access() {
    benchmark_code(
        "Object Field Access (OPT-015)",
        r"{
            let obj = { x: 10, y: 20 };
            obj.x + obj.y
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_object_literal() {
    benchmark_code(
        "Object Literal (OPT-016)",
        r#"{ name: "Alice", age: 30, score: 95 }"#,
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_tuple_literal() {
    benchmark_code(
        "Tuple Literal (OPT-017)",
        "(1, 2, 3, 4, 5)",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_match_simple() {
    benchmark_code(
        "Match Expression (OPT-018)",
        r"{
            let x = 2;
            match x {
                1 => 10,
                2 => 20,
                _ => 0,
            }
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_closure_simple() {
    benchmark_code(
        "Closure (OPT-019)",
        r"{
            let x = 10;
            let f = |y| x + y;
            f(5)
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_non_literal_array() {
    benchmark_code(
        "Non-Literal Array (OPT-020)",
        r"{
            let x = 10;
            let y = 20;
            let arr = [x, y, x + y];
            arr[0] + arr[1] + arr[2]
        }",
        10_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_fibonacci() {
    benchmark_code(
        "Fibonacci Iterative (Comprehensive)",
        r"{
            let mut a = 0;
            let mut b = 1;
            let mut i = 0;
            while i < 10 {
                let temp = a + b;
                a = b;
                b = temp;
                i = i + 1;
            }
            b
        }",
        1_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_data_processing() {
    benchmark_code(
        "Data Processing (Comprehensive)",
        r"{
            let data = [10, 20, 30, 40, 50];
            let mut sum = 0;
            let mut count = 0;

            for x in data {
                if x > 15 {
                    sum = sum + x;
                    count = count + 1;
                }
            }

            { sum: sum, count: count, avg: sum / count }
        }",
        1_000
    );
}

#[test]
#[ignore = "Manual run: cargo test --ignored --nocapture"]
fn test_opt_021_performance_summary() {
    println!("\n{}", "=".repeat(60));
    println!("BYTECODE VM PERFORMANCE VALIDATION - OPT-021");
    println!("{}", "=".repeat(60));
    println!("\nBaseline AST Interpreter Performance:");
    println!("- This test suite establishes baseline AST performance");
    println!("- Bytecode VM comparison will be added once VM is stable");
    println!("- Expected: 98-99% speedup (50-100x faster)");
    println!("\nTest Coverage:");
    println!("- Phase 1: OPT-001 to OPT-010 (Basic operations)");
    println!("- Phase 2: OPT-011 to OPT-020 (Complex features)");
    println!("\nRun all benchmarks:");
    println!("  cargo test --release --test bytecode_performance_validation -- --ignored --nocapture");
    println!("{}", "=".repeat(60));
}
