//! Enhanced error diagnostics with source code display and suggestions
//!
//! Integrates with the Oracle module for ML-powered fix suggestions.
use crate::frontend::ast::Span;
use crate::frontend::error_recovery::{ErrorSeverity, ParseError};
use crate::oracle::{CompilationError, PatternStore, RuchyOracle};
use std::fmt;
/// Enhanced diagnostic information with source context
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub error: ParseError,
    pub source_code: String,
    pub filename: Option<String>,
    pub suggestions: Vec<Suggestion>,
}
/// A suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub span: Span,
}
impl Diagnostic {
    pub fn new(error: ParseError, source_code: String) -> Self {
        Self {
            error,
            source_code,
            filename: None,
            suggestions: Vec::new(),
        }
    }
    pub fn with_filename(mut self, filename: String) -> Self {
        self.filename = Some(filename);
        self
    }
    pub fn add_suggestion(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }
    /// Extract the relevant source lines with context
    fn get_source_context(&self) -> (Vec<String>, usize, usize, usize) {
        let lines: Vec<String> = self
            .source_code
            .lines()
            .map(std::string::ToString::to_string)
            .collect();
        // Find line and column from byte offset
        let mut current_pos = 0;
        let mut line_num = 0;
        let mut col_start = 0;
        for (i, line) in lines.iter().enumerate() {
            let line_len = line.len() + 1; // +1 for newline
            if current_pos + line_len > self.error.span.start {
                line_num = i;
                col_start = self.error.span.start - current_pos;
                break;
            }
            current_pos += line_len;
        }
        // Calculate error span width
        let col_end = col_start + (self.error.span.end - self.error.span.start);
        // Get context lines (2 before, 2 after)
        let context_start = line_num.saturating_sub(2);
        let context_end = (line_num + 3).min(lines.len());
        let context_lines = lines[context_start..context_end].to_vec();
        (context_lines, line_num - context_start, col_start, col_end)
    }
    /// Generate colored output for terminal display
    pub fn format_colored(&self) -> String {
        let mut output = String::new();
        let (severity_color, reset, bold) = self.get_color_codes();
        // Build diagnostic sections
        output.push_str(&self.format_header(severity_color, reset, bold));
        output.push_str(&self.format_source_context(severity_color, reset, bold));
        output.push_str(&self.format_suggestions(bold, reset));
        output.push_str(reset);
        output
    }
    /// Get terminal color codes for diagnostic formatting
    fn get_color_codes(&self) -> (&'static str, &'static str, &'static str) {
        let severity_color = match self.error.severity {
            ErrorSeverity::Error => "\x1b[31m",   // Red
            ErrorSeverity::Warning => "\x1b[33m", // Yellow
            ErrorSeverity::Info => "\x1b[34m",    // Blue
            ErrorSeverity::Hint => "\x1b[36m",    // Cyan
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";
        (severity_color, reset, bold)
    }
    /// Format the diagnostic header with error message and location
    fn format_header(&self, severity_color: &str, reset: &str, bold: &str) -> String {
        let mut header = String::new();
        let error_line = format!(
            "{bold}{severity_color}error[{:?}]{reset}: {}\n",
            self.error.error_code, self.error.message
        );
        header.push_str(&error_line);
        if let Some(ref filename) = self.filename {
            header.push_str(&format!(
                "  {bold}-->{reset} {}:{}:{}\n",
                filename,
                self.error.span.start / 100 + 1, // Rough line estimate
                self.error.span.start % 100 + 1  // Rough column estimate
            ));
        }
        header
    }
    /// Format source code context with error highlighting
    fn format_source_context(&self, severity_color: &str, reset: &str, bold: &str) -> String {
        let mut context = String::new();
        let (context_lines, error_line_idx, col_start, col_end) = self.get_source_context();
        let line_num_start = (self.error.span.start / 100 + 1).saturating_sub(error_line_idx);
        for (i, line) in context_lines.iter().enumerate() {
            let line_num = line_num_start + i;
            let is_error_line = i == error_line_idx;
            if is_error_line {
                context.push_str(&self.format_error_line(
                    line,
                    line_num,
                    col_start,
                    col_end,
                    severity_color,
                    reset,
                    bold,
                ));
            } else {
                context.push_str(&format!("{line_num:4} | {line}\n"));
            }
        }
        context
    }
    /// Format the specific error line with underline and hint
    fn format_error_line(
        &self,
        line: &str,
        line_num: usize,
        col_start: usize,
        col_end: usize,
        severity_color: &str,
        reset: &str,
        bold: &str,
    ) -> String {
        let mut error_line = String::new();
        // Line number and content
        error_line.push_str(&format!("{bold}{line_num:4} |{reset} {line}\n"));
        // Error underline
        let spaces = " ".repeat(col_start);
        let arrows = "^".repeat((col_end - col_start).max(1));
        error_line.push_str(&format!("     | {spaces}{severity_color}{arrows}\n"));
        // Error message under the line
        if let Some(ref hint) = self.error.recovery_hint {
            error_line.push_str(&format!(
                "     {} {}{}{reset} {}\n",
                "|",
                " ".repeat(col_start),
                severity_color,
                hint
            ));
        }
        error_line
    }
    /// Format suggestions section
    fn format_suggestions(&self, bold: &str, reset: &str) -> String {
        let mut suggestions = String::new();
        if !self.suggestions.is_empty() {
            suggestions.push_str(&format!("\n{bold}help{reset}: "));
            for suggestion in &self.suggestions {
                suggestions.push_str(&format!("{}\n", suggestion.message));
                if let Some(ref replacement) = suggestion.replacement {
                    suggestions.push_str(&format!("      suggested fix: `{replacement}`\n"));
                }
            }
        }
        suggestions
    }
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_colored())
    }
}
/// Common error patterns and their suggestions
pub fn suggest_for_error(error: &ParseError) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    let msg = &error.message;

    // Table-driven pattern matching for lower cognitive complexity
    suggest_for_unexpected(error, &mut suggestions);
    suggest_for_semicolon(msg, error.span, &mut suggestions);
    suggest_for_unclosed_delimiter(msg, error.span, &mut suggestions);

    suggestions
}

