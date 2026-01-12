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
        Value::ObjectMut(cell) => cell
            .lock()
            .expect("Mutex poisoned in get_object_field - indicates panic in another thread")
            .get(field)
            .cloned(),
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
            cell.lock()
                .expect("Mutex poisoned in set_object_field - indicates panic in another thread")
                .insert(field.to_string(), new_value);
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
        Value::ObjectMut(cell) => Value::Object(Arc::new(
            cell.lock()
                .expect("Mutex poisoned in to_immutable - indicates panic in another thread")
                .clone(),
        )),
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

// === EXTREME TDD Round 20 tests ===

#[test]
fn test_to_immutable_idempotent() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(42));

    let immutable1 = new_immutable_object(map);
    let immutable2 = to_immutable(&immutable1);

    // Converting already immutable object should return clone
    assert!(!is_mutable_object(&immutable2));
    assert_eq!(
        get_object_field(&immutable2, "key"),
        Some(Value::Integer(42))
    );
}

#[test]
fn test_to_mutable_non_object() {
    let int_val = Value::Integer(42);
    let result = to_mutable(&int_val);

    // Non-object values should be cloned as-is
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_to_immutable_non_object() {
    let float_val = Value::Float(3.14);
    let result = to_immutable(&float_val);

    // Non-object values should be cloned as-is
    assert_eq!(result, Value::Float(3.14));
}

#[test]
fn test_set_object_field_immutable_error_message() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(42));
    let immutable = new_immutable_object(map);

    let err = set_object_field(&immutable, "field", Value::Integer(1)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("immutable"));
    assert!(msg.contains("field"));
}

#[test]
fn test_set_object_field_non_object_error_message() {
    let int_val = Value::Integer(42);

    let err = set_object_field(&int_val, "foo", Value::Integer(1)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
    assert!(msg.contains("foo"));
}

#[test]
fn test_empty_mutable_object() {
    let mutable = new_mutable_object(HashMap::new());

    assert!(is_mutable_object(&mutable));
    assert!(is_object(&mutable));
    assert_eq!(get_object_field(&mutable, "any"), None);
}

#[test]
fn test_empty_immutable_object() {
    let immutable = new_immutable_object(HashMap::new());

    assert!(!is_mutable_object(&immutable));
    assert!(is_object(&immutable));
    assert_eq!(get_object_field(&immutable, "any"), None);
}

// === EXTREME TDD Round 27 - Coverage Push Tests ===

#[test]
fn test_is_mutable_object_all_value_types() {
    use std::sync::Arc;

    // Test against all major Value types
    assert!(!is_mutable_object(&Value::Integer(1)));
    assert!(!is_mutable_object(&Value::Float(1.0)));
    assert!(!is_mutable_object(&Value::Bool(true)));
    assert!(!is_mutable_object(&Value::Nil));
    assert!(!is_mutable_object(&Value::from_string("test".to_string())));
    assert!(!is_mutable_object(&Value::Array(Arc::from(vec![]))));
    assert!(!is_mutable_object(&Value::Tuple(Arc::from(vec![]))));

    // Only ObjectMut returns true
    let mutable = new_mutable_object(HashMap::new());
    assert!(is_mutable_object(&mutable));
}

#[test]
fn test_is_object_all_value_types() {
    use std::sync::Arc;

    // Non-object types
    assert!(!is_object(&Value::Integer(1)));
    assert!(!is_object(&Value::Float(1.0)));
    assert!(!is_object(&Value::Bool(true)));
    assert!(!is_object(&Value::Nil));
    assert!(!is_object(&Value::from_string("test".to_string())));
    assert!(!is_object(&Value::Array(Arc::from(vec![]))));
    assert!(!is_object(&Value::Tuple(Arc::from(vec![]))));

    // Object types
    let immutable = new_immutable_object(HashMap::new());
    let mutable = new_mutable_object(HashMap::new());
    assert!(is_object(&immutable));
    assert!(is_object(&mutable));
}

#[test]
fn test_get_object_field_multiple_fields() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), Value::Integer(1));
    map.insert("b".to_string(), Value::Integer(2));
    map.insert("c".to_string(), Value::Integer(3));

    let obj = new_immutable_object(map);

    assert_eq!(get_object_field(&obj, "a"), Some(Value::Integer(1)));
    assert_eq!(get_object_field(&obj, "b"), Some(Value::Integer(2)));
    assert_eq!(get_object_field(&obj, "c"), Some(Value::Integer(3)));
    assert_eq!(get_object_field(&obj, "d"), None);
}

