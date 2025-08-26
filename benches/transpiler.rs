use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ruchy::{Parser, Transpiler};
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};

fn transpile_literals(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_literals");
    
    let literals = vec![
        ("integer", Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span::default(),
            attributes: vec![],
        }),
        ("float", Expr {
            kind: ExprKind::Literal(Literal::Float(3.1415)),
            span: Span::default(),
            attributes: vec![],
        }),
        ("string", Expr {
            kind: ExprKind::Literal(Literal::String("Hello, World!".to_string())),
            span: Span::default(),
            attributes: vec![],
        }),
        ("bool", Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span::default(),
            attributes: vec![],
        }),
    ];
    
    for (name, expr) in literals {
        group.bench_with_input(BenchmarkId::from_parameter(name), &expr, |b, expr| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(expr))
            });
        });
    }
    
    group.finish();
}

fn transpile_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_expressions");
    
    // Parse different expression types
    let test_cases = vec![
        ("binary_op", "1 + 2 * 3"),
        ("function_call", "foo(1, 2, 3)"),
        ("if_expr", "if x > 0 { x } else { -x }"),
        ("match_expr", "match x { 0 => \"zero\", _ => \"other\" }"),
        ("lambda", "|x| x * 2"),
        ("list", "[1, 2, 3, 4, 5]"),
    ];
    
    for (name, code) in test_cases {
        let ast = {
            let mut parser = Parser::new(code);
            parser.parse().expect("Should parse")
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(name), &ast, |b, ast| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(ast))
            });
        });
    }
    
    group.finish();
}

fn transpile_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_functions");
    
    let functions = vec![
        ("simple", "fun add(a: i32, b: i32) -> i32 { a + b }"),
        ("recursive", r"
            fun factorial(n: i32) -> i32 {
                if n <= 1 { 1 } else { n * factorial(n - 1) }
            }
        "),
        ("higher_order", r"
            fun map(f: (i32) -> i32, list: [i32]) -> [i32] {
                list.map(f)
            }
        "),
    ];
    
    for (name, code) in functions {
        let ast = {
            let mut parser = Parser::new(code);
            parser.parse().expect("Should parse")
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(name), &ast, |b, ast| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(ast))
            });
        });
    }
    
    group.finish();
}

fn transpile_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_patterns");
    
    let patterns = vec![
        ("literal", "match x { 42 => \"answer\" }"),
        ("wildcard", "match x { _ => \"any\" }"),
        ("identifier", "match x { y => y }"),
        ("tuple", "match pair { (a, b) => a + b }"),
        ("list", "match list { [first, second] => first }"),
        ("nested", "match opt { Some(Ok(x)) => x, _ => 0 }"),
    ];
    
    for (name, code) in patterns {
        let ast = {
            let mut parser = Parser::new(code);
            parser.parse().expect("Should parse")
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(name), &ast, |b, ast| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(ast))
            });
        });
    }
    
    group.finish();
}

fn transpile_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_scalability");
    
    for size in &[10, 100, 1000] {
        let code = format!("let x = {}; ", "1 + 2 * 3").repeat(*size);
        let ast = {
            let mut parser = Parser::new(&code);
            parser.parse().expect("Should parse")
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(size), &ast, |b, ast| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(ast))
            });
        });
    }
    
    group.finish();
}

fn transpile_real_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("transpile_real_world");
    
    let programs = vec![
        ("quicksort", r"
            fun quicksort(arr: [i32]) -> [i32] {
                if arr.len() <= 1 {
                    arr
                } else {
                    let pivot = arr[0]
                    let less = arr.filter(|x| x < pivot)
                    let equal = arr.filter(|x| x == pivot)
                    let greater = arr.filter(|x| x > pivot)
                    quicksort(less) + equal + quicksort(greater)
                }
            }
        "),
        ("fibonacci_memo", r"
            let mut cache = {}
            
            fun fib_memo(n: i32) -> i32 {
                if cache.has(n) {
                    cache.get(n)
                } else {
                    let result = if n <= 1 { n } else { fib_memo(n-1) + fib_memo(n-2) }
                    cache.set(n, result)
                    result
                }
            }
        "),
        ("data_pipeline", r"
            let data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            let result = data
                .map(|x| x * 2)
                .filter(|x| x > 10)
                .reduce(0, |acc, x| acc + x)
        "),
    ];
    
    for (name, code) in programs {
        let ast = {
            let mut parser = Parser::new(code);
            parser.parse().expect("Should parse")
        };
        
        group.bench_with_input(BenchmarkId::from_parameter(name), &ast, |b, ast| {
            b.iter(|| {
                let transpiler = Transpiler::new();
                transpiler.transpile(black_box(ast))
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    transpile_literals,
    transpile_expressions,
    transpile_functions,
    transpile_patterns,
    transpile_scalability,
    transpile_real_world
);
criterion_main!(benches);