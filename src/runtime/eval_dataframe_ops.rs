//! `DataFrame` operations evaluation module
//!
//! This module handles all DataFrame-specific operations including select, filter,
//! join, groupby, slice, and aggregations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{DataFrameOp, Expr};
use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::collections::HashMap;

/// Evaluate a `DataFrame` method call
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_dataframe_method(
    columns: &[DataFrameColumn],
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "select" => eval_dataframe_select(columns, arg_values),
        "sum" => eval_dataframe_sum(columns, arg_values),
        "slice" => eval_dataframe_slice(columns, arg_values),
        "join" => eval_dataframe_join(columns, arg_values),
        "groupby" => eval_dataframe_groupby(columns, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown DataFrame method: {method}"
        ))),
    }
}

/// Select specific columns by name
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_dataframe_select(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.select() requires exactly 1 argument (column_name)".to_string(),
        ));
    }

    if let Value::String(column_name) = &args[0] {
        for col in columns {
            if col.name == **column_name {
                // Return a DataFrame with just this column
                return Ok(Value::DataFrame {
                    columns: vec![col.clone()],
                });
            }
        }
        Err(InterpreterError::RuntimeError(format!(
            "Column '{column_name}' not found in DataFrame"
        )))
    } else {
        Err(InterpreterError::RuntimeError(
            "DataFrame.select() expects column name as string".to_string(),
        ))
    }
}

/// Sum all numeric values in all columns
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_dataframe_sum(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.sum() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    for col in columns {
        for val in &col.values {
            match val {
                Value::Integer(i) => total += *i as f64,
                Value::Float(f) => total += f,
                _ => {} // Skip non-numeric values
            }
        }
    }

    // Return as integer if it's a whole number, otherwise as float
    if total.fract() == 0.0 {
        Ok(Value::Integer(total as i64))
    } else {
        Ok(Value::Float(total))
    }
}

/// Slice `DataFrame` rows
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_dataframe_slice(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.slice() requires exactly 2 arguments (start, length)".to_string(),
        ));
    }

    let start = match &args[0] {
        Value::Integer(s) => *s as usize,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.slice() expects start as integer".to_string(),
            ))
        }
    };

    let length = match &args[1] {
        Value::Integer(l) => *l as usize,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.slice() expects length as integer".to_string(),
            ))
        }
    };

    let mut sliced_columns = Vec::new();

    for col in columns {
        let sliced_values = if start >= col.values.len() {
            Vec::new()
        } else {
            col.values[start..col.values.len().min(start + length)].to_vec()
        };

        sliced_columns.push(DataFrameColumn {
            name: col.name.clone(),
            values: sliced_values,
        });
    }

    Ok(Value::DataFrame {
        columns: sliced_columns,
    })
}

/// Join two `DataFrames`
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_dataframe_join(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.join() requires exactly 2 arguments (other_df, on)".to_string(),
        ));
    }

    let other_df = &args[0];
    let join_column = match &args[1] {
        Value::String(col_name) => &**col_name,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.join() expects 'on' as string column name".to_string(),
            ))
        }
    };

    if let Value::DataFrame {
        columns: other_columns,
    } = other_df
    {
        perform_dataframe_join(columns, other_columns, join_column)
    } else {
        Err(InterpreterError::RuntimeError(
            "DataFrame.join() expects first argument to be a DataFrame".to_string(),
        ))
    }
}

/// Perform the actual join operation (EXTREME TDD refactored)
///
/// # Complexity
/// Cyclomatic complexity: 4 (Toyota Way compliant)
fn perform_dataframe_join(
    left_cols: &[DataFrameColumn],
    right_cols: &[DataFrameColumn],
    join_column: &str,
) -> Result<Value, InterpreterError> {
    let join_cols = validate_and_find_join_columns(left_cols, right_cols, join_column)?;
    let mut joined_columns = initialize_result_columns(left_cols, right_cols, join_column);
    populate_joined_data(
        &mut joined_columns,
        left_cols,
        right_cols,
        &join_cols,
        join_column,
    );

    Ok(Value::DataFrame {
        columns: joined_columns,
    })
}

