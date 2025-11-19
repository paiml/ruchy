//! Integration tests for `stdlib::regex` module
//!
//! Target: 0% → 100% coverage for stdlib/regex.rs (269 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests from tests/ directory ARE tracked by llvm-cov.

use ruchy::stdlib::regex;

// ============================================================================
// BASIC FUNCTION TESTS
// ============================================================================

#[test]
fn test_regex_is_match_basic() {
    assert!(
        regex::is_match(r"\d+", "123").unwrap(),
        "\\d+ should match '123'"
    );
    assert!(
        regex::is_match(r"[a-z]+", "hello").unwrap(),
        "[a-z]+ should match 'hello'"
    );
    assert!(
        !regex::is_match(r"\d+", "abc").unwrap(),
        "\\d+ should not match 'abc'"
    );
}

#[test]
fn test_regex_is_match_invalid_pattern() {
    assert!(
        regex::is_match(r"[", "text").is_err(),
        "Unclosed bracket should return error"
    );
    assert!(
        regex::is_match(r"(unclosed", "text").is_err(),
        "Unclosed parenthesis should return error"
    );
}

#[test]
fn test_regex_find_first_basic() {
    assert_eq!(
        regex::find_first(r"\d+", "abc 123 def").unwrap(),
        Some("123".to_string()),
        "Should find first number"
    );
    assert_eq!(
        regex::find_first(r"[a-z]+", "123 hello 456").unwrap(),
        Some("hello".to_string()),
        "Should find first word"
    );
    assert_eq!(
        regex::find_first(r"\d+", "abc def").unwrap(),
        None,
        "Should return None when no match"
    );
}

#[test]
fn test_regex_find_first_multiple() {
    // Should only find first match
    assert_eq!(
        regex::find_first(r"\d+", "123 456 789").unwrap(),
        Some("123".to_string()),
        "Should find only first match"
    );
}

#[test]
fn test_regex_find_all_basic() {
    assert_eq!(
        regex::find_all(r"\d+", "abc 123 def 456").unwrap(),
        vec!["123", "456"],
        "Should find all numbers"
    );
    assert_eq!(
        regex::find_all(r"[a-z]+", "hello world rust").unwrap(),
        vec!["hello", "world", "rust"],
        "Should find all words"
    );
    assert_eq!(
        regex::find_all(r"\d+", "no numbers").unwrap(),
        Vec::<String>::new(),
        "Should return empty vector when no matches"
    );
}

#[test]
fn test_regex_find_all_empty() {
    assert!(
        regex::find_all(r"\d+", "").unwrap().is_empty(),
        "Should return empty vector for empty string"
    );
}

#[test]
fn test_regex_replace_first_basic() {
    assert_eq!(
        regex::replace_first(r"\d+", "abc 123 def 456", "X").unwrap(),
        "abc X def 456",
        "Should replace only first number"
    );
    assert_eq!(
        regex::replace_first(r"[a-z]+", "hello world", "X").unwrap(),
        "X world",
        "Should replace only first word"
    );
}

#[test]
fn test_regex_replace_first_no_match() {
    assert_eq!(
        regex::replace_first(r"\d+", "no numbers", "X").unwrap(),
        "no numbers",
        "Should return original when no match"
    );
}

#[test]
fn test_regex_replace_all_basic() {
    assert_eq!(
        regex::replace_all(r"\d+", "abc 123 def 456", "X").unwrap(),
        "abc X def X",
        "Should replace all numbers"
    );
    assert_eq!(
        regex::replace_all(r"[a-z]+", "hello world rust", "X").unwrap(),
        "X X X",
        "Should replace all words"
    );
}

#[test]
fn test_regex_replace_all_no_match() {
    assert_eq!(
        regex::replace_all(r"\d+", "no numbers", "X").unwrap(),
        "no numbers",
        "Should return original when no match"
    );
}

#[test]
fn test_regex_split_basic() {
    assert_eq!(
        regex::split(r"\s+", "hello world rust").unwrap(),
        vec!["hello", "world", "rust"],
        "Should split by whitespace"
    );
    assert_eq!(
        regex::split(r",", "a,b,c").unwrap(),
        vec!["a", "b", "c"],
        "Should split by comma"
    );
    assert_eq!(
        regex::split(r"\d+", "a1b2c").unwrap(),
        vec!["a", "b", "c"],
        "Should split by digits"
    );
}

#[test]
fn test_regex_split_no_match() {
    assert_eq!(
        regex::split(r"\d+", "no numbers").unwrap(),
        vec!["no numbers"],
        "Should return single element when no match"
    );
}

#[test]
fn test_regex_split_empty_parts() {
    // Split with trailing delimiter creates empty string
    let result = regex::split(r",", "a,b,").unwrap();
    assert_eq!(
        result,
        vec!["a", "b", ""],
        "Should handle trailing delimiter"
    );
}

