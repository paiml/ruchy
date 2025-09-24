//! Interpreter performance benchmarks
//!
//! Measures execution performance of the Ruchy interpreter.

use criterion::{criterion_group, criterion_main, Criterion};
use ruchy::runtime::repl::Repl;
use std::hint::black_box;
use std::path::PathBuf;

fn benchmark_arithmetic(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_integer", |b| {
        b.iter(|| repl.eval(black_box("42")).unwrap())
    });

    c.bench_function("eval_simple_arithmetic", |b| {
        b.iter(|| repl.eval(black_box("2 + 3 * 4")).unwrap())
    });

    c.bench_function("eval_complex_arithmetic", |b| {
        b.iter(|| {
            repl.eval(black_box("((2 + 3) * (4 - 1)) / (5 + (6 * 7))"))
                .unwrap()
        })
    });

    c.bench_function("eval_float_arithmetic", |b| {
        b.iter(|| repl.eval(black_box("3.14 * 2.0 + 1.5 / 0.5")).unwrap())
    });

    c.bench_function("eval_power", |b| {
        b.iter(|| repl.eval(black_box("2 ** 10")).unwrap())
    });
}

fn benchmark_variables(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_variable_assignment", |b| {
        b.iter(|| repl.eval(black_box("let x = 42")).unwrap())
    });

    c.bench_function("eval_variable_lookup", |b| {
        repl.eval("let x = 42").unwrap();
        b.iter(|| repl.eval(black_box("x")).unwrap())
    });

    c.bench_function("eval_multiple_variables", |b| {
        b.iter(|| {
            repl.eval(black_box("let x = 1; let y = 2; let z = 3; x + y + z"))
                .unwrap()
        })
    });

    c.bench_function("eval_nested_scopes", |b| {
        b.iter(|| {
            repl.eval(black_box(
                "let x = 1; let result = { let y = 2; { let z = 3; x + y + z } }; result",
            ))
            .unwrap()
        })
    });
}

fn benchmark_functions(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_lambda", |b| {
        b.iter(|| repl.eval(black_box("(x => x * 2)(21)")).unwrap())
    });

    c.bench_function("eval_function_call", |b| {
        repl.eval("fn double(x) { x * 2 }").unwrap();
        b.iter(|| repl.eval(black_box("double(21)")).unwrap())
    });

    c.bench_function("eval_recursive_factorial", |b| {
        repl.eval(
            r"
            fn factorial(n) {
                if n <= 1 { 1 } else { n * factorial(n - 1) }
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("factorial(5)")).unwrap())
    });

    c.bench_function("eval_fibonacci", |b| {
        repl.eval(
            r"
            fn fib(n) {
                if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("fib(10)")).unwrap())
    });

    c.bench_function("eval_higher_order", |b| {
        repl.eval(
            r"
            fn apply_twice(f, x) {
                f(f(x))
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("apply_twice(x => x * 2, 5)")).unwrap())
    });
}

fn benchmark_control_flow(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_if_else", |b| {
        b.iter(|| repl.eval(black_box("if true { 1 } else { 0 }")).unwrap())
    });

    c.bench_function("eval_nested_if", |b| {
        b.iter(|| {
            repl
                .eval(black_box(
                    "let x = 5; if x > 10 { \"large\" } else if x > 5 { \"medium\" } else if x > 0 { \"small\" } else { \"zero or negative\" }"
                ))
                .unwrap()
        })
    });

    c.bench_function("eval_match", |b| {
        b.iter(|| {
            repl.eval(black_box(
                "let x = 2; match x { 1 => \"one\", 2 => \"two\", 3 => \"three\", _ => \"other\" }",
            ))
            .unwrap()
        })
    });

    c.bench_function("eval_for_loop", |b| {
        b.iter(|| {
            repl.eval(black_box(
                "let sum = 0; for i in [1, 2, 3, 4, 5] { sum = sum + i }; sum",
            ))
            .unwrap()
        })
    });

    c.bench_function("eval_while_loop", |b| {
        b.iter(|| {
            repl.eval(black_box(
                "let x = 1; let count = 0; while x < 100 { x = x * 2; count = count + 1 }; count",
            ))
            .unwrap()
        })
    });
}

