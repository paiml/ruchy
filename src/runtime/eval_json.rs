//! JSON Operations for Interpreter - Extracted for 100% Coverage
//!
//! Handles JSON parsing and stringification for the runtime.

use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// ISSUE-117: Parse JSON string into Value (complexity: 8)
pub fn json_parse(json_str: &str) -> Result<Value, InterpreterError> {
    let json_value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| InterpreterError::RuntimeError(format!("JSON.parse() failed: {e}")))?;
    serde_to_value(&json_value)
}

/// ISSUE-117: Stringify Value to JSON string (complexity: 4)
pub fn json_stringify(value: &Value) -> Result<Value, InterpreterError> {
    let json_value = value_to_serde(value)?;
    let json_str = serde_json::to_string(&json_value).map_err(|e| {
        InterpreterError::RuntimeError(format!("JSON.stringify() failed: {e}"))
    })?;
    Ok(Value::from_string(json_str))
}

/// Convert `serde_json::Value` to interpreter Value (complexity: 9)
pub fn serde_to_value(json: &serde_json::Value) -> Result<Value, InterpreterError> {
    match json {
        serde_json::Value::Null => Ok(Value::Nil),
        serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(Value::Float(f))
            } else {
                Err(InterpreterError::RuntimeError(
                    "JSON number out of range".to_string(),
                ))
            }
        }
        serde_json::Value::String(s) => Ok(Value::from_string(s.clone())),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<Value>, InterpreterError> =
                arr.iter().map(serde_to_value).collect();
            Ok(Value::Array(Arc::from(values?.into_boxed_slice())))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key.clone(), serde_to_value(val)?);
            }
            Ok(Value::Object(Arc::new(map)))
        }
    }
}

/// Convert interpreter Value to `serde_json::Value` (complexity: 8)
pub fn value_to_serde(value: &Value) -> Result<serde_json::Value, InterpreterError> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::Value::Number((*i).into())),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                Ok(serde_json::Value::Number(n))
            } else {
                Err(InterpreterError::RuntimeError(
                    "Invalid float for JSON".to_string(),
                ))
            }
        }
        Value::String(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::Array(arr) => {
            let json_arr: Result<Vec<serde_json::Value>, InterpreterError> =
                arr.iter().map(value_to_serde).collect();
            Ok(serde_json::Value::Array(json_arr?))
        }
        Value::Object(obj) => {
            let mut json_obj = serde_json::Map::new();
            for (key, val) in obj.as_ref() {
                json_obj.insert(key.clone(), value_to_serde(val)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        }
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot convert {value:?} to JSON"
        ))),
    }
}

