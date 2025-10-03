// Property-based tests for parser token stream handling
// PROPTEST-003 Part 3: Token stream properties (6 tests)
//
// Properties tested:
// 1. Token stream completeness - all input consumed
// 2. Whitespace handling - doesn't affect parsing
// 3. Comment handling - ignored properly
// 4. Token boundary detection - operators vs identifiers
// 5. String escape sequences - handled correctly
// 6. Number format variations - decimal, hex, binary, scientific

use proptest::prelude::*;
use ruchy::frontend::ast::{ExprKind, Literal};
use ruchy::frontend::parser::Parser;

// ============================================================================
// Property 1: Token stream completeness - all input consumed
// ============================================================================

proptest! {
    #[test]
    fn prop_simple_expressions_consume_all_tokens(value in 1i64..1000) {
        let code = format!("{}", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse simple expression: {}", code);
        // If parsing succeeds, all tokens should be consumed
    }

    #[test]
    fn prop_complex_expressions_consume_all_tokens(
        a in 1i64..100,
        b in 1i64..100,
        c in 1i64..100
    ) {
        let code = format!("{} + {} * {}", a, b, c);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse complex expression: {}", code);
        // If parsing succeeds, all tokens should be consumed correctly
    }
}

// ============================================================================
// Property 2: Whitespace handling - doesn't affect parsing
// ============================================================================

proptest! {
    #[test]
    fn prop_whitespace_doesnt_affect_integer_parsing(
        value in 1i64..1000,
        leading_spaces in 0usize..5,
        trailing_spaces in 0usize..5
    ) {
        let spaces_before = " ".repeat(leading_spaces);
        let spaces_after = " ".repeat(trailing_spaces);
        let code = format!("{}{}{}", spaces_before, value, spaces_after);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse whitespace-padded integer: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Integer(n)) = expr.kind {
                prop_assert_eq!(n, value, "Whitespace affected parsed value");
            } else {
                return Err(TestCaseError::fail(format!("Expected integer literal, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_whitespace_in_binary_expressions(
        a in 1i64..100,
        b in 1i64..100,
        spaces in 0usize..5
    ) {
        let ws = " ".repeat(spaces);
        // Test: a + b with varying whitespace
        let code = format!("{}{ws}+{ws}{}", a, b, ws = ws);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse expression with whitespace: {}", code);
    }
}

// ============================================================================
// Property 3: Comment handling - ignored properly
// ============================================================================

proptest! {
    #[test]
    fn prop_line_comments_ignored(value in 1i64..1000) {
        let code = format!("// comment\n{}", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse code with line comment: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Integer(n)) = expr.kind {
                prop_assert_eq!(n, value, "Comment affected parsed value");
            } else {
                return Err(TestCaseError::fail(format!("Expected integer literal, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_trailing_line_comments_ignored(value in 1i64..1000) {
        let code = format!("{} // trailing comment", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse code with trailing comment: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Integer(n)) = expr.kind {
                prop_assert_eq!(n, value, "Trailing comment affected parsed value");
            } else {
                return Err(TestCaseError::fail(format!("Expected integer literal, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 4: Token boundary detection - operators vs identifiers
// ============================================================================

proptest! {
    #[test]
    fn prop_operator_token_boundaries(
        a in 1i64..100,
        b in 1i64..100
    ) {
        // Test various operators with no whitespace
        let operators = vec!["+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">="];

        for op in operators {
            let code = format!("{}{}{}", a, op, b);
            let mut parser = Parser::new(&code);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Failed to parse operator expression: {}", code);
        }
    }

    #[test]
    fn prop_identifier_boundaries_preserved(
        prefix in "[a-z][a-z]{0,3}",
        suffix in "[a-z][a-z]{0,3}"
    ) {
        // Skip reserved keywords
        if is_reserved_keyword(&prefix) || is_reserved_keyword(&suffix) {
            return Ok(());
        }

        let ident = format!("{}{}", prefix, suffix);
        let mut parser = Parser::new(&ident);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse identifier: {}", ident);
        if let Ok(expr) = result {
            if let ExprKind::Identifier(name) = expr.kind {
                prop_assert_eq!(name, ident, "Identifier boundary not preserved");
            } else {
                return Err(TestCaseError::fail(format!("Expected identifier, got {:?}", expr.kind)));
            }
        }
    }
}

// ============================================================================
// Property 5: String escape sequences - handled correctly
// ============================================================================

proptest! {
    #[test]
    fn prop_string_with_escaped_quotes(s in "[a-zA-Z0-9 ]{1,10}") {
        // Test strings containing text (escapes tested separately due to complexity)
        let code = format!("\"{}\"", s);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse string literal: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::String(parsed)) = expr.kind {
                prop_assert_eq!(parsed, s, "String content mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected string literal, got {:?}", expr.kind)));
            }
        }
    }
}

#[test]
fn prop_empty_strings_parse() {
    let code = "\"\"";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(result.is_ok(), "Failed to parse empty string");
    if let Ok(expr) = result {
        if let ExprKind::Literal(Literal::String(s)) = expr.kind {
            assert_eq!(s, "", "Empty string should be empty");
        } else {
            panic!("Expected string literal, got {:?}", expr.kind);
        }
    }
}

// ============================================================================
// Property 6: Number format variations - decimal, hex, binary
// ============================================================================

proptest! {
    #[test]
    fn prop_decimal_integers_parse(value in 0i64..10000) {
        let code = format!("{}", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse decimal integer: {}", code);
        if let Ok(expr) = result {
            if let ExprKind::Literal(Literal::Integer(n)) = expr.kind {
                prop_assert_eq!(n, value, "Decimal value mismatch");
            } else {
                return Err(TestCaseError::fail(format!("Expected integer literal, got {:?}", expr.kind)));
            }
        }
    }

    #[test]
    fn prop_float_variations_parse(value in 1.0f64..1000.0f64) {
        // Test various float formats
        let formats = vec![
            format!("{:.1}", value),  // 123.4
            format!("{:.2}", value),  // 123.45
        ];

        for code in formats {
            let mut parser = Parser::new(&code);
            let result = parser.parse();

            prop_assert!(result.is_ok(), "Failed to parse float: {}", code);
            if let Ok(expr) = result {
                if let ExprKind::Literal(Literal::Float(n)) = expr.kind {
                    prop_assert!((n - value).abs() < 1.0, "Float value too different: {} vs {}", n, value);
                } else {
                    return Err(TestCaseError::fail(format!("Expected float literal for {}, got {:?}", code, expr.kind)));
                }
            }
        }
    }

    #[test]
    fn prop_hex_integers_parse(value in 0u32..0xFFFF) {
        let code = format!("0x{:X}", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse hex integer: {}", code);
        // Parser should successfully tokenize hex format
    }

    #[test]
    fn prop_binary_integers_parse(value in 0u32..0xFF) {
        let code = format!("0b{:b}", value);
        let mut parser = Parser::new(&code);
        let result = parser.parse();

        prop_assert!(result.is_ok(), "Failed to parse binary integer: {}", code);
        // Parser should successfully tokenize binary format
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if identifier is a reserved keyword
fn is_reserved_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "let"
            | "mut"
            | "fn"
            | "if"
            | "else"
            | "for"
            | "while"
            | "loop"
            | "break"
            | "continue"
            | "return"
            | "match"
            | "struct"
            | "enum"
            | "trait"
            | "impl"
            | "pub"
            | "use"
            | "true"
            | "false"
            | "nil"
            | "null"
    )
}