/// Validate and find join columns in both `DataFrames`
/// Complexity: 3 (Toyota Way compliant)
fn validate_and_find_join_columns<'a>(
    left_cols: &'a [DataFrameColumn],
    right_cols: &'a [DataFrameColumn],
    join_column: &str,
) -> Result<(&'a DataFrameColumn, &'a DataFrameColumn), InterpreterError> {
    let left_join_col = left_cols
        .iter()
        .find(|col| col.name == join_column)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Join column '{join_column}' not found in left DataFrame"
            ))
        })?;

    let right_join_col = right_cols
        .iter()
        .find(|col| col.name == join_column)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Join column '{join_column}' not found in right DataFrame"
            ))
        })?;

    Ok((left_join_col, right_join_col))
}

/// Initialize result columns structure for join
/// Complexity: 4 (Toyota Way compliant)
fn initialize_result_columns(
    left_cols: &[DataFrameColumn],
    right_cols: &[DataFrameColumn],
    join_column: &str,
) -> Vec<DataFrameColumn> {
    let mut joined_columns = Vec::new();

    // Add all columns from left DataFrame
    for col in left_cols {
        joined_columns.push(DataFrameColumn {
            name: col.name.clone(),
            values: Vec::new(),
        });
    }

    // Add columns from right DataFrame (excluding join column)
    for col in right_cols {
        if col.name != join_column {
            joined_columns.push(DataFrameColumn {
                name: format!("{}_right", col.name),
                values: Vec::new(),
            });
        }
    }

    joined_columns
}

/// Populate joined data by performing inner join operation
/// Complexity: 6 (Toyota Way compliant)
fn populate_joined_data(
    joined_columns: &mut [DataFrameColumn],
    left_cols: &[DataFrameColumn],
    right_cols: &[DataFrameColumn],
    join_cols: &(&DataFrameColumn, &DataFrameColumn),
    join_column: &str,
) {
    let (left_join_col, right_join_col) = join_cols;

    for (left_idx, left_key) in left_join_col.values.iter().enumerate() {
        for (right_idx, right_key) in right_join_col.values.iter().enumerate() {
            if left_key == right_key {
                add_matched_row(
                    joined_columns,
                    left_cols,
                    right_cols,
                    left_idx,
                    right_idx,
                    join_column,
                );
            }
        }
    }
}

/// Add a matched row to the result
/// Complexity: 5 (Toyota Way compliant)
fn add_matched_row(
    joined_columns: &mut [DataFrameColumn],
    left_cols: &[DataFrameColumn],
    right_cols: &[DataFrameColumn],
    left_idx: usize,
    right_idx: usize,
    join_column: &str,
) {
    // Add values from left DataFrame
    for (col_idx, col) in left_cols.iter().enumerate() {
        if let Some(val) = col.values.get(left_idx) {
            joined_columns[col_idx].values.push(val.clone());
        }
    }

    // Add values from right DataFrame (excluding join column)
    let mut right_col_idx = left_cols.len();
    for col in right_cols {
        if col.name != join_column {
            if let Some(val) = col.values.get(right_idx) {
                joined_columns[right_col_idx].values.push(val.clone());
            }
            right_col_idx += 1;
        }
    }
}

/// Group by a column
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_dataframe_groupby(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.groupby() requires exactly 1 argument (column_name)".to_string(),
        ));
    }

    let group_column = match &args[0] {
        Value::String(col_name) => &**col_name,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.groupby() expects column name as string".to_string(),
            ))
        }
    };

    // Find the group column
    let group_col = columns.iter().find(|col| col.name == group_column);
    if group_col.is_none() {
        return Err(InterpreterError::RuntimeError(format!(
            "Group column '{group_column}' not found in DataFrame"
        )));
    }
    let group_col = group_col.unwrap();

    perform_groupby_aggregation(columns, group_col, group_column)
}

