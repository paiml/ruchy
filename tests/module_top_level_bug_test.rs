//! Module Top-Level Bug Test (TDD)
//! 
//! Documents the issue where module declarations are generated inside `main()` 
//! instead of at the top level of the Rust program.
//!
//! **Expected**: `mod math { ... }` at top level, then `fn main() { ... }`
//! **Actual**: `fn main() { mod math { ... } ... }` (currently broken)

use ruchy::{Parser, Transpiler};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_modules_should_be_top_level_not_inside_main() {
    // Setup: Create temporary module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy module
    let math_content = r"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}";
    fs::write(temp_dir.path().join("math.ruchy"), math_content)
        .expect("Failed to write math module");
    
    // Create main file that imports the module
    let main_content = r#"use math;

let result = add(5, 10);
println("Result:", result);"#;
    
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse main file with import");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(temp_dir.path().join("main.ruchy").as_path()))
        .expect("Should transpile with module context");
    let rust_string = rust_code.to_string();
    
    println!("Generated Rust code:\n{rust_string}");
    
    // CRITICAL: Module should be at top level, NOT inside main()
    assert!(rust_string.contains("mod math {"), "Module declaration should exist");
    
    // Find positions of module and main
    let mod_pos = rust_string.find("mod math").expect("Module should exist");
    let main_pos = rust_string.find("fn main").expect("Main function should exist");
    
    // Module should come BEFORE main function (top-level)
    assert!(mod_pos < main_pos, 
        "Module should be declared before main function (top-level), not inside it");
    
    // Main function should NOT contain the module declaration
    let main_fn_start = main_pos;
    let main_fn_end = rust_string[main_fn_start..].rfind('}').unwrap_or(rust_string.len() - main_fn_start) + main_fn_start + 1;
    let main_fn_body = &rust_string[main_fn_start..main_fn_end];
    
    assert!(!main_fn_body.contains("mod math"), 
        "Main function body should NOT contain module declarations");
}

#[test]
fn test_multiple_modules_all_top_level() {
    // Setup: Create multiple module files
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    fs::write(temp_dir.path().join("math.ruchy"), "pub fn add(x: i32, y: i32) -> i32 { x + y }")
        .expect("Failed to write math module");
    fs::write(temp_dir.path().join("utils.ruchy"), "pub fn format_result(n: i32) -> String { \"Result: \" + n.to_string() }")
        .expect("Failed to write utils module");
    
    let main_content = r"use math;
use utils;

let sum = add(5, 10);
let message = format_result(sum);
println(message);";
    
    let mut parser = Parser::new(main_content);
    let ast = parser.parse().expect("Should parse main with multiple imports");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(temp_dir.path().join("main.ruchy").as_path()))
        .expect("Should transpile multiple modules");
    let rust_string = rust_code.to_string();
    
    println!("Multiple modules Rust code:\n{rust_string}");
    
    // Both modules should be top-level
    assert!(rust_string.contains("mod math"), "Math module should exist");
    assert!(rust_string.contains("mod utils"), "Utils module should exist");
    
    let math_mod_pos = rust_string.find("mod math").expect("Math module should exist");
    let utils_mod_pos = rust_string.find("mod utils").expect("Utils module should exist");
    let main_pos = rust_string.find("fn main").expect("Main function should exist");
    
    // Both modules should come before main
    assert!(math_mod_pos < main_pos, "Math module should be top-level");
    assert!(utils_mod_pos < main_pos, "Utils module should be top-level");
}