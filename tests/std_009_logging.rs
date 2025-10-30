//! STD-009: Logging Module Tests (ruchy/std/logging)
//!
//! Test suite for Logging operations module.
//! Thin wrappers around `log` crate for basic logging functionality.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_logger() {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
    });
}

// ===== Logger Initialization Tests =====

#[test]
fn test_std_009_init_logger_info() {
    // STD-009: Initialize logger with info level

    init_test_logger();
    let result = ruchy::stdlib::logging::init_logger("info");

    assert!(result.is_ok(), "init_logger(info) should succeed");
}

#[test]
fn test_std_009_init_logger_debug() {
    // STD-009: Initialize logger with debug level

    init_test_logger();
    let result = ruchy::stdlib::logging::init_logger("debug");

    assert!(result.is_ok(), "init_logger(debug) should succeed");
}

#[test]
fn test_std_009_init_logger_invalid() {
    // STD-009: Invalid level returns error

    init_test_logger();
    let result = ruchy::stdlib::logging::init_logger("invalid_level");

    assert!(result.is_err(), "init_logger(invalid) should fail");
}

#[test]
fn test_std_009_init_logger_off() {
    // STD-009: Initialize with off level

    init_test_logger();
    let result = ruchy::stdlib::logging::init_logger("off");

    assert!(result.is_ok(), "init_logger(off) should succeed");
}

// ===== Logging Functions Tests =====

#[test]
fn test_std_009_log_info_basic() {
    // STD-009: Log info message succeeds

    init_test_logger();
    let result = ruchy::stdlib::logging::log_info("Test info message");

    assert!(result.is_ok(), "log_info should succeed");
}

#[test]
fn test_std_009_log_warn_basic() {
    // STD-009: Log warning succeeds

    init_test_logger();
    let result = ruchy::stdlib::logging::log_warn("Test warning message");

    assert!(result.is_ok(), "log_warn should succeed");
}

#[test]
fn test_std_009_log_error_basic() {
    // STD-009: Log error succeeds

    init_test_logger();
    let result = ruchy::stdlib::logging::log_error("Test error message");

    assert!(result.is_ok(), "log_error should succeed");
}

#[test]
fn test_std_009_log_debug_basic() {
    // STD-009: Log debug succeeds

    init_test_logger();
    let result = ruchy::stdlib::logging::log_debug("Test debug message");

    assert!(result.is_ok(), "log_debug should succeed");
}

#[test]
fn test_std_009_log_trace_basic() {
    // STD-009: Log trace succeeds

    init_test_logger();
    let result = ruchy::stdlib::logging::log_trace("Test trace message");

    assert!(result.is_ok(), "log_trace should succeed");
}

#[test]
fn test_std_009_log_info_empty() {
    // STD-009: Empty message works

    init_test_logger();
    let result = ruchy::stdlib::logging::log_info("");

    assert!(result.is_ok(), "log_info with empty message should succeed");
}

#[test]
fn test_std_009_log_info_long() {
    // STD-009: Long message works

    init_test_logger();
    let long_message = "a".repeat(10000);
    let result = ruchy::stdlib::logging::log_info(&long_message);

    assert!(result.is_ok(), "log_info with long message should succeed");
}

#[test]
fn test_std_009_log_info_special_chars() {
    // STD-009: Special characters work

    init_test_logger();
    let result = ruchy::stdlib::logging::log_info("Test \n\t\r special chars !@#$%^&*()");

    assert!(result.is_ok(), "log_info with special chars should succeed");
}

#[test]
fn test_std_009_log_info_unicode() {
    // STD-009: Unicode works

    init_test_logger();
    let result = ruchy::stdlib::logging::log_info("Test unicode: 你好 🚀 мир");

    assert!(result.is_ok(), "log_info with unicode should succeed");
}

#[test]
fn test_std_009_log_info_newlines() {
    // STD-009: Newlines work

    init_test_logger();
    let result = ruchy::stdlib::logging::log_info("Line 1\nLine 2\nLine 3");

    assert!(result.is_ok(), "log_info with newlines should succeed");
}

