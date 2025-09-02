// TDD Tests for function call fixes
use ruchy::{Parser, Transpiler};

#[test]
fn test_function_call_no_args() {
    let code = "func_name()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile func_name() without args: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("func_name"));
    assert!(rust_code.contains("()"));
}

#[test]
fn test_function_call_with_args() {
    let code = "func_name(1, 2, 3)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should transpile func_name with args: {:?}", result.err());
    let rust_code = result.unwrap().to_string();
    assert!(rust_code.contains("func_name"));
}

#[test]
fn test_unknown_function_should_not_validate_args() {
    // Unknown functions shouldn't have arg count validation
    let code = "my_custom_func()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_ok(), "Should not validate args for unknown functions: {:?}", result.err());
}

#[test]
fn test_type_conversion_requires_one_arg() {
    // Type conversion functions SHOULD validate
    let code = "int()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&ast);
    
    assert!(result.is_err(), "int() with no args should fail");
    assert!(result.err().unwrap().to_string().contains("expects exactly 1 argument"));
}