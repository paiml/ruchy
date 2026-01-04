//! String interpolation parsing functions
//!
//! This module contains all string interpolation logic extracted from utils.rs
//! to reduce file complexity and improve maintainability.

use super::super::StringPart;

/// Parse string interpolation from a string containing {expr} patterns
///
/// # Examples
///
/// ```
/// use ruchy::frontend::parser::utils::parse_string_interpolation;
/// use ruchy::frontend::parser::ParserState;
///
/// let parts = parse_string_interpolation(&mut ParserState::new(""), "Hello {name}");
/// assert_eq!(parts.len(), 2); // "Hello " and {name}
/// ```
pub fn parse_string_interpolation(
    _state: &mut super::super::ParserState,
    s: &str,
) -> Vec<StringPart> {
    let mut parts = Vec::new();
    let mut chars = s.chars().peekable();
    let mut current_text = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '{' if chars.peek() == Some(&'{') => {
                handle_escaped_brace(&mut chars, &mut current_text, '{');
            }
            '}' if chars.peek() == Some(&'}') => {
                handle_escaped_brace(&mut chars, &mut current_text, '}');
            }
            '{' => {
                handle_interpolation(&mut chars, &mut parts, &mut current_text);
            }
            _ => current_text.push(ch),
        }
    }
    finalize_text_part(&mut parts, current_text);
    parts
}

/// Handle escaped braces ({{ or }})
fn handle_escaped_brace<T: Iterator<Item = char>>(
    chars: &mut T,
    current_text: &mut String,
    brace_char: char,
) {
    chars.next(); // consume second brace
    current_text.push(brace_char);
}

/// Handle interpolation expressions
fn handle_interpolation<T: Iterator<Item = char>>(
    chars: &mut T,
    parts: &mut Vec<StringPart>,
    current_text: &mut String,
) {
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text.clone()));
        current_text.clear();
    }
    let expr_text = extract_expression_text(chars);
    let string_part = parse_interpolated_expr(&expr_text);
    parts.push(string_part);
}

/// Extract expression text from braces
fn extract_expression_text<T: Iterator<Item = char>>(chars: &mut T) -> String {
    let mut expr_text = String::new();
    let mut context = ExprContext::default();
    for expr_ch in chars {
        if process_character(expr_ch, &mut context, &mut expr_text) {
            break;
        }
    }
    expr_text
}

/// Process a single character in expression extraction
fn process_character(ch: char, context: &mut ExprContext, expr_text: &mut String) -> bool {
    let should_terminate = match ch {
        '"' if should_process_string_quote(context) => process_string_quote(context, expr_text, ch),
        '\'' if should_process_char_quote(context) => process_char_quote(context, expr_text, ch),
        '{' if should_process_brace(context) => process_open_brace(context, expr_text, ch),
        '}' if should_process_brace(context) => process_close_brace(context, expr_text, ch),
        '\\' if should_escape(context) => process_escape(context, expr_text, ch),
        _ => process_regular_character(context, expr_text, ch),
    };

    // Reset escape flag for non-backslash characters
    reset_escape_flag(context, ch);
    should_terminate
}

/// Process string quote character
fn process_string_quote(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_string_delimiter(context);
    expr_text.push(ch);
    false
}

/// Process char quote character
fn process_char_quote(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_char_delimiter(context);
    expr_text.push(ch);
    false
}

/// Process open brace character
fn process_open_brace(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_open_brace(context);
    expr_text.push(ch);
    false
}

/// Process close brace character (extracted to reduce nesting)
fn process_close_brace(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_close_brace(context);
    if should_terminate(context) {
        return true; // Signal to break the loop
    }
    expr_text.push(ch);
    false
}

/// Process escape character
fn process_escape(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_escape(context);
    expr_text.push(ch);
    false
}

/// Process regular character
fn process_regular_character(context: &mut ExprContext, expr_text: &mut String, ch: char) -> bool {
    handle_regular_char(context, ch);
    expr_text.push(ch);
    false
}

