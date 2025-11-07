//! EXTREME TDD Coverage Tests for stdlib::logging Module
//!
//! Target: 0% → 80% coverage (+32 lines)
//! Protocol: RED → GREEN → REFACTOR → VALIDATE
//! Quality: Property tests + mutation tests ≥75%

use ruchy::stdlib::logging;

// ============================================================================
// UNIT TESTS (Basic Function Coverage)
// ============================================================================

#[test]
fn test_init_logger_valid_levels() {
    // Test all valid log levels
    for level in &["trace", "debug", "info", "warn", "error", "off"] {
        let result = logging::init_logger(level);
        assert!(result.is_ok(), "Failed to init logger with level: {}", level);
    }
}

#[test]
fn test_init_logger_invalid() {
    // Test invalid log level
    let result = logging::init_logger("invalid_level");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid log level"));
}

#[test]
fn test_log_info() {
    logging::init_logger("info").ok();
    let result = logging::log_info("Test info message");
    assert!(result.is_ok());
}

#[test]
fn test_log_warn() {
    logging::init_logger("warn").ok();
    let result = logging::log_warn("Test warning message");
    assert!(result.is_ok());
}

#[test]
fn test_log_error() {
    logging::init_logger("error").ok();
    let result = logging::log_error("Test error message");
    assert!(result.is_ok());
}

#[test]
fn test_log_debug() {
    logging::init_logger("debug").ok();
    let result = logging::log_debug("Test debug message");
    assert!(result.is_ok());
}

#[test]
fn test_log_trace() {
    logging::init_logger("trace").ok();
    let result = logging::log_trace("Test trace message");
    assert!(result.is_ok());
}

#[test]
fn test_get_level() {
    logging::init_logger("info").ok();
    let result = logging::get_level();
    assert!(result.is_ok());
    let level = result.unwrap();
    assert!(["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()));
}

#[test]
fn test_is_level_enabled_valid() {
    logging::init_logger("info").ok();
    let result = logging::is_level_enabled("info");
    assert!(result.is_ok());
}

#[test]
fn test_is_level_enabled_invalid() {
    logging::init_logger("info").ok();
    let result = logging::is_level_enabled("invalid");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Invalid log level"));
}

// ============================================================================
// PROPERTY-BASED TESTS (High Coverage per Test)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn property_log_functions_never_fail(
        message in "[ -~]{1,100}" // Printable ASCII
    ) {
        // Property: All log functions should succeed
        logging::init_logger("trace").ok();

        prop_assert!(logging::log_trace(&message).is_ok());
        prop_assert!(logging::log_debug(&message).is_ok());
        prop_assert!(logging::log_info(&message).is_ok());
        prop_assert!(logging::log_warn(&message).is_ok());
        prop_assert!(logging::log_error(&message).is_ok());
    }

    #[test]
    fn property_get_level_always_valid(
        _dummy in 0..100i32
    ) {
        // Property: get_level() always returns valid level
        logging::init_logger("info").ok();
        let level = logging::get_level().unwrap();

        prop_assert!(
            ["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()),
            "Invalid level: {}", level
        );
    }

    #[test]
    fn property_is_level_enabled_consistency(
        level in prop::sample::select(vec!["trace", "debug", "info", "warn", "error"])
    ) {
        // Property: After setting level, querying it should work
        logging::init_logger(&level).ok();
        let result = logging::is_level_enabled(&level);

        prop_assert!(result.is_ok());
    }
}

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_log_empty_message() {
    logging::init_logger("info").ok();
    let result = logging::log_info("");
    assert!(result.is_ok());
}

#[test]
fn test_log_unicode() {
    logging::init_logger("info").ok();
    let result = logging::log_info("Hello 世界 🌍");
    assert!(result.is_ok());
}

#[test]
fn test_log_special_chars() {
    logging::init_logger("info").ok();
    let result = logging::log_info("Test: !@#$%^&*(){}[]|\\:;\"'<>,.?/~`");
    assert!(result.is_ok());
}

#[test]
fn test_log_long_message() {
    logging::init_logger("info").ok();
    let long_msg = "x".repeat(10000);
    let result = logging::log_info(&long_msg);
    assert!(result.is_ok());
}

#[test]
fn test_init_logger_case_variations() {
    // Both lowercase and uppercase should work
    assert!(logging::init_logger("info").is_ok());
    assert!(logging::init_logger("INFO").is_ok());
    assert!(logging::init_logger("Info").is_ok());
}

// ============================================================================
// INTEGRATION TESTS (Multiple Functions Together)
// ============================================================================

#[test]
fn test_logging_workflow() {
    // Initialize logger (may already be initialized by other tests)
    logging::init_logger("trace").ok();

    // Verify get_level returns valid level
    let level = logging::get_level().unwrap();
    assert!(["trace", "debug", "info", "warn", "error", "off"].contains(&level.as_str()));

    // Log messages at all levels (should always work)
    assert!(logging::log_trace("trace").is_ok());
    assert!(logging::log_debug("debug").is_ok());
    assert!(logging::log_info("info").is_ok());
    assert!(logging::log_warn("warn").is_ok());
    assert!(logging::log_error("error").is_ok());
}

#[test]
fn test_level_hierarchy() {
    // Info level should enable info, warn, error (but not debug, trace)
    logging::init_logger("info").ok();

    // Higher levels should be enabled
    assert!(logging::is_level_enabled("info").unwrap());
    assert!(logging::is_level_enabled("warn").unwrap());
    assert!(logging::is_level_enabled("error").unwrap());
}

#[test]
fn test_multiple_log_calls() {
    logging::init_logger("debug").ok();

    // Rapid logging should work
    for i in 0..100 {
        let msg = format!("Message {}", i);
        assert!(logging::log_debug(&msg).is_ok());
    }
}
