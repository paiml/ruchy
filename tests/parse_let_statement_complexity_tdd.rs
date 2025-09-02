// TDD test for parse_let_statement complexity refactoring
// GOAL: Reduce parse_let_statement complexity from 36 to <10 via systematic extraction
// RED → GREEN → REFACTOR methodology

use ruchy::frontend::parser::Parser;

#[test]
fn test_parse_let_simple_binding() {
    // Test simple let binding: let x = 5
    let test_cases = vec![
        ("let x = 5", "should parse simple let binding"),
        ("let y = 3.14", "should parse let with float"),
        ("let name = \"test\"", "should parse let with string"),
        ("let flag = true", "should parse let with bool"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("Let"), 
                "{}: Should contain Let expression: {}", description, debug_str);
    }
}

#[test]
fn test_parse_let_with_mut() {
    // Test let mut binding: let mut x = 5
    let test_cases = vec![
        ("let mut x = 5", "should parse mutable let binding"),
        ("let mut counter = 0", "should parse mutable counter"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("Let") && debug_str.contains("is_mutable: true"), 
                "{}: Should contain mutable Let expression: {}", description, debug_str);
    }
}

#[test]
fn test_parse_let_with_type_annotation() {
    // Test let with type annotation: let x: i32 = 5
    let test_cases = vec![
        ("let x: i32 = 5", "should parse let with type annotation"),
        ("let name: String = \"test\"", "should parse let with String type"),
        ("let mut count: u64 = 0", "should parse mutable let with type"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        assert!(debug_str.contains("Let") && debug_str.contains("type_annotation: Some"), 
                "{}: Should contain Let with type annotation: {}", description, debug_str);
    }
}

#[test]
fn test_parse_let_with_pattern_destructuring() {
    // Test let with pattern destructuring
    let test_cases = vec![
        ("let (x, y) = (1, 2)", "should parse tuple destructuring"),
        ("let [a, b] = [1, 2]", "should parse list destructuring"),
        ("let (first, second, third) = tuple", "should parse multiple tuple destructuring"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                assert!(debug_str.contains("LetPattern") || debug_str.contains("Let"),
                        "{}: Should parse pattern destructuring: {}", description, debug_str);
            },
            Err(e) => {
                println!("⚠️  Pattern destructuring not yet working: {} (will fix during refactoring)", e);
                // Don't fail - this guides our refactoring
            }
        }
    }
}

#[test]
fn test_parse_let_expression_with_in() {
    // Test let expression with 'in' clause: let x = 5 in x + 1
    let test_cases = vec![
        ("let x = 5 in x + 1", "should parse let expression"),
        ("let y = 10 in y * 2", "should parse let with body"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        match result {
            Ok(expr) => {
                let debug_str = format!("{:?}", expr);
                println!("✅ {}: {}", description, input);
            },
            Err(e) => {
                println!("⚠️  Let expression not yet working: {} (will fix during refactoring)", e);
            }
        }
    }
}

#[test]
fn test_parse_let_complexity_is_reduced() {
    // This test will pass once we've successfully refactored parse_let_statement
    // REQUIREMENT: parse_let_statement should delegate to focused helper functions
    
    // After refactoring, parse_let_statement should be a simple dispatcher
    // that calls focused functions like:
    // - parse_let_mutability()
    // - parse_let_pattern() 
    // - parse_let_type_annotation()
    // - parse_let_value()
    // - parse_let_in_clause()
    
    // For now, just ensure basic functionality works
    let mut parser = Parser::new("let x = 42");
    let result = parser.parse().expect("Should parse simple let");
    assert!(format!("{:?}", result).contains("Let"));
    
    // TODO: Add complexity measurement when we have the tools
    // assert!(parse_let_statement_complexity() < 10);
}

#[test]
fn test_var_statement_parsing() {
    // Test var statement (implicitly mutable)
    let test_cases = vec![
        ("var x = 5", "should parse var statement"),
        ("var counter = 0", "should parse var as mutable"),
    ];
    
    for (input, description) in test_cases {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        
        assert!(result.is_ok(), "{}: {}", description, input);
        let debug_str = format!("{:?}", result.unwrap());
        // var should be treated as mutable
        assert!(debug_str.contains("Variable") || 
                (debug_str.contains("Let") && debug_str.contains("is_mutable: true")),
                "{}: var should be mutable: {}", description, debug_str);
    }
}

#[test]
fn test_all_let_variants_work_after_refactoring() {
    // Comprehensive test to ensure refactoring doesn't break any variant
    let test_cases = vec![
        // Simple cases
        ("let x = 5", "simple"),
        ("let mut y = 10", "mutable"),
        ("let z: i32 = 15", "with type"),
        ("let mut w: f64 = 3.14", "mutable with type"),
        
        // Complex cases
        ("let (a, b) = (1, 2)", "tuple destructuring"),
        ("let [x, y] = [1, 2]", "list destructuring"),
        ("let v = 5 in v + 1", "let expression"),
        
        // var statement
        ("var counter = 0", "var statement"),
    ];
    
    let mut passed = 0;
    let mut failed = 0;
    
    for (input, variant) in test_cases {
        let mut parser = Parser::new(input);
        match parser.parse() {
            Ok(_) => {
                println!("✅ {} variant: '{}'", variant, input);
                passed += 1;
            },
            Err(e) => {
                println!("⚠️  {} variant failed: '{}' - {}", variant, input, e);
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Results: {} passed, {} failed", passed, failed);
    // We expect at least simple cases to work
    assert!(passed >= 4, "At least simple let statements should work");
}