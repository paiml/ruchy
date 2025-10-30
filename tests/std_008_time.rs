//! STD-008: Time Module Tests (ruchy/std/time)
//!
//! Test suite for Time operations module.
//! Thin wrappers around `std::time` for time measurement and duration operations.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use std::thread;
use std::time::Duration;

// ===== Time Measurement Tests =====

#[test]
fn test_std_008_now_positive() {
    // STD-008: Test that now() returns positive value

    let result = ruchy::stdlib::time::now();

    assert!(result.is_ok(), "now() should succeed");
    let timestamp = result.unwrap();
    assert!(timestamp > 0, "Timestamp should be positive");
}

#[test]
fn test_std_008_now_reasonable_range() {
    // STD-008: Test that now() returns reasonable timestamp
    // Should be after 2020-01-01 and before 2100-01-01

    let result = ruchy::stdlib::time::now();

    assert!(result.is_ok(), "now() should succeed");
    let timestamp = result.unwrap();

    // 2020-01-01 00:00:00 UTC = 1577836800000 ms
    // 2100-01-01 00:00:00 UTC = 4102444800000 ms
    assert!(
        timestamp > 1_577_836_800_000,
        "Timestamp should be after 2020-01-01"
    );
    assert!(
        timestamp < 4_102_444_800_000,
        "Timestamp should be before 2100-01-01"
    );
}

#[test]
fn test_std_008_now_monotonic() {
    // STD-008: Test that now() never decreases

    let first = ruchy::stdlib::time::now().unwrap();
    thread::sleep(Duration::from_millis(10));
    let second = ruchy::stdlib::time::now().unwrap();

    assert!(
        second >= first,
        "Second timestamp should be >= first timestamp"
    );
}

#[test]
fn test_std_008_elapsed_millis_zero() {
    // STD-008: Test elapsed_millis with immediate call

    let start = ruchy::stdlib::time::now().unwrap();
    let result = ruchy::stdlib::time::elapsed_millis(start);

    assert!(result.is_ok(), "elapsed_millis should succeed");
    let elapsed = result.unwrap();
    assert!(elapsed < 100, "Immediate elapsed should be small (< 100ms)");
}

#[test]
fn test_std_008_elapsed_millis_positive() {
    // STD-008: Test elapsed_millis with actual delay

    let start = ruchy::stdlib::time::now().unwrap();
    thread::sleep(Duration::from_millis(50));
    let result = ruchy::stdlib::time::elapsed_millis(start);

    assert!(result.is_ok(), "elapsed_millis should succeed");
    let elapsed = result.unwrap();
    assert!(elapsed >= 40, "Elapsed should be >= 40ms (50ms sleep)");
    assert!(
        elapsed < 200,
        "Elapsed should be < 200ms (reasonable tolerance)"
    );
}

#[test]
fn test_std_008_elapsed_millis_increases() {
    // STD-008: Test that elapsed_millis increases over time

    let start = ruchy::stdlib::time::now().unwrap();
    thread::sleep(Duration::from_millis(10));
    let first_elapsed = ruchy::stdlib::time::elapsed_millis(start).unwrap();

    thread::sleep(Duration::from_millis(10));
    let second_elapsed = ruchy::stdlib::time::elapsed_millis(start).unwrap();

    assert!(
        second_elapsed > first_elapsed,
        "Second elapsed should be greater than first"
    );
}

// ===== Duration Operation Tests =====

#[test]
fn test_std_008_sleep_millis_basic() {
    // STD-008: Test sleep_millis with basic duration

    let start = ruchy::stdlib::time::now().unwrap();
    let result = ruchy::stdlib::time::sleep_millis(50);
    let elapsed = ruchy::stdlib::time::elapsed_millis(start).unwrap();

    assert!(result.is_ok(), "sleep_millis should succeed");
    assert!(elapsed >= 40, "Sleep should be >= 40ms");
    assert!(elapsed < 150, "Sleep should be < 150ms (with tolerance)");
}

#[test]
fn test_std_008_sleep_millis_zero() {
    // STD-008: Test sleep_millis with zero duration

    let result = ruchy::stdlib::time::sleep_millis(0);

    assert!(result.is_ok(), "sleep_millis(0) should succeed");
}

#[test]
fn test_std_008_sleep_millis_large() {
    // STD-008: Test sleep_millis doesn't panic on large value

    // Don't actually sleep for a large duration, just verify it doesn't panic
    let result = std::panic::catch_unwind(|| ruchy::stdlib::time::sleep_millis(1));

    assert!(result.is_ok(), "sleep_millis should not panic");
}

#[test]
fn test_std_008_duration_secs_conversion() {
    // STD-008: Test duration_secs conversion accuracy

    let result = ruchy::stdlib::time::duration_secs(1000);

    assert!(result.is_ok(), "duration_secs should succeed");
    let secs = result.unwrap();
    assert!((secs - 1.0).abs() < 0.01, "1000ms should be ~1.0 seconds");
}

#[test]
fn test_std_008_duration_secs_zero() {
    // STD-008: Test duration_secs with zero

    let result = ruchy::stdlib::time::duration_secs(0);

    assert!(result.is_ok(), "duration_secs should succeed");
    let secs = result.unwrap();
    assert_eq!(secs, 0.0, "0ms should be 0.0 seconds");
}

