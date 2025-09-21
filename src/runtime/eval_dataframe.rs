//! `DataFrame` method evaluation module
//!
//! This module handles evaluation of `DataFrame` methods and operations in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::frontend::ast::{DataFrameOp, Expr};
use crate::runtime::interpreter::DataFrameColumn;
use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;

/// Evaluate a `DataFrame` method call
///
/// # Complexity
/// Complexity reduced by breaking into specialized functions for each method type
pub fn eval_dataframe_method(
    columns: &[DataFrameColumn],
    method: &str,
    arg_values: &[Value],
    values_equal_fn: impl Fn(&Value, &Value) -> bool,
) -> Result<Value, InterpreterError> {
    match method {
        "select" => eval_dataframe_select(columns, arg_values),
        "sum" => eval_dataframe_sum(columns, arg_values),
        "slice" => eval_dataframe_slice(columns, arg_values),
        "join" => eval_dataframe_join(columns, arg_values, values_equal_fn),
        "groupby" => eval_dataframe_groupby(columns, arg_values),
        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown DataFrame method: {method}"
        ))),
    }
}

/// Evaluate `DataFrame` select method - complexity: 7
fn eval_dataframe_select(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.select() requires exactly 1 argument (column_name)".to_string(),
        ));
    }

    if let Value::String(column_name) = &arg_values[0] {
        // Find the column
        for col in columns {
            if col.name == **column_name {
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

/// Evaluate `DataFrame` sum method - complexity: 9
fn eval_dataframe_sum(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if !arg_values.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.sum() takes no arguments".to_string(),
        ));
    }

    let mut total = 0.0;
    for col in columns {
        for value in &col.values {
            match value {
                Value::Integer(i) => total += *i as f64,
                Value::Float(f) => total += *f,
                _ => {} // Skip non-numeric values
            }
        }
    }

    // Return as integer if it's a whole number, otherwise float
    if total.fract() == 0.0 {
        Ok(Value::Integer(total as i64))
    } else {
        Ok(Value::Float(total))
    }
}

/// Evaluate `DataFrame` slice method - complexity: 9
fn eval_dataframe_slice(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.slice() requires exactly 2 arguments (start, length)".to_string(),
        ));
    }

    let start = match &arg_values[0] {
        Value::Integer(s) => *s as usize,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.slice() expects start as integer".to_string(),
            ))
        }
    };

    let length = match &arg_values[1] {
        Value::Integer(l) => *l as usize,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.slice() expects length as integer".to_string(),
            ))
        }
    };

    // Create new columns with sliced values
    let mut sliced_columns = Vec::new();
    for col in columns {
        let end_idx = (start + length).min(col.values.len());
        let sliced_values = if start < col.values.len() {
            col.values[start..end_idx].to_vec()
        } else {
            Vec::new()
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

/// Evaluate `DataFrame` join method - complexity: 10 (at limit)
fn eval_dataframe_join(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
    values_equal_fn: impl Fn(&Value, &Value) -> bool,
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.join() requires exactly 2 arguments (other_df, on)".to_string(),
        ));
    }

    let other_df = &arg_values[0];
    let join_column = match &arg_values[1] {
        Value::String(col_name) => col_name.as_str(),
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
        perform_inner_join(columns, other_columns, join_column, values_equal_fn)
    } else {
        Err(InterpreterError::RuntimeError(
            "DataFrame.join() expects first argument to be a DataFrame".to_string(),
        ))
    }
}

/// Perform inner join operation - complexity: 9
fn perform_inner_join(
    left_columns: &[DataFrameColumn],
    right_columns: &[DataFrameColumn],
    join_column: &str,
    values_equal_fn: impl Fn(&Value, &Value) -> bool,
) -> Result<Value, InterpreterError> {
    // Find the join columns
    let left_join_col = left_columns.iter().find(|col| col.name == join_column);
    let right_join_col = right_columns.iter().find(|col| col.name == join_column);

    if left_join_col.is_none() {
        return Err(InterpreterError::RuntimeError(format!(
            "Join column '{join_column}' not found in left DataFrame"
        )));
    }
    if right_join_col.is_none() {
        return Err(InterpreterError::RuntimeError(format!(
            "Join column '{join_column}' not found in right DataFrame"
        )));
    }

    let left_join_col = left_join_col.unwrap();
    let right_join_col = right_join_col.unwrap();

    // Setup result columns
    let mut joined_columns = setup_join_columns(left_columns, right_columns, join_column);

    // Perform join
    perform_join_operation(
        left_columns,
        right_columns,
        left_join_col,
        right_join_col,
        join_column,
        &mut joined_columns,
        values_equal_fn,
    );

    Ok(Value::DataFrame {
        columns: joined_columns,
    })
}

