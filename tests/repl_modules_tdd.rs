//! TDD tests for refactored REPL modules
//! Target: Improve coverage of new module structure with complexity ≤10

#[cfg(test)]
mod repl_config_tests {
    use ruchy::runtime::repl_modules::config::ReplConfig;
    
    // Test 1: Default config creation (complexity: 2)
    #[test]
    fn test_default_config() {
        let config = ReplConfig::default();
        assert!(config.enable_history);
        assert!(config.enable_multiline);
        assert_eq!(config.history_limit, 1000);
    }
    
    // Test 2: Config builder pattern (complexity: 3)
    #[test]
    fn test_config_builder() {
        let config = ReplConfig::default()
            .with_history(false)
            .with_multiline(false)
            .with_history_limit(500);
        
        assert!(!config.enable_history);
        assert!(!config.enable_multiline);
        assert_eq!(config.history_limit, 500);
    }
    
    // Test 3: Config validation (complexity: 4)
    #[test]
    fn test_config_validation() {
        let config = ReplConfig::default();
        assert!(config.validate().is_ok());
        
        let invalid_config = ReplConfig {
            history_limit: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }
}

#[cfg(test)]
mod repl_memory_tests {
    use ruchy::runtime::repl_modules::memory::{MemoryManager, MemoryStats};
    
    // Test 4: Memory manager creation (complexity: 2)
    #[test]
    fn test_memory_manager_new() {
        let manager = MemoryManager::new(1024);
        assert_eq!(manager.limit(), 1024);
        assert_eq!(manager.used(), 0);
    }
    
    // Test 5: Memory allocation (complexity: 5)
    #[test]
    fn test_memory_allocation() {
        let mut manager = MemoryManager::new(1024);
        
        assert!(manager.allocate(100).is_ok());
        assert_eq!(manager.used(), 100);
        
        assert!(manager.allocate(924).is_ok());
        assert_eq!(manager.used(), 1024);
        
        assert!(manager.allocate(1).is_err()); // Over limit
    }
    
    // Test 6: Memory deallocation (complexity: 4)
    #[test]
    fn test_memory_deallocation() {
        let mut manager = MemoryManager::new(1024);
        
        manager.allocate(500).unwrap();
        assert_eq!(manager.used(), 500);
        
        manager.deallocate(200);
        assert_eq!(manager.used(), 300);
        
        manager.deallocate(400); // More than allocated
        assert_eq!(manager.used(), 0); // Should clamp to 0
    }
    
    // Test 7: Memory stats (complexity: 3)
    #[test]
    fn test_memory_stats() {
        let mut manager = MemoryManager::new(1024);
        manager.allocate(256).unwrap();
        
        let stats = manager.stats();
        assert_eq!(stats.used, 256);
        assert_eq!(stats.limit, 1024);
        assert_eq!(stats.available(), 768);
    }
}

#[cfg(test)]
mod repl_error_recovery_tests {
    use ruchy::runtime::repl_modules::error_recovery::{ErrorRecovery, RecoveryStrategy};
    
    // Test 8: Error recovery creation (complexity: 2)
    #[test]
    fn test_error_recovery_new() {
        let recovery = ErrorRecovery::new();
        assert!(recovery.can_recover());
        assert_eq!(recovery.attempt_count(), 0);
    }
    
    // Test 9: Recovery attempt tracking (complexity: 5)
    #[test]
    fn test_recovery_attempts() {
        let mut recovery = ErrorRecovery::new();
        
        for i in 1..=3 {
            assert!(recovery.attempt_recovery().is_ok());
            assert_eq!(recovery.attempt_count(), i);
        }
        
        // After 3 attempts, should fail
        assert!(recovery.attempt_recovery().is_err());
        assert!(!recovery.can_recover());
    }
    
