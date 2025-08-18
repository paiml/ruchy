//! REPL demonstration example
//!
//! This example shows various REPL features and usage patterns

use anyhow::Result;
use ruchy::runtime::Repl;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    println!("REPL Demo - Testing various REPL features\n");
    
    // Create a new REPL instance
    let mut repl = Repl::new()?;
    
    // Test basic arithmetic
    println!("Testing basic arithmetic:");
    test_expression(&mut repl, "2 + 2")?;
    test_expression(&mut repl, "10 * 5")?;
    test_expression(&mut repl, "100 / 4")?;
    
    // Test variables
    println!("\nTesting variables:");
    test_expression(&mut repl, "let x = 42")?;
    test_expression(&mut repl, "let y = 10")?;
    test_expression(&mut repl, "x + y")?;
    
    // Test strings
    println!("\nTesting strings:");
    test_expression(&mut repl, r#""Hello, " + "World!""#)?;
    test_expression(&mut repl, r#"let name = "Ruchy""#)?;
    
    // Test booleans
    println!("\nTesting booleans:");
    test_expression(&mut repl, "true && false")?;
    test_expression(&mut repl, "true || false")?;
    test_expression(&mut repl, "!true")?;
    
    // Test if expressions
    println!("\nTesting if expressions:");
    test_expression(&mut repl, "if true { 100 } else { 200 }")?;
    test_expression(&mut repl, "if 5 > 3 { \"yes\" } else { \"no\" }")?;
    
    // Test blocks
    println!("\nTesting blocks:");
    test_expression(&mut repl, "{ 1 + 1 }")?;
    test_expression(&mut repl, "{ let a = 5; a }")?;
    
    // Test function calls
    println!("\nTesting function calls:");
    test_expression(&mut repl, r#"println("Hello from REPL!")"#)?;
    test_expression(&mut repl, r#"print("No newline")"#)?;
    
    // Test error handling
    println!("\nTesting error handling:");
    test_expression(&mut repl, "undefined_var")?; // Should error
    test_expression(&mut repl, "1 / 0")?; // Should error
    
    println!("\n✅ REPL demo completed successfully!");
    Ok(())
}

fn test_expression(repl: &mut Repl, expr: &str) -> Result<()> {
    println!("  > {}", expr);
    
    // Use a timeout for evaluation
    let deadline = Some(Instant::now() + Duration::from_millis(100));
    
    match repl.evaluate_expr_str(expr, deadline) {
        Ok(value) => {
            println!("  = {}", value);
        }
        Err(e) => {
            println!("  ! Error: {}", e);
        }
    }
    
    Ok(())
}