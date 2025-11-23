//! EXTREME TDD Coverage Tests for `stdlib::time` Module
//!
//! Target: 0% → 80% coverage (+72 lines)
//! Protocol: RED → GREEN → REFACTOR → VALIDATE
//! Quality: Property tests + mutation tests ≥75%

use ruchy::stdlib::time;

// ============================================================================
// UNIT TESTS (Basic Function Coverage)
// ============================================================================

#[test]
fn test_now_positive() {
    let timestamp = time::now().expect("operation should succeed in test");
    assert!(timestamp > 0);
    // Should be after year 2000 (946684800000 ms)
    assert!(timestamp > 946_684_800_000);
}

#[test]
fn test_now_increasing() {
    let t1 = time::now().expect("operation should succeed in test");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let t2 = time::now().expect("operation should succeed in test");
    assert!(t2 > t1);
}

#[test]
fn test_elapsed_millis_basic() {
    let start = time::now().expect("operation should succeed in test");
    std::thread::sleep(std::time::Duration::from_millis(50));
    let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");
    assert!(elapsed >= 50);
    assert!(elapsed < 200); // Allow for timing variance
}

#[test]
fn test_elapsed_millis_zero() {
    let start = time::now().expect("operation should succeed in test");
    let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");
    assert!(elapsed < 10); // Should be very small
}

#[test]
fn test_sleep_millis_basic() {
    let start = time::now().expect("operation should succeed in test");
    time::sleep_millis(100).expect("operation should succeed in test");
    let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");
    assert!(elapsed >= 100);
    assert!(elapsed < 200);
}

#[test]
fn test_sleep_millis_zero() {
    // Sleep for 0ms should work
    let result = time::sleep_millis(0);
    assert!(result.is_ok());
}

#[test]
fn test_duration_secs_basic() {
    let secs = time::duration_secs(1000).expect("operation should succeed in test");
    assert!((secs - 1.0).abs() < 0.01);

    let secs = time::duration_secs(1500).expect("operation should succeed in test");
    assert!((secs - 1.5).abs() < 0.01);

    let secs = time::duration_secs(60_000).expect("operation should succeed in test");
    assert!((secs - 60.0).abs() < 0.01);
}

#[test]
fn test_duration_secs_zero() {
    let secs = time::duration_secs(0).expect("operation should succeed in test");
    assert_eq!(secs, 0.0);
}

#[test]
fn test_format_duration_milliseconds() {
    assert_eq!(
        time::format_duration(0).expect("operation should succeed in test"),
        "0ms"
    );
    assert_eq!(
        time::format_duration(1).expect("operation should succeed in test"),
        "1ms"
    );
    assert_eq!(
        time::format_duration(999).expect("operation should succeed in test"),
        "999ms"
    );
}

#[test]
fn test_format_duration_seconds() {
    assert_eq!(
        time::format_duration(1000).expect("operation should succeed in test"),
        "1s"
    );
    assert_eq!(
        time::format_duration(5000).expect("operation should succeed in test"),
        "5s"
    );
    assert_eq!(
        time::format_duration(59_000).expect("operation should succeed in test"),
        "59s"
    );
}

#[test]
fn test_format_duration_minutes() {
    assert_eq!(
        time::format_duration(60_000).expect("operation should succeed in test"),
        "1m"
    );
    assert_eq!(
        time::format_duration(90_000).expect("operation should succeed in test"),
        "1m 30s"
    );
    assert_eq!(
        time::format_duration(90_500).expect("operation should succeed in test"),
        "1m 30s"
    );
}

#[test]
fn test_format_duration_hours() {
    assert_eq!(
        time::format_duration(3_600_000).expect("operation should succeed in test"),
        "1h"
    );
    assert_eq!(
        time::format_duration(3_660_000).expect("operation should succeed in test"),
        "1h 1m"
    );
    assert_eq!(
        time::format_duration(5_400_000).expect("operation should succeed in test"),
        "1h 30m"
    );
}

#[test]
fn test_format_duration_days() {
    assert_eq!(
        time::format_duration(86_400_000).expect("operation should succeed in test"),
        "1d"
    );
    assert_eq!(
        time::format_duration(90_000_000).expect("operation should succeed in test"),
        "1d 1h"
    );
    assert_eq!(
        time::format_duration(172_800_000).expect("operation should succeed in test"),
        "2d"
    );
}

