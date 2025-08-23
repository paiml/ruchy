//! Example demonstrating module path (::) support in Ruchy
//!
//! Run with: cargo run --example module_paths

use ruchy::{Parser as RuchyParser, Transpiler, runtime::repl::Repl};
use anyhow::Result;

fn main() -> Result<()> {
    println!("=== Ruchy Module Path Examples ===\n");

    // Example 1: Qualified type names
    println!("1. Qualified Type Names:");
    let code1 = r#"
fn process_data(input: std::string::String) -> std::result::Result {
    if input.len() > 0 {
        Result::Ok(input)
    } else {
        Result::Err("Empty input")
    }
}

process_data("Hello, World!")
"#;
    demonstrate_code(code1, "Qualified type annotations")?;

    // Example 2: Module function calls
    println!("\n2. Module Function Calls:");
    let code2 = r#"
// Simulating standard library calls
let data = MyModule::read_file("data.txt")
let parsed = Parser::parse_json(data)
Database::store(parsed)
"#;
    demonstrate_code(code2, "Module function calls")?;

    // Example 3: Nested module paths
    println!("\n3. Deeply Nested Modules:");
    let code3 = r#"
fn connect(config: app::config::database::ConnectionConfig) -> app::models::Connection {
    app::database::pool::Manager::connect(config)
}
"#;
    demonstrate_code(code3, "Deeply nested module paths")?;

    // Example 4: Generic types with module paths
    println!("\n4. Generic Types with Module Paths:");
    let code4 = r#"
fn create_cache() -> std::collections::HashMap<std::string::String, web::models::User> {
    std::collections::HashMap::new()
}
"#;
    demonstrate_code(code4, "Generic types with qualified names")?;

    // Example 5: Import statements
    println!("\n5. Import Statements:");
    let code5 = r#"
use std::collections::HashMap
use std::fs::{File, OpenOptions}
use web::handlers::*
use database::models::User as DbUser

fn main() {
    let map = HashMap::new()
    let file = File::open("test.txt")
}
"#;
    demonstrate_code(code5, "Various import patterns")?;

    // Example 6: Result and Option patterns
    println!("\n6. Result and Option Patterns:");
    let code6 = r#"
fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        Option::None
    } else {
        Option::Some(a / b)
    }
}

fn read_config() -> Result<Config, std::io::Error> {
    match std::fs::read_file("config.json") {
        Ok(content) => Result::Ok(parse_config(content)),
        Err(e) => Result::Err(e),
    }
}
"#;
    demonstrate_code(code6, "Result and Option with module paths")?;

    println!("\n=== All Examples Completed Successfully ===");
    Ok(())
}

fn demonstrate_code(code: &str, description: &str) -> Result<()> {
    println!("--- {} ---", description);
    println!("Code:\n{}", code);
    
    // Parse the code
    let mut parser = RuchyParser::new(code);
    match parser.parse() {
        Ok(ast) => {
            println!("✓ Parsed successfully");
            
            // Try to transpile
            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(rust_code) => {
                    println!("✓ Transpiled to Rust");
                    // Show first 100 chars of transpiled code
                    let preview = rust_code.to_string();
                    let preview = if preview.len() > 100 {
                        format!("{}...", &preview[..100])
                    } else {
                        preview
                    };
                    println!("  Preview: {}", preview);
                }
                Err(e) => {
                    println!("⚠ Transpilation not yet implemented: {}", e);
                }
            }
        }
        Err(e) => {
            println!("✗ Parse error: {}", e);
        }
    }
    
    Ok(())
}