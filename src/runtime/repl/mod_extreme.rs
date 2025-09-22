//! EXTREME Quality REPL Module Structure
//!
//! This module exports the new high-quality REPL implementation

pub mod commands;
pub mod completion;
pub mod evaluation;
pub mod formatting;
pub mod state;

// Re-export the main types
pub use self::commands::{CommandContext, CommandRegistry, CommandResult};
pub use self::completion::CompletionEngine;
pub use self::evaluation::{EvalResult, Evaluator};
pub use self::formatting::{format_ast, format_error, format_value};
pub use self::state::{ReplMode, ReplState};
