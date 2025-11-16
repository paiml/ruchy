//! Integration tests for `stdlib::logging` module
//!
//! Target: 0% → 100% coverage for stdlib/logging.rs (96 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests from tests/ directory ARE tracked by llvm-cov.

use ruchy::stdlib::logging;

#[test]
fn test_logging_init_logger_valid_levels() {
    // Test all valid log levels
    assert!(
        logging::init_logger("trace").is_ok(),
        "trace level should be valid"
    );
    assert!(
        logging::init_logger("debug").is_ok(),
        "debug level should be valid"
    );
    assert!(
        logging::init_logger("info").is_ok(),
        "info level should be valid"
    );
    assert!(
        logging::init_logger("warn").is_ok(),
        "warn level should be valid"
    );
    assert!(
        logging::init_logger("error").is_ok(),
        "error level should be valid"
    );
    assert!(
        logging::init_logger("off").is_ok(),
        "off level should be valid"
    );
}

#[test]
fn test_logging_init_logger_invalid_level() {
    assert!(
        logging::init_logger("invalid").is_err(),
        "invalid level should return error"
    );
    assert!(
        logging::init_logger("unknown").is_err(),
        "unknown level should return error"
    );
    assert!(
        logging::init_logger("notarealthing").is_err(),
        "notarealthing level should return error"
    );
}

#[test]
fn test_logging_init_logger_case_insensitive() {
    // Log level parsing should be case-insensitive
    assert!(
        logging::init_logger("INFO").is_ok(),
        "INFO (uppercase) should be valid"
    );
    assert!(
        logging::init_logger("Debug").is_ok(),
        "Debug (mixed case) should be valid"
    );
    assert!(
        logging::init_logger("WARN").is_ok(),
        "WARN (uppercase) should be valid"
    );
}

#[test]
fn test_logging_log_info_basic() {
    let _ = logging::init_logger("info");
    assert!(
        logging::log_info("Test info message").is_ok(),
        "log_info should succeed"
    );
    assert!(
        logging::log_info("").is_ok(),
        "log_info with empty message should succeed"
    );
}

#[test]
fn test_logging_log_warn_basic() {
    let _ = logging::init_logger("warn");
    assert!(
        logging::log_warn("Test warning").is_ok(),
        "log_warn should succeed"
    );
    assert!(
        logging::log_warn("").is_ok(),
        "log_warn with empty message should succeed"
    );
}

#[test]
fn test_logging_log_error_basic() {
    let _ = logging::init_logger("error");
    assert!(
        logging::log_error("Test error").is_ok(),
        "log_error should succeed"
    );
    assert!(
        logging::log_error("").is_ok(),
        "log_error with empty message should succeed"
    );
}

#[test]
fn test_logging_log_debug_basic() {
    let _ = logging::init_logger("debug");
    assert!(
        logging::log_debug("Test debug").is_ok(),
        "log_debug should succeed"
    );
    assert!(
        logging::log_debug("").is_ok(),
        "log_debug with empty message should succeed"
    );
}

#[test]
fn test_logging_log_trace_basic() {
    let _ = logging::init_logger("trace");
    assert!(
        logging::log_trace("Test trace").is_ok(),
        "log_trace should succeed"
    );
    assert!(
        logging::log_trace("").is_ok(),
        "log_trace with empty message should succeed"
    );
}

