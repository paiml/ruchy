//! CRITICAL PARSER BUG TEST - If expression handling 
//! Discovered during ruchy-book validation - TDD fix required

use ruchy::{Parser, Transpiler};

#[test]
fn test_basic_if_expression_parsing() {
    // Test that basic if expressions can be parsed
    let mut parser = Parser::new("if true { 42 } else { 0 }");
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse if expression: {:?}", result.err());
    
    let ast = result.unwrap();
    // Should create an If AST node
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::If { condition, then_branch, else_branch } => {
            println!("✅ Parsed if expression with condition, then, and else branches");
            assert!(else_branch.is_some(), "Should have else branch");
        }
        _ => panic!("Expected If AST node, got: {:?}", ast.kind)
    }
}

#[test]
fn test_if_expression_in_let_statement() {
    // Test the failing case from ruchy-book: let with if expression  
    let code = "let max_value = if 10 > 5 { 10 } else { 5 };";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse let with if expression: {:?}", result.err());
    
    let ast = result.unwrap();
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Let { name, value, .. } => {
            assert_eq!(name, "max_value");
            // Value should be an if expression
            match &value.kind {
                ruchy::frontend::ast::ExprKind::If { .. } => {
                    println!("✅ Let statement contains if expression");
                }
                _ => panic!("Expected If expression in let value, got: {:?}", value.kind)
            }
        }
        _ => panic!("Expected Let AST node, got: {:?}", ast.kind)
    }
}

#[test]
fn test_if_without_else() {
    // Test if without else clause
    let mut parser = Parser::new("if true { 42 }");
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse if without else: {:?}", result.err());
    
    let ast = result.unwrap();
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::If { else_branch, .. } => {
            assert!(else_branch.is_none(), "Should not have else branch");
        }
        _ => panic!("Expected If AST node")
    }
}

#[test]
fn test_nested_if_expressions() {
    // Test nested if expressions
    let code = "if true { if false { 1 } else { 2 } } else { 3 }";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse nested if expressions: {:?}", result.err());
}

#[test]
fn test_if_with_comparison_operators() {
    // Test if with various comparison operators
    let examples = vec![
        "if 10 > 5 { true } else { false }",
        "if x == y { 1 } else { 0 }",
        "if price >= 100.0 { \"expensive\" } else { \"cheap\" }",
    ];
    
    for example in examples {
        let mut parser = Parser::new(example);
        let result = parser.parse();
        assert!(result.is_ok(), "Should parse '{}': {:?}", example, result.err());
    }
}

#[test]
fn test_if_expression_transpilation() {
    // Test that if expressions transpile correctly
    let code = "if true { 42 } else { 0 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), "Should transpile if expression: {:?}", result.err());
    let generated = result.unwrap().to_string();
    
    println!("Generated: {}", generated);
    
    // Should contain if-else structure
    assert!(generated.contains("if"), "Should contain if keyword");
    assert!(generated.contains("else"), "Should contain else keyword");
}

#[test]
fn test_ruchy_book_failing_examples() {
    // Test the exact examples that are failing from ruchy-book validation
    let failing_examples = vec![
        "let max_value = if 10 > 5 { 10 } else { 5 };",
        r#"let boolean_result = if 100.0 > 50.0 { "expensive" } else { "cheap" };"#,
        r#"let result = if price > 100.0 { 
    price * 0.9
} else { 
    price * (1.0 + tax_rate)
};"#,
    ];
    
    for example in failing_examples {
        println!("Testing: {}", example);
        let mut parser = Parser::new(example);
        let result = parser.parse();
        
        assert!(result.is_ok(), 
            "Ruchy-book example should parse: '{}' - Error: {:?}", 
            example, result.err()
        );
    }
}

#[test]
fn test_if_expression_precedence() {
    // Test that if expressions work with operator precedence
    let code = "let x = 1 + if true { 2 } else { 3 } * 4;";
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), "Should parse if in arithmetic expression: {:?}", result.err());
}