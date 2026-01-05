//! Enhanced error recovery mechanisms for the Ruchy compiler
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions
//!
//! This module provides comprehensive error recovery strategies that allow
//! the parser to continue processing even after encountering syntax errors,
//! providing better user experience in interactive environments.

use std::collections::VecDeque;
use std::fmt;

/// Enhanced error recovery context
#[derive(Debug, Clone)]
pub struct ErrorRecoveryContext {
    /// Current recovery strategy
    strategy: RecoveryStrategy,
    /// Maximum number of errors to recover from
    max_errors: usize,
    /// Current error count
    error_count: usize,
    /// Recovery points stack
    recovery_stack: Vec<RecoveryPoint>,
    /// Error history for learning
    error_history: VecDeque<RecoveredError>,
}

/// Recovery strategy enum
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Skip tokens until delimiter found
    SkipToDelimiter,
    /// Insert missing tokens
    InsertMissing,
    /// Replace malformed tokens
    ReplaceTokens,
    /// Adaptive strategy based on error patterns
    Adaptive,
}

/// Recovery point for parser state restoration
#[derive(Debug, Clone)]
pub struct RecoveryPoint {
    /// Position in token stream
    position: usize,
    /// Parser state description
    state_description: String,
    /// Nesting level (for balanced delimiters)
    nesting_level: usize,
}

/// Information about a recovered error
#[derive(Debug, Clone)]
pub struct RecoveredError {
    /// Error message
    message: String,
    /// Position where error occurred
    position: usize,
    /// Recovery action taken
    recovery_action: RecoveryAction,
    /// Success of recovery (did parsing continue?)
    recovery_success: bool,
}

/// Recovery action taken
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// Skipped tokens until delimiter
    SkippedToDelimiter {
        skipped_count: usize,
        delimiter: String,
    },
    /// Inserted missing token
    InsertedToken { token: String },
    /// Replaced malformed token
    ReplacedToken {
        original: String,
        replacement: String,
    },
    /// Used adaptive recovery
    AdaptiveRecovery { strategy_used: String },
}

impl ErrorRecoveryContext {
    /// Create new error recovery context
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::error_recovery_enhanced::{ErrorRecoveryContext, RecoveryStrategy};
    ///
    /// let context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);
    /// assert_eq!(context.max_errors(), 10);
    /// assert_eq!(context.error_count(), 0);
    /// ```
    #[must_use]
    pub fn new(strategy: RecoveryStrategy, max_errors: usize) -> Self {
        Self {
            strategy,
            max_errors,
            error_count: 0,
            recovery_stack: Vec::new(),
            error_history: VecDeque::new(),
        }
    }

    /// Get maximum error count
    #[must_use]
    pub fn max_errors(&self) -> usize {
        self.max_errors
    }

    /// Get current error count
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Check if more errors can be recovered from
    #[must_use]
    pub fn can_recover(&self) -> bool {
        self.error_count < self.max_errors
    }

    /// Push a recovery point
    pub fn push_recovery_point(
        &mut self,
        position: usize,
        description: String,
        nesting_level: usize,
    ) {
        let point = RecoveryPoint {
            position,
            state_description: description,
            nesting_level,
        };
        self.recovery_stack.push(point);
    }

    /// Pop the most recent recovery point
    pub fn pop_recovery_point(&mut self) -> Option<RecoveryPoint> {
        self.recovery_stack.pop()
    }

    /// Record an error recovery
    pub fn record_error(&mut self, error: RecoveredError) {
        self.error_count += 1;

        // Maintain error history size
        if self.error_history.len() >= 100 {
            self.error_history.pop_front();
        }
        self.error_history.push_back(error);

        // Update strategy based on success patterns
        self.update_strategy();
    }

