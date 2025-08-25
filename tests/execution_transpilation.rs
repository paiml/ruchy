//! Execution Transpilation Tests
//! Toyota Way: Test the actual execution path that `ruchy run` uses
//! 
//! These tests target the specific transpilation path used by CLI execution
//! to ensure method name mapping works in the real execution environment.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_assert)]
#![allow(clippy::panic)]
#![allow(clippy::print_stdout)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::expect_fun_call)]

use ruchy::{Transpiler, Parser};
use std::fs;
use tempfile::TempDir;

/// Test the exact transpilation path used by `ruchy run`
#[test]
fn test_cli_execution_transpilation_path() {
    let transpiler = Transpiler::new();
    
    // Test the exact same path that `ruchy run` uses
    let code = r#""hello".to_upper()"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    
    // Use the same method that CLI uses for program transpilation
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile to program");
    let transpiled_code = result.to_string();
    
    println!("Transpiled code from CLI path: {}", transpiled_code);
    
    // This should contain the correct Rust method name
    assert!(
        transpiled_code.contains("to_uppercase"),
        "CLI transpilation path failed to map to_upper -> to_uppercase: {}",
        transpiled_code
    );
    
    // Should not contain the wrong method name
    assert!(
        !transpiled_code.contains("to_upper()"),
        "CLI transpilation path still contains 'to_upper()': {}",
        transpiled_code
    );
}

/// Test that program transpilation creates valid main function wrapper
#[test]  
fn test_program_transpilation_structure() {
    let transpiler = Transpiler::new();
    
    let code = r#""hello".to_upper()"#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");
    
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile to program");
    let transpiled_code = result.to_string();
    
    // Should have proper program structure
    assert!(
        transpiled_code.contains("fn main"),
        "Program transpilation should create main function: {}",
        transpiled_code
    );
    
    // Should be valid Rust
    assert!(
        transpiled_code.contains("{") && transpiled_code.contains("}"),
        "Program should have proper block structure: {}",
        transpiled_code
    );
}

/// Test different expression types in CLI execution context
#[test]
fn test_various_expressions_cli_transpilation() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        (r#""hello".to_upper()"#, "to_uppercase"),
        (r#""WORLD".to_lower()"#, "to_lowercase"), 
        (r#""test".len()"#, "len"),
    ];
    
    for (code, expected_method) in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", code));
        
        let result = transpiler.transpile_to_program(&ast)
            .expect(&format!("Failed to transpile to program: {}", code));
        let transpiled_code = result.to_string();
        
        println!("Code: {} -> {}", code, transpiled_code);
        
        assert!(
            transpiled_code.contains(expected_method),
            "Expected method '{}' in transpiled code for '{}': {}",
            expected_method, code, transpiled_code
        );
    }
}

/// Test write-compile-execute cycle with temp files
#[test]
fn test_temp_file_compilation_cycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source_file = temp_dir.path().join("test.ruchy");
    let rust_file = temp_dir.path().join("test.rs");
    
    // Write Ruchy source
    fs::write(&source_file, r#""hello".to_upper()"#).expect("Failed to write source");
    
    // Transpile to Rust using the CLI path
    let transpiler = Transpiler::new();
    let source_content = fs::read_to_string(&source_file).unwrap();
    let mut parser = Parser::new(&source_content);
    let ast = parser.parse().expect("Failed to parse");
    
    let result = transpiler.transpile_to_program(&ast).expect("Failed to transpile");
    let rust_code = result.to_string();
    
    // Write Rust code 
    fs::write(&rust_file, &rust_code).expect("Failed to write Rust");
    
    println!("Generated Rust code:\n{}", rust_code);
    
    // The generated Rust should be compilable and contain correct method
    assert!(
        rust_code.contains("to_uppercase"),
        "Generated Rust code should contain 'to_uppercase': {}",
        rust_code
    );
    
    // Try to compile the Rust code with rustc to verify it's valid
    let output = std::process::Command::new("rustc")
        .args(["--edition", "2021", "-o", "/tmp/test_exec"])
        .arg(&rust_file)
        .output();
        
    if let Ok(compilation) = output {
        if !compilation.status.success() {
            println!("Rustc compilation failed: {}", String::from_utf8_lossy(&compilation.stderr));
            println!("Rust code that failed:\n{}", rust_code);
        }
        
        // This assertion will help us understand if the generated Rust is valid
        assert!(
            compilation.status.success(),
            "Generated Rust code should compile with rustc"
        );
    }
}