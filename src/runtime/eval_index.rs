//! Index Operations for Interpreter - Extracted for 100% Coverage
//!
//! Handles array, string, tuple, object, and `DataFrame` indexing with
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
                        "Range start {i} is out of bounds for string of length {len}"
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            }
        }
        _ => {
            let type_name = start.type_name();
            return Err(InterpreterError::RuntimeError(format!(
                "Range start must be integer or nil, got {type_name}"
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
                        "Range end {i} is out of bounds for string of length {len}"
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            }
        }
        _ => {
            let type_name = end.type_name();
            return Err(InterpreterError::RuntimeError(format!(
                "Range end must be integer or nil, got {type_name}"
            )))
        }
    };

    // Validate range
    if start_idx > end_idx {
        return Err(InterpreterError::RuntimeError(format!(
            "Invalid range: start {start_idx} is greater than end {end_idx}"
        )));
    }

    if end_idx > len {
        return Err(InterpreterError::RuntimeError(format!(
            "Range end {end_idx} is out of bounds for string of length {len}"
        )));
    }

    // Perform the slice
    let sliced: String = chars[start_idx..end_idx].iter().collect();
    Ok(Value::from_string(sliced))
}

/// ARRAY-SLICE-FIX: Slice an array using a range like arr[0..3]
/// Supports: arr[0..3], arr[..3], arr[3..], arr[..], arr[0..=3] (inclusive)
/// Cyclomatic complexity: 9 (A+ standard: ≤10)
pub fn slice_array(
    array: &[Value],
    start: &Value,
    end: &Value,
    inclusive: bool,
) -> Result<Value, InterpreterError> {
    let len = array.len();

    // Extract start index (default to 0 for open ranges like ..5)
    let start_idx = match start {
        Value::Nil => 0,
        Value::Integer(i) => {
            if *i < 0 {
                let adjusted = len as i64 + i;
                if adjusted < 0 {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Range start {i} is out of bounds for array of length {len}"
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            }
        }
        _ => {
            let type_name = start.type_name();
            return Err(InterpreterError::RuntimeError(format!(
                "Range start must be integer or nil, got {type_name}"
            )));
        }
    };

    // Extract end index (default to len for open ranges like 5..)
    let end_idx = match end {
        Value::Nil => len,
        Value::Integer(i) => {
            let base_idx = if *i < 0 {
                let adjusted = len as i64 + i;
                if adjusted < 0 {
                    return Err(InterpreterError::RuntimeError(format!(
                        "Range end {i} is out of bounds for array of length {len}"
                    )));
                }
                adjusted as usize
            } else {
                *i as usize
            };
            // For inclusive ranges, add 1 to include the end index
            if inclusive {
                base_idx + 1
            } else {
                base_idx
            }
        }
        _ => {
            let type_name = end.type_name();
            return Err(InterpreterError::RuntimeError(format!(
                "Range end must be integer or nil, got {type_name}"
            )));
        }
    };

    // Validate range
    if start_idx > end_idx {
        return Err(InterpreterError::RuntimeError(format!(
            "Invalid range: start {start_idx} is greater than end {end_idx}"
        )));
    }

    // Clamp end_idx to len (allow slicing to end without error)
    let end_idx = end_idx.min(len);

    // Perform the slice
    let sliced: Vec<Value> = array[start_idx..end_idx].to_vec();
    Ok(Value::Array(Arc::from(sliced)))
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

    // === EXTREME TDD Round 23 - Coverage Push Tests ===

    #[test]
    fn test_index_array_boundary_negative() {
        let arr = vec![Value::Integer(1), Value::Integer(2)];
        // -2 should work (first element)
        assert_eq!(index_array(&arr, -2).unwrap(), Value::Integer(1));
        // -3 should fail (out of bounds)
        assert!(index_array(&arr, -3).is_err());
    }

    #[test]
    fn test_index_string_boundary_positive() {
        // Test exact boundary
        let result = index_string("ab", 1).unwrap();
        assert_eq!(result.to_string(), "\"b\"");
        // One past boundary should fail
        assert!(index_string("ab", 2).is_err());
    }

    #[test]
    fn test_slice_string_same_indices() {
        let result = slice_string("hello", &Value::Integer(2), &Value::Integer(2), false).unwrap();
        assert_eq!(result.to_string(), "\"\"");
    }

    #[test]
    fn test_slice_string_negative_both() {
        let result = slice_string("hello", &Value::Integer(-4), &Value::Integer(-1), false).unwrap();
        assert_eq!(result.to_string(), "\"ell\"");
    }

    #[test]
    fn test_index_tuple_string_element() {
        let tuple = vec![Value::from_string("first".to_string()), Value::from_string("second".to_string())];
        let result = index_tuple(&tuple, 1).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "second");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_object_multiple_keys() {
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), Value::Integer(1));
        obj.insert("b".to_string(), Value::Integer(2));
        obj.insert("c".to_string(), Value::Integer(3));
        assert_eq!(index_object(&obj, "a").unwrap(), Value::Integer(1));
        assert_eq!(index_object(&obj, "b").unwrap(), Value::Integer(2));
        assert_eq!(index_object(&obj, "c").unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_index_dataframe_row_second() {
        let columns = vec![
            DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
            },
        ];
        let row = index_dataframe_row(&columns, 2).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("x"), Some(&Value::Integer(3)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_dataframe_row_large_out_of_bounds() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        assert!(index_dataframe_row(&columns, 100).is_err());
    }

    #[test]
    fn test_slice_string_unicode() {
        let result = slice_string("日本語", &Value::Integer(0), &Value::Integer(2), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "日本");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_slice_string_unicode_negative() {
        let result = slice_string("日本語", &Value::Integer(-2), &Value::Nil, false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "本語");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_array_with_float_values() {
        let arr = vec![Value::Float(1.5), Value::Float(2.5), Value::Float(3.5)];
        assert_eq!(index_array(&arr, 1).unwrap(), Value::Float(2.5));
        assert_eq!(index_array(&arr, -2).unwrap(), Value::Float(2.5));
    }

    #[test]
    fn test_index_array_with_string_values() {
        let arr = vec![Value::from_string("a".to_string()), Value::from_string("b".to_string())];
        let result = index_array(&arr, 0).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "a");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_tuple_empty() {
        let tuple: Vec<Value> = vec![];
        assert!(index_tuple(&tuple, 0).is_err());
    }

    #[test]
    fn test_index_dataframe_column_multiple() {
        let columns = vec![
            DataFrameColumn {
                name: "col1".to_string(),
                values: vec![Value::Integer(1)],
            },
            DataFrameColumn {
                name: "col2".to_string(),
                values: vec![Value::Integer(2)],
            },
        ];
        let col1 = index_dataframe_column(&columns, "col1").unwrap();
        let col2 = index_dataframe_column(&columns, "col2").unwrap();
        if let (Value::Array(arr1), Value::Array(arr2)) = (col1, col2) {
            assert_eq!(arr1[0], Value::Integer(1));
            assert_eq!(arr2[0], Value::Integer(2));
        } else {
            panic!("Expected Arrays");
        }
    }

    #[test]
    fn test_slice_string_start_equals_len() {
        // Start at exact length should give empty string
        let result = slice_string("hi", &Value::Integer(2), &Value::Integer(2), false).unwrap();
        assert_eq!(result.to_string(), "\"\"");
    }

    #[test]
    fn test_index_object_nested_value() {
        let mut inner = HashMap::new();
        inner.insert("nested".to_string(), Value::Integer(42));
        let mut obj = HashMap::new();
        obj.insert("outer".to_string(), Value::Object(Arc::new(inner)));
        let result = index_object(&obj, "outer").unwrap();
        if let Value::Object(inner_obj) = result {
            assert_eq!(inner_obj.get("nested"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_array_nested() {
        let inner = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)].into_boxed_slice()));
        let arr = vec![inner.clone()];
        let result = index_array(&arr, 0).unwrap();
        if let Value::Array(inner_arr) = result {
            assert_eq!(inner_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_slice_string_single_char() {
        let result = slice_string("x", &Value::Integer(0), &Value::Integer(1), false).unwrap();
        assert_eq!(result.to_string(), "\"x\"");
    }

    #[test]
    fn test_index_dataframe_row_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::from_string("x".to_string()), Value::from_string("y".to_string())],
            },
            DataFrameColumn {
                name: "c".to_string(),
                values: vec![Value::Bool(true), Value::Bool(false)],
            },
        ];
        let row = index_dataframe_row(&columns, 1).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("a"), Some(&Value::Integer(2)));
            assert_eq!(obj.get("c"), Some(&Value::Bool(false)));
        } else {
            panic!("Expected Object");
        }
    }

    // === EXTREME TDD Round 135 - Push to 65+ Tests ===

    #[test]
    fn test_index_array_large_negative() {
        let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        // -100 is way out of bounds
        let result = index_array(&arr, -100);
        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert!(msg.contains("out of bounds"));
        }
    }

    #[test]
    fn test_index_array_large_positive() {
        let arr = vec![Value::Integer(1)];
        let result = index_array(&arr, 1000);
        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert!(msg.contains("out of bounds"));
        }
    }

    #[test]
    fn test_index_string_middle() {
        let result = index_string("abcdefg", 3).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "d");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_string_negative_middle() {
        let result = index_string("abcdefg", -4).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "d");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_slice_string_first_half() {
        let result = slice_string("abcdef", &Value::Integer(0), &Value::Integer(3), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "abc");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_slice_string_second_half() {
        let result = slice_string("abcdef", &Value::Integer(3), &Value::Nil, false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "def");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_tuple_with_nil() {
        let tuple = vec![Value::Nil, Value::Integer(1), Value::Nil];
        assert_eq!(index_tuple(&tuple, 0).unwrap(), Value::Nil);
        assert_eq!(index_tuple(&tuple, 2).unwrap(), Value::Nil);
    }

    #[test]
    fn test_index_tuple_boundary_negative() {
        let tuple = vec![Value::Integer(1), Value::Integer(2)];
        // -2 is first element
        assert_eq!(index_tuple(&tuple, -2).unwrap(), Value::Integer(1));
        // -3 is out of bounds
        assert!(index_tuple(&tuple, -3).is_err());
    }

    #[test]
    fn test_index_object_empty_string_key() {
        let mut obj = HashMap::new();
        obj.insert(String::new(), Value::Integer(42));
        assert_eq!(index_object(&obj, "").unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_index_object_mut_multiple_keys() {
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), Value::Integer(1));
        obj.insert("b".to_string(), Value::Integer(2));
        let cell = Arc::new(std::sync::Mutex::new(obj));
        assert_eq!(index_object_mut(&cell, "a").unwrap(), Value::Integer(1));
        assert_eq!(index_object_mut(&cell, "b").unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_index_dataframe_column_empty_values() {
        let columns = vec![DataFrameColumn {
            name: "empty".to_string(),
            values: vec![],
        }];
        let col = index_dataframe_column(&columns, "empty").unwrap();
        if let Value::Array(arr) = col {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_slice_string_with_spaces() {
        let result = slice_string("hello world", &Value::Integer(0), &Value::Integer(5), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "hello");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_slice_string_with_special_chars() {
        let result = slice_string("a!@#$b", &Value::Integer(1), &Value::Integer(5), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "!@#$");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_array_with_nil_values() {
        let arr = vec![Value::Nil, Value::Nil, Value::Nil];
        assert_eq!(index_array(&arr, 0).unwrap(), Value::Nil);
        assert_eq!(index_array(&arr, -1).unwrap(), Value::Nil);
    }

    #[test]
    fn test_index_array_with_bool_values() {
        let arr = vec![Value::Bool(true), Value::Bool(false)];
        assert_eq!(index_array(&arr, 0).unwrap(), Value::Bool(true));
        assert_eq!(index_array(&arr, 1).unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_slice_string_at_boundary() {
        // Slice exactly at the end
        let result = slice_string("abc", &Value::Integer(2), &Value::Integer(3), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "c");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_dataframe_row_first() {
        let columns = vec![DataFrameColumn {
            name: "col".to_string(),
            values: vec![Value::Integer(100), Value::Integer(200)],
        }];
        let row = index_dataframe_row(&columns, 0).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("col"), Some(&Value::Integer(100)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_tuple_with_mixed_types() {
        let tuple = vec![
            Value::Integer(1),
            Value::Float(2.5),
            Value::Bool(true),
            Value::from_string("test".to_string()),
            Value::Nil,
        ];
        assert_eq!(index_tuple(&tuple, 0).unwrap(), Value::Integer(1));
        assert_eq!(index_tuple(&tuple, 1).unwrap(), Value::Float(2.5));
        assert_eq!(index_tuple(&tuple, 2).unwrap(), Value::Bool(true));
        assert_eq!(index_tuple(&tuple, 4).unwrap(), Value::Nil);
    }

    #[test]
    fn test_slice_string_long_string() {
        let long_str = "a".repeat(100);
        let result = slice_string(&long_str, &Value::Integer(50), &Value::Integer(60), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.len(), 10);
        } else {
            panic!("Expected String");
        }
    }

    // === EXTREME TDD Round 159 - Coverage Push Tests ===

    #[test]
    fn test_index_array_first_element_r159() {
        let arr = vec![Value::from_string("first".to_string()), Value::from_string("second".to_string())];
        let result = index_array(&arr, 0).unwrap();
        assert_eq!(result.to_string(), "\"first\"");
    }

    #[test]
    fn test_index_array_last_element_r159() {
        let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let result = index_array(&arr, 2).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_index_array_negative_last_r159() {
        let arr = vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)];
        let result = index_array(&arr, -1).unwrap();
        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_index_string_first_char_r159() {
        let result = index_string("abc", 0).unwrap();
        assert_eq!(result.to_string(), "\"a\"");
    }

    #[test]
    fn test_index_string_last_char_r159() {
        let result = index_string("xyz", 2).unwrap();
        assert_eq!(result.to_string(), "\"z\"");
    }

    #[test]
    fn test_index_string_negative_last_r159() {
        let result = index_string("test", -1).unwrap();
        assert_eq!(result.to_string(), "\"t\"");
    }

    #[test]
    fn test_index_string_negative_first_r159() {
        let result = index_string("test", -4).unwrap();
        assert_eq!(result.to_string(), "\"t\"");
    }

    #[test]
    fn test_slice_string_middle_r159() {
        let result = slice_string("abcdefgh", &Value::Integer(2), &Value::Integer(6), false).unwrap();
        assert_eq!(result.to_string(), "\"cdef\"");
    }

    #[test]
    fn test_slice_string_single_char_r159() {
        let result = slice_string("hello", &Value::Integer(2), &Value::Integer(3), false).unwrap();
        assert_eq!(result.to_string(), "\"l\"");
    }

    #[test]
    fn test_index_tuple_first_r159() {
        let tuple = vec![Value::Bool(true), Value::Bool(false)];
        let result = index_tuple(&tuple, 0).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_index_tuple_negative_last_r159() {
        let tuple = vec![Value::Integer(1), Value::Integer(2)];
        let result = index_tuple(&tuple, -1).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_index_tuple_negative_first_r159() {
        let tuple = vec![Value::Integer(1), Value::Integer(2)];
        let result = index_tuple(&tuple, -2).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_index_object_nested_value_r159() {
        let mut inner = HashMap::new();
        inner.insert("inner".to_string(), Value::Integer(42));
        let mut obj = HashMap::new();
        obj.insert("outer".to_string(), Value::Object(Arc::new(inner)));
        let result = index_object(&obj, "outer").unwrap();
        if let Value::Object(o) = result {
            assert_eq!(o.get("inner"), Some(&Value::Integer(42)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_dataframe_row_last_r159() {
        let columns = vec![
            DataFrameColumn {
                name: "col".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
            },
        ];
        let row = index_dataframe_row(&columns, 2).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("col"), Some(&Value::Integer(3)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_index_dataframe_column_multiple_r159() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(2)],
            },
        ];
        let col_a = index_dataframe_column(&columns, "a").unwrap();
        let col_b = index_dataframe_column(&columns, "b").unwrap();
        if let (Value::Array(a), Value::Array(b)) = (col_a, col_b) {
            assert_eq!(a[0], Value::Integer(1));
            assert_eq!(b[0], Value::Integer(2));
        } else {
            panic!("Expected Arrays");
        }
    }

    #[test]
    fn test_slice_string_unicode_r159() {
        let result = slice_string("こんにちは", &Value::Integer(1), &Value::Integer(4), false).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.len(), 9); // 3 characters * 3 bytes each
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_index_array_various_types_r159() {
        let arr = vec![
            Value::Integer(1),
            Value::Float(2.5),
            Value::Bool(true),
            Value::Nil,
        ];
        assert_eq!(index_array(&arr, 0).unwrap(), Value::Integer(1));
        assert_eq!(index_array(&arr, 1).unwrap(), Value::Float(2.5));
        assert_eq!(index_array(&arr, 2).unwrap(), Value::Bool(true));
        assert_eq!(index_array(&arr, 3).unwrap(), Value::Nil);
    }

    #[test]
    fn test_index_object_empty_key_r159() {
        let mut obj = HashMap::new();
        obj.insert("".to_string(), Value::Integer(42));
        let result = index_object(&obj, "").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_index_dataframe_row_multiple_columns_r159() {
        let columns = vec![
            DataFrameColumn {
                name: "id".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![Value::from_string("Alice".to_string()), Value::from_string("Bob".to_string())],
            },
            DataFrameColumn {
                name: "active".to_string(),
                values: vec![Value::Bool(true), Value::Bool(false)],
            },
        ];
        let row = index_dataframe_row(&columns, 1).unwrap();
        if let Value::Object(obj) = row {
            assert_eq!(obj.get("id"), Some(&Value::Integer(2)));
            assert_eq!(obj.get("active"), Some(&Value::Bool(false)));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_slice_string_all_r159() {
        let result = slice_string("test", &Value::Integer(0), &Value::Integer(4), false).unwrap();
        assert_eq!(result.to_string(), "\"test\"");
    }

    #[test]
    fn test_index_tuple_empty_error_r159() {
        let tuple: Vec<Value> = vec![];
        let result = index_tuple(&tuple, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_index_object_mut_multiple_keys_r159() {
        let mut obj = HashMap::new();
        obj.insert("a".to_string(), Value::Integer(1));
        obj.insert("b".to_string(), Value::Integer(2));
        let cell = Arc::new(std::sync::Mutex::new(obj));
        assert_eq!(index_object_mut(&cell, "a").unwrap(), Value::Integer(1));
        assert_eq!(index_object_mut(&cell, "b").unwrap(), Value::Integer(2));
    }
}
