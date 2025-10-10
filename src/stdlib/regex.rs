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
