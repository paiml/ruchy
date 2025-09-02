//! Transpiler Architecture Bug: Statement/Expression Confusion
//! 
//! Documents the issue where the transpiler incorrectly treats statement
//! sequences as expressions, causing invalid Rust code generation.

use ruchy::{Parser, Transpiler};

#[test]
fn test_multiple_statements_in_main() {
    // Multiple let statements should work in main()
    let code = r"
        let x = 5;
        let y = 10;
        println(x + y);
    ";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse multiple statements");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .expect("Should transpile to program");
    let rust_string = rust_code.to_string();
    
    println!("Generated Rust code:\n{rust_string}");
    
    // Should generate a proper main function with statements
    assert!(rust_string.contains("fn main"));
    assert!(rust_string.contains("let mut x = 5"));
    assert!(rust_string.contains("let mut y = 10"));
    
    // Should NOT wrap everything in a result expression
    assert!(!rust_string.contains("let result ="), 
            "Should not wrap statements in result expression");
}

#[test]
fn test_mixed_statements_and_expressions() {
    let code = r"
        let x = 5;
        let y = x * 2;
        x + y  // Final expression value
    ";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse mixed code");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_program(&ast)
        .expect("Should transpile to program");
    let rust_string = rust_code.to_string();
    
    println!("Mixed code Rust:\n{rust_string}");
    
    // Statements should be statements
    assert!(rust_string.contains("let mut x = 5"));
    assert!(rust_string.contains("let mut y = x * 2"));
    
    // Final expression should be the result
    assert!(rust_string.contains("x + y"));
}

#[test]
fn test_function_with_statements() {
    let code = r"
        fn calculate(a: i32, b: i32) -> i32 {
            let sum = a + b;
            let product = a * b;
            sum + product
        }
    ";
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse function with statements");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)
        .expect("Should transpile function");
    let rust_string = rust_code.to_string();
    
    println!("Function Rust code:\n{rust_string}");
    
    // Should generate proper function
    assert!(rust_string.contains("fn calculate"));
    assert!(rust_string.contains("let sum = a + b") || rust_string.contains("let mut sum = a + b"));
    assert!(rust_string.contains("let product = a * b") || rust_string.contains("let mut product = a * b"));
    assert!(rust_string.contains("sum + product"));
}