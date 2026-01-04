//! JSON Module (STD-003)
//!
//! Thin wrappers around `serde_json` for Ruchy-friendly API.
//!
//! # Examples
//!
//! ```
//! use ruchy::stdlib::json;
//!
//! // Parse JSON string
//! let value = json::parse(r#"{"name": "Alice", "age": 30}"#)?;
//!
//! // Access fields
//! let name = json::get(&value, "name");
//! let name_str = json::as_string(name.unwrap());
//!
//! // Stringify back to JSON
//! let json_str = json::stringify(&value)?;
//!
//! // Pretty print
//! let pretty = json::pretty(&value)?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
pub use serde_json::Value;

/// Parse a JSON string into a Value
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#"{"key": "value"}"#)?;
/// assert!(value.is_object());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the JSON string is invalid
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn parse(json_str: &str) -> Result<Value> {
    serde_json::from_str(json_str).context("Failed to parse JSON")
}

/// Convert a Value to a JSON string
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#"{"key": "value"}"#)?;
/// let json_str = json::stringify(&value)?;
/// assert!(json_str.contains("key"));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the value cannot be serialized
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn stringify(value: &Value) -> Result<String> {
    serde_json::to_string(value).context("Failed to stringify JSON")
}

/// Convert a Value to a pretty-printed JSON string
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#"{"key":"value"}"#)?;
/// let pretty = json::pretty(&value)?;
/// assert!(pretty.contains('\n'));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the value cannot be serialized
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
pub fn pretty(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).context("Failed to pretty-print JSON")
}

/// Get a field from a JSON object
///
/// Returns `None` if the value is not an object or the field doesn't exist.
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#"{"name": "Alice"}"#)?;
/// let name = json::get(&value, "name");
/// assert!(name.is_some());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn get<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.get(key)
}

/// Get a nested field from a JSON object using a path
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#"{"user": {"name": "Alice"}}"#)?;
/// let name = json::get_path(&value, &["user", "name"]);
/// assert!(name.is_some());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
#[must_use]
pub fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for key in path {
        current = current.get(key)?;
    }
    Some(current)
}

/// Get an element from a JSON array by index
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r"[10, 20, 30]")?;
/// let elem = json::get_index(&value, 1);
/// assert!(elem.is_some());
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn get_index(value: &Value, index: usize) -> Option<&Value> {
    value.get(index)
}

/// Convert a JSON value to a Rust string
///
/// Returns `None` if the value is not a string.
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse(r#""hello""#)?;
/// let s = json::as_string(&value);
/// assert_eq!(s, Some("hello"));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn as_string(value: &Value) -> Option<&str> {
    value.as_str()
}

/// Convert a JSON value to an i64
///
/// Returns `None` if the value is not a number or doesn't fit in i64.
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse("42")?;
/// let n = json::as_i64(&value);
/// assert_eq!(n, Some(42));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn as_i64(value: &Value) -> Option<i64> {
    value.as_i64()
}

/// Convert a JSON value to an f64
///
/// Returns `None` if the value is not a number.
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse("3.15")?;
/// let n = json::as_f64(&value);
/// assert_eq!(n, Some(3.15));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn as_f64(value: &Value) -> Option<f64> {
    value.as_f64()
}

