// TDD tests for range patterns in match expressions
// This test captures the regression found in book compatibility testing

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;
use ruchy::runtime::interpreter::Interpreter;

#[test]
fn test_match_with_inclusive_range_pattern() {
    let code = r#"
        let age = 25
        match age {
            0 => "Zero",
            1..=17 => "Minor",
            18..=65 => "Adult",
            _ => "Senior"
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse range patterns in match");
    
    // Check that it parses correctly
    assert!(format!("{:?}", ast).contains("Range"));
}

#[test]
fn test_match_with_exclusive_range_pattern() {
    let code = r#"
        let x = 5
        match x {
            0 => "Zero",
            1..10 => "Single digit",
            10..100 => "Double digit",
            _ => "Large"
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse exclusive range patterns");
    
    // Check that it parses correctly
    assert!(format!("{:?}", ast).contains("Range"));
}

#[test]
fn test_match_range_pattern_evaluation() {
    let code = r#"
        fun check_age(age: i32) -> String {
            match age {
                0 => "Zero",
                1..=17 => "Minor",
                18..=65 => "Adult",
                _ => "Senior"
            }
        }
        
        check_age(25)
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile");
    
    // The transpiled code should handle range patterns
    let rust_str = rust_code.to_string();
    assert!(rust_str.contains("1..=17") || rust_str.contains("1 ..= 17") || rust_str.contains("1i32 ..= 17i32"), 
            "Should transpile inclusive range pattern. Got: {}", rust_str);
}

#[test]
fn test_match_range_pattern_interpreter() {
    let code = r#"
        let age = 25
        match age {
            0 => "Zero",
            1..=17 => "Minor",  
            18..=65 => "Adult",
            _ => "Senior"
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");
    
    // Should match the "Adult" case
    assert_eq!(format!("{}", result), "Adult");
}

#[test]
fn test_match_exclusive_range_pattern_interpreter() {
    let code = r#"
        let x = 5
        match x {
            0 => "Zero",
            1..10 => "Single digit",
            10..100 => "Double digit",
            _ => "Large"
        }
    "#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Should evaluate");
    
    // Should match the "Single digit" case
    assert_eq!(format!("{}", result), "Single digit");
}

#[test]
fn test_edge_cases_range_patterns() {
    // Test boundary values
    let test_cases = vec![
        (0, "Zero"),
        (1, "Minor"),
        (17, "Minor"),
        (18, "Adult"),
        (65, "Adult"),
        (66, "Senior"),
        (100, "Senior"),
    ];
    
    for (age, expected) in test_cases {
        let code = format!(r#"
            let age = {}
            match age {{
                0 => "Zero",
                1..=17 => "Minor",
                18..=65 => "Adult",
                _ => "Senior"
            }}
        "#, age);
        
        let mut parser = Parser::new(&code);
        let ast = parser.parse().expect("Should parse");
        
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval_expr(&ast).expect("Should evaluate");
        
        assert_eq!(format!("{}", result), expected,
                   "Age {} should be {}", age, expected);
    }
}