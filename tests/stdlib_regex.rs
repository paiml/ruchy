//! EXTREME TDD Coverage Tests for stdlib::regex Module
//!
//! Target: 0% → 80% coverage (+52 lines)
//! Protocol: RED → GREEN → REFACTOR → VALIDATE
//! Quality: Property tests + mutation tests ≥75%

use ruchy::stdlib::regex;

// ============================================================================
// UNIT TESTS (Basic Function Coverage)
// ============================================================================

#[test]
fn test_is_match_basic() {
    assert!(regex::is_match(r"\d+", "123").unwrap());
    assert!(regex::is_match(r"[a-z]+", "hello").unwrap());
    assert!(!regex::is_match(r"\d+", "abc").unwrap());
}

#[test]
fn test_is_match_invalid_pattern() {
    let result = regex::is_match(r"[", "test");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid regex pattern"));
}

#[test]
fn test_find_first_basic() {
    assert_eq!(regex::find_first(r"\d+", "abc 123 def").unwrap(), Some("123".to_string()));
    assert_eq!(regex::find_first(r"[a-z]+", "123 abc 456").unwrap(), Some("abc".to_string()));
    assert_eq!(regex::find_first(r"\d+", "no numbers").unwrap(), None);
}

#[test]
fn test_find_all_basic() {
    assert_eq!(regex::find_all(r"\d+", "123 456 789").unwrap(), vec!["123", "456", "789"]);
    assert_eq!(regex::find_all(r"[a-z]+", "hello world").unwrap(), vec!["hello", "world"]);
    assert_eq!(regex::find_all(r"\d+", "no numbers").unwrap(), Vec::<String>::new());
}

#[test]
fn test_replace_first_basic() {
    assert_eq!(regex::replace_first(r"\d+", "abc 123 def 456", "X").unwrap(), "abc X def 456");
    assert_eq!(regex::replace_first(r"[a-z]+", "hello world", "X").unwrap(), "X world");
}

#[test]
fn test_replace_all_basic() {
    assert_eq!(regex::replace_all(r"\d+", "abc 123 def 456", "X").unwrap(), "abc X def X");
    assert_eq!(regex::replace_all(r"[a-z]+", "hello world", "X").unwrap(), "X X");
}

#[test]
fn test_split_basic() {
    assert_eq!(regex::split(r"\s+", "hello world rust").unwrap(), vec!["hello", "world", "rust"]);
    assert_eq!(regex::split(r",", "a,b,c").unwrap(), vec!["a", "b", "c"]);
    assert_eq!(regex::split(r"\d+", "abc123def456ghi").unwrap(), vec!["abc", "def", "ghi"]);
}

#[test]
fn test_capture_first_basic() {
    let result = regex::capture_first(r"(\w+)@(\w+)", "user@example.com").unwrap().unwrap();
    assert_eq!(result[1], "user");
    assert_eq!(result[2], "example");

    let no_match = regex::capture_first(r"(\d+)", "no numbers").unwrap();
    assert!(no_match.is_none());
}

#[test]
fn test_capture_all_basic() {
    let result = regex::capture_all(r"(\w+):(\d+)", "name:123 age:45").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0][1], "name");
    assert_eq!(result[0][2], "123");
    assert_eq!(result[1][1], "age");
    assert_eq!(result[1][2], "45");
}

#[test]
fn test_is_valid_pattern() {
    assert!(regex::is_valid_pattern(r"\d+").unwrap());
    assert!(regex::is_valid_pattern(r"[a-z]+").unwrap());
    assert!(!regex::is_valid_pattern(r"[").unwrap());
    assert!(!regex::is_valid_pattern(r"(?P<unclosed").unwrap());
}

#[test]
fn test_escape_basic() {
    assert_eq!(regex::escape("a.b*c?").unwrap(), r"a\.b\*c\?");
    assert_eq!(regex::escape("hello").unwrap(), "hello");
    assert_eq!(regex::escape("[test]").unwrap(), r"\[test\]");
}

// ============================================================================
// PROPERTY-BASED TESTS (High Coverage per Test)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn property_is_match_consistency(
        text in "[a-zA-Z0-9 ]{1,50}"
    ) {
        // Property: Simple patterns should work consistently
        let has_digit = regex::is_match(r"\d", &text).unwrap();
        let digit_count = text.chars().filter(|c| c.is_numeric()).count();

        prop_assert_eq!(has_digit, digit_count > 0);
    }

    #[test]
    fn property_escape_makes_literal(
        text in "[ -~]{1,20}" // Printable ASCII
    ) {
        // Property: Escaped text should match literally
        let escaped = regex::escape(&text).unwrap();
        prop_assert!(regex::is_match(&escaped, &text).unwrap());
    }

    #[test]
    fn property_find_all_count(
        count in 0usize..10
    ) {
        // Property: find_all returns correct count
        let text = "x ".repeat(count) + "y";
        let matches = regex::find_all(r"x", &text).unwrap();

        prop_assert_eq!(matches.len(), count);
    }

    #[test]
    fn property_replace_all_removes(
        text in "[a-z ]{5,20}"
    ) {
        // Property: Replacing all letters with empty removes them
        let result = regex::replace_all(r"[a-z]", &text, "").unwrap();

        prop_assert!(result.chars().all(|c| !c.is_alphabetic()));
    }
}

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_empty_patterns() {
    // Empty pattern should work
    assert!(regex::is_match("", "test").unwrap());
    assert_eq!(regex::find_first("", "test").unwrap(), Some("".to_string()));
}

