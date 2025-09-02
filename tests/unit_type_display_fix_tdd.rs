// TDD test for Unit type display formatting fix
// Issue: Unit types () cause Display format error when transpiled

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_match_expression_unit_type() {
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
    
    // Should handle Unit type separately to avoid compilation error
    assert!(rust_code.contains(r#"== "()""#) || 
            rust_code.contains(r#"!= "()""#) ||
            rust_code.contains(r#""()" => { }"#),
            "Unit type should be handled without Display formatting: {}", rust_code);
}

#[test]
fn test_println_statement_unit_type() {
    let input = r#"
println("Hello")
println("World")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should not try to format Unit type results from println statements
    assert!(!rust_code.contains(r#"println ! ("{}" , result)"#) || 
            rust_code.contains("Unit") || rust_code.contains(r#"== "()""#) || rust_code.contains(r#"!= "()""#),
            "Unit type from println should be handled correctly: {}", rust_code);
}

#[test]
fn test_assignment_statement_unit_type() {
    let input = r#"
let mut x = 5
x = x + 1
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Assignment statements return Unit type, should be handled properly
    assert!(!rust_code.contains(r#"println ! ("{}" , result)"#) || 
            rust_code.contains(r#"== "()""#) || rust_code.contains(r#"!= "()""#),
            "Unit type from assignment should be handled correctly: {}", rust_code);
}