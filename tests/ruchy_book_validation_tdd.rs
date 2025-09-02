//! RUCHY-BOOK VALIDATION - TDD Test Suite 
//! Validates that ALL ruchy-book examples work with new parser fix

use ruchy::{Parser, Transpiler};

#[test]
fn test_ruchy_book_basic_arithmetic() {
    let examples = vec![
        "let basic_math = 2 + 2;",
        "let percentage_calc = 100.0 * 1.08;", 
        "let compound_interest = 1000.0 * (1.0 + 0.05) * (1.0 + 0.05);",
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
        
        // Validate AST structure
        let ast = result.unwrap();
        match &ast.kind {
            ruchy::frontend::ast::ExprKind::Let { name, .. } => {
                println!("✅ Parsed let statement: {}", name);
            }
            _ => panic!("Expected Let AST node for: {}", example)
        }
    }
}

#[test] 
fn test_ruchy_book_string_operations() {
    let examples = vec![
        r#"let string_concat = "Hello" + " " + "World";"#,
        r#"let string_length = "test".len();"#,
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
    }
}

#[test]
fn test_ruchy_book_conditionals() {
    let examples = vec![
        "let max_value = if 10 > 5 { 10 } else { 5 };",
        r#"let boolean_result = if 100.0 > 50.0 { "expensive" } else { "cheap" };"#,
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
    }
}

#[test]
fn test_ruchy_book_variables_calculations() {
    let examples = vec![
        "let price = 99.99;",
        "let tax_rate = 0.08;",
        "let total = price * (1.0 + tax_rate);",
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
    }
}

#[test]
fn test_ruchy_book_print_statements() {
    let examples = vec![
        r#"println("Basic calculations work!");"#,
        "println(total);",
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
    }
}

#[test]
fn test_ruchy_book_complex_expressions() {
    let code = r#"let result = if price > 100.0 { 
    price * 0.9  // 10% discount
} else { 
    price * (1.0 + tax_rate)  // add tax
};"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    assert!(result.is_ok(), "Should parse complex expression: {:?}", result.err());
}

#[test]
fn test_ruchy_book_complete_file_parsing() {
    // Test parsing the entire ruchy-book file
    let file_path = "/home/noah/src/ruchy-book/one_liner_tests.ruchy";
    let content = std::fs::read_to_string(file_path)
        .expect("Should read ruchy-book file");
    
    // Split into individual statements (simplified)
    let lines: Vec<&str> = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
        .collect();
    
    println!("Testing {} lines from ruchy-book", lines.len());
    
    for (i, line) in lines.iter().enumerate() {
        let mut parser = Parser::new(line);
        let result = parser.parse();
        
        if result.is_err() {
            println!("Failed line {}: {}", i + 1, line);
        }
        
        assert!(result.is_ok(), 
            "Line {} should parse: '{}' - Error: {:?}", 
            i + 1, line, result.err()
        );
    }
}

#[test]
fn test_ruchy_book_transpilation_works() {
    // Test that ruchy-book examples not only parse but also transpile
    let examples = vec![
        "let x = 42;",
        "let name = \"Alice\";", 
        "let result = 2 + 3;",
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let ast = parser.parse().expect(&format!("Should parse: {}", example));
        
        let transpiler = Transpiler::new();
        let transpile_result = transpiler.transpile_to_program(&ast);
        
        assert!(transpile_result.is_ok(), 
            "Should transpile '{}': {:?}", example, transpile_result.err());
        
        let code = transpile_result.unwrap().to_string();
        println!("Generated for '{}': {}", example, code);
        
        // Should generate valid-looking Rust (basic checks)
        assert!(code.contains("fn main"), "Should have main function");
        assert!(!code.contains("let result = let"), "Should not double-wrap let statements");
    }
}

#[test]
fn test_github_issue_17_specific_examples() {
    // Test the exact examples mentioned in GitHub Issue #17
    let failing_examples = vec![
        "let basic_math = 2 + 2;",
        r#"let string_concat = "Hello" + " " + "World";"#,
        r#"println("Basic calculations work!");"#,
    ];
    
    println!("Testing GitHub Issue #17 specific examples...");
    
    for example in failing_examples {
        println!("Testing: {}", example);
        
        let mut parser = Parser::new(example);
        let result = parser.parse();
        
        // These should now pass (was failing with "Unexpected token: Let")
        assert!(result.is_ok(), 
            "GitHub Issue #17 example should now work: '{}' - Error: {:?}", 
            example, result.err()
        );
        
        println!("✅ FIXED: {}", example);
    }
    
    println!("🎉 GitHub Issue #17 examples all working!");
}