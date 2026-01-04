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
#[derive(Debug)]
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

    /// Create a snapshot of current bindings (complexity: 1)
    pub fn bindings_snapshot(&self) -> HashMap<String, Value> {
        self.bindings.clone()
    }

    /// Restore bindings from snapshot (complexity: 1)
    pub fn restore_bindings(&mut self, snapshot: HashMap<String, Value>) {
        self.bindings = snapshot;
    }

    /// Set a variable in the bindings (complexity: 1)
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.bindings.insert(name, value);
    }

    /// Get a variable from the bindings (complexity: 1)
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }

    /// Check if in debug mode (complexity: 1)
    pub fn is_debug_mode(&self) -> bool {
        matches!(self.mode, ReplMode::Debug)
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

        state
            .get_bindings_mut()
            .insert("x".to_string(), Value::Integer(42));
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

    // ReplMode Display tests
    #[test]
    fn test_repl_mode_display_normal() {
        assert_eq!(format!("{}", ReplMode::Normal), "normal");
    }

    #[test]
    fn test_repl_mode_display_debug() {
        assert_eq!(format!("{}", ReplMode::Debug), "debug");
    }

    #[test]
    fn test_repl_mode_display_ast() {
        assert_eq!(format!("{}", ReplMode::Ast), "ast");
    }

    #[test]
    fn test_repl_mode_display_transpile() {
        assert_eq!(format!("{}", ReplMode::Transpile), "transpile");
    }

    // ReplMode Debug, Clone, Copy, PartialEq, Eq tests
    #[test]
    fn test_repl_mode_debug() {
        let mode = ReplMode::Normal;
        assert!(format!("{:?}", mode).contains("Normal"));
    }

    #[test]
    fn test_repl_mode_clone() {
        let mode = ReplMode::Debug;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_repl_mode_copy() {
        let mode = ReplMode::Ast;
        let copied = mode; // Copy semantics
        assert_eq!(mode, copied);
    }

    #[test]
    fn test_repl_mode_eq() {
        assert_eq!(ReplMode::Normal, ReplMode::Normal);
        assert_ne!(ReplMode::Normal, ReplMode::Debug);
    }

    // ReplState Default test
    #[test]
    fn test_repl_state_default() {
        let state = ReplState::default();
        assert!(matches!(state.get_mode(), ReplMode::Normal));
        assert!(state.get_bindings().is_empty());
    }

    // ReplState Debug test
    #[test]
    fn test_repl_state_debug() {
        let state = ReplState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("ReplState"));
    }

    // Result history tests
    #[test]
    fn test_add_to_result_history() {
        let mut state = ReplState::new();
        state.add_to_result_history(Value::Integer(42));
        assert_eq!(state.result_history_len(), 1);
    }

    #[test]
    fn test_get_result_history() {
        let mut state = ReplState::new();
        state.add_to_result_history(Value::Integer(1));
        state.add_to_result_history(Value::Integer(2));

        let history = state.get_result_history();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear_result_history() {
        let mut state = ReplState::new();
        state.add_to_result_history(Value::Integer(42));
        state.clear_result_history();
        assert_eq!(state.result_history_len(), 0);
    }

    #[test]
    fn test_result_history_max_size() {
        let mut state = ReplState::new();
        state.max_history = 2;

        state.add_to_result_history(Value::Integer(1));
        state.add_to_result_history(Value::Integer(2));
        state.add_to_result_history(Value::Integer(3)); // Should remove first

        assert_eq!(state.result_history_len(), 2);
        let history = state.get_result_history();
        assert!(matches!(history[0], Value::Integer(2)));
        assert!(matches!(history[1], Value::Integer(3)));
    }

    // Peak memory tests
    #[test]
    fn test_update_peak_memory_initial() {
        let mut state = ReplState::new();
        state.update_peak_memory(1000);
        assert_eq!(state.get_peak_memory(), 1000);
    }

    #[test]
    fn test_update_peak_memory_higher() {
        let mut state = ReplState::new();
        state.update_peak_memory(1000);
        state.update_peak_memory(2000);
        assert_eq!(state.get_peak_memory(), 2000);
    }

    #[test]
    fn test_update_peak_memory_lower_no_change() {
        let mut state = ReplState::new();
        state.update_peak_memory(2000);
        state.update_peak_memory(1000); // Should not change
        assert_eq!(state.get_peak_memory(), 2000);
    }

    // Bindings snapshot tests
    #[test]
    fn test_bindings_snapshot() {
        let mut state = ReplState::new();
        state.set_variable("x".to_string(), Value::Integer(42));

        let snapshot = state.bindings_snapshot();
        assert!(snapshot.contains_key("x"));
    }

    #[test]
    fn test_restore_bindings() {
        let mut state = ReplState::new();
        state.set_variable("x".to_string(), Value::Integer(42));
        let snapshot = state.bindings_snapshot();

        state.clear_bindings();
        assert!(state.get_bindings().is_empty());

        state.restore_bindings(snapshot);
        assert!(state.get_bindings().contains_key("x"));
    }

    // Variable methods tests
    #[test]
    fn test_set_variable() {
        let mut state = ReplState::new();
        state.set_variable("y".to_string(), Value::Float(3.14));
        assert!(state.get_bindings().contains_key("y"));
    }

    #[test]
    fn test_get_variable_exists() {
        let mut state = ReplState::new();
        state.set_variable("z".to_string(), Value::Bool(true));
        let value = state.get_variable("z");
        assert!(value.is_some());
        assert!(matches!(value.unwrap(), Value::Bool(true)));
    }

    #[test]
    fn test_get_variable_not_exists() {
        let state = ReplState::new();
        let value = state.get_variable("nonexistent");
        assert!(value.is_none());
    }

    // is_debug_mode tests
    #[test]
    fn test_is_debug_mode_true() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Debug);
        assert!(state.is_debug_mode());
    }

    #[test]
    fn test_is_debug_mode_false() {
        let state = ReplState::new();
        assert!(!state.is_debug_mode());
    }

    #[test]
    fn test_is_debug_mode_other_modes() {
        let mut state = ReplState::new();

        state.set_mode(ReplMode::Ast);
        assert!(!state.is_debug_mode());

        state.set_mode(ReplMode::Transpile);
        assert!(!state.is_debug_mode());
    }

    // Mode set/get round trips
    #[test]
    fn test_mode_transpile() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Transpile);
        assert!(matches!(state.get_mode(), ReplMode::Transpile));
    }
}
