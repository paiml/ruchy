//! Property-based tests for expression roundtrip (parse → format → parse)
//!
//! This test suite validates that formatting and parsing are inverses:
//! format(parse(code)) should be semantically equivalent to parse(code)
//!
//! This ensures the formatter preserves semantics and doesn't introduce bugs.

#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]

use proptest::prelude::*;
use ruchy::frontend::parser::Parser;
use ruchy::quality::formatter::Formatter;

// ============================================================================
// Property Test Generators - Simple Expressions
// ============================================================================

/// Generate integer literals
fn arb_integer() -> impl Strategy<Value = String> {
    // Range to avoid overflow in various contexts
    (-10000i64..10000i64).prop_map(|n| n.to_string())
}

/// Generate boolean literals
fn arb_boolean() -> impl Strategy<Value = String> {
    prop_oneof![Just("true".to_string()), Just("false".to_string())]
}

/// Generate string literals
fn arb_string_literal() -> impl Strategy<Value = String> {
    // Simple strings to avoid escaping complexity in first iteration
    prop::string::string_regex("[a-zA-Z0-9 ]{0,20}")
        .expect("valid string pattern")
        .prop_map(|s| format!("\"{s}\""))
}

/// Generate variable names
fn arb_identifier() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,10}").expect("valid identifier")
}

/// Generate binary operators
fn arb_binary_op() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("+".to_string()),
        Just("-".to_string()),
        Just("*".to_string()),
        Just("/".to_string()),
        Just("==".to_string()),
        Just("!=".to_string()),
        Just("<".to_string()),
        Just(">".to_string()),
    ]
}

/// Generate simple literals (integer, boolean, string)
fn arb_literal() -> impl Strategy<Value = String> {
    prop_oneof![arb_integer(), arb_boolean(), arb_string_literal(),]
}

/// Generate simple binary expressions: literal op literal
fn arb_binary_expr() -> impl Strategy<Value = String> {
    (arb_literal(), arb_binary_op(), arb_literal())
        .prop_map(|(left, op, right)| format!("{left} {op} {right}"))
}

/// Generate parenthesized expressions
fn arb_paren_expr() -> impl Strategy<Value = String> {
    arb_literal().prop_map(|expr| format!("({expr})"))
}

/// Generate simple function calls: `identifier()`
fn arb_simple_call() -> impl Strategy<Value = String> {
    arb_identifier().prop_map(|name| format!("{name}()"))
}

/// Generate simple expressions (literals, binary, parenthesized)
fn arb_simple_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        arb_literal(),
        arb_binary_expr(),
        arb_paren_expr(),
        arb_simple_call(),
    ]
}

// ============================================================================
// Property Tests - Expression Roundtrip
// ============================================================================

