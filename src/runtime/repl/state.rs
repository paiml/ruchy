//! REPL State Management
//!
//! Manages REPL state including variables, history, and mode.

use crate::runtime::interpreter::Value;
use std::collections::HashMap;

/// REPL operating modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplMode {
    /// Normal evaluation mode
    Normal,
    /// Debug mode with extra output
    Debug,
    /// AST display mode
    Ast,
    /// Transpile mode showing Rust output
    Transpile,
}

impl std::fmt::Display for ReplMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplMode::Normal => write!(f, "normal"),
            ReplMode::Debug => write!(f, "debug"),
            ReplMode::Ast => write!(f, "ast"),
            ReplMode::Transpile => write!(f, "transpile"),
        }
    }
}

/// REPL state container
pub struct ReplState {
    /// Current operating mode
    mode: ReplMode,
    /// Variable bindings
    bindings: HashMap<String, Value>,
    /// Command history
    history: Vec<String>,
    /// Result history (evaluation results)
    result_history: Vec<Value>,
    /// Maximum history size
    max_history: usize,
    /// Peak memory usage
    peak_memory: usize,
}

impl ReplState {
    /// Create a new REPL state (complexity: 2)
    pub fn new() -> Self {
        Self {
            mode: ReplMode::Normal,
            bindings: HashMap::new(),
            history: Vec::new(),
            result_history: Vec::new(),
            max_history: 1000,
            peak_memory: 0,
        }
    }

    /// Get current mode (complexity: 1)
    pub fn get_mode(&self) -> ReplMode {
        self.mode
    }

    /// Set current mode (complexity: 1)
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
    }

    /// Get variable bindings (complexity: 1)
    pub fn get_bindings(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    /// Get mutable variable bindings (complexity: 1)
    pub fn get_bindings_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.bindings
    }

    /// Clear variable bindings (complexity: 1)
    pub fn clear_bindings(&mut self) {
        self.bindings.clear();
    }

    /// Add to command history (complexity: 3)
    pub fn add_to_history(&mut self, command: String) {
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(command);
    }

    /// Get command history (complexity: 1)
    pub fn get_history(&self) -> &[String] {
        &self.history
    }

    /// Clear command history (complexity: 1)
    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Add to result history (complexity: 3)
    pub fn add_to_result_history(&mut self, result: Value) {
        if self.result_history.len() >= self.max_history {
            self.result_history.remove(0);
        }
        self.result_history.push(result);
    }

    /// Get result history length (complexity: 1)
    pub fn result_history_len(&self) -> usize {
        self.result_history.len()
    }

    /// Get result history (complexity: 1)
    pub fn get_result_history(&self) -> &[Value] {
        &self.result_history
    }

    /// Clear result history (complexity: 1)
    pub fn clear_result_history(&mut self) {
        self.result_history.clear();
    }

    /// Update peak memory (complexity: 2)
    pub fn update_peak_memory(&mut self, current: usize) {
        if current > self.peak_memory {
            self.peak_memory = current;
        }
    }

    /// Get peak memory (complexity: 1)
    pub fn get_peak_memory(&self) -> usize {
        self.peak_memory
    }
}

impl Default for ReplState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_state_creation() {
        let state = ReplState::new();
        assert!(matches!(state.get_mode(), ReplMode::Normal));
        assert!(state.get_bindings().is_empty());
        assert!(state.get_history().is_empty());
    }

    #[test]
    fn test_mode_changes() {
        let mut state = ReplState::new();

        state.set_mode(ReplMode::Debug);
        assert!(matches!(state.get_mode(), ReplMode::Debug));

        state.set_mode(ReplMode::Ast);
        assert!(matches!(state.get_mode(), ReplMode::Ast));
    }

    #[test]
    fn test_history_management() {
        let mut state = ReplState::new();

        state.add_to_history("first command".to_string());
        state.add_to_history("second command".to_string());

        let history = state.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "first command");
        assert_eq!(history[1], "second command");

        state.clear_history();
        assert!(state.get_history().is_empty());
    }

    #[test]
    fn test_bindings_management() {
        let mut state = ReplState::new();

        state.get_bindings_mut().insert("x".to_string(), Value::Integer(42));
        assert!(state.get_bindings().contains_key("x"));

        state.clear_bindings();
        assert!(state.get_bindings().is_empty());
    }

    #[test]
    fn test_history_max_size() {
        let mut state = ReplState::new();
        state.max_history = 2; // Set small limit for testing

        state.add_to_history("cmd1".to_string());
        state.add_to_history("cmd2".to_string());
        state.add_to_history("cmd3".to_string()); // Should remove cmd1

        let history = state.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "cmd2");
        assert_eq!(history[1], "cmd3");
    }
}