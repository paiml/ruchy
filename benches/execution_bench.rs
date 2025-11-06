//! Performance benchmarks for Ruchy execution modes
//!
//! Measures startup time, evaluation speed, and transpilation overhead

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ruchy::runtime::repl::Repl;
use ruchy::{Parser, Transpiler};
use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

fn benchmark_eval_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval_simple");

    let expressions = vec![
        ("arithmetic", "2 + 2"),
        ("string_concat", r#""hello" + " world""#),
        ("list_literal", "[1, 2, 3, 4, 5]"),
        ("function_call", "42.abs()"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(name), &expr, |b, &expr| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            b.iter(|| {
                let _ = repl.eval(black_box(expr));
            });
        });
    }

    group.finish();
}

fn benchmark_eval_complex(c: &mut Criterion) {
    let mut group = c.benchmark_group("eval_complex");
    group.measurement_time(Duration::from_secs(10));

    let expressions = vec![
        ("list_map", "[1, 2, 3, 4, 5].map(|x| x * 2)"),
        ("list_filter", "[1, 2, 3, 4, 5].filter(|x| x > 2)"),
        ("list_reduce", "[1, 2, 3, 4, 5].reduce(0, |acc, x| acc + x)"),
        (
            "string_interpolation",
            r#"let x = 42; f"The answer is {x}""#,
        ),
        ("match_expr", "match 5 { 1 => 10, 5 => 50, _ => 0 }"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(name), &expr, |b, &expr| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            b.iter(|| {
                let _ = repl.eval(black_box(expr));
            });
        });
    }

    group.finish();
}

fn benchmark_parse_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    let code_samples = vec![
        ("simple_expr", "2 + 2"),
        ("function_def", "fun square(x: i32) -> i32 { x * x }"),
        ("match_expr", "match x { 1 => 'a', 2 => 'b', _ => 'z' }"),
        ("list_comprehension", "[x * 2 for x in range(10) if x > 5]"),
    ];

    for (name, code) in code_samples {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, &code| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(code));
                let _ = parser.parse();
            });
        });
    }

    group.finish();
}

fn benchmark_transpile(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile");

    let code_samples = vec![
        ("arithmetic", "2 + 2 * 3"),
        ("function", "fun add(x: i32, y: i32) -> i32 { x + y }"),
        ("string_interp", r#"let name = "World"; f"Hello, {name}!""#),
        ("list_ops", "[1, 2, 3].map(|x| x * 2).filter(|x| x > 2)"),
    ];

    for (name, code) in code_samples {
        group.bench_with_input(BenchmarkId::from_parameter(name), &code, |b, &code| {
            let mut parser = Parser::new(code);
            let ast = parser.parse().expect("Failed to parse");
            let mut transpiler = Transpiler::new();

            b.iter(|| {
                let _ = transpiler.transpile(black_box(&ast));
            });
        });
    }

    group.finish();
}

fn benchmark_repl_startup(c: &mut Criterion) {
    c.bench_function("repl_startup", |b| {
        b.iter(|| {
            let _ = Repl::new(PathBuf::from("."));
        });
    });
}

fn benchmark_fibonacci(c: &mut Criterion) {
    let mut group = c.benchmark_group("fibonacci");
    group.measurement_time(Duration::from_secs(10));

    let fib_code = r"
        fun fib(n: i32) -> i32 {
            if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
        }
    ";

    let test_values = vec![5, 10, 15];

    for n in test_values {
        let expr = format!("{fib_code}\nfib({n})");

        group.bench_with_input(BenchmarkId::from_parameter(n), &expr, |b, expr| {
            let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");
            b.iter(|| {
                let _ = repl.eval(black_box(expr));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_eval_simple,
    benchmark_eval_complex,
    benchmark_parse_only,
    benchmark_transpile,
    benchmark_repl_startup,
    benchmark_fibonacci
);
criterion_main!(benches);
