//! Error handling and recovery module for REPL
//! Provides error recovery strategies and user-friendly error messages

use anyhow::Result;
use std::fmt;

/// REPL error types
#[derive(Debug, Clone)]
pub enum ReplError {
    /// Parse error
    ParseError { message: String, position: usize },
    /// Type error
    TypeError { expected: String, actual: String },
    /// Undefined variable
    UndefinedVariable { name: String },
    /// Undefined function
    UndefinedFunction { name: String },
    /// Arity mismatch
    ArityMismatch { expected: usize, actual: usize },
    /// Division by zero
    DivisionByZero,
    /// Index out of bounds
    IndexOutOfBounds { index: i64, length: usize },
    /// Key not found
    KeyNotFound { key: String },
    /// Stack overflow
    StackOverflow { depth: usize },
    /// Timeout
    Timeout { seconds: u64 },
    /// Iteration limit exceeded
    IterationLimit { limit: usize },
    /// Invalid operation
    InvalidOperation { operation: String, reason: String },
    /// IO error
    IoError { message: String },
    /// Custom error
    Custom { message: String },
}

impl fmt::Display for ReplError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReplError::ParseError { message, position } => {
                write!(f, "Parse error at position {}: {}", position, message)
            }
            ReplError::TypeError { expected, actual } => {
                write!(f, "Type error: expected {}, got {}", expected, actual)
            }
            ReplError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {}", name)
            }
            ReplError::UndefinedFunction { name } => {
                write!(f, "Undefined function: {}", name)
            }
            ReplError::ArityMismatch { expected, actual } => {
                write!(f, "Function expects {} arguments, got {}", expected, actual)
            }
            ReplError::DivisionByZero => {
                write!(f, "Division by zero")
            }
            ReplError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds (length: {})", index, length)
            }
            ReplError::KeyNotFound { key } => {
                write!(f, "Key '{}' not found", key)
            }
            ReplError::StackOverflow { depth } => {
                write!(f, "Stack overflow at depth {}", depth)
            }
            ReplError::Timeout { seconds } => {
                write!(f, "Evaluation timeout after {} seconds", seconds)
            }
            ReplError::IterationLimit { limit } => {
                write!(f, "Iteration limit {} exceeded", limit)
            }
            ReplError::InvalidOperation { operation, reason } => {
                write!(f, "Invalid operation '{}': {}", operation, reason)
            }
            ReplError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
            ReplError::Custom { message } => {
                write!(f, "{}", message)
            }
        }
    }
}

impl std::error::Error for ReplError {}

/// Error recovery strategy
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Skip the current statement
    Skip,
    /// Use a default value
    UseDefault(String),
    /// Retry with modification
    Retry { modification: String },
    /// Ask user for input
    AskUser { prompt: String },
    /// Abort execution
    Abort,
}

/// Error context for better diagnostics
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Source line where error occurred
    pub source_line: Option<String>,
    /// Line number
    pub line_number: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Call stack
    pub call_stack: Vec<String>,
    /// Suggestions for fixing
    pub suggestions: Vec<String>,
}

impl ErrorContext {
    /// Create empty context (complexity: 1)
    pub fn new() -> Self {
        Self {
            source_line: None,
            line_number: None,
            column: None,
            call_stack: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Add source location (complexity: 2)
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line_number = Some(line);
        self.column = Some(column);
        self
    }

    /// Add source line (complexity: 1)
    pub fn with_source(mut self, source: String) -> Self {
        self.source_line = Some(source);
        self
    }

    /// Add call stack frame (complexity: 2)
    pub fn push_frame(&mut self, frame: String) {
        self.call_stack.push(frame);
    }

    /// Add suggestion (complexity: 2)
    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }

