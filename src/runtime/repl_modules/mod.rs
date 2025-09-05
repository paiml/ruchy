//! Modularized REPL implementation
//! Refactored from monolithic 9,204-line repl.rs

pub mod value;
pub mod evaluator;
pub mod commands;
pub mod history;
pub mod error_recovery;
pub mod state;
pub mod config;
pub mod memory;

// Re-export commonly used types
pub use value::{Value, DataFrameColumn};
pub use evaluator::Evaluator;
pub use commands::CommandHandler;
pub use history::HistoryManager;
pub use error_recovery::{ErrorRecovery, RecoveryOption, RecoveryResult};
pub use state::{ReplState, Checkpoint};
pub use config::ReplConfig;
pub use memory::MemoryTracker;