//! Examples demonstrating error handling with Result types in Ruchy
//!
//! Run with: `cargo run --example error_handling`
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::ExprKind;
use ruchy::frontend::parser::Parser;

fn main() {
    println!("=== Ruchy Error Handling Examples ===\n");

    // Example 1: Result constructors
    example_result_constructors();

    // Example 2: Try operator (?)
    example_try_operator();

    // Example 3: Match on Result
    example_match_result();

    // Example 4: Error propagation
    example_error_propagation();
}

fn example_result_constructors() {
    println!("1. Result Constructors");
    println!("----------------------");

    let examples = vec![
        ("Ok value", "Ok(42)"),
        ("Error value", r#"Err("something went wrong")"#),
        ("Ok with complex type", "Ok([1, 2, 3])"),
    ];

    for (description, input) in examples {
        println!("{description}: {input}");

        let ast = Parser::new(input).parse().unwrap();
        let transpiler = Transpiler::new();
        let output = transpiler.transpile(&ast).unwrap();

        println!("  Transpiled: {output}");
    }
    println!();
}

fn example_try_operator() {
    println!("2. Try Operator (?)");
    println!("-------------------");

    let examples = vec![
        ("Simple try", "value?"),
        ("Try with call", "get_data()?"),
        ("Chained try", "fetch()?.process()?"),
    ];

    for (description, input) in examples {
        println!("{description}: {input}");

        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(output) => println!("  Transpiled: {output}"),
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
            Err(e) => println!("  ✗ Parse error: {e}"),
        }
    }
    println!();
}

fn example_match_result() {
    println!("3. Match on Result");
    println!("------------------");

    let input = r#"
        match get_result() {
            Ok(value) => value * 2,
            Err(e) => {
                println("Error: {e}")
                0
            }
        }
    "#;

    println!("Pattern matching on Result:");
    println!("{}", &input[..80]);
    println!("...");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Match { .. } = &ast.kind {
                println!("✓ Parsed as match expression");
            }

            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(output) => {
                    let output_str = output.to_string();
                    if output_str.contains("Ok") && output_str.contains("Err") {
                        println!("✓ Contains Ok and Err patterns");
                    }
                }
                Err(e) => println!("✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("✗ Parse error: {e}"),
    }
    println!();
}

fn example_error_propagation() {
    println!("4. Error Propagation");
    println!("--------------------");

    let input = r"
        fun process_data() -> Result<Data, Error> {
            let raw = fetch_raw_data()?
            let validated = validate(raw)?
            let processed = transform(validated)?
            Ok(processed)
        }
    ";

    println!("Function with error propagation:");
    println!("  - Multiple ? operators");
    println!("  - Returns Result type");
    println!("  - Explicit Ok() for success");

    match Parser::new(input).parse() {
        Ok(ast) => {
            if let ExprKind::Function {
                return_type: Some(ret_type),
                ..
            } = &ast.kind
            {
                println!("  Return type: {ret_type:?}");
            }

            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(output) => {
                    let output_str = output.to_string();
                    let question_marks = output_str.matches('?').count();
                    println!("  ✓ Contains {question_marks} ? operators");
                    if output_str.contains("Ok") {
                        println!("  ✓ Returns Ok for success");
                    }
                }
                Err(e) => println!("  ✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("  ✗ Parse error: {e}"),
    }

    println!("\n=== Error Handling Examples Complete ===");
}
