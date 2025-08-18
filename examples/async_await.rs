//! Examples demonstrating async/await functionality in Ruchy
//!
//! Run with: `cargo run --example async_await`
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::ExprKind;
use ruchy::backend::transpiler::Transpiler;

fn main() {
    println!("=== Ruchy Async/Await Examples ===\n");
    
    // Example 1: Async function
    example_async_function();
    
    // Example 2: Await expression
    example_await();
    
    // Example 3: Async with error handling
    example_async_error_handling();
    
    // Example 4: Multiple async operations
    example_multiple_async();
}

fn example_async_function() {
    println!("1. Async Function");
    println!("-----------------");
    
    let input = r#"
        async fun fetch_data() -> String {
            "data from server"
        }
    "#;
    println!("Input: {input}");
    
    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    
    println!("Transpiled to Rust async function:");
    let output_str = output.to_string();
    if output_str.len() > 100 {
        println!("{}", &output_str[..100]);
    } else {
        println!("{output_str}");
    }
    println!();
}

fn example_await() {
    println!("2. Await Expression");
    println!("-------------------");
    
    let input = "await fetch_data()";
    println!("Input: {input}");
    
    let ast = Parser::new(input).parse().unwrap();
    
    if let ExprKind::Await { expr } = &ast.kind {
        println!("Awaiting expression: {expr:?}");
    }
    
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    println!("Transpiled: {output}");
    println!();
}

fn example_async_error_handling() {
    println!("3. Async with Error Handling");
    println!("-----------------------------");
    
    let input = r"
        async fun fetch_with_retry() -> Result<String, Error> {
            let result = await fetch_data()?
            Ok(result)
        }
    ";
    println!("Input: {input}");
    
    match Parser::new(input).parse() {
        Ok(ast) => {
            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(tokens) => {
                    println!("✓ Successfully transpiled async function with error handling");
                    let output = tokens.to_string();
                    if output.contains("async") && output.contains(".await") {
                        println!("  Contains async and .await ✓");
                    }
                }
                Err(e) => println!("✗ Transpilation error: {e}"),
            }
        }
        Err(e) => println!("✗ Parse error: {e}"),
    }
    println!();
}

fn example_multiple_async() {
    println!("4. Multiple Async Operations");
    println!("-----------------------------");
    
    let examples = vec![
        ("Async block", "async { 42 }"),
        ("Await in expression", "let x = await get_value()"),
        ("Chained await", "await fetch().process()"),
        ("Async lambda", "async |x| x + 1"),
    ];
    
    for (description, input) in examples {
        println!("{description}: {input}");
        
        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        let output = tokens.to_string();
                        if output.contains("async") || output.contains(".await") {
                            println!("  ✓ Contains async/await constructs");
                        } else {
                            println!("  Output: {}", &output[..output.len().min(50)]);
                        }
                    }
                    Err(e) => println!("  ✗ Transpilation error: {e}"),
                }
            }
            Err(e) => println!("  ✗ Parse error: {e}"),
        }
    }
    
    println!("\n=== Async/Await Examples Complete ===");
}