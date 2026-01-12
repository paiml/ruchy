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

    // === Additional tests for improved coverage ===

    #[test]
    fn test_repl_config_debug_trait() {
        let config = ReplConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("ReplConfig"));
        assert!(debug_str.contains("max_memory"));
        assert!(debug_str.contains("timeout"));
        assert!(debug_str.contains("maxdepth"));
        assert!(debug_str.contains("debug"));
    }

    #[test]
    fn test_repl_config_custom_max_memory() {
        let config = ReplConfig {
            max_memory: 512 * 1024, // 512KB
            ..Default::default()
        };
        assert_eq!(config.max_memory, 512 * 1024);
        // Other fields should be defaults
        assert_eq!(config.timeout, Duration::from_millis(5000));
        assert_eq!(config.maxdepth, 100);
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_custom_timeout() {
        let config = ReplConfig {
            timeout: Duration::from_secs(30),
            ..Default::default()
        };
        assert_eq!(config.timeout, Duration::from_secs(30));
        // Other fields should be defaults
        assert_eq!(config.max_memory, 1024 * 1024);
        assert_eq!(config.maxdepth, 100);
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_custom_maxdepth() {
        let config = ReplConfig {
            maxdepth: 200,
            ..Default::default()
        };
        assert_eq!(config.maxdepth, 200);
        // Other fields should be defaults
        assert_eq!(config.max_memory, 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_millis(5000));
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_debug_enabled() {
        let config = ReplConfig {
            debug: true,
            ..Default::default()
        };
        assert!(config.debug);
        // Other fields should be defaults
        assert_eq!(config.max_memory, 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_millis(5000));
        assert_eq!(config.maxdepth, 100);
    }

    #[test]
    fn test_repl_config_sandbox_settings() {
        // Simulate sandbox configuration
        let config = ReplConfig {
            max_memory: 256 * 1024,              // 256KB limit
            timeout: Duration::from_millis(500), // 500ms timeout
            maxdepth: 25,                        // Low recursion
            debug: false,
        };
        assert_eq!(config.max_memory, 256 * 1024);
        assert_eq!(config.timeout, Duration::from_millis(500));
        assert_eq!(config.maxdepth, 25);
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_production_settings() {
        // Simulate production configuration
        let config = ReplConfig {
            max_memory: 10 * 1024 * 1024,     // 10MB limit
            timeout: Duration::from_secs(60), // 60s timeout
            maxdepth: 500,                    // Higher recursion
            debug: false,
        };
        assert_eq!(config.max_memory, 10 * 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.maxdepth, 500);
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_debug_development_settings() {
        // Simulate development configuration with debug enabled
        let config = ReplConfig {
            max_memory: 2 * 1024 * 1024,       // 2MB limit
            timeout: Duration::from_secs(120), // 2 minute timeout for debugging
            maxdepth: 100,
            debug: true,
        };
        assert_eq!(config.max_memory, 2 * 1024 * 1024);
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.maxdepth, 100);
        assert!(config.debug);
    }

    #[test]
    fn test_repl_config_timeout_zero() {
        let config = ReplConfig {
            timeout: Duration::ZERO,
            ..Default::default()
        };
        assert_eq!(config.timeout, Duration::ZERO);
    }

    #[test]
    fn test_repl_config_maxdepth_zero() {
        let config = ReplConfig {
            maxdepth: 0,
            ..Default::default()
        };
        assert_eq!(config.maxdepth, 0);
    }

    #[test]
    fn test_repl_config_max_memory_zero() {
        let config = ReplConfig {
            max_memory: 0,
            ..Default::default()
        };
        assert_eq!(config.max_memory, 0);
    }

    #[test]
    fn test_repl_config_default_values_are_reasonable() {
        let config = ReplConfig::default();
        // 1MB is reasonable for REPL usage
        assert!(config.max_memory >= 1024 * 1024);
        // 5 seconds is reasonable timeout for simple evaluations
        assert!(config.timeout >= Duration::from_millis(1000));
        // 100 recursion depth prevents most stack overflows
        assert!(config.maxdepth >= 50);
        // Debug should be off by default
        assert!(!config.debug);
    }

    #[test]
    fn test_repl_config_clone_independence() {
        let mut config = ReplConfig::default();
        let cloned = config.clone();

        // Modify original
        config.max_memory = 999;
        config.debug = true;

        // Clone should be unaffected (original changes don't affect clone)
        assert_eq!(cloned.max_memory, 1024 * 1024);
        assert!(!cloned.debug);

        // Verify original was modified (prevents unused_assignments warning)
        assert_eq!(config.max_memory, 999);
        assert!(config.debug);
    }

    #[test]
    fn test_repl_config_timeout_conversions() {
        let config = ReplConfig {
            timeout: Duration::from_millis(5000),
            ..Default::default()
        };
        assert_eq!(config.timeout.as_secs(), 5);
        assert_eq!(config.timeout.as_millis(), 5000);
    }

    #[test]
    fn test_repl_config_large_values() {
        let config = ReplConfig {
            max_memory: usize::MAX,
            timeout: Duration::MAX,
            maxdepth: usize::MAX,
            debug: true,
        };
        assert_eq!(config.max_memory, usize::MAX);
        assert_eq!(config.timeout, Duration::MAX);
        assert_eq!(config.maxdepth, usize::MAX);
    }
}