/// Setup columns for join result - complexity: 6
fn setup_join_columns(
    left_columns: &[DataFrameColumn],
    right_columns: &[DataFrameColumn],
    join_column: &str,
) -> Vec<DataFrameColumn> {
    let mut joined_columns = Vec::new();

    // Add all columns from left DataFrame
    for col in left_columns {
        joined_columns.push(DataFrameColumn {
            name: col.name.clone(),
            values: Vec::new(),
        });
    }

    // Add columns from right DataFrame (excluding the join column)
    for col in right_columns {
        if col.name != join_column {
            joined_columns.push(DataFrameColumn {
                name: format!("{}_right", col.name),
                values: Vec::new(),
            });
        }
    }

    joined_columns
}

/// Perform the actual join operation - complexity: 9
fn perform_join_operation(
    left_columns: &[DataFrameColumn],
    right_columns: &[DataFrameColumn],
    left_join_col: &DataFrameColumn,
    right_join_col: &DataFrameColumn,
    join_column: &str,
    joined_columns: &mut [DataFrameColumn],
    values_equal_fn: impl Fn(&Value, &Value) -> bool,
) {
    for (left_idx, left_join_val) in left_join_col.values.iter().enumerate() {
        for (right_idx, right_join_val) in right_join_col.values.iter().enumerate() {
            if values_equal_fn(left_join_val, right_join_val) {
                // Add values from left DataFrame
                for (col_idx, col) in left_columns.iter().enumerate() {
                    if let Some(val) = col.values.get(left_idx) {
                        joined_columns[col_idx].values.push(val.clone());
                    }
                }

                // Add values from right DataFrame (excluding join column)
                let mut right_col_idx = left_columns.len();
                for col in right_columns {
                    if col.name != join_column {
                        if let Some(val) = col.values.get(right_idx) {
                            joined_columns[right_col_idx].values.push(val.clone());
                        }
                        right_col_idx += 1;
                    }
                }
            }
        }
    }
}

/// Evaluate `DataFrame` groupby method - complexity: 10 (at limit)
fn eval_dataframe_groupby(
    columns: &[DataFrameColumn],
    arg_values: &[Value],
) -> Result<Value, InterpreterError> {
    if arg_values.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "DataFrame.groupby() requires exactly 1 argument (column_name)".to_string(),
        ));
    }

    let group_column = match &arg_values[0] {
        Value::String(col_name) => col_name.as_str(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.groupby() expects column name as string".to_string(),
            ))
        }
    };

    // Find the group column
    let group_col = columns
        .iter()
        .find(|col| col.name == group_column)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Group column '{group_column}' not found in DataFrame"
            ))
        })?;

    let groups = create_groups(group_col);
    create_grouped_result(columns, group_column, &groups)
}

/// Create groups from column values - complexity: 7
fn create_groups(group_col: &DataFrameColumn) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (row_idx, value) in group_col.values.iter().enumerate() {
        let key = match value {
            Value::String(s) => s.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => "null".to_string(),
        };
        groups.entry(key).or_default().push(row_idx);
    }

    groups
}