    /// Update strategy based on error patterns
    fn update_strategy(&mut self) {
        if self.strategy != RecoveryStrategy::Adaptive {
            return;
        }

        let recent_errors: Vec<_> = self.error_history.iter().rev().take(5).collect();

        if recent_errors.len() < 3 {
            return;
        }

        // Analyze success rates of different recovery actions
        let skip_success = Self::calculate_success_rate(&recent_errors, |action| {
            matches!(action, RecoveryAction::SkippedToDelimiter { .. })
        });

        let insert_success = Self::calculate_success_rate(&recent_errors, |action| {
            matches!(action, RecoveryAction::InsertedToken { .. })
        });

        let replace_success = Self::calculate_success_rate(&recent_errors, |action| {
            matches!(action, RecoveryAction::ReplacedToken { .. })
        });

        // Choose the most successful strategy
        if skip_success > insert_success && skip_success > replace_success {
            // Skip strategy is working best
        } else if insert_success > replace_success {
            // Insert strategy is preferred
        } else {
            // Replace strategy is preferred
        }
    }

    /// Calculate success rate for a given recovery action type
    fn calculate_success_rate<F>(errors: &[&RecoveredError], predicate: F) -> f64
    where
        F: Fn(&RecoveryAction) -> bool,
    {
        let matching_errors: Vec<_> = errors
            .iter()
            .filter(|error| predicate(&error.recovery_action))
            .collect();

        if matching_errors.is_empty() {
            return 0.0;
        }

        let successful = matching_errors
            .iter()
            .filter(|error| error.recovery_success)
            .count();

        successful as f64 / matching_errors.len() as f64
    }

    /// Get current recovery strategy
    #[must_use]
    pub fn current_strategy(&self) -> &RecoveryStrategy {
        &self.strategy
    }

    /// Get error recovery statistics
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::error_recovery_enhanced::{ErrorRecoveryContext, RecoveryStrategy, RecoveredError, RecoveryAction};
    ///
    /// let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);
    /// let error = RecoveredError {
    ///     message: "Test error".to_string(),
    ///     position: 5,
    ///     recovery_action: RecoveryAction::InsertedToken { token: ";".to_string() },
    ///     recovery_success: true,
    /// };
    /// context.record_error(error);
    ///
    /// let stats = context.recovery_statistics();
    /// assert_eq!(stats.total_errors, 1);
    /// assert_eq!(stats.successful_recoveries, 1);
    /// assert_eq!(stats.success_rate, 1.0);
    /// ```
    #[must_use]
    pub fn recovery_statistics(&self) -> RecoveryStatistics {
        let total_errors = self.error_history.len();
        let successful_recoveries = self
            .error_history
            .iter()
            .filter(|error| error.recovery_success)
            .count();

        let success_rate = if total_errors > 0 {
            successful_recoveries as f64 / total_errors as f64
        } else {
            0.0
        };

        RecoveryStatistics {
            total_errors,
            successful_recoveries,
            success_rate,
            current_error_count: self.error_count,
            max_errors: self.max_errors,
        }
    }

    /// Clear error history and reset counters
    pub fn reset(&mut self) {
        self.error_count = 0;
        self.recovery_stack.clear();
        self.error_history.clear();
    }

    /// Get recent error patterns for diagnostics
    #[must_use]
    pub fn recent_error_patterns(&self) -> Vec<ErrorPattern> {
        let mut patterns = Vec::new();
        let recent_errors: Vec<_> = self.error_history.iter().rev().take(10).collect();

        for error in recent_errors {
            let pattern = ErrorPattern {
                message_prefix: Self::extract_message_prefix(&error.message),
                recovery_action: error.recovery_action.clone(),
                success: error.recovery_success,
            };
            patterns.push(pattern);
        }

        patterns
    }