#[test]
fn test_get_object_field_nested_values() {
    use std::sync::Arc;

    let mut inner = HashMap::new();
    inner.insert("inner_key".to_string(), Value::Integer(42));

    let mut outer = HashMap::new();
    outer.insert("nested".to_string(), Value::Object(Arc::new(inner)));

    let obj = new_mutable_object(outer);

    if let Some(Value::Object(nested_obj)) = get_object_field(&obj, "nested") {
        assert_eq!(nested_obj.get("inner_key"), Some(&Value::Integer(42)));
    } else {
        panic!("Expected nested object");
    }
}

#[test]
fn test_set_object_field_overwrite_multiple() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(0));

    let mutable = new_mutable_object(map);

    // Overwrite multiple times
    for i in 1..=10 {
        assert!(set_object_field(&mutable, "key", Value::Integer(i)).is_ok());
        assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(i)));
    }
}

#[test]
fn test_set_object_field_different_types() {
    let mutable = new_mutable_object(HashMap::new());

    // Set string value
    assert!(set_object_field(&mutable, "str", Value::from_string("hello".to_string())).is_ok());

    // Set bool value
    assert!(set_object_field(&mutable, "bool", Value::Bool(true)).is_ok());

    // Set nil value
    assert!(set_object_field(&mutable, "nil", Value::Nil).is_ok());

    // Verify all values
    if let Some(Value::String(s)) = get_object_field(&mutable, "str") {
        assert_eq!(s.as_ref(), "hello");
    } else {
        panic!("Expected string");
    }
    assert_eq!(get_object_field(&mutable, "bool"), Some(Value::Bool(true)));
    assert_eq!(get_object_field(&mutable, "nil"), Some(Value::Nil));
}

#[test]
fn test_to_mutable_preserves_all_fields() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), Value::Integer(1));
    map.insert("b".to_string(), Value::Integer(2));
    map.insert("c".to_string(), Value::Integer(3));

    let immutable = new_immutable_object(map);
    let mutable = to_mutable(&immutable);

    // All fields should be preserved
    assert_eq!(get_object_field(&mutable, "a"), Some(Value::Integer(1)));
    assert_eq!(get_object_field(&mutable, "b"), Some(Value::Integer(2)));
    assert_eq!(get_object_field(&mutable, "c"), Some(Value::Integer(3)));
}

#[test]
fn test_to_immutable_preserves_all_fields() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), Value::Integer(10));
    map.insert("y".to_string(), Value::Integer(20));

    let mutable = new_mutable_object(map);
    let immutable = to_immutable(&mutable);

    // All fields should be preserved
    assert_eq!(get_object_field(&immutable, "x"), Some(Value::Integer(10)));
    assert_eq!(get_object_field(&immutable, "y"), Some(Value::Integer(20)));
}

#[test]
fn test_to_mutable_creates_independent_copy() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(1));

    let immutable = new_immutable_object(map);
    let mutable = to_mutable(&immutable);

    // Modify the mutable copy
    assert!(set_object_field(&mutable, "key", Value::Integer(100)).is_ok());

    // Original immutable should be unchanged
    assert_eq!(get_object_field(&immutable, "key"), Some(Value::Integer(1)));
    assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(100)));
}

#[test]
fn test_to_immutable_creates_independent_copy() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(1));

    let mutable = new_mutable_object(map);
    let immutable = to_immutable(&mutable);

    // Modify the mutable original
    assert!(set_object_field(&mutable, "key", Value::Integer(100)).is_ok());

    // Immutable copy should be unchanged
    assert_eq!(get_object_field(&immutable, "key"), Some(Value::Integer(1)));
    assert_eq!(get_object_field(&mutable, "key"), Some(Value::Integer(100)));
}

