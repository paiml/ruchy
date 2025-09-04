//! State management module for REPL
//! Handles REPL state, configuration, and session management

use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};


/// REPL execution mode
#[derive(Debug, Clone, PartialEq)]
pub enum ReplMode {
    /// Normal interactive mode
    Normal,
    /// Debug mode with extra output
    Debug,
    /// Test mode for automated testing
    Test,
    /// Script mode for batch execution
    Script,
}

impl ReplMode {
    /// Get display name (complexity: 1)
    pub fn as_str(&self) -> &str {
        match self {
            ReplMode::Normal => "normal",
            ReplMode::Debug => "debug",
            ReplMode::Test => "test",
            ReplMode::Script => "script",
        }
    }

    /// Parse from string (complexity: 2)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "normal" => Some(ReplMode::Normal),
            "debug" => Some(ReplMode::Debug),
            "test" => Some(ReplMode::Test),
            "script" => Some(ReplMode::Script),
            _ => None,
        }
    }
}

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Maximum recursion depth
    pub max_depth: usize,
    /// Maximum iterations in loops
    pub max_iterations: usize,
    /// Evaluation timeout
    pub timeout: Duration,
    /// Enable debug output
    pub debug_enabled: bool,
    /// Enable color output
    pub color_enabled: bool,
    /// Tab width for formatting
    pub tab_width: usize,
    /// Maximum display width
    pub max_width: usize,
    /// Prompt string
    pub prompt: String,
    /// Continuation prompt
    pub continuation_prompt: String,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            max_iterations: 100_000,
            timeout: Duration::from_secs(30),
            debug_enabled: false,
            color_enabled: true,
            tab_width: 4,
            max_width: 120,
            prompt: "ruchy> ".to_string(),
            continuation_prompt: "....> ".to_string(),
        }
    }
}

/// REPL session state
pub struct ReplState {
    /// Current mode
    mode: ReplMode,
    /// Configuration
    config: ReplConfig,
    /// Session metadata
    metadata: SessionMetadata,
    /// Feature flags
    features: HashMap<String, bool>,
    /// Statistics
    stats: ReplStatistics,
}

impl ReplState {
    /// Create new REPL state (complexity: 2)
    pub fn new() -> Self {
        Self {
            mode: ReplMode::Normal,
            config: ReplConfig::default(),
            metadata: SessionMetadata::new(),
            features: Self::default_features(),
            stats: ReplStatistics::new(),
        }
    }

    /// Create with custom config (complexity: 2)
    pub fn with_config(config: ReplConfig) -> Self {
        Self {
            mode: ReplMode::Normal,
            config,
            metadata: SessionMetadata::new(),
            features: Self::default_features(),
            stats: ReplStatistics::new(),
        }
    }

    /// Get current mode (complexity: 1)
    pub fn mode(&self) -> &ReplMode {
        &self.mode
    }

    /// Set mode (complexity: 2)
    pub fn set_mode(&mut self, mode: ReplMode) {
        self.mode = mode;
        self.stats.mode_changes += 1;
    }

    /// Get config (complexity: 1)
    pub fn config(&self) -> &ReplConfig {
        &self.config
    }

    /// Get mutable config (complexity: 1)
    pub fn config_mut(&mut self) -> &mut ReplConfig {
        &mut self.config
    }

    /// Update config (complexity: 2)
    pub fn update_config<F>(&mut self, updater: F) 
    where
        F: FnOnce(&mut ReplConfig),
    {
        updater(&mut self.config);
        self.stats.config_changes += 1;
    }

    /// Check if feature is enabled (complexity: 2)
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    /// Enable/disable feature (complexity: 3)
    pub fn set_feature(&mut self, feature: String, enabled: bool) {
        self.features.insert(feature, enabled);
    }

    /// Get session metadata (complexity: 1)
    pub fn metadata(&self) -> &SessionMetadata {
        &self.metadata
    }

    /// Update statistics (complexity: 2)
    pub fn record_evaluation(&mut self, success: bool, duration: Duration) {
        self.stats.total_evaluations += 1;
        if success {
            self.stats.successful_evaluations += 1;
        } else {
            self.stats.failed_evaluations += 1;
        }
        self.stats.total_eval_time += duration;
    }

    /// Get statistics (complexity: 1)
    pub fn stats(&self) -> &ReplStatistics {
        &self.stats
    }

