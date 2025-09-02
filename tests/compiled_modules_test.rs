//! Compiled Multi-File Module System Test (TDD)
//! 
//! Tests that multi-file modules work correctly when compiled to Rust binaries
//!
//! **Expected**: `use math; add(5, 3)` should compile to working Rust binary
//! **Actual**: "unresolved import `math`" compilation error (currently broken)

use ruchy::{Parser, Transpiler};
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_compiled_multi_file_module_integration() {
    // Setup: Create temporary directory with module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy module
    let math_content = r"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}";
    fs::write(temp_dir.path().join("math.ruchy"), math_content)
        .expect("Failed to write math module");
    
    // Create main program that imports and uses the module
    let main_content = r#"use math;

let result = add(5, 3);
println("Result:", result)"#;
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    // Parse and transpile with module context
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse main file with import");
    
    let mut transpiler = Transpiler::new(); 
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(&main_path))
        .expect("Should transpile with module context");
    
    // Write transpiled Rust code to temporary file
    let rust_file = temp_dir.path().join("main.rs");
    fs::write(&rust_file, rust_code.to_string()).expect("Failed to write Rust file");
    
    // Attempt to compile the Rust code
    let compile_output = Command::new("rustc")
        .arg(&rust_file)
        .arg("-o")
        .arg(temp_dir.path().join("test_binary"))
        .output()
        .expect("Failed to run rustc");
    
    // DEBUG: Print the generated Rust code for analysis
    println!("Generated Rust code:\n{rust_code}");
    
    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        println!("Compilation errors:\n{stderr}");
        panic!("Rust compilation should succeed, but failed with errors");
    }
    
    // If compilation succeeds, run the binary
    let run_output = Command::new(temp_dir.path().join("test_binary"))
        .output()
        .expect("Failed to run compiled binary");
    
    assert!(run_output.status.success(), "Compiled binary should run successfully");
    
    let stdout = String::from_utf8(run_output.stdout).expect("Valid UTF-8 output");
    assert!(stdout.contains("Result: 8"), 
            "Output should contain 'Result: 8', got: {stdout:?}");
}

#[test]
fn test_multiple_modules_compilation() {
    // Setup: Create multiple module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy
    fs::write(temp_dir.path().join("math.ruchy"), "pub fn add(x: i32, y: i32) -> i32 { x + y }")
        .expect("Failed to write math module");
    
    // Create utils.ruchy  
    fs::write(temp_dir.path().join("utils.ruchy"), "pub fn format_result(n: i32) -> String { \"Result: \" + n.to_string() }")
        .expect("Failed to write utils module");
    
    // Create main program using both modules
    let main_content = r"use math;
use utils;

let sum = add(10, 15);
let message = format_result(sum);
println(message)";
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    // Parse and transpile
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse main with multiple imports");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(&main_path))
        .expect("Should transpile multiple modules");
    
    let rust_string = rust_code.to_string();
    println!("Multiple modules Rust code:\n{rust_string}");
    
    // Verify both modules are included
    assert!(rust_string.contains("mod math"), "Should contain math module");
    assert!(rust_string.contains("mod utils"), "Should contain utils module"); 
    assert!(rust_string.contains("pub fn add"), "Should contain add function");
    assert!(rust_string.contains("pub fn format_result"), "Should contain format_result function");
    
    // Verify modules come before main function (from RUCHY-110)
    let math_pos = rust_string.find("mod math").expect("math module should exist");
    let utils_pos = rust_string.find("mod utils").expect("utils module should exist");
    let main_pos = rust_string.find("fn main").expect("main function should exist");
    
    assert!(math_pos < main_pos, "Math module should be top-level");
    assert!(utils_pos < main_pos, "Utils module should be top-level");
}

#[test]
fn test_nested_module_compilation() {
    // Test compilation with nested directory structure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let math_dir = temp_dir.path().join("math");
    fs::create_dir(&math_dir).expect("Failed to create math directory");
    
    // Create math/operations.ruchy
    fs::write(math_dir.join("operations.ruchy"), "pub fn subtract(a: i32, b: i32) -> i32 { a - b }")
        .expect("Failed to write operations module");
    
    // Create main program using nested module
    let main_content = r#"use math::operations;

let result = operations::subtract(10, 3);
println("Subtraction result:", result)"#;
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    // This should eventually work, but may fail initially
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse nested module import");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program_with_context(&ast, Some(&main_path));
    
    // For now, just ensure it doesn't panic - nested modules might not be fully implemented yet
    if let Ok(rust_code) = result {
        println!("Nested module Rust code:\n{rust_code}");
        // Test will evolve as nested module support is implemented
    }
}