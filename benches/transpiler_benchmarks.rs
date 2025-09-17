//! Transpiler performance benchmarks
//!
//! Measures performance of transpiling Ruchy code to Rust.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn parse_and_transpile(code: &str) -> String {
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    transpiler.transpile(&ast).unwrap()
}

fn benchmark_simple_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_literal", |b| {
        b.iter(|| {
            parse_and_transpile(black_box("42"))
        })
    });

    c.bench_function("transpile_arithmetic", |b| {
        b.iter(|| {
            parse_and_transpile(black_box("2 + 3 * 4 - 5"))
        })
    });

    c.bench_function("transpile_variable", |b| {
        b.iter(|| {
            parse_and_transpile(black_box("let x = 42; x + 1"))
        })
    });

    c.bench_function("transpile_string", |b| {
        b.iter(|| {
            parse_and_transpile(black_box(r#""hello, world""#))
        })
    });

    c.bench_function("transpile_string_interpolation", |b| {
        b.iter(|| {
            parse_and_transpile(black_box(r#"f"The value is {x} and {y}""#))
        })
    });
}

fn benchmark_control_flow_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_if_else", |b| {
        let code = "if x > 0 { x * 2 } else { x * -1 }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_match", |b| {
        let code = r#"
            match value {
                Some(x) => x * 2,
                None => 0
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_for_loop", |b| {
        let code = "for i in 0..10 { sum = sum + i }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_while_loop", |b| {
        let code = "while x < 100 { x = x * 2 }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_function_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_function", |b| {
        let code = "fn add(x: int, y: int) -> int { x + y }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_lambda", |b| {
        let code = "x => x * 2";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_recursive_function", |b| {
        let code = r#"
            fn factorial(n: int) -> int {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_generic_function", |b| {
        let code = "fn identity<T>(x: T) -> T { x }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_data_structure_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_list", |b| {
        let code = "[1, 2, 3, 4, 5]";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_tuple", |b| {
        let code = "(1, \"hello\", true)";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_object", |b| {
        let code = "{ name: \"John\", age: 30, active: true }";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_nested_structures", |b| {
        let code = "[[1, 2], [3, 4], [5, 6]]";
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_class_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_struct", |b| {
        let code = r#"
            struct Point {
                x: float,
                y: float
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_impl_block", |b| {
        let code = r#"
            impl Point {
                fn new(x: float, y: float) -> Point {
                    Point { x: x, y: y }
                }

                fn distance(self, other: Point) -> float {
                    sqrt((self.x - other.x) ** 2 + (self.y - other.y) ** 2)
                }
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_trait", |b| {
        let code = r#"
            trait Drawable {
                fn draw(self);
                fn get_bounds(self) -> Rect;
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_trait_impl", |b| {
        let code = r#"
            impl Drawable for Circle {
                fn draw(self) {
                    draw_circle(self.center, self.radius)
                }

                fn get_bounds(self) -> Rect {
                    Rect {
                        x: self.center.x - self.radius,
                        y: self.center.y - self.radius,
                        width: self.radius * 2,
                        height: self.radius * 2
                    }
                }
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_async_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_async_function", |b| {
        let code = r#"
            async fn fetch_data(url: string) -> Result<Data, Error> {
                let response = await http::get(url);
                await response.json()
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_async_block", |b| {
        let code = r#"
            async {
                let data = await fetch_data("https://api.example.com");
                process(data)
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_complex_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_real_world_function", |b| {
        let code = r#"
            fn process_data(items: List<Item>) -> Result<Summary, Error> {
                let mut total = 0;
                let mut count = 0;
                let mut errors = [];

                for item in items {
                    match validate(item) {
                        Ok(validated) => {
                            total = total + validated.value;
                            count = count + 1
                        },
                        Err(e) => {
                            errors.push(e)
                        }
                    }
                }

                if errors.len() > 0 {
                    Err(Error::ValidationErrors(errors))
                } else if count == 0 {
                    Err(Error::NoData)
                } else {
                    Ok(Summary {
                        total: total,
                        count: count,
                        average: total / count
                    })
                }
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_complex_match", |b| {
        let code = r#"
            match result {
                Ok(Some(User { id, name, age, .. })) if age >= 18 => {
                    println(f"Adult user: {name} (ID: {id})")
                },
                Ok(Some(User { name, age, .. })) => {
                    println(f"Minor user: {name} (age: {age})")
                },
                Ok(None) => {
                    println("User not found")
                },
                Err(DatabaseError::Connection(msg)) => {
                    log::error(f"Database connection error: {msg}");
                    retry()
                },
                Err(e) => {
                    log::error(f"Unexpected error: {e}");
                    panic(e)
                }
            }
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });

    c.bench_function("transpile_pipeline_operations", |b| {
        let code = r#"
            data
                |> filter(x => x.active)
                |> map(x => x.value * 2)
                |> group_by(x => x.category)
                |> map_values(group => group.sum())
                |> sort_by_value()
                |> take(10)
        "#;
        b.iter(|| {
            parse_and_transpile(black_box(code))
        })
    });
}

fn benchmark_stress_test_transpilation(c: &mut Criterion) {
    c.bench_function("transpile_deeply_nested", |b| {
        let mut code = String::new();
        for _ in 0..15 {
            code.push_str("if true { ");
        }
        code.push_str("42");
        for _ in 0..15 {
            code.push_str(" }");
        }
        b.iter(|| {
            parse_and_transpile(black_box(&code))
        })
    });

    c.bench_function("transpile_many_functions", |b| {
        let mut code = String::new();
        for i in 0..20 {
            code.push_str(&format!("fn func{}(x: int) -> int {{ x + {} }}\n", i, i));
        }
        b.iter(|| {
            parse_and_transpile(black_box(&code))
        })
    });

    c.bench_function("transpile_large_struct", |b| {
        let mut code = String::from("struct LargeStruct {\n");
        for i in 0..50 {
            code.push_str(&format!("    field{}: int,\n", i));
        }
        code.push_str("}");
        b.iter(|| {
            parse_and_transpile(black_box(&code))
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_transpilation,
    benchmark_control_flow_transpilation,
    benchmark_function_transpilation,
    benchmark_data_structure_transpilation,
    benchmark_class_transpilation,
    benchmark_async_transpilation,
    benchmark_complex_transpilation,
    benchmark_stress_test_transpilation
);

criterion_main!(benches);