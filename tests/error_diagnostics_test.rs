// Error Diagnostics Test Suite
// Testing enhanced error messages, source snippets, and suggestions

use ruchy::frontend::error_recovery::{ParseError, ErrorSeverity, ErrorCode};
use ruchy::frontend::diagnostics::{Diagnostic, suggest_for_error};
use ruchy::frontend::ast::Span;

#[test]
fn test_basic_diagnostic_display() {
    let error = ParseError::new(
        "Unexpected token '@@'".to_string(),
        Span { start: 15, end: 17 },
    );
    
    let source = "let x = 10\nlet y = @@ invalid\nlet z = 30".to_string();
    let diag = Diagnostic::new(error, source).with_filename("test.ruchy".to_string());
    
    let output = format!("{diag}");
    assert!(output.contains("Unexpected token"));
    assert!(output.contains("test.ruchy"));
}

#[test]
fn test_multiline_error_context() {
    let mut error = ParseError::new(
        "Unclosed parenthesis".to_string(),
        Span { start: 25, end: 26 },
    );
    error.recovery_hint = Some("Add closing parenthesis ')' here".to_string());
    
    let source = "fn add(a: i32, b: i32 -> i32 {\n    a + b\n}".to_string();
    let diag = Diagnostic::new(error, source);
    
    let output = diag.format_colored();
    assert!(output.contains("Unclosed parenthesis"));
    assert!(output.contains("Add closing parenthesis"));
}

#[test]
fn test_error_suggestions() {
    let mut error = ParseError::new(
        "unexpected '=' in expression".to_string(),
        Span { start: 8, end: 9 },
    );
    error.found = Some(ruchy::frontend::lexer::Token::Equal);
    
    let suggestions = suggest_for_error(&error);
    assert!(!suggestions.is_empty());
    assert!(suggestions[0].message.contains("Check for typos"));
}

#[test]
fn test_missing_semicolon_suggestion() {
    let error = ParseError::new(
        "expected semicolon".to_string(),
        Span { start: 10, end: 10 },
    );
    
    let suggestions = suggest_for_error(&error);
    let has_semicolon = suggestions.iter().any(|s| s.message.contains("semicolon"));
    assert!(has_semicolon);
}

#[test]
fn test_unclosed_brace_suggestion() {
    let error = ParseError::new(
        "unclosed brace".to_string(),
        Span { start: 50, end: 50 },
    );
    
    let suggestions = suggest_for_error(&error);
    let has_brace = suggestions.iter().any(|s| s.message.contains("brace"));
    assert!(has_brace);
}

#[test]
fn test_error_severity_colors() {
    // Test different severity levels
    let mut error = ParseError::new(
        "Warning message".to_string(),
        Span { start: 0, end: 5 },
    );
    
    let source = "test code".to_string();
    
    // Error severity (red)
    error.severity = ErrorSeverity::Error;
    let diag = Diagnostic::new(error.clone(), source.clone());
    let output = diag.format_colored();
    assert!(output.contains("\x1b[31m")); // Red color code
    
    // Warning severity (yellow)
    error.severity = ErrorSeverity::Warning;
    let diag = Diagnostic::new(error.clone(), source.clone());
    let output = diag.format_colored();
    assert!(output.contains("\x1b[33m")); // Yellow color code
    
    // Info severity (blue)
    error.severity = ErrorSeverity::Info;
    let diag = Diagnostic::new(error, source);
    let output = diag.format_colored();
    assert!(output.contains("\x1b[34m")); // Blue color code
}

#[test]
fn test_error_code_display() {
    let mut error = ParseError::new(
        "Type mismatch".to_string(),
        Span { start: 10, end: 15 },
    );
    error.error_code = ErrorCode::TypeMismatch;
    
    let source = "let x: i32 = \"string\"".to_string();
    let diag = Diagnostic::new(error, source);
    
    let output = format!("{diag}");
    assert!(output.contains("TypeMismatch"));
}

#[test]
fn test_multiline_source_context() {
    let error = ParseError::new(
        "Invalid syntax".to_string(),
        Span { start: 40, end: 45 }, // Somewhere in middle of source
    );
    
    let source = r"fn main() {
    let x = 10
    let y = 20
    let z = @invalid@
    let w = 30
    println(x + y + z + w)
}".to_string();
    
    let diag = Diagnostic::new(error, source);
    let output = diag.format_colored();
    
    // Should show context lines
    assert!(output.contains("let y = 20"));
    assert!(output.contains("let z = @invalid@"));
    assert!(output.contains("let w = 30"));
}

#[test]
fn test_error_underline_positioning() {
    let error = ParseError::new(
        "Unknown operator".to_string(),
        Span { start: 12, end: 14 },
    );
    
    let source = "let x = 10 @@ 20".to_string();
    let diag = Diagnostic::new(error, source);
    
    let output = diag.format_colored();
    // Should have carets under the error
    assert!(output.contains("^^"));
}

#[test]
fn test_multiple_suggestions() {
    let mut diag = Diagnostic::new(
        ParseError::new("Multiple issues".to_string(), Span { start: 0, end: 5 }),
        "test".to_string(),
    );
    
    diag.add_suggestion(ruchy::frontend::diagnostics::Suggestion {
        message: "First suggestion".to_string(),
        replacement: Some("fix1".to_string()),
        span: Span { start: 0, end: 5 },
    });
    
    diag.add_suggestion(ruchy::frontend::diagnostics::Suggestion {
        message: "Second suggestion".to_string(),
        replacement: Some("fix2".to_string()),
        span: Span { start: 0, end: 5 },
    });
    
    let output = format!("{diag}");
    assert!(output.contains("First suggestion"));
    assert!(output.contains("Second suggestion"));
    assert!(output.contains("fix1"));
    assert!(output.contains("fix2"));
}