#[test]
fn test_regex_capture_first_basic() {
    let result = regex::capture_first(r"(\w+)@(\w+)", "user@example.com")
        .unwrap()
        .unwrap();
    assert_eq!(result[0], "user@example", "Full match at index 0");
    assert_eq!(result[1], "user", "First capture group");
    assert_eq!(result[2], "example", "Second capture group");
}

#[test]
fn test_regex_capture_first_no_match() {
    assert_eq!(
        regex::capture_first(r"(\w+)@(\w+)", "no email here").unwrap(),
        None,
        "Should return None when no match"
    );
}

#[test]
fn test_regex_capture_first_no_groups() {
    // Pattern without groups still returns full match as [0]
    let result = regex::capture_first(r"\d+", "abc 123").unwrap().unwrap();
    assert_eq!(result[0], "123", "Full match without groups");
}

#[test]
fn test_regex_capture_all_basic() {
    let result = regex::capture_all(r"(\w+):(\d+)", "name:123 age:45").unwrap();
    assert_eq!(result.len(), 2, "Should find two matches");
    assert_eq!(result[0][1], "name", "First match, first group");
    assert_eq!(result[0][2], "123", "First match, second group");
    assert_eq!(result[1][1], "age", "Second match, first group");
    assert_eq!(result[1][2], "45", "Second match, second group");
}

#[test]
fn test_regex_capture_all_no_match() {
    assert!(
        regex::capture_all(r"(\w+):(\d+)", "no matches")
            .unwrap()
            .is_empty(),
        "Should return empty vector when no matches"
    );
}

#[test]
fn test_regex_is_valid_pattern_valid() {
    assert!(
        regex::is_valid_pattern(r"\d+").unwrap(),
        "\\d+ should be valid"
    );
    assert!(
        regex::is_valid_pattern(r"[a-z]+").unwrap(),
        "[a-z]+ should be valid"
    );
    assert!(
        regex::is_valid_pattern(r"(\w+)@(\w+)").unwrap(),
        "Email pattern should be valid"
    );
}

#[test]
fn test_regex_is_valid_pattern_invalid() {
    assert!(
        !regex::is_valid_pattern(r"[").unwrap(),
        "Unclosed bracket should be invalid"
    );
    assert!(
        !regex::is_valid_pattern(r"(unclosed").unwrap(),
        "Unclosed parenthesis should be invalid"
    );
    assert!(
        !regex::is_valid_pattern(r"\k<invalid>").unwrap(),
        "Invalid escape should be invalid"
    );
}

#[test]
fn test_regex_escape_basic() {
    assert_eq!(
        regex::escape("a.b*c?").unwrap(),
        r"a\.b\*c\?",
        "Should escape special characters"
    );
    assert_eq!(
        regex::escape("hello").unwrap(),
        "hello",
        "Should not modify regular characters"
    );
    assert_eq!(
        regex::escape("[abc]").unwrap(),
        r"\[abc\]",
        "Should escape brackets"
    );
    assert_eq!(
        regex::escape("(a|b)").unwrap(),
        r"\(a\|b\)",
        "Should escape parens and pipe"
    );
}

// ============================================================================
// WORKFLOW/INTEGRATION TESTS
// ============================================================================

#[test]
fn test_regex_email_extraction_workflow() {
    let text = "Contact: user@example.com or admin@test.org";

    // Check if emails exist
    assert!(
        regex::is_match(r"\w+@\w+\.\w+", text).unwrap(),
        "Should match email pattern"
    );

    // Find first email
    let first = regex::find_first(r"\w+@\w+\.\w+", text).unwrap();
    assert_eq!(
        first,
        Some("user@example.com".to_string()),
        "Should find first email"
    );

    // Find all emails
    let all = regex::find_all(r"\w+@\w+\.\w+", text).unwrap();
    assert_eq!(
        all,
        vec!["user@example.com", "admin@test.org"],
        "Should find all emails"
    );
}

#[test]
fn test_regex_text_cleanup_workflow() {
    let text = "Hello   World    Rust   Programming";

    // Replace multiple spaces with single space
    let cleaned = regex::replace_all(r"\s+", text, " ").unwrap();
    assert_eq!(
        cleaned, "Hello World Rust Programming",
        "Should normalize whitespace"
    );

    // Split into words
    let words = regex::split(r"\s+", text).unwrap();
    assert_eq!(
        words,
        vec!["Hello", "World", "Rust", "Programming"],
        "Should split into words"
    );
}

