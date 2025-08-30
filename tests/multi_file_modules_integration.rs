//! Integration tests for multi-file module system
//! 
//! Tests the complete workflow: `ModuleResolver` + Transpiler for multi-file programs

use ruchy::{ModuleResolver, Parser, Transpiler};
use tempfile::TempDir;
use std::fs;
use anyhow::Result;

fn create_test_module(temp_dir: &TempDir, name: &str, content: &str) -> Result<()> {
    let file_path = temp_dir.path().join(format!("{name}.ruchy"));
    fs::write(file_path, content)?;
    Ok(())
}

#[test]
fn test_basic_multi_file_program() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create math.ruchy module
    create_test_module(&temp_dir, "math", r"
        pub fun add(a: i32, b: i32) -> i32 {
            a + b
        }
        
        pub fun multiply(x: i32, y: i32) -> i32 {
            x * y
        }
    ")?;
    
    // Create main program that uses math module
    let main_code = r"
        use math;
        
        fun main() {
            let result = math::add(5, 3);
            let doubled = math::multiply(result, 2);
            println(doubled);
        }
    ";
    
    // Parse the main program
    let mut parser = Parser::new(main_code);
    let ast = parser.parse()?;
    
    // Resolve imports
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    let resolved_ast = resolver.resolve_imports(ast)?;
    
    // Transpile to Rust
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&resolved_ast)?;
    let rust_string = rust_code.to_string();
    
    // Verify the generated Rust code contains expected elements
    assert!(rust_string.contains("mod math"), "Should contain math module declaration");
    assert!(rust_string.contains("pub fn add"), "Should contain add function");
    assert!(rust_string.contains("pub fn multiply"), "Should contain multiply function");
    assert!(rust_string.contains("fn main"), "Should contain main function");
    assert!(rust_string.contains("math :: add"), "Should contain qualified function call");
    
    println!("Generated Rust code:\n{rust_string}");
    
    Ok(())
}

#[test]
fn test_multiple_module_imports() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create utils.ruchy
    create_test_module(&temp_dir, "utils", r#"
        pub fun greet(name: String) -> String {
            "Hello " + name
        }
    "#)?;
    
    // Create math.ruchy
    create_test_module(&temp_dir, "math", r"
        pub fun square(n: i32) -> i32 {
            n * n
        }
    ")?;
    
    // Create main program that uses both modules
    let main_code = r#"
        use utils;
        use math;
        
        fun main() {
            let greeting = utils::greet("World");
            let number = math::square(4);
            println(greeting);
            println(number);
        }
    "#;
    
    let mut parser = Parser::new(main_code);
    let ast = parser.parse()?;
    
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    let resolved_ast = resolver.resolve_imports(ast)?;
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&resolved_ast)?;
    let rust_string = rust_code.to_string();
    
    assert!(rust_string.contains("mod utils"));
    assert!(rust_string.contains("mod math"));
    assert!(rust_string.contains("pub fn greet"));
    assert!(rust_string.contains("pub fn square"));
    
    println!("Generated Rust code for multiple modules:\n{rust_string}");
    
    Ok(())
}

#[test]
fn test_nested_module_imports() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create helper.ruchy (dependency)
    create_test_module(&temp_dir, "helper", r#"
        pub fun format_number(n: i32) -> String {
            n.to_string() + "!"
        }
    "#)?;
    
    // Create printer.ruchy (uses helper)
    create_test_module(&temp_dir, "printer", r"
        use helper;
        
        pub fun print_formatted(num: i32) {
            let formatted = helper::format_number(num);
            println(formatted);
        }
    ")?;
    
    // Create main program that uses printer (which internally uses helper)
    let main_code = r"
        use printer;
        
        fun main() {
            printer::print_formatted(42);
        }
    ";
    
    let mut parser = Parser::new(main_code);
    let ast = parser.parse()?;
    
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    let resolved_ast = resolver.resolve_imports(ast)?;
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&resolved_ast)?;
    let rust_string = rust_code.to_string();
    
    // Should contain both modules due to nested imports
    assert!(rust_string.contains("mod printer"));
    assert!(rust_string.contains("mod helper"));
    assert!(rust_string.contains("pub fn print_formatted"));
    assert!(rust_string.contains("pub fn format_number"));
    
    println!("Generated Rust code for nested imports:\n{rust_string}");
    
    Ok(())
}

