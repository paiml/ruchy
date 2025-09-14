//! Parser module with error recovery
//!
//! This module provides deterministic error recovery for the Ruchy parser,
//! ensuring predictable behavior even on malformed input.
pub mod error_recovery;
pub use error_recovery::{
    ErrorContext, ErrorNode, ErrorRecoverable, ErrorRecovery, ExprWithError, RecoveryRules,
    RecoveryStrategy, SourceLocation,
};

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 12: Parser module tests

    #[test]
    fn test_error_recovery_exports() {
        // Verify that all error recovery types are exported
        // This is a compile-time test - if it compiles, exports exist
        let _ = ErrorContext;
        let _ = ErrorNode;
        let _ = ErrorRecovery;
        let _ = RecoveryStrategy;
        let _ = SourceLocation;
    }

    #[test]
    fn test_error_recovery_module_exists() {
        // Verify error_recovery module exists - compile-time test
        use super::error_recovery;
        let _ = error_recovery;
    }
}