#[test]
fn test_special_field_names() {
    let mutable = new_mutable_object(HashMap::new());

    // Empty string field name
    assert!(set_object_field(&mutable, "", Value::Integer(1)).is_ok());
    assert_eq!(get_object_field(&mutable, ""), Some(Value::Integer(1)));

    // Field name with spaces
    assert!(set_object_field(&mutable, "field with spaces", Value::Integer(2)).is_ok());
    assert_eq!(
        get_object_field(&mutable, "field with spaces"),
        Some(Value::Integer(2))
    );

    // Field name with special characters
    assert!(set_object_field(&mutable, "field_123", Value::Integer(3)).is_ok());
    assert_eq!(
        get_object_field(&mutable, "field_123"),
        Some(Value::Integer(3))
    );
}

// === EXTREME TDD Round 162 - Quick Wins Coverage Push ===

#[test]
fn test_unicode_field_names_r162() {
    let mutable = new_mutable_object(HashMap::new());

    // Japanese
    assert!(set_object_field(&mutable, "日本語", Value::Integer(1)).is_ok());
    assert_eq!(
        get_object_field(&mutable, "日本語"),
        Some(Value::Integer(1))
    );

    // Emoji
    assert!(set_object_field(&mutable, "🎉🚀", Value::Integer(2)).is_ok());
    assert_eq!(get_object_field(&mutable, "🎉🚀"), Some(Value::Integer(2)));

    // Arabic
    assert!(set_object_field(&mutable, "مرحبا", Value::Integer(3)).is_ok());
    assert_eq!(get_object_field(&mutable, "مرحبا"), Some(Value::Integer(3)));

    // Mixed script
    assert!(set_object_field(&mutable, "hello世界", Value::Integer(4)).is_ok());
    assert_eq!(
        get_object_field(&mutable, "hello世界"),
        Some(Value::Integer(4))
    );
}

#[test]
fn test_large_object_50_fields_r162() {
    let mut map = HashMap::new();
    for i in 0..50 {
        map.insert(format!("field_{i}"), Value::Integer(i));
    }

    let obj = new_mutable_object(map);

    for i in 0..50 {
        assert_eq!(
            get_object_field(&obj, &format!("field_{i}")),
            Some(Value::Integer(i))
        );
    }
}

#[test]
fn test_large_object_100_fields_r162() {
    let mut map = HashMap::new();
    for i in 0..100 {
        map.insert(format!("key_{i}"), Value::Integer(i * 2));
    }

    let immutable = new_immutable_object(map);

    for i in 0..100 {
        assert_eq!(
            get_object_field(&immutable, &format!("key_{i}")),
            Some(Value::Integer(i * 2))
        );
    }
}

#[test]
fn test_nested_mutable_in_immutable_r162() {
    use std::sync::Arc;

    let inner_mutable = new_mutable_object(HashMap::new());
    set_object_field(&inner_mutable, "inner", Value::Integer(42)).unwrap();

    let mut outer = HashMap::new();
    outer.insert("nested".to_string(), inner_mutable);

    let outer_immutable = new_immutable_object(outer);

    if let Some(nested) = get_object_field(&outer_immutable, "nested") {
        assert!(is_mutable_object(&nested));
        assert_eq!(get_object_field(&nested, "inner"), Some(Value::Integer(42)));
    } else {
        panic!("Expected nested object");
    }
}

#[test]
fn test_nested_immutable_in_mutable_r162() {
    use std::sync::Arc;

    let inner = new_immutable_object(HashMap::from([
        ("x".to_string(), Value::Integer(10)),
        ("y".to_string(), Value::Integer(20)),
    ]));

    let outer = new_mutable_object(HashMap::new());
    set_object_field(&outer, "inner", inner).unwrap();

    if let Some(nested) = get_object_field(&outer, "inner") {
        assert!(!is_mutable_object(&nested));
        assert_eq!(get_object_field(&nested, "x"), Some(Value::Integer(10)));
    } else {
        panic!("Expected nested object");
    }
}