// ============================================================================
// Tests - 100% Coverage Target
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // json_parse tests
    #[test]
    fn test_json_parse_null() {
        let result = json_parse("null").unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_json_parse_bool_true() {
        let result = json_parse("true").unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_json_parse_bool_false() {
        let result = json_parse("false").unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_json_parse_integer() {
        let result = json_parse("42").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_json_parse_negative_integer() {
        let result = json_parse("-123").unwrap();
        assert_eq!(result, Value::Integer(-123));
    }

    #[test]
    fn test_json_parse_float() {
        let result = json_parse("3.14").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_json_parse_string() {
        let result = json_parse("\"hello\"").unwrap();
        assert_eq!(result.to_string(), "\"hello\"");
    }

    #[test]
    fn test_json_parse_empty_string() {
        let result = json_parse("\"\"").unwrap();
        assert_eq!(result.to_string(), "\"\"");
    }

    #[test]
    fn test_json_parse_array() {
        let result = json_parse("[1, 2, 3]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_json_parse_empty_array() {
        let result = json_parse("[]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_json_parse_object() {
        let result = json_parse("{\"key\": 42}").unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("key"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_json_parse_empty_object() {
        let result = json_parse("{}").unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.len(), 0);
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_json_parse_nested() {
        let result = json_parse("{\"arr\": [1, {\"nested\": true}]}").unwrap();
        if let Value::Object(obj) = result {
            if let Some(Value::Array(arr)) = obj.get("arr") {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected arr to be Array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_json_parse_invalid() {
        let result = json_parse("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_parse_incomplete() {
        let result = json_parse("{\"key\":");
        assert!(result.is_err());
    }

    // json_stringify tests
    #[test]
    fn test_json_stringify_null() {
        let result = json_stringify(&Value::Nil).unwrap();
        assert_eq!(result.to_string(), "\"null\"");
    }

    #[test]
    fn test_json_stringify_bool() {
        let result = json_stringify(&Value::Bool(true)).unwrap();
        assert_eq!(result.to_string(), "\"true\"");
    }

    #[test]
    fn test_json_stringify_integer() {
        let result = json_stringify(&Value::Integer(42)).unwrap();
        assert_eq!(result.to_string(), "\"42\"");
    }

    #[test]
    fn test_json_stringify_float() {
        let result = json_stringify(&Value::Float(3.14)).unwrap();
        assert_eq!(result.to_string(), "\"3.14\"");
    }

    #[test]
    fn test_json_stringify_string() {
        let result = json_stringify(&Value::from_string("hello".to_string())).unwrap();
        // Result is Value::String containing "\"hello\"" (JSON encoded)
        // to_string() wraps it in quotes, so we get: "\"hello\""
        assert_eq!(result.to_string(), "\"\"hello\"\"");
    }

    #[test]
    fn test_json_stringify_array() {
        let arr = Value::Array(Arc::from(
            vec![Value::Integer(1), Value::Integer(2)].into_boxed_slice(),
        ));
        let result = json_stringify(&arr).unwrap();
        assert_eq!(result.to_string(), "\"[1,2]\"");
    }

    #[test]
    fn test_json_stringify_object() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Integer(1));
        let obj = Value::Object(Arc::new(map));
        let result = json_stringify(&obj).unwrap();
        assert!(result.to_string().contains("\"a\""));
    }

    #[test]
    fn test_json_stringify_nan() {
        let result = json_stringify(&Value::Float(f64::NAN));
        assert!(result.is_err());
    }

    #[test]
    fn test_json_stringify_infinity() {
        let result = json_stringify(&Value::Float(f64::INFINITY));
        assert!(result.is_err());
    }

    #[test]
    fn test_json_stringify_unsupported_type() {
        // Tuples are not JSON-serializable
        let tuple = Value::Tuple(Arc::from(vec![Value::Integer(1)].into_boxed_slice()));
        let result = json_stringify(&tuple);
        assert!(result.is_err());
    }

    // serde_to_value tests
    #[test]
    fn test_serde_to_value_large_number() {
        // Test with a float that has no exact i64 representation
        let json = serde_json::Value::Number(serde_json::Number::from_f64(1.5).unwrap());
        let result = serde_to_value(&json).unwrap();
        assert_eq!(result, Value::Float(1.5));
    }

    // value_to_serde tests
    #[test]
    fn test_value_to_serde_empty_array() {
        let arr = Value::Array(Arc::from(Vec::<Value>::new().into_boxed_slice()));
        let result = value_to_serde(&arr).unwrap();
        assert_eq!(result, serde_json::Value::Array(vec![]));
    }

    #[test]
    fn test_value_to_serde_nested_object() {
        let mut inner = HashMap::new();
        inner.insert("x".to_string(), Value::Integer(1));
        let mut outer = HashMap::new();
        outer.insert("inner".to_string(), Value::Object(Arc::new(inner)));
        let obj = Value::Object(Arc::new(outer));
        let result = value_to_serde(&obj).unwrap();
        assert!(result.is_object());
    }

    // Round-trip tests
    #[test]
    fn test_round_trip_integer() {
        let original = Value::Integer(42);
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            assert_eq!(back, original);
        }
    }

    #[test]
    fn test_round_trip_array() {
        let original = Value::Array(Arc::from(
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into_boxed_slice(),
        ));
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            if let (Value::Array(orig_arr), Value::Array(back_arr)) = (&original, &back) {
                assert_eq!(orig_arr.len(), back_arr.len());
            }
        }
    }

    #[test]
    fn test_round_trip_object() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), Value::from_string("value".to_string()));
        let original = Value::Object(Arc::new(map));
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            if let Value::Object(back_obj) = back {
                assert!(back_obj.contains_key("key"));
            }
        }
    }

    // === EXTREME TDD Round 22 - Coverage Push Tests ===

    #[test]
    fn test_json_parse_negative_float() {
        let result = json_parse("-3.14").unwrap();
        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_json_parse_scientific_notation() {
        let result = json_parse("1e10").unwrap();
        if let Value::Float(f) = result {
            assert!((f - 1e10).abs() < 1e5);
        } else if let Value::Integer(i) = result {
            assert_eq!(i, 10_000_000_000);
        }
    }

    #[test]
    fn test_json_parse_unicode_string() {
        let result = json_parse("\"Hello 世界\"").unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "Hello 世界");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_json_parse_escaped_string() {
        let result = json_parse("\"line1\\nline2\"").unwrap();
        if let Value::String(s) = result {
            assert!(s.contains('\n'));
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_json_parse_mixed_array() {
        let result = json_parse("[1, \"two\", true, null]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 4);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[2], Value::Bool(true));
            assert_eq!(arr[3], Value::Nil);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_json_parse_deeply_nested() {
        let result = json_parse("{\"a\": {\"b\": {\"c\": 42}}}").unwrap();
        if let Value::Object(obj) = result {
            if let Some(Value::Object(a)) = obj.get("a") {
                if let Some(Value::Object(b)) = a.get("b") {
                    assert_eq!(b.get("c"), Some(&Value::Integer(42)));
                } else {
                    panic!("Expected nested object b");
                }
            } else {
                panic!("Expected nested object a");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_json_stringify_negative_integer() {
        let result = json_stringify(&Value::Integer(-100)).unwrap();
        assert_eq!(result.to_string(), "\"-100\"");
    }

    #[test]
    fn test_json_stringify_false() {
        let result = json_stringify(&Value::Bool(false)).unwrap();
        assert_eq!(result.to_string(), "\"false\"");
    }

    #[test]
    fn test_json_stringify_empty_object() {
        let obj = Value::Object(Arc::new(HashMap::new()));
        let result = json_stringify(&obj).unwrap();
        assert_eq!(result.to_string(), "\"{}\"");
    }

    #[test]
    fn test_json_stringify_nested_array() {
        let inner = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)].into_boxed_slice()));
        let outer = Value::Array(Arc::from(vec![inner].into_boxed_slice()));
        let result = json_stringify(&outer).unwrap();
        assert!(result.to_string().contains("[[1,2]]"));
    }

    #[test]
    fn test_json_stringify_neg_infinity() {
        let result = json_stringify(&Value::Float(f64::NEG_INFINITY));
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_to_value_i64_max() {
        let json = serde_json::Value::Number(i64::MAX.into());
        let result = serde_to_value(&json).unwrap();
        assert_eq!(result, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_value_to_serde_nil() {
        let result = value_to_serde(&Value::Nil).unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }

    #[test]
    fn test_value_to_serde_integer() {
        let result = value_to_serde(&Value::Integer(42)).unwrap();
        assert_eq!(result, serde_json::Value::Number(42.into()));
    }

    #[test]
    fn test_value_to_serde_string() {
        let result = value_to_serde(&Value::from_string("test".to_string())).unwrap();
        assert_eq!(result, serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_value_to_serde_bool() {
        let result = value_to_serde(&Value::Bool(true)).unwrap();
        assert_eq!(result, serde_json::Value::Bool(true));
    }

    #[test]
    fn test_round_trip_bool() {
        let original = Value::Bool(true);
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            assert_eq!(back, original);
        }
    }

    #[test]
    fn test_round_trip_nil() {
        let original = Value::Nil;
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            assert_eq!(back, original);
        }
    }

    #[test]
    fn test_round_trip_float() {
        let original = Value::Float(3.14159);
        let json = json_stringify(&original).unwrap();
        if let Value::String(s) = json {
            let back = json_parse(&s).unwrap();
            if let Value::Float(f) = back {
                assert!((f - 3.14159).abs() < 0.00001);
            }
        }
    }

    #[test]
    fn test_json_parse_trailing_comma() {
        // JSON doesn't allow trailing commas
        let result = json_parse("{\"a\": 1,}");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_parse_single_quoted_string() {
        // JSON requires double quotes
        let result = json_parse("{'a': 1}");
        assert!(result.is_err());
    }
}
