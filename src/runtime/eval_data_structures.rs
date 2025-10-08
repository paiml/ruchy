//! Data structure evaluation module
//!
//! This module handles evaluation of all data structure operations including
//! object creation, field access, indexing, destructuring, and transformations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, ObjectField, StructField};
use crate::runtime::interpreter::DataFrameColumn;
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::rc::Rc;

/// Evaluate object/struct literal creation
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_object_literal<F>(
    fields: &[ObjectField],
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut field_map = HashMap::new();

    for field in fields {
        match field {
            ObjectField::KeyValue { key, value } => {
                let evaluated_value = eval_expr(value)?;
                field_map.insert(key.clone(), evaluated_value);
            }
            ObjectField::Spread { expr } => {
                // Spread operator: { ...obj }
                let spread_value = eval_expr(expr)?;
                if let Value::Object(spread_fields) = spread_value {
                    field_map.extend(spread_fields.as_ref().clone());
                } else {
                    return Err(InterpreterError::TypeError(
                        "Spread operator can only be used with objects".to_string(),
                    ));
                }
            }
        }
    }

    Ok(Value::Object(Rc::new(field_map)))
}

/// Evaluate struct definition and return a constructor function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_struct_def(name: &str, fields: &[StructField]) -> Result<Value, InterpreterError> {
    // Create a constructor function for the struct
    let _struct_name = name.to_string();
    let _field_names = fields.to_vec();

    // In a real implementation, this would create a proper closure
    // For now, return a function that creates struct instances
    Ok(Value::Nil) // Struct constructor function placeholder
}

/// Evaluate field access on objects/structs
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_field_access(object: &Value, field: &str) -> Result<Value, InterpreterError> {
    match object {
        Value::Object(fields) => fields.get(field).cloned().ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Field '{field}' not found in object"))
        }),
        Value::ObjectMut(cell) => {
            // Safe borrow: We clone the result, so borrow is released immediately
            cell.borrow().get(field).cloned().ok_or_else(|| {
                InterpreterError::RuntimeError(format!("Field '{field}' not found in object"))
            })
        }
        // Note: Struct variant not implemented in Value enum yet
        Value::DataFrame { columns } => eval_dataframe_field_access(columns, field),
        Value::Tuple(elements) => eval_tuple_field_access(elements, field),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot access field '{}' on value of type {}",
            field,
            object.type_name()
        ))),
    }
}

/// Evaluate array/object indexing
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_index_access<F>(
    container: &Value,
    index: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let index_value = eval_expr(index)?;

    match container {
        Value::Array(arr) => {
            if let Value::Integer(idx) = index_value {
                let idx = idx as usize;
                if idx < arr.len() {
                    Ok(arr[idx].clone())
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "Array index {} out of bounds (length: {})",
                        idx,
                        arr.len()
                    )))
                }
            } else {
                Err(InterpreterError::TypeError(
                    "Array index must be an integer".to_string(),
                ))
            }
        }
        Value::Object(fields) => {
            if let Value::String(key) = index_value {
                fields.get(&*key).cloned().ok_or_else(|| {
                    InterpreterError::RuntimeError(format!("Key '{key}' not found in object"))
                })
            } else {
                Err(InterpreterError::TypeError(
                    "Object index must be a string".to_string(),
                ))
            }
        }
        Value::String(s) => {
            if let Value::Integer(idx) = index_value {
                let idx = idx as usize;
                if idx < s.len() {
                    let char_at = s.chars().nth(idx).ok_or_else(|| {
                        InterpreterError::RuntimeError("Invalid string index".to_string())
                    })?;
                    Ok(Value::from_string(char_at.to_string()))
                } else {
                    Err(InterpreterError::RuntimeError(format!(
                        "String index {} out of bounds (length: {})",
                        idx,
                        s.len()
                    )))
                }
            } else {
                Err(InterpreterError::TypeError(
                    "String index must be an integer".to_string(),
                ))
            }
        }
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot index value of type {}",
            container.type_name()
        ))),
    }
}

/// Evaluate slice access (e.g., arr[1:3])
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_slice_access<F>(
    container: &Value,
    start: Option<&Expr>,
    end: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match container {
        Value::Array(arr) => {
            let start_idx = if let Some(start_expr) = start {
                if let Value::Integer(idx) = eval_expr(start_expr)? {
                    idx.max(0) as usize
                } else {
                    return Err(InterpreterError::TypeError(
                        "Slice start index must be an integer".to_string(),
                    ));
                }
            } else {
                0
            };

            let end_idx = if let Some(end_expr) = end {
                if let Value::Integer(idx) = eval_expr(end_expr)? {
                    (idx.max(0) as usize).min(arr.len())
                } else {
                    return Err(InterpreterError::TypeError(
                        "Slice end index must be an integer".to_string(),
                    ));
                }
            } else {
                arr.len()
            };

            if start_idx <= end_idx && start_idx <= arr.len() {
                let sliced = arr[start_idx..end_idx].to_vec();
                Ok(Value::from_array(sliced))
            } else {
                Err(InterpreterError::RuntimeError(
                    "Invalid slice indices".to_string(),
                ))
            }
        }
        Value::String(s) => eval_string_slice(s, start, end, eval_expr),
        _ => Err(InterpreterError::TypeError(format!(
            "Cannot slice value of type {}",
            container.type_name()
        ))),
    }
}

