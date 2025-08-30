//! Parentheses Precedence Regression Test (TDD)
//! 
//! Tests that parentheses are preserved in expressions for correct operator precedence
//!
//! **Critical Issue**: Variable Math one-liner failing due to lost parentheses
//! **Expected**: `price * (1.0 + tax)` should maintain parentheses 
//! **Actual**: `price * 1f64 + tax` loses parentheses, wrong calculation

use ruchy::{Parser, Transpiler};

#[test]
fn test_parentheses_preserved_in_multiplication() {
    // This is the exact failing one-liner case
    let source = "let price = 99.99; let tax = 0.08; price * (1.0 + tax)";
    
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse variable math expression");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .expect("Should transpile variable math");
    
    let rust_string = rust_code.to_string();
    println!("Generated Rust code: {rust_string}");
    
    // Critical assertion: The parentheses must be preserved in transpiled code
    // Should NOT generate: price * 1f64 + tax  (wrong precedence)
    // Should generate: price * (1.0f64 + tax)  (correct precedence)
    
    assert!(!rust_string.contains("price * 1f64 + tax") && 
            !rust_string.contains("price * 1.0f64 + tax"),
            "Should not generate wrong precedence: {rust_string}");
    
    // Should contain proper parentheses grouping
    assert!(rust_string.contains('(') && rust_string.contains(')'),
            "Should preserve parentheses for correct precedence: {rust_string}");
}

#[test]
fn test_complex_parentheses_expressions() {
    // Test various parentheses scenarios that might be failing
    let test_cases = vec![
        ("a * (b + c)", "Multiplication with addition in parentheses"),
        ("(a + b) * c", "Addition in parentheses times multiplication"),
        ("a + (b * c)", "Addition with multiplication in parentheses"), 
        ("(a + b) / (c - d)", "Both operands in parentheses"),
        ("x * (y + z) * w", "Middle expression in parentheses"),
    ];
    
    for (expression, description) in test_cases {
        println!("\nTesting: {expression} - {description}");
        
        let full_source = format!("let a = 1; let b = 2; let c = 3; let d = 4; let x = 5; let y = 6; let z = 7; let w = 8; {expression}");
        
        let mut parser = Parser::new(&full_source);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Should parse: {expression}"));
        
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile_to_program(&ast)
            .unwrap_or_else(|_| panic!("Should transpile: {expression}"));
        
        let rust_string = rust_code.to_string();
        println!("Generated: {rust_string}");
        
        // Should preserve parentheses structure
        assert!(rust_string.contains('(') && rust_string.contains(')'),
                "Should preserve parentheses in '{expression}': {rust_string}");
    }
}

#[test]
fn test_variable_math_end_to_end() {
    // End-to-end test of the failing Variable Math one-liner
    use std::process::Command;
    use tempfile::NamedTempFile;
    use std::fs;
    
    let source = "let price = 99.99; let tax = 0.08; price * (1.0 + tax)";
    
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse variable math");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .expect("Should transpile variable math");
    
    // Write to temporary Rust file and compile
    let rust_file = NamedTempFile::with_suffix(".rs").expect("Create temp file");
    fs::write(rust_file.path(), rust_code.to_string()).expect("Write Rust code");
    
    let output = Command::new("rustc")
        .arg(rust_file.path())
        .arg("-o")
        .arg("/tmp/test_variable_math")
        .output()
        .expect("Compile Rust code");
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Rust compilation error: {stderr}");
        println!("Generated Rust code: {rust_code}");
    }
    assert!(output.status.success(), "Rust compilation should succeed");
    
    let run_output = Command::new("/tmp/test_variable_math")
        .output()
        .expect("Run compiled binary");
    
    assert!(run_output.status.success(), "Compiled program should run");
    
    let stdout = String::from_utf8(run_output.stdout).expect("Valid UTF-8 output");
    
    // The correct answer: 99.99 * (1.0 + 0.08) = 99.99 * 1.08 = 107.9892
    assert!(stdout.contains("107.9892"), 
            "Should output correct calculation 107.9892, got: {stdout}");
    
    // Should NOT output wrong calculation: 99.99 * 1.0 + 0.08 = 100.07
    assert!(!stdout.contains("100.07"),
            "Should NOT output incorrect calculation 100.07, got: {stdout}");
}