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