#[test]
fn test_value_array_in_object_r162() {
    use std::sync::Arc;

    let arr = Value::Array(Arc::from(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]));

    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "arr", arr.clone()).unwrap();

    let retrieved = get_object_field(&obj, "arr").unwrap();
    if let Value::Array(elems) = retrieved {
        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0], Value::Integer(1));
        assert_eq!(elems[2], Value::Integer(3));
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_value_tuple_in_object_r162() {
    use std::sync::Arc;

    let tuple = Value::Tuple(Arc::from(vec![
        Value::from_string("hello".to_string()),
        Value::Integer(42),
    ]));

    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "tup", tuple.clone()).unwrap();

    let retrieved = get_object_field(&obj, "tup").unwrap();
    if let Value::Tuple(elems) = retrieved {
        assert_eq!(elems.len(), 2);
    } else {
        panic!("Expected tuple");
    }
}

#[test]
fn test_nil_value_storage_r162() {
    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "nil_field", Value::Nil).unwrap();

    assert_eq!(get_object_field(&obj, "nil_field"), Some(Value::Nil));
}

#[test]
fn test_bool_values_in_object_r162() {
    let obj = new_mutable_object(HashMap::new());

    set_object_field(&obj, "true_field", Value::Bool(true)).unwrap();
    set_object_field(&obj, "false_field", Value::Bool(false)).unwrap();

    assert_eq!(
        get_object_field(&obj, "true_field"),
        Some(Value::Bool(true))
    );
    assert_eq!(
        get_object_field(&obj, "false_field"),
        Some(Value::Bool(false))
    );
}

#[test]
fn test_float_values_in_object_r162() {
    let obj = new_mutable_object(HashMap::new());

    set_object_field(&obj, "pi", Value::Float(3.14159)).unwrap();
    set_object_field(&obj, "neg", Value::Float(-2.5)).unwrap();
    set_object_field(&obj, "zero", Value::Float(0.0)).unwrap();

    assert_eq!(get_object_field(&obj, "pi"), Some(Value::Float(3.14159)));
    assert_eq!(get_object_field(&obj, "neg"), Some(Value::Float(-2.5)));
    assert_eq!(get_object_field(&obj, "zero"), Some(Value::Float(0.0)));
}

#[test]
fn test_negative_integers_in_object_r162() {
    let obj = new_mutable_object(HashMap::new());

    set_object_field(&obj, "neg", Value::Integer(-100)).unwrap();
    set_object_field(&obj, "min", Value::Integer(i64::MIN)).unwrap();
    set_object_field(&obj, "max", Value::Integer(i64::MAX)).unwrap();

    assert_eq!(get_object_field(&obj, "neg"), Some(Value::Integer(-100)));
    assert_eq!(
        get_object_field(&obj, "min"),
        Some(Value::Integer(i64::MIN))
    );
    assert_eq!(
        get_object_field(&obj, "max"),
        Some(Value::Integer(i64::MAX))
    );
}

#[test]
fn test_string_values_in_object_r162() {
    let obj = new_mutable_object(HashMap::new());

    set_object_field(&obj, "empty", Value::from_string("".to_string())).unwrap();
    set_object_field(&obj, "short", Value::from_string("a".to_string())).unwrap();
    set_object_field(&obj, "long", Value::from_string("a".repeat(1000))).unwrap();

    if let Some(Value::String(s)) = get_object_field(&obj, "empty") {
        assert_eq!(s.as_ref(), "");
    }
    if let Some(Value::String(s)) = get_object_field(&obj, "long") {
        assert_eq!(s.len(), 1000);
    }
}

#[test]
fn test_field_overwrite_with_different_type_r162() {
    let obj = new_mutable_object(HashMap::new());

    // Start with integer
    set_object_field(&obj, "field", Value::Integer(42)).unwrap();
    assert_eq!(get_object_field(&obj, "field"), Some(Value::Integer(42)));

    // Overwrite with string
    set_object_field(&obj, "field", Value::from_string("hello".to_string())).unwrap();
    if let Some(Value::String(s)) = get_object_field(&obj, "field") {
        assert_eq!(s.as_ref(), "hello");
    } else {
        panic!("Expected string");
    }

    // Overwrite with bool
    set_object_field(&obj, "field", Value::Bool(true)).unwrap();
    assert_eq!(get_object_field(&obj, "field"), Some(Value::Bool(true)));

    // Overwrite with nil
    set_object_field(&obj, "field", Value::Nil).unwrap();
    assert_eq!(get_object_field(&obj, "field"), Some(Value::Nil));
}

