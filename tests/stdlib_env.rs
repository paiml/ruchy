//! EXTREME TDD Coverage Tests for `stdlib::env` Module
//!
//! Target: 0% → 80% coverage (+26 lines)
//! Protocol: RED → GREEN → REFACTOR → VALIDATE
//! Quality: Property tests + mutation tests ≥75%

use ruchy::stdlib::env;
use std::collections::HashMap;
use tempfile::TempDir;

// ============================================================================
// UNIT TESTS (Basic Function Coverage)
// ============================================================================

#[test]
fn test_var_success() {
    // Setup
    env::set_var("TEST_VAR_123", "test_value").unwrap();

    // Execute
    let result = env::var("TEST_VAR_123");

    // Verify
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test_value");

    // Cleanup
    env::remove_var("TEST_VAR_123").unwrap();
}

#[test]
fn test_var_missing() {
    // Execute on non-existent variable
    let result = env::var("NONEXISTENT_VAR_XYZ_999");

    // Verify error (error message varies by platform)
    assert!(result.is_err());
    // Just verify we got an error - the specific message varies
}

#[test]
fn test_set_var_and_get() {
    // Execute
    env::set_var("RUCHY_TEST_SET", "set_value").unwrap();
    let result = env::var("RUCHY_TEST_SET");

    // Verify
    assert_eq!(result.unwrap(), "set_value");

    // Cleanup
    env::remove_var("RUCHY_TEST_SET").unwrap();
}

#[test]
fn test_remove_var() {
    // Setup
    env::set_var("RUCHY_TEST_REMOVE", "temporary").unwrap();
    assert!(env::var("RUCHY_TEST_REMOVE").is_ok());

    // Execute
    env::remove_var("RUCHY_TEST_REMOVE").unwrap();

    // Verify removed
    assert!(env::var("RUCHY_TEST_REMOVE").is_err());
}

#[test]
fn test_vars_returns_hashmap() {
    // Execute
    let result = env::vars();

    // Verify
    assert!(result.is_ok());
    let vars: HashMap<String, String> = result.unwrap();
    assert!(!vars.is_empty());

    // Standard env vars should exist
    assert!(vars.contains_key("PATH") || vars.contains_key("Path"));
}

#[test]
fn test_vars_contains_custom() {
    // Setup
    env::set_var("RUCHY_CUSTOM_VAR", "custom_value").unwrap();

    // Execute
    let vars = env::vars().unwrap();

    // Verify
    assert_eq!(vars.get("RUCHY_CUSTOM_VAR"), Some(&"custom_value".to_string()));

    // Cleanup
    env::remove_var("RUCHY_CUSTOM_VAR").unwrap();
}

#[test]
fn test_current_dir_success() {
    // Execute
    let result = env::current_dir();

    // Verify
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(!dir.is_empty());
    assert!(std::path::Path::new(&dir).exists());
}

#[test]
fn test_set_current_dir_success() {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    let original = env::current_dir().unwrap();

    // Execute
    let result = env::set_current_dir(temp_dir.path().to_str().unwrap());

    // Verify
    assert!(result.is_ok());
    let new_dir = env::current_dir().unwrap();
    assert!(new_dir.contains(temp_dir.path().file_name().unwrap().to_str().unwrap()));

    // Restore
    env::set_current_dir(&original).unwrap();
}

#[test]
fn test_set_current_dir_invalid() {
    // Execute with invalid path
    let result = env::set_current_dir("/nonexistent/invalid/path/xyz");

    // Verify error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("No such file") ||
            err.contains("NotFound") ||
            err.contains("cannot find"));
}

#[test]
fn test_args_not_empty() {
    // Execute
    let result = env::args();

    // Verify
    assert!(result.is_ok());
    let args = result.unwrap();
    assert!(!args.is_empty()); // At least program name
}

#[test]
fn test_temp_dir_exists() {
    // Execute
    let result = env::temp_dir();

    // Verify
    assert!(result.is_ok());
    let temp = result.unwrap();
    assert!(!temp.is_empty());
    assert!(std::path::Path::new(&temp).exists());
}

