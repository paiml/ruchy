// OPT-010: Performance Validation - Bytecode VM vs AST Interpreter
//
// Validates that bytecode execution is faster than AST interpretation.
// Uses simple timing measurements instead of full criterion benchmarks.
//
// Requirements from docs/execution/roadmap.yaml:
// - Bytecode should be 40-60% faster than AST for arithmetic/loops
// - Measure execution time for various workloads

#![allow(clippy::ignore_without_reason)] // Performance tests run with --ignored flag
#![allow(missing_docs)]

use ruchy::frontend::parser::Parser;
use ruchy::runtime::bytecode::{Compiler, VM};
use ruchy::runtime::interpreter::{Interpreter, Value};
use std::time::Instant;

/// Helper: Measure execution time for AST mode (returns microseconds)
fn measure_ast(source: &str, iterations: u32) -> u128 {
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse failed");

    let start = Instant::now();
    for _ in 0..iterations {
        let mut interpreter = Interpreter::new();
        let _ = interpreter.eval_expr(&ast);
    }
    start.elapsed().as_micros()
}

/// Helper: Measure execution time for bytecode mode (returns microseconds)
fn measure_bytecode(source: &str, iterations: u32) -> u128 {
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse failed");
    let mut compiler = Compiler::new("bench".to_string());
    compiler.compile_expr(&ast).expect("Compile failed");
    let chunk = compiler.finalize();

    let start = Instant::now();
    for _ in 0..iterations {
        let mut vm = VM::new();
        let _ = vm.execute(&chunk);
    }
    start.elapsed().as_micros()
}

/// Calculate speedup percentage: (`ast_time` - `bytecode_time`) / `ast_time` * 100
fn speedup_percentage(ast_time: u128, bytecode_time: u128) -> f64 {
    if ast_time == 0 { return 0.0; }
    ((ast_time as f64 - bytecode_time as f64) / ast_time as f64) * 100.0
}

#[test]
fn test_opt_010_arithmetic_speedup() {
    let workloads = vec![
        ("simple", "10 + 32", 10000),
        ("complex", "(10 + 5) * 2 + 12", 10000),
        ("nested", "((10 + 5) * 2) + ((20 - 8) / 2)", 10000),
    ];

    for (name, code, iterations) in workloads {
        let ast_time = measure_ast(code, iterations);
        let bytecode_time = measure_bytecode(code, iterations);
        let speedup = speedup_percentage(ast_time, bytecode_time);

        println!("Arithmetic/{name}: AST={ast_time}µs, Bytecode={bytecode_time}µs, Speedup={speedup:.1}%");

        // Bytecode should be faster (positive speedup)
        assert!(speedup > 0.0,
                "Bytecode should be faster than AST for {name}: speedup={speedup:.1}%");
    }
}

#[test]
fn test_opt_010_loop_speedup() {
    let workloads = vec![
        ("count_to_10", "{ let mut i = 0; while i < 10 { i = i + 1 }; i }", 1000),
        ("sum_1_to_10", "{ let mut sum = 0; let mut i = 1; while i <= 10 { sum = sum + i; i = i + 1 }; sum }", 1000),
        ("countdown", "{ let mut i = 10; while i > 0 { i = i - 1 }; i }", 1000),
    ];

    for (name, code, iterations) in workloads {
        let ast_time = measure_ast(code, iterations);
        let bytecode_time = measure_bytecode(code, iterations);
        let speedup = speedup_percentage(ast_time, bytecode_time);

        println!("Loop/{name}: AST={ast_time}µs, Bytecode={bytecode_time}µs, Speedup={speedup:.1}%");

        // Bytecode should be faster (positive speedup)
        assert!(speedup > 0.0,
                "Bytecode should be faster than AST for {name}: speedup={speedup:.1}%");
    }
}

#[test]
fn test_opt_010_comparison_speedup() {
    let workloads = vec![
        ("simple_eq", "42 == 42", 10000),
        ("simple_lt", "10 < 20", 10000),
        ("complex_and", "(10 > 5) && (20 < 30)", 10000),
        ("chain", "(10 < 20) && (20 < 30) && (30 < 40)", 10000),
    ];

    for (name, code, iterations) in workloads {
        let ast_time = measure_ast(code, iterations);
        let bytecode_time = measure_bytecode(code, iterations);
        let speedup = speedup_percentage(ast_time, bytecode_time);

        println!("Comparison/{name}: AST={ast_time}µs, Bytecode={bytecode_time}µs, Speedup={speedup:.1}%");

        // Bytecode should be faster (positive speedup)
        assert!(speedup > 0.0,
                "Bytecode should be faster than AST for {name}: speedup={speedup:.1}%");
    }
}

