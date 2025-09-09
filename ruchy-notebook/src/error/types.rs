use serde::{Deserialize, Serialize};
use std::fmt;

/// Main error type for notebook operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<ErrorSpan>,
    pub suggestions: Vec<String>,
    pub severity: ErrorSeverity,
    pub help: Option<String>,
}

/// Types of errors that can occur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorKind {
    /// Syntax errors during parsing
    SyntaxError,
    /// Runtime errors during execution
    RuntimeError,
    /// Type errors
    TypeError,
    /// Undefined variable or function
    UndefinedError,
    /// Import/module errors
    ModuleError,
    /// Memory errors (out of bounds, etc.)
    MemoryError,
    /// I/O errors
    IoError,
    /// Conversion errors
    ConversionError,
    /// VM execution errors
    VmError,
}

/// Source location span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSpan {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational messages
    Info,
    /// Warnings that don't prevent execution
    Warning,
    /// Errors that prevent execution
    Error,
    /// Critical errors that may cause crashes
    Critical,
}

impl NotebookError {
    /// Create a new error
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            span: None,
            suggestions: Vec::new(),
            severity: ErrorSeverity::Error,
            help: None,
        }
    }
    
    /// Add source location
    pub fn with_span(mut self, span: ErrorSpan) -> Self {
        self.span = Some(span);
        self
    }
    
    /// Add suggestions
    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
    
    /// Add help text
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    
    /// Set severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
    
    /// Create a syntax error
    pub fn syntax(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::SyntaxError, message)
    }
    
    /// Create a runtime error
    pub fn runtime(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::RuntimeError, message)
    }
    
    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::TypeError, message)
    }
    
    /// Create an undefined error
    pub fn undefined(name: impl Into<String>) -> Self {
        Self::new(ErrorKind::UndefinedError, format!("'{}' is not defined", name.into()))
    }
    
    /// Create a VM error
    pub fn vm_error(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::VmError, message)
    }
    
    /// Get formatted error message
    pub fn formatted_message(&self) -> String {
        let mut result = format!("{}: {}", self.kind_name(), self.message);
        
        if let Some(span) = &self.span {
            result.push_str(&format!(" at line {}, column {}", span.line, span.column));
            if let Some(file) = &span.file {
                result.push_str(&format!(" in {}", file));
            }
        }
        
        if !self.suggestions.is_empty() {
            result.push_str("\n\nSuggestions:");
            for suggestion in &self.suggestions {
                result.push_str(&format!("\n  - {}", suggestion));
            }
        }
        
        if let Some(help) = &self.help {
            result.push_str(&format!("\n\nHelp: {}", help));
        }
        
        result
    }
    
    /// Get error kind name
    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            ErrorKind::SyntaxError => "SyntaxError",
            ErrorKind::RuntimeError => "RuntimeError", 
            ErrorKind::TypeError => "TypeError",
            ErrorKind::UndefinedError => "UndefinedError",
            ErrorKind::ModuleError => "ModuleError",
            ErrorKind::MemoryError => "MemoryError",
            ErrorKind::IoError => "IoError",
            ErrorKind::ConversionError => "ConversionError",
            ErrorKind::VmError => "VmError",
        }
    }
}

impl ErrorSpan {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
            file: None,
        }
    }
    
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }
}

impl fmt::Display for NotebookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted_message())
    }
}

impl std::error::Error for NotebookError {}

impl From<anyhow::Error> for NotebookError {
    fn from(err: anyhow::Error) -> Self {
        NotebookError::runtime(err.to_string())
    }
}

impl From<std::io::Error> for NotebookError {
    fn from(err: std::io::Error) -> Self {
        NotebookError::new(ErrorKind::IoError, err.to_string())
    }
}

impl From<serde_json::Error> for NotebookError {
    fn from(err: serde_json::Error) -> Self {
        NotebookError::new(ErrorKind::ConversionError, format!("JSON error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_creation() {
        let err = NotebookError::syntax("Expected ';'")
            .with_span(ErrorSpan::new(10, 15, 2, 5))
            .with_suggestions(vec!["Add semicolon".to_string()])
            .with_help("Statements must end with semicolons");
        
        assert_eq!(err.kind, ErrorKind::SyntaxError);
        assert!(err.message.contains("Expected ';'"));
        assert_eq!(err.suggestions.len(), 1);
        assert!(err.help.is_some());
    }
    
    #[test]
    fn test_error_formatting() {
        let err = NotebookError::undefined("my_var")
            .with_span(ErrorSpan::new(0, 6, 1, 1).with_file("test.ruchy"))
            .with_suggestions(vec![
                "Did you mean 'my_val'?".to_string(),
                "Check variable spelling".to_string(),
            ]);
        
        let formatted = err.formatted_message();
        assert!(formatted.contains("UndefinedError"));
        assert!(formatted.contains("'my_var' is not defined"));
        assert!(formatted.contains("line 1, column 1"));
        assert!(formatted.contains("test.ruchy"));
        assert!(formatted.contains("Did you mean 'my_val'?"));
    }
    
    #[test]
    fn test_error_severity() {
        let warning = NotebookError::syntax("Unused variable")
            .with_severity(ErrorSeverity::Warning);
        
        let error = NotebookError::runtime("Division by zero");
        
        assert_eq!(warning.severity, ErrorSeverity::Warning);
        assert_eq!(error.severity, ErrorSeverity::Error);
        assert!(error.severity > warning.severity);
    }
}