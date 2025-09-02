use ruchy::{Parser, Transpiler};

#[test]
fn test_string_literal_parameter() {
    let input = r#"
fn greet(name: String) {
    println("Hello, " + name)
}
greet("Alice")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    // Should accept String literal directly
    assert!(rust_code.contains("to_string") || 
            rust_code.contains("String::from"),
            "String literal should be converted to String type: {}", rust_code);
}

#[test]
fn test_string_variable_parameter() {
    let input = r#"
fn greet(name: String) {
    println("Hello, " + name)
}
let person = "Bob"
greet(person)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    // Variable holding string should work
    assert!(rust_code.contains("greet") && rust_code.contains("person"), 
            "Should pass string variable to function: {}", rust_code);
}

#[test]
fn test_str_reference_parameter() {
    let input = r#"
fn print_len(text: &str) {
    println(text.len())
}
print_len("hello")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    // Should pass string literal as &str (no conversion needed)
    assert!(rust_code.contains("print_len") && rust_code.contains("hello") && !rust_code.contains("to_string"),
            "String literal should be passed as &str without conversion: {}", rust_code);
}

#[test]
fn test_string_method_calls() {
    let input = r#"
fn process(text: String) -> String {
    text.to_uppercase()
}
let result = process("hello")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    // Should handle String return type
    assert!(rust_code.contains("-> String"),
            "Should preserve String return type");
}

#[test]
fn test_mixed_string_types() {
    let input = r#"
fn concat(a: String, b: &str) -> String {
    a + b
}
let result = concat("hello", " world")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    // Should handle mixed String and &str parameters
    assert!(rust_code.contains("concat") && rust_code.contains("to_string") && rust_code.contains("world"),
            "Should handle mixed String and &str parameters: {}", rust_code);
}