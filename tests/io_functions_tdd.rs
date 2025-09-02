use ruchy::{Parser, Transpiler};

#[test]
fn test_print_function() {
    // print should output without newline
    let input = r#"
print("Hello")
print(" ")
print("World")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse print");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should use print! macro (without ln)
    assert!(rust_code.contains("print !"),
            "print should transpile to print! macro");
}

#[test]
fn test_println_function() {
    // println should output with newline
    let input = r#"
println("Hello World")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse println");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should use println! macro
    assert!(rust_code.contains("println !"),
            "println should transpile to println! macro");
}

#[test]
fn test_input_function() {
    // input() should read from stdin
    let input = r#"
let name = input("Enter your name: ")
println("Hello, " + name)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse input function");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should handle stdin input
    assert!(rust_code.contains("std") || rust_code.contains("io"),
            "input should use std::io for reading");
}

#[test]
fn test_input_no_prompt() {
    // input() without prompt
    let input = r#"
let value = input()
println(value)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse input without prompt");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile input() without prompt");
}

#[test]
fn test_mixed_io() {
    // Mix of print, println, and input
    let input = r#"
print("Enter a number: ")
let num = input()
println("You entered: " + num)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse mixed I/O");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    assert!(rust_code.contains("print !"),
            "Should have print! for prompt");
    assert!(rust_code.contains("println !"),
            "Should have println! for output");
}