/// Convert a JSON value to a bool
///
/// Returns `None` if the value is not a boolean.
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::json;
///
/// let value = json::parse("true")?;
/// let b = json::as_bool(&value);
/// assert_eq!(b, Some(true));
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Complexity
///
/// Complexity: 1 (within Toyota Way limits ≤10)
#[must_use]
pub fn as_bool(value: &Value) -> Option<bool> {
    value.as_bool()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== parse tests ====================

    #[test]
    fn test_parse_simple() {
        let result = parse(r#"{"key": "value"}"#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse("{invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_array() {
        let result = parse(r"[1, 2, 3]");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_array());
    }

    #[test]
    fn test_parse_number() {
        let result = parse("42");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_number());
    }

    #[test]
    fn test_parse_string() {
        let result = parse(r#""hello""#);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_string());
    }

    #[test]
    fn test_parse_bool() {
        let result = parse("true");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_boolean());
    }

    #[test]
    fn test_parse_null() {
        let result = parse("null");
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_null());
    }

    #[test]
    fn test_parse_nested() {
        let result = parse(r#"{"user": {"name": "Alice", "age": 30}}"#);
        assert!(result.is_ok());
    }

    // ==================== stringify tests ====================

    #[test]
    fn test_stringify_object() {
        let value = parse(r#"{"key": "value"}"#).unwrap();
        let result = stringify(&value);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("key"));
    }

    #[test]
    fn test_stringify_array() {
        let value = parse(r"[1, 2, 3]").unwrap();
        let result = stringify(&value);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains('1'));
        assert!(s.contains('2'));
        assert!(s.contains('3'));
    }

    #[test]
    fn test_stringify_primitive() {
        let value = parse("42").unwrap();
        let result = stringify(&value);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    // ==================== pretty tests ====================

    #[test]
    fn test_pretty_object() {
        let value = parse(r#"{"key":"value"}"#).unwrap();
        let result = pretty(&value);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains('\n'), "Pretty output should have newlines");
    }

    #[test]
    fn test_pretty_nested() {
        let value = parse(r#"{"user":{"name":"Alice"}}"#).unwrap();
        let result = pretty(&value);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains('\n'));
        assert!(s.contains("  ")); // Should have indentation
    }

    #[test]
    fn test_pretty_array() {
        let value = parse(r"[1,2,3]").unwrap();
        let result = pretty(&value);
        assert!(result.is_ok());
    }

    // ==================== get tests ====================

    #[test]
    fn test_get_existing_field() {
        let value = parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
        let name = get(&value, "name");
        assert!(name.is_some());
        assert_eq!(name.unwrap().as_str(), Some("Alice"));
    }

    #[test]
    fn test_get_missing_field() {
        let value = parse(r#"{"name": "Alice"}"#).unwrap();
        let missing = get(&value, "age");
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_from_non_object() {
        let value = parse("[1, 2, 3]").unwrap();
        let result = get(&value, "key");
        assert!(result.is_none());
    }

    // ==================== get_path tests ====================

    #[test]
    fn test_get_path_simple() {
        let value = parse(r#"{"user": {"name": "Alice"}}"#).unwrap();
        let name = get_path(&value, &["user", "name"]);
        assert!(name.is_some());
        assert_eq!(name.unwrap().as_str(), Some("Alice"));
    }

    #[test]
    fn test_get_path_deep() {
        let value = parse(r#"{"a": {"b": {"c": {"d": 42}}}}"#).unwrap();
        let d = get_path(&value, &["a", "b", "c", "d"]);
        assert!(d.is_some());
        assert_eq!(d.unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_get_path_missing() {
        let value = parse(r#"{"user": {"name": "Alice"}}"#).unwrap();
        let missing = get_path(&value, &["user", "age"]);
        assert!(missing.is_none());
    }

    #[test]
    fn test_get_path_empty() {
        let value = parse(r#"{"key": "value"}"#).unwrap();
        let result = get_path(&value, &[]);
        assert!(result.is_some()); // Returns the value itself
    }

    // ==================== get_index tests ====================

    #[test]
    fn test_get_index_valid() {
        let value = parse(r"[10, 20, 30]").unwrap();
        let elem = get_index(&value, 1);
        assert!(elem.is_some());
        assert_eq!(elem.unwrap().as_i64(), Some(20));
    }

    #[test]
    fn test_get_index_first() {
        let value = parse(r"[10, 20, 30]").unwrap();
        let elem = get_index(&value, 0);
        assert!(elem.is_some());
        assert_eq!(elem.unwrap().as_i64(), Some(10));
    }

    #[test]
    fn test_get_index_last() {
        let value = parse(r"[10, 20, 30]").unwrap();
        let elem = get_index(&value, 2);
        assert!(elem.is_some());
        assert_eq!(elem.unwrap().as_i64(), Some(30));
    }

    #[test]
    fn test_get_index_out_of_bounds() {
        let value = parse(r"[10, 20, 30]").unwrap();
        let elem = get_index(&value, 10);
        assert!(elem.is_none());
    }

    #[test]
    fn test_get_index_from_object() {
        let value = parse(r#"{"key": "value"}"#).unwrap();
        let elem = get_index(&value, 0);
        assert!(elem.is_none());
    }

    // ==================== as_string tests ====================

    #[test]
    fn test_as_string_valid() {
        let value = parse(r#""hello""#).unwrap();
        let s = as_string(&value);
        assert_eq!(s, Some("hello"));
    }

    #[test]
    fn test_as_string_from_number() {
        let value = parse("42").unwrap();
        let s = as_string(&value);
        assert!(s.is_none());
    }

    #[test]
    fn test_as_string_empty() {
        let value = parse(r#""""#).unwrap();
        let s = as_string(&value);
        assert_eq!(s, Some(""));
    }

    // ==================== as_i64 tests ====================

    #[test]
    fn test_as_i64_valid() {
        let value = parse("42").unwrap();
        let n = as_i64(&value);
        assert_eq!(n, Some(42));
    }

    #[test]
    fn test_as_i64_negative() {
        let value = parse("-100").unwrap();
        let n = as_i64(&value);
        assert_eq!(n, Some(-100));
    }

    #[test]
    fn test_as_i64_from_string() {
        let value = parse(r#""42""#).unwrap();
        let n = as_i64(&value);
        assert!(n.is_none());
    }

    #[test]
    fn test_as_i64_zero() {
        let value = parse("0").unwrap();
        let n = as_i64(&value);
        assert_eq!(n, Some(0));
    }

    // ==================== as_f64 tests ====================

    #[test]
    fn test_as_f64_valid() {
        let value = parse("3.15").unwrap();
        let n = as_f64(&value);
        assert_eq!(n, Some(3.15));
    }

    #[test]
    fn test_as_f64_integer() {
        let value = parse("42").unwrap();
        let n = as_f64(&value);
        assert_eq!(n, Some(42.0));
    }

    #[test]
    fn test_as_f64_negative() {
        let value = parse("-3.15").unwrap();
        let n = as_f64(&value);
        assert_eq!(n, Some(-3.15));
    }

    #[test]
    fn test_as_f64_from_string() {
        let value = parse(r#""3.15""#).unwrap();
        let n = as_f64(&value);
        assert!(n.is_none());
    }

    // ==================== as_bool tests ====================

    #[test]
    fn test_as_bool_true() {
        let value = parse("true").unwrap();
        let b = as_bool(&value);
        assert_eq!(b, Some(true));
    }

    #[test]
    fn test_as_bool_false() {
        let value = parse("false").unwrap();
        let b = as_bool(&value);
        assert_eq!(b, Some(false));
    }

    #[test]
    fn test_as_bool_from_number() {
        let value = parse("1").unwrap();
        let b = as_bool(&value);
        assert!(b.is_none()); // Numbers are not booleans
    }

    #[test]
    fn test_as_bool_from_string() {
        let value = parse(r#""true""#).unwrap();
        let b = as_bool(&value);
        assert!(b.is_none()); // Strings are not booleans
    }

    // ==================== roundtrip tests ====================

    #[test]
    fn test_roundtrip() {
        let original = r#"{"key":"value"}"#;
        let value = parse(original).unwrap();
        let stringified = stringify(&value).unwrap();
        let reparsed = parse(&stringified);
        assert!(reparsed.is_ok());
    }

    #[test]
    fn test_roundtrip_complex() {
        let original = r#"{"users":[{"name":"Alice","age":30},{"name":"Bob","age":25}],"count":2}"#;
        let value = parse(original).unwrap();
        let stringified = stringify(&value).unwrap();
        let reparsed = parse(&stringified).unwrap();

        // Verify content is preserved
        let users = get(&reparsed, "users");
        assert!(users.is_some());
    }
}
