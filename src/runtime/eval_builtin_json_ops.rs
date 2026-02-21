//! JSON builtin functions
//!
//! This module handles JSON operations including parse, stringify, pretty-print,
//! read, write, validate, type detection, merge, get, and set.

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

/// `json_parse(json_string)` - Parse JSON string to Ruchy value
/// Complexity: 3 (reduced by extracting conversion logic)
pub(crate) fn eval_json_parse(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_parse", args, 1)?;
    match &args[0] {
        Value::String(s) => parse_json_string_to_value(s),
        _ => Err(InterpreterError::RuntimeError(
            "json_parse() expects a string argument".to_string(),
        )),
    }
}

/// Parse JSON string and convert to Ruchy value
/// Complexity: 3 (parse + convert + error handling)
pub(crate) fn parse_json_string_to_value(s: &str) -> Result<Value, InterpreterError> {
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(json) => Ok(json_to_ruchy_value(json)),
        Err(e) => Err(InterpreterError::RuntimeError(format!(
            "JSON parse error: {e}"
        ))),
    }
}

/// Convert `serde_json::Value` to Ruchy Value
/// Complexity: 5 (6 match arms, reduced by extracting helpers)
pub(crate) fn json_to_ruchy_value(json: serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Nil,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => convert_json_number(n),
        serde_json::Value::String(s) => Value::from_string(s),
        serde_json::Value::Array(arr) => convert_json_array(arr),
        serde_json::Value::Object(obj) => convert_json_object(obj),
    }
}

/// Convert JSON number to Ruchy value
/// Complexity: 2 (try integer, fallback to float)
pub(crate) fn convert_json_number(n: serde_json::Number) -> Value {
    if let Some(i) = n.as_i64() {
        Value::Integer(i)
    } else if let Some(f) = n.as_f64() {
        Value::Float(f)
    } else {
        Value::Nil
    }
}

/// Convert JSON array to Ruchy array
/// Complexity: 2 (map + collect)
pub(crate) fn convert_json_array(arr: Vec<serde_json::Value>) -> Value {
    let values: Vec<Value> = arr.into_iter().map(json_to_ruchy_value).collect();
    Value::Array(values.into())
}

/// Convert JSON object to Ruchy object
/// Complexity: 3 (iteration + recursive conversion)
pub(crate) fn convert_json_object(obj: serde_json::Map<String, serde_json::Value>) -> Value {
    let mut map = std::collections::HashMap::new();
    for (k, v) in obj {
        map.insert(k, json_to_ruchy_value(v));
    }
    Value::Object(std::sync::Arc::new(map))
}

/// Convert Ruchy Value to `serde_json::Value`
/// Complexity: 5 (reduced by extracting array and object converters)
pub(crate) fn value_to_json(value: &Value) -> Result<serde_json::Value, InterpreterError> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::json!(*i)),
        Value::Float(f) => Ok(serde_json::json!(*f)),
        Value::String(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::Array(arr) => convert_ruchy_array_to_json(arr),
        Value::Object(map) => convert_ruchy_object_to_json(map),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Cannot convert {value:?} to JSON"
        ))),
    }
}

/// Convert Ruchy array to JSON array
/// Complexity: 2 (map + collect with error handling)
pub(crate) fn convert_ruchy_array_to_json(
    arr: &[Value],
) -> Result<serde_json::Value, InterpreterError> {
    let json_arr: Result<Vec<serde_json::Value>, _> = arr.iter().map(value_to_json).collect();
    Ok(serde_json::Value::Array(json_arr?))
}

/// Convert Ruchy object to JSON object
/// Complexity: 3 (iteration + recursive conversion)
pub(crate) fn convert_ruchy_object_to_json(
    map: &std::collections::HashMap<String, Value>,
) -> Result<serde_json::Value, InterpreterError> {
    let mut json_obj = serde_json::Map::new();
    for (k, v) in map {
        json_obj.insert(k.clone(), value_to_json(v)?);
    }
    Ok(serde_json::Value::Object(json_obj))
}

/// `json_stringify` operation
/// Complexity: 2
pub(crate) fn eval_json_stringify(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_stringify", args, 1)?;
    let json = value_to_json(&args[0])?;
    match serde_json::to_string(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!(
            "JSON stringify error: {e}"
        ))),
    }
}

/// `json_pretty` operation
/// Complexity: 2
pub(crate) fn eval_json_pretty(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_pretty", args, 1)?;
    let json = value_to_json(&args[0])?;
    match serde_json::to_string_pretty(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!(
            "JSON pretty error: {e}"
        ))),
    }
}

/// `json_read` operation
/// Complexity: 3
pub(crate) fn eval_json_read(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_read", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let content = std::fs::read_to_string(path.as_ref())
                .map_err(|e| InterpreterError::RuntimeError(format!("Failed to read file: {e}")))?;
            eval_json_parse(&[Value::from_string(content)])
        }
        _ => Err(InterpreterError::RuntimeError(
            "json_read() expects a string argument".to_string(),
        )),
    }
}