#[test]
fn test_mixed_imports_inline_and_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Create file module
    create_test_module(&temp_dir, "file_math", r"
        pub fun add(a: i32, b: i32) -> i32 {
            a + b
        }
    ")?;
    
    // Create main program with mixed imports
    let main_code = r"
        use std::collections::HashMap;
        use file_math;
        
        mod inline_utils {
            pub fun double(x: i32) -> i32 {
                x * 2
            }
        }
        
        fun main() {
            let result = file_math::add(5, 3);
            let doubled = inline_utils::double(result);
            println(doubled);
        }
    ";
    
    let mut parser = Parser::new(main_code);
    let ast = parser.parse()?;
    
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    let resolved_ast = resolver.resolve_imports(ast)?;
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&resolved_ast)?;
    let rust_string = rust_code.to_string();
    
    // Should contain file module (resolved)
    assert!(rust_string.contains("mod file_math"));
    assert!(rust_string.contains("pub fn add"));
    
    // Should contain standard library import (unchanged)
    assert!(rust_string.contains("use std :: collections :: HashMap"));
    
    // Should contain inline module (unchanged)
    assert!(rust_string.contains("mod inline_utils"));
    assert!(rust_string.contains("pub fn double"));
    
    println!("Generated Rust code for mixed imports:\n{rust_string}");
    
    Ok(())
}

#[test]
fn test_module_resolver_stats() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    create_test_module(&temp_dir, "test1", "pub fun test1() {}")?;
    create_test_module(&temp_dir, "test2", "pub fun test2() {}")?;
    
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    
    let initial_stats = resolver.stats();
    assert_eq!(initial_stats.files_loaded, 0);
    assert_eq!(initial_stats.cached_modules, 0);
    
    // Load first module
    let main_code1 = "use test1; test1::test1();";
    let mut parser = Parser::new(main_code1);
    let ast = parser.parse()?;
    resolver.resolve_imports(ast)?;
    
    let after_first = resolver.stats();
    assert_eq!(after_first.files_loaded, 1);
    assert_eq!(after_first.cached_modules, 1);
    
    // Load second module
    let main_code2 = "use test2; test2::test2();";
    let mut parser = Parser::new(main_code2);
    let ast = parser.parse()?;
    resolver.resolve_imports(ast)?;
    
    let after_second = resolver.stats();
    assert_eq!(after_second.files_loaded, 2);
    assert_eq!(after_second.cached_modules, 2);
    
    // Load first module again (should hit cache)
    let mut parser = Parser::new(main_code1);
    let ast = parser.parse()?;
    resolver.resolve_imports(ast)?;
    
    let after_cache_hit = resolver.stats();
    assert_eq!(after_cache_hit.files_loaded, 2); // Same as before
    assert_eq!(after_cache_hit.cache_hits, 1); // Should have cache hit
    
    Ok(())
}

#[test]
fn test_module_not_found_fallback() -> Result<()> {
    let temp_dir = TempDir::new()?;
    
    // Don't create any modules - test fallback behavior
    let main_code = r#"
        use nonexistent_module;
        
        fun main() {
            println("This should fall back to inline import");
        }
    "#;
    
    let mut parser = Parser::new(main_code);
    let ast = parser.parse()?;
    
    let mut resolver = ModuleResolver::new();
    resolver.add_search_path(temp_dir.path());
    
    // This should not panic, but gracefully fall back
    // The exact behavior depends on how we handle missing modules
    let result = resolver.resolve_imports(ast);
    
    // For now, we expect this to fail with a clear error message
    // In a production system, we might want to fall back to treating it as an inline import
    assert!(result.is_err(), "Should error when module is not found");
    
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("nonexistent_module"), "Error should mention the module name");
    
    Ok(())
}