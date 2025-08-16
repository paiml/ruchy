//! String Interpolation Example for Ruchy
//!
//! This example demonstrates how string interpolation works in Ruchy,
//! showing parsing and transpilation to Rust code.
//! 
//! Run with: cargo run --example `string_interpolation_demo`

#![allow(clippy::print_stdout)] // Example needs to print
#![allow(clippy::expect_used)]   // Example can panic

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

fn main() {
    println!("🎨 Ruchy String Interpolation Demo");
    println!("=================================");
    
    // Example 1: Simple interpolation
    println!("\n📝 Example 1: Simple interpolation");
    demo_interpolation("\"Hello, {name}!\"");
    
    // Example 2: Multiple expressions
    println!("\n📝 Example 2: Multiple expressions");
    demo_interpolation("\"User {user_id}: {first_name} {last_name}\"");
    
    // Example 3: Complex expressions
    println!("\n📝 Example 3: Complex expressions");
    demo_interpolation("\"Result: {x + y} (calculated at {timestamp()})\"");
    
    // Example 4: Escaped braces
    println!("\n📝 Example 4: Escaped braces");
    demo_interpolation("\"JSON: {{\\\"key\\\": {value}}}\"");
    
    // Example 5: No interpolation
    println!("\n📝 Example 5: Regular string (no interpolation)");
    demo_interpolation("\"This is just a regular string\"");
    
    // Example 6: Nested expressions
    println!("\n📝 Example 6: Nested function calls");
    demo_interpolation("\"Processing {process_data(input.filter(valid))}\"");
    
    println!("\n✅ String Interpolation Demo Complete!");
    println!("\n🎯 Key Features:");
    println!("  • Simple variable interpolation: {{var}}");
    println!("  • Complex expression support: {{expr + other}}");
    println!("  • Function call interpolation: {{func(args)}}");
    println!("  • Escaped braces: {{literal braces}}");
    println!("  • Transpiles to efficient Rust format! calls");
}

fn demo_interpolation(input: &str) {
    println!("🔸 Input: {input}");
    
    // Parse the string
    let mut parser = Parser::new(input);
    match parser.parse() {
        Ok(expr) => {
            println!("  ✅ Parsed successfully");
            
            // Show the AST structure
            match &expr.kind {
                ruchy::frontend::ast::ExprKind::StringInterpolation { parts } => {
                    println!("  📊 String interpolation with {} parts:", parts.len());
                    for (i, part) in parts.iter().enumerate() {
                        match part {
                            ruchy::frontend::ast::StringPart::Text(text) => {
                                println!("    {}. Text: {:?}", i + 1, text);
                            }
                            ruchy::frontend::ast::StringPart::Expr(_) => {
                                println!("    {}. Expression: {{...}}", i + 1);
                            }
                        }
                    }
                }
                ruchy::frontend::ast::ExprKind::Literal(ruchy::frontend::ast::Literal::String(s)) => {
                    println!("  📄 Regular string literal: {s:?}");
                }
                _ => {
                    println!("  ❓ Unexpected expression type");
                }
            }
            
            // Transpile to Rust
            let transpiler = Transpiler::new();
            match transpiler.transpile(&expr) {
                Ok(rust_tokens) => {
                    let rust_code = rust_tokens.to_string();
                    println!("  🦀 Rust code: {rust_code}");
                }
                Err(e) => {
                    println!("  ❌ Transpilation error: {e}");
                }
            }
        }
        Err(e) => {
            println!("  ❌ Parse error: {e}");
        }
    }
}

/// Example of how string interpolation would be used in actual Ruchy code
#[allow(dead_code)]
fn usage_examples() {
    // These are conceptual examples of how string interpolation
    // would work in actual Ruchy programs
    
    // Simple greeting
    // let name = "Alice";
    // let greeting = "Hello, {name}!";
    
    // Complex formatting
    // let user_id = 123;
    // let score = calculate_score();
    // let message = "User {user_id} scored {score} points!";
    
    // With calculations
    // let x = 10;
    // let y = 20;
    // let result = "Sum: {x + y}, Product: {x * y}";
    
    // Function calls
    // let timestamp = get_timestamp();
    // let log_entry = "Event at {timestamp}: {get_event_details()}";
}