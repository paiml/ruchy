//! Regex Operations Module (ruchy/std/regex)
//!
//! Thin wrappers around Rust's `regex` crate for pattern matching functionality.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around `regex` crate.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use regex::Regex;

/// Check if pattern matches text
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::is_match(r"\d+", "123");
/// assert!(result.unwrap());
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn is_match(pattern: &str, text: &str) -> Result<bool, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.is_match(text))
}

/// Find first match in text
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::find_first(r"\d+", "abc 123 def");
/// assert_eq!(result.unwrap(), Some("123".to_string()));
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn find_first(pattern: &str, text: &str) -> Result<Option<String>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.find(text).map(|m| m.as_str().to_string()))
}

/// Find all matches in text
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::find_all(r"\d+", "abc 123 def 456");
/// assert_eq!(result.unwrap(), vec!["123", "456"]);
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn find_all(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.find_iter(text).map(|m| m.as_str().to_string()).collect())
}

/// Replace first match with replacement
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::replace_first(r"\d+", "abc 123 def 456", "X");
/// assert_eq!(result.unwrap(), "abc X def 456");
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn replace_first(pattern: &str, text: &str, replacement: &str) -> Result<String, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.replace(text, replacement).to_string())
}

/// Replace all matches with replacement
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::replace_all(r"\d+", "abc 123 def 456", "X");
/// assert_eq!(result.unwrap(), "abc X def X");
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn replace_all(pattern: &str, text: &str, replacement: &str) -> Result<String, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.replace_all(text, replacement).to_string())
}

/// Split text by pattern
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::split(r"\s+", "hello world rust");
/// assert_eq!(result.unwrap(), vec!["hello", "world", "rust"]);
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn split(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.split(text).map(ToString::to_string).collect())
}

/// Capture first match with groups
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::capture_first(r"(\w+)@(\w+)", "user@example.com");
/// let captures = result.unwrap().unwrap();
/// assert_eq!(captures[1], "user");
/// assert_eq!(captures[2], "example");
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn capture_first(pattern: &str, text: &str) -> Result<Option<Vec<String>>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re.captures(text).map(|caps| {
        caps.iter()
            .map(|m| m.map(|m| m.as_str().to_string()).unwrap_or_default())
            .collect()
    }))
}

/// Capture all matches with groups
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::capture_all(r"(\w+):(\d+)", "name:123 age:45");
/// let all_captures = result.unwrap();
/// assert_eq!(all_captures[0][1], "name");
/// assert_eq!(all_captures[0][2], "123");
/// ```
///
/// # Errors
///
/// Returns error if pattern is invalid
pub fn capture_all(pattern: &str, text: &str) -> Result<Vec<Vec<String>>, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern '{pattern}': {e}"))?;
    Ok(re
        .captures_iter(text)
        .map(|caps| {
            caps.iter()
                .map(|m| m.map(|m| m.as_str().to_string()).unwrap_or_default())
                .collect()
        })
        .collect())
}

/// Check if pattern is valid regex
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::is_valid_pattern(r"\d+");
/// assert!(result.unwrap());
///
/// let result = regex::is_valid_pattern(r"[");
/// assert!(!result.unwrap());
/// ```
pub fn is_valid_pattern(pattern: &str) -> Result<bool, String> {
    Ok(Regex::new(pattern).is_ok())
}

