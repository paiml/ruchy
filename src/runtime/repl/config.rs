//! REPL Configuration
//!
//! Configuration options for the Ruchy REPL.

use std::time::Duration;

/// REPL configuration
#[derive(Debug, Clone)]
pub struct ReplConfig {
    /// Maximum memory limit in bytes
    pub max_memory: usize,
    /// Execution timeout
    pub timeout: Duration,
    /// Maximum recursion depth
    pub maxdepth: usize,
    /// Debug mode flag
    pub debug: bool,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024,              // 1MB
            timeout: Duration::from_millis(5000), // 5 seconds
            maxdepth: 100,
            debug: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_default_config() {
        let config = ReplConfig::default();
        assert_eq!(config.max_memory, 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_millis(5000));
        assert_eq!(config.maxdepth, 100);
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_clone() {
        let config = ReplConfig {
            max_memory: 2048,
            timeout: Duration::from_secs(10),
            maxdepth: 50,
            debug: true,
        };
        let cloned = config.clone();
        assert_eq!(cloned.max_memory, 2048);
        assert_eq!(cloned.timeout, Duration::from_secs(10));
        assert_eq!(cloned.maxdepth, 50);
        assert!(cloned.debug);
    }
}
