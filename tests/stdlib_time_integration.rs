//! Integration tests for stdlib::time module
//!
//! Target: 0% → 100% coverage for stdlib/time.rs (131 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests from tests/ directory ARE tracked by llvm-cov.

use ruchy::stdlib::time;

#[test]
fn test_time_now_returns_positive_timestamp() {
    let timestamp = time::now().unwrap();
    assert!(timestamp > 0, "Timestamp should be positive");
    assert!(
        timestamp > 946_684_800_000,
        "Timestamp should be after year 2000"
    );
}

#[test]
fn test_time_elapsed_millis_calculates_duration() {
    let start = time::now().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    let elapsed = time::elapsed_millis(start).unwrap();
    assert!(
        elapsed >= 10,
        "Elapsed time should be at least 10ms, got {elapsed}ms"
    );
}

#[test]
fn test_time_elapsed_millis_zero_duration() {
    let start = time::now().unwrap();
    let elapsed = time::elapsed_millis(start).unwrap();
    // elapsed is u128, always non-negative - just verify it's defined
    assert!(
        elapsed < 10,
        "Elapsed time should be very small immediately after start, got {elapsed}ms"
    );
}

#[test]
fn test_time_sleep_millis_succeeds() {
    let result = time::sleep_millis(1);
    assert!(result.is_ok(), "Sleep should succeed");
}

#[test]
fn test_time_sleep_millis_duration() {
    let start = time::now().unwrap();
    time::sleep_millis(50).unwrap();
    let elapsed = time::elapsed_millis(start).unwrap();
    assert!(
        elapsed >= 50,
        "Sleep should last at least 50ms, got {elapsed}ms"
    );
}

#[test]
fn test_time_duration_secs_whole_seconds() {
    let secs = time::duration_secs(1000).unwrap();
    assert_eq!(secs, 1.0, "1000ms should be 1.0 seconds");
}

#[test]
fn test_time_duration_secs_fractional() {
    let secs = time::duration_secs(1500).unwrap();
    assert!(
        (secs - 1.5).abs() < 0.01,
        "1500ms should be ~1.5 seconds, got {secs}"
    );
}

#[test]
fn test_time_duration_secs_zero() {
    let secs = time::duration_secs(0).unwrap();
    assert_eq!(secs, 0.0, "0ms should be 0.0 seconds");
}

#[test]
fn test_time_format_duration_milliseconds() {
    assert_eq!(
        time::format_duration(0).unwrap(),
        "0ms",
        "0ms should format as '0ms'"
    );
    assert_eq!(
        time::format_duration(500).unwrap(),
        "500ms",
        "500ms should format as '500ms'"
    );
    assert_eq!(
        time::format_duration(999).unwrap(),
        "999ms",
        "999ms should format as '999ms'"
    );
}

#[test]
fn test_time_format_duration_seconds() {
    assert_eq!(
        time::format_duration(1000).unwrap(),
        "1s",
        "1000ms should format as '1s'"
    );
    assert_eq!(
        time::format_duration(5000).unwrap(),
        "5s",
        "5000ms should format as '5s'"
    );
}

#[test]
fn test_time_format_duration_minutes() {
    assert_eq!(
        time::format_duration(60_000).unwrap(),
        "1m",
        "60000ms should format as '1m'"
    );
    assert_eq!(
        time::format_duration(90_000).unwrap(),
        "1m 30s",
        "90000ms should format as '1m 30s'"
    );
}

#[test]
fn test_time_format_duration_hours() {
    assert_eq!(
        time::format_duration(3_600_000).unwrap(),
        "1h",
        "3600000ms should format as '1h'"
    );
    assert_eq!(
        time::format_duration(5_400_000).unwrap(),
        "1h 30m",
        "5400000ms should format as '1h 30m'"
    );
}

#[test]
fn test_time_format_duration_days() {
    assert_eq!(
        time::format_duration(86_400_000).unwrap(),
        "1d",
        "86400000ms should format as '1d'"
    );
    assert_eq!(
        time::format_duration(90_000_000).unwrap(),
        "1d 1h",
        "90000000ms should format as '1d 1h'"
    );
}

#[test]
fn test_time_format_duration_complex() {
    // 1 day + 2 hours + 30 minutes + 45 seconds
    let millis = 86_400_000 + 7_200_000 + 1_800_000 + 45_000;
    let formatted = time::format_duration(millis).unwrap();
    assert_eq!(formatted, "1d 2h 30m 45s");
}

