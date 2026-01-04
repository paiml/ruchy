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
        obj.insert("data".to_string(), Value::Array(Arc::from(vec![].into_boxed_slice())));
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_type_and_data_wrong_type() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("NotMessage".to_string()));
        let msg = Value::Object(Arc::new(obj));
        let result = extract_message_type_and_data(&msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_missing_type_field() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        // No "type" field
        let msg = Value::Object(Arc::new(obj));
        let (msg_type, _) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Unknown"); // Default
    }

    #[test]
    fn test_extract_message_type_not_string() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
        obj.insert("type".to_string(), Value::Integer(123)); // Not a string
        let msg = Value::Object(Arc::new(obj));
        let (msg_type, _) = extract_message_type_and_data(&msg).unwrap();
        assert_eq!(msg_type, "Unknown"); // Default when not string
    }

    #[test]
    fn test_extract_message_data_not_array() {
        let mut obj = HashMap::new();
        obj.insert("__type".to_string(), Value::from_string("Message".to_string()));
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
        obj.insert("__type".to_string(), Value::from_string("Actor".to_string()));
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
}
