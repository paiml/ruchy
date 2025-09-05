//! REPL state management module
//! Extracted from repl.rs for modularity (complexity: ≤10 per function)

use super::value::Value;
use super::error_recovery::DebugInfo;
use std::collections::HashMap;
use std::time::Instant;

/// REPL mode determines how input is processed
#[derive(Debug, Clone, PartialEq)]
pub enum ReplMode {
    /// Normal evaluation mode
    Normal,
    /// Debug mode with step-by-step evaluation
    Debug,
    /// Interactive tutorial mode
    Tutorial,
    /// Benchmark mode for performance testing
    Benchmark,
    /// Script mode for file execution
    Script,
}

impl ReplMode {
    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            ReplMode::Normal => "Normal",
            ReplMode::Debug => "Debug",
            ReplMode::Tutorial => "Tutorial",
            ReplMode::Benchmark => "Benchmark",
            ReplMode::Script => "Script",
        }
    }

    /// Check if mode supports interactive features
    pub fn is_interactive(&self) -> bool {
        matches!(self, ReplMode::Normal | ReplMode::Debug | ReplMode::Tutorial)
    }
}

/// Checkpoint for O(1) state recovery using persistent data structures
#[derive(Debug, Clone)]
pub struct Checkpoint {
    pub id: usize,
    pub bindings: HashMap<String, Value>,
    pub timestamp: Instant,
    pub description: String,
    pub parent_id: Option<usize>,
}

/// REPL transaction state for reliable evaluation
#[derive(Debug, Clone)]
pub enum ReplState {
    /// Ready for new evaluation
    Ready,
    /// Currently evaluating expression
    Evaluating {
        start_time: Instant,
        expression: String,
    },
    /// Evaluation completed successfully
    Completed {
        result: Value,
        duration: std::time::Duration,
    },
    /// Error occurred during evaluation
    Error {
        message: String,
        debug_info: Option<DebugInfo>,
    },
    /// In recovery mode after error
    Recovering {
        checkpoint_id: usize,
    },
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        id: usize,
        bindings: HashMap<String, Value>,
        description: String,
        parent_id: Option<usize>,
    ) -> Self {
        Self {
            id,
            bindings,
            timestamp: Instant::now(),
            description,
            parent_id,
        }
    }

    /// Get elapsed time since checkpoint creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }

    /// Check if checkpoint has a parent
    pub fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }

    /// Create a child checkpoint
    pub fn create_child(&self, id: usize, description: String) -> Self {
        Self {
            id,
            bindings: self.bindings.clone(),
            timestamp: Instant::now(),
            description,
            parent_id: Some(self.id),
        }
    }
}

impl ReplState {
    /// Check if REPL is ready for input
    pub fn is_ready(&self) -> bool {
        matches!(self, ReplState::Ready)
    }

    /// Check if REPL is currently evaluating
    pub fn is_evaluating(&self) -> bool {
        matches!(self, ReplState::Evaluating { .. })
    }

    /// Check if REPL has an error
    pub fn is_error(&self) -> bool {
        matches!(self, ReplState::Error { .. })
    }

    /// Check if REPL is in recovery mode
    pub fn is_recovering(&self) -> bool {
        matches!(self, ReplState::Recovering { .. })
    }

    /// Transition to evaluating state
    pub fn start_evaluation(expression: String) -> Self {
        ReplState::Evaluating {
            start_time: Instant::now(),
            expression,
        }
    }

    /// Transition to completed state
    pub fn complete_evaluation(result: Value, start_time: Instant) -> Self {
        ReplState::Completed {
            result,
            duration: start_time.elapsed(),
        }
    }

    /// Transition to error state
    pub fn error(message: String, debug_info: Option<DebugInfo>) -> Self {
        ReplState::Error {
            message,
            debug_info,
        }
    }

    /// Transition to recovery state
    pub fn start_recovery(checkpoint_id: usize) -> Self {
        ReplState::Recovering { checkpoint_id }
    }

    /// Get state description
    pub fn description(&self) -> String {
        match self {
            ReplState::Ready => "Ready".to_string(),
            ReplState::Evaluating { expression, .. } => {
                format!("Evaluating: {}", expression)
            }
            ReplState::Completed { duration, .. } => {
                format!("Completed in {:?}", duration)
            }
            ReplState::Error { message, .. } => {
                format!("Error: {}", message)
            }
            ReplState::Recovering { checkpoint_id } => {
                format!("Recovering from checkpoint {}", checkpoint_id)
            }
        }
    }
}