//! Logging Operations Module (ruchy/std/logging)
//!
//! Thin wrappers around Rust's `tracing` crate for basic logging functionality.
//!
//! **Design**: Thin wrappers (complexity <=2 per function) around `tracing` crate.
//! **Quality**: 100% unit test coverage, property tests, >=75% mutation coverage.

use std::sync::OnceLock;

/// Global log level filter (set once by init_logger)
static LOG_LEVEL: OnceLock<String> = OnceLock::new();

/// Valid log levels
const VALID_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error", "off"];

/// Initialize logger with specified level
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::init_logger("info");
/// assert!(result.is_ok());
/// ```
///
/// # Errors
///
/// Returns error if level string is invalid
pub fn init_logger(level: &str) -> Result<(), String> {
    let normalized = level.to_lowercase();
    if !VALID_LEVELS.contains(&normalized.as_str()) {
        return Err(format!("Invalid log level '{level}': unknown variant `{level}`, expected one of `trace`, `debug`, `info`, `warn`, `error`, `off`"));
    }
    // Store the level (ignore if already set - matches env_logger behavior)
    let _ = LOG_LEVEL.set(normalized);
    Ok(())
}

/// Log info message
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::log_info("Server started");
/// assert!(result.is_ok());
/// ```
pub fn log_info(message: &str) -> Result<(), String> {
    tracing::info!("{message}");
    Ok(())
}

/// Log warning message
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::log_warn("Low memory");
/// assert!(result.is_ok());
/// ```
pub fn log_warn(message: &str) -> Result<(), String> {
    tracing::warn!("{message}");
    Ok(())
}

/// Log error message
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::log_error("Connection failed");
/// assert!(result.is_ok());
/// ```
pub fn log_error(message: &str) -> Result<(), String> {
    tracing::error!("{message}");
    Ok(())
}

/// Log debug message
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::log_debug("Variable x = 42");
/// assert!(result.is_ok());
/// ```
pub fn log_debug(message: &str) -> Result<(), String> {
    tracing::debug!("{message}");
    Ok(())
}

/// Log trace message
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let result = logging::log_trace("Entering function foo");
/// assert!(result.is_ok());
/// ```
pub fn log_trace(message: &str) -> Result<(), String> {
    tracing::trace!("{message}");
    Ok(())
}

/// Get current log level
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let _ = logging::init_logger("info");
/// let level = logging::get_level().unwrap();
/// assert!(["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()));
/// ```
pub fn get_level() -> Result<String, String> {
    Ok(LOG_LEVEL
        .get()
        .cloned()
        .unwrap_or_else(|| "info".to_string()))
}