#[test]
fn test_to_mutable_on_string_value_r162() {
    let string_val = Value::from_string("test".to_string());
    let result = to_mutable(&string_val);
    // Non-object should be cloned as-is
    if let Value::String(s) = result {
        assert_eq!(s.as_ref(), "test");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_to_immutable_on_array_value_r162() {
    use std::sync::Arc;

    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = to_immutable(&arr);
    // Non-object should be cloned as-is
    if let Value::Array(elems) = result {
        assert_eq!(elems.len(), 2);
    } else {
        panic!("Expected array");
    }
}

#[test]
fn test_to_mutable_on_bool_value_r162() {
    let bool_val = Value::Bool(false);
    let result = to_mutable(&bool_val);
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_to_immutable_on_bool_value_r162() {
    let bool_val = Value::Bool(true);
    let result = to_immutable(&bool_val);
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_to_mutable_on_nil_value_r162() {
    let nil_val = Value::Nil;
    let result = to_mutable(&nil_val);
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_to_immutable_on_nil_value_r162() {
    let nil_val = Value::Nil;
    let result = to_immutable(&nil_val);
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_to_mutable_on_tuple_value_r162() {
    use std::sync::Arc;

    let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1)]));
    let result = to_mutable(&tuple);
    if let Value::Tuple(elems) = result {
        assert_eq!(elems.len(), 1);
    } else {
        panic!("Expected tuple");
    }
}

#[test]
fn test_to_immutable_on_tuple_value_r162() {
    use std::sync::Arc;

    let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = to_immutable(&tuple);
    if let Value::Tuple(elems) = result {
        assert_eq!(elems.len(), 2);
    } else {
        panic!("Expected tuple");
    }
}

#[test]
fn test_is_mutable_object_on_nil_r162() {
    assert!(!is_mutable_object(&Value::Nil));
}

#[test]
fn test_is_object_on_nil_r162() {
    assert!(!is_object(&Value::Nil));
}

#[test]
fn test_get_field_from_array_r162() {
    use std::sync::Arc;

    let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
    assert_eq!(get_object_field(&arr, "0"), None);
}

#[test]
fn test_get_field_from_tuple_r162() {
    use std::sync::Arc;

    let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1)]));
    assert_eq!(get_object_field(&tuple, "0"), None);
}

#[test]
fn test_set_field_on_array_r162() {
    use std::sync::Arc;

    let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
    let err = set_object_field(&arr, "0", Value::Integer(2)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
}

#[test]
fn test_set_field_on_tuple_r162() {
    use std::sync::Arc;

    let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1)]));
    let err = set_object_field(&tuple, "0", Value::Integer(2)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
}

#[test]
fn test_set_field_on_string_r162() {
    let s = Value::from_string("test".to_string());
    let err = set_object_field(&s, "len", Value::Integer(4)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
}

#[test]
fn test_set_field_on_float_r162() {
    let f = Value::Float(3.14);
    let err = set_object_field(&f, "int_part", Value::Integer(3)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
}

#[test]
fn test_set_field_on_bool_r162() {
    let b = Value::Bool(true);
    let err = set_object_field(&b, "value", Value::Integer(1)).unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("non-object"));
}

#[test]
fn test_multiple_sequential_mutations_r162() {
    let obj = new_mutable_object(HashMap::new());

    for i in 0..20 {
        set_object_field(&obj, &format!("field_{i}"), Value::Integer(i)).unwrap();
    }

    for i in 0..20 {
        assert_eq!(
            get_object_field(&obj, &format!("field_{i}")),
            Some(Value::Integer(i))
        );
    }

    // Now update all fields
    for i in 0..20 {
        set_object_field(&obj, &format!("field_{i}"), Value::Integer(i * 10)).unwrap();
    }

    for i in 0..20 {
        assert_eq!(
            get_object_field(&obj, &format!("field_{i}")),
            Some(Value::Integer(i * 10))
        );
    }
}

#[test]
fn test_delete_field_by_setting_nil_r162() {
    let obj = new_mutable_object(HashMap::from([
        ("a".to_string(), Value::Integer(1)),
        ("b".to_string(), Value::Integer(2)),
    ]));

    // "Delete" by setting to nil
    set_object_field(&obj, "a", Value::Nil).unwrap();
    assert_eq!(get_object_field(&obj, "a"), Some(Value::Nil));
    // b should still exist
    assert_eq!(get_object_field(&obj, "b"), Some(Value::Integer(2)));
}

#[test]
fn test_field_name_with_newlines_r162() {
    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "field\nwith\nnewlines", Value::Integer(1)).unwrap();
    assert_eq!(
        get_object_field(&obj, "field\nwith\nnewlines"),
        Some(Value::Integer(1))
    );
}

