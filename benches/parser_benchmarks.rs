//! Parser performance benchmarks
//!
//! Measures performance of the Ruchy parser on various code patterns.

use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use ruchy::frontend::parser::Parser;

fn benchmark_literals(c: &mut Criterion) {
    c.bench_function("parse_integer", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box("42"));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_float", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box("3.14159"));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_string", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(r#""hello world""#));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_interpolated_string", |b| {
        let code = r#"f"The answer is {x + y} and pi is {pi:.2}""#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_expressions(c: &mut Criterion) {
    c.bench_function("parse_arithmetic", |b| {
        let code = "2 + 3 * 4 - 5 / 2";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_nested_arithmetic", |b| {
        let code = "((2 + 3) * (4 - 1)) / (5 + (6 * 7))";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_method_chain", |b| {
        let code = "list.filter(x => x > 0).map(x => x * 2).reduce(0, (a, b) => a + b)";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_pipeline", |b| {
        let code = "data |> filter(x => x.active) |> map(x => x.value) |> sum()";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_control_flow(c: &mut Criterion) {
    c.bench_function("parse_if_else", |b| {
        let code = "if x > 0 { x * 2 } else if x < 0 { x * -1 } else { 0 }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_match", |b| {
        let code = r#"
            match value {
                Some(x) if x > 0 => x * 2,
                Some(x) => x,
                None => 0,
                _ => -1
            }
        "#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_for_loop", |b| {
        let code = "for i in 0..100 { sum += i * i }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_while_loop", |b| {
        let code = "while x < 100 { x = x * 2 + 1 }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("parse_function_def", |b| {
        let code = "fn factorial(n: int) -> int { if n <= 1 { 1 } else { n * factorial(n - 1) } }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_lambda", |b| {
        let code = "x => x * 2";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_nested_lambda", |b| {
        let code = "x => y => z => x + y + z";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_generic_function", |b| {
        let code = "fn map<T, U>(list: List<T>, f: T -> U) -> List<U> { list.map(f) }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_data_structures(c: &mut Criterion) {
    c.bench_function("parse_list", |b| {
        let code = "[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_nested_list", |b| {
        let code = "[[1, 2], [3, 4], [5, 6], [7, 8], [9, 10]]";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_object", |b| {
        let code = "{ name: \"John\", age: 30, active: true, scores: [85, 90, 95] }";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_tuple", |b| {
        let code = "(1, \"hello\", true, 3.14, [1, 2, 3])";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_patterns(c: &mut Criterion) {
    c.bench_function("parse_simple_pattern", |b| {
        let code = "let x = 42";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_destructuring", |b| {
        let code = "let [first, second, ...rest] = list";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_object_destructuring", |b| {
        let code = "let { name, age, ...other } = person";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_nested_destructuring", |b| {
        let code = "let { user: { name, email }, timestamp } = event";
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_real_world(c: &mut Criterion) {
    c.bench_function("parse_fibonacci", |b| {
        let code = r#"
            fn fibonacci(n: int) -> int {
                if n <= 1 {
                    n
                } else {
                    fibonacci(n - 1) + fibonacci(n - 2)
                }
            }
        "#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_quicksort", |b| {
        let code = r#"
            fn quicksort(arr: List<int>) -> List<int> {
                if arr.len() <= 1 {
                    arr
                } else {
                    let pivot = arr[0]
                    let less = arr[1..].filter(x => x < pivot)
                    let greater = arr[1..].filter(x => x >= pivot)
                    quicksort(less) + [pivot] + quicksort(greater)
                }
            }
        "#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_class_definition", |b| {
        let code = r#"
            struct Person {
                name: string,
                age: int,
                email: Option<string>
            }

            impl Person {
                fn new(name: string, age: int) -> Person {
                    Person { name: name, age: age, email: None }
                }

                fn greet(self) -> string {
                    f"Hello, my name is {self.name}"
                }

                fn is_adult(self) -> bool {
                    self.age >= 18
                }
            }
        "#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_async_function", |b| {
        let code = r#"
            async fn fetch_data(url: string) -> Result<Data, Error> {
                let response = await http::get(url)
                if response.status == 200 {
                    let data = await response.json()
                    Ok(data)
                } else {
                    Err(Error::HttpError(response.status))
                }
            }
        "#;
        b.iter(|| {
            let mut parser = Parser::new(black_box(code));
            parser.parse().unwrap()
        })
    });
}

fn benchmark_stress_test(c: &mut Criterion) {
    c.bench_function("parse_deeply_nested", |b| {
        let mut code = String::new();
        for _ in 0..20 {
            code.push_str("if true { ");
        }
        code.push_str("42");
        for _ in 0..20 {
            code.push_str(" }");
        }
        b.iter(|| {
            let mut parser = Parser::new(black_box(&code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_long_chain", |b| {
        let mut code = String::from("x");
        for i in 0..50 {
            code.push_str(&format!(".method{}()", i));
        }
        b.iter(|| {
            let mut parser = Parser::new(black_box(&code));
            parser.parse().unwrap()
        })
    });

    c.bench_function("parse_large_list", |b| {
        let mut code = String::from("[");
        for i in 0..100 {
            if i > 0 {
                code.push_str(", ");
            }
            code.push_str(&i.to_string());
        }
        code.push(']');
        b.iter(|| {
            let mut parser = Parser::new(black_box(&code));
            parser.parse().unwrap()
        })
    });
}

criterion_group!(
    benches,
    benchmark_literals,
    benchmark_expressions,
    benchmark_control_flow,
    benchmark_functions,
    benchmark_data_structures,
    benchmark_patterns,
    benchmark_real_world,
    benchmark_stress_test
);

criterion_main!(benches);
