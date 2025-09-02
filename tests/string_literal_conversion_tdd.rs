use ruchy::{Parser, Transpiler};

#[test]
fn test_string_literal_to_string_conversion() {
    // Simple case: String literals should be converted when needed
    let input = r#""hello".to_string()"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should transpile to_string() method call correctly
    assert!(rust_code.contains("\"hello\".to_string()"),
            "Should transpile string literal with to_string() method");
}

#[test]
fn test_string_literal_as_is() {
    // String literals without conversion should remain as literals
    let input = r#""hello""#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should remain as string literal
    assert!(rust_code.contains("\"hello\""),
            "Should keep string literal as is");
}

#[test]
fn test_string_from_conversion() {
    // Test String::from() conversion
    let input = r#"String.from("hello")"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should transpile String::from correctly
    assert!(rust_code.contains("String::from(\"hello\")") || 
            rust_code.contains("String :: from (\"hello\")"),
            "Should transpile String::from() correctly");
}