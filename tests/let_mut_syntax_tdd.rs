use ruchy::{Parser, Transpiler};

#[test]
fn test_let_mut_basic() {
    // Basic let mut syntax should work
    let input = r#"
let mut x = 5
x = 10
println(x)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse let mut syntax");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should transpile to mutable variable
    assert!(rust_code.contains("let mut x"),
            "let mut should transpile to Rust's let mut");
}

#[test]
fn test_let_mut_in_loop() {
    // let mut should work in control flow
    let input = r#"
let mut count = 0
while count < 3 {
    println("Count: " + count)
    count = count + 1
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse let mut in while loop");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should have mutable counter
    assert!(rust_code.contains("let mut count"),
            "let mut count should be mutable in transpiled code");
}

#[test]
fn test_let_immutable_unchanged() {
    // Regular let should remain immutable
    let input = r#"
let x = 5
let y = x + 10
println(y)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse regular let");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should NOT have mut for regular let
    assert!(!rust_code.contains("let mut x"),
            "Regular let should not be mutable");
    assert!(!rust_code.contains("let mut y"),
            "Regular let should not be mutable");
}

#[test]
fn test_let_mut_with_type_annotation() {
    // let mut with type annotation
    let input = r#"
let mut total: i32 = 0
total = total + 5
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse let mut with type");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should preserve mutability and type
    assert!(rust_code.contains("let mut total"),
            "let mut with type should be mutable");
    assert!(rust_code.contains("i32"),
            "Type annotation should be preserved");
}