    // Test 10: Recovery strategy selection (complexity: 4)
    #[test]
    fn test_recovery_strategy() {
        let recovery = ErrorRecovery::new();
        
        let strategy1 = recovery.get_strategy_for_error("syntax error");
        assert_eq!(strategy1, RecoveryStrategy::ParsePartial);
        
        let strategy2 = recovery.get_strategy_for_error("undefined variable");
        assert_eq!(strategy2, RecoveryStrategy::SkipStatement);
        
        let strategy3 = recovery.get_strategy_for_error("unknown error");
        assert_eq!(strategy3, RecoveryStrategy::Reset);
    }
    
    // Test 11: Recovery reset (complexity: 3)
    #[test]
    fn test_recovery_reset() {
        let mut recovery = ErrorRecovery::new();
        
        recovery.attempt_recovery().unwrap();
        recovery.attempt_recovery().unwrap();
        assert_eq!(recovery.attempt_count(), 2);
        
        recovery.reset();
        assert_eq!(recovery.attempt_count(), 0);
        assert!(recovery.can_recover());
    }
}

#[cfg(test)]
mod repl_state_tests {
    use ruchy::runtime::repl_modules::state::{ReplState, ExecutionMode};
    use std::collections::HashMap;
    
    // Test 12: State initialization (complexity: 3)
    #[test]
    fn test_state_initialization() {
        let state = ReplState::new();
        assert_eq!(state.execution_mode(), ExecutionMode::Interactive);
        assert!(state.variables().is_empty());
        assert!(!state.is_recording());
    }
    
    // Test 13: Variable management (complexity: 5)
    #[test]
    fn test_variable_management() {
        let mut state = ReplState::new();
        
        state.set_variable("x", "10");
        state.set_variable("y", "20");
        
        assert_eq!(state.get_variable("x"), Some(&"10".to_string()));
        assert_eq!(state.get_variable("y"), Some(&"20".to_string()));
        assert_eq!(state.get_variable("z"), None);
        
        assert_eq!(state.variables().len(), 2);
    }
    
    // Test 14: Execution mode switching (complexity: 4)
    #[test]
    fn test_execution_mode_switching() {
        let mut state = ReplState::new();
        
        assert_eq!(state.execution_mode(), ExecutionMode::Interactive);
        
        state.set_execution_mode(ExecutionMode::Script);
        assert_eq!(state.execution_mode(), ExecutionMode::Script);
        
        state.set_execution_mode(ExecutionMode::Debug);
        assert_eq!(state.execution_mode(), ExecutionMode::Debug);
    }
    
    // Test 15: Recording state (complexity: 3)
    #[test]
    fn test_recording_state() {
        let mut state = ReplState::new();
        
        assert!(!state.is_recording());
        
        state.start_recording();
        assert!(state.is_recording());
        
        state.stop_recording();
        assert!(!state.is_recording());
    }
    
    // Test 16: State reset (complexity: 4)
    #[test]
    fn test_state_reset() {
        let mut state = ReplState::new();
        
        state.set_variable("x", "10");
        state.set_execution_mode(ExecutionMode::Debug);
        state.start_recording();
        
        state.reset();
        
        assert!(state.variables().is_empty());
        assert_eq!(state.execution_mode(), ExecutionMode::Interactive);
        assert!(!state.is_recording());
    }
}

#[cfg(test)]
mod repl_history_tests {
    use ruchy::runtime::repl_modules::history::{History, HistoryEntry};
    
    // Test 17: History creation (complexity: 2)
    #[test]
    fn test_history_creation() {
        let history = History::new(100);
        assert_eq!(history.len(), 0);
        assert_eq!(history.capacity(), 100);
    }
    
    // Test 18: Adding entries (complexity: 5)
    #[test]
    fn test_adding_entries() {
        let mut history = History::new(100);
        
        history.add("let x = 5");
        history.add("print(x)");
        history.add("x + 10");
        
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().command, "let x = 5");
        assert_eq!(history.get(2).unwrap().command, "x + 10");
    }
    