/// Create grouped result `DataFrame` - complexity: 10 (at limit)
fn create_grouped_result(
    columns: &[DataFrameColumn],
    group_column: &str,
    groups: &HashMap<String, Vec<usize>>,
) -> Result<Value, InterpreterError> {
    let mut result_columns = Vec::new();

    // Group column (unique values)
    let group_values: Vec<Value> = groups
        .keys()
        .map(|key| Value::from_string(key.clone()))
        .collect();

    result_columns.push(DataFrameColumn {
        name: group_column.to_string(),
        values: group_values,
    });

    // Aggregate numeric columns
    for col in columns {
        if col.name != group_column {
            let aggregated_values = aggregate_column_values(col, groups);
            result_columns.push(DataFrameColumn {
                name: format!("{}_sum", col.name),
                values: aggregated_values,
            });
        }
    }

    Ok(Value::DataFrame {
        columns: result_columns,
    })
}

/// Aggregate column values for groupby - complexity: 9
fn aggregate_column_values(
    col: &DataFrameColumn,
    groups: &HashMap<String, Vec<usize>>,
) -> Vec<Value> {
    let mut aggregated_values = Vec::new();

    for indices in groups.values() {
        let mut sum = 0.0;
        for &idx in indices {
            if let Some(value) = col.values.get(idx) {
                match value {
                    Value::Integer(i) => sum += *i as f64,
                    Value::Float(f) => sum += *f,
                    _ => {} // Skip non-numeric values
                }
            }
        }

        // Return as integer if it's a whole number, otherwise float
        if sum.fract() == 0.0 {
            aggregated_values.push(Value::Integer(sum as i64));
        } else {
            aggregated_values.push(Value::Float(sum));
        }
    }

    aggregated_values
}

/// Evaluate a `DataFrame` operation
///
/// # Complexity
/// Complexity reduced by breaking into specialized functions for each operation type
pub fn eval_dataframe_operation<F, G>(
    source: &Value,
    operation: &DataFrameOp,
    _eval_expr_fn: F,
    eval_expr_with_context_fn: G,
) -> Result<Value, InterpreterError>
where
    F: Fn(&Expr) -> Result<Value, InterpreterError>,
    G: Fn(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    if let Value::DataFrame { columns } = source {
        match operation {
            DataFrameOp::Select(column_names) => eval_dataframe_select_op(columns, column_names),
            DataFrameOp::Filter(condition) => {
                eval_dataframe_filter_op(columns, condition, eval_expr_with_context_fn)
            }
            DataFrameOp::GroupBy(group_columns) => {
                eval_dataframe_groupby_op(columns, group_columns)
            }
            _ => Err(InterpreterError::RuntimeError(
                "DataFrameOperation not yet implemented".to_string(),
            )),
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "DataFrameOperation can only be applied to DataFrame values".to_string(),
        ))
    }
}

