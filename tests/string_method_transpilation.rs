//! String Method Transpilation Tests
//! Toyota Way: Automated tests institutionalize correct behavior permanently
//! 
//! These tests ensure string methods are correctly mapped from Ruchy names
//! to Rust names during transpilation, preventing regressions.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_assert)]
#![allow(clippy::panic)]
#![allow(clippy::expect_fun_call)]

use ruchy::{Transpiler, Parser};
use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Test string method name mapping in transpilation
#[test]
fn test_string_method_name_mapping() {
    let transpiler = Transpiler::new();
    
    // Test cases: (ruchy_method, expected_rust_method)
    let test_cases = [
        ("to_upper", "to_uppercase"),
        ("to_lower", "to_lowercase"),
        ("length", "len"),
    ];
    
    for (ruchy_method, rust_method) in test_cases {
        let code = format!(r#""hello".{}()"#, ruchy_method);
        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Failed to parse");
        
        let result = transpiler.transpile(&ast).expect("Failed to transpile");
        let transpiled_code = result.to_string();
        
        // The transpiled code should contain the correct Rust method name
        assert!(
            transpiled_code.contains(rust_method),
            "Expected '{}' to be transpiled to '{}', but got: {}",
            ruchy_method, rust_method, transpiled_code
        );
        
        // The transpiled code should NOT contain the original Ruchy method name
        assert!(
            !transpiled_code.contains(&format!(".{}", ruchy_method)),
            "Transpiled code still contains original method '{}': {}",
            ruchy_method, transpiled_code
        );
    }
}

/// Test string method compilation (end-to-end)
#[test]
fn test_string_method_compilation() {
    let test_cases = [
        (r#""hello".to_upper()"#, "HELLO"),
        (r#""WORLD".to_lower()"#, "world"),
        (r#""test".len()"#, "4"),
    ];
    
    for (code, expected_output) in test_cases {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.ruchy");
        
        fs::write(&test_file, code).expect("Failed to write test file");
        
        let output = Command::new("./target/debug/ruchy")
            .arg("run")
            .arg(&test_file)
            .output()
            .expect("Failed to execute ruchy");
        
        if !output.status.success() {
            panic!(
                "Compilation failed for '{}': {}",
                code,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected_output),
            "Expected output '{}' for '{}', got: '{}'",
            expected_output, code, stdout
        );
    }
}

/// Property test: All string methods should transpile to valid Rust
#[test]
fn test_string_method_transpilation_validity() {
    let transpiler = Transpiler::new();
    
    let string_methods = [
        "to_upper", "to_lower", "len", "trim", "chars", "reverse",
        "contains", "starts_with", "ends_with", "replace", "split"
    ];
    
    for method in string_methods {
        let code = match method {
            "replace" => format!(r#""hello".{}("l", "x")"#, method),
            "contains" | "starts_with" | "ends_with" => format!(r#""hello".{}("he")"#, method),
            "split" => format!(r#""hello".{}(" ")"#, method),
            _ => format!(r#""hello".{}()"#, method),
        };
        
        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect(&format!("Failed to parse: {}", code));
        
        let result = transpiler.transpile(&ast).expect(&format!("Failed to transpile: {}", code));
        let transpiled_code = result.to_string();
        
        // The transpiled code should be valid Rust syntax (basic check)
        assert!(
            transpiled_code.contains("fn main"),
            "Transpiled code should contain main function: {}",
            transpiled_code
        );
        
        // Should not contain obvious syntax errors
        assert!(
            !transpiled_code.contains(".."),
            "Transpiled code contains double dots (syntax error): {}",
            transpiled_code
        );
    }
}

/// Integration test: String methods in complex expressions
#[test]
fn test_string_methods_in_expressions() {
    let test_cases = [
        (r#"let x = "hello".to_upper(); println("{}", x)"#, "HELLO"),
        (r#"println("{}", "WORLD".to_lower().len())"#, "5"),
        (r#"if "test".len() > 3 { println("long") } else { println("short") }"#, "long"),
    ];
    
    for (code, expected_output) in test_cases {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.ruchy");
        
        fs::write(&test_file, code).expect("Failed to write test file");
        
        let output = Command::new("./target/debug/ruchy")
            .arg("run")
            .arg(&test_file)
            .output()
            .expect("Failed to execute ruchy");
        
        if !output.status.success() {
            panic!(
                "Compilation failed for complex expression '{}': {}",
                code,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected_output),
            "Expected output '{}' for complex expression '{}', got: '{}'",
            expected_output, code, stdout
        );
    }
}

/// Regression test: Ensure one-liner compatibility
#[test]
fn test_string_method_one_liners() {
    let one_liners = [
        r#""hello".to_upper()"#,
        r#""WORLD".to_lower()"#,
        r#""test".len()"#,
        r#""  spaced  ".trim()"#,
    ];
    
    for code in one_liners {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file = temp_dir.path().join("test.ruchy");
        
        fs::write(&test_file, code).expect("Failed to write test file");
        
        let output = Command::new("./target/debug/ruchy")
            .arg("run")
            .arg(&test_file)
            .output()
            .expect("Failed to execute ruchy");
        
        assert!(
            output.status.success(),
            "One-liner '{}' failed to compile: {}",
            code,
            String::from_utf8_lossy(&output.stderr)
        );
        
        // Should produce some output (not empty)
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.trim().is_empty(),
            "One-liner '{}' produced no output",
            code
        );
    }
}