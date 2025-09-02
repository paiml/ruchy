use ruchy::{Parser, Transpiler};

#[test]
fn test_var_keyword_basic() {
    // var should create a mutable variable
    let input = r#"
var x = 5
x = 10
println(x)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse var keyword");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // var should transpile to let mut
    assert!(rust_code.contains("let mut x"),
            "var should transpile to let mut");
}

#[test]
fn test_var_vs_let() {
    // var should be mutable, let should be immutable
    let input = r#"
let a = 5
var b = 10
b = 20
println(a + b)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse var and let");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // let should be immutable, var should be mutable
    assert!(!rust_code.contains("let mut a"),
            "let should remain immutable");
    assert!(rust_code.contains("let mut b"),
            "var should be mutable");
}

#[test]
fn test_var_with_type() {
    // var with type annotation
    let input = r#"
var count: i32 = 0
count = count + 1
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse var with type");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should preserve type and mutability
    assert!(rust_code.contains("let mut count"),
            "var with type should be mutable");
    assert!(rust_code.contains("i32"),
            "Type annotation should be preserved");
}