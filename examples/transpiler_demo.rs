#![allow(clippy::print_stdout)]

use ruchy::{Parser, Transpiler};

fn main() {
    let examples = vec![
        ("Simple arithmetic", "1 + 2 * 3"),
        (
            "Function definition",
            "fun add(x: i32, y: i32) -> i32 { x + y }",
        ),
        ("If expression", "if x > 0 { positive } else { negative }"),
        ("List creation", "[1, 2, 3]"),
        ("Let binding", "let x = 42 in x + 1"),
        ("Pipeline", "[1, 2, 3] |> map(double) |> filter(even)"),
        (
            "Fibonacci",
            "fun fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }",
        ),
    ];

    let transpiler = Transpiler::new();

    for (name, input) in examples {
        println!("\n=== {name} ===");
        println!("Ruchy input:");
        println!("{input}");

        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(ast) => match transpiler.transpile_to_string(&ast) {
                Ok(rust_code) => {
                    println!("\nTranspiled Rust code:");
                    println!("{rust_code}");
                }
                Err(e) => {
                    println!("✗ Transpilation error: {e}");
                }
            },
            Err(e) => {
                println!("✗ Parse error: {e}");
            }
        }
    }
}
