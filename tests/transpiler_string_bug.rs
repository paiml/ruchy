//! CRITICAL TRANSPILER BUG TEST - String type handling (Issue #13)
//! This follows EXTREME TDD protocol for transpiler bugs

use ruchy::{Parser, Transpiler};

#[test]
fn test_str_parameter_transpilation() {
    // Test that Ruchy 'str' type maps to Rust '&str' in parameters
    let mut parser = Parser::new("fun greet(name: str) { println(name) }");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    println!("Generated code: {}", code); // Debug output
    
    // Should use &str, not str (accounting for spacing)
    assert!(code.contains("& str") || code.contains("&str"), "Function parameter should be &str");
    assert!(!code.contains("name : str"), "Should not use unsized str type");
}

#[test]
fn test_string_literal_no_unnecessary_to_string() {
    // Test that string literals don't get .to_string() when passed to &str parameters
    let mut parser = Parser::new("fun main() { greet(\"Hello\") }");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    // Should NOT add unnecessary .to_string()
    assert!(!code.contains("\"Hello\" . to_string ()"), "Should not add .to_string() to string literals");
}

#[test] 
fn test_println_format_correctness() {
    // Test that println!("{}", name) generates correct format, not "{} {:?}"
    let mut parser = Parser::new("fun test() { println(\"Hello, {}!\", \"World\") }");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");  
    let code = result.unwrap().to_string();
    
    // Should use correct println format
    assert!(!code.contains("\"{} {:?}\""), "Should not use debug format");
    assert!(code.contains("println !"), "Should generate println! macro");
}

#[test]
fn test_no_unnecessary_hashmap_import() {
    // Test that simple functions don't generate unused HashMap import
    let mut parser = Parser::new("fun main() { println(\"Hello\") }");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    // Should NOT include unnecessary HashMap import
    assert!(!code.contains("use std :: collections :: HashMap"), "Should not import HashMap unnecessarily");
}

#[test]
fn test_no_extra_braces() {
    // Test that function bodies don't get double-wrapped in braces
    let mut parser = Parser::new("fun test() { 42 }");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    // Should not have double braces {{ }}
    assert!(!code.contains("{ {"), "Should not generate nested braces");
    assert!(!code.contains("} }"), "Should not generate nested braces");
}

#[test]
fn test_complete_string_example_compiles() {
    // Integration test: Full example from Issue #13 should compile successfully
    let code = r#"
        fun greet(name: str) {
            println("Hello, {}!", name)
        }
        
        fun main() {
            greet("World")
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile without errors");
    
    // The generated Rust code should compile (we'll test this in CI)
    let generated = result.unwrap().to_string();
    
    
    // Basic sanity checks for correct transpilation
    assert!(generated.contains("&str") || generated.contains("& str"), "Should use &str for parameters");
    assert!(!generated.contains("name : str)"), "Should not use unsized str");
    assert!(!generated.contains("\"World\" . to_string"), "Should not add .to_string() unnecessarily");
}

