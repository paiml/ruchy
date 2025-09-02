//! Multi-arg println Bug Test
//! 
//! Expected: println("Hello", "World", 42) should output "Hello World 42"
//! Actual: Currently broken - treats first arg as format string incorrectly

use ruchy::{Parser, Transpiler};

#[test]
fn test_println_multi_args() {
    let mut parser = Parser::new(r#"
        println("Hello", "World", 42)
    "#);
    let ast = parser.parse().expect("Should parse multi-arg println");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Generated Rust code: {rust_string}");
    
    // Should generate: println!("{} {} {:?}", "Hello", "World", 42)
    // With {} for strings and {:?} for numbers
    
    assert!(rust_string.contains("println !"));
    // The format string should have mixed placeholders: {} for strings, {:?} for numbers
    assert!(rust_string.contains(r#""{} {} {:?}""#), 
            "Expected format string with mixed placeholders, got: {rust_string}");
}

#[test] 
fn test_println_single_arg_still_works() {
    let mut parser = Parser::new(r#"println("Just one string")"#);
    let ast = parser.parse().expect("Should parse single-arg println");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Single arg Rust code: {rust_string}");
    
    // Single arg should work as before
    assert!(rust_string.contains("println !"));
    assert!(rust_string.contains(r#""Just one string""#));
}