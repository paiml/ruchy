//! Index Operations for Interpreter - Extracted for 100% Coverage
//!
//! Handles array, string, tuple, object, and DataFrame indexing with
//! Python/Ruby-style negative indexing support.

use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Index into an array (complexity: 5 - added negative indexing support)
/// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
pub fn index_array(array: &[Value], idx: i64) -> Result<Value, InterpreterError> {
    let len = array.len() as i64;
    let actual_index = if idx < 0 {
        // Negative index: count from the end
        // -1 => len-1 (last), -2 => len-2 (second-to-last), etc.
        len + idx
    } else {
        idx
    };

    // Check bounds (actual_index must be in range [0, len))
    if actual_index < 0 || actual_index >= len {
        return Err(InterpreterError::RuntimeError(format!(
            "Index {idx} out of bounds for array of length {len}"
        )));
    }

    #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
    Ok(array[actual_index as usize].clone())
}

/// Index into a string (complexity: 5 - added negative indexing support)
/// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
pub fn index_string(s: &str, idx: i64) -> Result<Value, InterpreterError> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let actual_index = if idx < 0 {
        // Negative index: count from the end
        len + idx
    } else {
        idx
    };

    // Check bounds
    if actual_index < 0 || actual_index >= len {
        return Err(InterpreterError::RuntimeError(format!(
            "Index {idx} out of bounds for string of length {len}"
        )));
    }

    #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
    Ok(Value::from_string(chars[actual_index as usize].to_string()))
}

/// Slice a string using a range (ISSUE-094, GitHub Issue #94)
/// Supports: text[0..5], text[..5], text[5..], text[..]
/// Cyclomatic complexity: 9 (A+ standard: ≤10)
pub fn slice_string(
    s: &str,
    start: &Value,
    end: &Value,
    _inclusive: bool,
) -> Result<Value, InterpreterError> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    // Extract start index (default to 0 for open ranges like ..5)
    let start_idx = match start {
        Value::Nil => 0,
        Value::Integer(i) => {
            if *i < 0 {
                let adjusted = len as i64 + i;
                if adjusted < 0 {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Range start {} is out of bounds for string of length {}",
                        i, len
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            }
        }
        _ => {
            return Err(InterpreterError::RuntimeError(format!(
                "Range start must be integer or nil, got {}",
                start.type_name()
            )))
        }
    };

    // Extract end index (default to len for open ranges like 5..)
    let end_idx = match end {
        Value::Nil => len,
        Value::Integer(i) => {
            if *i < 0 {
                let adjusted = len as i64 + i;
                if adjusted < 0 {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Range end {} is out of bounds for string of length {}",
                        i, len
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            }
        }
        _ => {
            return Err(InterpreterError::RuntimeError(format!(
                "Range end must be integer or nil, got {}",
                end.type_name()
            )))
        }
    };

    // Validate range
    if start_idx > end_idx {
        return Err(InterpreterError::RuntimeError(format!(
            "Invalid range: start {} is greater than end {}",
            start_idx, end_idx
        )));
    }

    if end_idx > len {
        return Err(InterpreterError::RuntimeError(format!(
            "Range end {} is out of bounds for string of length {}",
            end_idx, len
        )));
    }

    // Perform the slice
    let sliced: String = chars[start_idx..end_idx].iter().collect();
    Ok(Value::from_string(sliced))
}

/// Index into a tuple (complexity: 5 - added negative indexing support)
/// FEATURE-042 (GitHub Issue #46): Support Python/Ruby-style negative indexing
pub fn index_tuple(tuple: &[Value], idx: i64) -> Result<Value, InterpreterError> {
    let len = tuple.len() as i64;
    let actual_index = if idx < 0 {
        // Negative index: count from the end
        len + idx
    } else {
        idx
    };

    // Check bounds
    if actual_index < 0 || actual_index >= len {
        return Err(InterpreterError::RuntimeError(format!(
            "Index {idx} out of bounds for tuple of length {len}"
        )));
    }

    #[allow(clippy::cast_sign_loss)] // Safe: we've verified actual_index >= 0
    Ok(tuple[actual_index as usize].clone())
}

/// Index into an object with string key (complexity: 1)
pub fn index_object(fields: &HashMap<String, Value>, key: &str) -> Result<Value, InterpreterError> {
    fields.get(key).cloned().ok_or_else(|| {
        InterpreterError::RuntimeError(format!("Key '{key}' not found in object"))
    })
}

/// Index into a mutable object with string key (complexity: 1)
pub fn index_object_mut(
    cell: &Arc<std::sync::Mutex<HashMap<String, Value>>>,
    key: &str,
) -> Result<Value, InterpreterError> {
    cell.lock()
        .expect("Mutex poisoned: object lock is corrupted")
        .get(key)
        .cloned()
        .ok_or_else(|| InterpreterError::RuntimeError(format!("Key '{key}' not found in object")))
}

/// Index into a `DataFrame` by row index (complexity: 5)
/// Returns a row as an Object with column names as keys
pub fn index_dataframe_row(
    columns: &[DataFrameColumn],
    row_idx: i64,
) -> Result<Value, InterpreterError> {
    if columns.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "Cannot index empty DataFrame".to_string(),
        ));
    }

    let index = row_idx as usize;
    let num_rows = columns[0].values.len();

    if index >= num_rows {
        return Err(InterpreterError::RuntimeError(format!(
            "Row index {index} out of bounds for DataFrame with {num_rows} rows"
        )));
    }

    // Build row as Object with column names as keys
    let mut row = HashMap::new();
    for col in columns {
        row.insert(col.name.clone(), col.values[index].clone());
    }

    Ok(Value::Object(Arc::new(row)))
}

