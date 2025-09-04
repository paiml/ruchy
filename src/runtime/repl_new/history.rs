//! History management module for REPL
//! Handles command history, result tracking, and session management

use anyhow::Result;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use crate::runtime::repl::Value;

/// Configuration for history management
pub struct HistoryConfig {
    /// Maximum number of commands to keep in memory
    pub max_commands: usize,
    /// Maximum number of results to keep for _ and __ variables
    pub max_results: usize,
    /// Whether to persist history to disk
    pub persist: bool,
    /// Path to history file
    pub history_file: Option<String>,
}

impl Default for HistoryConfig {
    fn default() -> Self {
        Self {
            max_commands: 1000,
            max_results: 10,
            persist: true,
            history_file: Some(".ruchy_history".to_string()),
        }
    }
}

/// Manages REPL command and result history
pub struct HistoryManager {
    /// Command history
    commands: VecDeque<String>,
    /// Result history for _ and __ variables
    results: VecDeque<Value>,
    /// Configuration
    config: HistoryConfig,
    /// Current position in history for navigation
    position: Option<usize>,
}

impl HistoryManager {
    /// Create a new history manager (complexity: 3)
    pub fn new(config: HistoryConfig) -> Self {
        Self {
            commands: VecDeque::with_capacity(config.max_commands),
            results: VecDeque::with_capacity(config.max_results),
            config,
            position: None,
        }
    }

    /// Add a command to history (complexity: 4)
    pub fn add_command(&mut self, command: String) {
        // Skip empty commands
        if command.trim().is_empty() {
            return;
        }
        
        // Skip duplicates of the last command
        if let Some(last) = self.commands.back() {
            if last == &command {
                return;
            }
        }
        
        // Add command and trim if needed
        self.commands.push_back(command);
        if self.commands.len() > self.config.max_commands {
            self.commands.pop_front();
        }
        
        // Reset navigation position
        self.position = None;
    }

    /// Add a result to history (complexity: 3)
    pub fn add_result(&mut self, result: Value) {
        self.results.push_back(result);
        if self.results.len() > self.config.max_results {
            self.results.pop_front();
        }
    }

    /// Get the last result (for _ variable) (complexity: 1)
    pub fn last_result(&self) -> Option<&Value> {
        self.results.back()
    }

    /// Get the second-to-last result (for __ variable) (complexity: 2)
    pub fn second_last_result(&self) -> Option<&Value> {
        let len = self.results.len();
        if len >= 2 {
            self.results.get(len - 2)
        } else {
            None
        }
    }

    /// Get result by index from the end (complexity: 2)
    pub fn get_result(&self, index: usize) -> Option<&Value> {
        let len = self.results.len();
        if index < len {
            self.results.get(len - 1 - index)
        } else {
            None
        }
    }

    /// Navigate to previous command in history (complexity: 3)
    pub fn previous_command(&mut self) -> Option<&str> {
        if self.commands.is_empty() {
            return None;
        }
        
        match self.position {
            None => {
                self.position = Some(self.commands.len() - 1);
                self.commands.get(self.commands.len() - 1).map(|s| s.as_str())
            }
            Some(pos) if pos > 0 => {
                self.position = Some(pos - 1);
                self.commands.get(pos - 1).map(|s| s.as_str())
            }
            _ => None,
        }
    }

    /// Navigate to next command in history (complexity: 3)
    pub fn next_command(&mut self) -> Option<&str> {
        match self.position {
            Some(pos) if pos < self.commands.len() - 1 => {
                self.position = Some(pos + 1);
                self.commands.get(pos + 1).map(|s| s.as_str())
            }
            Some(_) => {
                self.position = None;
                None
            }
            None => None,
        }
    }

    /// Reset navigation position (complexity: 1)
    pub fn reset_position(&mut self) {
        self.position = None;
    }

    /// Get all commands as a vector (complexity: 2)
    pub fn get_commands(&self) -> Vec<String> {
        self.commands.iter().cloned().collect()
    }

    /// Get command count (complexity: 1)
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Clear all history (complexity: 1)
    pub fn clear(&mut self) {
        self.commands.clear();
        self.results.clear();
        self.position = None;
    }

    /// Search history for commands containing a pattern (complexity: 5)
    pub fn search(&self, pattern: &str) -> Vec<(usize, String)> {
        let pattern_lower = pattern.to_lowercase();
        self.commands
            .iter()
            .enumerate()
            .filter(|(_, cmd)| cmd.to_lowercase().contains(&pattern_lower))
            .map(|(idx, cmd)| (idx, cmd.clone()))
            .collect()
    }

