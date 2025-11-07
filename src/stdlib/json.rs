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
/// let value = json::parse(r#"[10, 20, 30]"#)?;
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
    fn test_roundtrip() {
        let original = r#"{"key":"value"}"#;
        let value = parse(original).unwrap();
        let stringified = stringify(&value).unwrap();
        let reparsed = parse(&stringified);
        assert!(reparsed.is_ok());
    }
}