fn benchmark_data_structures(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_list_creation", |b| {
        b.iter(|| {
            repl.eval(black_box("[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]"))
                .unwrap()
        })
    });

    c.bench_function("eval_list_indexing", |b| {
        repl.eval("let list = [1, 2, 3, 4, 5]").unwrap();
        b.iter(|| repl.eval(black_box("list[2]")).unwrap())
    });

    c.bench_function("eval_tuple_creation", |b| {
        b.iter(|| repl.eval(black_box("(1, \"hello\", true, 3.14)")).unwrap())
    });

    c.bench_function("eval_nested_structures", |b| {
        b.iter(|| repl.eval(black_box("[[1, 2], [3, 4], [5, 6]]")).unwrap())
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_string_concatenation", |b| {
        b.iter(|| repl.eval(black_box(r#""hello" + " " + "world""#)).unwrap())
    });

    c.bench_function("eval_string_interpolation", |b| {
        repl.eval("let name = \"Ruchy\"").unwrap();
        repl.eval("let version = 3").unwrap();
        b.iter(|| {
            repl.eval(black_box(r#"f"Welcome to {name} v{version}!""#))
                .unwrap()
        })
    });

    c.bench_function("eval_string_methods", |b| {
        repl.eval(r#"let s = "hello world""#).unwrap();
        b.iter(|| repl.eval(black_box("s.len()")).unwrap())
    });
}

fn benchmark_builtin_functions(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_max", |b| {
        b.iter(|| repl.eval(black_box("max(5, 10)")).unwrap())
    });

    c.bench_function("eval_min", |b| {
        b.iter(|| repl.eval(black_box("min(5, 10)")).unwrap())
    });

    c.bench_function("eval_abs", |b| {
        b.iter(|| repl.eval(black_box("abs(-42)")).unwrap())
    });

    c.bench_function("eval_sqrt", |b| {
        b.iter(|| repl.eval(black_box("sqrt(16.0)")).unwrap())
    });

    c.bench_function("eval_floor", |b| {
        b.iter(|| repl.eval(black_box("floor(3.7)")).unwrap())
    });
}

fn benchmark_real_world(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_sum_of_squares", |b| {
        repl.eval(
            r"
            fn sum_of_squares(n) {
                let sum = 0;
                for i in 1..=n {
                    sum = sum + i * i
                };
                sum
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("sum_of_squares(10)")).unwrap())
    });

    c.bench_function("eval_bubble_sort", |b| {
        repl.eval(
            r"
            fn bubble_sort(arr) {
                let n = arr.len();
                for i in 0..n {
                    for j in 0..(n - i - 1) {
                        if arr[j] > arr[j + 1] {
                            let temp = arr[j];
                            arr[j] = arr[j + 1];
                            arr[j + 1] = temp
                        }
                    }
                };
                arr
            }
        ",
        )
        .unwrap();
        b.iter(|| {
            repl.eval(black_box("bubble_sort([5, 2, 8, 1, 9, 3, 7, 4, 6])"))
                .unwrap()
        })
    });

    c.bench_function("eval_prime_check", |b| {
        repl.eval(
            r"
            fn is_prime(n) {
                if n <= 1 {
                    false
                } else {
                    let is_prime = true;
                    for i in 2..(n / 2 + 1) {
                        if n % i == 0 {
                            is_prime = false;
                            break
                        }
                    };
                    is_prime
                }
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("is_prime(97)")).unwrap())
    });

    c.bench_function("eval_gcd", |b| {
        repl.eval(
            r"
            fn gcd(a, b) {
                if b == 0 { a } else { gcd(b, a % b) }
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("gcd(48, 18)")).unwrap())
    });
}

fn benchmark_stress_test(c: &mut Criterion) {
    let mut repl = Repl::new(PathBuf::from(".")).expect("Failed to create REPL");

    c.bench_function("eval_deep_recursion", |b| {
        repl.eval(
            r"
            fn count_down(n) {
                if n <= 0 { 0 } else { count_down(n - 1) + 1 }
            }
        ",
        )
        .unwrap();
        b.iter(|| repl.eval(black_box("count_down(20)")).unwrap())
    });

    c.bench_function("eval_many_variables", |b| {
        let mut code = String::new();
        for i in 0..50 {
            code.push_str(&format!("let v{} = {}; ", i, i));
        }
        code.push_str("v0 + v49");
        b.iter(|| repl.eval(black_box(&code)).unwrap())
    });

    c.bench_function("eval_long_expression", |b| {
        let mut code = String::from("1");
        for i in 2..=20 {
            code.push_str(&format!(" + {}", i));
        }
        b.iter(|| repl.eval(black_box(&code)).unwrap())
    });
}

criterion_group!(
    benches,
    benchmark_arithmetic,
    benchmark_variables,
    benchmark_functions,
    benchmark_control_flow,
    benchmark_data_structures,
    benchmark_string_operations,
    benchmark_builtin_functions,
    benchmark_real_world,
    benchmark_stress_test
);

criterion_main!(benches);