    /// Save history to file (complexity: 5)
    pub fn save_to_file(&self) -> Result<()> {
        if !self.config.persist {
            return Ok(());
        }
        
        if let Some(ref path) = self.config.history_file {
            let content = self.commands
                .iter()
                .map(|cmd| cmd.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(path, content)?;
        }
        
        Ok(())
    }

    /// Load history from file (complexity: 6)
    pub fn load_from_file(&mut self) -> Result<()> {
        if !self.config.persist {
            return Ok(());
        }
        
        if let Some(ref path) = self.config.history_file {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                for line in content.lines() {
                    if !line.trim().is_empty() {
                        self.add_command(line.to_string());
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Get formatted history display (complexity: 4)
    pub fn format_display(&self) -> String {
        if self.commands.is_empty() {
            return "No history yet".to_string();
        }
        
        self.commands
            .iter()
            .enumerate()
            .map(|(i, cmd)| format!("{:4}: {}", i + 1, cmd))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get recent commands (complexity: 3)
    pub fn recent_commands(&self, count: usize) -> Vec<String> {
        let start = self.commands.len().saturating_sub(count);
        self.commands
            .iter()
            .skip(start)
            .cloned()
            .collect()
    }

    /// Check if command exists in history (complexity: 3)
    pub fn contains_command(&self, command: &str) -> bool {
        self.commands.iter().any(|cmd| cmd == command)
    }

    /// Remove command at index (complexity: 4)
    pub fn remove_command(&mut self, index: usize) -> Option<String> {
        if index < self.commands.len() {
            self.commands.remove(index)
        } else {
            None
        }
    }

    /// Get statistics about history (complexity: 3)
    pub fn get_stats(&self) -> HistoryStats {
        HistoryStats {
            total_commands: self.commands.len(),
            unique_commands: self.count_unique_commands(),
            total_results: self.results.len(),
            session_size: self.estimate_memory_usage(),
        }
    }

    /// Count unique commands (complexity: 4)
    fn count_unique_commands(&self) -> usize {
        use std::collections::HashSet;
        let unique: HashSet<_> = self.commands.iter().collect();
        unique.len()
    }

    /// Estimate memory usage in bytes (complexity: 4)
    fn estimate_memory_usage(&self) -> usize {
        let commands_size: usize = self.commands
            .iter()
            .map(|cmd| cmd.len() + std::mem::size_of::<String>())
            .sum();
        
        let results_size = self.results.len() * std::mem::size_of::<Value>();
        
        commands_size + results_size
    }
}

/// Statistics about history
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub unique_commands: usize,
    pub total_results: usize,
    pub session_size: usize,
}

impl HistoryStats {
    /// Format stats for display (complexity: 2)
    pub fn format_display(&self) -> String {
        format!(
            "History Statistics:\n  \
             Commands: {} ({}% unique)\n  \
             Results cached: {}\n  \
             Memory usage: ~{} KB",
            self.total_commands,
            if self.total_commands > 0 {
                (self.unique_commands * 100) / self.total_commands
            } else {
                0
            },
            self.total_results,
            self.session_size / 1024
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_manager_creation() {
        let mgr = HistoryManager::new(HistoryConfig::default());
        assert_eq!(mgr.command_count(), 0);
    }

    #[test]
    fn test_add_command() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_command("test command".to_string());
        assert_eq!(mgr.command_count(), 1);
    }

    #[test]
    fn test_skip_duplicate_command() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_command("test".to_string());
        mgr.add_command("test".to_string());
        assert_eq!(mgr.command_count(), 1);
    }

    #[test]
    fn test_result_history() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_result(Value::Int(10));
        mgr.add_result(Value::Int(20));
        
        assert_eq!(mgr.last_result(), Some(&Value::Int(20)));
        assert_eq!(mgr.second_last_result(), Some(&Value::Int(10)));
    }

    #[test]
    fn test_history_navigation() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_command("first".to_string());
        mgr.add_command("second".to_string());
        mgr.add_command("third".to_string());
        
        assert_eq!(mgr.previous_command(), Some("third"));
        assert_eq!(mgr.previous_command(), Some("second"));
        assert_eq!(mgr.previous_command(), Some("first"));
        assert_eq!(mgr.previous_command(), None);
        
        assert_eq!(mgr.next_command(), Some("second"));
        assert_eq!(mgr.next_command(), Some("third"));
        assert_eq!(mgr.next_command(), None);
    }

    #[test]
    fn test_history_search() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_command("let x = 10".to_string());
        mgr.add_command("println(x)".to_string());
        mgr.add_command("let y = 20".to_string());
        
        let results = mgr.search("let");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1, "let x = 10");
        assert_eq!(results[1].1, "let y = 20");
    }

    #[test]
    fn test_history_limit() {
        let mut config = HistoryConfig::default();
        config.max_commands = 3;
        let mut mgr = HistoryManager::new(config);
        
        mgr.add_command("cmd1".to_string());
        mgr.add_command("cmd2".to_string());
        mgr.add_command("cmd3".to_string());
        mgr.add_command("cmd4".to_string());
        
        assert_eq!(mgr.command_count(), 3);
        assert!(!mgr.contains_command("cmd1"));
        assert!(mgr.contains_command("cmd4"));
    }

    #[test]
    fn test_history_stats() {
        let mut mgr = HistoryManager::new(HistoryConfig::default());
        mgr.add_command("test".to_string());
        mgr.add_command("test".to_string()); // Duplicate, won't be added
        mgr.add_command("other".to_string());
        mgr.add_result(Value::Int(42));
        
        let stats = mgr.get_stats();
        assert_eq!(stats.total_commands, 2);
        assert_eq!(stats.unique_commands, 2);
        assert_eq!(stats.total_results, 1);
        assert!(stats.session_size > 0);
    }
}