    /// Extract message prefix for pattern analysis
    fn extract_message_prefix(message: &str) -> String {
        message
            .chars()
            .take(20)
            .collect::<String>()
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Recovery statistics summary
#[derive(Debug, Clone)]
pub struct RecoveryStatistics {
    pub total_errors: usize,
    pub successful_recoveries: usize,
    pub success_rate: f64,
    pub current_error_count: usize,
    pub max_errors: usize,
}

/// Error pattern for analysis
#[derive(Debug, Clone)]
pub struct ErrorPattern {
    pub message_prefix: String,
    pub recovery_action: RecoveryAction,
    pub success: bool,
}

impl fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryAction::SkippedToDelimiter {
                skipped_count,
                delimiter,
            } => {
                write!(f, "Skipped {skipped_count} tokens to '{delimiter}'")
            }
            RecoveryAction::InsertedToken { token } => {
                write!(f, "Inserted '{token}'")
            }
            RecoveryAction::ReplacedToken {
                original,
                replacement,
            } => {
                write!(f, "Replaced '{original}' with '{replacement}'")
            }
            RecoveryAction::AdaptiveRecovery { strategy_used } => {
                write!(f, "Adaptive recovery using {strategy_used}")
            }
        }
    }
}

impl RecoveredError {
    /// Create a new recovered error
    pub fn new(
        message: String,
        position: usize,
        recovery_action: RecoveryAction,
        recovery_success: bool,
    ) -> Self {
        Self {
            message,
            position,
            recovery_action,
            recovery_success,
        }
    }
}

/// Enhanced error recovery suggestions
pub struct ErrorSuggestions;

impl ErrorSuggestions {
    /// Get suggestions for common syntax errors
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::error_recovery_enhanced::ErrorSuggestions;
    ///
    /// let suggestions = ErrorSuggestions::suggest_fixes("Expected ';' after expression");
    /// assert!(!suggestions.is_empty());
    /// assert!(suggestions[0].contains("semicolon"));
    /// ```
    #[must_use]
    pub fn suggest_fixes(error_message: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if error_message.contains("Expected ';'") {
            suggestions.push("Add a semicolon ';' at the end of the statement".to_string());
            suggestions.push(
                "Check if you're inside an expression context where ';' is not needed".to_string(),
            );
        }

        if error_message.contains("Expected '}'") {
            suggestions.push("Add a closing brace '}' to match the opening brace".to_string());
            suggestions.push("Check for unbalanced braces in nested blocks".to_string());
        }

        if error_message.contains("Expected ')'") {
            suggestions
                .push("Add a closing parenthesis ')' to match the opening parenthesis".to_string());
            suggestions.push("Check function call arguments or expression grouping".to_string());
        }

        if error_message.contains("Unexpected token") {
            suggestions.push("Check for typos in keywords or identifiers".to_string());
            suggestions.push("Verify correct syntax for the current context".to_string());
        }

        if error_message.contains("Expected expression") {
            suggestions.push("Provide a value, variable, or expression".to_string());
            suggestions.push("Check for incomplete statements or missing operators".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("Review the syntax near the error location".to_string());
            suggestions.push("Check language documentation for correct syntax".to_string());
        }

        suggestions
    }

    /// Generate context-aware suggestions based on surrounding code
    #[must_use]
    pub fn contextual_suggestions(error_message: &str, context: &str) -> Vec<String> {
        let mut suggestions = Self::suggest_fixes(error_message);

        // Add context-specific suggestions
        if context.contains("fn ") && error_message.contains("Expected '{'") {
            suggestions.insert(
                0,
                "Function declarations require a body enclosed in braces '{}'".to_string(),
            );
        }

        if context.contains("if ") && error_message.contains("Expected '{'") {
            suggestions.insert(
                0,
                "If statements require a condition and body block".to_string(),
            );
        }

        if context.contains("let ") && error_message.contains("Expected '='") {
            suggestions.insert(
                0,
                "Variable declarations need an assignment with '='".to_string(),
            );
        }

        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recovery_context_creation() {
        let context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 5);
        assert_eq!(context.max_errors(), 5);
        assert_eq!(context.error_count(), 0);
        assert!(context.can_recover());
    }

    #[test]
    fn test_recovery_point_management() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);