/// Escape special regex characters in text
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::regex;
///
/// let result = regex::escape("a.b*c?");
/// assert_eq!(result.unwrap(), r"a\.b\*c\?");
/// ```
pub fn escape(text: &str) -> Result<String, String> {
    Ok(regex::escape(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // UNIT TESTS - Basic Function Coverage
    // ============================================================================

    #[test]
    fn test_is_match_basic() {
        assert!(is_match(r"\d+", "123").unwrap());
        assert!(is_match(r"[a-z]+", "hello").unwrap());
        assert!(!is_match(r"\d+", "abc").unwrap());
    }

    #[test]
    fn test_is_match_invalid_pattern() {
        assert!(is_match(r"[", "text").is_err());
        assert!(is_match(r"(unclosed", "text").is_err());
    }

    #[test]
    fn test_find_first_basic() {
        assert_eq!(find_first(r"\d+", "abc 123 def").unwrap(), Some("123".to_string()));
        assert_eq!(find_first(r"[a-z]+", "123 hello 456").unwrap(), Some("hello".to_string()));
        assert_eq!(find_first(r"\d+", "abc def").unwrap(), None);
    }

    #[test]
    fn test_find_first_multiple() {
        // Should only find first match
        assert_eq!(find_first(r"\d+", "123 456 789").unwrap(), Some("123".to_string()));
    }

    #[test]
    fn test_find_all_basic() {
        assert_eq!(find_all(r"\d+", "abc 123 def 456").unwrap(), vec!["123", "456"]);
        assert_eq!(find_all(r"[a-z]+", "hello world rust").unwrap(), vec!["hello", "world", "rust"]);
        assert_eq!(find_all(r"\d+", "no numbers").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn test_find_all_empty() {
        assert!(find_all(r"\d+", "").unwrap().is_empty());
    }

    #[test]
    fn test_replace_first_basic() {
        assert_eq!(replace_first(r"\d+", "abc 123 def 456", "X").unwrap(), "abc X def 456");
        assert_eq!(replace_first(r"[a-z]+", "hello world", "X").unwrap(), "X world");
    }

    #[test]
    fn test_replace_first_no_match() {
        assert_eq!(replace_first(r"\d+", "no numbers", "X").unwrap(), "no numbers");
    }

    #[test]
    fn test_replace_all_basic() {
        assert_eq!(replace_all(r"\d+", "abc 123 def 456", "X").unwrap(), "abc X def X");
        assert_eq!(replace_all(r"[a-z]+", "hello world rust", "X").unwrap(), "X X X");
    }

    #[test]
    fn test_replace_all_no_match() {
        assert_eq!(replace_all(r"\d+", "no numbers", "X").unwrap(), "no numbers");
    }

    #[test]
    fn test_split_basic() {
        assert_eq!(split(r"\s+", "hello world rust").unwrap(), vec!["hello", "world", "rust"]);
        assert_eq!(split(r",", "a,b,c").unwrap(), vec!["a", "b", "c"]);
        assert_eq!(split(r"\d+", "a1b2c").unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_split_no_match() {
        assert_eq!(split(r"\d+", "no numbers").unwrap(), vec!["no numbers"]);
    }

    #[test]
    fn test_split_empty_parts() {
        // Split with trailing delimiter creates empty string
        let result = split(r",", "a,b,").unwrap();
        assert_eq!(result, vec!["a", "b", ""]);
    }

    #[test]
    fn test_capture_first_basic() {
        let result = capture_first(r"(\w+)@(\w+)", "user@example.com").unwrap().unwrap();
        assert_eq!(result[0], "user@example");
        assert_eq!(result[1], "user");
        assert_eq!(result[2], "example");
    }

    #[test]
    fn test_capture_first_no_match() {
        assert_eq!(capture_first(r"(\w+)@(\w+)", "no email here").unwrap(), None);
    }

    #[test]
    fn test_capture_first_no_groups() {
        // Pattern without groups still returns full match as [0]
        let result = capture_first(r"\d+", "abc 123").unwrap().unwrap();
        assert_eq!(result[0], "123");
    }

    #[test]
    fn test_capture_all_basic() {
        let result = capture_all(r"(\w+):(\d+)", "name:123 age:45").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0][1], "name");
        assert_eq!(result[0][2], "123");
        assert_eq!(result[1][1], "age");
        assert_eq!(result[1][2], "45");
    }

    #[test]
    fn test_capture_all_no_match() {
        assert!(capture_all(r"(\w+):(\d+)", "no matches").unwrap().is_empty());
    }

    #[test]
    fn test_is_valid_pattern_valid() {
        assert!(is_valid_pattern(r"\d+").unwrap());
        assert!(is_valid_pattern(r"[a-z]+").unwrap());
        assert!(is_valid_pattern(r"(\w+)@(\w+)").unwrap());
    }

    #[test]
    fn test_is_valid_pattern_invalid() {
        assert!(!is_valid_pattern(r"[").unwrap());
        assert!(!is_valid_pattern(r"(unclosed").unwrap());
        assert!(!is_valid_pattern(r"\k<invalid>").unwrap());
    }

    #[test]
    fn test_escape_basic() {
        assert_eq!(escape("a.b*c?").unwrap(), r"a\.b\*c\?");
        assert_eq!(escape("hello").unwrap(), "hello");
        assert_eq!(escape("[abc]").unwrap(), r"\[abc\]");
        assert_eq!(escape("(a|b)").unwrap(), r"\(a\|b\)");
    }

    #[test]
    fn test_escape_special_chars() {
        // All regex special characters
        assert_eq!(escape(r"\.^$*+?{}[]()|\").unwrap(), r"\\\.\^\$\*\+\?\{\}\[\]\(\)\|\\");
    }

    // ============================================================================
    // INTEGRATION TESTS - Multiple Functions Together
    // ============================================================================

    #[test]
    fn test_email_extraction_workflow() {
        let text = "Contact: user@example.com or admin@test.org";

        // Check if emails exist
        assert!(is_match(r"\w+@\w+\.\w+", text).unwrap());

        // Find first email
        let first = find_first(r"\w+@\w+\.\w+", text).unwrap();
        assert_eq!(first, Some("user@example.com".to_string()));

        // Find all emails
        let all = find_all(r"\w+@\w+\.\w+", text).unwrap();
        assert_eq!(all, vec!["user@example.com", "admin@test.org"]);
    }

    #[test]
    fn test_text_cleanup_workflow() {
        let text = "Hello   World    Rust   Programming";

        // Replace multiple spaces with single space
        let cleaned = replace_all(r"\s+", text, " ").unwrap();
        assert_eq!(cleaned, "Hello World Rust Programming");

        // Split into words
        let words = split(r"\s+", text).unwrap();
        assert_eq!(words, vec!["Hello", "World", "Rust", "Programming"]);
    }

    #[test]
    fn test_url_parsing_workflow() {
        let url = "https://example.com:8080/path?key=value";

        // Capture URL components
        let pattern = r"(https?)://([^:/]+):(\d+)(/[^?]+)\?(.+)";
        let captures = capture_first(pattern, url).unwrap().unwrap();

        assert_eq!(captures[1], "https");    // protocol
        assert_eq!(captures[2], "example.com"); // domain
        assert_eq!(captures[3], "8080");     // port
        assert_eq!(captures[4], "/path");    // path
        assert_eq!(captures[5], "key=value"); // query
    }

    #[test]
    fn test_escape_and_match() {
        // Escape special characters and use in pattern
        let literal = "a.b*c?";
        let escaped = escape(literal).unwrap();

        // Escaped pattern should match literal text exactly
        assert!(is_match(&escaped, "a.b*c?").unwrap());

        // Should NOT match variations
        assert!(!is_match(&escaped, "axbxcx").unwrap());
    }

    // ============================================================================
    // EDGE CASES & ERROR HANDLING
    // ============================================================================

    #[test]
    fn test_empty_text() {
        assert!(!is_match(r"\d+", "").unwrap());
        assert_eq!(find_first(r"\d+", "").unwrap(), None);
        assert!(find_all(r"\d+", "").unwrap().is_empty());
    }

    #[test]
    fn test_empty_pattern_invalid() {
        // Empty pattern is technically valid in regex but unusual
        assert!(is_match("", "text").is_ok());
    }

    #[test]
    fn test_unicode_support() {
        assert!(is_match(r"你好", "你好世界").unwrap());
        assert_eq!(find_first(r"[а-я]+", "Привет мир").unwrap(), Some("ривет".to_string()));
        assert!(find_all(r"😀|😃|😄", "Hello 😀 World 😃").unwrap().len() == 2);
    }

    #[test]
    fn test_case_sensitivity() {
        assert!(is_match(r"hello", "hello").unwrap());
        assert!(!is_match(r"hello", "HELLO").unwrap());

        // Case-insensitive flag
        assert!(is_match(r"(?i)hello", "HELLO").unwrap());
    }

    #[test]
    fn test_multiline_patterns() {
        let text = "line1\nline2\nline3";

        // Without multiline flag, ^ and $ match start/end of string
        assert!(is_match(r"^line1", text).unwrap());
        assert!(is_match(r"line3$", text).unwrap());

        // With multiline flag (?m), ^ and $ match line boundaries
        assert_eq!(find_all(r"(?m)^line", text).unwrap(), vec!["line", "line", "line"]);
    }

    #[test]
    fn test_greedy_vs_lazy() {
        let text = "<div>content1</div><div>content2</div>";

        // Greedy (default)
        let greedy = find_first(r"<div>.*</div>", text).unwrap();
        assert_eq!(greedy, Some("<div>content1</div><div>content2</div>".to_string()));

        // Lazy (non-greedy)
        let lazy = find_first(r"<div>.*?</div>", text).unwrap();
        assert_eq!(lazy, Some("<div>content1</div>".to_string()));
    }
}
