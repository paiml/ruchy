//! Environment Operations Module (ruchy/std/env)
//!
//! Thin wrappers around Rust's `std::env` for environment variable and system operations.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around proven Rust `std::env`.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use std::collections::HashMap;
use std::env;

/// Get environment variable value
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// // Get PATH variable
/// if let Ok(path) = env::var("PATH") {
///     assert!(!path.is_empty());
/// }
/// ```
pub fn var(key: &str) -> Result<String, String> {
    env::var(key).map_err(|e| e.to_string())
}

/// Set environment variable
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// env::set_var("TEST_VAR", "test_value").unwrap();
/// assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
/// ```
pub fn set_var(key: &str, value: &str) -> Result<(), String> {
    env::set_var(key, value);
    Ok(())
}

/// Remove environment variable
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// env::set_var("TEMP_VAR", "value").unwrap();
/// env::remove_var("TEMP_VAR").unwrap();
/// assert!(env::var("TEMP_VAR").is_err());
/// ```
pub fn remove_var(key: &str) -> Result<(), String> {
    env::remove_var(key);
    Ok(())
}

/// Get all environment variables as `HashMap`
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// let vars = env::vars().unwrap();
/// assert!(!vars.is_empty());
/// assert!(vars.contains_key("PATH"));
/// ```
pub fn vars() -> Result<HashMap<String, String>, String> {
    Ok(env::vars().collect())
}

/// Get current working directory
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// let dir = env::current_dir().unwrap();
/// assert!(!dir.is_empty());
/// ```
pub fn current_dir() -> Result<String, String> {
    env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// Set current working directory
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
/// use tempfile::TempDir;
///
/// let temp_dir = TempDir::new().unwrap();
/// let original = env::current_dir().unwrap();
///
/// env::set_current_dir(temp_dir.path().to_str().unwrap()).unwrap();
///
/// // Restore
/// env::set_current_dir(&original).unwrap();
/// ```
pub fn set_current_dir(path: &str) -> Result<(), String> {
    env::set_current_dir(path).map_err(|e| e.to_string())
}

/// Get command line arguments
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// let args = env::args().unwrap();
/// assert!(!args.is_empty()); // At least program name
/// ```
pub fn args() -> Result<Vec<String>, String> {
    Ok(env::args().collect())
}

/// Get temporary directory path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::env;
///
/// let temp = env::temp_dir().unwrap();
/// assert!(!temp.is_empty());
/// assert!(std::path::Path::new(&temp).exists());
/// ```
pub fn temp_dir() -> Result<String, String> {
    Ok(env::temp_dir().to_string_lossy().to_string())
}
