//! Examples demonstrating import and module functionality in Ruchy
//!
//! Run with: cargo run --example imports
#![allow(clippy::print_stdout)] // Examples should print output
#![allow(clippy::unwrap_used)] // Examples can use unwrap for simplicity

use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{ExprKind, ImportItem};
use ruchy::backend::transpiler::Transpiler;

fn main() {
    println!("=== Ruchy Import/Module Examples ===\n");
    
    // Example 1: Simple import
    example_simple_import();
    
    // Example 2: Multiple imports from same module
    example_multiple_imports();
    
    // Example 3: Aliased imports
    example_aliased_imports();
    
    // Example 4: Wildcard imports
    example_wildcard_import();
    
    // Example 5: Module declarations
    example_module_declaration();
    
    // Example 6: Export statements
    example_exports();
    
    // Example 7: Complex import scenarios
    example_complex_imports();
}

fn example_simple_import() {
    println!("1. Simple Import Example");
    println!("------------------------");
    
    let input = "import std::collections::HashMap";
    println!("Input: {input}");
    
    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Transpiled: {output}");
    
    if let ExprKind::Import { path, items } = &ast.kind {
        println!("Path: {path}");
        println!("Items: {items:?}");
    }
    
    println!();
}

fn example_multiple_imports() {
    println!("2. Multiple Imports Example");
    println!("---------------------------");
    
    let input = "import std::io::{Read, Write, BufReader}";
    println!("Input: {input}");
    
    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Transpiled: {output}");
    
    if let ExprKind::Import { path, items } = &ast.kind {
        println!("Path: {path}");
        println!("Imported items:");
        for item in items {
            if let ImportItem::Named(name) = item {
                println!("  - {name}");
            }
        }
    }
    
    println!();
}

fn example_aliased_imports() {
    println!("3. Aliased Imports Example");
    println!("--------------------------");
    
    let examples = vec![
        "import std::collections::HashMap as Map",
        "import std::collections::{HashMap as Map, Vec as List}",
    ];
    
    for input in examples {
        println!("Input: {input}");
        
        let ast = Parser::new(input).parse().unwrap();
        let transpiler = Transpiler::new();
        let output = transpiler.transpile(&ast).unwrap().to_string();
        
        println!("Transpiled: {output}");
        
        if let ExprKind::Import { items, .. } = &ast.kind {
            for item in items {
                if let ImportItem::Aliased { name, alias } = item {
                    println!("  {name} aliased as {alias}");
                }
            }
        }
        println!();
    }
}

fn example_wildcard_import() {
    println!("4. Wildcard Import Example");
    println!("--------------------------");
    
    let input = "import std::collections::*";
    println!("Input: {input}");
    
    let ast = Parser::new(input).parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap().to_string();
    
    println!("Transpiled: {output}");
    
    if let ExprKind::Import { path, items } = &ast.kind {
        println!("Path: {path}");
        if matches!(items.first(), Some(ImportItem::Wildcard)) {
            println!("Type: Wildcard import (imports all public items)");
        }
    }
    
    println!();
}

fn example_module_declaration() {
    println!("5. Module Declaration Example");
    println!("-----------------------------");
    
    let examples = vec![
        ("Empty module", "module Utils {}"),
        ("Module with content", "module Math { 42 }"),
        ("Module with function", r"module StringUtils { fn trim(s) { s } }"),
    ];
    
    for (description, input) in examples {
        println!("{description}: {input}");
        
        let ast = Parser::new(input).parse().unwrap();
        
        if let ExprKind::Module { name, body } = &ast.kind {
            println!("  Module name: {name}");
            println!("  Has body: {}", match &body.kind {
                ExprKind::Literal(_) => "literal value",
                ExprKind::Block(_) => "block of expressions",
                ExprKind::Function { .. } => "function definition",
                _ => "other expression",
            });
        }
        println!();
    }
}

fn example_exports() {
    println!("6. Export Statements Example");
    println!("----------------------------");
    
    let examples = vec![
        "export myFunction",
        "export { add, subtract, multiply, divide }",
    ];
    
    for input in examples {
        println!("Input: {input}");
        
        let ast = Parser::new(input).parse().unwrap();
        
        if let ExprKind::Export { items } = &ast.kind {
            println!("Exported items:");
            for item in items {
                println!("  - {item}");
            }
        }
        println!();
    }
}

fn example_complex_imports() {
    println!("7. Complex Import Scenarios");
    println!("----------------------------");
    
    // Test various import patterns
    let test_cases = vec![
        ("Nested path", "import std::sync::Arc"),
        ("Deep nesting", "import tokio::net::tcp::TcpStream"),
        ("Mixed items", "import std::collections::{HashMap, Vec as List, BTreeMap}"),
    ];
    
    for (description, input) in test_cases {
        println!("{description}: {input}");
        
        match Parser::new(input).parse() {
            Ok(ast) => {
                let transpiler = Transpiler::new();
                match transpiler.transpile(&ast) {
                    Ok(tokens) => {
                        println!("  ✓ Successfully transpiled to: {tokens}");
                    }
                    Err(e) => {
                        println!("  ✗ Transpilation error: {e}");
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Parse error: {e}");
            }
        }
        println!();
    }
    
    println!("=== Examples Complete ===");
}