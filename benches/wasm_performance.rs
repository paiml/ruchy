// benches/wasm_performance.rs
// WebAssembly Extreme Quality Assurance Framework v3.0
// WASM Performance Benchmark Suite

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

fn benchmark_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_operations");

    // Test various payload sizes as specified in the framework
    for size in &[1024, 10_240, 102_400, 1_048_576] {
        group.bench_with_input(BenchmarkId::new("allocation", size), size, |b, &size| {
            b.iter(|| {
                let data = vec![0u8; size];
                black_box(data);
            });
        });

        group.bench_with_input(BenchmarkId::new("processing", size), size, |b, &size| {
            let data = vec![0u8; size];
            b.iter(|| {
                // Simulate processing bytes (checksum calculation)
                let checksum: u32 = data.iter().map(|&x| u32::from(x)).sum();
                black_box(checksum)
            });
        });

        group.bench_with_input(BenchmarkId::new("parser_stress", size), size, |b, &size| {
            // Create input proportional to size
            let input_size = std::cmp::min(size / 100, 1000); // Scale down for parsing
            let source = format!("let x = {}; x + 1", "1 + ".repeat(input_size));

            b.iter(|| {
                let mut parser = ruchy::frontend::Parser::new(black_box(&source));
                let _ = black_box(parser.parse());
            });
        });
    }

    group.finish();
}

fn benchmark_compilation_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_compilation");

    let test_cases = vec![
        ("simple", "let x = 42"),
        ("function", "fn add(a: i32, b: i32) -> i32 { a + b }"),
        (
            "complex",
            "fn factorial(n: i32) -> i32 { if n <= 1 { 1 } else { n * factorial(n - 1) } }",
        ),
        (
            "control_flow",
            "fn test(x: i32) -> i32 { if x > 10 { x * 2 } else { x + 1 } }",
        ),
    ];

    for (name, source) in test_cases {
        group.bench_with_input(BenchmarkId::new("parse", name), &source, |b, &source| {
            b.iter(|| {
                let mut parser = ruchy::frontend::Parser::new(black_box(source));
                let _ = black_box(parser.parse());
            });
        });

        group.bench_with_input(
            BenchmarkId::new("transpile", name),
            &source,
            |b, &source| {
                // Pre-parse for transpilation benchmark
                let mut parser = ruchy::frontend::Parser::new(source);
                if let Ok(ast) = parser.parse() {
                    let transpiler = ruchy::backend::Transpiler::new();
                    b.iter(|| {
                        let _ = black_box(transpiler.transpile(black_box(&ast)));
                    });
                }
            },
        );
    }

    group.finish();
}

fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("wasm_memory");

    group.bench_function("repeated_allocation", |b| {
        b.iter(|| {
            for i in 0..100 {
                let data = vec![i as u8; 1024];
                black_box(data);
            }
        });
    });

    group.bench_function("string_operations", |b| {
        b.iter(|| {
            for i in 0..50 {
                let s = format!("test string number {i}");
                let processed = s.to_uppercase();
                black_box(processed);
            }
        });
    });

    group.bench_function("parser_reuse", |b| {
        let sources = vec!["let a = 1", "let b = 2", "let c = 3", "fn test() { 42 }"];

        b.iter(|| {
            for source in &sources {
                let mut parser = ruchy::frontend::Parser::new(source);
                let _ = black_box(parser.parse());
            }
        });
    });

    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(200)
        .measurement_time(std::time::Duration::from_secs(10))
        .warm_up_time(std::time::Duration::from_secs(3));
    targets = benchmark_suite, benchmark_compilation_pipeline, benchmark_memory_patterns
);

criterion_main!(benches);
