//! String operation parsing
//!
//! Handles parsing of string interpolation and f-string expressions.
//! F-strings allow embedding Ruchy expressions within string literals using `{}` syntax.
//!
//! # Syntax
//! ```ruchy
//! f"Hello {name}"                    // Simple interpolation
//! f"Value: {x + 1}"                  // Expression interpolation
//! f"Formatted: {value:precision}"    // With format specifier
//! f"Escaped: {{"                     // Escaped braces
//! ```
//!
//! # Features
//! - Expression interpolation: `{expr}`
//! - Format specifiers: `{expr:spec}`
//! - Escaped braces: `{{` and `}}`
//! - Nested expressions with proper brace matching
//!
//! Extracted from expressions.rs to improve maintainability (TDG Structural improvement).

use crate::frontend::ast::StringPart;
use crate::frontend::parser::Result;
use anyhow::bail;

/// Parse f-string into interpolation parts
///
/// Converts an f-string template into a sequence of text and expression parts.
/// Handles brace escaping and nested expressions.
///
/// # Examples
/// ```ruchy
/// f"Hello {name}"           // Text("Hello "), Expr(name)
/// f"Value {{literal}}"      // Text("Value {literal}")
/// f"Sum: {a + b}"           // Text("Sum: "), Expr(a + b)
/// ```
pub(in crate::frontend::parser) fn parse_fstring_into_parts(
    input: &str,
) -> Result<Vec<StringPart>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => handle_opening_brace(&mut chars, &mut parts, &mut current)?,
            '}' => handle_closing_brace(&mut chars, &mut current)?,
            _ => current.push(ch),
        }
    }

    if !current.is_empty() {
        parts.push(StringPart::Text(current));
    }

    Ok(parts)
}

/// Handle opening brace in f-string
///
/// Distinguishes between:
/// - `{{` - Escaped literal brace
/// - `{expr}` - Expression interpolation
fn handle_opening_brace(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    parts: &mut Vec<StringPart>,
    current: &mut String,
) -> Result<()> {
    if chars.peek() == Some(&'{') {
        // Escaped brace: {{ → {
        chars.next();
        current.push('{');
    } else {
        // Start interpolation
        flush_text_part(parts, current);
        let expr_str = extract_fstring_expr(chars)?;
        parts.push(parse_interpolation(&expr_str)?);
    }
    Ok(())
}

/// Handle closing brace in f-string
///
/// Distinguishes between:
/// - `}}` - Escaped literal brace
/// - `}` - Unmatched closing brace (error)
fn handle_closing_brace(
    chars: &mut std::iter::Peekable<std::str::Chars>,
    current: &mut String,
) -> Result<()> {
    if chars.peek() == Some(&'}') {
        // Escaped brace: }} → }
        chars.next();
        current.push('}');
        Ok(())
    } else {
        bail!("Unmatched '}}' in f-string")
    }
}

/// Flush accumulated text as a `StringPart`
fn flush_text_part(parts: &mut Vec<StringPart>, current: &mut String) {
    if !current.is_empty() {
        parts.push(StringPart::Text(current.clone()));
        current.clear();
    }
}

/// Parse interpolation expression with optional format specifier
///
/// Supports:
/// - `{expr}` - Simple expression
/// - `{expr:format}` - Expression with format specifier
/// - `{}` - Empty placeholder (positional argument)
///
/// Note: Don't confuse `::` (path separator) with `:` (format specifier)
fn parse_interpolation(expr_str: &str) -> Result<StringPart> {
    use crate::frontend::parser::Parser;

    // DEFECT-PARSER-012 FIX: Handle empty placeholders {} for positional arguments
    if expr_str.is_empty() {
        // Empty {} is a positional placeholder - create a placeholder expression
        return Ok(StringPart::Text("{}".to_string()));
    }

    // Find single colon (format specifier) while ignoring :: (path separator)
    let colon_pos = find_format_specifier_colon(expr_str);

    if let Some(pos) = colon_pos {
        // Expression with format specifier: {expr:spec}
        let expr_part = &expr_str[..pos];
        let format_spec = &expr_str[pos..];
        let mut parser = Parser::new(expr_part);
        let expr = parser.parse_expr()?;
        Ok(StringPart::ExprWithFormat {
            expr: Box::new(expr),
            format_spec: format_spec.to_string(),
        })
    } else {
        // Simple expression: {expr}
        let mut parser = Parser::new(expr_str);
        let expr = parser.parse_expr()?;
        Ok(StringPart::Expr(Box::new(expr)))
    }
}

/// Find the position of a format specifier colon, ignoring :: path separators
fn find_format_specifier_colon(expr_str: &str) -> Option<usize> {
    let mut chars = expr_str.char_indices().peekable();

    while let Some((i, ch)) = chars.next() {
        if ch == ':' {
            // Check if next char is also ':' (path separator ::)
            if let Some((_, next_ch)) = chars.peek() {
                if *next_ch == ':' {
                    // This is ::, skip both colons
                    chars.next();
                    continue;
                }
            }
            // Single colon found - this is a format specifier
            // But only if the part before it doesn't contain quotes
            let before_colon = &expr_str[..i];
            if !before_colon.contains('"') && !before_colon.contains('\'') {
                return Some(i);
            }
        }
    }
    None
}

