//! Helper functions for working with Object and `ObjectMut` values
//! All functions maintain ≤10 complexity budget following Toyota Way

use super::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Check if value is a mutable object
/// Complexity: 1
#[inline]
pub fn is_mutable_object(value: &Value) -> bool {
    matches!(value, Value::ObjectMut(_))
}

/// Check if value is any kind of object (mutable or immutable)
/// Complexity: 2
#[inline]
pub fn is_object(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::ObjectMut(_))
}

/// Get field from object (handles both Object and `ObjectMut`)
///
/// # Complexity
/// Cyclomatic complexity: 5
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key".to_string(), Value::Integer(42));
/// let obj = object_helpers::new_mutable_object(map);
///
/// assert_eq!(
///     object_helpers::get_object_field(&obj, "key"),
///     Some(Value::Integer(42))
/// );
/// ```
pub fn get_object_field(value: &Value, field: &str) -> Option<Value> {
    match value {
        Value::Object(map) => map.get(field).cloned(),
        Value::ObjectMut(cell) => cell.lock().unwrap().get(field).cloned(),
        _ => None,
    }
}

/// Set field in mutable object (returns error for immutable)
///
/// # Complexity
/// Cyclomatic complexity: 7
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key".to_string(), Value::Integer(42));
/// let obj = object_helpers::new_mutable_object(map);
///
/// // Mutate the object
/// assert!(object_helpers::set_object_field(&obj, "key", Value::Integer(99)).is_ok());
/// assert_eq!(
///     object_helpers::get_object_field(&obj, "key"),
///     Some(Value::Integer(99))
/// );
/// ```
pub fn set_object_field(
    value: &Value,
    field: &str,
    new_value: Value,
) -> Result<(), InterpreterError> {
    match value {
        Value::Object(_) => Err(InterpreterError::RuntimeError(format!(
            "Cannot mutate immutable object field '{field}'"
        ))),
        Value::ObjectMut(cell) => {
            cell.lock().unwrap().insert(field.to_string(), new_value);
            Ok(())
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot access field '{field}' on non-object"
        ))),
    }
}

/// Create new mutable object from `HashMap`
///
/// # Complexity
/// Cyclomatic complexity: 2
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), Value::Integer(10));
/// let obj = object_helpers::new_mutable_object(map);
///
/// assert!(object_helpers::is_mutable_object(&obj));
/// ```
#[inline]
pub fn new_mutable_object(map: HashMap<String, Value>) -> Value {
    Value::ObjectMut(Arc::new(Mutex::new(map)))
}

/// Create new immutable object from `HashMap`
///
/// # Complexity
/// Cyclomatic complexity: 2
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), Value::Integer(10));
/// let obj = object_helpers::new_immutable_object(map);
///
/// assert!(!object_helpers::is_mutable_object(&obj));
/// ```
#[inline]
pub fn new_immutable_object(map: HashMap<String, Value>) -> Value {
    Value::Object(Arc::new(map))
}

/// Convert immutable Object to mutable `ObjectMut` (copies data)
///
/// # Complexity
/// Cyclomatic complexity: 4
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), Value::Integer(10));
/// let immutable = object_helpers::new_immutable_object(map);
///
/// let mutable = object_helpers::to_mutable(&immutable);
/// assert!(object_helpers::is_mutable_object(&mutable));
/// ```
pub fn to_mutable(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::ObjectMut(Arc::new(Mutex::new((**map).clone()))),
        Value::ObjectMut(_) => value.clone(),
        _ => value.clone(),
    }
}

/// Convert mutable `ObjectMut` to immutable Object (copies data)
///
/// # Complexity
/// Cyclomatic complexity: 4
///
/// # Examples
///
/// ```
/// use ruchy::runtime::object_helpers;
/// use ruchy::runtime::interpreter::Value;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), Value::Integer(10));
/// let mutable = object_helpers::new_mutable_object(map);
///
/// let immutable = object_helpers::to_immutable(&mutable);
/// assert!(!object_helpers::is_mutable_object(&immutable));
/// ```
pub fn to_immutable(value: &Value) -> Value {
    match value {
        Value::ObjectMut(cell) => Value::Object(Arc::new(cell.lock().unwrap().clone())),
        Value::Object(_) => value.clone(),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_mutable_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        assert!(!is_mutable_object(&immutable));
        assert!(is_mutable_object(&mutable));
    }

    #[test]
    fn test_is_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        assert!(is_object(&immutable));
        assert!(is_object(&mutable));
        assert!(!is_object(&Value::Integer(42)));
    }

    #[test]
    fn test_get_object_field() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        assert_eq!(
            get_object_field(&immutable, "key"),
            Some(Value::Integer(42))
        );
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(42)));
        assert_eq!(get_object_field(&immutable, "missing"), None);
    }

    #[test]
    fn test_set_object_field() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map.clone());
        let mutable = new_mutable_object(map);

        // Immutable should fail
        assert!(set_object_field(&immutable, "key", Value::Integer(99)).is_err());

        // Mutable should succeed
        assert!(set_object_field(&mutable, "key", Value::Integer(99)).is_ok());
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(99)));
    }

    #[test]
    fn test_set_new_field() {
        let map = HashMap::new();
        let mutable = new_mutable_object(map);

        // Add new field
        assert!(set_object_field(&mutable, "new_key", Value::Integer(123)).is_ok());
        assert_eq!(
            get_object_field(&mutable, "new_key"),
            Some(Value::Integer(123))
        );
    }

    #[test]
    fn test_to_mutable() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let immutable = new_immutable_object(map);
        let mutable = to_mutable(&immutable);

        assert!(is_mutable_object(&mutable));
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(42)));

        // Should be able to mutate it now
        assert!(set_object_field(&mutable, "key", Value::Integer(99)).is_ok());
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(99)));
    }

    #[test]
    fn test_to_immutable() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let mutable = new_mutable_object(map);
        let immutable = to_immutable(&mutable);

        assert!(!is_mutable_object(&immutable));
        assert_eq!(
            get_object_field(&immutable, "key"),
            Some(Value::Integer(42))
        );
    }

    #[test]
    fn test_to_mutable_idempotent() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::Integer(42));

        let mutable1 = new_mutable_object(map);
        let mutable2 = to_mutable(&mutable1);

        // Converting already mutable object should return clone
        assert!(is_mutable_object(&mutable2));
    }

    #[test]
    fn test_non_object_type() {
        let int_val = Value::Integer(42);

        // Getting field from non-object returns None
        assert_eq!(get_object_field(&int_val, "key"), None);

        // Setting field on non-object returns error
        assert!(set_object_field(&int_val, "key", Value::Integer(99)).is_err());
    }
}

#[test]
fn test_to_immutable_object_match_arm() {
    // MISSED: delete match arm Value::Object(_) in to_immutable (line 189)
    use std::sync::Arc;

    let mut map = HashMap::new();
    map.insert("test".to_string(), Value::Integer(42));

    let immutable_obj = Value::Object(Arc::new(map));
    let result = to_immutable(&immutable_obj);

    // Should return clone of immutable object (match arm test)
    if let Value::Object(obj) = result {
        assert_eq!(obj.get("test"), Some(&Value::Integer(42)));
    } else {
        panic!("Should return Object variant");
    }
}