    // Test 19: History limit enforcement (complexity: 6)
    #[test]
    fn test_history_limit() {
        let mut history = History::new(3);
        
        history.add("cmd1");
        history.add("cmd2");
        history.add("cmd3");
        history.add("cmd4"); // Should evict cmd1
        
        assert_eq!(history.len(), 3);
        assert_eq!(history.get(0).unwrap().command, "cmd2");
        assert_eq!(history.get(2).unwrap().command, "cmd4");
    }
    
    // Test 20: History search (complexity: 5)
    #[test]
    fn test_history_search() {
        let mut history = History::new(100);
        
        history.add("let x = 5");
        history.add("let y = 10");
        history.add("print(x)");
        history.add("let z = 15");
        
        let results = history.search("let");
        assert_eq!(results.len(), 3);
        
        let print_results = history.search("print");
        assert_eq!(print_results.len(), 1);
    }
    
    // Test 21: History navigation (complexity: 5)
    #[test]
    fn test_history_navigation() {
        let mut history = History::new(100);
        
        history.add("cmd1");
        history.add("cmd2");
        history.add("cmd3");
        
        assert_eq!(history.previous(), Some("cmd3"));
        assert_eq!(history.previous(), Some("cmd2"));
        assert_eq!(history.previous(), Some("cmd1"));
        assert_eq!(history.previous(), None);
        
        assert_eq!(history.next(), Some("cmd1"));
        assert_eq!(history.next(), Some("cmd2"));
    }
    
    // Test 22: History clear (complexity: 3)
    #[test]
    fn test_history_clear() {
        let mut history = History::new(100);
        
        history.add("cmd1");
        history.add("cmd2");
        assert_eq!(history.len(), 2);
        
        history.clear();
        assert_eq!(history.len(), 0);
        assert_eq!(history.get(0), None);
    }
}

#[cfg(test)]
mod repl_commands_tests {
    use ruchy::runtime::repl_modules::commands::{Command, CommandParser, CommandResult};
    
    // Test 23: Command parsing (complexity: 5)
    #[test]
    fn test_command_parsing() {
        let parser = CommandParser::new();
        
        assert_eq!(parser.parse(":help"), Some(Command::Help));
        assert_eq!(parser.parse(":quit"), Some(Command::Quit));
        assert_eq!(parser.parse(":clear"), Some(Command::Clear));
        assert_eq!(parser.parse(":load file.ruchy"), Some(Command::Load("file.ruchy".to_string())));
        assert_eq!(parser.parse("let x = 5"), None); // Not a command
    }
    
    // Test 24: Command execution (complexity: 6)
    #[test]
    fn test_command_execution() {
        let executor = CommandExecutor::new();
        
        let result1 = executor.execute(Command::Help);
        assert!(matches!(result1, CommandResult::Success(_)));
        
        let result2 = executor.execute(Command::Clear);
        assert!(matches!(result2, CommandResult::Success(_)));
        
        let result3 = executor.execute(Command::Load("nonexistent.ruchy".to_string()));
        assert!(matches!(result3, CommandResult::Error(_)));
    }
    
    // Test 25: Custom command registration (complexity: 5)
    #[test]
    fn test_custom_command_registration() {
        let mut parser = CommandParser::new();
        
        parser.register(":custom", Command::Custom("test".to_string()));
        
        assert_eq!(parser.parse(":custom"), Some(Command::Custom("test".to_string())));
        assert_eq!(parser.parse(":unknown"), None);
        
        assert!(parser.commands().contains(&":custom"));
    }
}

// Mock implementations for testing (complexity ≤10 each)
mod mock_impls {
    use std::collections::HashMap;
    
    // Mock ReplConfig implementation
    #[derive(Debug, Clone)]
    pub struct ReplConfig {
        pub enable_history: bool,
        pub enable_multiline: bool,
        pub history_limit: usize,
    }
    
    impl Default for ReplConfig {
        fn default() -> Self {
            Self {
                enable_history: true,
                enable_multiline: true,
                history_limit: 1000,
            }
        }
    }
    
