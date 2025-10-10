//! STD-010: Regex Module Tests (ruchy/std/regex)
//!
//! Test suite for Regular Expression operations module.
//! Thin wrappers around `regex` crate for pattern matching functionality.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

// ===== Pattern Matching Tests =====

#[test]
fn test_std_010_is_match_basic() {
    // STD-010: Basic pattern matching

    let result = ruchy::stdlib::regex::is_match(r"\d+", "123");

    assert!(result.is_ok(), "is_match should succeed");
    assert!(result.unwrap(), "Should match digits");
}

#[test]
fn test_std_010_is_match_no_match() {
    // STD-010: Pattern doesn't match

    let result = ruchy::stdlib::regex::is_match(r"\d+", "abc");

    assert!(result.is_ok(), "is_match should succeed");
    assert!(!result.unwrap(), "Should not match");
}

#[test]
fn test_std_010_is_match_invalid_pattern() {
    // STD-010: Invalid regex pattern

    let result = ruchy::stdlib::regex::is_match(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

// ===== Find Operations Tests =====

#[test]
fn test_std_010_find_first_basic() {
    // STD-010: Find first match

    let result = ruchy::stdlib::regex::find_first(r"\d+", "abc 123 def 456");

    assert!(result.is_ok(), "find_first should succeed");
    assert_eq!(result.unwrap(), Some("123".to_string()));
}

#[test]
fn test_std_010_find_first_no_match() {
    // STD-010: No match found

    let result = ruchy::stdlib::regex::find_first(r"\d+", "abc");

    assert!(result.is_ok(), "find_first should succeed");
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_std_010_find_first_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::find_first(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

#[test]
fn test_std_010_find_all_basic() {
    // STD-010: Find all matches

    let result = ruchy::stdlib::regex::find_all(r"\d+", "abc 123 def 456");

    assert!(result.is_ok(), "find_all should succeed");
    let matches = result.unwrap();
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0], "123");
    assert_eq!(matches[1], "456");
}

#[test]
fn test_std_010_find_all_empty() {
    // STD-010: No matches

    let result = ruchy::stdlib::regex::find_all(r"\d+", "abc");

    assert!(result.is_ok(), "find_all should succeed");
    assert!(result.unwrap().is_empty(), "Should return empty vector");
}

#[test]
fn test_std_010_find_all_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::find_all(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

// ===== Replace Operations Tests =====

#[test]
fn test_std_010_replace_first_basic() {
    // STD-010: Replace first match

    let result = ruchy::stdlib::regex::replace_first(r"\d+", "abc 123 def 456", "X");

    assert!(result.is_ok(), "replace_first should succeed");
    assert_eq!(result.unwrap(), "abc X def 456");
}

#[test]
fn test_std_010_replace_first_no_match() {
    // STD-010: No match to replace

    let result = ruchy::stdlib::regex::replace_first(r"\d+", "abc", "X");

    assert!(result.is_ok(), "replace_first should succeed");
    assert_eq!(result.unwrap(), "abc");
}

#[test]
fn test_std_010_replace_first_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::replace_first(r"[", "test", "X");

    assert!(result.is_err(), "Invalid pattern should return error");
}

#[test]
fn test_std_010_replace_all_basic() {
    // STD-010: Replace all matches

    let result = ruchy::stdlib::regex::replace_all(r"\d+", "abc 123 def 456", "X");

    assert!(result.is_ok(), "replace_all should succeed");
    assert_eq!(result.unwrap(), "abc X def X");
}

#[test]
fn test_std_010_replace_all_no_match() {
    // STD-010: No matches to replace

    let result = ruchy::stdlib::regex::replace_all(r"\d+", "abc", "X");

    assert!(result.is_ok(), "replace_all should succeed");
    assert_eq!(result.unwrap(), "abc");
}

#[test]
fn test_std_010_replace_all_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::replace_all(r"[", "test", "X");

    assert!(result.is_err(), "Invalid pattern should return error");
}

// ===== Split Operations Tests =====

#[test]
fn test_std_010_split_basic() {
    // STD-010: Split by pattern

    let result = ruchy::stdlib::regex::split(r"\s+", "hello world rust");

    assert!(result.is_ok(), "split should succeed");
    let parts = result.unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts, vec!["hello", "world", "rust"]);
}

#[test]
fn test_std_010_split_no_match() {
    // STD-010: No match, returns original

    let result = ruchy::stdlib::regex::split(r"\d+", "hello");

    assert!(result.is_ok(), "split should succeed");
    assert_eq!(result.unwrap(), vec!["hello"]);
}

