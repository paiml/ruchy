//! Error recovery and debugging module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use crate::frontend::ast::Expr;
use super::value::Value;
use std::collections::HashMap;

/// Debug information for post-mortem analysis
#[derive(Debug, Clone)]
pub struct DebugInfo {
    pub expression: Expr,
    pub error_message: String,
    pub stack_trace: Vec<String>,
    pub local_bindings: HashMap<String, Value>,
    pub call_depth: usize,
    pub timestamp: std::time::Instant,
}

/// Interactive error recovery options
#[derive(Debug, Clone)]
pub enum RecoveryOption {
    /// Retry evaluation with modified bindings
    RetryWithBindings(HashMap<String, Value>),
    /// Skip current expression and continue
    Skip,
    /// Enter debug mode at error point
    Debug,
    /// Rollback to previous checkpoint
    Rollback,
    /// Abort evaluation
    Abort,
    /// View detailed error information
    ViewDetails,
}

/// Error recovery context with available options
#[derive(Debug, Clone)]
pub struct ErrorRecovery {
    pub error: String,
    pub debug_info: DebugInfo,
    pub available_options: Vec<RecoveryOption>,
    pub suggestions: Vec<String>,
    pub recovery_history: Vec<RecoveryOption>,
}

/// Recovery result after user chooses an option
#[derive(Debug, Clone)]
pub enum RecoveryResult {
    /// Continue with modified state
    Continue(HashMap<String, Value>),
    /// Skip and proceed
    Skip,
    /// Enter debug mode
    Debug,
    /// Rollback executed
    Rollback,
    /// Abort execution
    Abort,
}

impl DebugInfo {
    /// Create new debug info
    pub fn new(
        expression: Expr,
        error_message: String,
        local_bindings: HashMap<String, Value>,
        call_depth: usize,
    ) -> Self {
        Self {
            expression,
            error_message,
            stack_trace: Vec::new(),
            local_bindings,
            call_depth,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Add a frame to the stack trace
    pub fn add_stack_frame(&mut self, frame: String) {
        self.stack_trace.push(frame);
    }

    /// Get elapsed time since error
    pub fn elapsed(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

impl ErrorRecovery {
    /// Create new error recovery context
    pub fn new(error: String, debug_info: DebugInfo) -> Self {
        let mut recovery = Self {
            error,
            debug_info,
            available_options: Vec::new(),
            suggestions: Vec::new(),
            recovery_history: Vec::new(),
        };
        recovery.populate_default_options();
        recovery.generate_suggestions();
        recovery
    }

    /// Populate default recovery options
    fn populate_default_options(&mut self) {
        self.available_options.push(RecoveryOption::ViewDetails);
        self.available_options.push(RecoveryOption::Skip);
        self.available_options.push(RecoveryOption::Debug);
        self.available_options.push(RecoveryOption::Rollback);
        self.available_options.push(RecoveryOption::Abort);
    }

    /// Generate suggestions based on error type
    fn generate_suggestions(&mut self) {
        if self.error.contains("undefined variable") {
            self.suggestions.push("Define the variable before use".to_string());
            self.suggestions.push("Check for typos in variable name".to_string());
        } else if self.error.contains("type mismatch") {
            self.suggestions.push("Check argument types".to_string());
            self.suggestions.push("Use type conversion functions".to_string());
        } else if self.error.contains("division by zero") {
            self.suggestions.push("Add zero check before division".to_string());
        }
    }

    /// Add custom recovery option
    pub fn add_option(&mut self, option: RecoveryOption) {
        self.available_options.push(option);
    }

    /// Record recovery attempt
    pub fn record_attempt(&mut self, option: RecoveryOption) {
        self.recovery_history.push(option);
    }

    /// Check if recovery has been attempted
    pub fn has_attempts(&self) -> bool {
        !self.recovery_history.is_empty()
    }

    /// Get number of recovery attempts
    pub fn attempt_count(&self) -> usize {
        self.recovery_history.len()
    }
}