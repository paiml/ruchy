//! EXTREME Quality REPL Module Structure
//!
//! This module exports the new high-quality REPL implementation

pub mod commands;
pub mod state;
pub mod evaluation;
pub mod completion;
pub mod formatting;

// Re-export the main types
pub use self::commands::{CommandRegistry, CommandResult, CommandContext};
pub use self::state::{ReplState, ReplMode};
pub use self::evaluation::{Evaluator, EvalResult};
pub use self::completion::CompletionEngine;
pub use self::formatting::{format_value, format_error, format_ast};