/// Check if string quote should be processed
fn should_process_string_quote(context: &ExprContext) -> bool {
    !context.in_char && !context.escaped
}

/// Check if char quote should be processed
fn should_process_char_quote(context: &ExprContext) -> bool {
    !context.in_string && !context.escaped
}

/// Check if brace should be processed
fn should_process_brace(context: &ExprContext) -> bool {
    !context.in_string && !context.in_char
}

/// Check if escape should be handled
fn should_escape(context: &ExprContext) -> bool {
    (context.in_string || context.in_char) && !context.escaped
}

/// Toggle string delimiter state
fn handle_string_delimiter(context: &mut ExprContext) {
    context.in_string = !context.in_string;
}

/// Toggle char delimiter state
fn handle_char_delimiter(context: &mut ExprContext) {
    context.in_char = !context.in_char;
}

/// Increment brace count
fn handle_open_brace(context: &mut ExprContext) {
    context.brace_count += 1;
}

/// Decrement brace count
fn handle_close_brace(context: &mut ExprContext) {
    context.brace_count -= 1;
}

/// Set escape flag
fn handle_escape(context: &mut ExprContext) {
    context.escaped = true;
}

/// Handle regular character
fn handle_regular_char(context: &mut ExprContext, _ch: char) {
    context.escaped = false;
}

/// Reset escape flag if needed
fn reset_escape_flag(context: &mut ExprContext, ch: char) {
    if ch != '\\' {
        context.escaped = false;
    }
}

/// Check if we should terminate extraction
fn should_terminate(context: &ExprContext) -> bool {
    context.brace_count == 0
}

/// Parse interpolated expression with format specifier
fn parse_interpolated_expr(expr_text: &str) -> StringPart {
    let (expr_part, format_spec) = split_format_specifier(expr_text);
    let mut expr_parser = crate::frontend::parser::Parser::new(expr_part);
    match expr_parser.parse() {
        Ok(expr) => {
            if let Some(spec) = format_spec {
                StringPart::ExprWithFormat {
                    expr: Box::new(expr),
                    format_spec: spec.to_string(),
                }
            } else {
                StringPart::Expr(Box::new(expr))
            }
        }
        Err(_) => {
            // Fallback to text if parsing fails
            StringPart::Text(format!("{{{expr_text}}}"))
        }
    }
}

/// Split format specifier from expression
fn split_format_specifier(expr_text: &str) -> (&str, Option<&str>) {
    if let Some(colon_pos) = expr_text.find(':') {
        let before_colon = &expr_text[..colon_pos];
        if !before_colon.contains('"') && !before_colon.contains('\'') {
            (before_colon, Some(&expr_text[colon_pos..]))
        } else {
            (expr_text, None)
        }
    } else {
        (expr_text, None)
    }
}

/// Finalize remaining text
fn finalize_text_part(parts: &mut Vec<StringPart>, current_text: String) {
    if !current_text.is_empty() {
        parts.push(StringPart::Text(current_text));
    }
}

/// Helper struct to track expression parsing context
#[derive(Default)]
struct ExprContext {
    brace_count: i32,
    in_string: bool,
    in_char: bool,
    escaped: bool,
}

impl ExprContext {
    fn default() -> Self {
        Self {
            brace_count: 1,
            in_string: false,
            in_char: false,
            escaped: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_state() -> super::super::super::ParserState<'static> {
        super::super::super::ParserState::new("")
    }

    #[test]
    fn test_parse_simple_text() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Hello World");
        assert_eq!(parts.len(), 1);
        match &parts[0] {
            StringPart::Text(s) => assert_eq!(s, "Hello World"),
            _ => panic!("Expected Text part"),
        }
    }