#[test]
fn test_time_parse_duration_milliseconds() {
    assert_eq!(
        time::parse_duration("500ms").unwrap(),
        500,
        "500ms should parse to 500"
    );
    assert_eq!(
        time::parse_duration("1000ms").unwrap(),
        1000,
        "1000ms should parse to 1000"
    );
}

#[test]
fn test_time_parse_duration_seconds() {
    assert_eq!(
        time::parse_duration("1s").unwrap(),
        1_000,
        "1s should parse to 1000ms"
    );
    assert_eq!(
        time::parse_duration("5s").unwrap(),
        5_000,
        "5s should parse to 5000ms"
    );
}

#[test]
fn test_time_parse_duration_minutes() {
    assert_eq!(
        time::parse_duration("1m").unwrap(),
        60_000,
        "1m should parse to 60000ms"
    );
    assert_eq!(
        time::parse_duration("10m").unwrap(),
        600_000,
        "10m should parse to 600000ms"
    );
}

#[test]
fn test_time_parse_duration_hours() {
    assert_eq!(
        time::parse_duration("1h").unwrap(),
        3_600_000,
        "1h should parse to 3600000ms"
    );
    assert_eq!(
        time::parse_duration("2h").unwrap(),
        7_200_000,
        "2h should parse to 7200000ms"
    );
}

#[test]
fn test_time_parse_duration_days() {
    assert_eq!(
        time::parse_duration("1d").unwrap(),
        86_400_000,
        "1d should parse to 86400000ms"
    );
    assert_eq!(
        time::parse_duration("7d").unwrap(),
        604_800_000,
        "7d should parse to 604800000ms"
    );
}

#[test]
fn test_time_parse_duration_compound() {
    assert_eq!(
        time::parse_duration("1h 30m").unwrap(),
        5_400_000,
        "1h 30m should parse to 5400000ms"
    );
    assert_eq!(
        time::parse_duration("1d 2h").unwrap(),
        93_600_000,
        "1d 2h should parse to 93600000ms"
    );
    assert_eq!(
        time::parse_duration("2h 15m 30s").unwrap(),
        8_130_000,
        "2h 15m 30s should parse to 8130000ms"
    );
}

#[test]
fn test_time_parse_duration_invalid_format() {
    assert!(
        time::parse_duration("invalid").is_err(),
        "Invalid format should return error"
    );
    assert!(
        time::parse_duration("10x").is_err(),
        "Invalid unit 'x' should return error"
    );
    assert!(
        time::parse_duration("10").is_err(),
        "Missing unit should return error"
    );
}

#[test]
fn test_time_parse_duration_empty_string() {
    assert!(
        time::parse_duration("").is_err(),
        "Empty string should return error"
    );
}

#[test]
fn test_time_parse_duration_zero_not_allowed() {
    assert!(
        time::parse_duration("0s").is_err(),
        "Zero duration should return error"
    );
    assert!(
        time::parse_duration("0m").is_err(),
        "Zero duration should return error"
    );
}

#[test]
fn test_time_format_parse_roundtrip() {
    // Test that formatting and parsing are inverse operations
    let test_values = [
        1_000,        // 1s
        60_000,       // 1m
        90_000,       // 1m 30s
        3_600_000,    // 1h
        5_400_000,    // 1h 30m
        86_400_000,   // 1d
        90_000_000,   // 1d 1h
    ];

    for millis in test_values {
        let formatted = time::format_duration(millis).unwrap();
        let parsed = time::parse_duration(&formatted).unwrap();
        assert_eq!(
            parsed, millis,
            "Roundtrip failed for {millis}ms: formatted as '{formatted}', parsed as {parsed}ms"
        );
    }
}

#[test]
fn test_time_workflow_measure_operation() {
    // Complete workflow: measure time of an operation
    let start = time::now().unwrap();

    // Simulate work
    time::sleep_millis(20).unwrap();

    // Calculate elapsed time
    let elapsed = time::elapsed_millis(start).unwrap();
    assert!(elapsed >= 20, "Operation should take at least 20ms");

    // Convert to seconds
    let secs = time::duration_secs(elapsed).unwrap();
    assert!(secs >= 0.02, "Operation should take at least 0.02 seconds");

    // Format duration
    let formatted = time::format_duration(elapsed).unwrap();
    assert!(!formatted.is_empty(), "Formatted duration should not be empty");
}