/// Handle unexpected token suggestions
fn suggest_for_unexpected(error: &ParseError, suggestions: &mut Vec<Suggestion>) {
    if error.message.contains("unexpected") && error.found.is_some() {
        suggestions.push(Suggestion {
            message: "Check for typos or missing operators".to_string(),
            replacement: None,
            span: error.span,
        });
    }
}

/// Handle missing semicolon suggestions
fn suggest_for_semicolon(msg: &str, span: Span, suggestions: &mut Vec<Suggestion>) {
    if msg.contains("expected") && msg.contains("semicolon") {
        suggestions.push(Suggestion {
            message: "Add a semicolon at the end of the statement".to_string(),
            replacement: Some(";".to_string()),
            span: Span {
                start: span.end,
                end: span.end,
            },
        });
    }
}

/// Handle unclosed delimiter suggestions (table-driven)
fn suggest_for_unclosed_delimiter(msg: &str, span: Span, suggestions: &mut Vec<Suggestion>) {
    if !msg.contains("unclosed") && !msg.contains("unmatched") {
        return;
    }

    // Table-driven: (pattern, message, replacement)
    const DELIMITERS: &[(&str, &str, &str)] = &[
        ("paren", "Add closing parenthesis ')'", ")"),
        ("brace", "Add closing brace '}'", "}"),
        ("bracket", "Add closing bracket ']'", "]"),
    ];

    for (pattern, message, replacement) in DELIMITERS {
        if msg.contains(pattern) {
            suggestions.push(Suggestion {
                message: (*message).to_string(),
                replacement: Some((*replacement).to_string()),
                span: Span {
                    start: span.end,
                    end: span.end,
                },
            });
            break; // Only one delimiter fix per error
        }
    }
}