    /// Reset statistics (complexity: 1)
    pub fn reset_stats(&mut self) {
        self.stats = ReplStatistics::new();
    }

    /// Create checkpoint (complexity: 3)
    pub fn checkpoint(&self) -> StateCheckpoint {
        StateCheckpoint {
            mode: self.mode.clone(),
            config: self.config.clone(),
            features: self.features.clone(),
            timestamp: Instant::now(),
        }
    }

    /// Restore from checkpoint (complexity: 2)
    pub fn restore(&mut self, checkpoint: StateCheckpoint) {
        self.mode = checkpoint.mode;
        self.config = checkpoint.config;
        self.features = checkpoint.features;
        self.stats.checkpoints_restored += 1;
    }

    /// Default feature flags (complexity: 2)
    fn default_features() -> HashMap<String, bool> {
        let mut features = HashMap::new();
        features.insert("async".to_string(), true);
        features.insert("macros".to_string(), true);
        features.insert("imports".to_string(), true);
        features.insert("dataframes".to_string(), true);
        features.insert("pattern_guards".to_string(), true);
        features.insert("string_interpolation".to_string(), true);
        features.insert("pipeline_operator".to_string(), true);
        features
    }

    /// Get feature list (complexity: 3)
    pub fn list_features(&self) -> Vec<(String, bool)> {
        let mut features: Vec<_> = self.features
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        features.sort_by(|a, b| a.0.cmp(&b.0));
        features
    }

    /// Export configuration (complexity: 4)
    pub fn export_config(&self) -> String {
        format!(
            "# REPL Configuration\n\
             mode = {}\n\
             max_depth = {}\n\
             max_iterations = {}\n\
             timeout_seconds = {}\n\
             debug_enabled = {}\n\
             color_enabled = {}\n\
             tab_width = {}\n\
             max_width = {}\n",
            self.mode.as_str(),
            self.config.max_depth,
            self.config.max_iterations,
            self.config.timeout.as_secs(),
            self.config.debug_enabled,
            self.config.color_enabled,
            self.config.tab_width,
            self.config.max_width
        )
    }

    /// Import configuration (complexity: 8)
    pub fn import_config(&mut self, config_str: &str) -> Result<()> {
        for line in config_str.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "mode" => {
                        if let Some(mode) = ReplMode::from_str(value) {
                            self.mode = mode;
                        }
                    }
                    "max_depth" => {
                        if let Ok(n) = value.parse() {
                            self.config.max_depth = n;
                        }
                    }
                    "max_iterations" => {
                        if let Ok(n) = value.parse() {
                            self.config.max_iterations = n;
                        }
                    }
                    "timeout_seconds" => {
                        if let Ok(n) = value.parse::<u64>() {
                            self.config.timeout = Duration::from_secs(n);
                        }
                    }
                    "debug_enabled" => {
                        self.config.debug_enabled = value == "true";
                    }
                    "color_enabled" => {
                        self.config.color_enabled = value == "true";
                    }
                    "tab_width" => {
                        if let Ok(n) = value.parse() {
                            self.config.tab_width = n;
                        }
                    }
                    "max_width" => {
                        if let Ok(n) = value.parse() {
                            self.config.max_width = n;
                        }
                    }
                    _ => {} // Ignore unknown keys
                }
            }
        }
        
        Ok(())
    }
}

/// Session metadata
#[derive(Debug, Clone)]
pub struct SessionMetadata {
    /// Session ID
    pub session_id: String,
    /// Start time
    pub start_time: Instant,
    /// Version
    pub version: String,
    /// Platform
    pub platform: String,
}

impl SessionMetadata {
    /// Create new metadata (complexity: 2)
    fn new() -> Self {
        Self {
            session_id: Self::generate_session_id(),
            start_time: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
        }
    }

    /// Generate session ID (complexity: 3)
    fn generate_session_id() -> String {
        use std::time::SystemTime;
        
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        
        format!("repl-{}-{}", timestamp, std::process::id())
    }