#[test]
fn test_field_name_with_tabs_r162() {
    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "field\twith\ttabs", Value::Integer(2)).unwrap();
    assert_eq!(
        get_object_field(&obj, "field\twith\ttabs"),
        Some(Value::Integer(2))
    );
}

#[test]
fn test_field_name_with_quotes_r162() {
    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "field\"with\"quotes", Value::Integer(3)).unwrap();
    assert_eq!(
        get_object_field(&obj, "field\"with\"quotes"),
        Some(Value::Integer(3))
    );
}

#[test]
fn test_field_name_with_backslash_r162() {
    let obj = new_mutable_object(HashMap::new());
    set_object_field(&obj, "field\\with\\backslash", Value::Integer(4)).unwrap();
    assert_eq!(
        get_object_field(&obj, "field\\with\\backslash"),
        Some(Value::Integer(4))
    );
}

#[test]
fn test_very_long_field_name_r162() {
    let obj = new_mutable_object(HashMap::new());
    let long_name = "a".repeat(1000);
    set_object_field(&obj, &long_name, Value::Integer(999)).unwrap();
    assert_eq!(
        get_object_field(&obj, &long_name),
        Some(Value::Integer(999))
    );
}

#[test]
fn test_mutable_object_clone_behavior_r162() {
    let obj = new_mutable_object(HashMap::from([("x".to_string(), Value::Integer(10))]));

    // Clone the value
    let cloned = obj.clone();

    // Both should point to same Arc<Mutex<...>>
    set_object_field(&obj, "x", Value::Integer(20)).unwrap();

    // The cloned version should see the update (shared reference)
    assert_eq!(get_object_field(&cloned, "x"), Some(Value::Integer(20)));
}

#[test]
fn test_immutable_object_clone_behavior_r162() {
    let obj = new_immutable_object(HashMap::from([("x".to_string(), Value::Integer(10))]));

    // Clone should share the same Arc
    let cloned = obj.clone();
    assert_eq!(get_object_field(&cloned, "x"), Some(Value::Integer(10)));
}

#[test]
fn test_deeply_nested_objects_r162() {
    // Create 5 levels of nesting
    let mut level5 = HashMap::new();
    level5.insert("value".to_string(), Value::Integer(42));
    let obj5 = new_immutable_object(level5);

    let mut level4 = HashMap::new();
    level4.insert("nested".to_string(), obj5);
    let obj4 = new_immutable_object(level4);

    let mut level3 = HashMap::new();
    level3.insert("nested".to_string(), obj4);
    let obj3 = new_immutable_object(level3);

    let mut level2 = HashMap::new();
    level2.insert("nested".to_string(), obj3);
    let obj2 = new_immutable_object(level2);

    let mut level1 = HashMap::new();
    level1.insert("nested".to_string(), obj2);
    let obj1 = new_immutable_object(level1);

    // Navigate down
    let n1 = get_object_field(&obj1, "nested").unwrap();
    let n2 = get_object_field(&n1, "nested").unwrap();
    let n3 = get_object_field(&n2, "nested").unwrap();
    let n4 = get_object_field(&n3, "nested").unwrap();
    let final_val = get_object_field(&n4, "value").unwrap();

    assert_eq!(final_val, Value::Integer(42));
}