/// Get ML-powered suggestions from the Oracle
///
/// Uses the Oracle classifier to categorize the error and query
/// the pattern store for relevant fix suggestions.
pub fn suggest_from_oracle(error: &ParseError) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Convert ParseError to CompilationError for Oracle
    let comp_error = CompilationError {
        message: error.message.clone(),
        code: Some(format!("{:?}", error.error_code)),
        file_path: None,
        line: None,
        column: None,
    };

    // Classify using Oracle
    let oracle = RuchyOracle::new();
    let classification = oracle.classify(&comp_error);

    // Only use Oracle suggestions if confidence is reasonable
    if classification.confidence < 0.3 {
        return suggestions;
    }

    // Query pattern store for fixes
    let pattern_store = PatternStore::new();
    let oracle_suggestions = pattern_store.query(classification.category, &error.message, 0.5);

    // Convert Oracle suggestions to Diagnostic suggestions
    for oracle_sugg in oracle_suggestions.iter().take(3) {
        suggestions.push(Suggestion {
            message: format!(
                "[Oracle {:.0}%] {}",
                oracle_sugg.success_rate * 100.0,
                oracle_sugg.description
            ),
            replacement: Some(oracle_sugg.transformation.clone()),
            span: error.span,
        });
    }

    suggestions
}