/// Perform groupby aggregation (helper for complexity reduction)
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn perform_groupby_aggregation(
    columns: &[DataFrameColumn],
    group_col: &DataFrameColumn,
    group_column: &str,
) -> Result<Value, InterpreterError> {
    let groups = create_group_mapping_for_aggregation(group_col);
    let mut result_columns = initialize_group_columns_for_aggregation(&groups, group_column);
    add_aggregated_numeric_columns_for_groupby(&mut result_columns, columns, &groups, group_column);

    Ok(Value::DataFrame {
        columns: result_columns,
    })
}

/// Create mapping from group keys to row indices for aggregation
fn create_group_mapping_for_aggregation(
    group_col: &DataFrameColumn,
) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, val) in group_col.values.iter().enumerate() {
        let key = val.to_string();
        groups.entry(key).or_default().push(idx);
    }
    groups
}

/// Initialize result columns with group column for aggregation
fn initialize_group_columns_for_aggregation(
    groups: &HashMap<String, Vec<usize>>,
    group_column: &str,
) -> Vec<DataFrameColumn> {
    let mut result_columns = Vec::new();
    let mut group_values = Vec::new();

    for key in groups.keys() {
        group_values.push(Value::from_string(key.clone()));
    }

    result_columns.push(DataFrameColumn {
        name: group_column.to_string(),
        values: group_values,
    });

    result_columns
}

/// Add aggregated numeric columns to result for groupby
fn add_aggregated_numeric_columns_for_groupby(
    result_columns: &mut Vec<DataFrameColumn>,
    columns: &[DataFrameColumn],
    groups: &HashMap<String, Vec<usize>>,
    group_column: &str,
) {
    for col in columns {
        if col.name == group_column {
            continue;
        }

        if is_numeric_column_for_aggregation(col) {
            let aggregated_column = create_aggregated_column_for_groupby(col, groups);
            result_columns.push(aggregated_column);
        }
    }
}

/// Check if a column contains numeric data for aggregation
fn is_numeric_column_for_aggregation(col: &DataFrameColumn) -> bool {
    col.values
        .iter()
        .any(|v| matches!(v, Value::Integer(_) | Value::Float(_)))
}

/// Create aggregated column for a numeric column in groupby
fn create_aggregated_column_for_groupby(
    col: &DataFrameColumn,
    groups: &HashMap<String, Vec<usize>>,
) -> DataFrameColumn {
    let mut aggregated_values = Vec::new();

    for key in groups.keys() {
        let sum_value = calculate_group_sum_for_aggregation(col, &groups[key]);
        aggregated_values.push(sum_value);
    }

    DataFrameColumn {
        name: format!("{}_sum", col.name),
        values: aggregated_values,
    }
}

/// Calculate sum for a group of indices in a column for aggregation
fn calculate_group_sum_for_aggregation(col: &DataFrameColumn, indices: &[usize]) -> Value {
    let mut sum = 0.0;
    let mut has_numeric = false;

    for &idx in indices {
        if let Some(val) = col.values.get(idx) {
            match val {
                Value::Integer(i) => {
                    sum += *i as f64;
                    has_numeric = true;
                }
                Value::Float(f) => {
                    sum += f;
                    has_numeric = true;
                }
                _ => {} // Skip non-numeric values
            }
        }
    }

    if has_numeric {
        if sum.fract() == 0.0 {
            Value::Integer(sum as i64)
        } else {
            Value::Float(sum)
        }
    } else {
        Value::Nil
    }
}

/// Evaluate `DataFrame` filter with condition
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
pub fn eval_dataframe_filter<F>(
    columns: &[DataFrameColumn],
    condition: &Expr,
    mut eval_with_context: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    if columns.is_empty() {
        return Ok(Value::DataFrame {
            columns: columns.to_vec(),
        });
    }

    let num_rows = columns[0].values.len();
    let mut keep_mask = Vec::with_capacity(num_rows);

    // Evaluate condition for each row
    for row_idx in 0..num_rows {
        let condition_result = eval_with_context(condition, columns, row_idx)?;

        let keep = match condition_result {
            Value::Bool(b) => b,
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "Filter condition must evaluate to boolean".to_string(),
                ))
            }
        };

        keep_mask.push(keep);
    }

    // Create new DataFrame with only rows that meet the condition
    let mut new_columns = Vec::new();

    for col in columns {
        let mut filtered_values = Vec::new();
        for (idx, &keep) in keep_mask.iter().enumerate() {
            if keep {
                if let Some(val) = col.values.get(idx) {
                    filtered_values.push(val.clone());
                }
            }
        }
        new_columns.push(DataFrameColumn {
            name: col.name.clone(),
            values: filtered_values,
        });
    }

    Ok(Value::DataFrame {
        columns: new_columns,
    })
}

