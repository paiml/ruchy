#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args
)]

use proptest::prelude::*;
use ruchy::{Parser, Transpiler};

/// Generate valid Ruchy identifiers
fn valid_identifier() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,20}".prop_map(String::from)
}

/// Generate valid integers
fn valid_integer() -> impl Strategy<Value = i64> {
    any::<i64>()
}

/// Generate valid strings
fn valid_string() -> impl Strategy<Value = String> {
    ".*".prop_map(|s: String| s.chars().take(100).collect())
}

/// Generate simple expressions
fn simple_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        valid_integer().prop_map(|i| i.to_string()),
        valid_string().prop_map(|s| format!(r#""{}""#, s.escape_default())),
        valid_identifier(),
    ]
}

proptest! {
    /// Test that valid let statements always parse
    #[test]
    fn prop_let_statement_parses(
        var in valid_identifier(),
        value in simple_expr()
    ) {
        let input = format!("let {} = {}", var, value);
        let mut parser = Parser::new(&input);

        // Should parse without panicking
        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse: {}", input);
    }

    /// Test that parsed code can be transpiled
    #[test]
    fn prop_parse_transpile_pipeline(
        var in valid_identifier(),
        value in valid_integer()
    ) {
        let input = format!("let {} = {}", var, value);
        let mut parser = Parser::new(&input);

        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            prop_assert!(result.is_ok(), "Failed to transpile: {}", input);
        }
    }

    /// Test function definitions
    #[test]
    fn prop_function_definition(
        name in valid_identifier(),
        param in valid_identifier(),
        body in valid_integer()
    ) {
        let input = format!(
            "fn {}({}: i32) -> i32 {{ {} }}",
            name, param, body
        );
        let mut parser = Parser::new(&input);

        let result = parser.parse();
        prop_assert!(result.is_ok(), "Failed to parse function: {}", input);
    }

    /// Test binary operations maintain precedence
    #[test]
    fn prop_binary_op_precedence(
        a in 1i32..100,
        b in 1i32..100,
        c in 1i32..100
    ) {
        let input = format!("{} + {} * {}", a, b, c);
        let mut parser = Parser::new(&input);

        if let Ok(ast) = parser.parse_expr() {
            let transpiler = Transpiler::new();
            if let Ok(rust_code) = transpiler.transpile(&ast) {
                let rust_str = rust_code.to_string();
                // The multiplication should have higher precedence
                prop_assert!(
                    rust_str.contains(&format!("({} * {})", b, c)) ||
                    rust_str.contains(&format!("{} + {} * {}", a, b, c)),
                    "Precedence not preserved in: {}", rust_code
                );
            }
        }
    }

    /// Test string literals are properly escaped
    #[test]
    fn prop_string_escaping(s in ".*") {
        let input = format!(r#"let x = "{}""#, s.escape_default());
        let mut parser = Parser::new(&input);

        // Should handle any string content
        let result = parser.parse();
        if result.is_ok() {
            let transpiler = Transpiler::new();
            let ast = result.unwrap();
            let transpiled = transpiler.transpile(&ast);
            prop_assert!(transpiled.is_ok(), "Failed to transpile string: {:?}", s);
        }
    }
}