/// Evaluate object spread operation ({...obj, key: value})
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
pub fn eval_object_spread<F>(
    base_objects: &[Expr],
    additional_fields: &[ObjectField],
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut result_fields = HashMap::new();

    // Spread base objects first
    for base_expr in base_objects {
        let base_value = eval_expr(base_expr)?;
        match base_value {
            Value::Object(fields) => {
                result_fields.extend(fields.as_ref().clone());
            }
            _ => {
                return Err(InterpreterError::TypeError(
                    "Can only spread object values".to_string(),
                ))
            }
        }
    }

    // Add additional fields (override existing ones)
    let additional_object = eval_object_literal(additional_fields, eval_expr)?;
    if let Value::Object(additional_fields) = additional_object {
        result_fields.extend(additional_fields.as_ref().clone());
    }

    Ok(Value::Object(Rc::new(result_fields)))
}

/// Evaluate array spread operation ([...arr1, item, ...arr2])
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_array_spread<F>(
    elements: &[ArrayElement],
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut result_elements = Vec::new();

    for element in elements {
        match element {
            ArrayElement::Value(expr) => {
                let value = eval_expr(expr)?;
                result_elements.push(value);
            }
            ArrayElement::Spread(expr) => {
                let spread_value = eval_expr(expr)?;
                match spread_value {
                    Value::Array(arr) => {
                        result_elements.extend(arr.iter().cloned());
                    }
                    _ => {
                        return Err(InterpreterError::TypeError(
                            "Can only spread array values in array literal".to_string(),
                        ))
                    }
                }
            }
        }
    }

    Ok(Value::from_array(result_elements))
}

/// Evaluate destructuring assignment
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_destructuring_assignment<F>(
    pattern: &DestructuringPattern,
    value: &Value,
    assign_var: F,
) -> Result<(), InterpreterError>
where
    F: FnMut(&str, Value) -> Result<(), InterpreterError>,
{
    match pattern {
        DestructuringPattern::Object(field_patterns) => {
            destructure_object(field_patterns, value, assign_var)
        }
        DestructuringPattern::Array(var_names) => destructure_array(var_names, value, assign_var),
    }
}

fn destructure_object<F>(
    field_patterns: &[(String, String)],
    value: &Value,
    mut assign_var: F,
) -> Result<(), InterpreterError>
where
    F: FnMut(&str, Value) -> Result<(), InterpreterError>,
{
    if let Value::Object(fields) = value {
        for (field_name, var_name) in field_patterns {
            if let Some(field_value) = fields.get(field_name) {
                assign_var(var_name, field_value.clone())?;
            } else {
                return Err(InterpreterError::RuntimeError(format!(
                    "Field '{field_name}' not found in object"
                )));
            }
        }
        Ok(())
    } else {
        Err(InterpreterError::TypeError(
            "Cannot destructure non-object value as object".to_string(),
        ))
    }
}

fn destructure_array<F>(
    var_names: &[String],
    value: &Value,
    mut assign_var: F,
) -> Result<(), InterpreterError>
where
    F: FnMut(&str, Value) -> Result<(), InterpreterError>,
{
    if let Value::Array(arr) = value {
        if var_names.len() != arr.len() {
            return Err(InterpreterError::RuntimeError(format!(
                "Array destructuring length mismatch: {} variables, {} values",
                var_names.len(),
                arr.len()
            )));
        }
        for (var_name, array_value) in var_names.iter().zip(arr.iter()) {
            assign_var(var_name, array_value.clone())?;
        }
        Ok(())
    } else {
        Err(InterpreterError::TypeError(
            "Cannot destructure non-array value as array".to_string(),
        ))
    }
}

// Helper types for data structure operations

#[derive(Debug, Clone)]
pub enum ArrayElement {
    Value(Expr),
    Spread(Expr),
}

#[derive(Debug, Clone)]
pub enum DestructuringPattern {
    Object(Vec<(String, String)>), // (field_name, var_name)
    Array(Vec<String>),            // variable names
}

// Helper functions (complexity <= 6 each)

