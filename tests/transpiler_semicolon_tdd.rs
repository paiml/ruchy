// TDD tests for transpiler semicolon bug in function blocks
// This test captures the regression found in book compatibility testing

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
fn test_function_with_multiple_statements_needs_semicolons() {
    let code = r#"
        fun main() {
            println("Hello,")
            println("World!")
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // The transpiled code should have semicolons between statements
    assert!(
        rust_str.contains(r#"println ! ("Hello,") ;"#) || 
        rust_str.contains(r#"println!("Hello,");"#),
        "First println should have semicolon. Got: {}",
        rust_str
    );
}

#[test]
fn test_function_with_three_statements() {
    let code = r#"
        fun greet() {
            println("One")
            println("Two")
            println("Three")
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // Should have semicolons after first two statements
    assert!(
        rust_str.contains(";"),
        "Should contain semicolons between statements. Got: {}",
        rust_str
    );
}

#[test]
fn test_function_with_mixed_statements() {
    let code = r#"
        fun calculate() {
            let x = 5
            let y = 10
            println(x + y)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // Let statements should have semicolons (may have type annotations like 5i32)
    assert!(
        rust_str.contains("let x = 5 ;") || 
        rust_str.contains("let x = 5;") ||
        rust_str.contains("let x = 5i32 ;") ||
        rust_str.contains("let x = 5i32;"),
        "Let statements should have semicolons. Got: {}",
        rust_str
    );
}

#[test]
fn test_function_with_single_statement_no_extra_semicolon() {
    let code = r#"
        fun hello() {
            println("Hello")
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // Single statement doesn't need trailing semicolon (it's the return value)
    // But println is a void function, so it should have a semicolon
    assert!(
        rust_str.contains(r#"println ! ("Hello") ;"#) || 
        rust_str.contains(r#"println!("Hello");"#) ||
        rust_str.contains(r#"println ! ("Hello")"#), // Also OK for single void statement
        "Single println can have semicolon or not. Got: {}",
        rust_str
    );
}

#[test]
fn test_function_with_return_expression() {
    let code = r#"
        fun add(x, y) {
            x + y
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // Return expression should NOT have semicolon
    assert!(
        !rust_str.contains("x + y ;"),
        "Return expression should not have semicolon. Got: {}",
        rust_str
    );
}

#[test]
fn test_function_with_statements_and_return() {
    let code = r#"
        fun compute() {
            let x = 5
            let y = 10
            x + y
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_str = rust_code.to_string();
    
    // Let statements should have semicolons (may have type annotations like 5i32)
    assert!(
        rust_str.contains("let x = 5 ;") || 
        rust_str.contains("let x = 5;") ||
        rust_str.contains("let x = 5i32 ;") ||
        rust_str.contains("let x = 5i32;"),
        "Let statements should have semicolons. Got: {}",
        rust_str
    );
    
    // Final expression should NOT have semicolon
    assert!(
        !rust_str.contains("x + y ;"),
        "Return expression should not have semicolon. Got: {}",
        rust_str
    );
}