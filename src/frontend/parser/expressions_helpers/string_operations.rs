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
fn parse_interpolation(expr_str: &str) -> Result<StringPart> {
    use crate::frontend::parser::Parser;

    // DEFECT-PARSER-012 FIX: Handle empty placeholders {} for positional arguments
    if expr_str.is_empty() {
        // Empty {} is a positional placeholder - create a placeholder expression
        return Ok(StringPart::Text("{}".to_string()));
    }

    if let Some(colon_pos) = expr_str.find(':') {
        // Expression with format specifier: {expr:spec}
        let expr_part = &expr_str[..colon_pos];
        let format_spec = &expr_str[colon_pos..];
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
