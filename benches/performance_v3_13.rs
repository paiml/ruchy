//! Performance benchmarks for Sprint v3.13.0
//! Focus on interpreter speed, memory usage, compilation time

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ruchy::backend::transpiler::Transpiler;
use ruchy::compile;
use ruchy::frontend::parser::Parser;
use std::hint::black_box;

fn bench_parser_simple(c: &mut Criterion) {
    let inputs = vec![
        ("literal", "42"),
        ("binary", "1 + 2 * 3"),
        ("variable", "let x = 10"),
        ("function", "fn add(a, b) { a + b }"),
        ("match", "match x { 1 => 'one', _ => 'other' }"),
    ];

    let mut group = c.benchmark_group("parser_simple");
    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("parse", name), &input, |b, &input| {
            b.iter(|| {
                let mut parser = Parser::new(input);
                let _ = black_box(parser.parse());
            });
        });
    }
    group.finish();
}

fn bench_parser_complex(c: &mut Criterion) {
    let complex_program = r#"
    fn fibonacci(n) {
        if n <= 1 {
            n
        } else {
            fibonacci(n - 1) + fibonacci(n - 2)
        }
    }

    fn main() {
        let result = fibonacci(10);
        println("Result: " + result.to_string());

        let list = [1, 2, 3, 4, 5];
        let sum = list.reduce(|a, b| a + b);

        match sum {
            15 => println("Correct sum"),
            _ => println("Wrong sum")
        }
    }
    "#;

    c.bench_function("parser_complex_program", |b| {
        b.iter(|| {
            let mut parser = Parser::new(complex_program);
            let _ = black_box(parser.parse());
        });
    });
}

fn bench_transpiler(c: &mut Criterion) {
    let programs = vec![
        ("hello_world", "println('Hello, World!')"),
        ("arithmetic", "(1 + 2) * (3 - 4) / 5"),
        ("function_def", "fn square(x) { x * x }"),
        ("struct_def", "struct Point { x: i32, y: i32 }"),
        ("pattern_match", "match x { Some(v) => v, None => 0 }"),
    ];

    let mut group = c.benchmark_group("transpiler");
    for (name, input) in programs {
        group.bench_with_input(BenchmarkId::new("transpile", name), &input, |b, &input| {
            let mut parser = Parser::new(input);
            let ast = parser.parse().unwrap();
            let transpiler = Transpiler::new();

            b.iter(|| {
                let _ = black_box(transpiler.transpile_to_string(&ast));
            });
        });
    }
    group.finish();
}

fn bench_compile_pipeline(c: &mut Criterion) {
    let expressions = vec![
        ("literal", "42"),
        ("arithmetic", "1 + 2 * 3 - 4 / 2"),
        ("variable", "let x = 10; x + 5"),
        ("function_def", "fn add(a, b) { a + b }"),
        ("list_ops", "[1, 2, 3].map(|x| x * 2)"),
    ];

    let mut group = c.benchmark_group("compile_pipeline");
    for (name, input) in expressions {
        group.bench_with_input(BenchmarkId::new("compile", name), &input, |b, &input| {
            b.iter(|| {
                let _ = black_box(compile(input));
            });
        });
    }
    group.finish();
}

fn bench_memory_allocation(c: &mut Criterion) {
    c.bench_function("parser_allocation_stress", |b| {
        let input =
            "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10].map(|x| x * 2).filter(|x| x > 5).reduce(|a, b| a + b)";
        b.iter(|| {
            let mut parser = Parser::new(input);
            let _ = black_box(parser.parse());
        });
    });

    c.bench_function("compile_allocation_stress", |b| {
        let input = "let list = []; for i in 0..100 { list.push(i * 2) }; list";

        b.iter(|| {
            let _ = black_box(compile(input));
        });
    });
}

fn bench_recursive_compilation(c: &mut Criterion) {
    let factorial = r"
    fn factorial(n) {
        if n <= 1 {
            1
        } else {
            n * factorial(n - 1)
        }
    }
    factorial(10)
    ";

    c.bench_function("compile_recursive_factorial", |b| {
        b.iter(|| {
            let _ = black_box(compile(factorial));
        });
    });
}

criterion_group!(parser_benches, bench_parser_simple, bench_parser_complex);

criterion_group!(transpiler_benches, bench_transpiler);

criterion_group!(
    pipeline_benches,
    bench_compile_pipeline,
    bench_memory_allocation,
    bench_recursive_compilation
);

criterion_main!(parser_benches, transpiler_benches, pipeline_benches);