    /// Get session duration (complexity: 1)
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// REPL statistics
#[derive(Debug, Clone)]
pub struct ReplStatistics {
    /// Total evaluations
    pub total_evaluations: usize,
    /// Successful evaluations
    pub successful_evaluations: usize,
    /// Failed evaluations
    pub failed_evaluations: usize,
    /// Total evaluation time
    pub total_eval_time: Duration,
    /// Mode changes
    pub mode_changes: usize,
    /// Config changes
    pub config_changes: usize,
    /// Checkpoints restored
    pub checkpoints_restored: usize,
}

impl ReplStatistics {
    /// Create new statistics (complexity: 1)
    fn new() -> Self {
        Self {
            total_evaluations: 0,
            successful_evaluations: 0,
            failed_evaluations: 0,
            total_eval_time: Duration::ZERO,
            mode_changes: 0,
            config_changes: 0,
            checkpoints_restored: 0,
        }
    }

    /// Get success rate (complexity: 2)
    pub fn success_rate(&self) -> f64 {
        if self.total_evaluations == 0 {
            0.0
        } else {
            (self.successful_evaluations as f64) / (self.total_evaluations as f64) * 100.0
        }
    }

    /// Get average evaluation time (complexity: 2)
    pub fn average_eval_time(&self) -> Duration {
        if self.total_evaluations == 0 {
            Duration::ZERO
        } else {
            self.total_eval_time / self.total_evaluations as u32
        }
    }

    /// Format statistics (complexity: 3)
    pub fn format_display(&self) -> String {
        format!(
            "REPL Statistics:\n  \
             Total evaluations: {}\n  \
             Successful: {} ({:.1}%)\n  \
             Failed: {}\n  \
             Average eval time: {:?}\n  \
             Mode changes: {}\n  \
             Config changes: {}\n  \
             Checkpoints restored: {}",
            self.total_evaluations,
            self.successful_evaluations,
            self.success_rate(),
            self.failed_evaluations,
            self.average_eval_time(),
            self.mode_changes,
            self.config_changes,
            self.checkpoints_restored
        )
    }
}

/// State checkpoint for save/restore
#[derive(Debug, Clone)]
pub struct StateCheckpoint {
    /// Mode at checkpoint
    pub mode: ReplMode,
    /// Config at checkpoint
    pub config: ReplConfig,
    /// Features at checkpoint
    pub features: HashMap<String, bool>,
    /// Checkpoint timestamp
    pub timestamp: Instant,
}

impl StateCheckpoint {
    /// Get age of checkpoint (complexity: 1)
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_mode() {
        assert_eq!(ReplMode::Normal.as_str(), "normal");
        assert_eq!(ReplMode::from_str("debug"), Some(ReplMode::Debug));
        assert_eq!(ReplMode::from_str("invalid"), None);
    }

    #[test]
    fn test_repl_state_creation() {
        let state = ReplState::new();
        assert_eq!(state.mode(), &ReplMode::Normal);
        assert!(!state.config().debug_enabled);
    }

    #[test]
    fn test_mode_change() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Debug);
        assert_eq!(state.mode(), &ReplMode::Debug);
        assert_eq!(state.stats().mode_changes, 1);
    }

    #[test]
    fn test_feature_flags() {
        let mut state = ReplState::new();
        assert!(state.is_feature_enabled("async"));
        
        state.set_feature("custom".to_string(), true);
        assert!(state.is_feature_enabled("custom"));
    }

    #[test]
    fn test_statistics() {
        let mut state = ReplState::new();
        state.record_evaluation(true, Duration::from_millis(100));
        state.record_evaluation(false, Duration::from_millis(50));
        
        let stats = state.stats();
        assert_eq!(stats.total_evaluations, 2);
        assert_eq!(stats.successful_evaluations, 1);
        assert_eq!(stats.failed_evaluations, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }

    #[test]
    fn test_checkpoint_restore() {
        let mut state = ReplState::new();
        state.set_mode(ReplMode::Debug);
        
        let checkpoint = state.checkpoint();
        state.set_mode(ReplMode::Test);
        
        state.restore(checkpoint);
        assert_eq!(state.mode(), &ReplMode::Debug);
        assert_eq!(state.stats().checkpoints_restored, 1);
    }

    #[test]
    fn test_config_export_import() {
        let mut state = ReplState::new();
        state.config_mut().max_depth = 500;
        state.config_mut().debug_enabled = true;
        
        let exported = state.export_config();
        
        let mut new_state = ReplState::new();
        new_state.import_config(&exported).unwrap();
        
        assert_eq!(new_state.config().max_depth, 500);
        assert!(new_state.config().debug_enabled);
    }
}