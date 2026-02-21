//! `DataFrame` operations evaluation module
//!
//! This module handles all DataFrame-specific operations including select, filter,
//! join, groupby, slice, and aggregations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{DataFrameOp, Expr};
use crate::runtime::validation::{validate_arg_count, validate_arg_range};
use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::collections::HashMap;

/// Evaluate a `DataFrame` method call
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
pub fn eval_dataframe_method(
    columns: &[DataFrameColumn],
    method: &str,
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    match method {
        "select" => eval_dataframe_select(columns, arg_values),
        "sum" => eval_dataframe_sum(columns, arg_values),
        "mean" => eval_dataframe_mean(columns, arg_values),
        "max" => eval_dataframe_max(columns, arg_values),
        "min" => eval_dataframe_min(columns, arg_values),
        "std" => eval_dataframe_std(columns, arg_values),
        "var" => eval_dataframe_var(columns, arg_values),
        "slice" => eval_dataframe_slice(columns, arg_values),
        "join" => eval_dataframe_join(columns, arg_values),
        "groupby" => eval_dataframe_groupby(columns, arg_values),
        "rows" => eval_dataframe_rows(columns, arg_values),
        "columns" => eval_dataframe_columns_count(columns, arg_values),
        "column_names" => eval_dataframe_column_names(columns, arg_values),
        "sort_by" => eval_dataframe_sort_by(columns, arg_values),
        "get" => eval_dataframe_get(columns, arg_values),
        "to_csv" => eval_dataframe_to_csv(columns, arg_values),
        "to_json" => eval_dataframe_to_json(columns, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown DataFrame method: {method}"
        ))),
    }
}

/// Get the number of rows in the `DataFrame`
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_dataframe_rows(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.rows() takes no arguments".to_string(),
        ));
    }

    // Return the number of rows (length of first column, or 0 if no columns)
    let row_count = columns.first().map_or(0, |col| col.values.len());

    Ok(Value::Integer(row_count as i64))
}

/// Get the number of columns in the `DataFrame`
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_dataframe_columns_count(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.columns() takes no arguments".to_string(),
        ));
    }

    Ok(Value::Integer(columns.len() as i64))
}

/// Get the column names as an array of strings
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_dataframe_column_names(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.column_names() takes no arguments".to_string(),
        ));
    }

    // Return array of column names as strings
    let names: Vec<Value> = columns
        .iter()
        .map(|col| Value::from_string(col.name.clone()))
        .collect();

    Ok(Value::from_array(names))
}

/// Sort `DataFrame` by column values
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits, reduced from 9)
fn eval_dataframe_sort_by(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    validate_arg_range("DataFrame.sort_by", args, 1, 2)?;

    // Get column name
    let col_name = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.sort_by() expects column name as string".to_string(),
            ))
        }
    };

    // Check for descending flag
    let descending = if args.len() == 2 {
        match &args[1] {
            Value::Bool(b) => *b,
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.sort_by() descending flag must be boolean".to_string(),
                ))
            }
        }
    } else {
        false
    };

    // Find the sort column
    let sort_col_idx = columns
        .iter()
        .position(|c| c.name == col_name)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Column '{col_name}' not found in DataFrame"))
        })?;

    // Create indices and sort them based on column values
    let mut indices: Vec<usize> = (0..columns[sort_col_idx].values.len()).collect();
    indices.sort_by(|&a, &b| {
        let val_a = &columns[sort_col_idx].values[a];
        let val_b = &columns[sort_col_idx].values[b];
        let cmp = compare_values_for_sort(val_a, val_b);
        if descending {
            cmp.reverse()
        } else {
            cmp
        }
    });

    // Reorder all columns based on sorted indices
    let mut sorted_columns = Vec::new();
    for col in columns {
        let sorted_values: Vec<Value> = indices.iter().map(|&i| col.values[i].clone()).collect();
        sorted_columns.push(DataFrameColumn {
            name: col.name.clone(),
            values: sorted_values,
        });
    }

    Ok(Value::DataFrame {
        columns: sorted_columns,
    })
}

/// Compare two values for sorting
/// Complexity: 5 (within Toyota Way limits)
fn compare_values_for_sort(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Integer(ia), Value::Integer(ib)) => ia.cmp(ib),
        (Value::Float(fa), Value::Float(fb)) => fa.partial_cmp(fb).unwrap_or(Ordering::Equal),
        (Value::Integer(i), Value::Float(f)) | (Value::Float(f), Value::Integer(i)) => {
            let i_as_f = *i as f64;
            i_as_f.partial_cmp(f).unwrap_or(Ordering::Equal)
        }
        (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
        (Value::Bool(ba), Value::Bool(bb)) => ba.cmp(bb),
        _ => Ordering::Equal, // Default for incomparable types
    }
}

/// Select specific columns by name
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn find_column_by_name<'a>(
    columns: &'a [DataFrameColumn],
    name: &str,
) -> Result<&'a DataFrameColumn, InterpreterError> {
    columns.iter().find(|col| col.name == name).ok_or_else(|| {
        InterpreterError::RuntimeError(format!("Column '{name}' not found in DataFrame"))
    })
}

