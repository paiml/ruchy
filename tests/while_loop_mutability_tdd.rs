use ruchy::{Parser, Transpiler};

#[test]
fn test_while_loop_counter_mutability() {
    // Variables modified in while loops should be automatically mutable
    let input = r#"
let i = 0
while i < 5 {
    i = i + 1
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Variable i should be declared as mutable since it's reassigned in the loop
    assert!(rust_code.contains("let mut i") || rust_code.contains("let mut i ="),
            "Variable reassigned in while loop should be mutable");
}

#[test]
fn test_while_loop_accumulator_mutability() {
    // Accumulator pattern should work with auto-mutability
    let input = r#"
let total = 0
let i = 0
while i < 10 {
    total = total + i
    i = i + 1
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Both variables should be mutable
    assert!(rust_code.contains("let mut total"),
            "Accumulator variable should be mutable");
    assert!(rust_code.contains("let mut i"),
            "Loop counter should be mutable");
}

#[test]
fn test_while_loop_no_mutation() {
    // Variables not modified should remain immutable
    let input = r#"
let limit = 5
let mut i = 0
while i < limit {
    println(i)
    i = i + 1
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // limit should NOT be mutable (only read, not modified)
    assert!(!rust_code.contains("let mut limit"),
            "Variable not modified should remain immutable");
}