    impl ReplConfig {
        pub fn with_history(mut self, enable: bool) -> Self {
            self.enable_history = enable;
            self
        }
        
        pub fn with_multiline(mut self, enable: bool) -> Self {
            self.enable_multiline = enable;
            self
        }
        
        pub fn with_history_limit(mut self, limit: usize) -> Self {
            self.history_limit = limit;
            self
        }
        
        pub fn validate(&self) -> Result<(), String> {
            if self.history_limit == 0 {
                Err("History limit must be greater than 0".to_string())
            } else {
                Ok(())
            }
        }
    }
    
    // Mock MemoryManager implementation
    pub struct MemoryManager {
        limit: usize,
        used: usize,
    }
    
    impl MemoryManager {
        pub fn new(limit: usize) -> Self {
            Self { limit, used: 0 }
        }
        
        pub fn limit(&self) -> usize {
            self.limit
        }
        
        pub fn used(&self) -> usize {
            self.used
        }
        
        pub fn allocate(&mut self, size: usize) -> Result<(), String> {
            if self.used + size > self.limit {
                Err("Memory limit exceeded".to_string())
            } else {
                self.used += size;
                Ok(())
            }
        }
        
        pub fn deallocate(&mut self, size: usize) {
            self.used = self.used.saturating_sub(size);
        }
        
        pub fn stats(&self) -> MemoryStats {
            MemoryStats {
                used: self.used,
                limit: self.limit,
            }
        }
    }
    
    pub struct MemoryStats {
        pub used: usize,
        pub limit: usize,
    }
    
    impl MemoryStats {
        pub fn available(&self) -> usize {
            self.limit - self.used
        }
    }
    
    // Mock ErrorRecovery implementation
    pub struct ErrorRecovery {
        attempts: usize,
        max_attempts: usize,
    }
    
    impl ErrorRecovery {
        pub fn new() -> Self {
            Self {
                attempts: 0,
                max_attempts: 3,
            }
        }
        
        pub fn can_recover(&self) -> bool {
            self.attempts < self.max_attempts
        }
        
        pub fn attempt_count(&self) -> usize {
            self.attempts
        }
        
        pub fn attempt_recovery(&mut self) -> Result<(), String> {
            if self.can_recover() {
                self.attempts += 1;
                Ok(())
            } else {
                Err("Max recovery attempts exceeded".to_string())
            }
        }
        
        pub fn get_strategy_for_error(&self, error: &str) -> RecoveryStrategy {
            match error {
                e if e.contains("syntax") => RecoveryStrategy::ParsePartial,
                e if e.contains("undefined") => RecoveryStrategy::SkipStatement,
                _ => RecoveryStrategy::Reset,
            }
        }
        
        pub fn reset(&mut self) {
            self.attempts = 0;
        }
    }
    
    #[derive(Debug, PartialEq)]
    pub enum RecoveryStrategy {
        ParsePartial,
        SkipStatement,
        Reset,
    }
    
    // Mock ReplState implementation
    pub struct ReplState {
        variables: HashMap<String, String>,
        mode: ExecutionMode,
        recording: bool,
    }
    
    impl ReplState {
        pub fn new() -> Self {
            Self {
                variables: HashMap::new(),
                mode: ExecutionMode::Interactive,
                recording: false,
            }
        }
        
        pub fn execution_mode(&self) -> ExecutionMode {
            self.mode
        }
        
        pub fn set_execution_mode(&mut self, mode: ExecutionMode) {
            self.mode = mode;
        }
        
        pub fn variables(&self) -> &HashMap<String, String> {
            &self.variables
        }
        
        pub fn set_variable(&mut self, name: &str, value: &str) {
            self.variables.insert(name.to_string(), value.to_string());
        }
        
        pub fn get_variable(&self, name: &str) -> Option<&String> {
            self.variables.get(name)
        }
        
        pub fn is_recording(&self) -> bool {
            self.recording
        }
        
        pub fn start_recording(&mut self) {
            self.recording = true;
        }
        
