use ruchy::{Parser, Transpiler};

#[test]
fn debug_string_literal_parameter() {
    let input = r#"
fn greet(name: String) {
    println("Hello, " + name)
}
greet("Alice")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    println!("Generated Rust code:");
    println!("{rust_code}");
    
    // Check what we're actually generating
    assert!(rust_code.contains("greet"), "Should contain greet function");
}