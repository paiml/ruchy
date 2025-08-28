//! Enhanced error diagnostics with source code display and suggestions

use crate::frontend::error_recovery::{ParseError, ErrorSeverity};
use crate::frontend::ast::Span;
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
        let lines: Vec<String> = self.source_code.lines().map(std::string::ToString::to_string).collect();
        
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
        
        // Header with severity and location
        let severity_color = match self.error.severity {
            ErrorSeverity::Error => "\x1b[31m",   // Red
            ErrorSeverity::Warning => "\x1b[33m", // Yellow
            ErrorSeverity::Info => "\x1b[34m",    // Blue
            ErrorSeverity::Hint => "\x1b[36m",    // Cyan
        };
        let reset = "\x1b[0m";
        let bold = "\x1b[1m";
        
        // File and location header
        if let Some(ref filename) = self.filename {
            output.push_str(&format!(
                "{bold}{severity_color}error[{:?}]{reset}: {}\n",
                self.error.error_code,
                self.error.message
            ));
            output.push_str(&format!(
                "  {bold}-->{reset} {}:{}:{}\n",
                filename,
                self.error.span.start / 100 + 1, // Rough line estimate
                self.error.span.start % 100 + 1  // Rough column estimate
            ));
        } else {
            output.push_str(&format!(
                "{bold}{severity_color}error[{:?}]{reset}: {}\n",
                self.error.error_code,
                self.error.message
            ));
        }
        
        // Source code context with error highlighting
        let (context_lines, error_line_idx, col_start, col_end) = self.get_source_context();
        let line_num_start = (self.error.span.start / 100 + 1).saturating_sub(error_line_idx);
        
        for (i, line) in context_lines.iter().enumerate() {
            let line_num = line_num_start + i;
            let is_error_line = i == error_line_idx;
            
            // Line number and content
            if is_error_line {
                output.push_str(&format!(
                    "{bold}{line_num:4} |{reset} {line}\n"
                ));
                
                // Error underline
                let spaces = " ".repeat(col_start);
                let arrows = "^".repeat((col_end - col_start).max(1));
                output.push_str(&format!(
                    "     | {spaces}{severity_color}{arrows}\n"
                ));
                
                // Error message under the line
                if let Some(ref hint) = self.error.recovery_hint {
                    output.push_str(&format!(
                        "     {} {}{}{reset} {}\n",
                        "|",
                        " ".repeat(col_start),
                        severity_color,
                        hint
                    ));
                }
            } else {
                output.push_str(&format!("{line_num:4} | {line}\n"));
            }
        }
        
        // Suggestions
        if !self.suggestions.is_empty() {
            output.push_str(&format!("\n{bold}help{reset}: "));
            for suggestion in &self.suggestions {
                output.push_str(&format!("{}\n", suggestion.message));
                if let Some(ref replacement) = suggestion.replacement {
                    output.push_str(&format!("      suggested fix: `{replacement}`\n"));
                }
            }
        }
        
        output.push_str(reset);
        output
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
    
    // Common typo suggestions
    if error.message.contains("unexpected") {
        if let Some(ref _found) = error.found {
            // Token-specific suggestions would go here
            suggestions.push(Suggestion {
                message: "Check for typos or missing operators".to_string(),
                replacement: None,
                span: error.span,
            });
        }
    }
    
    // Missing semicolon suggestion
    if error.message.contains("expected") && error.message.contains("semicolon") {
        suggestions.push(Suggestion {
            message: "Add a semicolon at the end of the statement".to_string(),
            replacement: Some(";".to_string()),
            span: Span {
                start: error.span.end,
                end: error.span.end,
            },
        });
    }
    
    // Unclosed delimiter suggestions
    if error.message.contains("unclosed") || error.message.contains("unmatched") {
        if error.message.contains("paren") {
            suggestions.push(Suggestion {
                message: "Add closing parenthesis ')'".to_string(),
                replacement: Some(")".to_string()),
                span: Span {
                    start: error.span.end,
                    end: error.span.end,
                },
            });
        } else if error.message.contains("brace") {
            suggestions.push(Suggestion {
                message: "Add closing brace '}'".to_string(),
                replacement: Some("}".to_string()),
                span: Span {
                    start: error.span.end,
                    end: error.span.end,
                },
            });
        } else if error.message.contains("bracket") {
            suggestions.push(Suggestion {
                message: "Add closing bracket ']'".to_string(),
                replacement: Some("]".to_string()),
                span: Span {
                    start: error.span.end,
                    end: error.span.end,
                },
            });
        }
    }
    
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display() {
        let error = ParseError::new(
            "Unexpected token".to_string(),
            Span { start: 10, end: 15 },
        );
        
        let source = "let x = 10\nlet y = @invalid\nlet z = 30".to_string();
        let diag = Diagnostic::new(error, source);
        
        let output = format!("{diag}");
        assert!(output.contains("Unexpected token"));
    }

    #[test]
    fn test_suggestions() {
        let mut error = ParseError::new(
            "unexpected '='".to_string(),
            Span { start: 5, end: 6 },
        );
        error.found = Some(crate::frontend::lexer::Token::Equal);
        
        let suggestions = suggest_for_error(&error);
        assert!(!suggestions.is_empty());
    }
}