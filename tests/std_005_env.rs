//! STD-005: Environment Module Tests (ruchy/std/env)
//!
//! Test suite for environment operations module.
//! Thin wrappers around Rust's `std::env` with Ruchy-friendly API.
//!

#![allow(clippy::ignore_without_reason)] // Test file with known limitations
#![allow(missing_docs)]
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use std::env;
use tempfile::TempDir;

/// Helper to create unique test environment variable names
fn test_var_name(suffix: &str) -> String {
    format!("RUCHY_TEST_{suffix}")
}

#[test]
fn test_std_005_get_env_var() {
    // STD-005: Test getting environment variable

    let var_name = test_var_name("GET");
    env::set_var(&var_name, "test_value");

    // Call ruchy::stdlib::env::var
    let result = ruchy::stdlib::env::var(&var_name);

    assert!(result.is_ok(), "var should succeed for existing variable");
    let value = result.unwrap();
    assert_eq!(value, "test_value", "Value must match exactly");
    assert_eq!(value.len(), 10, "Value length must be 10");
    assert!(value.contains("test"), "Value must contain 'test'");
    assert!(!value.is_empty(), "Value must not be empty");

    // Cleanup
    env::remove_var(&var_name);
}

#[test]
fn test_std_005_get_env_var_missing() {
    // STD-005: Test getting missing environment variable returns error

    let var_name = test_var_name("MISSING");
    env::remove_var(&var_name); // Ensure it doesn't exist

    let result = ruchy::stdlib::env::var(&var_name);

    assert!(
        result.is_err(),
        "var should return error for missing variable"
    );
}

#[test]
fn test_std_005_set_env_var() {
    // STD-005: Test setting environment variable

    let var_name = test_var_name("SET");

    // Call ruchy::stdlib::env::set_var
    let result = ruchy::stdlib::env::set_var(&var_name, "new_value");

    assert!(result.is_ok(), "set_var should succeed");

    // Verify it was set
    let value = env::var(&var_name).expect("Variable should exist");
    assert_eq!(value, "new_value", "Value must match what was set");
    assert_eq!(value.len(), 9, "Value length must be 9");
    assert!(value.contains("new"), "Value must contain 'new'");
    assert!(!value.is_empty(), "Value must not be empty");

    // Cleanup
    env::remove_var(&var_name);
}

#[test]
fn test_std_005_set_env_var_overwrites() {
    // STD-005: Test setting environment variable overwrites existing value

    let var_name = test_var_name("OVERWRITE");
    env::set_var(&var_name, "old_value");

    let result = ruchy::stdlib::env::set_var(&var_name, "new_value");

    assert!(result.is_ok(), "set_var should succeed");
    let value = env::var(&var_name).expect("Variable should exist");
    assert_eq!(value, "new_value", "Must have new value");
    assert_ne!(value, "old_value", "Must not have old value");
    assert!(!value.contains("old"), "Must not contain 'old'");
    assert!(value.contains("new"), "Must contain 'new'");

    // Cleanup
    env::remove_var(&var_name);
}

#[test]
fn test_std_005_remove_env_var() {
    // STD-005: Test removing environment variable

    let var_name = test_var_name("REMOVE");
    env::set_var(&var_name, "to_remove");

    // Call ruchy::stdlib::env::remove_var
    let result = ruchy::stdlib::env::remove_var(&var_name);

    assert!(result.is_ok(), "remove_var should succeed");

    // Verify it was removed
    let result = env::var(&var_name);
    assert!(result.is_err(), "Variable should no longer exist");
}

#[test]
fn test_std_005_remove_env_var_nonexistent() {
    // STD-005: Test removing nonexistent variable (should succeed - idempotent)

    let var_name = test_var_name("NONEXISTENT");
    env::remove_var(&var_name); // Ensure it doesn't exist

    let result = ruchy::stdlib::env::remove_var(&var_name);

    assert!(
        result.is_ok(),
        "remove_var should succeed even if variable doesn't exist"
    );
}

#[test]
fn test_std_005_get_all_vars() {
    // STD-005: Test getting all environment variables

    let var1 = test_var_name("ALL1");
    let var2 = test_var_name("ALL2");
    env::set_var(&var1, "value1");
    env::set_var(&var2, "value2");

    // Call ruchy::stdlib::env::vars
    let result = ruchy::stdlib::env::vars();

    assert!(result.is_ok(), "vars should succeed");
    let vars = result.unwrap();

    assert!(!vars.is_empty(), "vars must not be empty");
    assert!(vars.len() > 2, "vars must contain multiple variables");
    assert!(
        vars.contains_key(&var1),
        "vars must contain test variable 1"
    );
    assert!(
        vars.contains_key(&var2),
        "vars must contain test variable 2"
    );
    assert_eq!(
        vars.get(&var1),
        Some(&"value1".to_string()),
        "Value 1 must match"
    );
    assert_eq!(
        vars.get(&var2),
        Some(&"value2".to_string()),
        "Value 2 must match"
    );

    // Cleanup
    env::remove_var(&var1);
    env::remove_var(&var2);
}