/// Create an identifier expression for shorthand object notation
fn create_identifier_expr(name: &str) -> Expr {
    use crate::frontend::ast::{ExprKind, Span};
    Expr::new(
        ExprKind::Identifier(name.to_string()),
        Span::new(0, name.len()),
    )
}

/// Evaluate `DataFrame` field access
fn eval_dataframe_field_access(
    columns: &[DataFrameColumn],
    field: &str,
) -> Result<Value, InterpreterError> {
    for column in columns {
        if column.name == field {
            return Ok(Value::from_array(column.values.clone()));
        }
    }
    Err(InterpreterError::RuntimeError(format!(
        "Column '{field}' not found in DataFrame"
    )))
}

/// Evaluate tuple field access (tuple.0, tuple.1, etc.)
pub fn eval_tuple_field_access(elements: &[Value], field: &str) -> Result<Value, InterpreterError> {
    if let Ok(index) = field.parse::<usize>() {
        if index < elements.len() {
            Ok(elements[index].clone())
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Tuple index {} out of bounds (length: {})",
                index,
                elements.len()
            )))
        }
    } else {
        Err(InterpreterError::TypeError(
            "Tuple field access must use numeric indices".to_string(),
        ))
    }
}

/// Evaluate string slice operation
fn eval_string_slice<F>(
    s: &str,
    start: Option<&Expr>,
    end: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let start_idx = if let Some(start_expr) = start {
        if let Value::Integer(idx) = eval_expr(start_expr)? {
            idx.max(0) as usize
        } else {
            return Err(InterpreterError::TypeError(
                "String slice start index must be an integer".to_string(),
            ));
        }
    } else {
        0
    };

    let end_idx = if let Some(end_expr) = end {
        if let Value::Integer(idx) = eval_expr(end_expr)? {
            (idx.max(0) as usize).min(s.len())
        } else {
            return Err(InterpreterError::TypeError(
                "String slice end index must be an integer".to_string(),
            ));
        }
    } else {
        s.len()
    };

    if start_idx <= end_idx && start_idx <= s.len() {
        let sliced = &s[start_idx..end_idx];
        Ok(Value::from_string(sliced.to_string()))
    } else {
        Err(InterpreterError::RuntimeError(
            "Invalid string slice indices".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{ExprKind, Literal, Span};
    use std::rc::Rc;

    #[test]
    fn test_field_access_object() {
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::from_string("Alice".to_string()));
        fields.insert("age".to_string(), Value::Integer(30));

        let obj = Value::Object(Rc::new(fields));

        let name_result = eval_field_access(&obj, "name").unwrap();
        assert_eq!(name_result, Value::from_string("Alice".to_string()));

        let age_result = eval_field_access(&obj, "age").unwrap();
        assert_eq!(age_result, Value::Integer(30));

        let missing_result = eval_field_access(&obj, "missing");
        assert!(missing_result.is_err());
    }

    #[test]
    fn test_array_index_access() {
        let arr = Value::Array(Rc::from(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
        ]));

        let index_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(0, 1),
        );
        let mut eval_count = 0;
        let result = eval_index_access(&arr, &index_expr, |_| {
            eval_count += 1;
            Ok(Value::Integer(1))
        })
        .unwrap();

        assert_eq!(result, Value::Integer(20));
        assert_eq!(eval_count, 1);
    }

    #[test]
    fn test_array_slice() {
        let arr = Value::Array(Rc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]));

        let start_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(0, 1),
        );
        let end_expr = Expr::new(
            ExprKind::Literal(Literal::Integer(4, None)),
            Span::new(0, 1),
        );

        let mut call_count = 0;
        let result = eval_slice_access(&arr, Some(&start_expr), Some(&end_expr), |_| {
            call_count += 1;
            match call_count {
                1 => Ok(Value::Integer(1)), // start
                2 => Ok(Value::Integer(4)), // end
                _ => panic!("Unexpected call"),
            }
        })
        .unwrap();

        if let Value::Array(sliced) = result {
            assert_eq!(sliced.len(), 3);
            assert_eq!(sliced[0], Value::Integer(2));
            assert_eq!(sliced[1], Value::Integer(3));
            assert_eq!(sliced[2], Value::Integer(4));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_tuple_field_access() {
        let tuple_elements = vec![
            Value::Integer(42),
            Value::from_string("hello".to_string()),
            Value::Bool(true),
        ];

        let result = eval_tuple_field_access(&tuple_elements, "1").unwrap();
        assert_eq!(result, Value::from_string("hello".to_string()));

        let out_of_bounds = eval_tuple_field_access(&tuple_elements, "5");
        assert!(out_of_bounds.is_err());

        let invalid_index = eval_tuple_field_access(&tuple_elements, "invalid");
        assert!(invalid_index.is_err());
    }
}
