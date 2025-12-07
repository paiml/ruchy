//! Property-based tests for attribute and decorator parsing
#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]

//!
//! This test suite validates attribute/decorator parsing functionality
//! using property-based testing to ensure correctness across random inputs.

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property Test Generators
// ============================================================================

/// Generate valid decorator names
fn arb_decorator_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,15}").expect("valid decorator name regex")
}

/// Generate decorator arguments
fn arb_decorator_args() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::string::string_regex("[a-zA-Z0-9_]{1,10}").expect("valid arg regex"),
        0..5,
    )
}

/// Generate @-style decorator statement
fn arb_at_decorator() -> impl Strategy<Value = String> {
    (arb_decorator_name(), arb_decorator_args()).prop_map(|(name, args)| {
        if args.is_empty() {
            format!("@{name}\nfun test() {{}}")
        } else {
            format!("@{}({})\nfun test() {{}}", name, args.join(", "))
        }
    })
}

/// Generate #[...] style attribute statement
fn arb_rust_attribute() -> impl Strategy<Value = String> {
    (arb_decorator_name(), arb_decorator_args()).prop_map(|(name, args)| {
        if args.is_empty() {
            format!("#[{name}]\nfun test() {{}}")
        } else {
            format!("#[{}({})]\nfun test() {{}}", name, args.join(", "))
        }
    })
}

/// Generate multiple decorators
fn arb_multiple_decorators() -> impl Strategy<Value = String> {
    prop::collection::vec(arb_decorator_name(), 1..4).prop_map(|names| {
        let decorators = names
            .iter()
            .map(|name| format!("@{name}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{decorators}\nfun test() {{}}")
    })
}

// ============================================================================
// Property Tests
// ============================================================================

proptest! {
    /// Property: Parser never panics on @-style decorator input
    #[test]
    #[ignore]
    fn prop_parse_at_decorator_never_panics(decorator in arb_at_decorator()) {
        let result = std::panic::catch_unwind(|| {
            Parser::new(&decorator).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on decorator: {}", decorator);
    }

    /// Property: Parser never panics on #[...] attribute input
    #[test]
    #[ignore]
    fn prop_parse_rust_attribute_never_panics(attr in arb_rust_attribute()) {
        let result = std::panic::catch_unwind(|| {
            Parser::new(&attr).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on attribute: {}", attr);
    }

    /// Property: Parser never panics on multiple decorators
    #[test]
    #[ignore]
    fn prop_parse_multiple_decorators_never_panics(code in arb_multiple_decorators()) {
        let result = std::panic::catch_unwind(|| {
            Parser::new(&code).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on multiple decorators: {}", code);
    }

    /// Property: Parsing is deterministic
    #[test]
    #[ignore]
    fn prop_decorator_parsing_deterministic(decorator in arb_at_decorator()) {
        let result1 = Parser::new(&decorator).parse();
        let result2 = Parser::new(&decorator).parse();

        match (result1, result2) {
            (Ok(_), Ok(_)) => prop_assert!(true),
            (Err(e1), Err(e2)) => {
                prop_assert_eq!(e1.to_string(), e2.to_string(),
                    "Error messages differ for same input: {}", decorator);
            }
            _ => prop_assert!(false, "Inconsistent parse results for: {}", decorator),
        }
    }

    /// Property: Decorator names are preserved
    #[test]
    #[ignore]
    fn prop_decorator_name_preservation(name in arb_decorator_name()) {
        let code = format!("@{name}\nfun test() {{}}");
        let result = Parser::new(&code).parse();

        if let Ok(ast) = result {
            let ast_str = format!("{ast:?}");
            prop_assert!(
                ast_str.contains(&name),
                "Decorator name '{}' not preserved in AST", name
            );
        }
    }

    /// Property: Invalid decorators produce clear errors
    #[test]
    #[ignore]
    fn prop_invalid_decorator_clear_errors(invalid_char in "[^a-zA-Z0-9_@#\\[\\](),\\s]") {
        let code = format!("@test{invalid_char}\nfun test() {{}}");
        let result = Parser::new(&code).parse();

        if let Err(e) = result {
            let error_msg = e.to_string();
            prop_assert!(
                !error_msg.is_empty(),
                "Error message should not be empty"
            );
        }
    }
}

// ============================================================================
// Unit Tests (Sanity Checks)
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// Sanity check: Basic @-style decorators parse
    #[test]
    fn test_basic_at_decorators_parse() {
        let test_cases = vec![
            "@test\nfun foo() {}",
            "@deprecated\nfun bar() {}",
            "@inline\nfun baz() {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Decorators with arguments parse
    #[test]
    fn test_decorators_with_args_parse() {
        let test_cases = vec![
            "@test(quick)\nfun foo() {}",
            "@deprecated(since)\nfun bar() {}",
            "@timeout(fast)\nfun baz() {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Rust-style attributes parse
    #[test]
    #[ignore = "Rust-style attributes not fully supported"]
    fn test_rust_attributes_parse() {
        let test_cases = vec![
            "#[test]\nfun foo() {}",
            "#[inline]\nfun bar() {}",
            "#[derive(Debug)]\nstruct Point {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Multiple decorators parse
    #[test]
    fn test_multiple_decorators_parse() {
        let test_cases = vec![
            "@test\n@inline\nfun foo() {}",
            "@deprecated\n@test\n@slow\nfun bar() {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }

    /// Sanity check: Mixed attribute styles parse
    #[test]
    #[ignore = "Mixed attributes not fully supported"]
    fn test_mixed_attributes_parse() {
        let test_cases = vec![
            "@test\n#[inline]\nfun foo() {}",
            "#[derive(Debug)]\n@test\nstruct Point {}",
        ];

        for code in test_cases {
            let result = Parser::new(code).parse();
            assert!(result.is_ok(), "Failed to parse: {code}");
        }
    }
}
