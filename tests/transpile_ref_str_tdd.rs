use ruchy::{Parser, Transpiler};

#[test]
fn test_transpile_ref_str_function() {
    let input = r#"
fn greet(name: &str) {
    println(name)
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
    
    // Check that &str type is correctly transpiled
    assert!(rust_code.contains("& str") || rust_code.contains("&str"), 
            "Should transpile &str type correctly");
    // String literal should be passed directly without .to_string()
    assert!(rust_code.contains("greet (\"Alice\")"), 
            "String literal should be passed directly to &str parameter");
}

#[test]
fn test_transpile_string_vs_ref_str() {
    let input = r#"
fn takes_string(s: String) {
    println(s)
}
fn takes_ref_str(s: &str) {
    println(s)
}
takes_string("hello")
takes_ref_str("world")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    println!("Generated Rust code:");
    println!("{rust_code}");
    
    // Check the generated code
    assert!(rust_code.contains("fn takes_string (s : String)"), 
            "Should have String parameter");
    assert!(rust_code.contains("fn takes_ref_str (s : & str)") || 
            rust_code.contains("fn takes_ref_str (s : &str)"), 
            "Should have &str parameter");
}