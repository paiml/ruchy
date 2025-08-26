//! Statement transpilation test coverage
//! Toyota Way: Target statements module to reach 70% coverage

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::similar_names)]

use ruchy::{Transpiler, Parser};

/// Test assignment statements
#[test]
fn test_transpile_assignments() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple assignment
        ("x = 42", vec!["x", "=", "42"]),
        
        // Complex expression assignment
        ("result = x + y * 2", vec!["result", "=", "x", "+", "y", "*", "2"]),
        
        // String assignment
        (r#"msg = "hello""#, vec!["msg", "=", "\"hello\""]),
        
        // Array assignment
        ("arr = [1, 2, 3]", vec!["arr", "=", "vec", "1", "2", "3"]),
        
        // Function call assignment
        ("value = compute(x, y)", vec!["value", "=", "compute", "x", "y"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Assignment '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test compound assignments
#[test]
fn test_transpile_compound_assignments() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        ("x += 1", vec!["x", "+=", "1"]),
        ("y -= 2", vec!["y", "-=", "2"]),
        ("z *= 3", vec!["z", "*=", "3"]),
        ("w /= 4", vec!["w", "/=", "4"]),
        ("a %= 5", vec!["a", "%=", "5"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Compound assignment '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test return statements
#[test]
fn test_transpile_return_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Return with value
        ("return 42", vec!["return", "42"]),
        
        // Return without value
        ("return", vec!["return"]),
        
        // Return expression
        ("return x + y", vec!["return", "x", "+", "y"]),
        
        // Return in function
        ("fun test() -> i32 { return 42 }", vec!["fn", "test", "return", "42"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Return statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test break and continue statements
#[test]
fn test_transpile_loop_control_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Break in while loop
        ("while true { break }", vec!["while", "true", "break"]),
        
        // Continue in for loop
        ("for i in 0..10 { continue }", vec!["for", "i", "in", "0", "10", "continue"]),
        
        // Break with nested loops
        ("while x > 0 { for i in 0..5 { break } }", vec!["while", "for", "break"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Loop control '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test expression statements
#[test]
fn test_transpile_expression_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Function call statement
        ("println(42)", vec!["println", "42"]),
        
        // Method call statement
        ("vec.push(1)", vec!["vec", "push", "1"]),
        
        // Standalone expression
        ("x + y", vec!["x", "+", "y"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Expression statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test let statements with various patterns
#[test]
fn test_transpile_let_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple let
        ("let x = 42", vec!["let", "x", "=", "42"]),
        
        // Let with type
        ("let x: i32 = 42", vec!["let", "x", "i32", "42"]),
        
        // Let with mutable
        ("let mut x = 0", vec!["let", "mut", "x", "=", "0"]),
        
        // Let with pattern
        ("let (a, b) = (1, 2)", vec!["let", "a", "b", "1", "2"]),
        
        // Let with complex expression
        ("let result = if x > 0 { x } else { -x }", vec!["let", "result", "=", "if", "x", ">", "0"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Let statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test multi-statement blocks
#[test]
fn test_transpile_statement_blocks() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Multiple statements
        ("{ let x = 1; let y = 2; x + y }", vec!["let", "x", "1", "let", "y", "2", "x", "+", "y"]),
        
        // Statements with side effects
        ("{ println(1); println(2); 3 }", vec!["println", "1", "println", "2", "3"]),
        
        // Mixed statement types
        ("{ let a = 1; a += 2; return a }", vec!["let", "a", "1", "+=", "2", "return", "a"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Statement block '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test import statements
#[test]
fn test_transpile_import_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Simple import
        ("import std", vec!["use", "std"]),
        
        // Import with path
        ("import std.collections", vec!["use", "std", "collections"]),
        
        // Import specific items
        ("import std.{HashMap, Vec}", vec!["use", "std", "HashMap", "Vec"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Import statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test export statements
#[test]
fn test_transpile_export_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Export function
        ("export fun test() -> i32 { 42 }", vec!["pub", "fn", "test", "42"]),
        
        // Export variable
        ("export let x = 42", vec!["pub", "let", "x", "42"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Export statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}

/// Test chained statements
#[test]
fn test_transpile_chained_statements() {
    let transpiler = Transpiler::new();
    
    let test_cases = [
        // Chained assignments
        ("x = y = z = 0", vec!["x", "=", "y", "=", "z", "=", "0"]),
        
        // Pipeline statements
        ("data |> filter |> map |> collect", vec!["data", "filter", "map", "collect"]),
    ];
    
    for (input, expected_parts) in test_cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse().unwrap_or_else(|_| panic!("Failed to parse: {input}"));
        
        let result = transpiler.transpile(&ast).unwrap_or_else(|_| panic!("Failed to transpile: {input}"));
        let transpiled = result.to_string();
        
        for part in expected_parts {
            assert!(
                transpiled.contains(part),
                "Chained statement '{}' should contain '{}', got: '{}'",
                input, part, transpiled
            );
        }
    }
}