    #[test]
    fn test_parse_single_interpolation() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Hello {name}!");
        assert_eq!(parts.len(), 3);
        assert!(matches!(&parts[0], StringPart::Text(s) if s == "Hello "));
        assert!(matches!(&parts[1], StringPart::Expr(_)));
        assert!(matches!(&parts[2], StringPart::Text(s) if s == "!"));
    }

    #[test]
    fn test_parse_escaped_braces() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Value: {{42}}");
        assert_eq!(parts.len(), 1);
        match &parts[0] {
            StringPart::Text(s) => assert_eq!(s, "Value: {42}"),
            _ => panic!("Expected Text part with escaped braces"),
        }
    }

    #[test]
    fn test_parse_multiple_interpolations() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "{a} + {b} = {c}");
        assert_eq!(parts.len(), 5);
        assert!(matches!(&parts[0], StringPart::Expr(_)));
        assert!(matches!(&parts[1], StringPart::Text(s) if s == " + "));
        assert!(matches!(&parts[2], StringPart::Expr(_)));
        assert!(matches!(&parts[3], StringPart::Text(s) if s == " = "));
        assert!(matches!(&parts[4], StringPart::Expr(_)));
    }

    #[test]
    fn test_parse_format_specifier() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Value: {x:02}");
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[1], StringPart::ExprWithFormat { format_spec, .. } if format_spec == ":02"));
    }

    #[test]
    fn test_parse_empty_string() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "");
        assert_eq!(parts.len(), 0);
    }

    #[test]
    fn test_parse_only_interpolation() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "{x}");
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], StringPart::Expr(_)));
    }

    #[test]
    fn test_parse_nested_braces_in_expr() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Result: {obj.get()}");
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[0], StringPart::Text(s) if s == "Result: "));
        assert!(matches!(&parts[1], StringPart::Expr(_)));
    }

    #[test]
    fn test_parse_string_in_interpolation() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Say: {\"hello\"}");
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[0], StringPart::Text(s) if s == "Say: "));
        assert!(matches!(&parts[1], StringPart::Expr(_)));
    }

    #[test]
    fn test_handle_escaped_brace_function() {
        let mut text = String::from("test");
        let mut chars = "abc".chars().peekable();
        handle_escaped_brace(&mut chars, &mut text, '{');
        assert_eq!(text, "test{");
        // First char consumed
        assert_eq!(chars.next(), Some('b'));
    }

    #[test]
    fn test_expr_context_default() {
        let ctx = ExprContext::default();
        assert_eq!(ctx.brace_count, 1);
        assert!(!ctx.in_string);
        assert!(!ctx.in_char);
        assert!(!ctx.escaped);
    }

    #[test]
    fn test_should_process_string_quote() {
        let ctx = ExprContext {
            in_char: false,
            escaped: false,
            ..ExprContext::default()
        };
        assert!(should_process_string_quote(&ctx));

        let ctx2 = ExprContext {
            in_char: true,
            ..ExprContext::default()
        };
        assert!(!should_process_string_quote(&ctx2));
    }

    #[test]
    fn test_should_process_char_quote() {
        let ctx = ExprContext {
            in_string: false,
            escaped: false,
            ..ExprContext::default()
        };
        assert!(should_process_char_quote(&ctx));

        let ctx2 = ExprContext {
            in_string: true,
            ..ExprContext::default()
        };
        assert!(!should_process_char_quote(&ctx2));
    }

    #[test]
    fn test_should_process_brace() {
        let ctx = ExprContext {
            in_string: false,
            in_char: false,
            ..ExprContext::default()
        };
        assert!(should_process_brace(&ctx));

        let ctx2 = ExprContext {
            in_string: true,
            ..ExprContext::default()
        };
        assert!(!should_process_brace(&ctx2));
    }

    #[test]
    fn test_should_escape() {
        let ctx = ExprContext {
            in_string: true,
            escaped: false,
            ..ExprContext::default()
        };
        assert!(should_escape(&ctx));

        let ctx2 = ExprContext {
            in_string: false,
            in_char: false,
            ..ExprContext::default()
        };
        assert!(!should_escape(&ctx2));
    }

    #[test]
    fn test_handle_string_delimiter() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.in_string);
        handle_string_delimiter(&mut ctx);
        assert!(ctx.in_string);
        handle_string_delimiter(&mut ctx);
        assert!(!ctx.in_string);
    }

    #[test]
    fn test_handle_char_delimiter() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.in_char);
        handle_char_delimiter(&mut ctx);
        assert!(ctx.in_char);
        handle_char_delimiter(&mut ctx);
        assert!(!ctx.in_char);
    }

    #[test]
    fn test_handle_open_close_brace() {
        let mut ctx = ExprContext::default();
        assert_eq!(ctx.brace_count, 1);
        handle_open_brace(&mut ctx);
        assert_eq!(ctx.brace_count, 2);
        handle_close_brace(&mut ctx);
        assert_eq!(ctx.brace_count, 1);
    }

    #[test]
    fn test_handle_escape() {
        let mut ctx = ExprContext::default();
        assert!(!ctx.escaped);
        handle_escape(&mut ctx);
        assert!(ctx.escaped);
    }

    #[test]
    fn test_handle_regular_char() {
        let mut ctx = ExprContext {
            escaped: true,
            ..ExprContext::default()
        };
        handle_regular_char(&mut ctx, 'x');
        assert!(!ctx.escaped);
    }

    #[test]
    fn test_reset_escape_flag() {
        let mut ctx = ExprContext {
            escaped: true,
            ..ExprContext::default()
        };
        reset_escape_flag(&mut ctx, 'a');
        assert!(!ctx.escaped);

        ctx.escaped = true;
        reset_escape_flag(&mut ctx, '\\');
        assert!(ctx.escaped);
    }

    #[test]
    fn test_should_terminate() {
        let ctx = ExprContext {
            brace_count: 0,
            ..ExprContext::default()
        };
        assert!(should_terminate(&ctx));

        let ctx2 = ExprContext::default();
        assert!(!should_terminate(&ctx2));
    }

    #[test]
    fn test_split_format_specifier_with_spec() {
        let (expr, spec) = split_format_specifier("x:02d");
        assert_eq!(expr, "x");
        assert_eq!(spec, Some(":02d"));
    }

    #[test]
    fn test_split_format_specifier_no_spec() {
        let (expr, spec) = split_format_specifier("variable");
        assert_eq!(expr, "variable");
        assert_eq!(spec, None);
    }

    #[test]
    fn test_split_format_specifier_colon_in_string() {
        let (expr, spec) = split_format_specifier("\"key:value\"");
        assert_eq!(expr, "\"key:value\"");
        assert_eq!(spec, None);
    }

    #[test]
    fn test_finalize_text_part_non_empty() {
        let mut parts = vec![];
        finalize_text_part(&mut parts, "hello".to_string());
        assert_eq!(parts.len(), 1);
        assert!(matches!(&parts[0], StringPart::Text(s) if s == "hello"));
    }

    #[test]
    fn test_finalize_text_part_empty() {
        let mut parts = vec![];
        finalize_text_part(&mut parts, String::new());
        assert_eq!(parts.len(), 0);
    }

    #[test]
    fn test_extract_expression_text() {
        let mut chars = "abc}rest".chars();
        let result = extract_expression_text(&mut chars);
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_process_character_regular() {
        let mut ctx = ExprContext::default();
        let mut text = String::new();
        let terminated = process_character('x', &mut ctx, &mut text);
        assert!(!terminated);
        assert_eq!(text, "x");
    }

    #[test]
    fn test_process_character_close_brace_terminate() {
        let mut ctx = ExprContext {
            brace_count: 1,
            ..ExprContext::default()
        };
        let mut text = String::new();
        let terminated = process_character('}', &mut ctx, &mut text);
        assert!(terminated);
    }

    #[test]
    fn test_parse_char_in_interpolation() {
        let mut state = create_state();
        let parts = parse_string_interpolation(&mut state, "Char: {'a'}");
        assert_eq!(parts.len(), 2);
        assert!(matches!(&parts[0], StringPart::Text(s) if s == "Char: "));
    }

    #[test]
    fn test_parse_invalid_expr_fallback() {
        // Test that invalid expressions fallback to text
        let result = parse_interpolated_expr("@#$%^&invalid");
        assert!(matches!(result, StringPart::Text(_)));
    }
}