#[test]
fn test_format_duration_complex() {
    // 1 day, 2 hours, 3 minutes, 4 seconds
    let millis = 86_400_000 + 7_200_000 + 180_000 + 4_000;
    let formatted = time::format_duration(millis).expect("operation should succeed in test");
    assert_eq!(formatted, "1d 2h 3m 4s");
}

#[test]
fn test_parse_duration_milliseconds() {
    assert_eq!(
        time::parse_duration("500ms").expect("operation should succeed in test"),
        500
    );
    assert_eq!(
        time::parse_duration("1ms").expect("operation should succeed in test"),
        1
    );
}

#[test]
fn test_parse_duration_seconds() {
    assert_eq!(
        time::parse_duration("1s").expect("operation should succeed in test"),
        1_000
    );
    assert_eq!(
        time::parse_duration("30s").expect("operation should succeed in test"),
        30_000
    );
}

#[test]
fn test_parse_duration_minutes() {
    assert_eq!(
        time::parse_duration("1m").expect("operation should succeed in test"),
        60_000
    );
    assert_eq!(
        time::parse_duration("5m").expect("operation should succeed in test"),
        300_000
    );
}

#[test]
fn test_parse_duration_hours() {
    assert_eq!(
        time::parse_duration("1h").expect("operation should succeed in test"),
        3_600_000
    );
    assert_eq!(
        time::parse_duration("2h").expect("operation should succeed in test"),
        7_200_000
    );
}

#[test]
fn test_parse_duration_days() {
    assert_eq!(
        time::parse_duration("1d").expect("operation should succeed in test"),
        86_400_000
    );
    assert_eq!(
        time::parse_duration("7d").expect("operation should succeed in test"),
        604_800_000
    );
}

#[test]
fn test_parse_duration_compound() {
    assert_eq!(
        time::parse_duration("1h 30m").expect("operation should succeed in test"),
        5_400_000
    );
    assert_eq!(
        time::parse_duration("1d 2h 3m 4s").expect("operation should succeed in test"),
        86_400_000 + 7_200_000 + 180_000 + 4_000
    );
    assert_eq!(
        time::parse_duration("500ms 1s").expect("operation should succeed in test"),
        1_500
    );
}

#[test]
fn test_parse_duration_invalid() {
    assert!(time::parse_duration("invalid").is_err());
    assert!(time::parse_duration("10x").is_err());
    assert!(time::parse_duration("").is_err());
}

#[test]
fn test_format_parse_roundtrip() {
    // Test that format → parse → format produces same result
    let millis = vec![1000, 60_000, 90_000, 3_600_000, 86_400_000];

    for m in millis {
        let formatted = time::format_duration(m).expect("operation should succeed in test");
        let parsed = time::parse_duration(&formatted).expect("operation should succeed in test");
        assert_eq!(parsed, m, "Roundtrip failed for {m}");
    }
}

