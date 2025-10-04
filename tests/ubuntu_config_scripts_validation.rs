//! Ubuntu Config Scripts Validation Tests
//! Ensuring compatibility with real-world Ruchy usage

use ruchy::{Parser, Transpiler};

#[test]
fn test_ubuntu_config_basic_if_expression() {
    // Check pattern from ubuntu-config-scripts: if expression without semicolon
    let code = r#"if 2 + 3 == 5 { "passed" } else { "failed" }"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse ubuntu-config if expression: '{}' - Error: {:?}",
        code,
        result.err()
    );

    // Check transpilation too
    let ast = result.unwrap();
    let mut transpiler = Transpiler::new();
    let transpile_result = transpiler.transpile_to_program(&ast);

    assert!(
        transpile_result.is_ok(),
        "Should transpile ubuntu-config if expression: {:?}",
        transpile_result.err()
    );

    println!("Generated: {}", transpile_result.unwrap().to_string());
}

#[test]
fn test_ubuntu_config_comparison_operators() {
    // Check various comparison patterns used in ubuntu-config-scripts
    let examples = vec![
        "2 + 3 == 5",
        "result == 5",
        "10 > 5",
        "price >= 100.0",
        "name != \"\"",
    ];

    for code in examples {
        println!("Testing comparison: {}", code);
        let mut parser = Parser::new(code);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Should parse comparison: '{}' - Error: {:?}",
            code,
            result.err()
        );
    }
}

#[test]
fn test_ubuntu_config_string_literals() {
    // Check string patterns from ubuntu-config-scripts
    let examples = vec![
        r#""✅ Test passed""#,
        r#""❌ Test failed""#,
        r#""Hello, World!""#,
        r#""Bridge Test""#,
    ];

    for code in examples {
        println!("Testing string literal: {}", code);
        let mut parser = Parser::new(code);
        let result = parser.parse();

        assert!(
            result.is_ok(),
            "Should parse string literal: '{}' - Error: {:?}",
            code,
            result.err()
        );
    }
}

#[test]
fn test_ubuntu_config_function_definition() {
    // Check function definition pattern from ubuntu-config-scripts
    let code = r#"fun test_addition() {
        let result = 2 + 3
    }"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse ubuntu-config function: Error: {:?}",
        result.err()
    );
}

#[test]
fn test_ubuntu_config_real_world_patterns() {
    // Check actual patterns from ubuntu-config-scripts files
    let patterns = vec![
        // Basic computation
        ("let result = 2 + 3", "basic computation"),
        // Function call (simplified)
        (r#"println("Hello")"#, "function call"),
        // Object/struct syntax (if supported)
        (
            "let config = { name: \"test\", version: 1 }",
            "object literal",
        ),
    ];

    for (code, description) in patterns {
        println!("Testing {} pattern: {}", description, code);
        let mut parser = Parser::new(code);
        let result = parser.parse();

        // For now, just check if it parses - we'll refine based on results
        if result.is_err() {
            println!("⚠️  {} not yet supported: {:?}", description, result.err());
        } else {
            println!("✅ {} works!", description);
        }
    }
}

#[test]
fn test_ubuntu_config_block_expressions() {
    // Check that block expressions work (needed for function bodies)
    let code = r#"{ let x = 5; x + 1 }"#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse block expression: Error: {:?}",
        result.err()
    );
}
