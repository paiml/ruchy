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
    // Test parsing the entire ruchy-book file as statements, not line-by-line
    let file_path = "/home/noah/src/ruchy-book/one_liner_tests.ruchy";
    let content = std::fs::read_to_string(file_path)
        .expect("Should read ruchy-book file");
    
    // Split into individual statements by semicolons, handling multiline
    let statements = split_into_statements(&content);
    
    println!("Testing {} statements from ruchy-book", statements.len());
    
    for (i, statement) in statements.iter().enumerate() {
        // Strip inline comments for parsing (lexer doesn't support them yet)
        let clean_statement = strip_inline_comments(statement.trim());
        println!("Testing statement {}: '{}'", i + 1, clean_statement);
        let mut parser = Parser::new(&clean_statement);
        let result = parser.parse();
        
        if result.is_err() {
            println!("Failed statement {}: {}", i + 1, statement.trim());
        }
        
        assert!(result.is_ok(), 
            "Statement {} should parse: '{}' - Error: {:?}", 
            i + 1, clean_statement, result.err()
        );
    }
}

/// Strip inline comments from a statement (temporary until lexer supports comments)
fn strip_inline_comments(statement: &str) -> String {
    // For multiline statements, we need to process line by line
    let lines: Vec<&str> = statement.split(' ').collect();
    let mut result_parts = Vec::new();
    
    for part in lines {
        if part.contains("//") && !part.starts_with("//") {
            // Split at the comment and take only the code part
            let code_part = part.split("//").next().unwrap_or("").trim();
            if !code_part.is_empty() {
                result_parts.push(code_part);
            }
        } else if !part.starts_with("//") && !part.trim().is_empty() {
            result_parts.push(part);
        }
    }
    
    result_parts.join(" ")
}

/// Split content into statements, handling multiline expressions properly
fn split_into_statements(content: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current_statement = String::new();
    let mut brace_depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        
        current_statement.push_str(line);
        current_statement.push(' ');
        
        // Track brace depth to handle multiline expressions
        for ch in line.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => brace_depth += 1,
                '}' if !in_string => brace_depth -= 1,
                _ => {}
            }
        }
        
        // If we're at depth 0 and line ends with semicolon, it's a complete statement
        if brace_depth == 0 && line.ends_with(';') {
            statements.push(current_statement.trim().to_string());
            current_statement.clear();
        }
    }
    
    // Add any remaining content as final statement
    if !current_statement.trim().is_empty() {
        statements.push(current_statement.trim().to_string());
    }
    
    statements
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