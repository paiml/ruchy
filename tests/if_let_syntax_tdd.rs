// TDD for if-let syntax implementation
// Missing feature: if-let for pattern matching with Option/Result types

use ruchy::{Parser, Transpiler};

#[test]
fn test_if_let_option_some() {
    // Test case 1: if-let with Option::Some
    let code = r#"
        let maybe = Some(42);
        if let Some(x) = maybe {
            println("Got value: " + x.to_string());
        } else {
            println("Got None");
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse if-let with Some");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should generate Rust if-let pattern matching
    assert!(rust_code.contains("if let Some"), "Should contain 'if let Some' pattern");
    assert!(!rust_code.contains("if Some"), "Should not be a regular if statement");
}

#[test]
fn test_if_let_result_ok() {
    // Test case 2: if-let with Result::Ok
    let code = r#"
        let result = Ok(100);
        if let Ok(value) = result {
            println("Success: " + value.to_string());
        } else {
            println("Error occurred");
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse if-let with Ok");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should generate Rust if-let pattern matching
    assert!(rust_code.contains("if let Ok"), "Should contain 'if let Ok' pattern");
}

#[test]
fn test_if_let_custom_enum() {
    // Test case 3: if-let with custom enum
    let code = r#"
        enum Message {
            Text(String),
            Number(i32),
            None
        }
        
        let msg = Message::Text("hello");
        if let Message::Text(s) = msg {
            println("Text message: " + s);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse if-let with custom enum");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should generate Rust if-let pattern matching
    assert!(rust_code.contains("if let Message::Text"), "Should contain enum pattern matching");
}

#[test]
fn test_if_let_nested_pattern() {
    // Test case 4: if-let with nested patterns
    let code = r#"
        let data = Some((42, "answer"));
        if let Some((num, text)) = data {
            println("Number: " + num.to_string() + ", Text: " + text);
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse nested if-let");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should handle nested destructuring in if-let
    assert!(rust_code.contains("if let Some(("), "Should handle nested patterns");
}

#[test]
fn test_if_let_with_guard() {
    // Test case 5: if-let with additional guard condition
    let code = r#"
        let maybe = Some(42);
        if let Some(x) = maybe && x > 40 {
            println("Got large value: " + x.to_string());
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse if-let with guard");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should handle if-let with guard condition
    assert!(rust_code.contains("if let Some"), "Should contain if-let pattern");
    assert!(rust_code.contains("&&"), "Should include guard condition");
}

#[test]
fn test_while_let() {
    // Test case 6: while-let for iteration
    let code = r#"
        let mut iter = Some(1);
        while let Some(item) = iter {
            println("Item: " + item.to_string());
            iter = None;
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse while-let");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should generate while-let loop
    assert!(rust_code.contains("while let Some"), "Should contain 'while let Some' pattern");
}

#[test]
fn test_if_let_else_if_let() {
    // Test case 7: chained if-let else if-let
    let code = r#"
        let value = Some(42);
        if let Some(x) = value && x < 10 {
            println("Small value");
        } else if let Some(x) = value && x >= 10 {
            println("Large value");  
        } else {
            println("No value");
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse chained if-let");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile_to_string(&ast).expect("Failed to transpile");
    
    // Should handle chained if-let patterns
    assert!(rust_code.contains("if let Some"), "Should contain first if-let");
    assert!(rust_code.contains("else if let Some"), "Should contain else if-let");
}