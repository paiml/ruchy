//! Logging Operations Module (ruchy/std/logging)
//!
//! Thin wrappers around Rust's `log` crate for basic logging functionality.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around `log` crate.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use log::{Level, LevelFilter};
use std::str::FromStr;

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
    let level_filter =
        LevelFilter::from_str(level).map_err(|e| format!("Invalid log level '{level}': {e}"))?;

    // Use try_init() to handle case where logger already initialized
    env_logger::Builder::from_default_env()
        .filter_level(level_filter)
        .try_init()
        .ok(); // Ignore error if already initialized

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
    log::info!("{message}");
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
    log::warn!("{message}");
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
    log::error!("{message}");
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
    log::debug!("{message}");
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
    log::trace!("{message}");
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
    let max_level = log::max_level();
    Ok(max_level.to_string().to_lowercase())
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
    let log_level =
        Level::from_str(level).map_err(|e| format!("Invalid log level '{level}': {e}"))?;
    Ok(log::log_enabled!(log_level))
}
