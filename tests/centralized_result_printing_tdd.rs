// TDD test for centralized result printing refactoring
// Goal: ONE place that handles all result printing logic

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[test]
fn test_centralized_result_printing_match_expression() {
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
    
    // Should use centralized result printing pattern
    assert!(rust_code.contains("match & result") && rust_code.contains("type_name_of_val"), 
            "Should use centralized result printing pattern: {}", rust_code);
            
    // Should NOT use old downcast pattern  
    assert!(!rust_code.contains("downcast_ref"), 
            "Should not use old downcast_ref pattern: {}", rust_code);
    
    // Should handle Unit types without Display errors
    assert!(rust_code.contains(r#"!= "()""#) || !rust_code.contains(r#"println!("{}", result)"#),
            "Unit types should be handled safely: {}", rust_code);
}

#[test]
fn test_centralized_result_printing_simple_expression() {
    let input = r#"5 + 3"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should use centralized result printing pattern
    assert!(rust_code.contains("match & result") && rust_code.contains("type_name_of_val"), 
            "Should use centralized result printing pattern with match statement: {}", rust_code);
            
    // Should NOT use old downcast pattern
    assert!(!rust_code.contains("downcast_ref"), 
            "Should not use old downcast_ref pattern: {}", rust_code);
}

#[test]
fn test_centralized_result_printing_println_statements() {
    let input = r#"
println("Hello")
println("World")
"#;
    
    let mut parser = Parser::new(input);
    let ast = parser.parse().expect("Failed to parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    let rust_code = result.expect("Failed to transpile").to_string();
    
    // Should have AT MOST one result printing pattern (may have zero if no result printing needed)
    let result_printing_count = rust_code.matches("std::any::type_name_of_val").count();
    assert!(result_printing_count <= 1, 
            "Should have at most one result printing location: {}", rust_code);
}

#[test]
fn test_result_printing_consistency() {
    // Test multiple different input types to ensure consistent result printing
    let test_cases = vec![
        r#"let x = 5; x"#,
        r#"[1, 2, 3]"#,
        r#""hello world""#,
        r#"if true { "yes" } else { "no" }"#,
    ];
    
    for (i, input) in test_cases.iter().enumerate() {
        let mut parser = Parser::new(input);
        let ast = parser.parse().expect(&format!("Failed to parse case {}", i));
        
        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&ast);
        let rust_code = result.expect(&format!("Failed to transpile case {}", i)).to_string();
        
        // Each should have centralized result printing approach  
        let result_printing_count = rust_code.matches("std::any::type_name_of_val").count();
        assert!(result_printing_count >= 1 && result_printing_count <= 2, 
                "Case {}: Should have centralized result printing (1-2 type_name_of_val): {}", i, rust_code);
                
        // Should have exactly one match statement (centralized)
        let match_count = rust_code.matches("match & result").count();  
        assert_eq!(match_count, 1,
                   "Case {}: Should have exactly one centralized match statement: {}", i, rust_code);
                   
        // All should use the same safe pattern
        if rust_code.contains("std::any::type_name_of_val") {
            assert!(rust_code.contains(r#"!= "()""#) || !rust_code.contains(r#"println!("{}", result)"#),
                    "Case {}: Should use safe Unit type handling: {}", i, rust_code);
        }
    }
}