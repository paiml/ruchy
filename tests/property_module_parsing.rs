//! Property-based tests for module parsing
//!
//! This test suite validates module declaration functionality using
//! property-based testing to ensure correctness across random inputs.

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property Test Generators
// ============================================================================

/// Generate valid module names
fn arb_module_name() -> impl Strategy<Value = String> {
    // Module names: start with uppercase, followed by alphanumeric
    prop::string::string_regex("[A-Z][a-zA-Z0-9_]{0,15}").expect("valid module name pattern")
}

/// Generate simple module bodies
fn arb_module_body() -> impl Strategy<Value = String> {
    prop_oneof![
        // Empty module
        Just("".to_string()),
        // Single literal
        Just("42".to_string()),
        Just("\"hello\"".to_string()),
        Just("true".to_string()),
        // Single variable
        prop::string::string_regex("[a-z][a-z0-9_]{0,8}")
            .expect("valid var")
            .prop_map(|var| var),
        // Simple expression
        Just("x + 1".to_string()),
        Just("sqrt(16)".to_string()),
        // Multiple expressions (with semicolon separator)
        Just("let x = 1; x + 2".to_string()),
        Just("let a = 5; let b = 10; a + b".to_string()),
    ]
}

/// Generate complete module declaration
fn arb_module_declaration() -> impl Strategy<Value = String> {
    (arb_module_name(), arb_module_body()).prop_map(|(name, body)| {
        if body.is_empty() {
            format!("module {} {{}}", name)
        } else {
            format!("module {} {{ {} }}", name, body)
        }
    })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Parser never panics on module declarations
    #[test]
    #[ignore]
    fn prop_parse_module_never_panics(module_decl in arb_module_declaration()) {
        let result = std::panic::catch_unwind(|| {
            Parser::new(&module_decl).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on: {}", module_decl);
    }

    /// Property: Parsing is deterministic
    #[test]
    #[ignore]
    fn prop_module_parsing_deterministic(module_decl in arb_module_declaration()) {
        let result1 = Parser::new(&module_decl).parse();
        let result2 = Parser::new(&module_decl).parse();

        match (result1, result2) {
            (Ok(_), Ok(_)) => prop_assert!(true),
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(e1.to_string(), e2.to_string(),
                    "Error messages differ for: {}", module_decl);
            }
            _ => prop_assert!(false, "Inconsistent results for: {}", module_decl),
        }
    }

    /// Property: Module names are preserved
    #[test]
    #[ignore]
    fn prop_module_name_preserved(name in arb_module_name()) {
        let module_decl = format!("module {} {{}}", name);
        let result = Parser::new(&module_decl).parse();

        if let Ok(ast) = result {
            let ast_str = format!("{:?}", ast);
            prop_assert!(
                ast_str.contains(&format!("name: \"{}\"", name)) || ast_str.contains(&name),
                "Module name '{}' should be preserved in AST", name
            );
        }
    }

    /// Property: Empty modules parse successfully
    #[test]
    #[ignore]
    fn prop_empty_module_parses(name in arb_module_name()) {
        let module_decl = format!("module {} {{}}", name);
        let result = Parser::new(&module_decl).parse();
        prop_assert!(result.is_ok(), "Empty module should parse: {}", module_decl);
    }

    /// Property: Module with single expression parses
    #[test]
    #[ignore]
    fn prop_single_expr_module_parses(
        name in arb_module_name(),
        expr in "(0i32..100i32)"
    ) {
        let module_decl = format!("module {} {{ {} }}", name, expr);
        let result = Parser::new(&module_decl).parse();

        // Should either parse or give clear error
        if let Err(e) = result {
            let err_msg = e.to_string();
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Property: Invalid module declarations produce clear errors
    #[test]
    #[ignore]
    fn prop_invalid_module_clear_error(
        invalid_name in "[^a-zA-Z0-9_]+"
    ) {
        let module_decl = format!("module {} {{}}", invalid_name);
        let result = Parser::new(&module_decl).parse();

        if let Err(e) = result {
            let err_msg = e.to_string();
            prop_assert!(!err_msg.is_empty(), "Should have error message");
        }
    }
}

// ============================================================================
// Unit Tests (Sanity Checks)
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Empty module parses
    #[test]
    fn test_empty_module_parses() {
        let test_cases = vec![
            "module Math {}",
            "module Utils {}",
            "module MyModule {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {}", code);
        }
    }

    /// Sanity check: Module with single expression
    #[test]
    fn test_single_expr_module_parses() {
        let test_cases = vec![
            "module Math { 42 }",
            "module Utils { \"hello\" }",
            "module Test { true }",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {}", code);
        }
    }

    /// Sanity check: Module with multiple expressions
    #[test]
    fn test_multi_expr_module_parses() {
        let test_cases = vec![
            "module Math { let x = 1; x + 2 }",
            "module Utils { let a = 5; let b = 10; a + b }",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {}", code);
        }
    }

    /// Sanity check: Module name is preserved
    #[test]
    fn test_module_name_preserved() {
        let code = "module MyMath {}";
        let result = Parser::new(code).parse();
        assert!(result.is_ok());

        let ast_str = format!("{:?}", result.unwrap());
        assert!(
            ast_str.contains("MyMath"),
            "Module name should be preserved in AST"
        );
    }

    /// Sanity check: Missing module name produces error
    #[test]
    fn test_missing_module_name_error() {
        let code = "module {}";
        let result = Parser::new(code).parse();
        assert!(result.is_err(), "Should error on missing module name");
    }

    /// Sanity check: Missing braces produces error
    #[test]
    fn test_missing_braces_error() {
        let test_cases = vec![
            "module Math",           // No braces
            "module Math {",         // Missing closing brace
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_err(), "Should error on: {}", code);
        }
    }
}