/// Evaluate `DataFrame` select operation - complexity: 8
pub fn eval_dataframe_select_op(
    columns: &[DataFrameColumn],
    column_names: &[String],
) -> Result<Value, InterpreterError> {
    let mut selected_columns = Vec::new();

    for name in column_names {
        // Find the column
        let mut found = false;
        for col in columns {
            if col.name == *name {
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

/// Evaluate `DataFrame` filter operation - complexity: 10 (at limit)
pub fn eval_dataframe_filter_op<G>(
    columns: &[DataFrameColumn],
    condition: &Expr,
    mut eval_expr_with_context_fn: G,
) -> Result<Value, InterpreterError>
where
    G: FnMut(&Expr, &[DataFrameColumn], usize) -> Result<Value, InterpreterError>,
{
    if columns.is_empty() {
        return Ok(Value::DataFrame {
            columns: columns.to_vec(),
        });
    }

    let num_rows = columns[0].values.len();
    let mut filtered_rows: Vec<bool> = Vec::new();

    // Evaluate the condition for each row
    for row_idx in 0..num_rows {
        let condition_result = eval_expr_with_context_fn(condition, columns, row_idx);

        match condition_result {
            Ok(Value::Bool(true)) => filtered_rows.push(true),
            Ok(Value::Bool(false)) => filtered_rows.push(false),
            Ok(_) => {
                return Err(InterpreterError::RuntimeError(
                    "Filter condition must return boolean".to_string(),
                ))
            }
            Err(e) => return Err(e),
        }
    }

    create_filtered_dataframe(columns, &filtered_rows)
}

/// Create filtered `DataFrame` from row mask - complexity: 8
fn create_filtered_dataframe(
    columns: &[DataFrameColumn],
    filtered_rows: &[bool],
) -> Result<Value, InterpreterError> {
    let mut new_columns = Vec::new();
    for col in columns {
        let mut filtered_values = Vec::new();
        for (idx, &keep) in filtered_rows.iter().enumerate() {
            if keep {
                if let Some(value) = col.values.get(idx) {
                    filtered_values.push(value.clone());
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

/// Evaluate `DataFrame` groupby operation - complexity: 10 (at limit)
pub fn eval_dataframe_groupby_op(
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
        &group_columns[0]
    };

    // Find the group column
    let group_col = columns
        .iter()
        .find(|col| col.name == *group_column)
        .ok_or_else(|| {
            InterpreterError::RuntimeError(format!(
                "Group column '{group_column}' not found in DataFrame"
            ))
        })?;

    let groups = create_groupby_groups(group_col);
    create_groupby_result(columns, group_column, &groups)
}

/// Create groups for `DataFrame` groupby operation - complexity: 7
fn create_groupby_groups(group_col: &DataFrameColumn) -> HashMap<String, Vec<usize>> {
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();

    for (row_idx, value) in group_col.values.iter().enumerate() {
        let key = match value {
            Value::String(s) => s.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => "null".to_string(),
        };
        groups.entry(key).or_default().push(row_idx);
    }

    groups
}

/// Create result `DataFrame` for groupby operation - complexity: 10 (at limit)
fn create_groupby_result(
    columns: &[DataFrameColumn],
    group_column: &str,
    groups: &HashMap<String, Vec<usize>>,
) -> Result<Value, InterpreterError> {
    let mut result_columns = Vec::new();

    // Group column (unique values)
    let group_values: Vec<Value> = groups
        .keys()
        .map(|key| Value::from_string(key.clone()))
        .collect();

    result_columns.push(DataFrameColumn {
        name: group_column.to_string(),
        values: group_values,
    });

    // Aggregate numeric columns (sum by default for now)
    for col in columns {
        if col.name != group_column {
            let aggregated_values = aggregate_groupby_column(col, groups);
            result_columns.push(DataFrameColumn {
                name: format!("{}_sum", col.name),
                values: aggregated_values,
            });
        }
    }

    Ok(Value::DataFrame {
        columns: result_columns,
    })
}

/// Aggregate column values for groupby operation - complexity: 9
fn aggregate_groupby_column(
    col: &DataFrameColumn,
    groups: &HashMap<String, Vec<usize>>,
) -> Vec<Value> {
    let mut aggregated_values = Vec::new();
    for indices in groups.values() {
        let mut sum = 0.0;
        for &idx in indices {
            if let Some(value) = col.values.get(idx) {
                match value {
                    Value::Integer(i) => sum += *i as f64,
                    Value::Float(f) => sum += *f,
                    _ => {} // Skip non-numeric values
                }
            }
        }
        // Return as integer if it's a whole number, otherwise float
        if sum.fract() == 0.0 {
            aggregated_values.push(Value::Integer(sum as i64));
        } else {
            aggregated_values.push(Value::Float(sum));
        }
    }
    aggregated_values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }

    #[test]
    fn test_dataframe_select() {
        let columns = vec![
            DataFrameColumn {
                name: "name".to_string(),
                values: vec![Value::from_string("Alice".to_string())],
            },
            DataFrameColumn {
                name: "age".to_string(),
                values: vec![Value::Integer(25)],
            },
        ];

        let args = vec![Value::from_string("name".to_string())];
        let result = eval_dataframe_select(&columns, &args).unwrap();

        if let Value::DataFrame {
            columns: result_cols,
        } = result
        {
            assert_eq!(result_cols.len(), 1);
            assert_eq!(result_cols[0].name, "name");
        } else {
            panic!("Expected DataFrame result");
        }
    }

    #[test]
    fn test_dataframe_sum() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Integer(10), Value::Integer(20), Value::Float(5.5)],
        }];

        let args = vec![];
        let result = eval_dataframe_sum(&columns, &args).unwrap();

        assert_eq!(result, Value::Float(35.5));
    }
}