/// Index into a `DataFrame` by column name (complexity: 3)
/// Returns a column as an Array
pub fn index_dataframe_column(
    columns: &[DataFrameColumn],
    col_name: &str,
) -> Result<Value, InterpreterError> {
    columns
        .iter()
        .find(|col| col.name == col_name)
        .map(|col| Value::Array(Arc::from(col.values.clone())))
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Column '{col_name}' not found in DataFrame"))
        })
}

// ============================================================================
// Tests - 100% Coverage Target
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // index_array tests
    #[test]
    fn test_index_array_positive() {
        let arr = vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)];
        assert_eq!(index_array(&arr, 0).unwrap(), Value::Integer(10));
        assert_eq!(index_array(&arr, 1).unwrap(), Value::Integer(20));
        assert_eq!(index_array(&arr, 2).unwrap(), Value::Integer(30));
    }

    #[test]
    fn test_index_array_negative() {
        let arr = vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)];
        assert_eq!(index_array(&arr, -1).unwrap(), Value::Integer(30));
        assert_eq!(index_array(&arr, -2).unwrap(), Value::Integer(20));
        assert_eq!(index_array(&arr, -3).unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_index_array_out_of_bounds() {
        let arr = vec![Value::Integer(10)];
        assert!(index_array(&arr, 5).is_err());
        assert!(index_array(&arr, -5).is_err());
    }

    #[test]
    fn test_index_array_empty() {
        let arr: Vec<Value> = vec![];
        assert!(index_array(&arr, 0).is_err());
    }

    // index_string tests
    #[test]
    fn test_index_string_positive() {
        let result = index_string("hello", 0).unwrap();
        assert_eq!(result.to_string(), "\"h\"");
        let result = index_string("hello", 4).unwrap();
        assert_eq!(result.to_string(), "\"o\"");
    }

    #[test]
    fn test_index_string_negative() {
        let result = index_string("hello", -1).unwrap();
        assert_eq!(result.to_string(), "\"o\"");
        let result = index_string("hello", -5).unwrap();
        assert_eq!(result.to_string(), "\"h\"");
    }

    #[test]
    fn test_index_string_out_of_bounds() {
        assert!(index_string("hi", 10).is_err());
        assert!(index_string("hi", -10).is_err());
    }

    #[test]
    fn test_index_string_unicode() {
        let result = index_string("日本語", 1).unwrap();
        assert_eq!(result.to_string(), "\"本\"");
    }

    // slice_string tests
    #[test]
    fn test_slice_string_basic() {
        let result = slice_string("hello", &Value::Integer(1), &Value::Integer(4), false).unwrap();
        assert_eq!(result.to_string(), "\"ell\"");
    }

    #[test]
    fn test_slice_string_from_start() {
        let result = slice_string("hello", &Value::Nil, &Value::Integer(3), false).unwrap();
        assert_eq!(result.to_string(), "\"hel\"");
    }

    #[test]
    fn test_slice_string_to_end() {
        let result = slice_string("hello", &Value::Integer(2), &Value::Nil, false).unwrap();
        assert_eq!(result.to_string(), "\"llo\"");
    }

    #[test]
    fn test_slice_string_full() {
        let result = slice_string("hello", &Value::Nil, &Value::Nil, false).unwrap();
        assert_eq!(result.to_string(), "\"hello\"");
    }

    #[test]
    fn test_slice_string_negative_start() {
        let result = slice_string("hello", &Value::Integer(-3), &Value::Nil, false).unwrap();
        assert_eq!(result.to_string(), "\"llo\"");
    }

    #[test]
    fn test_slice_string_negative_end() {
        let result = slice_string("hello", &Value::Integer(0), &Value::Integer(-1), false).unwrap();
        assert_eq!(result.to_string(), "\"hell\"");
    }

    #[test]
    fn test_slice_string_invalid_start_type() {
        let result = slice_string("hello", &Value::Bool(true), &Value::Integer(3), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_string_invalid_end_type() {
        let result = slice_string("hello", &Value::Integer(0), &Value::Bool(false), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_string_invalid_range() {
        let result = slice_string("hello", &Value::Integer(4), &Value::Integer(2), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_string_out_of_bounds_end() {
        let result = slice_string("hi", &Value::Integer(0), &Value::Integer(10), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_string_negative_out_of_bounds_start() {
        let result = slice_string("hi", &Value::Integer(-10), &Value::Integer(2), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_slice_string_negative_out_of_bounds_end() {
        let result = slice_string("hi", &Value::Integer(0), &Value::Integer(-10), false);
        assert!(result.is_err());
    }

    // index_tuple tests
    #[test]
    fn test_index_tuple_positive() {
        let tuple = vec![Value::Integer(1), Value::from_string("two".to_string())];
        assert_eq!(index_tuple(&tuple, 0).unwrap(), Value::Integer(1));
    }

    #[test]
    fn test_index_tuple_negative() {
        let tuple = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        assert_eq!(index_tuple(&tuple, -1).unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_index_tuple_out_of_bounds() {
        let tuple = vec![Value::Integer(1)];
        assert!(index_tuple(&tuple, 5).is_err());
        assert!(index_tuple(&tuple, -5).is_err());
    }

    // index_object tests
    #[test]
    fn test_index_object_found() {
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Value::Integer(42));
        assert_eq!(index_object(&obj, "key").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_index_object_not_found() {
        let obj = HashMap::new();
        assert!(index_object(&obj, "missing").is_err());
    }

    // index_object_mut tests
    #[test]
    fn test_index_object_mut_found() {
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), Value::Integer(42));
        let cell = Arc::new(std::sync::Mutex::new(obj));
        assert_eq!(index_object_mut(&cell, "key").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_index_object_mut_not_found() {
        let obj = HashMap::new();
        let cell = Arc::new(std::sync::Mutex::new(obj));
        assert!(index_object_mut(&cell, "missing").is_err());
    }

    // index_dataframe_row tests
    #[test]
    fn test_index_dataframe_row_valid() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(10), Value::Integer(20)],
            },
        ];
        let row = index_dataframe_row(&columns, 0).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("a"), Some(&Value::Integer(1)));
            assert_eq!(obj.get("b"), Some(&Value::Integer(10)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_dataframe_row_empty() {
        let columns: Vec<DataFrameColumn> = vec![];
        assert!(index_dataframe_row(&columns, 0).is_err());
    }

    #[test]
    fn test_index_dataframe_row_out_of_bounds() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        assert!(index_dataframe_row(&columns, 5).is_err());
    }

    // index_dataframe_column tests
    #[test]
    fn test_index_dataframe_column_found() {
        let columns = vec![DataFrameColumn {
            name: "col1".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2)],
        }];
        let col = index_dataframe_column(&columns, "col1").unwrap();
        if let Value::Array(arr) = col {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_index_dataframe_column_not_found() {
        let columns = vec![DataFrameColumn {
            name: "col1".to_string(),
            values: vec![Value::Integer(1)],
        }];
        assert!(index_dataframe_column(&columns, "missing").is_err());
    }

    // Edge case tests
    #[test]
    fn test_index_array_single_element() {
        let arr = vec![Value::Bool(true)];
        assert_eq!(index_array(&arr, 0).unwrap(), Value::Bool(true));
        assert_eq!(index_array(&arr, -1).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_slice_string_empty() {
        let result = slice_string("", &Value::Nil, &Value::Nil, false).unwrap();
        assert_eq!(result.to_string(), "\"\"");
    }

    #[test]
    fn test_index_string_empty() {
        assert!(index_string("", 0).is_err());
    }
}
