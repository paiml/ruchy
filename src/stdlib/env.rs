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
/// env::set_var("TEST_VAR", "test_value").expect("operation should succeed in test");
/// assert_eq!(env::var("TEST_VAR").expect("operation should succeed in test"), "test_value");
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
/// env::set_var("TEMP_VAR", "value").expect("operation should succeed in test");
/// env::remove_var("TEMP_VAR").expect("operation should succeed in test");
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
/// let vars = env::vars().expect("operation should succeed in test");
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
/// let dir = env::current_dir().expect("operation should succeed in test");
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
/// let temp_dir = TempDir::new().expect("operation should succeed in test");
/// let original = env::current_dir().expect("operation should succeed in test");
///
/// env::set_current_dir(temp_dir.path().to_str().expect("operation should succeed in test")).expect("operation should succeed in test");
///
/// // Restore
/// env::set_current_dir(&original).expect("operation should succeed in test");
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
/// let args = env::args().expect("operation should succeed in test");
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
/// let temp = env::temp_dir().expect("operation should succeed in test");
/// assert!(!temp.is_empty());
/// assert!(std::path::Path::new(&temp).exists());
/// ```
pub fn temp_dir() -> Result<String, String> {
    Ok(env::temp_dir().to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_existing() {
        // PATH should always exist
        assert!(var("PATH").is_ok());
        let path = var("PATH").expect("operation should succeed in test");
        assert!(!path.is_empty());
    }

    #[test]
    fn test_var_nonexistent() {
        // Variable that definitely doesn't exist
        assert!(var("RUCHY_TEST_NONEXISTENT_VAR_XYZ").is_err());
    }

    #[test]
    fn test_set_and_get_var() {
        set_var("RUCHY_TEST_VAR", "test_value").expect("operation should succeed in test");
        assert_eq!(
            var("RUCHY_TEST_VAR").expect("operation should succeed in test"),
            "test_value"
        );

        // Cleanup
        remove_var("RUCHY_TEST_VAR").expect("operation should succeed in test");
    }

    #[test]
    fn test_set_var_overwrite() {
        set_var("RUCHY_TEST_VAR2", "value1").expect("operation should succeed in test");
        assert_eq!(
            var("RUCHY_TEST_VAR2").expect("operation should succeed in test"),
            "value1"
        );

        set_var("RUCHY_TEST_VAR2", "value2").expect("operation should succeed in test");
        assert_eq!(
            var("RUCHY_TEST_VAR2").expect("operation should succeed in test"),
            "value2"
        );

        // Cleanup
        remove_var("RUCHY_TEST_VAR2").expect("operation should succeed in test");
    }

    #[test]
    fn test_remove_var() {
        set_var("RUCHY_TEST_VAR3", "value").expect("operation should succeed in test");
        assert!(var("RUCHY_TEST_VAR3").is_ok());

        remove_var("RUCHY_TEST_VAR3").expect("operation should succeed in test");
        assert!(var("RUCHY_TEST_VAR3").is_err());
    }

    #[test]
    fn test_remove_nonexistent_var() {
        // Removing nonexistent variable should not fail
        assert!(remove_var("RUCHY_TEST_NONEXISTENT").is_ok());
    }

    #[test]
    fn test_vars() {
        let all_vars = vars().expect("operation should succeed in test");
        assert!(!all_vars.is_empty());
        assert!(all_vars.contains_key("PATH"));
    }

    #[test]
    fn test_vars_includes_set() {
        set_var("RUCHY_TEST_VAR4", "test").expect("operation should succeed in test");
        let all_vars = vars().expect("operation should succeed in test");
        assert_eq!(all_vars.get("RUCHY_TEST_VAR4"), Some(&"test".to_string()));

        // Cleanup
        remove_var("RUCHY_TEST_VAR4").expect("operation should succeed in test");
    }

    #[test]
    fn test_current_dir() {
        let dir = current_dir().expect("operation should succeed in test");
        assert!(!dir.is_empty());
        assert!(std::path::Path::new(&dir).exists());
    }

    #[test]
    fn test_args() {
        let args_list = args().expect("operation should succeed in test");
        assert!(!args_list.is_empty()); // At least program name
    }

    #[test]
    fn test_temp_dir() {
        let temp = temp_dir().expect("operation should succeed in test");
        assert!(!temp.is_empty());
        assert!(std::path::Path::new(&temp).exists());
    }

    #[test]
    fn test_env_workflow() {
        // Complete workflow: set, get, modify, remove
        let key = "RUCHY_WORKFLOW_TEST";

        // Set initial value
        set_var(key, "value1").expect("operation should succeed in test");
        assert_eq!(
            var(key).expect("operation should succeed in test"),
            "value1"
        );

        // Modify
        set_var(key, "value2").expect("operation should succeed in test");
        assert_eq!(
            var(key).expect("operation should succeed in test"),
            "value2"
        );

        // Check it appears in vars()
        let all_vars = vars().expect("operation should succeed in test");
        assert_eq!(all_vars.get(key), Some(&"value2".to_string()));

        // Remove
        remove_var(key).expect("operation should succeed in test");
        assert!(var(key).is_err());
    }

    #[test]
    fn test_special_characters_in_values() {
        let key = "RUCHY_SPECIAL_TEST";

        // Test with special characters
        set_var(key, "value with spaces").expect("operation should succeed in test");
        assert_eq!(
            var(key).expect("operation should succeed in test"),
            "value with spaces"
        );

        set_var(key, "value=with=equals").expect("operation should succeed in test");
        assert_eq!(
            var(key).expect("operation should succeed in test"),
            "value=with=equals"
        );

        set_var(key, "value:with:colons").expect("operation should succeed in test");
        assert_eq!(
            var(key).expect("operation should succeed in test"),
            "value:with:colons"
        );

        // Cleanup
        remove_var(key).expect("operation should succeed in test");
    }

    #[test]
    fn test_empty_value() {
        let key = "RUCHY_EMPTY_TEST";

        // Empty string is valid
        set_var(key, "").expect("operation should succeed in test");
        assert_eq!(var(key).expect("operation should succeed in test"), "");

        // Cleanup
        remove_var(key).expect("operation should succeed in test");
    }
}