#[test]
fn test_empty_text() {
    assert!(!regex::is_match(r"\d+", "").unwrap());
    assert_eq!(regex::find_first(r"\d+", "").unwrap(), None);
    assert_eq!(regex::find_all(r"\d+", "").unwrap(), Vec::<String>::new());
}

#[test]
fn test_unicode_patterns() {
    assert!(regex::is_match(r"世界", "Hello 世界").unwrap());
    assert_eq!(regex::find_first(r"[一-龥]+", "中文 test 文字").unwrap(), Some("中文".to_string()));
}

#[test]
fn test_complex_patterns() {
    // Email pattern
    let email_pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
    assert!(regex::is_match(email_pattern, "test@example.com").unwrap());

    // URL pattern
    let url_pattern = r"https?://[^\s]+";
    assert!(regex::is_match(url_pattern, "https://example.com/path").unwrap());
}

#[test]
fn test_capture_groups_edge_cases() {
    // No groups in pattern
    let result = regex::capture_first(r"\d+", "123").unwrap().unwrap();
    assert_eq!(result[0], "123"); // Full match
    assert_eq!(result.len(), 1);

    // Optional groups
    let result = regex::capture_first(r"(\d+)?", "abc").unwrap().unwrap();
    assert_eq!(result[1], ""); // Empty capture
}

#[test]
fn test_split_edge_cases() {
    // Multiple consecutive delimiters
    assert_eq!(regex::split(r"\s+", "a  b   c").unwrap(), vec!["a", "b", "c"]);

    // Leading/trailing delimiters
    let result = regex::split(r",", ",a,b,").unwrap();
    assert!(result[0].is_empty());
    assert!(result[result.len() - 1].is_empty());
}

#[test]
fn test_replace_special_replacement() {
    // Note: "$" in replacement has special meaning (capture group reference)
    // Testing actual behavior
    assert_eq!(regex::replace_all(r"\d+", "abc 123 def", "***").unwrap(), "abc *** def");
    assert_eq!(regex::replace_all(r"[a-z]+", "hello world", "XXX").unwrap(), "XXX XXX");
}

// ============================================================================
// INTEGRATION TESTS (Multiple Functions Together)
// ============================================================================

#[test]
fn test_regex_workflow() {
    let text = "Error: 404 - Page not found. Warning: 500 - Server error.";

    // Step 1: Check if pattern matches
    assert!(regex::is_match(r"\d+", text).unwrap());

    // Step 2: Find all error codes
    let codes = regex::find_all(r"\d+", text).unwrap();
    assert_eq!(codes, vec!["404", "500"]);

    // Step 3: Replace error codes
    let redacted = regex::replace_all(r"\d+", text, "XXX").unwrap();
    assert!(redacted.contains("XXX"));

    // Step 4: Split by punctuation
    let parts = regex::split(r"[.:]", text).unwrap();
    assert!(parts.len() > 2);
}

#[test]
fn test_email_extraction() {
    let text = "Contact us at support@example.com or sales@company.org";

    // Extract email pattern
    let pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";

    // Find all emails
    let emails = regex::find_all(pattern, text).unwrap();
    assert_eq!(emails.len(), 2);

    // Capture domain parts
    let domain_pattern = r"(\w+)@(\w+)\.(\w+)";
    let captures = regex::capture_all(domain_pattern, text).unwrap();
    assert_eq!(captures[0][1], "support");
    assert_eq!(captures[0][2], "example");
}

#[test]
fn test_sanitize_input() {
    let user_input = "Hello, <script>alert('xss')</script> World!";

    // Escape special chars for safe regex
    let escaped = regex::escape(user_input).unwrap();

    // Should match literally
    assert!(regex::is_match(&escaped, user_input).unwrap());

    // Remove HTML tags
    let sanitized = regex::replace_all(r"<[^>]+>", user_input, "").unwrap();
    assert!(!sanitized.contains("<script>"));
}

#[test]
fn test_validation_workflow() {
    // Step 1: Check if pattern is valid
    let pattern = r"\d{3}-\d{2}-\d{4}"; // SSN pattern
    assert!(regex::is_valid_pattern(pattern).unwrap());

    // Step 2: Validate input
    assert!(regex::is_match(pattern, "123-45-6789").unwrap());
    assert!(!regex::is_match(pattern, "invalid").unwrap());

    // Step 3: Extract if valid
    let ssn = "SSN: 123-45-6789";
    let found = regex::find_first(pattern, ssn).unwrap();
    assert_eq!(found, Some("123-45-6789".to_string()));
}