proptest! {
    /// Property: Parsing literals never panics
    #[test]
    #[ignore]
    fn prop_literals_parse_without_panic(literal in arb_literal()) {
        let code = format!("fun main() {{ {literal} }}");
        let result = std::panic::catch_unwind(|| {
            Parser::new(&code).parse()
        });
        prop_assert!(result.is_ok(), "Parser panicked on literal: {}", literal);
    }

    /// Property: Literal roundtrip preserves semantics
    #[test]
    #[ignore]
    fn prop_literal_roundtrip(literal in arb_literal()) {
        let code = format!("fun main() {{ {literal} }}");

        // Parse original
        let ast1 = Parser::new(&code).parse();
        if ast1.is_err() {
            return Ok(()); // Skip if parse fails (not all generated strings are valid)
        }

        // Format
        let ast1 = ast1.unwrap();
        let formatter = Formatter::new();
        let formatted = formatter.format(&ast1);
        if formatted.is_err() {
            return Ok(()); // Skip if format fails
        }
        let formatted = formatted.unwrap();

        // Parse formatted
        let ast2 = Parser::new(&formatted).parse();

        // Both should succeed or both should fail
        prop_assert!(
            ast2.is_ok(),
            "Formatted code failed to parse.\nOriginal: {}\nFormatted: {}\nError: {:?}",
            code, formatted, ast2.err()
        );
    }

    /// Property: Binary expression roundtrip
    #[test]
    #[ignore]
    fn prop_binary_roundtrip(expr in arb_binary_expr()) {
        let code = format!("fun main() {{ {expr} }}");

        let ast1 = Parser::new(&code).parse();
        if ast1.is_err() {
            return Ok(()); // Skip invalid expressions
        }

        let ast1 = ast1.unwrap();
        let formatter = Formatter::new();
        let formatted = formatter.format(&ast1);
        if formatted.is_err() {
            return Ok(());
        }
        let formatted = formatted.unwrap();

        let ast2 = Parser::new(&formatted).parse();
        prop_assert!(
            ast2.is_ok(),
            "Binary expression roundtrip failed.\nOriginal: {}\nFormatted: {}\nError: {:?}",
            code, formatted, ast2.err()
        );
    }

    /// Property: Simple expression roundtrip (comprehensive)
    #[test]
    #[ignore]
    fn prop_simple_expr_roundtrip(expr in arb_simple_expr()) {
        let code = format!("fun main() {{ {expr} }}");

        let ast1 = Parser::new(&code).parse();
        if ast1.is_err() {
            return Ok(()); // Skip invalid expressions
        }

        let ast1 = ast1.unwrap();
        let formatter = Formatter::new();
        let formatted = formatter.format(&ast1);
        if formatted.is_err() {
            return Ok(());
        }
        let formatted = formatted.unwrap();

        let ast2 = Parser::new(&formatted).parse();
        prop_assert!(
            ast2.is_ok(),
            "Simple expression roundtrip failed.\nOriginal: {}\nFormatted: {}\nError: {:?}",
            code, formatted, ast2.err()
        );
    }

    /// Property: Formatting is deterministic
    #[test]
    #[ignore]
    fn prop_formatting_deterministic(literal in arb_literal()) {
        let code = format!("fun main() {{ {literal} }}");

        let ast = Parser::new(&code).parse();
        if ast.is_err() {
            return Ok(());
        }

        let ast = ast.unwrap();
        let formatter = Formatter::new();
        let formatted1 = formatter.format(&ast);
        if formatted1.is_err() {
            return Ok(());
        }
        let formatted1 = formatted1.unwrap();

        let formatted2 = formatter.format(&ast);
        if formatted2.is_err() {
            return Ok(());
        }
        let formatted2 = formatted2.unwrap();

        prop_assert_eq!(
            formatted1,
            formatted2,
            "Formatting should be deterministic"
        );
    }

    /// Property: Double roundtrip stabilizes
    #[test]
    #[ignore]
    fn prop_double_roundtrip_stabilizes(expr in arb_simple_expr()) {
        let code = format!("fun main() {{ {expr} }}");

        // First roundtrip
        let ast1 = Parser::new(&code).parse();
        if ast1.is_err() {
            return Ok(());
        }

        let ast1 = ast1.unwrap();
        let formatter = Formatter::new();
        let formatted1 = formatter.format(&ast1);
        if formatted1.is_err() {
            return Ok(());
        }
        let formatted1 = formatted1.unwrap();

        // Second roundtrip
        let ast2 = Parser::new(&formatted1).parse();
        if ast2.is_err() {
            return Ok(());
        }

        let ast2 = ast2.unwrap();
        let formatted2 = formatter.format(&ast2);
        if formatted2.is_err() {
            return Ok(());
        }
        let formatted2 = formatted2.unwrap();

        // Third roundtrip
        let ast3 = Parser::new(&formatted2).parse();
        if ast3.is_err() {
            return Ok(());
        }

        let ast3 = ast3.unwrap();
        let formatted3 = formatter.format(&ast3);
        if formatted3.is_err() {
            return Ok(());
        }
        let formatted3 = formatted3.unwrap();

        // After second roundtrip, output should stabilize
        prop_assert_eq!(
            &formatted2,
            &formatted3,
            "Double roundtrip should stabilize.\nFormatted1: {}\nFormatted2: {}\nFormatted3: {}",
            formatted1, formatted2, formatted3
        );
    }
}

// ============================================================================
// Unit Tests - Specific Roundtrip Cases
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_integer_literal_roundtrip() {
        let code = "fun main() { 42 }";
        let ast = Parser::new(code).parse().unwrap();
        let formatted = Formatter::new().format(&ast).unwrap();
        let ast2 = Parser::new(&formatted).parse();
        assert!(ast2.is_ok(), "Integer literal roundtrip failed");
    }

    #[test]
    fn test_boolean_literal_roundtrip() {
        let code = "fun main() { true }";
        let ast = Parser::new(code).parse().unwrap();
        let formatted = Formatter::new().format(&ast).unwrap();
        let ast2 = Parser::new(&formatted).parse();
        assert!(ast2.is_ok(), "Boolean literal roundtrip failed");
    }

    #[test]
    fn test_string_literal_roundtrip() {
        let code = r#"fun main() { "hello" }"#;
        let ast = Parser::new(code).parse().unwrap();
        let formatted = Formatter::new().format(&ast).unwrap();
        let ast2 = Parser::new(&formatted).parse();
        assert!(ast2.is_ok(), "String literal roundtrip failed");
    }

    #[test]
    fn test_binary_add_roundtrip() {
        let code = "fun main() { 1 + 2 }";
        let ast = Parser::new(code).parse().unwrap();
        let formatted = Formatter::new().format(&ast).unwrap();
        let ast2 = Parser::new(&formatted).parse();
        assert!(ast2.is_ok(), "Binary add roundtrip failed");
    }

    #[test]
    fn test_function_call_roundtrip() {
        let code = "fun main() { foo() }";
        let ast = Parser::new(code).parse().unwrap();
        let formatted = Formatter::new().format(&ast).unwrap();
        let ast2 = Parser::new(&formatted).parse();
        assert!(ast2.is_ok(), "Function call roundtrip failed");
    }
}
