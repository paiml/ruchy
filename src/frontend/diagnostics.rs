//! Enhanced error diagnostics with helpful messages
//!
//! This module provides user-friendly error messages with context,
//! suggestions, and code snippets similar to Rust and Elm compilers.

use crate::frontend::ast::Span;
use colored::Colorize;
use std::fmt;

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "{}", "error".red().bold()),
            Severity::Warning => write!(f, "{}", "warning".yellow().bold()),
            Severity::Info => write!(f, "{}", "info".blue().bold()),
            Severity::Hint => write!(f, "{}", "hint".green()),
        }
    }
}

/// A diagnostic message with context
pub struct Diagnostic {
    /// Severity of the diagnostic
    pub severity: Severity,
    /// Primary error message
    pub message: String,
    /// Source code that caused the error
    pub source: String,
    /// Span in the source where error occurred
    pub span: Span,
    /// Optional error code (e.g., E0001)
    pub code: Option<String>,
    /// Helpful suggestion to fix the error
    pub suggestion: Option<String>,
    /// Additional notes
    pub notes: Vec<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Error,
            message: message.into(),
            source: String::new(),
            span: Span { start: 0, end: 0 },
            code: None,
            suggestion: None,
            notes: Vec::new(),
        }
    }

    /// Create a new warning diagnostic
    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            severity: Severity::Warning,
            message: message.into(),
            source: String::new(),
            span: Span { start: 0, end: 0 },
            code: None,
            suggestion: None,
            notes: Vec::new(),
        }
    }

    /// Set the source code
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Set the span
    #[must_use]
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = span;
        self
    }

    /// Add an error code
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add a suggestion
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add a note
    #[must_use]
    pub fn add_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Render the diagnostic as a formatted string
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Error code and message
        if let Some(code) = &self.code {
            let _ = writeln!(output, "{} [{}]: {}", self.severity, code, self.message);
        } else {
            let _ = writeln!(output, "{}: {}", self.severity, self.message);
        }

        // Source context with line numbers
        if !self.source.is_empty() && self.span.end > self.span.start {
            output.push_str(&self.render_source_context());
        }

        // Suggestion
        if let Some(suggestion) = &self.suggestion {
            let _ = writeln!(output, "\n{}: {}", "help".green().bold(), suggestion);
        }

        // Notes
        for note in &self.notes {
            let _ = writeln!(output, "{}: {}", "note".blue(), note);
        }

        output
    }

    /// Render the source code context with error highlighting
    fn render_source_context(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        // Find the line containing the error
        let lines: Vec<&str> = self.source.lines().collect();
        let mut current_pos = 0;
        let mut error_line = 0;
        let mut error_col = 0;

        for (i, line) in lines.iter().enumerate() {
            let line_start = current_pos;
            let line_end = current_pos + line.len();

            if self.span.start >= line_start && self.span.start <= line_end {
                error_line = i;
                error_col = self.span.start - line_start;
                break;
            }

            current_pos = line_end + 1; // +1 for newline
        }

        // Show context (line before, error line, line after)
        let start_line = error_line.saturating_sub(1);
        let end_line = (error_line + 2).min(lines.len());

        let _ = writeln!(output, "\n{}:", " --> source".blue());

        for (i, line) in lines
            .iter()
            .enumerate()
            .skip(start_line)
            .take(end_line - start_line)
        {
            let line_num = i + 1;

            // Line number and content
            let _ = writeln!(output, "{line_num:4} | {line}");

            // Error underline
            if i == error_line {
                let underline_start = error_col;
                let underline_end = (error_col + (self.span.end - self.span.start)).min(line.len());
                let padding = " ".repeat(6 + underline_start);
                let underline = "^".repeat(underline_end - underline_start).red().bold();
                let _ = writeln!(output, "{padding}{underline}");
            }
        }

        output
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Common error messages with helpful suggestions
pub struct DiagnosticBuilder;

impl DiagnosticBuilder {
    /// Unexpected token error
    pub fn unexpected_token(token: &str, expected: &str, source: &str, span: Span) -> Diagnostic {
        Diagnostic::error(format!("unexpected token `{token}`"))
            .with_source(source)
            .with_span(span)
            .with_code("E0001")
            .with_suggestion(format!("expected {expected}"))
    }

    /// Unknown variable error
    pub fn unknown_variable(name: &str, source: &str, span: Span) -> Diagnostic {
        Diagnostic::error(format!("unknown variable `{name}`"))
            .with_source(source)
            .with_span(span)
            .with_code("E0002")
            .with_suggestion(format!(
                "did you mean to define it with `let {name} = ...`?"
            ))
            .add_note("variables must be defined before use")
    }

    /// Type mismatch error
    pub fn type_mismatch(expected: &str, found: &str, source: &str, span: Span) -> Diagnostic {
        Diagnostic::error(format!(
            "type mismatch: expected `{expected}`, found `{found}`"
        ))
        .with_source(source)
        .with_span(span)
        .with_code("E0003")
        .add_note("types must match exactly")
    }

    /// Function not found error
    pub fn function_not_found(name: &str, source: &str, span: Span) -> Diagnostic {
        Diagnostic::error(format!("function `{name}` not found"))
            .with_source(source)
            .with_span(span)
            .with_code("E0004")
            .with_suggestion(format!(
                "define the function with `fn {name}(...) {{ ... }}`"
            ))
    }

    /// Syntax error
    pub fn syntax_error(message: &str, source: &str, span: Span) -> Diagnostic {
        Diagnostic::error(message)
            .with_source(source)
            .with_span(span)
            .with_code("E0005")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diagnostic() {
        let diag = Diagnostic::error("test error")
            .with_code("E0001")
            .with_suggestion("try this instead");

        let rendered = diag.render();
        assert!(rendered.contains("error"));
        assert!(rendered.contains("E0001"));
        assert!(rendered.contains("try this instead"));
    }

    #[test]
    fn test_source_context() {
        let source = "let x = 42\nlet y = unknown\nlet z = 10";
        let diag = Diagnostic::error("unknown variable")
            .with_source(source)
            .with_span(Span { start: 19, end: 26 }) // "unknown"
            .with_suggestion("define the variable first");

        let rendered = diag.render();
        assert!(rendered.contains("let y = unknown"));
        assert!(rendered.contains("^^^^^^^")); // Error underline
    }

    #[test]
    fn test_diagnostic_builder() {
        let source = "let x = 42 + \"hello\"";
        let diag = DiagnosticBuilder::type_mismatch(
            "Integer",
            "String",
            source,
            Span { start: 13, end: 20 },
        );

        let rendered = diag.render();
        assert!(rendered.contains("type mismatch"));
        assert!(rendered.contains("Integer"));
        assert!(rendered.contains("String"));
    }
}