#[test]
fn test_std_005_current_dir() {
    // STD-005: Test getting current working directory

    // Call ruchy::stdlib::env::current_dir
    let result = ruchy::stdlib::env::current_dir();

    assert!(result.is_ok(), "current_dir should succeed");
    let dir = result.unwrap();
    assert!(!dir.is_empty(), "Current directory must not be empty");
    assert!(dir.contains('/'), "Path must contain separators");
    // Should be an absolute path
    assert!(
        dir.starts_with('/') || dir.contains(":\\"),
        "Must be absolute path"
    );
}

#[test]
fn test_std_005_set_current_dir() {
    // STD-005: Test setting current working directory

    let original_dir = env::current_dir().expect("Failed to get current dir");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Call ruchy::stdlib::env::set_current_dir
    let result = ruchy::stdlib::env::set_current_dir(temp_dir.path().to_str().unwrap());

    assert!(result.is_ok(), "set_current_dir should succeed");

    // Verify directory changed
    let new_dir = env::current_dir().expect("Failed to get current dir");
    assert_eq!(
        new_dir,
        temp_dir.path(),
        "Current directory must match temp dir"
    );
    assert_ne!(new_dir, original_dir, "Directory must have changed");

    // Restore original directory
    env::set_current_dir(&original_dir).expect("Failed to restore dir");
}

#[test]
fn test_std_005_set_current_dir_invalid() {
    // STD-005: Test setting current dir to invalid path returns error

    let result = ruchy::stdlib::env::set_current_dir("/nonexistent/path/that/does/not/exist");

    assert!(
        result.is_err(),
        "set_current_dir should fail for invalid path"
    );
}

#[test]
fn test_std_005_args() {
    // STD-005: Test getting command line arguments

    // Call ruchy::stdlib::env::args
    let result = ruchy::stdlib::env::args();

    assert!(result.is_ok(), "args should succeed");
    let args = result.unwrap();

    assert!(!args.is_empty(), "args must not be empty");
    // First arg should be program name
    assert!(
        !args[0].is_empty(),
        "First arg (program name) must not be empty"
    );
    assert!(!args.is_empty(), "Must have at least program name");
}

#[test]
fn test_std_005_temp_dir() {
    // STD-005: Test getting temporary directory path

    // Call ruchy::stdlib::env::temp_dir
    let result = ruchy::stdlib::env::temp_dir();

    assert!(result.is_ok(), "temp_dir should succeed");
    let dir = result.unwrap();
    assert!(!dir.is_empty(), "Temp directory must not be empty");
    assert!(dir.contains('/') || dir.contains('\\'), "Must be a path");
    // Should be an absolute path
    assert!(
        dir.starts_with('/') || dir.contains(":\\"),
        "Must be absolute path"
    );
    // Verify the directory exists
    assert!(
        std::path::Path::new(&dir).exists(),
        "Temp directory must exist"
    );
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_005_set_get_roundtrip(value in "[a-zA-Z0-9]{1,50}") {
            // Property: Setting then getting a variable returns same value

            let var_name = test_var_name("ROUNDTRIP");

            let set_result = ruchy::stdlib::env::set_var(&var_name, &value);
            assert!(set_result.is_ok(), "set_var should succeed");

            let get_result = ruchy::stdlib::env::var(&var_name);
            assert!(get_result.is_ok(), "var should succeed");
            assert_eq!(get_result.unwrap(), value, "Value must roundtrip");

            // Cleanup
            env::remove_var(&var_name);
        }

        #[test]
        fn test_std_005_remove_idempotent(n in 0u8..5) {
            // Property: Removing a variable multiple times should not fail

            let var_name = test_var_name("IDEMPOTENT");
            env::set_var(&var_name, "value");

            // Remove n times
            for _ in 0..n {
                let result = ruchy::stdlib::env::remove_var(&var_name);
                assert!(result.is_ok(), "remove_var should always succeed");
            }
        }

        #[test]
        fn test_std_005_vars_contains_set(key in "[A-Z_]{5,20}", value in "[a-z0-9]{1,20}") {
            // Property: vars() contains any variable we set

            let var_name = format!("RUCHY_PROP_{key}");

            let set_result = ruchy::stdlib::env::set_var(&var_name, &value);
            assert!(set_result.is_ok(), "set_var should succeed");

            let vars_result = ruchy::stdlib::env::vars();
            assert!(vars_result.is_ok(), "vars should succeed");
            let vars = vars_result.unwrap();

            assert!(vars.contains_key(&var_name), "vars must contain set variable");
            assert_eq!(vars.get(&var_name), Some(&value), "Value must match");

            // Cleanup
            env::remove_var(&var_name);
        }
    }
}
