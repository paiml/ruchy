//! Integration tests for `stdlib::env` module
//!
//! Target: 0% → 100% coverage for stdlib/env.rs (43 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests from tests/ directory ARE tracked by llvm-cov.

use ruchy::stdlib::env;

#[test]
fn test_env_var_get_existing() {
    // PATH should exist on all systems
    let result = env::var("PATH");
    assert!(result.is_ok(), "PATH variable should exist");
    assert!(!result.unwrap().is_empty(), "PATH should not be empty");
}

#[test]
fn test_env_var_get_nonexistent() {
    let result = env::var("RUCHY_TEST_NONEXISTENT_VARIABLE_XYZ_999");
    assert!(result.is_err(), "Nonexistent variable should return error");
}

#[test]
fn test_env_set_and_get_var() {
    // Set a test variable
    let key = "RUCHY_INTEGRATION_TEST_VAR";
    let value = "integration_test_value";

    assert!(env::set_var(key, value).is_ok());

    // Verify it was set
    let result = env::var(key);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), value);

    // Cleanup
    env::remove_var(key).unwrap();
}

#[test]
fn test_env_set_var_overwrite() {
    let key = "RUCHY_INTEGRATION_OVERWRITE_TEST";

    // Set initial value
    env::set_var(key, "value1").unwrap();
    assert_eq!(env::var(key).unwrap(), "value1");

    // Overwrite with new value
    env::set_var(key, "value2").unwrap();
    assert_eq!(env::var(key).unwrap(), "value2");

    // Cleanup
    env::remove_var(key).unwrap();
}

#[test]
fn test_env_remove_var() {
    let key = "RUCHY_INTEGRATION_REMOVE_TEST";

    // Set variable
    env::set_var(key, "test_value").unwrap();
    assert!(env::var(key).is_ok());

    // Remove variable
    env::remove_var(key).unwrap();
    assert!(
        env::var(key).is_err(),
        "Variable should not exist after removal"
    );
}

#[test]
fn test_env_remove_nonexistent_var() {
    // Removing nonexistent variable should succeed (no-op)
    let result = env::remove_var("RUCHY_INTEGRATION_NONEXISTENT_999");
    assert!(result.is_ok(), "Removing nonexistent var should not fail");
}

#[test]
fn test_env_vars_returns_hashmap() {
    let vars = env::vars().unwrap();

    // Should be a HashMap
    assert!(
        !vars.is_empty(),
        "Environment should have at least some variables"
    );

    // PATH should be in the environment
    assert!(
        vars.contains_key("PATH"),
        "PATH should be in environment variables"
    );
}

#[test]
fn test_env_vars_includes_set_variable() {
    let key = "RUCHY_INTEGRATION_VARS_TEST";
    let value = "vars_test_value";

    // Set a variable
    env::set_var(key, value).unwrap();

    // Get all variables
    let vars = env::vars().unwrap();

    // Verify our variable is in the map
    assert_eq!(vars.get(key), Some(&value.to_string()));

    // Cleanup
    env::remove_var(key).unwrap();
}

#[test]
fn test_env_current_dir() {
    let dir = env::current_dir().unwrap();

    // Should return a non-empty path
    assert!(!dir.is_empty(), "Current directory should not be empty");

    // Directory should exist
    assert!(
        std::path::Path::new(&dir).exists(),
        "Current directory should exist"
    );
}

#[test]
fn test_env_set_current_dir() {
    use tempfile::TempDir;

    // Save original directory
    let original = env::current_dir().unwrap();

    // Create temp directory
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().to_str().unwrap();

    // Change to temp directory
    assert!(env::set_current_dir(temp_path).is_ok());

    // Verify we changed
    let new_dir = env::current_dir().unwrap();
    assert!(
        new_dir.contains(temp_path)
            || std::path::Path::new(&new_dir).canonicalize().unwrap()
                == std::path::Path::new(temp_path).canonicalize().unwrap()
    );

    // Restore original directory
    env::set_current_dir(&original).unwrap();
}

#[test]
fn test_env_args() {
    let args = env::args().unwrap();

    // Should have at least the program name
    assert!(
        !args.is_empty(),
        "Args should contain at least program name"
    );

    // First arg is typically the program path/name
    assert!(!args[0].is_empty(), "Program name should not be empty");
}

#[test]
fn test_env_temp_dir() {
    let temp = env::temp_dir().unwrap();

    // Should return a non-empty path
    assert!(!temp.is_empty(), "Temp directory path should not be empty");

    // Temp directory should exist
    assert!(
        std::path::Path::new(&temp).exists(),
        "Temp directory should exist"
    );
}

#[test]
fn test_env_workflow_complete() {
    // Complete workflow: set, verify, modify, check in vars(), remove, verify removal
    let key = "RUCHY_INTEGRATION_WORKFLOW_TEST";

    // Set initial value
    env::set_var(key, "workflow_value_1").unwrap();
    assert_eq!(env::var(key).unwrap(), "workflow_value_1");

    // Modify value
    env::set_var(key, "workflow_value_2").unwrap();
    assert_eq!(env::var(key).unwrap(), "workflow_value_2");

    // Verify it appears in vars()
    let all_vars = env::vars().unwrap();
    assert_eq!(all_vars.get(key), Some(&"workflow_value_2".to_string()));

    // Remove variable
    env::remove_var(key).unwrap();

    // Verify removal
    assert!(env::var(key).is_err());
    let all_vars_after = env::vars().unwrap();
    assert!(!all_vars_after.contains_key(key));
}

#[test]
fn test_env_special_characters_in_values() {
    let key = "RUCHY_INTEGRATION_SPECIAL_CHARS";

    // Test with spaces
    env::set_var(key, "value with spaces").unwrap();
    assert_eq!(env::var(key).unwrap(), "value with spaces");

    // Test with equals signs
    env::set_var(key, "value=with=equals").unwrap();
    assert_eq!(env::var(key).unwrap(), "value=with=equals");

    // Test with colons
    env::set_var(key, "value:with:colons").unwrap();
    assert_eq!(env::var(key).unwrap(), "value:with:colons");

    // Test with empty string
    env::set_var(key, "").unwrap();
    assert_eq!(env::var(key).unwrap(), "");

    // Cleanup
    env::remove_var(key).unwrap();
}