/// Extract expression from f-string interpolation
///
/// Handles nested braces by tracking depth.
/// Extracts text between `{` and matching `}`.
fn extract_fstring_expr(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
    let mut expr = String::new();
    let mut depth = 1;
    for ch in chars.by_ref() {
        if ch == '{' {
            depth += 1;
            expr.push(ch);
        } else if ch == '}' {
            depth -= 1;
            if depth == 0 {
                return Ok(expr);
            }
            expr.push(ch);
        } else {
            expr.push(ch);
        }
    }
    bail!("Unclosed interpolation in f-string")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_text() {
        let result = parse_fstring_into_parts("Hello World");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 1);
        assert!(matches!(parts[0], StringPart::Text(_)));
    }

    #[test]
    fn test_simple_interpolation() {
        let result = parse_fstring_into_parts("Hello {name}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 2);
        assert!(matches!(parts[0], StringPart::Text(_)));
        assert!(matches!(parts[1], StringPart::Expr(_)));
    }

    #[test]
    fn test_escaped_braces() {
        let result = parse_fstring_into_parts("Value {{literal}}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 1);
        if let StringPart::Text(text) = &parts[0] {
            assert_eq!(text, "Value {literal}");
        } else {
            panic!("Expected Text part");
        }
    }

    #[test]
    fn test_format_specifier() {
        let result = parse_fstring_into_parts("{value:.2f}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 1);
        assert!(matches!(parts[0], StringPart::ExprWithFormat { .. }));
    }

    #[test]
    fn test_empty_placeholder() {
        let result = parse_fstring_into_parts("Value: {}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 2);
        assert!(matches!(parts[0], StringPart::Text(_)));
        assert!(matches!(parts[1], StringPart::Text(_)));
    }

    #[test]
    fn test_multiple_interpolations() {
        let result = parse_fstring_into_parts("{a} + {b} = {c}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 5); // expr, text, expr, text, expr
    }

    #[test]
    fn test_unmatched_closing_brace() {
        let result = parse_fstring_into_parts("Value }");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_interpolation() {
        let result = parse_fstring_into_parts("Value {name");
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_braces() {
        let result = parse_fstring_into_parts("Result: {obj.method()}");
        assert!(result.is_ok());
        let parts = result.expect("operation should succeed in test");
        assert_eq!(parts.len(), 2);
    }

    // ============================================================
    // Additional comprehensive tests for EXTREME TDD coverage
    // ============================================================

    // ============================================================
    // Text-only variations
    // ============================================================

    #[test]
    fn test_empty_string() {
        let result = parse_fstring_into_parts("");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 0);
    }

    #[test]
    fn test_single_char() {
        let result = parse_fstring_into_parts("a");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_long_text() {
        let result = parse_fstring_into_parts("This is a very long string with many words");
        assert!(result.is_ok());
    }

    #[test]
    fn test_text_with_spaces() {
        let result = parse_fstring_into_parts("   spaced   ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_text_with_numbers() {
        let result = parse_fstring_into_parts("Value 123 and 456");
        assert!(result.is_ok());
    }

    #[test]
    fn test_text_with_punctuation() {
        let result = parse_fstring_into_parts("Hello, world! How are you?");
        assert!(result.is_ok());
    }

    // ============================================================
    // Single interpolation variations
    // ============================================================

    #[test]
    fn test_interpolation_at_start() {
        let result = parse_fstring_into_parts("{name} is here");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert!(matches!(parts[0], StringPart::Expr(_)));
    }

    #[test]
    fn test_interpolation_at_end() {
        let result = parse_fstring_into_parts("Hello {name}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_only() {
        let result = parse_fstring_into_parts("{name}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 1);
    }

    #[test]
    fn test_interpolation_single_letter() {
        let result = parse_fstring_into_parts("{x}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_long_name() {
        let result = parse_fstring_into_parts("{very_long_variable_name}");
        assert!(result.is_ok());
    }

    // ============================================================
    // Multiple interpolations
    // ============================================================

    #[test]
    fn test_two_interpolations() {
        let result = parse_fstring_into_parts("{a} and {b}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 3); // expr, text, expr
    }

    #[test]
    fn test_three_interpolations() {
        let result = parse_fstring_into_parts("{a}{b}{c}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 3); // expr, expr, expr
    }

    #[test]
    fn test_adjacent_interpolations() {
        let result = parse_fstring_into_parts("{x}{y}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spaced_interpolations() {
        let result = parse_fstring_into_parts("{a} {b} {c}");
        assert!(result.is_ok());
    }

    // ============================================================
    // Expression interpolations
    // ============================================================

    #[test]
    fn test_interpolation_addition() {
        let result = parse_fstring_into_parts("{a + b}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_multiplication() {
        let result = parse_fstring_into_parts("{x * 2}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_comparison() {
        let result = parse_fstring_into_parts("{x > 0}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_function_call() {
        let result = parse_fstring_into_parts("{foo()}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_method_call() {
        let result = parse_fstring_into_parts("{obj.method()}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_interpolation_field_access() {
        let result = parse_fstring_into_parts("{obj.field}");
        assert!(result.is_ok());
    }

    // ============================================================
    // Format specifiers
    // ============================================================

    #[test]
    fn test_format_decimal() {
        let result = parse_fstring_into_parts("{x:.2f}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert!(matches!(parts[0], StringPart::ExprWithFormat { .. }));
    }

    #[test]
    fn test_format_width() {
        let result = parse_fstring_into_parts("{x:10}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_zero_pad() {
        let result = parse_fstring_into_parts("{x:05}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_hex() {
        let result = parse_fstring_into_parts("{x:x}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_binary() {
        let result = parse_fstring_into_parts("{x:b}");
        assert!(result.is_ok());
    }

    // ============================================================
    // Escaped braces
    // ============================================================

    #[test]
    fn test_escaped_open_brace() {
        let result = parse_fstring_into_parts("{{");
        assert!(result.is_ok());
        let parts = result.unwrap();
        if let StringPart::Text(t) = &parts[0] {
            assert_eq!(t, "{");
        }
    }

    #[test]
    fn test_escaped_close_brace() {
        let result = parse_fstring_into_parts("}}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        if let StringPart::Text(t) = &parts[0] {
            assert_eq!(t, "}");
        }
    }

    #[test]
    fn test_escaped_both() {
        let result = parse_fstring_into_parts("{{}}");
        assert!(result.is_ok());
        let parts = result.unwrap();
        if let StringPart::Text(t) = &parts[0] {
            assert_eq!(t, "{}");
        }
    }

    #[test]
    fn test_escaped_with_text() {
        let result = parse_fstring_into_parts("a{{b}}c");
        assert!(result.is_ok());
    }

    #[test]
    fn test_escaped_multiple() {
        let result = parse_fstring_into_parts("{{{{}}}}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_escaped_mixed_with_interpolation() {
        let result = parse_fstring_into_parts("{{x}} = {x}");
        assert!(result.is_ok());
    }

    // ============================================================
    // Error cases
    // ============================================================

    #[test]
    fn test_unmatched_open() {
        let result = parse_fstring_into_parts("{x");
        assert!(result.is_err());
    }

    #[test]
    fn test_unmatched_close() {
        let result = parse_fstring_into_parts("x}");
        assert!(result.is_err());
    }

    #[test]
    fn test_unmatched_close_middle() {
        let result = parse_fstring_into_parts("a } b");
        assert!(result.is_err());
    }

    // Property tests for string operations
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            #[ignore = "Property tests run with --ignored flag"] // Run with: cargo test property_tests -- --ignored
            fn prop_plain_text_always_parses(text in "[a-zA-Z0-9 ]{0,100}") {
                let result = parse_fstring_into_parts(&text);
                prop_assert!(result.is_ok(), "Plain text should always parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_escaped_braces_parse(n in 0..10usize) {
                let text = "{{".repeat(n);
                let result = parse_fstring_into_parts(&text);
                prop_assert!(result.is_ok(), "Escaped braces should parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_simple_variable_interpolation(var in "[a-z][a-z0-9]{0,10}") {
                let text = format!("Value: {{{var}}}");
                let result = parse_fstring_into_parts(&text);
                prop_assert!(result.is_ok(), "Variable interpolation {} should parse", text);
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_multiple_interpolations(n in 1..5usize) {
                let text = (0..n).map(|i| format!("{{x{i}}}")).collect::<Vec<_>>().join(" ");
                let result = parse_fstring_into_parts(&text);
                prop_assert!(result.is_ok(), "Multiple interpolations should parse");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_unmatched_closing_brace_fails(text in "[a-zA-Z0-9]{0,10}") {
                let bad_text = format!("{text}}}");
                let result = parse_fstring_into_parts(&bad_text);
                prop_assert!(result.is_err(), "Unmatched }} should fail");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_empty_string_parses(_n in 0..100) {
                let result = parse_fstring_into_parts("");
                prop_assert!(result.is_ok(), "Empty string should parse");
                let parts = result.expect("operation should succeed in test");
                prop_assert_eq!(parts.len(), 0, "Empty string should have no parts");
            }

            #[test]
            #[ignore = "Property tests run with --ignored flag"]
            fn prop_escaped_sequences_have_half_braces(n in 1..10usize) {
                let input = "{{".repeat(n);
                let result = parse_fstring_into_parts(&input);
                prop_assert!(result.is_ok());
                let parts = result.expect("operation should succeed in test");
                if let Some(StringPart::Text(text)) = parts.first() {
                    prop_assert_eq!(text.len(), n, "Escaped {{{{ should produce half as many braces");
                }
            }
        }
    }
}
