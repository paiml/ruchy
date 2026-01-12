//! Actor Operations for Interpreter - Extracted for 100% Coverage
//!
//! Handles actor spawn, send, and query operations.

use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Helper: Extract message type and data from actor message Value
/// Complexity: 5 (within Toyota Way limits)
pub fn extract_message_type_and_data(
    message: &Value,
) -> Result<(String, Vec<Value>), InterpreterError> {
    if let Value::Object(msg_obj) = message {
        if let Some(Value::String(type_str)) = msg_obj.get("__type") {
            if type_str.as_ref() == "Message" {
                let msg_type = msg_obj
                    .get("type")
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.to_string())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                let msg_data = msg_obj
                    .get("data")
                    .and_then(|v| {
                        if let Value::Array(arr) = v {
                            Some(arr.to_vec())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(Vec::new);
                return Ok((msg_type, msg_data));
            }
        }
    }
    Err(InterpreterError::RuntimeError(
        "Invalid message format - expected Message object".to_string(),
    ))
}

/// Create a Message object with type and data
pub fn create_message_object(msg_type: &str, data: Vec<Value>) -> Value {
    let mut msg = HashMap::new();
    msg.insert(
        "__type".to_string(),
        Value::from_string("Message".to_string()),
    );
    msg.insert("type".to_string(), Value::from_string(msg_type.to_string()));
    msg.insert(
        "data".to_string(),
        Value::Array(Arc::from(data.into_boxed_slice())),
    );
    Value::Object(Arc::new(msg))
}

/// Check if a value is an actor instance
pub fn is_actor_instance(value: &Value) -> bool {
    if let Value::ObjectMut(cell) = value {
        if let Ok(guard) = cell.lock() {
            if let Some(Value::String(type_str)) = guard.get("__type") {
                return type_str.as_ref() == "ActorInstance";
            }
        }
    }
    false
}

/// Check if a value is an actor definition
pub fn is_actor_definition(value: &Value) -> bool {
    if let Value::Object(obj) = value {
        if let Some(Value::String(type_str)) = obj.get("__type") {
            return type_str.as_ref() == "Actor";
        }
    }
    false
}

/// Get actor name from an actor instance
pub fn get_actor_name(value: &Value) -> Option<String> {
    if let Value::ObjectMut(cell) = value {
        if let Ok(guard) = cell.lock() {
            if let Some(Value::String(name)) = guard.get("__actor_name") {
                return Some(name.to_string());
            }
        }
    }
    None
}

// ============================================================================
// Tests - 100% Coverage Target
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_message_type_and_data_valid() {
        let msg = create_message_object("Ping", vec![Value::Integer(42)]);
        let (msg_type, data) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Ping");
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], Value::Integer(42));
    }

    #[test]
    fn test_extract_message_type_and_data_empty_data() {
        let msg = create_message_object("Hello", vec![]);
        let (msg_type, data) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Hello");
        assert!(data.is_empty());
    }

    #[test]
    fn test_extract_message_type_and_data_invalid_not_object() {
        let msg = Value::Integer(42);
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_type_and_data_invalid_no_type() {
        let mut obj = HashMap::new();
        obj.insert(
            "data".to_string(),
            Value::Array(Arc::from(vec![].into_boxed_slice())),
        );
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_type_and_data_wrong_type() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("NotMessage".to_string()),
        );
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_missing_type_field() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        // No "type" field
        let msg = Value::Object(Arc::new(obj));
        let (msg_type, _) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Unknown"); // Default
    }

    #[test]
    fn test_extract_message_type_not_string() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        obj.insert("type".to_string(), Value::Integer(123)); // Not a string
        let msg = Value::Object(Arc::new(obj));
        let (msg_type, _) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Unknown"); // Default when not string
    }

    #[test]
    fn test_extract_message_data_not_array() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Message".to_string()),
        );
        obj.insert("type".to_string(), Value::from_string("Test".to_string()));
        obj.insert("data".to_string(), Value::Integer(42)); // Not an array
        let msg = Value::Object(Arc::new(obj));
        let (_, data) = extract_message_type_and_data(&msg).unwrap();
        assert!(data.is_empty()); // Default empty vec when not array
    }

    #[test]
    fn test_create_message_object() {
        let msg = create_message_object("Test", vec![Value::Bool(true)]);
        if let Value::Object(obj) = msg {
            assert_eq!(
                obj.get("__type"),
                Some(&Value::from_string("Message".to_string()))
            );
            assert_eq!(
                obj.get("type"),
                Some(&Value::from_string("Test".to_string()))
            );
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr.len(), 1);
            } else {
                panic!("Expected Array for data");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_create_message_object_multiple_data() {
        let msg = create_message_object(
            "Multi",
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr.len(), 3);
            }
        }
    }

    #[test]
    fn test_is_actor_instance_true() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("ActorInstance".to_string()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(is_actor_instance(&actor));
    }

    #[test]
    fn test_is_actor_instance_false_wrong_type() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("NotActor".to_string()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(!is_actor_instance(&actor));
    }

    #[test]
    fn test_is_actor_instance_false_no_type() {
        let obj = HashMap::new();
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(!is_actor_instance(&actor));
    }

    #[test]
    fn test_is_actor_instance_false_not_object_mut() {
        let actor = Value::Integer(42);
        assert!(!is_actor_instance(&actor));
    }

    #[test]
    fn test_is_actor_definition_true() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        let def = Value::Object(Arc::new(obj));
        assert!(is_actor_definition(&def));
    }

    #[test]
    fn test_is_actor_definition_false_wrong_type() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("NotActor".to_string()),
        );
        let def = Value::Object(Arc::new(obj));
        assert!(!is_actor_definition(&def));
    }

    #[test]
    fn test_is_actor_definition_false_not_object() {
        let def = Value::Integer(42);
        assert!(!is_actor_definition(&def));
    }

    #[test]
    fn test_get_actor_name_found() {
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string("MyActor".to_string()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), Some("MyActor".to_string()));
    }

    #[test]
    fn test_get_actor_name_not_found() {
        let obj = HashMap::new();
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), None);
    }

    #[test]
    fn test_get_actor_name_not_object_mut() {
        let actor = Value::Integer(42);
        assert_eq!(get_actor_name(&actor), None);
    }

    // ============================================================================
    // EXTREME TDD Round 132: Additional comprehensive tests
    // Target: 20 → 40+ tests
    // ============================================================================

    // --- extract_message_type_and_data edge cases ---
    #[test]
    fn test_extract_message_with_multiple_data_items() {
        let msg = create_message_object(
            "Complex",
            vec![
                Value::Integer(1),
                Value::from_string("hello".to_string()),
                Value::Bool(true),
            ],
        );
        let (msg_type, data) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Complex");
        assert_eq!(data.len(), 3);
        assert_eq!(data[0], Value::Integer(1));
    }

    #[test]
    fn test_extract_message_nil_value() {
        let msg = Value::Nil;
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid message format"));
    }

    #[test]
    fn test_extract_message_bool_value() {
        let msg = Value::Bool(true);
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_float_value() {
        let msg = Value::Float(3.14);
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_string_value() {
        let msg = Value::from_string("not a message".to_string());
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_array_value() {
        let msg = Value::Array(Arc::from(vec![Value::Integer(1)].into_boxed_slice()));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_empty_object() {
        let obj: HashMap<String, Value> = HashMap::new();
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_type_is_not_string_integer() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::Integer(123)); // Type should be string
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    // --- create_message_object edge cases ---
    #[test]
    fn test_create_message_object_empty_type() {
        let msg = create_message_object("", vec![]);
        if let Value::Object(obj) = msg {
            assert_eq!(obj.get("type"), Some(&Value::from_string(String::new())));
        }
    }

    #[test]
    fn test_create_message_object_unicode_type() {
        let msg = create_message_object("日本語", vec![]);
        if let Value::Object(obj) = msg {
            assert_eq!(
                obj.get("type"),
                Some(&Value::from_string("日本語".to_string()))
            );
        }
    }

    #[test]
    fn test_create_message_object_special_chars() {
        let msg = create_message_object("Type-With_Special.Chars!", vec![]);
        if let Value::Object(obj) = msg {
            assert_eq!(
                obj.get("type"),
                Some(&Value::from_string("Type-With_Special.Chars!".to_string()))
            );
        }
    }

    #[test]
    fn test_create_message_object_nested_arrays() {
        let nested = Value::Array(Arc::from(vec![Value::Integer(1)].into_boxed_slice()));
        let msg = create_message_object("Nested", vec![nested.clone()]);
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr.len(), 1);
                assert_eq!(arr[0], nested);
            }
        }
    }

    #[test]
    fn test_create_message_object_with_nil() {
        let msg = create_message_object("WithNil", vec![Value::Nil]);
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr[0], Value::Nil);
            }
        }
    }

    // --- is_actor_instance edge cases ---
    #[test]
    fn test_is_actor_instance_with_immutable_object() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("ActorInstance".to_string()),
        );
        let val = Value::Object(Arc::new(obj)); // Immutable object, not ObjectMut
        assert!(!is_actor_instance(&val));
    }

    #[test]
    fn test_is_actor_instance_type_is_integer() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::Integer(123));
        let val = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(!is_actor_instance(&val));
    }

    #[test]
    fn test_is_actor_instance_with_nil() {
        assert!(!is_actor_instance(&Value::Nil));
    }

    #[test]
    fn test_is_actor_instance_with_string() {
        assert!(!is_actor_instance(&Value::from_string("actor".to_string())));
    }

    #[test]
    fn test_is_actor_instance_with_array() {
        let val = Value::Array(Arc::from(vec![].into_boxed_slice()));
        assert!(!is_actor_instance(&val));
    }

    // --- is_actor_definition edge cases ---
    #[test]
    fn test_is_actor_definition_with_mutable_object() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        let val = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj))); // Mutable, not immutable
        assert!(!is_actor_definition(&val));
    }

    #[test]
    fn test_is_actor_definition_type_is_integer() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::Integer(123));
        let val = Value::Object(Arc::new(obj));
        assert!(!is_actor_definition(&val));
    }

    #[test]
    fn test_is_actor_definition_with_nil() {
        assert!(!is_actor_definition(&Value::Nil));
    }

    #[test]
    fn test_is_actor_definition_with_bool() {
        assert!(!is_actor_definition(&Value::Bool(true)));
    }

    #[test]
    fn test_is_actor_definition_empty_object() {
        let obj: HashMap<String, Value> = HashMap::new();
        let val = Value::Object(Arc::new(obj));
        assert!(!is_actor_definition(&val));
    }

    #[test]
    fn test_is_actor_definition_no_type() {
        let mut obj = HashMap::new();
        obj.insert(
            "name".to_string(),
            Value::from_string("MyActor".to_string()),
        );
        let val = Value::Object(Arc::new(obj));
        assert!(!is_actor_definition(&val));
    }

    // --- get_actor_name edge cases ---
    #[test]
    fn test_get_actor_name_with_immutable_object() {
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string("Test".to_string()),
        );
        let val = Value::Object(Arc::new(obj)); // Immutable, not ObjectMut
        assert_eq!(get_actor_name(&val), None);
    }

    #[test]
    fn test_get_actor_name_name_is_integer() {
        let mut obj = HashMap::new();
        obj.insert("__actor_name".to_string(), Value::Integer(123));
        let val = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&val), None);
    }

    #[test]
    fn test_get_actor_name_with_nil() {
        assert_eq!(get_actor_name(&Value::Nil), None);
    }

    #[test]
    fn test_get_actor_name_with_bool() {
        assert_eq!(get_actor_name(&Value::Bool(false)), None);
    }

    #[test]
    fn test_get_actor_name_with_float() {
        assert_eq!(get_actor_name(&Value::Float(1.5)), None);
    }

    #[test]
    fn test_get_actor_name_unicode() {
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string("アクター".to_string()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), Some("アクター".to_string()));
    }

    #[test]
    fn test_get_actor_name_empty_string() {
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string(String::new()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), Some(String::new()));
    }
}

