// TDD test for double semicolon transpiler bug fix
// Issue: "let number = 2i32 ; ;" - extra semicolon after variable declarations

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_single_variable_declaration_semicolon() {
    let input = r#"
let number = 2
match number {
    1 => println("One"),
    2 => println("Two"),
    _ => println("Other")
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not contain double semicolons after variable declarations
    assert!(!rust_code.contains(" ; ;"), 
            "Code should not contain double semicolons: {}", rust_code);
    
    // Should contain proper single semicolon after variable declaration
    assert!(rust_code.contains("let number = 2i32 ;") || rust_code.contains("let number = 2i32;"), 
            "Should contain proper variable declaration: {}", rust_code);
}

#[test]
fn test_multiple_variable_declarations_semicolon() {
    let input = r#"
let x = 1
let y = 2
let result = x + y
println(result)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not contain any double semicolons
    assert!(!rust_code.contains(" ; ;"), 
            "Code should not contain double semicolons: {}", rust_code);
    
    // Should contain proper single semicolons after each variable
    let semicolon_count = rust_code.matches(" ; ").count();
    assert!(semicolon_count >= 3, 
            "Should contain proper semicolons for each statement: {}", rust_code);
}

#[test]
fn test_variable_followed_by_expression_semicolon() {
    let input = r#"
let data = [1, 2, 3]
data.len()
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not contain double semicolons
    assert!(!rust_code.contains(" ; ;"), 
            "Code should not contain double semicolons: {}", rust_code);
    
    // Should have proper separation between statements
    assert!(rust_code.contains("let data") && (rust_code.contains("data.len()") || rust_code.contains("data . len ()")), 
            "Should contain both statements properly: {}", rust_code);
}

#[test] 
fn test_println_macro_not_escaped() {
    let input = r#"
let msg = "Hello"
println(msg)
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not contain escaped println macro
    assert!(!rust_code.contains("println \\!"), 
            "println! macro should not be escaped: {}", rust_code);
    
    // Should contain proper println! macro call
    assert!(rust_code.contains("println !") || rust_code.contains("println!"), 
            "Should contain proper println! macro call: {}", rust_code);
}

#[test]
fn test_unit_type_result_handling() {
    let input = r#"
let number = 2
match number {
    1 => println("One"),
    2 => println("Two"),
    _ => println("Other")
}
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not try to display unit type with {}
    // Unit types should either be suppressed or use {:?}
    if rust_code.contains("match result") {
        assert!(!rust_code.contains("println ! (\"{}\", result)") || 
                rust_code.contains("println ! (\"{:?}\", result)"),
                "Unit types should not use {{}} formatting: {}", rust_code);
    }
}