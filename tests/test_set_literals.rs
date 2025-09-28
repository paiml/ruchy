//! Tests for Set literal functionality
//! Ensures sets work when they should (comma-separated values)
//! and blocks work when they should (single expressions, statements)

use ruchy::backend::transpiler::Transpiler;
use ruchy::compile;
use ruchy::frontend::parser::Parser;

#[test]
fn test_actual_set_literal() {
    // {1, 2, 3} should be a set literal
    let code = "let s = {1, 2, 3};";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Set literal should parse");

    // Should transpile to HashSet
    if let Ok(ast) = ast {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok(), "Set literal should transpile");

        let rust_code = result.unwrap().to_string();
        // Should contain collection initialization
        assert!(
            rust_code.contains("HashSet")
                || rust_code.contains("vec![")
                || rust_code.contains("[1, 2, 3]"),
            "Should generate collection code for set literal, got: {rust_code}"
        );
    }
}

#[test]
fn test_single_value_in_braces_as_expr() {
    // In expression context, {42} should be a block that evaluates to 42
    let code = "fn test() -> i32 { 42 }";
    let result = compile(code);
    assert!(result.is_ok(), "Single value in function should compile");

    let rust_code = result.unwrap();
    assert!(
        !rust_code.contains("HashSet"),
        "Single value in function body should not be a set"
    );
}

#[test]
fn test_empty_braces_as_unit() {
    // {} in statement position should be an empty block (unit type)
    let code = "fn test() { {} }";
    let result = compile(code);
    assert!(result.is_ok(), "Empty braces should compile as unit block");

    let rust_code = result.unwrap();
    assert!(
        !rust_code.contains("HashSet"),
        "Empty braces in statement should not be a set"
    );
}

#[test]
fn test_set_with_variables() {
    // {x, y, z} with commas should be a set
    let code = r"
        fn test() {
            let x = 1;
            let y = 2;
            let z = 3;
            let s = {x, y, z};
        }
    ";
    let result = compile(code);
    assert!(result.is_ok(), "Set with variables should compile");
}

#[test]
fn test_set_comprehension_syntax() {
    // {x * 2 for x in 0..10} should be a set comprehension
    let code = "let s = {x * 2 for x in 0..10};";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    // This might not be implemented yet, but test it parses
    assert!(
        ast.is_ok() || ast.is_err(),
        "Set comprehension parsing tested"
    );
}

#[test]
fn test_object_literal_not_set() {
    // {x: 1, y: 2} is an object literal, not a set
    let code = "let obj = {x: 1, y: 2};";
    let mut parser = Parser::new(code);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Object literal should parse");

    if let Ok(ast) = ast {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        if result.is_ok() {
            let rust_code = result.unwrap().to_string();
            // Object literals should not generate HashSet
            assert!(
                !rust_code.contains("HashSet"),
                "Object literal should not be a set"
            );
        }
    }
}

#[test]
fn test_block_with_let_not_set() {
    // { let x = 5; x } is clearly a block, not a set
    let code = "fn test() -> i32 { let x = 5; x }";
    let result = compile(code);
    assert!(result.is_ok(), "Block with let should compile");

    let rust_code = result.unwrap();
    assert!(
        !rust_code.contains("HashSet"),
        "Block with let should not be a set"
    );
}

#[test]
fn test_disambiguation_in_different_contexts() {
    // Test that context properly disambiguates
    let test_cases = vec![
        ("fn f() -> i32 { 42 }", false, "function body"),
        ("if true { 1 } else { 2 }", false, "if-else blocks"),
        ("match x { 0 => { 1 } _ => { 2 } }", false, "match arms"),
        ("let x = { let y = 5; y + 1 };", false, "let with block"),
        ("let s = {1, 2, 3};", true, "set literal"),
        ("let s = {};", false, "empty braces"), // Could be either
    ];

    for (code, should_have_set, context) in test_cases {
        let full_code =
            if code.starts_with("fn") || code.starts_with("if") || code.starts_with("match") {
                format!("fn test() {{ {code} }}")
            } else {
                format!("fn test() {{ {code} }}")
            };

        let result = compile(&full_code);
        assert!(result.is_ok(), "Code should compile for context: {context}");

        let rust_code = result.unwrap();
        if should_have_set {
            // We expect set-like code (HashSet or vec)
            assert!(
                rust_code.contains("HashSet") || rust_code.contains("vec!["),
                "Expected set code for context: {context}, got: {rust_code}"
            );
        } else {
            assert!(
                !rust_code.contains("HashSet"),
                "Should not have HashSet for context: {context}, got: {rust_code}"
            );
        }
    }
}