#[test]
fn test_regex_url_parsing_workflow() {
    let url = "https://example.com:8080/path?key=value";

    // Capture URL components
    let pattern = r"(https?)://([^:/]+):(\d+)(/[^?]+)\?(.+)";
    let captures = regex::capture_first(pattern, url).unwrap().unwrap();

    assert_eq!(captures[1], "https", "Should capture protocol");
    assert_eq!(captures[2], "example.com", "Should capture domain");
    assert_eq!(captures[3], "8080", "Should capture port");
    assert_eq!(captures[4], "/path", "Should capture path");
    assert_eq!(captures[5], "key=value", "Should capture query");
}

#[test]
fn test_regex_escape_and_match() {
    // Escape special characters and use in pattern
    let literal = "a.b*c?";
    let escaped = regex::escape(literal).unwrap();

    // Escaped pattern should match literal text exactly
    assert!(
        regex::is_match(&escaped, "a.b*c?").unwrap(),
        "Escaped pattern should match literal"
    );

    // Should NOT match variations
    assert!(
        !regex::is_match(&escaped, "axbxcx").unwrap(),
        "Escaped pattern should not match variations"
    );
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_regex_empty_text() {
    assert!(
        !regex::is_match(r"\d+", "").unwrap(),
        "Should not match empty string"
    );
    assert_eq!(
        regex::find_first(r"\d+", "").unwrap(),
        None,
        "Should return None for empty string"
    );
    assert!(
        regex::find_all(r"\d+", "").unwrap().is_empty(),
        "Should return empty vector for empty string"
    );
}

#[test]
fn test_regex_empty_pattern() {
    // Empty pattern is technically valid in regex
    assert!(
        regex::is_match("", "text").is_ok(),
        "Empty pattern should be valid"
    );
}

#[test]
fn test_regex_unicode_support() {
    assert!(
        regex::is_match(r"你好", "你好世界").unwrap(),
        "Should match Chinese characters"
    );
    assert_eq!(
        regex::find_first(r"[а-я]+", "Привет мир").unwrap(),
        Some("ривет".to_string()),
        "Should match Cyrillic characters"
    );
    assert_eq!(
        regex::find_all(r"😀|😃|😄", "Hello 😀 World 😃")
            .unwrap()
            .len(),
        2,
        "Should match emoji"
    );
}

#[test]
fn test_regex_case_sensitivity() {
    assert!(
        regex::is_match(r"hello", "hello").unwrap(),
        "Should match same case"
    );
    assert!(
        !regex::is_match(r"hello", "HELLO").unwrap(),
        "Should not match different case by default"
    );

    // Case-insensitive flag
    assert!(
        regex::is_match(r"(?i)hello", "HELLO").unwrap(),
        "Should match with case-insensitive flag"
    );
}

#[test]
fn test_regex_multiline_patterns() {
    let text = "line1\nline2\nline3";

    // Without multiline flag, ^ and $ match start/end of string
    assert!(
        regex::is_match(r"^line1", text).unwrap(),
        "Should match start of string"
    );
    assert!(
        regex::is_match(r"line3$", text).unwrap(),
        "Should match end of string"
    );

    // With multiline flag (?m), ^ and $ match line boundaries
    assert_eq!(
        regex::find_all(r"(?m)^line", text).unwrap(),
        vec!["line", "line", "line"],
        "Should match line boundaries with multiline flag"
    );
}

#[test]
fn test_regex_greedy_vs_lazy() {
    let text = "<div>content1</div><div>content2</div>";

    // Greedy (default)
    let greedy = regex::find_first(r"<div>.*</div>", text).unwrap();
    assert_eq!(
        greedy,
        Some("<div>content1</div><div>content2</div>".to_string()),
        "Greedy should match longest"
    );

    // Lazy (non-greedy)
    let lazy = regex::find_first(r"<div>.*?</div>", text).unwrap();
    assert_eq!(
        lazy,
        Some("<div>content1</div>".to_string()),
        "Lazy should match shortest"
    );
}

#[test]
fn test_regex_complete_workflow() {
    // Complete regex workflow: validate, search, extract, replace
    let text = "Error: code 404, Error: code 500, Warning: code 200";

    // 1. Validate pattern is correct
    let pattern = r"Error: code (\d+)";
    assert!(
        regex::is_valid_pattern(pattern).unwrap(),
        "Pattern should be valid"
    );

    // 2. Check if pattern matches
    assert!(
        regex::is_match(pattern, text).unwrap(),
        "Pattern should match text"
    );

    // 3. Find all error codes
    let codes = regex::capture_all(pattern, text).unwrap();
    assert_eq!(codes.len(), 2, "Should find 2 errors");
    assert_eq!(codes[0][1], "404", "First error code");
    assert_eq!(codes[1][1], "500", "Second error code");

    // 4. Replace errors with generic message
    let cleaned = regex::replace_all(pattern, text, "Error: [REDACTED]").unwrap();
    assert_eq!(
        cleaned, "Error: [REDACTED], Error: [REDACTED], Warning: code 200",
        "Should redact error codes"
    );
}
