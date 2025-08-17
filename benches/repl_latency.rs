//! REPL Performance Benchmarks
//!
//! Benchmarks startup time, evaluation latency, type lookup, and cache hit rates
//! as specified in ruchy-repl-testing-todo.yaml

#![allow(clippy::expect_used)] // Benchmarks can panic on failures
#![allow(clippy::uninlined_format_args)] // Format args readability in benchmarks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ruchy::runtime::repl::Repl;
use std::time::Duration;

/// Benchmark REPL startup time
fn bench_repl_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("repl_startup");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("new", |b| {
        b.iter(|| {
            let repl = Repl::new().expect("Failed to create REPL");
            black_box(repl);
        });
    });

    group.finish();
}

/// Benchmark evaluation latency for different expression types
fn bench_repl_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("repl_eval");
    group.measurement_time(Duration::from_secs(20));

    let test_cases = vec![
        ("literal_int", "42"),
        ("literal_float", "3.14"),
        ("literal_string", r#""hello world""#),
        ("literal_bool", "true"),
        ("simple_arithmetic", "1 + 2 * 3"),
        ("complex_arithmetic", "((1 + 2) * 3 - 4) / 5"),
        ("let_binding", "let x = 42"),
        ("variable_ref", "x"), // Assumes previous let binding
        ("function_def", "fun add(a: i32, b: i32) -> i32 { a + b }"),
        ("list_literal", "[1, 2, 3, 4, 5]"),
        ("list_comprehension", "[x * 2 for x in [1, 2, 3]]"),
        ("if_expression", "if true { 1 } else { 2 }"),
        (
            "match_expression",
            "match 42 { 42 => \"found\", _ => \"not found\" }",
        ),
    ];

    for (name, expr) in test_cases {
        group.bench_with_input(BenchmarkId::new("parse_only", name), expr, |b, expr| {
            b.iter(|| {
                let mut parser = ruchy::frontend::parser::Parser::new(black_box(expr));
                let ast = parser.parse().expect("Failed to parse");
                black_box(ast);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("parse_and_transpile", name),
            expr,
            |b, expr| {
                b.iter(|| {
                    let mut parser = ruchy::frontend::parser::Parser::new(black_box(expr));
                    let ast = parser.parse().expect("Failed to parse");
                    let transpiler = ruchy::backend::transpiler::Transpiler::new();
                    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");
                    black_box(rust_code);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark REPL show operations
fn bench_repl_show_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("repl_show");
    group.measurement_time(Duration::from_secs(10));

    let mut repl = Repl::new().expect("Failed to create REPL");

    // Set up some state
    let _ = repl.eval("let x = 42");
    let _ = repl.eval("let y = \"hello\"");
    let _ = repl.eval("fun double(n: i32) -> i32 { n * 2 }");

    group.bench_function("show_history", |b| {
        b.iter(|| {
            let history = repl.show_history();
            black_box(history);
        });
    });

    group.bench_function("show_type", |b| {
        b.iter(|| {
            let type_info = repl.show_type(black_box("x")).expect("Failed to get type");
            black_box(type_info);
        });
    });

    group.bench_function("show_ast", |b| {
        b.iter(|| {
            let ast = repl
                .show_ast(black_box("1 + 2"))
                .expect("Failed to get AST");
            black_box(ast);
        });
    });

    group.bench_function("show_rust", |b| {
        b.iter(|| {
            let mut repl_mut = Repl::new().expect("Failed to create REPL");
            let rust_code = repl_mut
                .show_rust(black_box("true"))
                .expect("Failed to get Rust code");
            black_box(rust_code);
        });
    });

    group.finish();
}

/// Benchmark type inference performance
fn bench_type_inference(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_inference");
    group.measurement_time(Duration::from_secs(15));

    let expressions = vec![
        ("simple", "42"),
        ("arithmetic", "1 + 2 * 3"),
        ("nested", "((1 + 2) * (3 + 4)) / 5"),
        ("function", "fun id(x: i32) -> i32 { x }"),
        ("lambda", "|x: i32| x * 2"),
        ("list", "[1, 2, 3, 4, 5]"),
        ("if_else", "if true { 1 } else { 2 }"),
        ("let_chain", "let x = 1; let y = x + 2; y * 3"),
    ];

    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::new("infer", name), expr, |b, expr| {
            b.iter(|| {
                let mut parser = ruchy::frontend::parser::Parser::new(black_box(expr));
                let ast = parser.parse().expect("Failed to parse");
                let mut ctx = ruchy::middleend::infer::InferenceContext::new();
                let ty = ctx.infer(&ast).expect("Failed to infer type");
                black_box(ty);
            });
        });
    }

    group.finish();
}

/// Benchmark memory usage and allocation patterns
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));

    // Simulate a session with many operations
    let session_operations = (0..100)
        .map(|i| format!("let var_{} = {}", i, i))
        .collect::<Vec<_>>();

    group.throughput(Throughput::Elements(session_operations.len() as u64));
    group.bench_function("large_session", |b| {
        b.iter(|| {
            let mut repl = Repl::new().expect("Failed to create REPL");
            for op in &session_operations {
                let _ = repl.eval(black_box(op));
            }
            black_box(repl);
        });
    });

    group.bench_function("session_with_clear", |b| {
        b.iter(|| {
            let mut repl = Repl::new().expect("Failed to create REPL");
            for (i, op) in session_operations.iter().enumerate() {
                let _ = repl.eval(black_box(op));
                if i % 20 == 19 {
                    repl.clear_session();
                }
            }
            black_box(repl);
        });
    });

    group.finish();
}

/// Benchmark throughput for batch operations
fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(20));

    let batch_sizes = vec![10, 50, 100, 500];

    for batch_size in batch_sizes {
        let operations: Vec<String> = (0..batch_size)
            .map(|i| format!("{} + {}", i, i + 1))
            .collect();

        group.throughput(Throughput::Elements(batch_size));
        group.bench_with_input(
            BenchmarkId::new("eval_batch", batch_size),
            &operations,
            |b, ops| {
                b.iter(|| {
                    let mut repl = Repl::new().expect("Failed to create REPL");
                    for op in ops {
                        let _ = repl.eval(black_box(op));
                    }
                    black_box(repl);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("parse_batch", batch_size),
            &operations,
            |b, ops| {
                b.iter(|| {
                    for op in ops {
                        let mut parser = ruchy::frontend::parser::Parser::new(black_box(op));
                        let ast = parser.parse().expect("Failed to parse");
                        black_box(ast);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark incremental compilation simulation
fn bench_incremental_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");
    group.measurement_time(Duration::from_secs(15));

    // Simulate incremental development where we modify previous definitions
    let base_definitions = vec![
        "let x = 42",
        "let y = x + 1",
        "fun add(a: i32, b: i32) -> i32 { a + b }",
        "let z = add(x, y)",
    ];

    let modifications = vec![
        "let x = 43",                                   // Modify x
        "let w = x * 2",                                // Add new definition
        "fun add(a: i32, b: i32) -> i32 { a + b + 1 }", // Modify function
    ];

    group.bench_function("cold_start", |b| {
        b.iter(|| {
            let mut repl = Repl::new().expect("Failed to create REPL");
            for def in &base_definitions {
                let _ = repl.eval(black_box(def));
            }
            for modif in &modifications {
                let _ = repl.eval(black_box(modif));
            }
            black_box(repl);
        });
    });

    group.bench_function("warm_session", |b| {
        b.iter_with_setup(
            || {
                let mut repl = Repl::new().expect("Failed to create REPL");
                for def in &base_definitions {
                    let _ = repl.eval(def);
                }
                repl
            },
            |mut repl| {
                for modif in &modifications {
                    let _ = repl.eval(black_box(modif));
                }
                black_box(repl);
            },
        );
    });

    group.finish();
}

criterion_group!(
    repl_benches,
    bench_repl_startup,
    bench_repl_eval,
    bench_repl_show_operations,
    bench_type_inference,
    bench_memory_usage,
    bench_throughput,
    bench_incremental_compilation,
);

criterion_main!(repl_benches);