/// Evaluate `DataFrame` operation from AST
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
pub fn eval_dataframe_operation<F>(
    columns: Vec<DataFrameColumn>,
    operation: &DataFrameOp,
    eval_with_context: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    match operation {
        DataFrameOp::Select(column_names) => eval_dataframe_select_multiple(&columns, column_names),
        DataFrameOp::Filter(condition) => {
            eval_dataframe_filter(&columns, condition, eval_with_context)
        }
        DataFrameOp::GroupBy(group_columns) => {
            eval_dataframe_groupby_multiple(&columns, group_columns)
        }
        _ => Err(InterpreterError::RuntimeError(
            "DataFrameOperation not yet implemented".to_string(),
        )),
    }
}

/// Select multiple columns by name
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_dataframe_select_multiple(
    columns: &[DataFrameColumn],
    column_names: &[String],
) -> Result<Value, InterpreterError> {
    let mut selected_columns = Vec::new();

    for name in column_names {
        let mut found = false;
        for col in columns {
            if &col.name == name {
                selected_columns.push(col.clone());
                found = true;
                break;
            }
        }
        if !found {
            return Err(InterpreterError::RuntimeError(format!(
                "Column '{name}' not found in DataFrame"
            )));
        }
    }

    Ok(Value::DataFrame {
        columns: selected_columns,
    })
}

/// Group by multiple columns
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits)
fn eval_dataframe_groupby_multiple(
    columns: &[DataFrameColumn],
    group_columns: &[String],
) -> Result<Value, InterpreterError> {
    // Parser limitation: use first column as default when no columns provided
    let group_column = if group_columns.is_empty() {
        if columns.is_empty() {
            return Err(InterpreterError::RuntimeError(
                "Cannot group by empty DataFrame".to_string(),
            ));
        }
        &columns[0].name // Default to first column
    } else {
        &group_columns[0] // Use only first group column for now
    };

    // Find the group column
    let group_col = columns.iter().find(|col| col.name == *group_column);
    if group_col.is_none() {
        return Err(InterpreterError::RuntimeError(format!(
            "Group column '{group_column}' not found in DataFrame"
        )));
    }
    let group_col = group_col.unwrap();

    perform_groupby_aggregation(columns, group_col, group_column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dataframe_sum() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Float(1.5), Value::Float(2.5), Value::Float(3.5)],
            },
        ];

        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(13.5));
    }

    #[test]
    fn test_dataframe_select() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(4)],
            },
        ];

        let args = vec![Value::from_string("a".to_string())];
        let result = eval_dataframe_select(&columns, &args).unwrap();

        if let Value::DataFrame {
            columns: result_cols,
        } = result
        {
            assert_eq!(result_cols.len(), 1);
            assert_eq!(result_cols[0].name, "a");
        } else {
            panic!("Expected DataFrame result");
        }
    }

    #[test]
    fn test_dataframe_slice() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![
                Value::Integer(1),
                Value::Integer(2),
                Value::Integer(3),
                Value::Integer(4),
            ],
        }];

        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = eval_dataframe_slice(&columns, &args).unwrap();

        if let Value::DataFrame {
            columns: result_cols,
        } = result
        {
            assert_eq!(result_cols[0].values.len(), 2);
            assert_eq!(result_cols[0].values[0], Value::Integer(2));
            assert_eq!(result_cols[0].values[1], Value::Integer(3));
        } else {
            panic!("Expected DataFrame result");
        }
    }
}