fn select_multiple_columns(
    columns: &[DataFrameColumn],
    col_names: &[Value],
) -> Result<Value, InterpreterError> {
    let mut selected = Vec::new();
    for name_val in col_names {
        if let Value::String(column_name) = name_val {
            selected.push(find_column_by_name(columns, column_name)?.clone());
        } else {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.select() array elements must be strings".to_string(),
            ));
        }
    }
    Ok(Value::DataFrame { columns: selected })
}

fn eval_dataframe_select(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    validate_arg_count("DataFrame.select", args, 1)?;

    match &args[0] {
        Value::String(column_name) => {
            let col = find_column_by_name(columns, column_name)?;
            Ok(Value::DataFrame {
                columns: vec![col.clone()],
            })
        }
        Value::Array(col_names) => select_multiple_columns(columns, col_names),
        _ => Err(InterpreterError::RuntimeError(
            "DataFrame.select() expects column name as string or array of strings".to_string(),
        )),
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

/// Calculate mean (average) of all numeric values in all columns
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_dataframe_mean(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.mean() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    let mut count = 0;

    for col in columns {
        for val in &col.values {
            match val {
                Value::Integer(i) => {
                    total += *i as f64;
                    count += 1;
                }
                Value::Float(f) => {
                    total += f;
                    count += 1;
                }
                _ => {} // Skip non-numeric values
            }
        }
    }

    // Avoid division by zero
    if count == 0 {
        return Ok(Value::Integer(0));
    }

    let mean = total / f64::from(count);

    // Return as integer if it's a whole number, otherwise as float
    if mean.fract() == 0.0 {
        Ok(Value::Integer(mean as i64))
    } else {
        Ok(Value::Float(mean))
    }
}

/// Find maximum numeric value across all columns
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_dataframe_max(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.max() takes no arguments".to_string(),
        ));
    }

    let mut max_value: Option<f64> = None;

    for col in columns {
        for val in &col.values {
            let numeric_val = match val {
                Value::Integer(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None, // Skip non-numeric values
            };

            if let Some(v) = numeric_val {
                max_value = Some(match max_value {
                    Some(current_max) => current_max.max(v),
                    None => v,
                });
            }
        }
    }

    // Return the max value or error if no numeric values found
    match max_value {
        Some(max) => {
            if max.fract() == 0.0 {
                Ok(Value::Integer(max as i64))
            } else {
                Ok(Value::Float(max))
            }
        }
        None => Err(InterpreterError::RuntimeError(
            "DataFrame.max() found no numeric values".to_string(),
        )),
    }
}

/// Find minimum numeric value across all columns
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_dataframe_min(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.min() takes no arguments".to_string(),
        ));
    }

    let mut min_value: Option<f64> = None;

    for col in columns {
        for val in &col.values {
            let numeric_val = match val {
                Value::Integer(i) => Some(*i as f64),
                Value::Float(f) => Some(*f),
                _ => None, // Skip non-numeric values
            };

            if let Some(v) = numeric_val {
                min_value = Some(match min_value {
                    Some(current_min) => current_min.min(v),
                    None => v,
                });
            }
        }
    }

    // Return the min value or error if no numeric values found
    match min_value {
        Some(min) => {
            if min.fract() == 0.0 {
                Ok(Value::Integer(min as i64))
            } else {
                Ok(Value::Float(min))
            }
        }
        None => Err(InterpreterError::RuntimeError(
            "DataFrame.min() found no numeric values".to_string(),
        )),
    }
}

/// Calculate standard deviation of all numeric values in all columns
///
/// # Complexity
/// Cyclomatic complexity: 10 (at Toyota Way limit)
///
/// # Formula
/// std = sqrt(variance) = sqrt(sum((x - mean)^2) / N)
fn eval_dataframe_std(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.std() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    let mut count = 0;
    let mut values: Vec<f64> = Vec::new();

    // Collect all numeric values and calculate mean
    for col in columns {
        for val in &col.values {
            match val {
                Value::Integer(i) => {
                    let f = *i as f64;
                    total += f;
                    count += 1;
                    values.push(f);
                }
                Value::Float(f) => {
                    total += f;
                    count += 1;
                    values.push(*f);
                }
                _ => {} // Skip non-numeric values
            }
        }
    }

    // Handle empty or single-value cases
    if count == 0 || count == 1 {
        return Ok(Value::Float(0.0));
    }

    let mean = total / f64::from(count);

    // Calculate variance: sum((x - mean)^2) / N
    let variance: f64 = values
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / f64::from(count);

    // Standard deviation is sqrt(variance)
    let std = variance.sqrt();

    Ok(Value::Float(std))
}