    /// Format for display (complexity: 6)
    pub fn format_display(&self) -> String {
        let mut output = String::new();
        
        if let Some(ref line) = self.source_line {
            output.push_str(&format!("\n  {}\n", line));
            
            if let Some(col) = self.column {
                output.push_str(&format!("  {}^\n", " ".repeat(col)));
            }
        }
        
        if !self.call_stack.is_empty() {
            output.push_str("\nCall stack:\n");
            for (i, frame) in self.call_stack.iter().rev().enumerate() {
                output.push_str(&format!("  #{}: {}\n", i, frame));
            }
        }
        
        if !self.suggestions.is_empty() {
            output.push_str("\nSuggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }
        
        output
    }
}

/// Error handler for REPL
pub struct ErrorHandler {
    /// Error history
    error_history: Vec<(ReplError, ErrorContext)>,
    /// Recovery strategies
    recovery_strategies: Vec<(ErrorPattern, RecoveryStrategy)>,
    /// Maximum errors to keep
    max_history: usize,
    /// Enable suggestions
    suggestions_enabled: bool,
}

impl ErrorHandler {
    /// Create new error handler (complexity: 2)
    pub fn new() -> Self {
        Self {
            error_history: Vec::new(),
            recovery_strategies: Self::default_strategies(),
            max_history: 100,
            suggestions_enabled: true,
        }
    }

    /// Handle an error (complexity: 7)
    pub fn handle_error(&mut self, error: ReplError, context: ErrorContext) -> Result<RecoveryStrategy> {
        // Record error
        self.record_error(error.clone(), context.clone());
        
        // Find recovery strategy
        let strategy = self.find_recovery_strategy(&error);
        
        // Generate suggestions if enabled
        let mut enhanced_context = context;
        if self.suggestions_enabled {
            for suggestion in self.generate_suggestions(&error) {
                enhanced_context.add_suggestion(suggestion);
            }
        }
        
        // Format error message
        let message = format!(
            "Error: {}{}", 
            error, 
            enhanced_context.format_display()
        );
        
        eprintln!("{}", message);
        
        Ok(strategy)
    }

    /// Record error in history (complexity: 3)
    fn record_error(&mut self, error: ReplError, context: ErrorContext) {
        self.error_history.push((error, context));
        
        if self.error_history.len() > self.max_history {
            self.error_history.remove(0);
        }
    }

    /// Find recovery strategy (complexity: 5)
    fn find_recovery_strategy(&self, error: &ReplError) -> RecoveryStrategy {
        for (pattern, strategy) in &self.recovery_strategies {
            if pattern.matches(error) {
                return strategy.clone();
            }
        }
        
        RecoveryStrategy::Skip
    }

    /// Generate suggestions (complexity: 8)
    fn generate_suggestions(&self, error: &ReplError) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match error {
            ReplError::UndefinedVariable { name } => {
                suggestions.push(format!("Did you mean to define '{}'? Use: let {} = value", name, name));
                
                // Find similar variable names
                if let Some(similar) = self.find_similar_name(name) {
                    suggestions.push(format!("Did you mean '{}'?", similar));
                }
            }
            ReplError::UndefinedFunction { name } => {
                suggestions.push(format!("Define the function with: fn {}() {{ ... }}", name));
            }
            ReplError::TypeError { expected, .. } => {
                suggestions.push(format!("Convert to {} type", expected));
            }
            ReplError::DivisionByZero => {
                suggestions.push("Check for zero before division".to_string());
                suggestions.push("Use a conditional: if divisor != 0 { ... }".to_string());
            }
            ReplError::IndexOutOfBounds { index, length } => {
                if *index < 0 {
                    suggestions.push("Use positive indices".to_string());
                } else {
                    suggestions.push(format!("Valid range: 0..{}", length - 1));
                }
            }
            ReplError::ArityMismatch { expected, actual } => {
                if actual < expected {
                    suggestions.push(format!("Add {} more argument(s)", expected - actual));
                } else {
                    suggestions.push(format!("Remove {} argument(s)", actual - expected));
                }
            }
            _ => {}
        }
        
        suggestions
    }

    /// Find similar name (complexity: 6)
    fn find_similar_name(&self, name: &str) -> Option<String> {
        // This would need access to the symbol table
        // For now, return common typos
        match name {
            "pritnln" | "prtinln" | "pritln" => Some("println".to_string()),
            "lenth" | "lenght" => Some("length".to_string()),
            "ture" => Some("true".to_string()),
            "flase" | "fales" => Some("false".to_string()),
            "fucntion" | "funtion" => Some("function".to_string()),
            _ => None,
        }
    }

    /// Get error statistics (complexity: 5)
    pub fn get_statistics(&self) -> ErrorStatistics {
        let mut stats = ErrorStatistics::new();
        
        for (error, _) in &self.error_history {
            stats.record_error(error);
        }
        
        stats
    }

    /// Clear error history (complexity: 1)
    pub fn clear_history(&mut self) {
        self.error_history.clear();
    }

    /// Get recent errors (complexity: 3)
    pub fn recent_errors(&self, count: usize) -> Vec<(ReplError, ErrorContext)> {
        let start = self.error_history.len().saturating_sub(count);
        self.error_history[start..].to_vec()
    }

    /// Default recovery strategies (complexity: 2)
    fn default_strategies() -> Vec<(ErrorPattern, RecoveryStrategy)> {
        vec![
            (ErrorPattern::TypeError, RecoveryStrategy::Skip),
            (ErrorPattern::UndefinedVar, RecoveryStrategy::UseDefault("null".to_string())),
            (ErrorPattern::DivByZero, RecoveryStrategy::UseDefault("0".to_string())),
            (ErrorPattern::Timeout, RecoveryStrategy::Abort),
            (ErrorPattern::StackOverflow, RecoveryStrategy::Abort),
        ]
    }

    /// Add custom recovery strategy (complexity: 2)
    pub fn add_recovery_strategy(&mut self, pattern: ErrorPattern, strategy: RecoveryStrategy) {
        self.recovery_strategies.push((pattern, strategy));
    }
}

/// Error pattern for matching
#[derive(Debug, Clone)]
pub enum ErrorPattern {
    TypeError,
    UndefinedVar,
    UndefinedFunc,
    DivByZero,
    OutOfBounds,
    Timeout,
    StackOverflow,
    Any,
}

impl ErrorPattern {
    /// Check if pattern matches error (complexity: 3)
    fn matches(&self, error: &ReplError) -> bool {
        match (self, error) {
            (ErrorPattern::TypeError, ReplError::TypeError { .. }) => true,
            (ErrorPattern::UndefinedVar, ReplError::UndefinedVariable { .. }) => true,
            (ErrorPattern::UndefinedFunc, ReplError::UndefinedFunction { .. }) => true,
            (ErrorPattern::DivByZero, ReplError::DivisionByZero) => true,
            (ErrorPattern::OutOfBounds, ReplError::IndexOutOfBounds { .. }) => true,
            (ErrorPattern::Timeout, ReplError::Timeout { .. }) => true,
            (ErrorPattern::StackOverflow, ReplError::StackOverflow { .. }) => true,
            (ErrorPattern::Any, _) => true,
            _ => false,
        }
    }
}

/// Error statistics
#[derive(Debug, Clone)]
pub struct ErrorStatistics {
    /// Total errors
    pub total_errors: usize,
    /// Errors by type
    pub errors_by_type: std::collections::HashMap<String, usize>,
    /// Most common error
    pub most_common: Option<String>,
}

impl ErrorStatistics {
    /// Create new statistics (complexity: 1)
    fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_type: std::collections::HashMap::new(),
            most_common: None,
        }
    }

    /// Record an error (complexity: 4)
    fn record_error(&mut self, error: &ReplError) {
        self.total_errors += 1;
        
        let error_type = self.error_type_name(error);
        *self.errors_by_type.entry(error_type.clone()).or_insert(0) += 1;
        
        // Update most common
        let max_count = self.errors_by_type.values().max().copied().unwrap_or(0);
        self.most_common = self.errors_by_type
            .iter()
            .find(|(_, &count)| count == max_count)
            .map(|(name, _)| name.clone());
    }

    /// Get error type name (complexity: 3)
    fn error_type_name(&self, error: &ReplError) -> String {
        match error {
            ReplError::ParseError { .. } => "ParseError",
            ReplError::TypeError { .. } => "TypeError",
            ReplError::UndefinedVariable { .. } => "UndefinedVariable",
            ReplError::UndefinedFunction { .. } => "UndefinedFunction",
            ReplError::ArityMismatch { .. } => "ArityMismatch",
            ReplError::DivisionByZero => "DivisionByZero",
            ReplError::IndexOutOfBounds { .. } => "IndexOutOfBounds",
            ReplError::KeyNotFound { .. } => "KeyNotFound",
            ReplError::StackOverflow { .. } => "StackOverflow",
            ReplError::Timeout { .. } => "Timeout",
            ReplError::IterationLimit { .. } => "IterationLimit",
            ReplError::InvalidOperation { .. } => "InvalidOperation",
            ReplError::IoError { .. } => "IoError",
            ReplError::Custom { .. } => "Custom",
        }.to_string()
    }

    /// Format statistics (complexity: 4)
    pub fn format_display(&self) -> String {
        let mut output = format!("Error Statistics:\n  Total: {}\n", self.total_errors);
        
        if let Some(ref common) = self.most_common {
            output.push_str(&format!("  Most common: {}\n", common));
        }
        
        output.push_str("  By type:\n");
        for (error_type, count) in &self.errors_by_type {
            output.push_str(&format!("    {}: {}\n", error_type, count));
        }
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ReplError::UndefinedVariable { 
            name: "x".to_string() 
        };
        assert_eq!(err.to_string(), "Undefined variable: x");
    }

    #[test]
    fn test_error_context() {
        let mut ctx = ErrorContext::new()
            .with_location(10, 5)
            .with_source("let x = y + 1".to_string());
        
        ctx.add_suggestion("Define 'y' before using it".to_string());
        
        let display = ctx.format_display();
        assert!(display.contains("let x = y + 1"));
        assert!(display.contains("Define 'y'"));
    }

    #[test]
    fn test_error_handler() {
        let mut handler = ErrorHandler::new();
        
        let error = ReplError::DivisionByZero;
        let context = ErrorContext::new();
        
        let strategy = handler.handle_error(error, context).unwrap();
        assert!(matches!(strategy, RecoveryStrategy::UseDefault(_)));
    }

    #[test]
    fn test_error_pattern_matching() {
        let pattern = ErrorPattern::TypeError;
        let error = ReplError::TypeError {
            expected: "Int".to_string(),
            actual: "String".to_string(),
        };
        
        assert!(pattern.matches(&error));
    }

    #[test]
    fn test_error_statistics() {
        let mut stats = ErrorStatistics::new();
        
        stats.record_error(&ReplError::DivisionByZero);
        stats.record_error(&ReplError::DivisionByZero);
        stats.record_error(&ReplError::Timeout { seconds: 30 });
        
        assert_eq!(stats.total_errors, 3);
        assert_eq!(stats.most_common, Some("DivisionByZero".to_string()));
    }
}