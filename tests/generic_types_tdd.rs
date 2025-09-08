//! TDD tests for Generic Type support
//! 
//! These tests systematically drive the implementation of generic types
//! needed for Result<T,E> and other generic constructs

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_parse_simple_result_type() {
    let code = "let x: Result<i32, String> = Ok(42);";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Result<i32, String> type: {:?}", result.err());
}

#[test]
fn test_parse_vec_type() {
    let code = "let x: Vec<i32> = [];";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Vec<i32> type: {:?}", result.err());
}

#[test]
fn test_parse_option_type() {
    let code = "let x: Option<String> = Some(\"hello\");";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse Option<String> type: {:?}", result.err());
}

#[test]
fn test_parse_nested_generics() {
    let code = "let x: Vec<Option<i32>> = [];";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse nested generics: {:?}", result.err());
}

#[test]
fn test_parse_function_with_result_return() {
    let code = "fun divide(a: i32, b: i32) -> Result<i32, String> { Ok(a / b) }";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse function with Result return type: {:?}", result.err());
}

#[test]
fn test_transpile_result_function() {
    let code = "fun divide(a: i32, b: i32) -> Result<i32, String> { Ok(a / b) }";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile Result function: {:?}", result.err());
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("Result"), "Should preserve Result in output");
    assert!(rust_code.contains("i32"), "Should preserve i32 type");
    assert!(rust_code.contains("String"), "Should preserve String type");
}

#[test]
fn test_parse_complex_result_usage() {
    let code = r#"
        fun safe_divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 {
                return Err("Division by zero");
            }
            Ok(a / b)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse complex Result usage: {:?}", result.err());
}

#[test]
fn test_transpile_complex_result_usage() {
    let code = r#"
        fun safe_divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 {
                return Err("Division by zero");
            }
            Ok(a / b)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    assert!(result.is_ok(), "Should transpile complex Result usage: {:?}", result.err());
    
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("Result") && rust_code.contains("f64"), "Should contain Result<f64, String>");
    assert!(rust_code.contains("Ok(") || rust_code.contains("Ok ("), "Should contain Ok expression");
    assert!(rust_code.contains("Err(") || rust_code.contains("Err ("), "Should contain Err expression");
}

#[test]  
fn test_parse_generic_method_call() {
    let code = "let result: Vec<i32> = Vec::new();";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse generic method call: {:?}", result.err());
}

#[test]
fn test_parse_multiple_generic_params() {
    let code = "let map: HashMap<String, i32> = {};";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse multiple generic params: {:?}", result.err());
}