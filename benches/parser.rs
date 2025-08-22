use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ruchy::frontend::parser::Parser;
use std::time::Duration;

// Benchmark simple expressions
fn parse_simple_expr(c: &mut Criterion) {
    c.bench_function("parse_simple_expr", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box("1 + 2 * 3"));
            parser.parse()
        });
    });
}

// Benchmark complex function definitions
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
        });
    });
}

// Benchmark pipeline operators
fn parse_pipeline(c: &mut Criterion) {
    let input = "[1, 2, 3, 4, 5] >> map(double) >> filter(even) >> reduce(sum)";

    c.bench_function("parse_pipeline", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(input));
            parser.parse()
        });
    });
}

// Benchmark parsing performance with different input sizes
fn parse_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_scalability");

    for size in &[10, 100, 1000, 10000] {
        let input = generate_large_input(*size);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &input, |b, input| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(input));
                parser.parse()
            });
        });
    }
    group.finish();
}

// Benchmark deeply nested expressions
fn parse_nested_expr(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser_nesting");

    for depth in &[5, 10, 20, 50] {
        let input = generate_nested_expr(*depth);
        group.bench_with_input(BenchmarkId::from_parameter(depth), &input, |b, input| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(input));
                parser.parse()
            });
        });
    }
    group.finish();
}

// Benchmark operator precedence handling
fn parse_precedence(c: &mut Criterion) {
    let expressions = vec![
        ("simple_math", "1 + 2 * 3 - 4 / 2"),
        ("comparison", "x > 5 && y < 10 || z == 0"),
        ("mixed", "a + b * c > d && e || f == g + h"),
        ("bitwise", "a & b | c ^ d << 2 >> 1"),
    ];

    let mut group = c.benchmark_group("parser_precedence");
    for (name, expr) in expressions {
        group.bench_with_input(BenchmarkId::from_parameter(name), expr, |b, input| {
            b.iter(|| {
                let mut parser = Parser::new(black_box(input));
                parser.parse()
            });
        });
    }
    group.finish();
}

// Benchmark pattern matching parsing
fn parse_pattern_matching(c: &mut Criterion) {
    let input = r"
        match value {
            Some(Ok(x)) if x > 0 => x * 2,
            Some(Err(e)) => panic!(e),
            None => 0,
            _ => -1,
        }
    ";

    c.bench_function("parse_pattern_matching", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(input));
            parser.parse()
        });
    });
}

// Benchmark actor definition parsing
fn parse_actor(c: &mut Criterion) {
    let input = r"
        actor MessageProcessor {
            mut queue: Vec<Message> = vec![];
            mut processed: usize = 0;
            
            pub fn process(msg: Message) {
                self.queue.push(msg);
                self.processed += 1;
            }
            
            pub fn get_stats() -> Stats {
                Stats {
                    queued: self.queue.len(),
                    processed: self.processed,
                }
            }
        }
    ";

    c.bench_function("parse_actor", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(input));
            parser.parse()
        });
    });
}

// Helper function to generate large input
fn generate_large_input(num_statements: usize) -> String {
    use std::fmt::Write;
    let mut input = String::new();
    for i in 0..num_statements {
        let _ = write!(&mut input, "let var_{i} = {i}; ");
    }
    input
}

// Helper function to generate nested expressions
fn generate_nested_expr(depth: usize) -> String {
    let mut expr = "42".to_string();
    for _ in 0..depth {
        expr = format!("({expr} + 1)");
    }
    expr
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .significance_level(0.05)
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets = parse_simple_expr,
              parse_complex_expr,
              parse_pipeline,
              parse_scalability,
              parse_nested_expr,
              parse_precedence,
              parse_pattern_matching,
              parse_actor
}

criterion_main!(benches);
