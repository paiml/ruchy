//! Error recovery and restart system for REPL v3
//!
//! Implements a Common Lisp-style condition/restart system for
//! graceful error recovery with user choice.

#![allow(clippy::print_stdout)] // REPL needs to print to stdout
#![allow(clippy::print_stderr)] // REPL needs to print errors

use anyhow::Result;
use colored::Colorize;
use std::fmt;

/// Restart option for error recovery
#[derive(Clone, Debug)]
pub struct Restart {
    pub name: String,
    pub description: String,
    pub handler: RestartHandler,
}

/// Handler types for different restart strategies
#[derive(Clone, Debug)]
pub enum RestartHandler {
    /// Use a default value
    UseDefault(String),
    /// Retry with modified input
    RetryWith(String),
    /// Skip this operation
    Skip,
    /// Abort and return to prompt
    Abort,
    /// Custom handler
    Custom(String),
}

impl fmt::Display for Restart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name.yellow(), self.description)
    }
}

/// Error with recovery options
pub struct RecoverableError {
    pub message: String,
    pub restarts: Vec<Restart>,
    pub context: ErrorContext,
}

/// Context information for errors
#[derive(Clone, Debug)]
pub struct ErrorContext {
    pub line: usize,
    pub column: usize,
    pub source: String,
    pub suggestion: Option<String>,
}

impl RecoverableError {
    /// Create a new recoverable error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            restarts: vec![Restart {
                name: "abort".to_string(),
                description: "Cancel operation and return to prompt".to_string(),
                handler: RestartHandler::Abort,
            }],
            context: ErrorContext {
                line: 0,
                column: 0,
                source: String::new(),
                suggestion: None,
            },
        }
    }

    /// Add a restart option
    #[must_use]
    pub fn add_restart(mut self, restart: Restart) -> Self {
        self.restarts.insert(0, restart); // Insert at beginning for priority
        self
    }

    /// Set error context
    #[must_use]
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = context;
        self
    }

    /// Display error with recovery options
    pub fn display(&self) {
        // Error header
        eprintln!("{}: {}", "Error".red().bold(), self.message);

        // Source context if available
        if !self.context.source.is_empty() {
            eprintln!();
            eprintln!("  {} | {}", self.context.line, self.context.source);
            eprintln!(
                "  {} | {}",
                " ".repeat(self.context.line.to_string().len()),
                "^".repeat(self.context.column).red()
            );
        }

        // Suggestion if available
        if let Some(ref suggestion) = self.context.suggestion {
            eprintln!();
            eprintln!("{}: {}", "Hint".green(), suggestion);
        }

        // Restart options
        if !self.restarts.is_empty() {
            eprintln!();
            eprintln!("{}:", "Available restarts".cyan());
            for (i, restart) in self.restarts.iter().enumerate() {
                eprintln!("  {}: {}", i + 1, restart);
            }
        }
    }

    /// Handle user's restart choice
    ///
    /// # Errors
    ///
    /// Returns an error if the choice is invalid
    ///
    /// # Example
    ///
    /// ```
    /// use ruchy::runtime::repl_v3::recovery::{RecoverableError, Restart, RestartHandler};
    ///
    /// let error = RecoverableError::new("Test error");
    /// let action = error.handle_restart(1);  // Select first restart
    /// assert!(action.is_ok());
    /// ```
    pub fn handle_restart(&self, choice: usize) -> Result<RestartAction> {
        if choice == 0 || choice > self.restarts.len() {
            return Ok(RestartAction::Abort);
        }

        let restart = &self.restarts[choice - 1];

        match &restart.handler {
            RestartHandler::UseDefault(value) => Ok(RestartAction::UseValue(value.clone())),
            RestartHandler::RetryWith(input) => Ok(RestartAction::Retry(input.clone())),
            RestartHandler::Skip => Ok(RestartAction::Skip),
            RestartHandler::Abort => Ok(RestartAction::Abort),
            RestartHandler::Custom(action) => Ok(RestartAction::Custom(action.clone())),
        }
    }
}

/// Action to take after restart selection
#[derive(Debug)]
pub enum RestartAction {
    /// Use this value instead
    UseValue(String),
    /// Retry with new input
    Retry(String),
    /// Skip this operation
    Skip,
    /// Abort to prompt
    Abort,
    /// Custom action
    Custom(String),
}

/// Recovery manager for the REPL
pub struct RecoveryManager {
    /// History of errors for pattern detection
    error_history: Vec<String>,
    /// Common recovery patterns
    patterns: Vec<RecoveryPattern>,
}

/// Pattern for automatic recovery suggestions
struct RecoveryPattern {
    error_pattern: String,
    suggestion: String,
    restart: Restart,
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecoveryManager {
    pub fn new() -> Self {
        Self {
            error_history: Vec::new(),
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<RecoveryPattern> {
        vec![
            RecoveryPattern {
                error_pattern: "undefined variable".to_string(),
                suggestion: "Did you mean to define it first?".to_string(),
                restart: Restart {
                    name: "define".to_string(),
                    description: "Define with default value".to_string(),
                    handler: RestartHandler::UseDefault("0".to_string()),
                },
            },
            RecoveryPattern {
                error_pattern: "type mismatch".to_string(),
                suggestion: "Check the expected types".to_string(),
                restart: Restart {
                    name: "coerce".to_string(),
                    description: "Try type coercion".to_string(),
                    handler: RestartHandler::Custom("coerce".to_string()),
                },
            },
        ]
    }

    /// Enhance error with recovery suggestions
    pub fn enhance_error(&mut self, mut error: RecoverableError) -> RecoverableError {
        // Record error
        self.error_history.push(error.message.clone());

        // Find matching patterns
        for pattern in &self.patterns {
            if error
                .message
                .to_lowercase()
                .contains(&pattern.error_pattern)
            {
                error = error.add_restart(pattern.restart.clone());
                if error.context.suggestion.is_none() {
                    error.context.suggestion = Some(pattern.suggestion.clone());
                }
            }
        }

        error
    }
}
