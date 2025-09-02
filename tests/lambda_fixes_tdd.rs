// TDD Tests for lambda expression fixes
use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_lambda() {
    let code = "|x| x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse lambda");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile simple lambda: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    println!("Generated lambda code: {}", rust_code);
    assert!(rust_code.contains("| x |") || rust_code.contains("|x|") || rust_code.contains("move"));
}

#[test]
fn test_lambda_with_multiple_params() {
    let code = "|x, y| x + y";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse lambda with multiple params");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile lambda with multiple params: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    // Check for lambda with spaces (proc_macro2 formatting)
    assert!(rust_code.contains("| x") || rust_code.contains("|x"));
    assert!(rust_code.contains("y |") || rust_code.contains("y|"));
}

#[test]
fn test_lambda_no_params() {
    let code = "|| 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse lambda with no params");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile lambda with no params: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("| |") || rust_code.contains("||"));
}

#[test]
fn test_lambda_with_parentheses() {
    let code = "(x) => x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse lambda with parentheses");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile (x) => x + 1: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("| x |") || rust_code.contains("|x|"));
}

#[test]
fn test_lambda_in_map() {
    let code = "[1, 2, 3].map(|x| x * 2)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse lambda in map");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile lambda in map: {:?}", result.err());
}