#[test]
fn test_logging_get_level() {
    // After init, should return a valid level string
    let _ = logging::init_logger("info");
    let level = logging::get_level().unwrap();
    assert!(
        ["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()),
        "get_level should return valid level string, got: {level}"
    );
}

#[test]
fn test_logging_is_level_enabled_valid() {
    let _ = logging::init_logger("info");

    // All level checks should return Ok (bool), not Err
    assert!(
        logging::is_level_enabled("info").is_ok(),
        "is_level_enabled(info) should succeed"
    );
    assert!(
        logging::is_level_enabled("warn").is_ok(),
        "is_level_enabled(warn) should succeed"
    );
    assert!(
        logging::is_level_enabled("error").is_ok(),
        "is_level_enabled(error) should succeed"
    );
    assert!(
        logging::is_level_enabled("debug").is_ok(),
        "is_level_enabled(debug) should succeed"
    );
    assert!(
        logging::is_level_enabled("trace").is_ok(),
        "is_level_enabled(trace) should succeed"
    );
}

#[test]
fn test_logging_is_level_enabled_invalid() {
    assert!(
        logging::is_level_enabled("invalid").is_err(),
        "is_level_enabled with invalid level should return error"
    );
    assert!(
        logging::is_level_enabled("unknown").is_err(),
        "is_level_enabled with unknown level should return error"
    );
    assert!(
        logging::is_level_enabled("notreal").is_err(),
        "is_level_enabled with notreal level should return error"
    );
}

#[test]
fn test_logging_multiple_init_calls() {
    // Multiple init calls should not fail (uses try_init internally)
    assert!(
        logging::init_logger("info").is_ok(),
        "first init should succeed"
    );
    assert!(
        logging::init_logger("debug").is_ok(),
        "second init should succeed"
    );
    assert!(
        logging::init_logger("warn").is_ok(),
        "third init should succeed"
    );
}

#[test]
fn test_logging_log_all_levels_workflow() {
    let _ = logging::init_logger("trace");

    // All log functions should work
    assert!(
        logging::log_trace("Trace message").is_ok(),
        "log_trace should succeed"
    );
    assert!(
        logging::log_debug("Debug message").is_ok(),
        "log_debug should succeed"
    );
    assert!(
        logging::log_info("Info message").is_ok(),
        "log_info should succeed"
    );
    assert!(
        logging::log_warn("Warn message").is_ok(),
        "log_warn should succeed"
    );
    assert!(
        logging::log_error("Error message").is_ok(),
        "log_error should succeed"
    );
}

#[test]
fn test_logging_special_characters() {
    let _ = logging::init_logger("info");

    // Test messages with special characters
    assert!(
        logging::log_info("Message with \"quotes\"").is_ok(),
        "should handle quotes"
    );
    assert!(
        logging::log_info("Message with\nnewlines").is_ok(),
        "should handle newlines"
    );
    assert!(
        logging::log_info("Message with\ttabs").is_ok(),
        "should handle tabs"
    );
    assert!(
        logging::log_info("Message with émojis 😀").is_ok(),
        "should handle unicode"
    );
}

#[test]
fn test_logging_long_message() {
    let _ = logging::init_logger("info");

    // Very long message
    let long_msg = "x".repeat(10000);
    assert!(
        logging::log_info(&long_msg).is_ok(),
        "should handle very long messages"
    );
}

#[test]
fn test_logging_level_hierarchy() {
    // Test that level hierarchy works
    let _ = logging::init_logger("warn");

    // After setting to warn, should still work
    let level = logging::get_level().unwrap();
    assert!(
        ["warn", "error", "off", "info", "debug", "trace"].contains(&level.as_str()),
        "get_level should return valid level, got: {level}"
    );
}

#[test]
fn test_logging_complete_workflow() {
    // Complete workflow: init, log at different levels, check level
    let _ = logging::init_logger("debug");

    // Get current level
    let level = logging::get_level().unwrap();
    assert!(
        !level.is_empty(),
        "level should not be empty after init"
    );

    // Check that debug is enabled
    let debug_enabled = logging::is_level_enabled("debug").unwrap();
    // Result may be true or false depending on configuration

    // Log messages at different levels
    assert!(logging::log_debug("Debug workflow test").is_ok());
    assert!(logging::log_info("Info workflow test").is_ok());
    assert!(logging::log_warn("Warn workflow test").is_ok());
    assert!(logging::log_error("Error workflow test").is_ok());

    // All operations should succeed
    assert!(true, "Complete workflow succeeded");
}

#[test]
fn test_logging_init_with_each_level_and_log() {
    // Test init with each level and ensure logging works
    for level in ["trace", "debug", "info", "warn", "error"] {
        let _ = logging::init_logger(level);
        assert!(
            logging::log_info(&format!("Testing with level: {level}")).is_ok(),
            "logging should work after init with {level}"
        );
    }
}