// ============================================================================
// PROPERTY-BASED TESTS (High Coverage per Test)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn property_set_get_roundtrip(
        key in "[A-Z_]{5,20}",
        value in "[ -~]{1,100}" // Printable ASCII
    ) {
        // Property: set_var → var should return same value
        let test_key = format!("RUCHY_PROP_{key}");

        env::set_var(&test_key, &value).unwrap();
        let result = env::var(&test_key).unwrap();

        // Verify roundtrip
        prop_assert_eq!(result, value);

        // Cleanup
        env::remove_var(&test_key).unwrap();
    }

    #[test]
    fn property_remove_makes_var_missing(
        key in "[A-Z_]{5,20}"
    ) {
        // Property: remove_var → var should fail
        let test_key = format!("RUCHY_REM_{key}");

        env::set_var(&test_key, "temp").unwrap();
        env::remove_var(&test_key).unwrap();

        // Verify missing
        prop_assert!(env::var(&test_key).is_err());
    }

    #[test]
    fn property_vars_always_contains_path(
        _dummy in 0..100i32 // Run 100 times
    ) {
        // Property: vars() always contains PATH (on Unix/Windows)
        let vars = env::vars().unwrap();

        prop_assert!(
            vars.contains_key("PATH") || vars.contains_key("Path"),
            "Environment should contain PATH variable"
        );
    }
}

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_var_empty_string() {
    // Setup: Empty string value
    env::set_var("RUCHY_EMPTY", "").unwrap();

    // Execute
    let result = env::var("RUCHY_EMPTY");

    // Verify: Empty string is valid
    assert_eq!(result.unwrap(), "");

    // Cleanup
    env::remove_var("RUCHY_EMPTY").unwrap();
}

#[test]
fn test_var_unicode() {
    // Setup: Unicode characters
    env::set_var("RUCHY_UNICODE", "Hello 世界 🌍").unwrap();

    // Execute
    let result = env::var("RUCHY_UNICODE");

    // Verify
    assert_eq!(result.unwrap(), "Hello 世界 🌍");

    // Cleanup
    env::remove_var("RUCHY_UNICODE").unwrap();
}

#[test]
fn test_var_special_chars() {
    // Setup: Special characters
    let special = r#"!@#$%^&*(){}[]|\:;"'<>,.?/~`"#;
    env::set_var("RUCHY_SPECIAL", special).unwrap();

    // Execute
    let result = env::var("RUCHY_SPECIAL");

    // Verify
    assert_eq!(result.unwrap(), special);

    // Cleanup
    env::remove_var("RUCHY_SPECIAL").unwrap();
}

#[test]
fn test_current_dir_is_absolute() {
    // Execute
    let dir = env::current_dir().unwrap();

    // Verify: Should be absolute path
    assert!(std::path::Path::new(&dir).is_absolute());
}

// ============================================================================
// INTEGRATION TESTS (Multiple Functions Together)
// ============================================================================

#[test]
fn test_env_workflow() {
    // Step 1: Set variable
    env::set_var("RUCHY_WORKFLOW", "step1").unwrap();

    // Step 2: Verify in vars()
    let all_vars = env::vars().unwrap();
    assert_eq!(all_vars.get("RUCHY_WORKFLOW"), Some(&"step1".to_string()));

    // Step 3: Update variable
    env::set_var("RUCHY_WORKFLOW", "step2").unwrap();
    assert_eq!(env::var("RUCHY_WORKFLOW").unwrap(), "step2");

    // Step 4: Remove variable
    env::remove_var("RUCHY_WORKFLOW").unwrap();
    assert!(env::var("RUCHY_WORKFLOW").is_err());
}

#[test]
fn test_dir_workflow() {
    // Step 1: Get original
    let original = env::current_dir().unwrap();

    // Step 2: Change to temp
    let temp_dir = TempDir::new().unwrap();
    env::set_current_dir(temp_dir.path().to_str().unwrap()).unwrap();

    // Step 3: Verify changed
    let new_dir = env::current_dir().unwrap();
    assert_ne!(new_dir, original);

    // Step 4: Restore
    env::set_current_dir(&original).unwrap();
    assert_eq!(env::current_dir().unwrap(), original);
}