// ============================================================================
// EXTREME TDD Round 135: Additional comprehensive tests
// Target: 51 → 65+ tests
// ============================================================================
#[cfg(test)]
mod round_135_tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    // --- create_message_object edge cases ---
    #[test]
    fn test_create_message_object_empty_name() {
        let msg = create_message_object("", vec![]);
        if let Value::Object(obj) = msg {
            assert_eq!(obj.get("type"), Some(&Value::from_string("".to_string())));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_create_message_object_unicode_name() {
        let msg = create_message_object("メッセージ", vec![Value::Integer(42)]);
        if let Value::Object(obj) = msg {
            assert_eq!(
                obj.get("type"),
                Some(&Value::from_string("メッセージ".to_string()))
            );
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_create_message_object_many_values() {
        let values: Vec<Value> = (0..10).map(Value::Integer).collect();
        let msg = create_message_object("Batch", values);
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr.len(), 10);
            } else {
                panic!("Expected data array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_create_message_object_with_bool() {
        let msg = create_message_object("Flag", vec![Value::Bool(true)]);
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr[0], Value::Bool(true));
            }
        }
    }

    #[test]
    fn test_create_message_object_with_float() {
        let msg = create_message_object("Number", vec![Value::Float(3.14159)]);
        if let Value::Object(obj) = msg {
            if let Some(Value::Array(arr)) = obj.get("data") {
                assert_eq!(arr[0], Value::Float(3.14159));
            }
        }
    }

    // --- is_actor_instance additional tests ---
    #[test]
    fn test_is_actor_instance_with_integer() {
        assert!(!is_actor_instance(&Value::Integer(0)));
        assert!(!is_actor_instance(&Value::Integer(i64::MAX)));
    }

    #[test]
    fn test_is_actor_instance_with_float() {
        assert!(!is_actor_instance(&Value::Float(0.0)));
        assert!(!is_actor_instance(&Value::Float(f64::INFINITY)));
    }

    #[test]
    fn test_is_actor_instance_wrong_type_string() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("NotAnActor".to_string()),
        );
        let val = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(!is_actor_instance(&val));
    }

    #[test]
    fn test_is_actor_instance_valid() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("ActorInstance".to_string()),
        );
        let val = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert!(is_actor_instance(&val));
    }

    // --- is_actor_definition additional tests ---
    #[test]
    fn test_is_actor_definition_with_integer() {
        assert!(!is_actor_definition(&Value::Integer(42)));
    }

    #[test]
    fn test_is_actor_definition_with_float() {
        assert!(!is_actor_definition(&Value::Float(3.14)));
    }

    #[test]
    fn test_is_actor_definition_wrong_type_string() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("NotAnActor".to_string()),
        );
        let val = Value::Object(Arc::new(obj));
        assert!(!is_actor_definition(&val));
    }

    #[test]
    fn test_is_actor_definition_valid() {
        let mut obj = HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Actor".to_string()),
        );
        let val = Value::Object(Arc::new(obj));
        assert!(is_actor_definition(&val));
    }

    // --- get_actor_name additional tests ---
    #[test]
    fn test_get_actor_name_with_integer_value() {
        assert_eq!(get_actor_name(&Value::Integer(123)), None);
    }

    #[test]
    fn test_get_actor_name_with_array() {
        let val = Value::Array(Arc::from(vec![Value::Integer(1)].into_boxed_slice()));
        assert_eq!(get_actor_name(&val), None);
    }

    #[test]
    fn test_get_actor_name_special_chars() {
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string("My-Actor_123".to_string()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), Some("My-Actor_123".to_string()));
    }

    #[test]
    fn test_get_actor_name_very_long() {
        let long_name = "a".repeat(1000);
        let mut obj = HashMap::new();
        obj.insert(
            "__actor_name".to_string(),
            Value::from_string(long_name.clone()),
        );
        let actor = Value::ObjectMut(Arc::new(std::sync::Mutex::new(obj)));
        assert_eq!(get_actor_name(&actor), Some(long_name));
    }

    // === EXTREME TDD Round 139 tests ===

    #[test]
    fn test_create_message_object_empty_type() {
        let msg = create_message_object("", vec![]);
        match msg {
            Value::Object(obj) => {
                assert!(obj.contains_key("type"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_create_message_object_with_multiple_data() {
        let msg = create_message_object(
            "data_msg",
            vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        match msg {
            Value::Object(obj) => {
                assert!(obj.contains_key("type"));
                assert!(obj.contains_key("data"));
            }
            _ => panic!("Expected object"),
        }
    }

    #[test]
    fn test_is_actor_instance_regular_object() {
        let obj = Value::Object(Arc::new(HashMap::new()));
        assert!(!is_actor_instance(&obj));
    }

    #[test]
    fn test_is_actor_instance_nil() {
        assert!(!is_actor_instance(&Value::Nil));
    }

    #[test]
    fn test_is_actor_definition_nil() {
        assert!(!is_actor_definition(&Value::Nil));
    }

    #[test]
    fn test_is_actor_definition_integer() {
        assert!(!is_actor_definition(&Value::Integer(42)));
    }

    #[test]
    fn test_get_actor_name_nil() {
        assert_eq!(get_actor_name(&Value::Nil), None);
    }
}
