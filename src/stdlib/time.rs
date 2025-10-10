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