/// `json_write` operation
/// Complexity: 3
pub(crate) fn eval_json_write(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_write", args, 2)?;
    match &args[0] {
        Value::String(path) => {
            let json = value_to_json(&args[1])?;
            let content = serde_json::to_string_pretty(&json).map_err(|e| {
                InterpreterError::RuntimeError(format!("JSON stringify error: {e}"))
            })?;
            std::fs::write(path.as_ref(), content).map_err(|e| {
                InterpreterError::RuntimeError(format!("Failed to write file: {e}"))
            })?;
            Ok(Value::Bool(true))
        }
        _ => Err(InterpreterError::RuntimeError(
            "json_write() expects first argument to be string".to_string(),
        )),
    }
}

/// `json_validate` operation
/// Complexity: 2
pub(crate) fn eval_json_validate(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_validate", args, 1)?;
    match &args[0] {
        Value::String(s) => {
            let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();
            Ok(Value::Bool(is_valid))
        }
        _ => Err(InterpreterError::RuntimeError(
            "json_validate() expects a string argument".to_string(),
        )),
    }
}

/// `json_type` operation
/// Complexity: 3
pub(crate) fn eval_json_type(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_type", args, 1)?;
    match &args[0] {
        Value::String(s) => match serde_json::from_str::<serde_json::Value>(s) {
            Ok(json) => {
                let type_str = match json {
                    serde_json::Value::Null => "null",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                };
                Ok(Value::from_string(type_str.to_string()))
            }
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "JSON parse error: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "json_type() expects a string argument".to_string(),
        )),
    }
}

/// `json_merge(json1, json2)` - Deep merge two JSON values
/// Complexity: 3 (reduced by extracting merge logic)
pub(crate) fn eval_json_merge(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_merge", args, 2)?;
    let json1 = value_to_json(&args[0])?;
    let json2 = value_to_json(&args[1])?;

    let merged = merge_json_values(json1, json2);
    eval_json_parse(&[Value::from_string(merged.to_string())])
}

/// Recursively merge two JSON values
/// Complexity: 4
pub(crate) fn merge_json_values(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(mut a_map), serde_json::Value::Object(b_map)) => {
            merge_json_objects(&mut a_map, b_map);
            serde_json::Value::Object(a_map)
        }
        (_, b_val) => b_val,
    }
}

/// Merge JSON object maps recursively
/// Complexity: 4
pub(crate) fn merge_json_objects(
    a_map: &mut serde_json::Map<String, serde_json::Value>,
    b_map: serde_json::Map<String, serde_json::Value>,
) {
    for (k, v) in b_map {
        if let Some(a_val) = a_map.get_mut(&k) {
            *a_val = merge_json_values(a_val.clone(), v);
        } else {
            a_map.insert(k, v);
        }
    }
}

/// `json_get(json_value, path)` - Get value at JSON path
/// Complexity: 4
pub(crate) fn eval_json_get(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_get", args, 2)?;
    let json = value_to_json(&args[0])?;

    match &args[1] {
        Value::String(path) => get_json_value_at_path(&json, path),
        _ => Err(InterpreterError::RuntimeError(
            "json_get() expects second argument to be string".to_string(),
        )),
    }
}

/// Get JSON value at dot-separated path
pub(crate) fn get_json_value_at_path(
    json: &serde_json::Value,
    path: &str,
) -> Result<Value, InterpreterError> {
    let parts: Vec<&str> = path.split('.').collect();
    match get_json_path_recursive(json, &parts) {
        Some(val) => eval_json_parse(&[Value::from_string(val.to_string())]),
        None => Ok(Value::Nil),
    }
}

/// Recursively get JSON value at path
pub(crate) fn get_json_path_recursive<'a>(
    json: &'a serde_json::Value,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(json);
    }
    match json {
        serde_json::Value::Object(map) => map
            .get(path[0])
            .and_then(|v| get_json_path_recursive(v, &path[1..])),
        _ => None,
    }
}

/// `json_set(json_value, path, new_value)` - Set value at JSON path
/// Complexity: 4
pub(crate) fn eval_json_set(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_set", args, 3)?;
    let mut json = value_to_json(&args[0])?;
    let new_value = value_to_json(&args[2])?;

    match &args[1] {
        Value::String(path) => {
            set_json_path_from_string(&mut json, path, new_value)?;
            eval_json_parse(&[Value::from_string(json.to_string())])
        }
        _ => Err(InterpreterError::RuntimeError(
            "json_set() expects second argument to be string".to_string(),
        )),
    }
}

/// Set JSON value at dot-separated path
pub(crate) fn set_json_path_from_string(
    json: &mut serde_json::Value,
    path: &str,
    value: serde_json::Value,
) -> Result<(), InterpreterError> {
    let parts: Vec<&str> = path.split('.').collect();
    set_json_path_recursive(json, &parts, value);
    Ok(())
}

