//! CRITICAL PARSER BUG TEST - Let statement handling (Issue #17)
//! This follows EXTREME TDD protocol for parser bugs

use ruchy::{Parser, Transpiler};

#[test]
fn test_basic_let_statement_parsing() {
    // Test that basic let statements can be parsed
    let mut parser = Parser::new("let x = 5");
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse let statement: {:?}", result.err());
    
    let ast = result.unwrap();
    // Should create a Let AST node
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { name, value, .. } => {
            assert_eq!(name, "x");
            // Value should be integer 5
            match &value.kind {
                ruchy::frontend::ast::ExprKind::Literal(ruchy::frontend::ast::Literal::Integer(5)) => {},
                _ => panic!("Expected integer 5, got: {:?}", value.kind)
            }
        }
        _ => panic!("Expected Let AST node, got: {:?}", ast.kind)
    }
}

#[test]
fn test_let_statement_compilation() {
    // Test that let statements compile to valid Rust
    let mut parser = Parser::new("let x = 5");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    println!("Generated code: {}", code); // Debug output
    
    // Should generate valid Rust with let statement
    assert!(code.contains("let x = 5"), "Should contain let statement");
    assert!(!code.contains("let result = let x"), "Should not double-wrap");
}

#[test]
fn test_ruchy_book_basic_math_example() {
    // Test first failing example from ruchy-book
    let code = "let basic_math = 2 + 2";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse ruchy-book example: {:?}", result.err());
}

#[test]
fn test_ruchy_book_string_concat_example() {
    // Test second failing example from ruchy-book
    let code = r#"let string_concat = "Hello" + " " + "World""#;
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse string concatenation: {:?}", result.err());
}

#[test]
fn test_ruchy_book_variable_declaration() {
    // Test variable declaration example from ruchy-book
    let code = "let price = 99.99";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse variable declaration: {:?}", result.err());
}

#[test]
fn test_let_with_type_annotation() {
    // Test let with explicit type annotation
    let code = "let x: int = 42";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse let with type annotation: {:?}", result.err());
    
    let ast = result.unwrap();
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { name, type_annotation, .. } => {
            assert_eq!(name, "x");
            assert!(type_annotation.is_some(), "Should have type annotation");
        }
        _ => panic!("Expected Let AST node, got: {:?}", ast.kind)
    }
}

#[test]
fn test_let_statement_end_to_end_compilation() {
    // Integration test: let statement should compile and run
    let code = "let x = 42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile without errors");
    
    // The generated Rust code should compile (we test compilation in CI)
    let generated = result.unwrap().to_string();
    
    // Should not contain malformed code patterns
    assert!(!generated.contains("let result = let"), "Should not double-wrap let statements");
    assert!(!generated.contains("let x = 42 ;"), "Should not add unnecessary semicolons");
}

#[test]
fn test_multiple_ruchy_book_examples_together() {
    // Test that multiple let statements from ruchy-book work together
    let code = r#"
        let basic_math = 2 + 2;
        let string_concat = "Hello" + " " + "World";
        println("Basic calculations work!");
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse multiple let statements: {:?}", result.err());
}

#[test]
fn test_let_statement_vs_let_expression() {
    // Test that we can distinguish between let statements and let expressions
    
    // Let statement (no 'in' clause)
    let mut parser1 = Parser::new("let x = 5");
    let result1 = parser1.parse();
    assert!(result1.is_ok(), "Should parse let statement");
    
    // Let expression (with 'in' clause) - if supported
    let mut parser2 = Parser::new("let x = 5 in x + 1");
    let result2 = parser2.parse();
    // This may fail if let expressions aren't implemented yet, but test documents the expected behavior
    if result2.is_ok() {
        let ast = result2.unwrap();
        match &ast.kind {
            ruchy::frontend::ast::ExprKind::Let { body, .. } => {
                // Body should not be unit for let expressions
                assert!(!matches!(body.kind, ruchy::frontend::ast::ExprKind::Literal(ruchy::frontend::ast::Literal::Unit)));
            }
            _ => panic!("Expected Let AST node")
        }
    }
}

#[test] 
fn test_let_statement_generates_valid_rust() {
    let mut parser = Parser::new("let x = 5");
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile successfully");
    let code = result.unwrap().to_string();
    
    println!("Generated: {}", code);
    
    // Should NOT generate invalid 'let result = let x' pattern
    assert!(!code.contains("let result = let"), "Should not wrap let statements in let result");
    
    // Should generate valid Rust let statement
    assert!(code.contains("let x = 5"), "Should contain valid let statement");
    assert!(code.contains("fn main"), "Should have main function");
}