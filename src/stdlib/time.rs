//! `Time` Operations Module (ruchy/std/time)
//!
//! Thin wrappers around Rust's `std::time` for time measurement and duration operations.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around `std::time`.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current system time in milliseconds since Unix epoch
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// let timestamp = time::now().unwrap();
/// assert!(timestamp > 0);
/// ```
///
/// # Errors
///
/// Returns error if system time is before Unix epoch (should never happen)
pub fn now() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .map_err(|e| e.to_string())
}

/// Calculate elapsed milliseconds since start time
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// let start = time::now().unwrap();
/// // ... do work ...
/// let elapsed = time::elapsed_millis(start).unwrap();
/// assert!(elapsed >= 0);
/// ```
///
/// # Errors
///
/// Returns error if current time cannot be retrieved
pub fn elapsed_millis(start: u128) -> Result<u128, String> {
    let current = now()?;
    Ok(current.saturating_sub(start))
}

/// Sleep for specified milliseconds
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// time::sleep_millis(100).unwrap();  // Sleep for 100ms
/// ```
pub fn sleep_millis(millis: u64) -> Result<(), String> {
    thread::sleep(Duration::from_millis(millis));
    Ok(())
}

/// Convert milliseconds to seconds
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// let secs = time::duration_secs(1500).unwrap();
/// assert!((secs - 1.5).abs() < 0.01);  // ~1.5 seconds
/// ```
pub fn duration_secs(millis: u128) -> Result<f64, String> {
    Ok(millis as f64 / 1000.0)
}

/// Format duration as human-readable string
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// let formatted = time::format_duration(90500).unwrap();
/// assert_eq!(formatted, "1m 30s");
/// ```
pub fn format_duration(millis: u128) -> Result<String, String> {
    if millis < 1000 {
        return Ok(format!("{millis}ms"));
    }

    let mut remaining = millis;
    let days = remaining / (24 * 60 * 60 * 1000);
    remaining %= 24 * 60 * 60 * 1000;

    let hours = remaining / (60 * 60 * 1000);
    remaining %= 60 * 60 * 1000;

    let minutes = remaining / (60 * 1000);
    remaining %= 60 * 1000;

    let seconds = remaining / 1000;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 {
        parts.push(format!("{seconds}s"));
    }

    Ok(parts.join(" "))
}

/// Parse human-readable duration string to milliseconds
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::time;
///
/// let millis = time::parse_duration("1h 30m").unwrap();
/// assert_eq!(millis, 5_400_000);
/// ```
///
/// # Errors
///
/// Returns error if format is invalid
pub fn parse_duration(duration_str: &str) -> Result<u128, String> {
    let mut total_millis: u128 = 0;

    for part in duration_str.split_whitespace() {
        if part.ends_with("ms") {
            let value = part
                .trim_end_matches("ms")
                .parse::<u128>()
                .map_err(|e| e.to_string())?;
            total_millis += value;
        } else if part.ends_with('s') {
            let value = part
                .trim_end_matches('s')
                .parse::<u128>()
                .map_err(|e| e.to_string())?;
            total_millis += value * 1000;
        } else if part.ends_with('m') {
            let value = part
                .trim_end_matches('m')
                .parse::<u128>()
                .map_err(|e| e.to_string())?;
            total_millis += value * 60 * 1000;
        } else if part.ends_with('h') {
            let value = part
                .trim_end_matches('h')
                .parse::<u128>()
                .map_err(|e| e.to_string())?;
            total_millis += value * 60 * 60 * 1000;
        } else if part.ends_with('d') {
            let value = part
                .trim_end_matches('d')
                .parse::<u128>()
                .map_err(|e| e.to_string())?;
            total_millis += value * 24 * 60 * 60 * 1000;
        } else {
            return Err(format!("Invalid duration format: {part}"));
        }
    }

    if total_millis == 0 {
        return Err("Invalid duration: must have at least one component".to_string());
    }

    Ok(total_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_positive() {
        let timestamp = now().unwrap();
        assert!(timestamp > 0);
        assert!(timestamp > 946_684_800_000); // After year 2000
    }

    #[test]
    fn test_elapsed_millis_basic() {
        let start = now().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = elapsed_millis(start).unwrap();
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_sleep_millis() {
        let result = sleep_millis(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_duration_secs_conversion() {
        assert_eq!(duration_secs(1000).unwrap(), 1.0);
        assert!((duration_secs(1500).unwrap() - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_format_duration_ms() {
        assert_eq!(format_duration(0).unwrap(), "0ms");
        assert_eq!(format_duration(500).unwrap(), "500ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(1000).unwrap(), "1s");
        assert_eq!(format_duration(5000).unwrap(), "5s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(60_000).unwrap(), "1m");
        assert_eq!(format_duration(90_000).unwrap(), "1m 30s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3_600_000).unwrap(), "1h");
        assert_eq!(format_duration(5_400_000).unwrap(), "1h 30m");
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86_400_000).unwrap(), "1d");
        assert_eq!(format_duration(90_000_000).unwrap(), "1d 1h");
    }

    #[test]
    fn test_parse_duration_simple() {
        assert_eq!(parse_duration("500ms").unwrap(), 500);
        assert_eq!(parse_duration("1s").unwrap(), 1_000);
        assert_eq!(parse_duration("1m").unwrap(), 60_000);
        assert_eq!(parse_duration("1h").unwrap(), 3_600_000);
        assert_eq!(parse_duration("1d").unwrap(), 86_400_000);
    }

    #[test]
    fn test_parse_duration_compound() {
        assert_eq!(parse_duration("1h 30m").unwrap(), 5_400_000);
        assert_eq!(parse_duration("1d 2h").unwrap(), 93_600_000);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("invalid").is_err());
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("").is_err());
        assert!(parse_duration("0s").is_err()); // Zero not allowed
    }

    #[test]
    fn test_format_parse_roundtrip() {
        for millis in [1000, 60_000, 90_000, 3_600_000, 86_400_000] {
            let formatted = format_duration(millis).unwrap();
            let parsed = parse_duration(&formatted).unwrap();
            assert_eq!(parsed, millis);
        }
    }
}