/// Recursively set JSON value at path
pub(crate) fn set_json_path_recursive(
    json: &mut serde_json::Value,
    path: &[&str],
    value: serde_json::Value,
) {
    if path.is_empty() {
        *json = value;
        return;
    }

    if path.len() == 1 {
        set_json_single_key(json, path[0], value);
    } else {
        set_json_nested_path(json, path, value);
    }
}

/// Set JSON value at single key
pub(crate) fn set_json_single_key(
    json: &mut serde_json::Value,
    key: &str,
    value: serde_json::Value,
) {
    if let serde_json::Value::Object(map) = json {
        map.insert(key.to_string(), value);
    }
}

/// Set JSON value at nested path
pub(crate) fn set_json_nested_path(
    json: &mut serde_json::Value,
    path: &[&str],
    value: serde_json::Value,
) {
    if let serde_json::Value::Object(map) = json {
        if let Some(next) = map.get_mut(path[0]) {
            set_json_path_recursive(next, &path[1..], value);
        }
    }
}

/// Dispatch JSON functions - Part 1a (parse/stringify)
pub(crate) fn try_eval_json_part1a(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_parse__" | "JSON_parse" | "parse_json" => Ok(Some(eval_json_parse(args)?)),
        "__builtin_json_stringify__" | "JSON_stringify" | "stringify_json" => {
            Ok(Some(eval_json_stringify(args)?))
        }
        "__builtin_json_pretty__" => Ok(Some(eval_json_pretty(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch JSON functions - Part 1b (read/write)
pub(crate) fn try_eval_json_part1b(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_read__" => Ok(Some(eval_json_read(args)?)),
        "__builtin_json_write__" => Ok(Some(eval_json_write(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch JSON functions - Part 1 (combined)
pub(crate) fn try_eval_json_part1(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_json_part1a(name, args)? {
        return Ok(Some(result));
    }
    try_eval_json_part1b(name, args)
}

/// Dispatch JSON functions - Part 2a (validate/type/merge)
pub(crate) fn try_eval_json_part2a(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_validate__" => Ok(Some(eval_json_validate(args)?)),
        "__builtin_json_type__" => Ok(Some(eval_json_type(args)?)),
        "__builtin_json_merge__" => Ok(Some(eval_json_merge(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch JSON functions - Part 2b (get/set)
pub(crate) fn try_eval_json_part2b(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_get__" => Ok(Some(eval_json_get(args)?)),
        "__builtin_json_set__" => Ok(Some(eval_json_set(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch JSON functions - Part 2 (combined)
pub(crate) fn try_eval_json_part2(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_json_part2a(name, args)? {
        return Ok(Some(result));
    }
    try_eval_json_part2b(name, args)
}

/// Dispatcher for JSON functions
pub(crate) fn try_eval_json_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    let dispatchers: &[fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>] =
        &[try_eval_json_part1, try_eval_json_part2];

    for dispatcher in dispatchers {
        if let Some(result) = dispatcher(name, args)? {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests_json_ops {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    // ==================== eval_json_write tests ====================

    #[test]
    fn test_json_write_success() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_json_write.json");
        let path_str = path.to_string_lossy().to_string();

        // Clean up any leftover file
        let _ = std::fs::remove_file(&path);

        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Value::from_string("value".to_string()));
        obj.insert("num".to_string(), Value::Integer(42));

        let args = vec![
            Value::from_string(path_str.clone()),
            Value::Object(Arc::new(obj)),
        ];

        let result = eval_json_write(&args).unwrap();
        assert_eq!(result, Value::Bool(true));

        // Verify file contents
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"key\""));
        assert!(content.contains("\"value\""));
        assert!(content.contains("42"));

        // Clean up
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_json_write_non_string_path() {
        let args = vec![Value::Integer(42), Value::Bool(true)];
        let result = eval_json_write(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects first argument to be string"));
    }

    #[test]
    fn test_json_write_wrong_arg_count() {
        let args = vec![Value::from_string("path.json".to_string())];
        let result = eval_json_write(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_write_invalid_path() {
        let args = vec![
            Value::from_string("/nonexistent/dir/file.json".to_string()),
            Value::Integer(42),
        ];
        let result = eval_json_write(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to write file"));
    }

    #[test]
    fn test_json_write_nested_object() {
        let dir = std::env::temp_dir();
        let path = dir.join("ruchy_test_json_write_nested.json");
        let path_str = path.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&path);

        let mut inner = HashMap::new();
        inner.insert("nested".to_string(), Value::Bool(true));
        let mut outer = HashMap::new();
        outer.insert("inner".to_string(), Value::Object(Arc::new(inner)));

        let args = vec![Value::from_string(path_str), Value::Object(Arc::new(outer))];

        let result = eval_json_write(&args).unwrap();
        assert_eq!(result, Value::Bool(true));

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"nested\""));
        assert!(content.contains("true"));

        let _ = std::fs::remove_file(&path);
    }
}