// ============================================================================
// PROPERTY-BASED TESTS (High Coverage per Test)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn property_now_always_positive(
        _dummy in 0..100i32
    ) {
        // Property: now() always returns positive timestamp
        let timestamp = time::now().expect("operation should succeed in test");
        prop_assert!(timestamp > 0);
        prop_assert!(timestamp > 946_684_800_000); // After year 2000
    }

    #[test]
    fn property_elapsed_never_negative(
        sleep_ms in 0u64..100
    ) {
        // Property: Elapsed time is never negative
        let start = time::now().expect("operation should succeed in test");
        time::sleep_millis(sleep_ms).ok();
        let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");

        prop_assert!(elapsed >= u128::from(sleep_ms));
    }

    #[test]
    fn property_duration_secs_conversion(
        millis in 0u128..1_000_000
    ) {
        // Property: Converting ms to seconds and back
        let secs = time::duration_secs(millis).expect("operation should succeed in test");
        let back_to_millis = (secs * 1000.0) as u128;

        prop_assert!((back_to_millis as i128 - millis as i128).abs() <= 1);
    }

    #[test]
    fn property_format_parse_roundtrip(
        seconds in 1u128..10_000
    ) {
        // Property: format → parse should be identity
        let millis = seconds * 1000;
        let formatted = time::format_duration(millis).expect("operation should succeed in test");
        let parsed = time::parse_duration(&formatted).expect("operation should succeed in test");

        prop_assert_eq!(parsed, millis);
    }

    #[test]
    fn property_sleep_respects_duration(
        sleep_ms in 10u64..100
    ) {
        // Property: sleep_millis actually sleeps for requested time
        let start = time::now().expect("operation should succeed in test");
        time::sleep_millis(sleep_ms).expect("operation should succeed in test");
        let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");

        prop_assert!(elapsed >= u128::from(sleep_ms));
        prop_assert!(elapsed < (u128::from(sleep_ms) + 100)); // Allow variance
    }
}

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_format_duration_edge_cases() {
    // Just under thresholds
    assert_eq!(
        time::format_duration(999).expect("operation should succeed in test"),
        "999ms"
    );
    assert_eq!(
        time::format_duration(59_999).expect("operation should succeed in test"),
        "59s"
    );

    // Exactly at thresholds
    assert_eq!(
        time::format_duration(1_000).expect("operation should succeed in test"),
        "1s"
    );
    assert_eq!(
        time::format_duration(60_000).expect("operation should succeed in test"),
        "1m"
    );
}

#[test]
fn test_parse_duration_whitespace() {
    // Multiple spaces
    assert_eq!(
        time::parse_duration("1h  30m").expect("operation should succeed in test"),
        5_400_000
    );

    // Leading/trailing spaces
    assert_eq!(
        time::parse_duration("  1h 30m  ").expect("operation should succeed in test"),
        5_400_000
    );
}

#[test]
fn test_parse_duration_zero_values() {
    // Zero is invalid (must have at least one component)
    assert!(time::parse_duration("0s").is_err());
    assert!(time::parse_duration("0ms").is_err());
}

#[test]
fn test_large_durations() {
    // Test very large durations don't overflow
    let large = time::format_duration(u128::MAX / 2).expect("operation should succeed in test");
    assert!(!large.is_empty());

    // Very large parse
    let result = time::parse_duration("365d");
    assert!(result.is_ok());
    assert_eq!(
        result.expect("operation should succeed in test"),
        31_536_000_000
    );
}

// ============================================================================
// INTEGRATION TESTS (Multiple Functions Together)
// ============================================================================

#[test]
fn test_timing_workflow() {
    // Step 1: Get start time
    let start = time::now().expect("operation should succeed in test");

    // Step 2: Sleep for known duration
    time::sleep_millis(50).expect("operation should succeed in test");

    // Step 3: Calculate elapsed
    let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");
    assert!(elapsed >= 50);

    // Step 4: Convert to seconds
    let secs = time::duration_secs(elapsed).expect("operation should succeed in test");
    assert!(secs >= 0.05);

    // Step 5: Format as string
    let formatted = time::format_duration(elapsed).expect("operation should succeed in test");
    assert!(!formatted.is_empty());
}

#[test]
fn test_benchmark_simple_operation() {
    // Benchmark a simple operation
    let start = time::now().expect("operation should succeed in test");

    // Do some work
    let mut sum = 0;
    for i in 0..1000 {
        sum += i;
    }

    let elapsed = time::elapsed_millis(start).expect("operation should succeed in test");
    let formatted = time::format_duration(elapsed).expect("operation should succeed in test");

    // Should complete very quickly
    assert!(elapsed < 1000); // Less than 1 second
    assert!(sum > 0); // Prevent optimization
    assert!(!formatted.is_empty());
}

#[test]
fn test_duration_parsing_workflow() {
    // Parse user input
    let user_input = "1h 30m 45s";
    let millis = time::parse_duration(user_input).expect("operation should succeed in test");

    // Convert to seconds
    let secs = time::duration_secs(millis).expect("operation should succeed in test");
    assert!((secs - 5445.0).abs() < 0.01); // 1.5 hours + 45s

    // Format back
    let formatted = time::format_duration(millis).expect("operation should succeed in test");
    assert_eq!(formatted, "1h 30m 45s");
}