#[test]
fn test_std_010_split_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::split(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

// ===== Capture Groups Tests =====

#[test]
fn test_std_010_capture_first_basic() {
    // STD-010: Capture groups

    let result = ruchy::stdlib::regex::capture_first(r"(\w+)@(\w+)", "user@example.com");

    assert!(result.is_ok(), "capture_first should succeed");
    let captures = result.unwrap();
    assert!(captures.is_some());
    let caps = captures.unwrap();
    assert_eq!(caps.len(), 3); // Full match + 2 groups
    assert_eq!(caps[0], "user@example");
    assert_eq!(caps[1], "user");
    assert_eq!(caps[2], "example");
}

#[test]
fn test_std_010_capture_first_no_match() {
    // STD-010: No capture

    let result = ruchy::stdlib::regex::capture_first(r"(\d+)", "abc");

    assert!(result.is_ok(), "capture_first should succeed");
    assert_eq!(result.unwrap(), None);
}

#[test]
fn test_std_010_capture_first_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::capture_first(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

#[test]
fn test_std_010_capture_all_basic() {
    // STD-010: Capture all matches

    let result = ruchy::stdlib::regex::capture_all(r"(\w+):(\d+)", "name:123 age:45");

    assert!(result.is_ok(), "capture_all should succeed");
    let all_captures = result.unwrap();
    assert_eq!(all_captures.len(), 2);
    assert_eq!(all_captures[0], vec!["name:123", "name", "123"]);
    assert_eq!(all_captures[1], vec!["age:45", "age", "45"]);
}

#[test]
fn test_std_010_capture_all_empty() {
    // STD-010: No captures

    let result = ruchy::stdlib::regex::capture_all(r"(\d+)", "abc");

    assert!(result.is_ok(), "capture_all should succeed");
    assert!(result.unwrap().is_empty(), "Should return empty vector");
}

#[test]
fn test_std_010_capture_all_invalid() {
    // STD-010: Invalid pattern

    let result = ruchy::stdlib::regex::capture_all(r"[", "test");

    assert!(result.is_err(), "Invalid pattern should return error");
}

// ===== Utility Functions Tests =====

#[test]
fn test_std_010_is_valid_pattern_valid() {
    // STD-010: Valid pattern check

    let result = ruchy::stdlib::regex::is_valid_pattern(r"\d+");

    assert!(result.is_ok(), "is_valid_pattern should succeed");
    assert!(result.unwrap(), "Should be valid pattern");
}

#[test]
fn test_std_010_is_valid_pattern_invalid() {
    // STD-010: Invalid pattern check

    let result = ruchy::stdlib::regex::is_valid_pattern(r"[");

    assert!(result.is_ok(), "is_valid_pattern should succeed");
    assert!(!result.unwrap(), "Should be invalid pattern");
}

#[test]
fn test_std_010_escape_basic() {
    // STD-010: Escape special characters

    let result = ruchy::stdlib::regex::escape("a.b*c?");

    assert!(result.is_ok(), "escape should succeed");
    assert_eq!(result.unwrap(), r"a\.b\*c\?");
}

#[test]
fn test_std_010_escape_empty() {
    // STD-010: Empty string

    let result = ruchy::stdlib::regex::escape("");

    assert!(result.is_ok(), "escape should succeed");
    assert_eq!(result.unwrap(), "");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_010_regex_never_panics(text: String) {
            // Property: Regex functions never panic on any input

            let _ = ruchy::stdlib::regex::is_match(r"\d+", &text);
            let _ = ruchy::stdlib::regex::find_first(r"\w+", &text);
            let _ = ruchy::stdlib::regex::find_all(r"\w+", &text);
            let _ = ruchy::stdlib::regex::split(r"\s+", &text);
            // Should not panic
        }

        #[test]
        fn test_std_010_escape_roundtrip(text: String) {
            // Property: Escaped string can be used as literal pattern

            if let Ok(escaped) = ruchy::stdlib::regex::escape(&text) {
                let result = ruchy::stdlib::regex::is_match(&escaped, &text);
                prop_assert!(result.is_ok(), "Escaped pattern should be valid");
            }
        }

        #[test]
        fn test_std_010_invalid_pattern_consistent(pattern: String) {
            // Property: Invalid patterns always return errors (not panics)

            let result = ruchy::stdlib::regex::is_match(&pattern, "test");
            prop_assert!(result.is_ok() || result.is_err(), "Should not panic");
        }
    }
}
