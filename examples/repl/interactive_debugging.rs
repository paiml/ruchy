//! Interactive Debugging Example for Ruchy REPL
//!
//! This example shows how to use the REPL for debugging and code exploration.
//! Run with: cargo run --example interactive_debugging

use anyhow::Result;
use ruchy::runtime::repl::Repl;

fn main() -> Result<()> {
    println!("🐛 Ruchy REPL: Interactive Debugging Example");
    println!("===========================================");
    
    let mut repl = Repl::new()?;
    
    println!("\n🔍 Step 1: Define a buggy function");
    debug_step(&mut repl, r#"
        fun factorial(n: i32) -> i32 {
            if n <= 0 { 1 } else { n * factorial(n - 1) }
        }
    "#)?;
    
    println!("\n🧪 Step 2: Test the function");
    debug_step(&mut repl, "factorial(5)")?;
    debug_step(&mut repl, "factorial(0)")?;  // Edge case
    debug_step(&mut repl, "factorial(-1)")?; // Bug: should handle negative numbers
    
    println!("\n🔧 Step 3: Debug by inspecting intermediate values");
    debug_step(&mut repl, "let test_input = 3")?;
    debug_step(&mut repl, "test_input <= 0")?;
    debug_step(&mut repl, "test_input * factorial(test_input - 1)")?;
    
    println!("\n📋 Step 4: Examine the generated Rust code");
    match repl.show_rust("factorial(5)") {
        Ok(rust_code) => {
            println!("🦀 Generated Rust code:");
            println!("{}", rust_code);
        }
        Err(e) => println!("❌ Failed to show Rust: {}", e),
    }
    
    println!("\n🌳 Step 5: Look at the AST structure");
    match repl.show_ast("factorial(5)") {
        Ok(ast) => {
            println!("🌳 AST structure:");
            let ast_lines: Vec<&str> = ast.lines().take(10).collect(); // First 10 lines
            for line in ast_lines {
                println!("   {}", line);
            }
            if ast.lines().count() > 10 {
                println!("   ... (truncated)");
            }
        }
        Err(e) => println!("❌ Failed to show AST: {}", e),
    }
    
    println!("\n🎯 Step 6: Check types for debugging");
    show_type(&mut repl, "factorial")?;
    show_type(&mut repl, "test_input")?;
    show_type(&mut repl, "5")?;
    
    println!("\n✅ Debugging session completed!");
    println!("💡 In a real session, you could fix the function and test again.");
    
    Ok(())
}

fn debug_step(repl: &mut Repl, code: &str) -> Result<()> {
    println!("🟢 Debug: {}", code.trim());
    match repl.eval(code) {
        Ok(result) => {
            if !result.trim().is_empty() {
                println!("   ✅ {}", result);
            }
        }
        Err(e) => {
            println!("   🚫 Error: {}", e);
        }
    }
    Ok(())
}

fn show_type(repl: &mut Repl, expr: &str) -> Result<()> {
    match repl.show_type(expr) {
        Ok(type_info) => println!("🏷️  Type of '{}': {}", expr, type_info),
        Err(e) => println!("❌ Type error for '{}': {}", expr, e),
    }
    Ok(())
}