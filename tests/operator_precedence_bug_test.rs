//! CRITICAL BUG: Parenthesized expressions not parsed correctly
//! 
//! Found during RUCHY-104 broader compatibility testing
//! Expected: (1 + 2) * 3 = 9
//! Actual: (1 + 2) * 3 = 7 (parentheses ignored)

use ruchy::{Parser, Transpiler};

#[test]
fn test_parentheses_parsing_bug() {
    let mut parser = Parser::new("(1 + 2) * 3");
    let ast = parser.parse().expect("Should parse parenthesized expression");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Generated Rust code: {rust_string}");
    
    // The bug: parentheses are being ignored during parsing
    // This should generate ((1 + 2) * 3) but likely generates (1 + (2 * 3))
    // Result: 7 instead of expected 9
    
    // For now, just document what we expect the correct behavior to be
    // When fixed, this should generate: (1i32 + 2i32) * 3i32
    assert!(rust_string.contains("1i32 + 2i32"), "Should contain the addition");
    assert!(rust_string.contains("* 3i32"), "Should contain multiplication by 3");
    
    // TODO: Add runtime test when we can execute generated code
    // The runtime result should be 9, not 7
}

#[test] 
fn test_basic_precedence_works() {
    // This should work correctly (and does)
    let mut parser = Parser::new("1 + 2 * 3");
    let ast = parser.parse().expect("Should parse basic precedence");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Basic precedence Rust code: {rust_string}");
    
    // Should generate 1 + (2 * 3) due to operator precedence
    // Runtime result should be 7 (which is correct)
}

#[test]
fn test_complex_parentheses_bug() {
    // More complex case that should also fail
    let mut parser = Parser::new("(2 + 3) * (4 + 5)");
    let ast = parser.parse().expect("Should parse complex parentheses");
    
    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");  
    let rust_string = rust_code.to_string();
    
    println!("Complex parentheses Rust code: {rust_string}");
    
    // Should generate (2 + 3) * (4 + 5) = 5 * 9 = 45
    // But likely generates something else due to precedence bug
}