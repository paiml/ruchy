use ruchy::{Parser, Transpiler};

#[test]
fn test_auto_mutability_simple_reassignment() {
    // Variables that are reassigned should automatically be mutable
    let input = r#"
let x = 5
x = 10
println(x)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // x should be automatically mutable since it's reassigned
    assert!(rust_code.contains("let mut x"),
            "Variable x should be auto-detected as mutable");
}

#[test]
fn test_auto_mutability_in_loop() {
    // Loop counters that are modified should be auto-mutable
    let input = r#"
let i = 0
while i < 5 {
    println(i)
    i = i + 1
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // i should be mutable since it's modified in the loop
    assert!(rust_code.contains("let mut i"),
            "Loop counter should be auto-mutable");
}

#[test]
fn test_no_auto_mutability_for_const() {
    // Variables that are never reassigned should remain immutable
    let input = r#"
let x = 5
let y = x + 10
println(y)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // x and y should NOT be mutable
    assert!(!rust_code.contains("let mut x"),
            "Const variable x should remain immutable");
    assert!(!rust_code.contains("let mut y"),
            "Const variable y should remain immutable");
}

#[test]
fn test_auto_mutability_with_compound_assignment() {
    // Compound assignments should trigger auto-mutability
    let input = r#"
let total = 0
total += 5
total *= 2
println(total)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // total should be mutable due to compound assignments
    assert!(rust_code.contains("let mut total"),
            "Variable with compound assignment should be auto-mutable");
}

#[test]
fn test_explicit_mut_preserved() {
    // Explicit let mut should be preserved
    let input = r#"
let mut x = 5
println(x)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Explicit mut should be preserved
    assert!(rust_code.contains("let mut x"),
            "Explicit mut should be preserved");
}

#[test]
fn test_var_always_mutable() {
    // var should always produce mutable bindings
    let input = r#"
var x = 5
println(x)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // var should always be mutable
    assert!(rust_code.contains("let mut x"),
            "var should always produce mutable binding");
}