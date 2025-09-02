use ruchy::{Parser, Transpiler};

#[test]
fn test_undefined_variable_in_expression() {
    // This is what's failing in the book - using undefined variables
    let input = r#"
let step1 = initial_value * factor
let step2 = step1 + adjustment
let final_result = step2 / divisor
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse();
    
    // This should parse successfully (parser doesn't check if variables are defined)
    assert!(ast.is_ok(), "Parser should accept undefined variables");
    
    // But transpilation should handle this gracefully
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast.unwrap());
    
    // The transpiler will generate code with undefined variables
    // which will fail at Rust compilation time
    assert!(result.is_ok(), "Transpiler should generate code even with undefined vars");
}

#[test]
fn test_defined_variables_work() {
    // With defined variables, everything should work
    let input = r#"
let initial_value = 10
let factor = 2
let adjustment = 5
let divisor = 3
let step1 = initial_value * factor
let step2 = step1 + adjustment
let final_result = step2 / divisor
println(final_result)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Should parse");
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Should transpile").to_string();
    
    // Should have all the variables
    assert!(rust_code.contains("let initial_value"));
    assert!(rust_code.contains("let factor"));
    assert!(rust_code.contains("let step1"));
    assert!(rust_code.contains("let final_result"));
}