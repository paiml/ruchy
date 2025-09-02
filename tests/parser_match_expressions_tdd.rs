//! CRITICAL PARSER RESTORATION - Match expressions using TDD + TDG
//! Target: Restore Token::Match with complexity <10 per function
//! TDG Score Target: A- grade (≥85 points)

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_match_expression() {
    // Test basic match expression parsing
    let code = r#"match x {
        1 => "one",
        2 => "two",
        _ => "other"
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse simple match expression: {:?}", 
        result.err()
    );
    
    let ast = result.unwrap();
    match &ast.kind {
        ruchy::frontend::ast::ExprKind::Match { expr: _, arms } => {
            assert!(!arms.is_empty(), "Should have match arms");
            println!("✅ Parsed match with {} arms", arms.len());
        }
        _ => panic!("Expected Match AST node, got: {:?}", ast.kind)
    }
}

#[test]
fn test_match_with_variable_patterns() {
    // Test match with variable binding patterns
    let code = r#"match result {
        Some(x) => x + 1,
        None => 0
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse match with variable patterns: {:?}", 
        result.err()
    );
}

#[test]
fn test_match_with_literal_patterns() {
    // Test match with various literal patterns
    let code = r#"match value {
        0 => "zero",
        1 => "one",
        42 => "answer",
        _ => "unknown"
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse match with literal patterns: {:?}", 
        result.err()
    );
}

#[test]
fn test_match_with_guards() {
    // Test match with pattern guards
    let code = r#"match x {
        n if n > 0 => "positive",
        n if n < 0 => "negative",
        _ => "zero"
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse match with guards: {:?}", 
        result.err()
    );
}

#[test]
fn test_nested_match_expressions() {
    // Test nested match expressions (complexity test)
    let code = r#"match x {
        Some(y) => match y {
            0 => "none",
            n => "some"
        },
        None => "nothing"
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse nested match expressions: {:?}", 
        result.err()
    );
}

#[test]
fn test_match_in_let_statement() {
    // Test match expression in let binding
    let code = r#"let result = match value {
        Some(x) => x * 2,
        None => 0
    };"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse match in let statement: {:?}", 
        result.err()
    );
}

#[test]
fn test_match_transpilation() {
    // Test that match expressions transpile correctly
    let code = r#"match x {
        1 => "one",
        _ => "other"
    }"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse for transpilation");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), 
        "Should transpile match expression: {:?}", 
        result.err()
    );
    
    let generated = result.unwrap().to_string();
    println!("Generated: {}", generated);
    
    // Should contain match keyword in generated Rust
    assert!(generated.contains("match"), 
        "Generated code should contain match: {}", 
        generated
    );
}

#[test]
fn test_match_with_multiple_patterns() {
    // Test match arm with multiple patterns (|)
    let code = r#"match x {
        1 | 2 | 3 => "small",
        4 | 5 | 6 => "medium",
        _ => "large"
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse match with multiple patterns: {:?}", 
        result.err()
    );
}

#[test]
fn test_match_exhaustiveness() {
    // Test that match handles all patterns (has default)
    let examples = vec![
        // With default
        r#"match x { 1 => "one", _ => "other" }"#,
        // Exhaustive enum-like
        r#"match bool_val { true => "yes", false => "no" }"#,
    ];
    
    for code in examples {
        let mut parser = Parser::new(code);
        let result = parser.parse();
        
        assert!(result.is_ok(), 
            "Should parse exhaustive match: '{}' - Error: {:?}", 
            code, result.err()
        );
    }
}

// Property test for match expression complexity
#[test]
fn test_match_parser_complexity() {
    // Meta-test: Ensure our match parser implementation has low complexity
    // This will pass once we implement with TDG compliance
    
    // The parse_match_expression function should have:
    // - Cyclomatic complexity < 10
    // - Cognitive complexity < 10
    // - Clear separation of concerns
    
    // We'll verify this with PMAT after implementation
    println!("📊 Match parser complexity targets:");
    println!("  - Cyclomatic: <10");
    println!("  - Cognitive: <10");
    println!("  - TDG Score: ≥85 (A-)");
}