//! CRITICAL TRANSPILER BUG TEST - Invalid let statement generation
//! TDD fix for GitHub Issue #17 transpiler aspect: Invalid Rust code generation

use ruchy::{Parser, Transpiler};

#[test]
fn test_simple_let_statement_transpilation() {
    // Check that simple let statements transpile to valid Rust
    let code = "let x = 5;";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse let statement");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Should transpile let statement: {:?}",
        result.err()
    );
    let generated = result.unwrap().to_string();

    println!("Generated for '{}': {}", code, generated);

    // Should NOT generate invalid Rust like "let result = let x = 5"
    assert!(
        !generated.contains("let result = let"),
        "Should not generate invalid nested let: {}",
        generated
    );

    // Should generate valid Rust
    assert!(
        generated.contains("let x = 5"),
        "Should contain valid let statement: {}",
        generated
    );
}

#[test]
fn test_let_expression_vs_statement() {
    // Check the difference between let expressions and let statements
    let statement = "let x = 5;"; // Statement (ends with semicolon)
    let expression = "let x = 5 in x"; // Expression (has 'in' clause)

    for (code, desc) in [(statement, "statement"), (expression, "expression")] {
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect(&format!("Should parse let {}", desc));

        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&ast);

        assert!(
            result.is_ok(),
            "Should transpile let {}: {:?}",
            desc,
            result.err()
        );
        let generated = result.unwrap().to_string();

        println!("Generated for '{}' ({}): {}", code, desc, generated);

        // Should NOT generate invalid nested lets
        assert!(
            !generated.contains("let result = let"),
            "Should not generate invalid nested let for {}: {}",
            desc,
            generated
        );
    }
}

#[test]
fn test_nested_let_expressions() {
    // Check nested let expressions (valid Ruchy, should generate valid Rust)
    let code = "let x = let y = 5 in y * 2 in x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse nested let expressions");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);

    assert!(
        result.is_ok(),
        "Should transpile nested let expressions: {:?}",
        result.err()
    );
    let generated = result.unwrap().to_string();

    println!("Generated for nested lets: {}", generated);

    // Should generate valid Rust (no invalid "let result = let" patterns)
    assert!(
        !generated.contains("let result = let"),
        "Should not generate invalid nested let: {}",
        generated
    );
}

#[test]
fn test_github_issue_17_specific_case() {
    // Check the exact issue described in GitHub Issue #17
    // The transpiler was generating: let result = let x = 5
    let examples = vec![
        "let x = 5;",
        "let name = \"Alice\";",
        "let result = 2 + 3;",
        "let sum = let a = 1 in let b = 2 in a + b;", // Complex nested case
    ];

    for code in examples {
        println!("Testing GitHub Issue #17 case: {}", code);

        let mut parser = Parser::new(code);
        let ast = parser.parse().expect(&format!("Should parse: {}", code));

        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&ast);

        assert!(
            result.is_ok(),
            "Should transpile: {} - Error: {:?}",
            code,
            result.err()
        );
        let generated = result.unwrap().to_string();

        println!("Generated: {}", generated);

        // CRITICAL: Should never generate "let result = let" which is invalid Rust
        assert!(
            !generated.contains("let result = let"),
            "❌ GITHUB ISSUE #17: Invalid Rust generated for '{}': {}",
            code,
            generated
        );

        // Should compile as valid Rust (basic syntax check)
        assert!(
            generated.starts_with("fn main"),
            "Should wrap in main function: {}",
            generated
        );
        assert!(
            generated.contains('{') && generated.contains('}'),
            "Should have proper block structure: {}",
            generated
        );
    }
}

#[test]
fn test_statement_vs_expression_detection() {
    // Check that the transpiler correctly detects statements vs expressions
    let statement_examples = vec![
        ("let x = 5;", true),      // Statement (semicolon, no 'in')
        ("let x = 5 in x", false), // Expression ('in' clause)
        ("42", false),             // Expression (no let)
        ("let x = 42;", true),     // Statement
    ];

    for (code, should_be_statement) in statement_examples {
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect(&format!("Should parse: {}", code));

        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile_to_program(&ast);

        assert!(result.is_ok(), "Should transpile: {}", code);
        let generated = result.unwrap().to_string();

        println!(
            "Code: '{}' -> Statement: {} -> Generated: {}",
            code, should_be_statement, generated
        );

        if should_be_statement {
            // Statements should be wrapped properly in main, not assigned to 'result'
            assert!(
                !generated.contains("let result = let"),
                "Statement should not be assigned to result: {}",
                generated
            );
        }
    }
}