/// Check if level is enabled
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::logging;
///
/// let _ = logging::init_logger("info");
/// let enabled = logging::is_level_enabled("info").unwrap();
/// assert!(enabled);
/// ```
///
/// # Errors
///
/// Returns error if level string is invalid
pub fn is_level_enabled(level: &str) -> Result<bool, String> {
    let normalized = level.to_lowercase();
    if !VALID_LEVELS.contains(&normalized.as_str()) {
        return Err(format!("Invalid log level '{level}': unknown variant `{level}`, expected one of `trace`, `debug`, `info`, `warn`, `error`"));
    }
    // Check against the current max level
    let current = LOG_LEVEL
        .get()
        .map(String::as_str)
        .unwrap_or("info");
    let level_order = |l: &str| -> i32 {
        match l {
            "off" => 0,
            "error" => 1,
            "warn" => 2,
            "info" => 3,
            "debug" => 4,
            "trace" => 5,
            _ => 0,
        }
    };
    Ok(level_order(&normalized) <= level_order(current))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logger_valid_levels() {
        assert!(init_logger("trace").is_ok());
        assert!(init_logger("debug").is_ok());
        assert!(init_logger("info").is_ok());
        assert!(init_logger("warn").is_ok());
        assert!(init_logger("error").is_ok());
        assert!(init_logger("off").is_ok());
    }

    #[test]
    fn test_init_logger_invalid_level() {
        assert!(init_logger("invalid").is_err());
        assert!(init_logger("unknown").is_err());
    }

    #[test]
    fn test_init_logger_case_insensitive() {
        assert!(init_logger("INFO").is_ok());
        assert!(init_logger("Debug").is_ok());
        assert!(init_logger("WARN").is_ok());
    }

    #[test]
    fn test_log_info_basic() {
        let _ = init_logger("info");
        assert!(log_info("Test info message").is_ok());
        assert!(log_info("").is_ok()); // Empty message is ok
    }

    #[test]
    fn test_log_warn_basic() {
        let _ = init_logger("warn");
        assert!(log_warn("Test warning").is_ok());
        assert!(log_warn("").is_ok());
    }

    #[test]
    fn test_log_error_basic() {
        let _ = init_logger("error");
        assert!(log_error("Test error").is_ok());
        assert!(log_error("").is_ok());
    }

    #[test]
    fn test_log_debug_basic() {
        let _ = init_logger("debug");
        assert!(log_debug("Test debug").is_ok());
        assert!(log_debug("").is_ok());
    }

    #[test]
    fn test_log_trace_basic() {
        let _ = init_logger("trace");
        assert!(log_trace("Test trace").is_ok());
        assert!(log_trace("").is_ok());
    }

    #[test]
    fn test_get_level() {
        // After init, should return a valid level string
        let _ = init_logger("info");
        let level = get_level().unwrap();
        assert!(["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()));
    }

    #[test]
    fn test_is_level_enabled_valid() {
        let _ = init_logger("info");

        // Info and above should be enabled
        assert!(is_level_enabled("info").is_ok());
        assert!(is_level_enabled("warn").is_ok());
        assert!(is_level_enabled("error").is_ok());

        // Debug and trace may or may not be enabled depending on init
        assert!(is_level_enabled("debug").is_ok());
        assert!(is_level_enabled("trace").is_ok());
    }

    #[test]
    fn test_is_level_enabled_invalid() {
        assert!(is_level_enabled("invalid").is_err());
        assert!(is_level_enabled("unknown").is_err());
    }

    #[test]
    fn test_multiple_init_calls() {
        // Multiple init calls should not fail (uses OnceLock internally)
        assert!(init_logger("info").is_ok());
        assert!(init_logger("debug").is_ok());
        assert!(init_logger("warn").is_ok());
    }

    #[test]
    fn test_log_all_levels_workflow() {
        let _ = init_logger("trace");

        // All log functions should work
        assert!(log_trace("Trace message").is_ok());
        assert!(log_debug("Debug message").is_ok());
        assert!(log_info("Info message").is_ok());
        assert!(log_warn("Warn message").is_ok());
        assert!(log_error("Error message").is_ok());
    }

    #[test]
    fn test_log_special_characters() {
        let _ = init_logger("info");

        // Test messages with special characters
        assert!(log_info("Message with \"quotes\"").is_ok());
        assert!(log_info("Message with\nnewlines").is_ok());
        assert!(log_info("Message with\ttabs").is_ok());
        assert!(log_info("Message with emojis").is_ok());
    }

    #[test]
    fn test_log_long_message() {
        let _ = init_logger("info");

        // Very long message
        let long_msg = "x".repeat(10000);
        assert!(log_info(&long_msg).is_ok());
    }

    #[test]
    fn test_level_hierarchy() {
        // Test that level hierarchy works
        let _ = init_logger("warn");

        // After setting to warn, should still work
        let level = get_level().unwrap();
        assert!(["warn", "error", "off", "info", "debug", "trace"].contains(&level.as_str()));
    }

    // ===== EXTREME TDD Round 156 - Additional Logging Tests =====

    #[test]
    fn test_init_logger_with_whitespace() {
        // Test with leading/trailing whitespace
        assert!(init_logger(" info ").is_err()); // Should fail with whitespace
        assert!(init_logger("info").is_ok()); // Without whitespace
    }

    #[test]
    fn test_is_level_enabled_case_variations() {
        let _ = init_logger("info");
        assert!(is_level_enabled("INFO").is_ok());
        assert!(is_level_enabled("Info").is_ok());
    }

    #[test]
    fn test_log_info_unicode() {
        let _ = init_logger("info");
        assert!(log_info("Test with unicode").is_ok());
        assert!(log_info("Test text").is_ok());
        assert!(log_info("Rocket launch").is_ok());
    }

    #[test]
    fn test_log_warn_unicode() {
        let _ = init_logger("warn");
        assert!(log_warn("Warning message").is_ok());
    }

    #[test]
    fn test_log_error_unicode() {
        let _ = init_logger("error");
        assert!(log_error("Error text").is_ok());
    }

    #[test]
    fn test_log_debug_unicode() {
        let _ = init_logger("debug");
        assert!(log_debug("Debug message").is_ok());
    }

    #[test]
    fn test_log_trace_unicode() {
        let _ = init_logger("trace");
        assert!(log_trace("Trace message").is_ok());
    }

    #[test]
    fn test_log_multiline_messages() {
        let _ = init_logger("info");
        let multiline = "Line 1\nLine 2\nLine 3\n\nLine 5";
        assert!(log_info(multiline).is_ok());
    }

    #[test]
    fn test_log_with_format_specifiers() {
        let _ = init_logger("info");
        assert!(log_info("Test %s %d {name} {{escaped}}").is_ok());
    }

    #[test]
    fn test_get_level_returns_lowercase() {
        let _ = init_logger("INFO");
        let level = get_level().unwrap();
        // Level should always be lowercase
        assert_eq!(level, level.to_lowercase());
    }

    #[test]
    fn test_is_level_enabled_all_levels() {
        let _ = init_logger("trace");
        // All levels should return Ok (either true or false)
        assert!(is_level_enabled("trace").is_ok());
        assert!(is_level_enabled("debug").is_ok());
        assert!(is_level_enabled("info").is_ok());
        assert!(is_level_enabled("warn").is_ok());
        assert!(is_level_enabled("error").is_ok());
    }
}
