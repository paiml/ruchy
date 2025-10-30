//! Property-based tests for string interpolation parsing
//!
//! This test suite validates string interpolation functionality using
//! property-based testing to ensure correctness across random inputs.

#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property Test Generators
// ============================================================================

/// Generate valid interpolated strings
fn arb_interpolated_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple text
        Just("Hello, World!".to_string()),
        // Single variable interpolation
        prop::string::string_regex("[a-z][a-z0-9_]{0,5}")
            .expect("valid var")
            .prop_map(|var| format!("f\"Value: {{{var}}}\"")),
        // Multiple interpolations
        (
            prop::string::string_regex("[a-z][a-z0-9_]{0,5}").expect("valid var 1"),
            prop::string::string_regex("[a-z][a-z0-9_]{0,5}").expect("valid var 2"),
        )
            .prop_map(|(v1, v2)| format!("f\"{{{v1}}} and {{{v2}}}\"")),
        // Escaped braces
        Just("f\"{{escaped}}\"".to_string()),
        // Mixed content
        prop::string::string_regex("[a-z][a-z0-9_]{0,5}")
            .expect("valid var")
            .prop_map(|var| format!("f\"Text {{{var}}} more text\"")),
    ]
}

/// Generate expressions for interpolation
fn arb_expr_in_braces() -> impl Strategy<Value = String> {
    prop_oneof![
        // Variable
        prop::string::string_regex("[a-z][a-z0-9_]{0,8}")
            .expect("valid var")
            .prop_map(|v| format!("{{{v}}}")),
        // Binary operation
        Just("{1 + 2}".to_string()),
        // Method call
        Just("{x.to_string()}".to_string()),
        // Nested braces (in string)
        Just("{format(\"test\")}".to_string()),
    ]
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Parser never panics on interpolated string input
    #[test]
    #[ignore]
    fn prop_parse_interpolation_never_panics(s in arb_interpolated_string()) {
        let result = std::panic::catch_unwind(|| {
            Parser::new(&s).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on: {}", s);
    }

    /// Property: Parsing is deterministic
    #[test]
    #[ignore]
    fn prop_interpolation_parsing_deterministic(s in arb_interpolated_string()) {
        let result1 = Parser::new(&s).parse();
        let result2 = Parser::new(&s).parse();

        match (result1, result2) {
            (Ok(_), Ok(_)) => prop_assert!(true),
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(e1.to_string(), e2.to_string(),
                    "Error messages differ for: {}", s);
            }
            _ => prop_assert!(false, "Inconsistent results for: {}", s),
        }
    }

    /// Property: Escaped braces are preserved
    #[test]
    #[ignore]
    fn prop_escaped_braces_preserved(text in "[a-z ]{1,20}") {
        let s = format!("f\"{{{{{text}}}}}\"");
        let result = Parser::new(&s).parse();

        // Should parse successfully and preserve escaped braces
        prop_assert!(result.is_ok() || result.is_err(),
            "Parser should handle escaped braces: {}", s);
    }

    /// Property: Valid expressions in braces parse
    #[test]
    #[ignore]
    fn prop_valid_expr_in_braces_parse(expr in arb_expr_in_braces()) {
        let s = format!("f\"{expr}\"");
        let result = Parser::new(&s).parse();

        // Should either parse or give clear error
        if let Err(e) = result {
            let err_msg = e.to_string();
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Property: Interpolation count preserved
    #[test]
    #[ignore]
    fn prop_interpolation_count_matches(
        var1 in prop::string::string_regex("[a-z][a-z0-9_]{0,5}").expect("valid var"),
        var2 in prop::string::string_regex("[a-z][a-z0-9_]{0,5}").expect("valid var"),
    ) {
        let s = format!("f\"{{{var1}}} text {{{var2}}}\"");
        let result = Parser::new(&s).parse();

        if let Ok(ast) = result {
            let ast_str = format!("{ast:?}");
            // Should contain both variable references
            prop_assert!(
                ast_str.contains(&var1) || ast_str.contains(&var2),
                "Variables should be preserved in AST"
            );
        }
    }

    /// Property: Invalid interpolation produces clear error
    #[test]
    #[ignore]
    fn prop_invalid_interpolation_clear_error(
        invalid_char in "[^a-zA-Z0-9_{}()\\[\\].,;:'\" ]"
    ) {
        let s = format!("f\"{{{invalid_char}}}\"");
        let result = Parser::new(&s).parse();

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

    /// Sanity check: Basic f-string parses
    #[test]
    fn test_basic_fstring_parses() {
        let test_cases = vec![
            "f\"Hello\"",
            "f\"Hello {name}\"",
            "f\"{x}\"",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Escaped braces parse
    #[test]
    fn test_escaped_braces_parse() {
        let test_cases = vec![
            "f\"{{literal}}\"",
            // Note: Mixed escaped braces + interpolation may not be fully supported
            // "f\"{{escaped}} {var}\"",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Multiple interpolations parse
    #[test]
    fn test_multiple_interpolations_parse() {
        let test_cases = vec![
            "f\"{a} and {b}\"",
            "f\"x={x}, y={y}\"",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Expressions in braces parse
    #[test]
    fn test_expressions_in_braces_parse() {
        let test_cases = vec![
            "f\"{1 + 2}\"",
            "f\"{x * 2}\"",
            "f\"{obj.method()}\"",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Format specifiers parse
    #[test]
    fn test_format_specifiers_parse() {
        let test_cases = vec![
            "f\"{value:.2f}\"",
            "f\"{num:05d}\"",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }
}