#[test]
fn test_opt_010_control_flow_speedup() {
    let workloads = vec![
        ("if_true", "if true { 42 } else { 0 }", 10000),
        ("if_false", "if false { 0 } else { 42 }", 10000),
        ("nested_if", "if true { if false { 0 } else { 42 } } else { 100 }", 10000),
        ("if_comparison", "if 10 > 5 { 42 } else { 0 }", 10000),
    ];

    for (name, code, iterations) in workloads {
        let ast_time = measure_ast(code, iterations);
        let bytecode_time = measure_bytecode(code, iterations);
        let speedup = speedup_percentage(ast_time, bytecode_time);

        println!("ControlFlow/{name}: AST={ast_time}µs, Bytecode={bytecode_time}µs, Speedup={speedup:.1}%");

        // Bytecode should be faster (positive speedup)
        assert!(speedup > 0.0,
                "Bytecode should be faster than AST for {name}: speedup={speedup:.1}%");
    }
}

#[test]
fn test_opt_010_fibonacci_speedup() {
    // Iterative Fibonacci - good test for loops + mutations
    let fib_code = r"{
        let mut a = 0;
        let mut b = 1;
        let mut i = 0;
        while i < 20 {
            let temp = a + b;
            a = b;
            b = temp;
            i = i + 1
        };
        a
    }";

    let iterations = 1000;
    let ast_time = measure_ast(fib_code, iterations);
    let bytecode_time = measure_bytecode(fib_code, iterations);
    let speedup = speedup_percentage(ast_time, bytecode_time);

    println!("Fibonacci: AST={ast_time}µs, Bytecode={bytecode_time}µs, Speedup={speedup:.1}%");

    // Bytecode should be faster (positive speedup)
    assert!(speedup > 0.0,
            "Bytecode should be faster than AST for fibonacci: speedup={speedup:.1}%");

    // Verify correctness (Fib(20) = 6765)
    let mut parser = Parser::new(fib_code);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Eval failed");
    assert_eq!(result, Value::Integer(6765), "Fibonacci result should be 6765");
}

#[test]
#[ignore = Only run manually for detailed performance analysis
fn test_opt_010_comprehensive_performance_report() {
    println!("\n=== OPT-010: Comprehensive Performance Report ===\n");

    let workloads = vec![
        ("Arithmetic/Simple", "10 + 32", 10000),
        ("Arithmetic/Complex", "(10 + 5) * 2 + 12", 10000),
        ("Loop/Count10", "{ let mut i = 0; while i < 10 { i = i + 1 }; i }", 1000),
        ("Loop/Sum10", "{ let mut sum = 0; let mut i = 1; while i <= 10 { sum = sum + i; i = i + 1 }; sum }", 1000),
        ("Comparison/Eq", "42 == 42", 10000),
        ("Comparison/Chain", "(10 < 20) && (20 < 30) && (30 < 40)", 10000),
        ("ControlFlow/If", "if true { 42 } else { 0 }", 10000),
        ("ControlFlow/Nested", "if true { if false { 0 } else { 42 } } else { 100 }", 10000),
    ];

    let mut total_ast = 0u128;
    let mut total_bytecode = 0u128;

    println!("{:<30} {:>15} {:>15} {:>10}", "Workload", "AST (µs)", "Bytecode (µs)", "Speedup");
    println!("{:-<70}", "");

    for (name, code, iterations) in workloads {
        let ast_time = measure_ast(code, iterations);
        let bytecode_time = measure_bytecode(code, iterations);
        let speedup = speedup_percentage(ast_time, bytecode_time);

        total_ast += ast_time;
        total_bytecode += bytecode_time;

        println!("{name:<30} {ast_time:>15} {bytecode_time:>15} {speedup:>9.1}%");
    }

    println!("{:-<70}", "");
    let overall_speedup = speedup_percentage(total_ast, total_bytecode);
    println!("{:<30} {:>15} {:>15} {:>9.1}%",
             "TOTAL", total_ast, total_bytecode, overall_speedup);

    println!("\nTarget: 40-60% speedup");
    println!("Actual: {overall_speedup:.1}% speedup");

    if (40.0..=60.0).contains(&overall_speedup) {
        println!("✅ Performance target achieved!");
    } else if overall_speedup > 0.0 {
        println!("⚠️  Bytecode is faster, but outside target range");
    } else {
        println!("❌ Bytecode is slower than AST!");
    }
}