        pub fn stop_recording(&mut self) {
            self.recording = false;
        }
        
        pub fn reset(&mut self) {
            self.variables.clear();
            self.mode = ExecutionMode::Interactive;
            self.recording = false;
        }
    }
    
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ExecutionMode {
        Interactive,
        Script,
        Debug,
    }
    
    // Mock History implementation
    pub struct History {
        entries: Vec<HistoryEntry>,
        capacity: usize,
        current: usize,
    }
    
    impl History {
        pub fn new(capacity: usize) -> Self {
            Self {
                entries: Vec::new(),
                capacity,
                current: 0,
            }
        }
        
        pub fn len(&self) -> usize {
            self.entries.len()
        }
        
        pub fn capacity(&self) -> usize {
            self.capacity
        }
        
        pub fn add(&mut self, command: &str) {
            if self.entries.len() >= self.capacity {
                self.entries.remove(0);
            }
            self.entries.push(HistoryEntry {
                command: command.to_string(),
                timestamp: std::time::SystemTime::now(),
            });
            self.current = self.entries.len();
        }
        
        pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
            self.entries.get(index)
        }
        
        pub fn search(&self, query: &str) -> Vec<&HistoryEntry> {
            self.entries
                .iter()
                .filter(|e| e.command.contains(query))
                .collect()
        }
        
        pub fn previous(&mut self) -> Option<&str> {
            if self.current > 0 {
                self.current -= 1;
                self.entries.get(self.current).map(|e| e.command.as_str())
            } else {
                None
            }
        }
        
        pub fn next(&mut self) -> Option<&str> {
            if self.current < self.entries.len() {
                let result = self.entries.get(self.current).map(|e| e.command.as_str());
                self.current += 1;
                result
            } else {
                None
            }
        }
        
        pub fn clear(&mut self) {
            self.entries.clear();
            self.current = 0;
        }
    }
    
    pub struct HistoryEntry {
        pub command: String,
        pub timestamp: std::time::SystemTime,
    }
    
    // Mock Command types
    #[derive(Debug, PartialEq)]
    pub enum Command {
        Help,
        Quit,
        Clear,
        Load(String),
        Custom(String),
    }
    
    pub struct CommandParser {
        custom_commands: HashMap<String, Command>,
    }
    
    impl CommandParser {
        pub fn new() -> Self {
            Self {
                custom_commands: HashMap::new(),
            }
        }
        
        pub fn parse(&self, input: &str) -> Option<Command> {
            if !input.starts_with(':') {
                return None;
            }
            
            let parts: Vec<&str> = input.split_whitespace().collect();
            match parts[0] {
                ":help" => Some(Command::Help),
                ":quit" => Some(Command::Quit),
                ":clear" => Some(Command::Clear),
                ":load" if parts.len() > 1 => Some(Command::Load(parts[1].to_string())),
                cmd => self.custom_commands.get(cmd).cloned(),
            }
        }
        
        pub fn register(&mut self, name: &str, command: Command) {
            self.custom_commands.insert(name.to_string(), command);
        }
        
        pub fn commands(&self) -> Vec<&str> {
            let mut cmds = vec![":help", ":quit", ":clear", ":load"];
            cmds.extend(self.custom_commands.keys().map(|s| s.as_str()));
            cmds
        }
    }
    
    pub struct CommandExecutor;
    
    impl CommandExecutor {
        pub fn new() -> Self {
            Self
        }
        
        pub fn execute(&self, command: Command) -> CommandResult {
            match command {
                Command::Help => CommandResult::Success("Help text".to_string()),
                Command::Clear => CommandResult::Success("Cleared".to_string()),
                Command::Load(file) if file.contains("nonexistent") => {
                    CommandResult::Error("File not found".to_string())
                }
                _ => CommandResult::Success("OK".to_string()),
            }
        }
    }
    
    pub enum CommandResult {
        Success(String),
        Error(String),
    }
}

// Import mock implementations for testing
use mock_impls::*;