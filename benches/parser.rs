use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ruchy::Parser;

fn parse_simple_expr(c: &mut Criterion) {
    c.bench_function("parse_simple_expr", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box("1 + 2 * 3"));
            parser.parse()
        })
    });
}

fn parse_complex_expr(c: &mut Criterion) {
    let input = "fun fibonacci(n: i32) -> i32 {
        if n <= 1 {
            n
        } else {
            fibonacci(n - 1) + fibonacci(n - 2)
        }
    }";

    c.bench_function("parse_complex_expr", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(input));
            parser.parse()
        })
    });
}

fn parse_pipeline(c: &mut Criterion) {
    let input = "[1, 2, 3, 4, 5] |> map(double) |> filter(even) |> reduce(sum)";

    c.bench_function("parse_pipeline", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(input));
            parser.parse()
        })
    });
}

criterion_group!(
    benches,
    parse_simple_expr,
    parse_complex_expr,
    parse_pipeline
);
criterion_main!(benches);
