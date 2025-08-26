//! Multi-File Module System Test (TDD)
//! 
//! Tests the ability to import external .ruchy files as modules
//!
//! **Expected**: `use math; let result = add(5, 3); println(result)` should load math.ruchy
//! **Actual**: "Unknown function: add" (currently broken - no file loading)

use ruchy::runtime::repl::Repl;
use ruchy::{Parser, Transpiler};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_multi_file_module_import_in_repl() {
    // Setup: Create temporary directory with module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy module
    let math_content = r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b  
}"#;
    fs::write(temp_dir.path().join("math.ruchy"), math_content)
        .expect("Failed to write math module");
    
    // Setup REPL with temp directory as working directory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    // Test input: Import external module and use its functions
    let import_result = repl.evaluate_expr_str("use math", None);
    assert!(import_result.is_ok(), "Module import should succeed, got: {:?}", import_result);
    
    // Test that we can now call functions from the imported module
    let add_result = repl.evaluate_expr_str("add(5, 3)", None);
    assert!(add_result.is_ok(), "Should be able to call add() from imported math module");
    
    if let Ok(value) = add_result {
        assert_eq!(format!("{:?}", value), "Int(8)", "add(5, 3) should return 8");
    }
    
    // Test second function from same module
    let multiply_result = repl.evaluate_expr_str("multiply(4, 6)", None);
    assert!(multiply_result.is_ok(), "Should be able to call multiply() from imported math module");
    
    if let Ok(value) = multiply_result {
        assert_eq!(format!("{:?}", value), "Int(24)", "multiply(4, 6) should return 24");
    }
}

#[test]
fn test_multi_file_module_compilation() {
    // Setup: Create temporary directory with module files  
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create utils.ruchy module
    let utils_content = r#"pub fn greet(name: String) -> String {
    "Hello, " + name + "!"
}"#;
    fs::write(temp_dir.path().join("utils.ruchy"), utils_content)
        .expect("Failed to write utils module");
        
    // Create main program that imports and uses the module
    let main_content = r#"use utils;

let greeting = greet("World");
println(greeting)"#;
    
    let main_path = temp_dir.path().join("main.ruchy");
    fs::write(&main_path, main_content).expect("Failed to write main file");
    
    // Parse and transpile with module context
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse main file with import");
    
    let transpiler = Transpiler::new(); 
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(&main_path))
        .expect("Should transpile with module context");
    let rust_string = rust_code.to_string();
    
    // Should contain the utils module at top-level
    assert!(rust_string.contains("mod utils"), "Should contain utils module declaration");
    assert!(rust_string.contains("pub fn greet"), "Should contain greet function");
    
    // Verify module comes before main function (from RUCHY-110 fix)
    let mod_pos = rust_string.find("mod utils").expect("utils module should exist");
    let main_pos = rust_string.find("fn main").expect("main function should exist");
    assert!(mod_pos < main_pos, "Module should be declared before main function");
}

#[test] 
fn test_nested_module_directories() {
    // Setup: Create nested directory structure
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let math_dir = temp_dir.path().join("math");
    fs::create_dir(&math_dir).expect("Failed to create math directory");
    
    // Create math/operations.ruchy  
    let operations_content = r#"pub fn add(x: i32, y: i32) -> i32 { x + y }
pub fn subtract(x: i32, y: i32) -> i32 { x - y }"#;
    fs::write(math_dir.join("operations.ruchy"), operations_content)
        .expect("Failed to write operations module");
    
    // Test importing from subdirectory
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    let import_result = repl.evaluate_expr_str("use math::operations", None);
    assert!(import_result.is_ok(), "Nested module import should succeed");
    
    let result = repl.evaluate_expr_str("operations::add(10, 5)", None);
    assert!(result.is_ok(), "Should be able to call nested module function");
    
    if let Ok(value) = result {
        assert_eq!(format!("{:?}", value), "Integer(15)", "operations::add(10, 5) should return 15");
    }
}

#[test]
fn test_module_not_found_error() {
    // Test that importing non-existent module gives clear error
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    let result = repl.evaluate_expr_str("use nonexistent", None);
    assert!(result.is_err(), "Should error when trying to import non-existent module");
    
    let error_msg = format!("{:?}", result);
    assert!(error_msg.contains("nonexistent") || error_msg.contains("not found") || 
            error_msg.contains("module"), 
            "Error should mention the missing module, got: {}", error_msg);
}