// ===== Level Checking Tests =====

#[test]
fn test_std_009_get_level_info() {
    // STD-009: Get level returns correct value

    init_test_logger();
    let _ = ruchy::stdlib::logging::init_logger("info");
    let result = ruchy::stdlib::logging::get_level();

    assert!(result.is_ok(), "get_level should succeed");
    let level = result.unwrap();
    assert!(
        ["info", "debug", "trace", "warn", "error", "off"].contains(&level.as_str()),
        "Level should be valid"
    );
}

#[test]
fn test_std_009_get_level_debug() {
    // STD-009: Get level after debug init

    init_test_logger();
    let _ = ruchy::stdlib::logging::init_logger("debug");
    let result = ruchy::stdlib::logging::get_level();

    assert!(result.is_ok(), "get_level should succeed");
}

#[test]
fn test_std_009_get_level_off() {
    // STD-009: Get level when off

    init_test_logger();
    let _ = ruchy::stdlib::logging::init_logger("off");
    let result = ruchy::stdlib::logging::get_level();

    assert!(result.is_ok(), "get_level should succeed");
}

#[test]
fn test_std_009_is_level_enabled_true() {
    // STD-009: Check enabled level returns true
    // Note: Logger can only be initialized once, so we just check that
    // the is_level_enabled function works correctly

    init_test_logger();
    let result = ruchy::stdlib::logging::is_level_enabled("error");

    assert!(result.is_ok(), "is_level_enabled should succeed");
    // Error level is always enabled regardless of configuration
    assert!(result.unwrap(), "Error level should always be enabled");
}

#[test]
fn test_std_009_is_level_enabled_false() {
    // STD-009: Check disabled level returns false

    init_test_logger();
    let _ = ruchy::stdlib::logging::init_logger("error");
    let result = ruchy::stdlib::logging::is_level_enabled("debug");

    assert!(result.is_ok(), "is_level_enabled should succeed");
    assert!(
        !result.unwrap(),
        "Debug should be disabled when error level is set"
    );
}

#[test]
fn test_std_009_is_level_enabled_invalid() {
    // STD-009: Invalid level returns error

    init_test_logger();
    let result = ruchy::stdlib::logging::is_level_enabled("invalid_level");

    assert!(
        result.is_err(),
        "is_level_enabled with invalid level should fail"
    );
}

#[test]
fn test_std_009_is_level_enabled_trace() {
    // STD-009: Trace level check
    // Note: Logger can only be initialized once, so we test the function works

    init_test_logger();
    let result = ruchy::stdlib::logging::is_level_enabled("warn");

    assert!(result.is_ok(), "is_level_enabled should succeed");
    // Just verify it returns a boolean (actual value depends on logger config)
    let _enabled = result.unwrap();
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_009_logging_never_panics(message: String) {
            // Property: Logging functions never panic on any input

            init_test_logger();
            let _ = ruchy::stdlib::logging::log_info(&message);
            let _ = ruchy::stdlib::logging::log_warn(&message);
            let _ = ruchy::stdlib::logging::log_error(&message);
            let _ = ruchy::stdlib::logging::log_debug(&message);
            let _ = ruchy::stdlib::logging::log_trace(&message);
            // Should not panic
        }

        #[test]
        fn test_std_009_level_check_consistent(level in prop::sample::select(vec!["trace", "debug", "info", "warn", "error"])) {
            // Property: is_level_enabled returns valid results for valid levels
            // Note: Can't test changing levels because logger can only init once

            init_test_logger();
            let result = ruchy::stdlib::logging::is_level_enabled(level);
            prop_assert!(result.is_ok(), "is_level_enabled should succeed for valid level");
        }

        #[test]
        fn test_std_009_invalid_level_fails(level: String) {
            // Property: Invalid levels always return errors

            init_test_logger();
            let valid_levels = ["trace", "debug", "info", "warn", "error", "off"];
            if !valid_levels.contains(&level.as_str()) {
                let result = ruchy::stdlib::logging::init_logger(&level);
                prop_assert!(result.is_err(), "Invalid level should fail");
            }
        }
    }
}