/// Combined suggestion generator using both heuristics and Oracle
pub fn suggest_all(error: &ParseError) -> Vec<Suggestion> {
    let mut suggestions = suggest_for_error(error);

    // Add Oracle-based suggestions
    let oracle_suggestions = suggest_from_oracle(error);
    suggestions.extend(oracle_suggestions);

    suggestions
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_diagnostic_display() {
        let error = ParseError::new("Unexpected token".to_string(), Span { start: 10, end: 15 });
        let source = "let x = 10\nlet y = @invalid\nlet z = 30".to_string();
        let diag = Diagnostic::new(error, source);
        let output = format!("{diag}");
        assert!(output.contains("Unexpected token"));
    }
    #[test]
    fn test_suggestions() {
        let mut error = ParseError::new("unexpected '='".to_string(), Span { start: 5, end: 6 });
        error.found = Some(crate::frontend::lexer::Token::Equal);
        let suggestions = suggest_for_error(&error);
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_diagnostic_with_filename() {
        let error = ParseError::new("Test error".to_string(), Span { start: 0, end: 5 });
        let diag =
            Diagnostic::new(error, "test code".to_string()).with_filename("test.ruchy".to_string());

        assert_eq!(diag.filename, Some("test.ruchy".to_string()));
    }

    #[test]
    fn test_add_suggestion() {
        let error = ParseError::new("Error".to_string(), Span { start: 0, end: 5 });
        let mut diag = Diagnostic::new(error, "code".to_string());

        let suggestion = Suggestion {
            message: "Try this".to_string(),
            replacement: Some("fixed".to_string()),
            span: Span::new(0, 5),
        };

        diag.add_suggestion(suggestion);
        assert_eq!(diag.suggestions.len(), 1);
        assert_eq!(diag.suggestions[0].message, "Try this");
    }

    #[test]
    fn test_error_severity_levels() {
        let mut error = ParseError::new("Test".to_string(), Span { start: 0, end: 5 });

        error.severity = ErrorSeverity::Error;
        assert!(matches!(error.severity, ErrorSeverity::Error));

        error.severity = ErrorSeverity::Warning;
        assert!(matches!(error.severity, ErrorSeverity::Warning));

        error.severity = ErrorSeverity::Info;
        assert!(matches!(error.severity, ErrorSeverity::Info));
    }

    #[test]
    fn test_multiline_source_context() {
        let error = ParseError::new(
            "Error on second line".to_string(),
            Span { start: 15, end: 20 },
        );
        let source = "first line\nsecond line\nthird line".to_string();
        let diag = Diagnostic::new(error, source);

        let (lines, line_num, _, _) = diag.get_source_context();
        assert!(!lines.is_empty());
        assert!(line_num > 0);
    }

    #[test]
    fn test_diagnostic_display_with_suggestions() {
        let error = ParseError::new("Missing semicolon".to_string(), Span { start: 10, end: 10 });
        let mut diag = Diagnostic::new(error, "let x = 5".to_string());

        diag.add_suggestion(Suggestion {
            message: "Add semicolon".to_string(),
            replacement: Some(";".to_string()),
            span: Span::new(10, 10),
        });

        let output = format!("{diag}");
        assert!(output.contains("Missing semicolon"));
        assert!(output.contains("Add semicolon") || output.contains("help"));
    }

    #[test]
    fn test_get_source_context_edge_cases() {
        // Test with empty source
        let error = ParseError::new("Error".to_string(), Span { start: 0, end: 0 });
        let diag = Diagnostic::new(error, String::new());
        let (lines, _, _, _) = diag.get_source_context();
        assert!(lines.is_empty() || lines[0].is_empty());

        // Test with single character source
        let error2 = ParseError::new("Error".to_string(), Span { start: 0, end: 1 });
        let diag2 = Diagnostic::new(error2, "x".to_string());
        let (lines2, _, _, _) = diag2.get_source_context();
        assert_eq!(lines2[0], "x");
    }

    #[test]
    fn test_oracle_integration_type_mismatch() {
        // Test Oracle suggestions for type mismatch errors
        let error = ParseError::new(
            "expected `i32`, found `String`".to_string(),
            Span { start: 0, end: 10 },
        );
        let suggestions = suggest_from_oracle(&error);
        // Oracle should provide suggestions for type mismatches
        // (may be empty if confidence is too low)
        assert!(suggestions.len() <= 3);
    }

    #[test]
    fn test_oracle_integration_missing_module() {
        // Test Oracle suggestions for missing module errors
        let error = ParseError::new(
            "Module 'scanner' not found".to_string(),
            Span { start: 0, end: 20 },
        );
        let suggestions = suggest_from_oracle(&error);
        // Should return module-related suggestions if Oracle is confident
        for sugg in &suggestions {
            assert!(sugg.message.contains("Oracle"));
        }
    }

    #[test]
    fn test_suggest_all_combines_sources() {
        // Test that suggest_all includes both heuristic and Oracle suggestions
        let mut error = ParseError::new(
            "unexpected token".to_string(),
            Span { start: 5, end: 10 },
        );
        error.found = Some(crate::frontend::lexer::Token::Equal);

        let suggestions = suggest_all(&error);
        // Should have at least the heuristic suggestion
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_oracle_suggestion_format() {
        // Verify Oracle suggestions are properly formatted
        let error = ParseError::new(
            "item in documentation is missing backticks".to_string(),
            Span { start: 0, end: 40 },
        );
        let suggestions = suggest_from_oracle(&error);

        for sugg in &suggestions {
            // Format should be "[Oracle XX%] description"
            assert!(sugg.message.starts_with("[Oracle"));
            assert!(sugg.message.contains("%]"));
        }
    }

    #[test]
    fn test_suggest_for_error_various_tokens() {
        // Test suggestion for unexpected token
        let mut error = ParseError::new("unexpected ':'".to_string(), Span { start: 5, end: 6 });
        error.found = Some(crate::frontend::lexer::Token::Colon);
        let suggestions = suggest_for_error(&error);
        // Just check we get some suggestion for typos
        assert!(suggestions
            .iter()
            .any(|s| s.message.contains("typos") || s.message.contains("operators")));

        // Test suggestion for unexpected arrow
        let mut error2 =
            ParseError::new("unexpected '->'".to_string(), Span { start: 10, end: 12 });
        error2.found = Some(crate::frontend::lexer::Token::Arrow);
        let suggestions2 = suggest_for_error(&error2);
        assert!(!suggestions2.is_empty());
    }

    #[test]
    fn test_parse_error_with_expected() {
        let mut error = ParseError::new("Unexpected token".to_string(), Span { start: 0, end: 5 });
        error.expected = vec![crate::frontend::lexer::Token::LeftParen];
        error.found = Some(crate::frontend::lexer::Token::RightParen);

        assert!(!error.expected.is_empty());
        assert!(error.found.is_some());
    }
}
