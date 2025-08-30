//! Vec Display Bug: Vec<T> doesn't implement Display for println
//! 
//! Found during RUCHY-104 broader compatibility testing
//! Issue: println with Vec generates Display trait requirement instead of Debug

use ruchy::{Parser, Transpiler};

#[test]
fn test_vec_display_bug() {
    let mut parser = Parser::new(r#"
        let numbers = [1, 2, 3];
        println("Numbers:", numbers);
    "#);
    let ast = parser.parse().expect("Should parse array println");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Generated Rust code: {rust_string}");
    
    // The bug: println generates Display requirement for Vec
    // Should generate {:?} debug format for complex types like Vec
    
    // Current broken behavior: println!("{}", vec) -> Display trait error
    // Expected correct behavior: println!("{:?}", vec) -> Debug trait works
    
    if rust_string.contains("println !") {
        // If it generates println! macro, it should use debug formatting for vecs
        // This will currently fail to compile due to Display trait requirement
        println!("BUG: Vec will require Display trait which doesn't exist");
    }
}

#[test]
fn test_simple_println_works() {
    // This should work fine (baseline)
    let mut parser = Parser::new(r#"println("Hello", 42)"#);
    let ast = parser.parse().expect("Should parse simple println");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    let rust_string = rust_code.to_string();
    
    println!("Simple println Rust code: {rust_string}");
    
    // This should generate working code
    assert!(rust_string.contains("println !"));
}