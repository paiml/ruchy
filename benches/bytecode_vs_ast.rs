//! OPT-010: Bytecode VM vs AST Interpreter Performance Benchmarks
//!
//! Validates the claim that bytecode execution is 40-60% faster than AST interpretation.
//! Measures execution time for various workloads in both modes.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ruchy::frontend::parser::Parser;
use ruchy::runtime::bytecode::{Compiler, VM};
use ruchy::runtime::interpreter::Interpreter;
use std::hint::black_box;
use std::time::Duration;

/// Helper: Execute code in AST mode
fn execute_ast(source: &str) {
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse failed");
    let mut interpreter = Interpreter::new();
    let _ = interpreter.eval_expr(&ast);
}

/// Helper: Execute code in bytecode mode
fn execute_bytecode(source: &str) {
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Parse failed");
    let mut compiler = Compiler::new("bench".to_string());
    compiler.compile_expr(&ast).expect("Compile failed");
    let chunk = compiler.finalize();
    let mut vm = VM::new();
    let _ = vm.execute(&chunk);
}

fn bench_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("arithmetic");

    let workloads = vec![
        ("simple", "10 + 32"),
        ("complex", "(10 + 5) * 2 + 12"),
        ("nested", "((10 + 5) * 2) + ((20 - 8) / 2)"),
    ];

    for (name, code) in workloads {
        group.bench_with_input(BenchmarkId::new("ast", name), code, |b, code| {
            b.iter(|| execute_ast(black_box(code)));
        });

        group.bench_with_input(BenchmarkId::new("bytecode", name), code, |b, code| {
            b.iter(|| execute_bytecode(black_box(code)));
        });
    }

    group.finish();
}

fn bench_loops(c: &mut Criterion) {
    let mut group = c.benchmark_group("loops");
    group.measurement_time(Duration::from_secs(10));

    let workloads = vec![
        (
            "count_to_10",
            "{ let mut i = 0; while i < 10 { i = i + 1 }; i }",
        ),
        (
            "count_to_100",
            "{ let mut i = 0; while i < 100 { i = i + 1 }; i }",
        ),
        (
            "sum_1_to_10",
            "{ let mut sum = 0; let mut i = 1; while i <= 10 { sum = sum + i; i = i + 1 }; sum }",
        ),
        (
            "sum_1_to_50",
            "{ let mut sum = 0; let mut i = 1; while i <= 50 { sum = sum + i; i = i + 1 }; sum }",
        ),
    ];

    for (name, code) in workloads {
        group.bench_with_input(BenchmarkId::new("ast", name), code, |b, code| {
            b.iter(|| execute_ast(black_box(code)));
        });

        group.bench_with_input(BenchmarkId::new("bytecode", name), code, |b, code| {
            b.iter(|| execute_bytecode(black_box(code)));
        });
    }

    group.finish();
}

fn bench_comparisons(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparisons");

    let workloads = vec![
        ("simple_eq", "42 == 42"),
        ("simple_lt", "10 < 20"),
        ("complex_and", "(10 > 5) && (20 < 30)"),
        ("complex_or", "(10 < 5) || (20 == 20)"),
        ("chain", "(10 < 20) && (20 < 30) && (30 < 40)"),
    ];

    for (name, code) in workloads {
        group.bench_with_input(BenchmarkId::new("ast", name), code, |b, code| {
            b.iter(|| execute_ast(black_box(code)));
        });

        group.bench_with_input(BenchmarkId::new("bytecode", name), code, |b, code| {
            b.iter(|| execute_bytecode(black_box(code)));
        });
    }

    group.finish();
}

fn bench_control_flow(c: &mut Criterion) {
    let mut group = c.benchmark_group("control_flow");

    let workloads = vec![
        ("if_true", "if true { 42 } else { 0 }"),
        ("if_false", "if false { 0 } else { 42 }"),
        (
            "nested_if",
            "if true { if false { 0 } else { 42 } } else { 100 }",
        ),
        ("if_comparison", "if 10 > 5 { 42 } else { 0 }"),
    ];

    for (name, code) in workloads {
        group.bench_with_input(BenchmarkId::new("ast", name), code, |b, code| {
            b.iter(|| execute_ast(black_box(code)));
        });

        group.bench_with_input(BenchmarkId::new("bytecode", name), code, |b, code| {
            b.iter(|| execute_bytecode(black_box(code)));
        });
    }

    group.finish();
}

fn bench_fibonacci_iterative(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci_iterative");
    group.measurement_time(Duration::from_secs(15));

    // Iterative Fibonacci using while loops and mutations
    let fib_code = r"{
        let mut a = 0;
        let mut b = 1;
        let mut i = 0;
        while i < N {
            let temp = a + b;
            a = b;
            b = temp;
            i = i + 1
        };
        a
    }";

    let test_values = vec![7, 15, 25];

    for n in test_values {
        let code = fib_code.replace('N', &n.to_string());

        group.bench_with_input(BenchmarkId::new("ast", n), &code, |b, code| {
            b.iter(|| execute_ast(black_box(code)));
        });

        group.bench_with_input(BenchmarkId::new("bytecode", n), &code, |b, code| {
            b.iter(|| execute_bytecode(black_box(code)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_arithmetic,
    bench_loops,
    bench_comparisons,
    bench_control_flow,
    bench_fibonacci_iterative
);
criterion_main!(benches);
