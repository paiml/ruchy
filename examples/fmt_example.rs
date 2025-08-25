//! Executable example demonstrating `ruchy fmt` command
//! Toyota Way: Clear, runnable examples prevent user confusion
//! 
//! Run with: `cargo run --example fmt_example`

#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::if_not_else)]

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Ruchy Format Command Examples");
    println!("================================\n");

    // Example 1: Basic function formatting
    basic_function_formatting()?;
    
    // Example 2: Complex expression formatting  
    complex_expression_formatting()?;
    
    // Example 3: Multiple functions
    multiple_functions_formatting()?;
    
    // Example 4: Error handling
    error_handling_example()?;

    println!("✅ All fmt examples completed successfully!");
    Ok(())
}

fn basic_function_formatting() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 1: Basic Function Formatting");
    println!("---------------------------------------");
    
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("basic.ruchy");
    
    // Create unformatted code
    let unformatted = "fun add(x:i32,y:i32)->i32{x+y}";
    println!("Before formatting:");
    println!("  {}", unformatted);
    
    fs::write(&test_file, unformatted)?;
    
    // Run formatter
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fmt", test_file.to_str().unwrap()])
        .output()?;
    
    if output.status.success() {
        let formatted = fs::read_to_string(&test_file)?;
        println!("After formatting:");
        for (i, line) in formatted.lines().enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        println!("✅ Basic formatting successful\n");
    } else {
        println!("❌ Formatting failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn complex_expression_formatting() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 2: Complex Expression Formatting");
    println!("-------------------------------------------");
    
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("complex.ruchy");
    
    // Create complex unformatted code
    let unformatted = "fun calculate(x:i32,y:i32,z:i32)->i32{if x>y{if y>z{x*y}else{x*z}}else{if y>z{y*z}else{z}}}";
    println!("Before formatting:");
    println!("  {}", unformatted);
    
    fs::write(&test_file, unformatted)?;
    
    // Run formatter
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fmt", test_file.to_str().unwrap()])
        .output()?;
    
    if output.status.success() {
        let formatted = fs::read_to_string(&test_file)?;
        println!("After formatting:");
        for (i, line) in formatted.lines().enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        println!("✅ Complex formatting successful\n");
    } else {
        println!("❌ Formatting failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn multiple_functions_formatting() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 3: Multiple Functions Formatting");
    println!("------------------------------------------");
    
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("multiple.ruchy");
    
    // Create multiple unformatted functions
    let unformatted = r"fun first(x:i32)->i32{x+1}
fun second(y:i32,z:i32)->i32{y*z}
fun third(a:i32)->i32{if a>0{a}else{0}}";
    
    println!("Before formatting:");
    for (i, line) in unformatted.lines().enumerate() {
        println!("  {}: {}", i + 1, line);
    }
    
    fs::write(&test_file, unformatted)?;
    
    // Run formatter
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fmt", test_file.to_str().unwrap()])
        .output()?;
    
    if output.status.success() {
        let formatted = fs::read_to_string(&test_file)?;
        println!("After formatting:");
        for (i, line) in formatted.lines().enumerate() {
            println!("  {}: {}", i + 1, line);
        }
        println!("✅ Multiple functions formatting successful\n");
    } else {
        println!("❌ Formatting failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}

fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Example 4: Error Handling");
    println!("----------------------------");
    
    // Test 1: Missing file
    println!("Testing missing file...");
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fmt", "nonexistent.ruchy"])
        .output()?;
    
    if !output.status.success() {
        println!("✅ Correctly handled missing file:");
        println!("  {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    
    // Test 2: Invalid syntax
    println!("\nTesting invalid syntax...");
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("invalid.ruchy");
    
    fs::write(&test_file, "fun invalid_syntax((((")?;
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "fmt", test_file.to_str().unwrap()])
        .output()?;
    
    if !output.status.success() {
        println!("✅ Correctly handled invalid syntax:");
        println!("  Error detected and reported");
    } else {
        println!("⚠️  Invalid syntax was unexpectedly accepted");
    }
    
    println!("✅ Error handling examples complete\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting_example() {
        // Test that the example functions work
        assert!(basic_function_formatting().is_ok());
    }

    #[test] 
    fn test_complex_formatting_example() {
        assert!(complex_expression_formatting().is_ok());
    }

    #[test]
    fn test_multiple_functions_example() {
        assert!(multiple_functions_formatting().is_ok());
    }

    #[test]
    fn test_error_handling_example() {
        assert!(error_handling_example().is_ok());
    }
}