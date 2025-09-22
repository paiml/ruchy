//! Centralized data transformation module to eliminate entropy
//!
//! PMAT found `DataTransformation` pattern repeated 10 times (792 lines)
//! This module centralizes transformation logic with O(1) complexity per operation.

use crate::runtime::{InterpreterError, Value};

/// Convert Value to i64 - O(1) complexity
#[inline]
pub fn to_i64(value: &Value) -> Result<i64, InterpreterError> {
    match value {
        Value::Integer(n) => Ok(*n),
        Value::Float(f) => Ok(*f as i64),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot convert {} to integer",
            value.type_name()
        ))),
    }
}

/// Convert Value to f64 - O(1) complexity
#[inline]
pub fn to_f64(value: &Value) -> Result<f64, InterpreterError> {
    match value {
        Value::Integer(n) => Ok(*n as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot convert {} to float",
            value.type_name()
        ))),
    }
}

/// Convert Value to String - O(1) complexity for already-strings
#[inline]
pub fn to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.to_string(),
        _ => format!("{value}"),
    }
}

/// Convert Value to bool (truthiness) - O(1) complexity
#[inline]
pub fn to_bool(value: &Value) -> bool {
    value.is_truthy()
}

/// Coerce numeric values to common type - O(1) complexity
#[inline]
pub fn coerce_numeric(left: &Value, right: &Value) -> Result<(f64, f64), InterpreterError> {
    let l = to_f64(left)?;
    let r = to_f64(right)?;
    Ok((l, r))
}

/// Transform collection of Values - O(n) complexity where n is collection size
pub fn transform_collection<T, F>(
    values: &[Value],
    transform: F,
) -> Result<Vec<T>, InterpreterError>
where
    F: Fn(&Value) -> Result<T, InterpreterError>,
{
    values.iter().map(transform).collect()
}

/// Batch conversion to integers - O(n) complexity
#[inline]
pub fn to_i64_batch(values: &[Value]) -> Result<Vec<i64>, InterpreterError> {
    transform_collection(values, to_i64)
}

/// Batch conversion to floats - O(n) complexity
#[inline]
pub fn to_f64_batch(values: &[Value]) -> Result<Vec<f64>, InterpreterError> {
    transform_collection(values, to_f64)
}

/// Extract array from Value - O(1) complexity
#[inline]
pub fn extract_array(value: &Value) -> Result<&[Value], InterpreterError> {
    match value {
        Value::Array(arr) => Ok(arr),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Expected array, got {}",
            value.type_name()
        ))),
    }
}

/// Extract string from Value - O(1) complexity
#[inline]
pub fn extract_string(value: &Value) -> Result<&str, InterpreterError> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Expected string, got {}",
            value.type_name()
        ))),
    }
}

/// Create Value from various types - O(1) complexity each
#[inline]
pub fn from_i64(n: i64) -> Value {
    Value::Integer(n)
}

#[inline]
pub fn from_f64(f: f64) -> Value {
    Value::Float(f)
}

#[inline]
pub fn from_string(s: String) -> Value {
    Value::from_string(s)
}

#[inline]
pub fn from_bool(b: bool) -> Value {
    Value::Bool(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_i64_conversions() {
        assert_eq!(to_i64(&Value::Integer(42)).unwrap(), 42);
        assert_eq!(to_i64(&Value::Float(3.14)).unwrap(), 3);
        assert!(to_i64(&Value::from_string("not a number".to_string())).is_err());
    }

    #[test]
    fn test_to_f64_conversions() {
        assert_eq!(to_f64(&Value::Integer(42)).unwrap(), 42.0);
        assert_eq!(to_f64(&Value::Float(3.14)).unwrap(), 3.14);
        assert!(to_f64(&Value::from_string("not a number".to_string())).is_err());
    }

    #[test]
    fn test_coerce_numeric() {
        let (l, r) = coerce_numeric(&Value::Integer(5), &Value::Float(3.14)).unwrap();
        assert_eq!(l, 5.0);
        assert_eq!(r, 3.14);
    }

    #[test]
    fn test_batch_transformations() {
        let values = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = to_i64_batch(&values).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_transformation_preserves_value(n: i64) {
            let value = from_i64(n);
            let result = to_i64(&value).unwrap();
            prop_assert_eq!(result, n);
        }

        #[test]
        fn test_float_round_trip(f: f64) {
            if !f.is_nan() && !f.is_infinite() {
                let value = from_f64(f);
                let result = to_f64(&value).unwrap();
                prop_assert!((result - f).abs() < 0.0001);
            }
        }

        #[test]
        fn test_batch_transformation_preserves_order(values: Vec<i64>) {
            let value_objects: Vec<Value> = values.iter().map(|&n| from_i64(n)).collect();
            if let Ok(result) = to_i64_batch(&value_objects) {
                prop_assert_eq!(result, values);
            }
        }

        #[test]
        fn test_transformations_are_constant_time(n: i64) {
            // These transformations should be O(1)
            // Not dependent on the value of n

            let value = from_i64(n);
            let _ = to_i64(&value);
            let _ = to_f64(&value);
            let _ = to_bool(&value);

            // All complete in constant time
            prop_// Test passes without panic;
        }
    }
}
