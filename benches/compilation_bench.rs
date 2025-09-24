#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args,
    clippy::semicolon_if_nothing_returned
)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use ruchy::{Parser, Transpiler};
use std::time::Duration;

fn parse_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    group.measurement_time(Duration::from_secs(10));

    // Small program
    let small_program = "let x = 42";
    group.bench_function("small_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(small_program));
            parser.parse()
        })
    });

    // Medium program
    let medium_program = r"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        let result = fibonacci(10)
        println(result)
    ";
    group.bench_function("medium_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(medium_program));
            parser.parse()
        })
    });

    // Large program
    let large_program = include_str!("../examples/marco_polo_complete.ruchy");
    group.bench_function("large_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(large_program));
            parser.parse()
        })
    });

    group.finish();
}

fn transpile_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpilation");

    // Pre-parse ASTs for transpilation benchmarks
    let small_ast = {
        let mut parser = Parser::new("let x = 42");
        parser.parse().unwrap()
    };

    let medium_ast = {
        let mut parser = Parser::new(
            r"
            fn fibonacci(n: i32) -> i32 {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
        ",
        );
        parser.parse().unwrap()
    };

    group.bench_function("small_ast", |b| {
        b.iter(|| {
            let transpiler = Transpiler::new();
            transpiler.transpile(black_box(&small_ast))
        })
    });

    group.bench_function("medium_ast", |b| {
        b.iter(|| {
            let transpiler = Transpiler::new();
            transpiler.transpile(black_box(&medium_ast))
        })
    });

    group.finish();
}

fn throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Test parsing throughput in MB/s
    let sizes = [1_000, 10_000, 100_000];

    for size in &sizes {
        let input = "let x = 42; ".repeat(size / 12); // Each statement is ~12 bytes
        let bytes = input.len() as u64;

        group.throughput(criterion::Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(input));
                parser.parse()
            })
        });
    }

    group.finish();
}

fn end_to_end_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");

    // Benchmark full parse + transpile pipeline
    let programs = vec![
        ("hello_world", r#"println("Hello, World!")"#),
        ("arithmetic", "let x = 10 + 20 * 30 - 40 / 5"),
        (
            "function",
            r"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
            add(5, 3)
        ",
        ),
    ];

    for (name, program) in programs {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(program));
                let ast = parser.parse().unwrap();
                let transpiler = Transpiler::new();
                transpiler.transpile(&ast)
            })
        });
    }

    group.finish();
}

fn expression_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("expressions");

    // Benchmark different expression types
    let expressions = vec![
        ("literal", "42"),
        ("binary_op", "1 + 2"),
        ("complex_arithmetic", "(1 + 2) * (3 - 4) / (5 % 6)"),
        ("function_call", "foo(1, 2, 3)"),
        ("array", "[1, 2, 3, 4, 5]"),
        ("string", r#""Hello, World!""#),
    ];

    for (name, expr) in expressions {
        group.bench_function(name, |b| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(expr));
                parser.parse_expr()
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    parse_benchmark,
    transpile_benchmark,
    throughput_benchmark,
    end_to_end_benchmark,
    expression_benchmark
);
criterion_main!(benches);