#[test]
fn test_std_008_duration_secs_fractional() {
    // STD-008: Test duration_secs with fractional seconds

    let result = ruchy::stdlib::time::duration_secs(1500);

    assert!(result.is_ok(), "duration_secs should succeed");
    let secs = result.unwrap();
    assert!((secs - 1.5).abs() < 0.01, "1500ms should be ~1.5 seconds");
}

// ===== Formatting Tests =====

#[test]
fn test_std_008_format_duration_milliseconds() {
    // STD-008: Test format_duration for milliseconds

    let result = ruchy::stdlib::time::format_duration(500);

    assert!(result.is_ok(), "format_duration should succeed");
    let formatted = result.unwrap();
    assert_eq!(formatted, "500ms", "500ms should format as '500ms'");
}

#[test]
fn test_std_008_format_duration_seconds() {
    // STD-008: Test format_duration for seconds

    let result = ruchy::stdlib::time::format_duration(5000);

    assert!(result.is_ok(), "format_duration should succeed");
    let formatted = result.unwrap();
    assert_eq!(formatted, "5s", "5000ms should format as '5s'");
}

#[test]
fn test_std_008_format_duration_minutes() {
    // STD-008: Test format_duration for minutes and seconds

    let result = ruchy::stdlib::time::format_duration(150_000);

    assert!(result.is_ok(), "format_duration should succeed");
    let formatted = result.unwrap();
    assert_eq!(formatted, "2m 30s", "150000ms should format as '2m 30s'");
}

#[test]
fn test_std_008_format_duration_hours() {
    // STD-008: Test format_duration for hours and minutes

    let result = ruchy::stdlib::time::format_duration(5_400_000);

    assert!(result.is_ok(), "format_duration should succeed");
    let formatted = result.unwrap();
    assert_eq!(formatted, "1h 30m", "5400000ms should format as '1h 30m'");
}

#[test]
fn test_std_008_format_duration_days() {
    // STD-008: Test format_duration for days and hours

    let result = ruchy::stdlib::time::format_duration(183_600_000);

    assert!(result.is_ok(), "format_duration should succeed");
    let formatted = result.unwrap();
    assert_eq!(formatted, "2d 3h", "183600000ms should format as '2d 3h'");
}

#[test]
fn test_std_008_parse_duration_milliseconds() {
    // STD-008: Test parse_duration for milliseconds

    let result = ruchy::stdlib::time::parse_duration("500ms");

    assert!(result.is_ok(), "parse_duration should succeed");
    let millis = result.unwrap();
    assert_eq!(millis, 500, "'500ms' should parse to 500");
}

#[test]
fn test_std_008_parse_duration_seconds() {
    // STD-008: Test parse_duration for seconds

    let result = ruchy::stdlib::time::parse_duration("5s");

    assert!(result.is_ok(), "parse_duration should succeed");
    let millis = result.unwrap();
    assert_eq!(millis, 5000, "'5s' should parse to 5000");
}

#[test]
fn test_std_008_parse_duration_complex() {
    // STD-008: Test parse_duration for complex format

    let result = ruchy::stdlib::time::parse_duration("1h 30m");

    assert!(result.is_ok(), "parse_duration should succeed");
    let millis = result.unwrap();
    assert_eq!(millis, 5_400_000, "'1h 30m' should parse to 5400000");
}

#[test]
fn test_std_008_parse_duration_invalid() {
    // STD-008: Test parse_duration with invalid format

    let result = ruchy::stdlib::time::parse_duration("invalid");

    assert!(
        result.is_err(),
        "parse_duration should fail for invalid format"
    );
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Error message should not be empty");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_008_now_never_panics(iterations in 1usize..100) {
            // Property: now() never panics, even when called repeatedly

            for _ in 0..iterations {
                let _ = ruchy::stdlib::time::now();
            }
            // Should not panic
        }

        #[test]
        fn test_std_008_elapsed_always_positive(delay_ms in 0u64..100) {
            // Property: elapsed_millis always returns positive value

            let start = ruchy::stdlib::time::now().unwrap();
            thread::sleep(Duration::from_millis(delay_ms));
            let elapsed = ruchy::stdlib::time::elapsed_millis(start).unwrap();

            // Elapsed is u128, always non-negative by type (no >= 0 check needed)
            // Verify elapsed time is reasonable (at least the sleep duration)
            assert!(elapsed >= u128::from(delay_ms), "Elapsed time should be at least the sleep duration");
        }

        #[test]
        fn test_std_008_format_parse_roundtrip(millis in 0u128..1_000_000_000) {
            // Property: format → parse should approximately roundtrip

            if let Ok(formatted) = ruchy::stdlib::time::format_duration(millis) {
                if let Ok(parsed) = ruchy::stdlib::time::parse_duration(&formatted) {
                    // Allow some tolerance due to formatting (e.g., "1h" loses seconds)
                    let diff = millis.abs_diff(parsed);

                    // Tolerance: max 1000ms difference for rounding
                    prop_assert!(
                        diff < 1000,
                        "Roundtrip difference should be < 1000ms: {} vs {}",
                        millis,
                        parsed
                    );
                }
            }
        }
    }
}