/// Calculate variance of all numeric values in all columns
///
/// # Complexity
/// Cyclomatic complexity: 10 (at Toyota Way limit)
///
/// # Formula
/// var = sum((x - mean)^2) / N
fn eval_dataframe_var(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.var() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    let mut count = 0;
    let mut values: Vec<f64> = Vec::new();

    // Collect all numeric values and calculate mean
    for col in columns {
        for val in &col.values {
            match val {
                Value::Integer(i) => {
                    let f = *i as f64;
                    total += f;
                    count += 1;
                    values.push(f);
                }
                Value::Float(f) => {
                    total += f;
                    count += 1;
                    values.push(*f);
                }
                _ => {} // Skip non-numeric values
            }
        }
    }

    // Handle empty or single-value cases
    if count == 0 || count == 1 {
        return Ok(Value::Float(0.0));
    }

    let mean = total / f64::from(count);

    // Calculate variance: sum((x - mean)^2) / N
    let variance: f64 = values
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / f64::from(count);

    Ok(Value::Float(variance))
}

/// Slice `DataFrame` rows
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_dataframe_slice(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    validate_arg_count("DataFrame.slice", args, 2)?;

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
    validate_arg_count("DataFrame.join", args, 2)?;

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
    validate_arg_count("DataFrame.groupby", args, 1)?;

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
    let group_col = group_col.expect("group_col cannot be None after is_none() check on line 764");

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
    let group_col = group_col.expect("group_col cannot be None after is_none() check on line 1039");

    perform_groupby_aggregation(columns, group_col, group_column)
}

/// Export `DataFrame` to CSV format
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_dataframe_to_csv(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.to_csv() takes no arguments".to_string(),
        ));
    }

    // Handle empty DataFrame
    if columns.is_empty() {
        return Ok(Value::from_string(String::new()));
    }

    let mut csv = String::new();

    // Build header row
    let header: Vec<String> = columns.iter().map(|col| col.name.clone()).collect();
    csv.push_str(&header.join(","));
    csv.push('\n');

    // Build data rows
    let num_rows = columns.first().map_or(0, |col| col.values.len());
    for row_idx in 0..num_rows {
        let row_values: Vec<String> = columns
            .iter()
            .map(|col| {
                col.values
                    .get(row_idx)
                    .map(format_value_for_csv)
                    .unwrap_or_default()
            })
            .collect();

        csv.push_str(&row_values.join(","));
        csv.push('\n');
    }

    Ok(Value::from_string(csv))
}

/// Format a value for CSV output
/// Complexity: 4 (within Toyota Way limits)
fn format_value_for_csv(value: &Value) -> String {
    match value {
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.to_string(), // Note: Real CSV would need escaping
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

/// Export `DataFrame` to JSON format (array of objects)
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
fn eval_dataframe_to_json(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.to_json() takes no arguments".to_string(),
        ));
    }

    // Handle empty DataFrame
    if columns.is_empty() {
        return Ok(Value::from_string("[]".to_string()));
    }

    let num_rows = columns.first().map_or(0, |col| col.values.len());

    // Handle no rows
    if num_rows == 0 {
        return Ok(Value::from_string("[]".to_string()));
    }

    let mut json = String::from("[");

    // Build array of objects
    for row_idx in 0..num_rows {
        if row_idx > 0 {
            json.push(',');
        }

        json.push('{');

        for (col_idx, col) in columns.iter().enumerate() {
            if col_idx > 0 {
                json.push(',');
            }

            // Add field name
            json.push_str(&format!("\"{}\":", col.name));

            // Add field value
            if let Some(value) = col.values.get(row_idx) {
                json.push_str(&format_value_for_json(value));
            } else {
                json.push_str("null");
            }
        }

        json.push('}');
    }

    json.push(']');

    Ok(Value::from_string(json))
}

/// Format a value for JSON output
/// Complexity: 5 (within Toyota Way limits)
fn format_value_for_json(value: &Value) -> String {
    match value {
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{s}\""), // Note: Real JSON would need escaping
        Value::Bool(b) => b.to_string(),
        Value::Nil => "null".to_string(),
        _ => "null".to_string(),
    }
}

/// Get a specific value from `DataFrame` by column name and row index
///
/// Usage: `df.get("column_name", row_index)`
///
/// # Complexity
/// Cyclomatic complexity: 7 (within Toyota Way limits)
fn eval_dataframe_get(
    columns: &[DataFrameColumn],
    args: &[Value],
) -> Result<Value, InterpreterError> {
    validate_arg_count("DataFrame.get", args, 2)?;

    // Extract column name (first argument)
    let column_name = match &args[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.get() first argument must be a string (column name)".to_string(),
            ))
        }
    };

    // Extract row index (second argument)
    let row_index = match &args[1] {
        Value::Integer(i) => {
            if *i < 0 {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.get() row index must be non-negative".to_string(),
                ));
            }
            *i as usize
        }
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.get() second argument must be an integer (row index)".to_string(),
            ))
        }
    };

    // Find the column
    let column = columns
        .iter()
        .find(|col| col.name == *column_name)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!("Column '{column_name}' not found"))
        })?;

    // Get the value at the row index
    column.values.get(row_index).cloned().ok_or_else(|| {
        InterpreterError::RuntimeError(format!(
            "Row index {} out of bounds (DataFrame has {} rows)",
            row_index,
            column.values.len()
        ))
    })
}

#[cfg(test)]
#[path = "eval_dataframe_ops_tests.rs"]
mod tests;
