//! Parser module with error recovery
//!
//! This module provides deterministic error recovery for the Ruchy parser,
//! ensuring predictable behavior even on malformed input.

pub mod error_recovery;

pub use error_recovery::{
    ErrorContext, ErrorNode, ErrorRecoverable, ErrorRecovery, ExprWithError, RecoveryRules,
    RecoveryStrategy, SourceLocation,
};