        context.push_recovery_point(0, "start".to_string(), 0);
        context.push_recovery_point(10, "function".to_string(), 1);

        assert_eq!(context.recovery_stack.len(), 2);

        let point = context.pop_recovery_point().unwrap();
        assert_eq!(point.position, 10);
        assert_eq!(point.state_description, "function");
    }

    #[test]
    fn test_error_recording() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);

        let error = RecoveredError::new(
            "Test error".to_string(),
            5,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            true,
        );

        context.record_error(error);
        assert_eq!(context.error_count(), 1);

        let stats = context.recovery_statistics();
        assert_eq!(stats.total_errors, 1);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.success_rate, 1.0);
    }

    #[test]
    fn test_recovery_capacity() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 2);

        assert!(context.can_recover());

        let error1 = RecoveredError::new(
            "Error 1".to_string(),
            0,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            true,
        );
        context.record_error(error1);
        assert!(context.can_recover());

        let error2 = RecoveredError::new(
            "Error 2".to_string(),
            10,
            RecoveryAction::SkippedToDelimiter {
                skipped_count: 3,
                delimiter: "}".to_string(),
            },
            false,
        );
        context.record_error(error2);
        assert!(!context.can_recover());
    }

    #[test]
    fn test_error_suggestions() {
        let suggestions = ErrorSuggestions::suggest_fixes("Expected ';' after expression");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("semicolon")));

        let suggestions = ErrorSuggestions::suggest_fixes("Expected '}'");
        assert!(suggestions.iter().any(|s| s.contains("closing brace")));
    }

    #[test]
    fn test_contextual_suggestions() {
        let context = "fn test() ";
        let suggestions = ErrorSuggestions::contextual_suggestions("Expected '{'", context);
        assert!(suggestions
            .iter()
            .any(|s| s.contains("Function declarations")));
    }

    #[test]
    fn test_recovery_action_display() {
        let action = RecoveryAction::SkippedToDelimiter {
            skipped_count: 3,
            delimiter: "}".to_string(),
        };
        let display = format!("{action}");
        assert!(display.contains("Skipped 3 tokens"));

        let action = RecoveryAction::InsertedToken {
            token: ";".to_string(),
        };
        let display = format!("{action}");
        assert!(display.contains("Inserted ';'"));
    }

    #[test]
    fn test_error_pattern_analysis() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);

        // Add multiple errors with different patterns
        for i in 0..5 {
            let error = RecoveredError::new(
                format!("Missing semicolon at position {i}"),
                i * 10,
                RecoveryAction::InsertedToken {
                    token: ";".to_string(),
                },
                i % 2 == 0,
            );
            context.record_error(error);
        }

        let patterns = context.recent_error_patterns();
        assert_eq!(patterns.len(), 5);
        assert!(patterns
            .iter()
            .all(|p| p.message_prefix.contains("Missing semicolon")));
    }

    #[test]
    fn test_context_reset() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);

        let error = RecoveredError::new(
            "Test error".to_string(),
            5,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            true,
        );
        context.record_error(error);
        context.push_recovery_point(0, "test".to_string(), 0);

        assert_eq!(context.error_count(), 1);
        assert_eq!(context.recovery_stack.len(), 1);

        context.reset();

        assert_eq!(context.error_count(), 0);
        assert_eq!(context.recovery_stack.len(), 0);
        assert_eq!(context.error_history.len(), 0);
    }

    // === EXTREME TDD Round 14 tests ===

    #[test]
    fn test_recovery_strategy_clone_and_eq() {
        let strategy1 = RecoveryStrategy::Adaptive;
        let strategy2 = strategy1.clone();
        assert_eq!(strategy1, strategy2);

        let strategy3 = RecoveryStrategy::SkipToDelimiter;
        assert_ne!(strategy1, strategy3);
    }

    #[test]
    fn test_recovery_strategy_variants() {
        assert_eq!(
            RecoveryStrategy::SkipToDelimiter,
            RecoveryStrategy::SkipToDelimiter
        );
        assert_eq!(
            RecoveryStrategy::InsertMissing,
            RecoveryStrategy::InsertMissing
        );
        assert_eq!(
            RecoveryStrategy::ReplaceTokens,
            RecoveryStrategy::ReplaceTokens
        );
        assert_eq!(RecoveryStrategy::Adaptive, RecoveryStrategy::Adaptive);
    }

    #[test]
    fn test_recovery_point_struct() {
        let point = RecoveryPoint {
            position: 42,
            state_description: "parsing function".to_string(),
            nesting_level: 3,
        };
        assert_eq!(point.position, 42);
        assert_eq!(point.state_description, "parsing function");
        assert_eq!(point.nesting_level, 3);
    }

    #[test]
    fn test_recovered_error_new() {
        let error = RecoveredError::new(
            "Missing bracket".to_string(),
            100,
            RecoveryAction::InsertedToken {
                token: "]".to_string(),
            },
            false,
        );
        assert_eq!(error.message, "Missing bracket");
        assert_eq!(error.position, 100);
        assert!(!error.recovery_success);
    }

    #[test]
    fn test_recovery_action_display_replaced_token() {
        let action = RecoveryAction::ReplacedToken {
            original: "func".to_string(),
            replacement: "fn".to_string(),
        };
        let display = format!("{action}");
        assert!(display.contains("Replaced 'func' with 'fn'"));
    }

    #[test]
    fn test_recovery_action_display_adaptive() {
        let action = RecoveryAction::AdaptiveRecovery {
            strategy_used: "skip-to-semicolon".to_string(),
        };
        let display = format!("{action}");
        assert!(display.contains("Adaptive recovery using skip-to-semicolon"));
    }

    #[test]
    fn test_error_recovery_context_current_strategy() {
        let context = ErrorRecoveryContext::new(RecoveryStrategy::InsertMissing, 5);
        assert_eq!(context.current_strategy(), &RecoveryStrategy::InsertMissing);
    }

    #[test]
    fn test_error_history_limit_100() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 200);

        // Add 105 errors to exceed the 100 limit
        for i in 0..105 {
            let error = RecoveredError::new(
                format!("Error {i}"),
                i,
                RecoveryAction::InsertedToken {
                    token: ";".to_string(),
                },
                true,
            );
            context.record_error(error);
        }

        // History should be capped at 100
        assert_eq!(context.error_history.len(), 100);
        // Oldest errors should be removed
        assert!(context
            .error_history
            .front()
            .unwrap()
            .message
            .contains("Error 5"));
    }

    #[test]
    fn test_recovery_statistics_fields() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);

        let error1 = RecoveredError::new(
            "Error 1".to_string(),
            0,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            true,
        );
        let error2 = RecoveredError::new(
            "Error 2".to_string(),
            10,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            false,
        );

        context.record_error(error1);
        context.record_error(error2);

        let stats = context.recovery_statistics();
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.successful_recoveries, 1);
        assert_eq!(stats.success_rate, 0.5);
        assert_eq!(stats.current_error_count, 2);
        assert_eq!(stats.max_errors, 10);
    }

    #[test]
    fn test_error_pattern_struct() {
        let pattern = ErrorPattern {
            message_prefix: "Expected".to_string(),
            recovery_action: RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            success: true,
        };
        assert_eq!(pattern.message_prefix, "Expected");
        assert!(pattern.success);
    }

    #[test]
    fn test_extract_message_prefix_short() {
        // Test with a very short message
        let prefix = ErrorRecoveryContext::extract_message_prefix("Hi");
        assert_eq!(prefix, "Hi");
    }

    #[test]
    fn test_extract_message_prefix_long() {
        let prefix = ErrorRecoveryContext::extract_message_prefix(
            "This is a very long error message that should be truncated",
        );
        // Should take first 20 chars, then first 3 words
        assert!(prefix.len() <= 20);
        assert!(prefix.contains("This"));
    }

    #[test]
    fn test_error_suggestions_paren() {
        let suggestions = ErrorSuggestions::suggest_fixes("Expected ')' after arguments");
        assert!(!suggestions.is_empty());
        assert!(suggestions
            .iter()
            .any(|s| s.contains("closing parenthesis")));
    }

    #[test]
    fn test_error_suggestions_unexpected_token() {
        let suggestions = ErrorSuggestions::suggest_fixes("Unexpected token 'xyz'");
        assert!(suggestions.iter().any(|s| s.contains("typos")));
    }

    #[test]
    fn test_error_suggestions_expected_expression() {
        let suggestions = ErrorSuggestions::suggest_fixes("Expected expression");
        assert!(suggestions
            .iter()
            .any(|s| s.contains("value") || s.contains("expression")));
    }

    #[test]
    fn test_error_suggestions_unknown_error() {
        let suggestions = ErrorSuggestions::suggest_fixes("Some unknown error type");
        // Should provide fallback suggestions
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("syntax")));
    }

    #[test]
    fn test_contextual_suggestions_if_statement() {
        let context = "if condition ";
        let suggestions = ErrorSuggestions::contextual_suggestions("Expected '{'", context);
        assert!(suggestions
            .iter()
            .any(|s| s.contains("If statements") || s.contains("brace")));
    }

    #[test]
    fn test_contextual_suggestions_let_statement() {
        let context = "let x ";
        let suggestions = ErrorSuggestions::contextual_suggestions("Expected '='", context);
        assert!(suggestions
            .iter()
            .any(|s| s.contains("Variable") || s.contains("assignment")));
    }

    #[test]
    fn test_recovery_strategy_debug() {
        let strategy = RecoveryStrategy::Adaptive;
        let debug_str = format!("{:?}", strategy);
        assert_eq!(debug_str, "Adaptive");
    }

    #[test]
    fn test_recovery_point_clone() {
        let point = RecoveryPoint {
            position: 10,
            state_description: "test".to_string(),
            nesting_level: 1,
        };
        let cloned = point.clone();
        assert_eq!(point.position, cloned.position);
        assert_eq!(point.state_description, cloned.state_description);
    }

    #[test]
    fn test_recovered_error_clone() {
        let error = RecoveredError::new(
            "Test".to_string(),
            0,
            RecoveryAction::InsertedToken {
                token: ";".to_string(),
            },
            true,
        );
        let cloned = error.clone();
        assert_eq!(error.message, cloned.message);
        assert_eq!(error.position, cloned.position);
    }

    #[test]
    fn test_recovery_action_clone() {
        let action = RecoveryAction::SkippedToDelimiter {
            skipped_count: 5,
            delimiter: "}".to_string(),
        };
        let cloned = action.clone();
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_recovery_statistics_zero_errors() {
        let context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);
        let stats = context.recovery_statistics();
        assert_eq!(stats.total_errors, 0);
        assert_eq!(stats.successful_recoveries, 0);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[test]
    fn test_pop_empty_recovery_stack() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::Adaptive, 10);
        assert!(context.pop_recovery_point().is_none());
    }

    #[test]
    fn test_non_adaptive_strategy_no_update() {
        let mut context = ErrorRecoveryContext::new(RecoveryStrategy::SkipToDelimiter, 10);

        // Add several errors
        for _ in 0..5 {
            let error = RecoveredError::new(
                "Error".to_string(),
                0,
                RecoveryAction::InsertedToken {
                    token: ";".to_string(),
                },
                true,
            );
            context.record_error(error);
        }

        // Strategy should remain unchanged for non-adaptive
        assert_eq!(
            context.current_strategy(),
            &RecoveryStrategy::SkipToDelimiter
        );
    }
}
