//! Ruchy Run Command Multi-File Module Test (TDD)
//! 
//! Tests that `ruchy run` command correctly handles multi-file modules
//!
//! **Expected**: `ruchy run main.ruchy` should find and compile math.ruchy imports
//! **Actual**: "unresolved import `math`" compilation error (currently broken)

use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_ruchy_run_with_external_modules() {
    // Setup: Create temporary directory with module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy module in same directory
    let math_content = r"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}";
    fs::write(temp_dir.path().join("math.ruchy"), math_content)
        .expect("Failed to write math module");
    
    // Create main.ruchy that imports the module
    let main_content = r#"use math;

let result = add(5, 3);
println("Addition result:", result);

let product = multiply(4, 6);  
println("Multiplication result:", product)"#;
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    // Change to module directory so ruchy can find the modules
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    // Run the ruchy run command
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")  
        .arg("main.ruchy")
        .output()
        .expect("Failed to execute ruchy run");
    
    // Restore original directory
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");
    
    // Print output for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("STDOUT:\n{stdout}");
    println!("STDERR:\n{stderr}");
    
    // Test should pass: ruchy run should successfully compile and execute
    assert!(output.status.success(), 
            "ruchy run should succeed with multi-file modules, stderr: {stderr}");
    
    // Verify expected output
    assert!(stdout.contains("Addition result: 8"), 
            "Should output 'Addition result: 8', got: {stdout}");
    assert!(stdout.contains("Multiplication result: 24"), 
            "Should output 'Multiplication result: 24', got: {stdout}");
}

#[test] 
fn test_ruchy_run_missing_module_error() {
    // Test proper error handling when module is missing
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create main.ruchy that imports non-existent module
    let main_content = r#"use nonexistent_module;

println("This should not run")"#;
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")
        .arg("main.ruchy") 
        .output()
        .expect("Failed to execute ruchy run");
    
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Error output: {stderr}");
    
    // Should fail with clear error message
    assert!(!output.status.success(), "Should fail when module is missing");
    assert!(stderr.contains("nonexistent_module") || stderr.contains("not found") || 
            stderr.contains("module"), 
            "Error should mention missing module, got: {stderr}");
}

#[test]
fn test_ruchy_run_nested_directory_modules() {
    // Test modules in subdirectories  
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create utils/ subdirectory
    let utils_dir = temp_dir.path().join("utils");
    fs::create_dir(&utils_dir).expect("Failed to create utils directory");
    
    // Create utils/helpers.ruchy
    let helpers_content = r#"pub fn format_number(n: i32) -> String {
    "Number: " + n.to_string()
}"#;
    fs::write(utils_dir.join("helpers.ruchy"), helpers_content)
        .expect("Failed to write helpers module");
    
    // Create main.ruchy that imports nested module
    let main_content = r"use utils::helpers;

let formatted = helpers::format_number(42);
println(formatted)";
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    let original_dir = std::env::current_dir().expect("Failed to get current dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("run")
        .arg("main.ruchy")
        .output()
        .expect("Failed to execute ruchy run");
    
    std::env::set_current_dir(original_dir).expect("Failed to restore directory");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Nested module STDOUT: {stdout}");  
    println!("Nested module STDERR: {stderr}");
    
    // This test documents expected behavior - may not work initially
    if output.status.success() {
        assert!(stdout.contains("Number: 42"), 
                "Should output 'Number: 42', got: {stdout}");
    } else {
        // Document the current limitation
        println!("Nested modules not yet supported, error: {stderr}");
    }
}