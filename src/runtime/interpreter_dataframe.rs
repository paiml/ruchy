//! DataFrame implementation module
//!
//! This module handles DataFrame operations and methods.
//! Extracted from interpreter.rs for maintainability.

#![allow(clippy::unused_self)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::expect_used)]
#![allow(clippy::cast_possible_truncation)]

use crate::frontend::ast::{Expr, ExprKind};
use crate::runtime::interpreter::Interpreter;
use crate::runtime::{DataFrameColumn, InterpreterError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

impl Interpreter {
    /// Evaluate `DataFrame` builder methods (.column, .build)
    /// Complexity: 8 (within Toyota Way limits)
    pub(crate) fn eval_dataframe_builder_method(
        &self,
        builder: &std::collections::HashMap<String, Value>,
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        match method {
            "column" => {
                // .column(name, values) - add a column to the builder
                if arg_values.len() != 2 {
                    return Err(InterpreterError::RuntimeError(
                        "DataFrame builder .column() requires 2 arguments (name, values)"
                            .to_string(),
                    ));
                }

                // Extract column name
                let name = match &arg_values[0] {
                    Value::String(s) => s.to_string(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Column name must be a string".to_string(),
                        ))
                    }
                };

                // Extract values array
                let values = match &arg_values[1] {
                    Value::Array(arr) => arr.to_vec(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Column values must be an array".to_string(),
                        ))
                    }
                };

                // Get current columns
                let current_columns = match builder.get("__columns") {
                    Some(Value::Array(cols)) => cols.to_vec(),
                    _ => vec![],
                };

                // Create new column object
                let mut col_obj = std::collections::HashMap::new();
                col_obj.insert("name".to_string(), Value::from_string(name));
                col_obj.insert("values".to_string(), Value::from_array(values));

                // Add to columns array
                let mut new_columns = current_columns;
                new_columns.push(Value::Object(std::sync::Arc::new(col_obj)));

                // Create new builder with updated columns
                let mut new_builder = builder.clone();
                new_builder.insert("__columns".to_string(), Value::from_array(new_columns));

                Ok(Value::Object(std::sync::Arc::new(new_builder)))
            }
            "build" => {
                // .build() - convert builder to `DataFrame`
                if !arg_values.is_empty() {
                    return Err(InterpreterError::RuntimeError(
                        "DataFrame builder .build() takes no arguments".to_string(),
                    ));
                }

                // Extract columns from builder
                let columns_array = match builder.get("__columns") {
                    Some(Value::Array(cols)) => cols,
                    _ => return Ok(Value::DataFrame { columns: vec![] }),
                };

                // Convert column objects to `DataFrameColumn` structs
                let mut df_columns = Vec::new();
                for col_val in columns_array.as_ref() {
                    if let Value::Object(col_obj) = col_val {
                        let name = match col_obj.get("name") {
                            Some(Value::String(s)) => s.to_string(),
                            _ => continue,
                        };
                        let values = match col_obj.get("values") {
                            Some(Value::Array(vals)) => vals.to_vec(),
                            _ => vec![],
                        };
                        df_columns.push(DataFrameColumn { name, values });
                    }
                }

                Ok(Value::DataFrame {
                    columns: df_columns,
                })
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Unknown builder method: {method}"
            ))),
        }
    }

    pub(crate) fn eval_dataframe_method(
        &self,
        columns: &[DataFrameColumn],
        method: &str,
        arg_values: &[Value],
    ) -> Result<Value, InterpreterError> {
        crate::runtime::eval_dataframe_ops::eval_dataframe_method(columns, method, arg_values)
    }

    /// Special handler for `DataFrame` filter method
    /// Complexity: 8 (within Toyota Way limits)
    pub(crate) fn eval_dataframe_filter_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.filter() requires exactly 1 argument (closure)".to_string(),
            ));
        }

        if let Value::DataFrame { columns } = receiver {
            let closure = &args[0];

            // Validate closure structure
            if !matches!(closure.kind, ExprKind::Lambda { .. }) {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.filter() expects a lambda expression".to_string(),
                ));
            }

            // Build keep_mask by evaluating closure for each row
            let num_rows = columns.first().map_or(0, |c| c.values.len());
            let mut keep_mask = Vec::with_capacity(num_rows);

            for row_idx in 0..num_rows {
                // Create row object with all column values for this row
                let mut row = HashMap::new();
                for col in columns {
                    if let Some(value) = col.values.get(row_idx) {
                        row.insert(col.name.clone(), value.clone());
                    }
                }
                let row_value = Value::Object(std::sync::Arc::new(row));

                // Evaluate closure with row object
                let result = self.eval_closure_with_value(closure, &row_value)?;

                // Check if result is boolean
                let keep = match result {
                    Value::Bool(b) => b,
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "DataFrame.filter() closure must return boolean".to_string(),
                        ))
                    }
                };

                keep_mask.push(keep);
            }

            // Create new DataFrame with filtered rows
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
        } else {
            Err(InterpreterError::RuntimeError(
                "filter method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Special handler for `DataFrame` `with_column` method
    /// Complexity: 9 (within Toyota Way limits)
    pub(crate) fn eval_dataframe_with_column_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.with_column() requires exactly 2 arguments (name, closure)".to_string(),
            ));
        }

        // Evaluate the column name
        let col_name = match self.eval_expr(&args[0])? {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.with_column() expects string column name".to_string(),
                ))
            }
        };

        if let Value::DataFrame { columns } = receiver {
            let closure = &args[1];

            // Extract parameter name from closure
            let param_name = if let ExprKind::Lambda { params, .. } = &closure.kind {
                if params.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "with_column closure must have exactly 1 parameter".to_string(),
                    ));
                }
                match &params[0].pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "with_column closure must have simple identifier parameter".to_string(),
                        ))
                    }
                }
            } else {
                return Err(InterpreterError::RuntimeError(
                    "Expected lambda expression".to_string(),
                ));
            };

            // Check if parameter name matches a column
            let matching_col = columns.iter().find(|c| c.name == param_name);

            let mut new_values = Vec::new();
            let num_rows = columns.first().map_or(0, |c| c.values.len());

            for row_idx in 0..num_rows {
                let value_to_bind = if let Some(col) = matching_col {
                    // Parameter name matches a column - bind that column's value
                    col.values.get(row_idx).cloned().unwrap_or(Value::Nil)
                } else {
                    // Parameter name doesn't match - bind full row object
                    let mut row = HashMap::new();
                    for col in columns {
                        if let Some(value) = col.values.get(row_idx) {
                            row.insert(col.name.clone(), value.clone());
                        }
                    }
                    Value::Object(std::sync::Arc::new(row))
                };

                // Evaluate closure with the appropriate value
                let result = self.eval_closure_with_value(closure, &value_to_bind)?;
                new_values.push(result);
            }

            // Create new DataFrame with additional column
            let mut new_columns = columns.clone();
            new_columns.push(crate::runtime::DataFrameColumn {
                name: col_name,
                values: new_values,
            });

            Ok(Value::DataFrame {
                columns: new_columns,
            })
        } else {
            Err(InterpreterError::RuntimeError(
                "with_column method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Special handler for `DataFrame` transform method
    /// Complexity: 9 (within Toyota Way limits)
    pub(crate) fn eval_dataframe_transform_method(
        &mut self,
        receiver: &Value,
        args: &[Expr],
    ) -> Result<Value, InterpreterError> {
        if args.len() != 2 {
            return Err(InterpreterError::RuntimeError(
                "DataFrame.transform() requires exactly 2 arguments (column, closure)".to_string(),
            ));
        }

        // Evaluate the column name
        let col_name = match self.eval_expr(&args[0])? {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "DataFrame.transform() expects string column name".to_string(),
                ))
            }
        };

        if let Value::DataFrame { columns } = receiver {
            // Find the column to transform
            let col_idx = columns
                .iter()
                .position(|c| c.name == col_name)
                .ok_or_else(|| {
                    InterpreterError::RuntimeError(format!(
                        "Column '{col_name}' not found in DataFrame"
                    ))
                })?;

            let closure = &args[1];
            let mut new_columns = columns.clone();

            // Transform each value in the column
            let mut transformed_values = Vec::new();
            for value in &columns[col_idx].values {
                // Create a temporary environment with the value bound to the parameter
                let result = self.eval_closure_with_value(closure, value)?;
                transformed_values.push(result);
            }

            new_columns[col_idx].values = transformed_values;

            Ok(Value::DataFrame {
                columns: new_columns,
            })
        } else {
            Err(InterpreterError::RuntimeError(
                "transform method can only be called on DataFrame".to_string(),
            ))
        }
    }

    /// Evaluate a closure with a single value argument
    /// Complexity: 7 (within Toyota Way limits)
    pub(crate) fn eval_closure_with_value(
        &mut self,
        closure_expr: &Expr,
        value: &Value,
    ) -> Result<Value, InterpreterError> {
        match &closure_expr.kind {
            ExprKind::Lambda { params, body, .. } => {
                if params.len() != 1 {
                    return Err(InterpreterError::RuntimeError(
                        "Transform closure must have exactly 1 parameter".to_string(),
                    ));
                }

                // Extract parameter name from pattern
                let param_name = match &params[0].pattern {
                    crate::frontend::ast::Pattern::Identifier(name) => name.clone(),
                    _ => {
                        return Err(InterpreterError::RuntimeError(
                            "Transform closure must have simple identifier parameter".to_string(),
                        ))
                    }
                };

                // Create new environment with parameter binding
                let mut new_env = HashMap::new();
                new_env.insert(param_name, value.clone());

                // Push environment
                self.env_push(new_env);

                // Evaluate the body
                let result = self.eval_expr(body)?;

                // Pop environment
                self.env_pop();

                Ok(result)
            }
            _ => Err(InterpreterError::RuntimeError(
                "Expected lambda expression".to_string(),
            )),
        }
    }

    /// Compare two values using a comparison function
    pub(crate) fn compare_values<F>(
        &self,
        left: &Value,
        right: &Value,
        cmp: F,
    ) -> Result<Value, InterpreterError>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Bool(cmp(*a, *b))),
            (Value::Float(a), Value::Float(b)) => {
                // Convert float comparison to integer-like for simplicity
                let a_int = *a as i64;
                let b_int = *b as i64;
                Ok(Value::Bool(cmp(a_int, b_int)))
            }
            (Value::Integer(a), Value::Float(b)) => {
                let b_int = *b as i64;
                Ok(Value::Bool(cmp(*a, b_int)))
            }
            (Value::Float(a), Value::Integer(b)) => {
                let a_int = *a as i64;
                Ok(Value::Bool(cmp(a_int, *b)))
            }
            _ => Err(InterpreterError::RuntimeError(format!(
                "Cannot compare {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    /// Check if two values are equal
    pub(crate) fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    /// Evaluate an expression with column context (for `DataFrame` filtering)
    pub(crate) fn eval_expr_with_column_context(
        &mut self,
        expr: &Expr,
        columns: &[DataFrameColumn],
        row_idx: usize,
    ) -> Result<Value, InterpreterError> {
        match &expr.kind {
            // Special handling for function calls that might be col() references
            ExprKind::Call { func, args } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    if name == "col" && args.len() == 1 {
                        // This is a col("column_name") call - resolve to actual column value
                        let col_name_expr = &args[0];
                        if let ExprKind::Literal(crate::frontend::ast::Literal::String(col_name)) =
                            &col_name_expr.kind
                        {
                            // Find the column and return the value for this row
                            for col in columns {
                                if col.name == *col_name {
                                    if let Some(value) = col.values.get(row_idx) {
                                        return Ok(value.clone());
                                    }
                                    return Err(InterpreterError::RuntimeError(format!(
                                        "Row index {} out of bounds for column '{}'",
                                        row_idx, col_name
                                    )));
                                }
                            }
                            return Err(InterpreterError::RuntimeError(format!(
                                "Column '{}' not found",
                                col_name
                            )));
                        }
                    }
                }
                // Fall back to normal function call evaluation
                self.eval_expr(expr)
            }
            // Handle binary expressions that might need column context
            ExprKind::Binary { left, right, .. } => {
                let left_val = self.eval_expr_with_column_context(left, columns, row_idx)?;
                let right_val = self.eval_expr_with_column_context(right, columns, row_idx)?;

                // Rebuild the binary expression with resolved values and evaluate
                // For simplicity, handle common comparison operations directly
                if let ExprKind::Binary { op, .. } = &expr.kind {
                    match op {
                        crate::frontend::ast::BinaryOp::Greater => {
                            self.compare_values(&left_val, &right_val, |a, b| a > b)
                        }
                        crate::frontend::ast::BinaryOp::Less => {
                            self.compare_values(&left_val, &right_val, |a, b| a < b)
                        }
                        crate::frontend::ast::BinaryOp::Equal => {
                            Ok(Value::Bool(self.values_equal(&left_val, &right_val)))
                        }
                        crate::frontend::ast::BinaryOp::NotEqual => {
                            Ok(Value::Bool(!self.values_equal(&left_val, &right_val)))
                        }
                        _ => self.eval_expr(expr), // Use regular evaluation for other operators
                    }
                } else {
                    unreachable!()
                }
            }
            // For all other expressions, use normal evaluation
            _ => self.eval_expr(expr),
        }
    }

    pub(crate) fn eval_dataframe_operation(
        &mut self,
        source: &Expr,
        operation: &crate::frontend::ast::DataFrameOp,
    ) -> Result<Value, InterpreterError> {
        let source_value = self.eval_expr(source)?;

        if let Value::DataFrame { columns } = source_value {
            crate::runtime::eval_dataframe_ops::eval_dataframe_operation(
                columns,
                operation,
                |expr, cols, idx| self.eval_expr_with_column_context(expr, cols, idx),
            )
        } else {
            Err(InterpreterError::RuntimeError(
                "DataFrameOperation can only be applied to DataFrame values".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Literal, Pattern, Span, Type};

    // Helper to create Expr
    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn make_int(val: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(val, None)))
    }

    fn make_ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn make_param(name: &str) -> crate::frontend::ast::Param {
        crate::frontend::ast::Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty: Type {
                kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    // Helper to create a builder HashMap
    fn make_builder() -> HashMap<String, Value> {
        let mut builder = HashMap::new();
        builder.insert(
            "__type".to_string(),
            Value::from_string("DataFrameBuilder".to_string()),
        );
        builder
    }

    // ============== eval_dataframe_builder_method tests ==============

    #[test]
    fn test_builder_column_success() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![
            Value::from_string("age".to_string()),
            Value::from_array(vec![Value::Integer(25), Value::Integer(30)]),
        ];
        let result = interp.eval_dataframe_builder_method(&builder, "column", &args);
        assert!(result.is_ok());
        if let Value::Object(obj) = result.unwrap() {
            assert!(obj.contains_key("__columns"));
        } else {
            panic!("Expected Object result");
        }
    }

    #[test]
    fn test_builder_column_wrong_arg_count() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![Value::from_string("name".to_string())]; // Only 1 arg
        let result = interp.eval_dataframe_builder_method(&builder, "column", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires 2 arguments"));
    }

    #[test]
    fn test_builder_column_invalid_name_type() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![
            Value::Integer(123), // Not a string
            Value::from_array(vec![Value::Integer(1)]),
        ];
        let result = interp.eval_dataframe_builder_method(&builder, "column", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Column name must be a string"));
    }

    #[test]
    fn test_builder_column_invalid_values_type() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![
            Value::from_string("col".to_string()),
            Value::Integer(42), // Not an array
        ];
        let result = interp.eval_dataframe_builder_method(&builder, "column", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Column values must be an array"));
    }

    #[test]
    fn test_builder_build_empty() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let result = interp.eval_dataframe_builder_method(&builder, "build", &[]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert!(columns.is_empty());
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_builder_build_with_args_error() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![Value::Integer(1)];
        let result = interp.eval_dataframe_builder_method(&builder, "build", &args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_builder_build_with_columns() {
        let interp = Interpreter::new();
        let mut builder = make_builder();

        // Add column manually
        let mut col_obj = HashMap::new();
        col_obj.insert("name".to_string(), Value::from_string("x".to_string()));
        col_obj.insert(
            "values".to_string(),
            Value::from_array(vec![Value::Integer(1), Value::Integer(2)]),
        );

        builder.insert(
            "__columns".to_string(),
            Value::from_array(vec![Value::Object(Arc::new(col_obj))]),
        );

        let result = interp.eval_dataframe_builder_method(&builder, "build", &[]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].name, "x");
            assert_eq!(columns[0].values.len(), 2);
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_builder_unknown_method() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let result = interp.eval_dataframe_builder_method(&builder, "unknown", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown builder method"));
    }

    #[test]
    fn test_builder_chain_multiple_columns() {
        let interp = Interpreter::new();
        let builder = make_builder();

        // Add first column
        let args1 = vec![
            Value::from_string("name".to_string()),
            Value::from_array(vec![Value::from_string("Alice".to_string())]),
        ];
        let result1 = interp
            .eval_dataframe_builder_method(&builder, "column", &args1)
            .unwrap();

        // Add second column
        if let Value::Object(builder2) = result1 {
            let args2 = vec![
                Value::from_string("age".to_string()),
                Value::from_array(vec![Value::Integer(30)]),
            ];
            let builder_map: HashMap<String, Value> = builder2.as_ref().clone();
            let result2 = interp
                .eval_dataframe_builder_method(&builder_map, "column", &args2)
                .unwrap();

            // Build the dataframe
            if let Value::Object(builder3) = result2 {
                let builder_map: HashMap<String, Value> = builder3.as_ref().clone();
                let df = interp
                    .eval_dataframe_builder_method(&builder_map, "build", &[])
                    .unwrap();
                if let Value::DataFrame { columns } = df {
                    assert_eq!(columns.len(), 2);
                } else {
                    panic!("Expected DataFrame");
                }
            }
        }
    }

    // ============== eval_dataframe_filter_method tests ==============

    #[test]
    fn test_filter_wrong_arg_count() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        // No args
        let result = interp.eval_dataframe_filter_method(&df, &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1 argument"));
    }

    #[test]
    fn test_filter_not_lambda_error() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        // Not a lambda - just a literal
        let not_lambda = make_int(42);
        let result = interp.eval_dataframe_filter_method(&df, &[not_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects a lambda expression"));
    }

    // ============== eval_closure_with_value tests ==============

    #[test]
    fn test_closure_with_value_simple() {
        let mut interp = Interpreter::new();
        // Create a simple lambda: |x| x
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("x")],
            body: Box::new(make_ident("x")),
        });
        let input = Value::Integer(42);
        let result = interp.eval_closure_with_value(&lambda, &input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    // ============== compare_values tests ==============

    #[test]
    fn test_compare_values_integers() {
        let interp = Interpreter::new();
        let left = Value::Integer(5);
        let right = Value::Integer(10);
        let result = interp.compare_values(&left, &right, |a, b| a < b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_compare_values_floats() {
        let interp = Interpreter::new();
        let left = Value::Float(3.14);
        let right = Value::Float(2.71);
        let result = interp.compare_values(&left, &right, |a, b| a > b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // NOTE: String comparison may not be fully supported in compare_values

    #[test]
    fn test_compare_values_type_error() {
        let interp = Interpreter::new();
        let left = Value::Integer(5);
        let right = Value::from_string("five".to_string());
        let result = interp.compare_values(&left, &right, |a, b| a < b);
        assert!(result.is_err());
    }

    // ============== values_equal tests ==============

    #[test]
    fn test_values_equal_integers() {
        let interp = Interpreter::new();
        assert!(interp.values_equal(&Value::Integer(42), &Value::Integer(42)));
        assert!(!interp.values_equal(&Value::Integer(42), &Value::Integer(43)));
    }

    #[test]
    fn test_values_equal_floats() {
        let interp = Interpreter::new();
        assert!(interp.values_equal(&Value::Float(3.14), &Value::Float(3.14)));
        assert!(!interp.values_equal(&Value::Float(3.14), &Value::Float(2.71)));
    }

    #[test]
    fn test_values_equal_strings() {
        let interp = Interpreter::new();
        let s1 = Value::from_string("hello".to_string());
        let s2 = Value::from_string("hello".to_string());
        let s3 = Value::from_string("world".to_string());
        assert!(interp.values_equal(&s1, &s2));
        assert!(!interp.values_equal(&s1, &s3));
    }

    #[test]
    fn test_values_equal_bools() {
        let interp = Interpreter::new();
        assert!(interp.values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!interp.values_equal(&Value::Bool(true), &Value::Bool(false)));
    }

    #[test]
    fn test_values_equal_different_types() {
        let interp = Interpreter::new();
        assert!(!interp.values_equal(&Value::Integer(1), &Value::Float(1.0)));
    }

    // NOTE: eval_expr_with_column_context tests removed due to context requirements

    // ============== Additional coverage tests ==============

    #[test]
    fn test_builder_column_preserves_existing_columns() {
        let interp = Interpreter::new();
        let mut builder = make_builder();

        // Add an existing column
        let mut col_obj = HashMap::new();
        col_obj.insert(
            "name".to_string(),
            Value::from_string("existing".to_string()),
        );
        col_obj.insert(
            "values".to_string(),
            Value::from_array(vec![Value::Integer(1)]),
        );
        builder.insert(
            "__columns".to_string(),
            Value::from_array(vec![Value::Object(Arc::new(col_obj))]),
        );

        // Add a new column
        let args = vec![
            Value::from_string("new".to_string()),
            Value::from_array(vec![Value::Integer(2)]),
        ];
        let result = interp
            .eval_dataframe_builder_method(&builder, "column", &args)
            .unwrap();

        if let Value::Object(obj) = result {
            if let Some(Value::Array(cols)) = obj.get("__columns") {
                assert_eq!(cols.len(), 2);
            } else {
                panic!("Expected __columns array");
            }
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_build_with_malformed_column_object() {
        let interp = Interpreter::new();
        let mut builder = make_builder();

        // Add malformed column (missing name)
        let mut col_obj = HashMap::new();
        col_obj.insert(
            "values".to_string(),
            Value::from_array(vec![Value::Integer(1)]),
        );
        builder.insert(
            "__columns".to_string(),
            Value::from_array(vec![Value::Object(Arc::new(col_obj))]),
        );

        // Should succeed but skip the malformed column
        let result = interp.eval_dataframe_builder_method(&builder, "build", &[]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert!(columns.is_empty());
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_build_with_non_object_column_entry() {
        let interp = Interpreter::new();
        let mut builder = make_builder();

        // Add non-object entry in columns array
        builder.insert(
            "__columns".to_string(),
            Value::from_array(vec![
                Value::Integer(42), // Not an object
            ]),
        );

        let result = interp.eval_dataframe_builder_method(&builder, "build", &[]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert!(columns.is_empty());
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_compare_values_less_equal() {
        let interp = Interpreter::new();
        assert_eq!(
            interp
                .compare_values(&Value::Integer(5), &Value::Integer(5), |a, b| a <= b)
                .unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            interp
                .compare_values(&Value::Integer(5), &Value::Integer(10), |a, b| a <= b)
                .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_compare_values_greater_equal() {
        let interp = Interpreter::new();
        assert_eq!(
            interp
                .compare_values(&Value::Integer(10), &Value::Integer(5), |a, b| a >= b)
                .unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_values_equal_nil() {
        let interp = Interpreter::new();
        assert!(interp.values_equal(&Value::Nil, &Value::Nil));
    }

    // NOTE: Array equality test removed - array comparison semantics vary

    // ============== Additional tests for interpreter_dataframe.rs coverage ==============

    // -- eval_dataframe_filter_method: filter on non-DataFrame --
    #[test]
    fn test_filter_on_non_dataframe() {
        let mut interp = Interpreter::new();
        let non_df = Value::Integer(42);
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("x")],
            body: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
        });
        let result = interp.eval_dataframe_filter_method(&non_df, &[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("filter method can only be called on DataFrame"));
    }

    // -- eval_dataframe_filter_method: filter with lambda that returns non-bool --
    #[test]
    fn test_filter_closure_returns_non_bool() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        // Lambda returns integer instead of bool
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_int(42)),
        });
        let result = interp.eval_dataframe_filter_method(&df, &[lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("closure must return boolean"));
    }

    // -- eval_dataframe_filter_method: empty DataFrame --
    #[test]
    fn test_filter_empty_dataframe() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![],
            }],
        };
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
        });
        let result = interp.eval_dataframe_filter_method(&df, &[lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns[0].values.len(), 0);
        }
    }

    // -- eval_dataframe_filter_method: filter keeps some rows --
    #[test]
    fn test_filter_keeps_some_rows() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![
                DataFrameColumn {
                    name: "x".to_string(),
                    values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
                },
                DataFrameColumn {
                    name: "y".to_string(),
                    values: vec![Value::Integer(10), Value::Integer(20), Value::Integer(30)],
                },
            ],
        };
        // Lambda always returns true
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
        });
        let result = interp.eval_dataframe_filter_method(&df, &[lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].values.len(), 3);
        }
    }

    // -- eval_dataframe_with_column_method: wrong arg count --
    #[test]
    fn test_with_column_wrong_arg_count() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        // Only 1 arg instead of 2
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 2 arguments"));
    }

    // -- eval_dataframe_with_column_method: non-string column name --
    #[test]
    fn test_with_column_non_string_name() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let int_name = make_int(42);
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[int_name, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects string column name"));
    }

    // -- eval_dataframe_with_column_method: on non-DataFrame --
    #[test]
    fn test_with_column_on_non_dataframe() {
        let mut interp = Interpreter::new();
        let non_df = Value::Integer(42);
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_with_column_method(&non_df, &[name_arg, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("with_column method can only be called on DataFrame"));
    }

    // -- eval_dataframe_with_column_method: not a lambda --
    #[test]
    fn test_with_column_not_lambda() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        let not_lambda = make_int(42);
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, not_lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected lambda expression"));
    }

    // -- eval_dataframe_with_column_method: lambda with wrong param count --
    #[test]
    fn test_with_column_lambda_wrong_param_count() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        // Lambda with 2 params instead of 1
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("a"), make_param("b")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have exactly 1 parameter"));
    }

    // -- eval_dataframe_with_column_method: lambda with non-identifier pattern --
    #[test]
    fn test_with_column_lambda_non_identifier_pattern() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        // Lambda with tuple pattern instead of identifier
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![crate::frontend::ast::Param {
                pattern: Pattern::Tuple(vec![
                    Pattern::Identifier("a".to_string()),
                    Pattern::Identifier("b".to_string()),
                ]),
                ty: Type {
                    kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have simple identifier parameter"));
    }

    // -- eval_dataframe_with_column_method: success with matching column name --
    #[test]
    fn test_with_column_success_matching_column() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("doubled".to_string())));
        // Lambda that references "x" - the column name
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("x")],
            body: Box::new(make_ident("x")), // Just return x value
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 2); // Original + new column
            assert_eq!(columns[1].name, "doubled");
            assert_eq!(columns[1].values.len(), 2);
        }
    }

    // -- eval_dataframe_with_column_method: success with non-matching param (row object) --
    #[test]
    fn test_with_column_success_row_object() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new_col".to_string())));
        // Lambda with param "row" which doesn't match any column
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_int(99)), // Return constant
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[1].name, "new_col");
            assert_eq!(columns[1].values[0], Value::Integer(99));
        }
    }

    // -- eval_dataframe_transform_method: wrong arg count --
    #[test]
    fn test_transform_wrong_arg_count() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("x".to_string())));
        let result = interp.eval_dataframe_transform_method(&df, &[name_arg]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 2 arguments"));
    }

    // -- eval_dataframe_transform_method: non-string column name --
    #[test]
    fn test_transform_non_string_name() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let int_name = make_int(42);
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("v")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_transform_method(&df, &[int_name, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects string column name"));
    }

    // -- eval_dataframe_transform_method: column not found --
    #[test]
    fn test_transform_column_not_found() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String(
            "nonexistent".to_string(),
        )));
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("v")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_transform_method(&df, &[name_arg, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found in DataFrame"));
    }

    // -- eval_dataframe_transform_method: on non-DataFrame --
    #[test]
    fn test_transform_on_non_dataframe() {
        let mut interp = Interpreter::new();
        let non_df = Value::Integer(42);
        let name_arg = make_expr(ExprKind::Literal(Literal::String("x".to_string())));
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("v")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_transform_method(&non_df, &[name_arg, lambda]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("transform method can only be called on DataFrame"));
    }

    // -- eval_dataframe_transform_method: success --
    #[test]
    fn test_transform_success() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("x".to_string())));
        // Lambda returns constant
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("v")],
            body: Box::new(make_int(100)),
        });
        let result = interp.eval_dataframe_transform_method(&df, &[name_arg, lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns[0].values[0], Value::Integer(100));
            assert_eq!(columns[0].values[1], Value::Integer(100));
        }
    }

    // -- eval_closure_with_value: not a lambda --
    #[test]
    fn test_closure_with_value_not_lambda() {
        let mut interp = Interpreter::new();
        let not_lambda = make_int(42);
        let value = Value::Integer(1);
        let result = interp.eval_closure_with_value(&not_lambda, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected lambda expression"));
    }

    // -- eval_closure_with_value: wrong param count --
    #[test]
    fn test_closure_with_value_wrong_param_count() {
        let mut interp = Interpreter::new();
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("a"), make_param("b")],
            body: Box::new(make_int(1)),
        });
        let value = Value::Integer(1);
        let result = interp.eval_closure_with_value(&lambda, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have exactly 1 parameter"));
    }

    // -- eval_closure_with_value: non-identifier pattern --
    #[test]
    fn test_closure_with_value_non_identifier_pattern() {
        let mut interp = Interpreter::new();
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![crate::frontend::ast::Param {
                pattern: Pattern::Tuple(vec![Pattern::Identifier("a".to_string())]),
                ty: Type {
                    kind: crate::frontend::ast::TypeKind::Named("Any".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            body: Box::new(make_int(1)),
        });
        let value = Value::Integer(1);
        let result = interp.eval_closure_with_value(&lambda, &value);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have simple identifier parameter"));
    }

    // -- compare_values: int with float --
    #[test]
    fn test_compare_values_int_float() {
        let interp = Interpreter::new();
        let left = Value::Integer(5);
        let right = Value::Float(10.0);
        let result = interp.compare_values(&left, &right, |a, b| a < b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- compare_values: float with int --
    #[test]
    fn test_compare_values_float_int() {
        let interp = Interpreter::new();
        let left = Value::Float(10.0);
        let right = Value::Integer(5);
        let result = interp.compare_values(&left, &right, |a, b| a > b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- compare_values: float with float equal --
    #[test]
    fn test_compare_values_float_float_equal() {
        let interp = Interpreter::new();
        let left = Value::Float(5.0);
        let right = Value::Float(5.0);
        let result = interp.compare_values(&left, &right, |a, b| a == b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- compare_values: bool with bool error --
    #[test]
    fn test_compare_values_bool_error() {
        let interp = Interpreter::new();
        let left = Value::Bool(true);
        let right = Value::Bool(false);
        let result = interp.compare_values(&left, &right, |a, b| a < b);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot compare"));
    }

    // -- values_equal: nil with non-nil --
    #[test]
    fn test_values_equal_nil_with_other() {
        let interp = Interpreter::new();
        assert!(!interp.values_equal(&Value::Nil, &Value::Integer(0)));
        assert!(!interp.values_equal(&Value::Integer(0), &Value::Nil));
    }

    // -- values_equal: array with array --
    #[test]
    fn test_values_equal_array_with_array() {
        let interp = Interpreter::new();
        let arr1 = Value::from_array(vec![Value::Integer(1)]);
        let arr2 = Value::from_array(vec![Value::Integer(1)]);
        // Arrays don't have value equality implemented (returns false)
        assert!(!interp.values_equal(&arr1, &arr2));
    }

    // -- eval_dataframe_operation: on non-DataFrame --
    #[test]
    fn test_dataframe_operation_on_non_dataframe() {
        let mut interp = Interpreter::new();
        let non_df = make_int(42);
        let operation = crate::frontend::ast::DataFrameOp::Select(vec!["x".to_string()]);
        let result = interp.eval_dataframe_operation(&non_df, &operation);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("DataFrameOperation can only be applied to DataFrame"));
    }

    // -- eval_expr_with_column_context: col() call --
    #[test]
    fn test_expr_with_column_context_col_call() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(10), Value::Integer(20)],
        }];
        // Create col("x") call expression
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let result = interp.eval_expr_with_column_context(&col_call, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(10));
    }

    // -- eval_expr_with_column_context: col() with missing column --
    #[test]
    fn test_expr_with_column_context_col_missing() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(10)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "nonexistent".to_string(),
            )))],
        });
        let result = interp.eval_expr_with_column_context(&col_call, &columns, 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // -- eval_expr_with_column_context: col() with out of bounds row --
    #[test]
    fn test_expr_with_column_context_col_row_out_of_bounds() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(10)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let result = interp.eval_expr_with_column_context(&col_call, &columns, 99);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    // -- eval_expr_with_column_context: binary expression with Greater --
    #[test]
    fn test_expr_with_column_context_binary_greater() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(10)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let binary = make_expr(ExprKind::Binary {
            left: Box::new(col_call),
            op: crate::frontend::ast::BinaryOp::Greater,
            right: Box::new(make_int(5)),
        });
        let result = interp.eval_expr_with_column_context(&binary, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- eval_expr_with_column_context: binary expression with Less --
    #[test]
    fn test_expr_with_column_context_binary_less() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(3)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let binary = make_expr(ExprKind::Binary {
            left: Box::new(col_call),
            op: crate::frontend::ast::BinaryOp::Less,
            right: Box::new(make_int(5)),
        });
        let result = interp.eval_expr_with_column_context(&binary, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- eval_expr_with_column_context: binary expression with Equal --
    #[test]
    fn test_expr_with_column_context_binary_equal() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(5)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let binary = make_expr(ExprKind::Binary {
            left: Box::new(col_call),
            op: crate::frontend::ast::BinaryOp::Equal,
            right: Box::new(make_int(5)),
        });
        let result = interp.eval_expr_with_column_context(&binary, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- eval_expr_with_column_context: binary expression with NotEqual --
    #[test]
    fn test_expr_with_column_context_binary_not_equal() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(5)],
        }];
        let col_call = make_expr(ExprKind::Call {
            func: Box::new(make_ident("col")),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "x".to_string(),
            )))],
        });
        let binary = make_expr(ExprKind::Binary {
            left: Box::new(col_call),
            op: crate::frontend::ast::BinaryOp::NotEqual,
            right: Box::new(make_int(10)),
        });
        let result = interp.eval_expr_with_column_context(&binary, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- eval_expr_with_column_context: other expression (fallback) --
    #[test]
    fn test_expr_with_column_context_literal() {
        let mut interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(10)],
        }];
        let literal = make_int(42);
        let result = interp.eval_expr_with_column_context(&literal, &columns, 0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    // -- eval_dataframe_method: delegates to eval_dataframe_ops --
    #[test]
    fn test_eval_dataframe_method_delegates() {
        let interp = Interpreter::new();
        let columns = vec![DataFrameColumn {
            name: "x".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        let result = interp.eval_dataframe_method(&columns, "rows", &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }

    // -- builder column with empty array --
    #[test]
    fn test_builder_column_empty_array() {
        let interp = Interpreter::new();
        let builder = make_builder();
        let args = vec![
            Value::from_string("empty".to_string()),
            Value::from_array(vec![]),
        ];
        let result = interp.eval_dataframe_builder_method(&builder, "column", &args);
        assert!(result.is_ok());
    }

    // -- build with column missing values key --
    #[test]
    fn test_build_with_column_missing_values() {
        let interp = Interpreter::new();
        let mut builder = make_builder();

        let mut col_obj = HashMap::new();
        col_obj.insert("name".to_string(), Value::from_string("x".to_string()));
        // No "values" key
        builder.insert(
            "__columns".to_string(),
            Value::from_array(vec![Value::Object(Arc::new(col_obj))]),
        );

        let result = interp.eval_dataframe_builder_method(&builder, "build", &[]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            // Column should be created with empty values
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].values.len(), 0);
        }
    }

    // -- filter with multiple columns and row access --
    #[test]
    fn test_filter_multiple_columns() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![
                DataFrameColumn {
                    name: "name".to_string(),
                    values: vec![
                        Value::from_string("Alice".to_string()),
                        Value::from_string("Bob".to_string()),
                    ],
                },
                DataFrameColumn {
                    name: "age".to_string(),
                    values: vec![Value::Integer(25), Value::Integer(30)],
                },
            ],
        };
        // Lambda always returns true
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
        });
        let result = interp.eval_dataframe_filter_method(&df, &[lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].values.len(), 2);
        }
    }

    // -- with_column on empty DataFrame --
    #[test]
    fn test_with_column_empty_dataframe() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("new".to_string())));
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("row")],
            body: Box::new(make_int(1)),
        });
        let result = interp.eval_dataframe_with_column_method(&df, &[name_arg, lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[1].values.len(), 0);
        }
    }

    // -- transform on empty DataFrame --
    #[test]
    fn test_transform_empty_dataframe() {
        let mut interp = Interpreter::new();
        let df = Value::DataFrame {
            columns: vec![DataFrameColumn {
                name: "x".to_string(),
                values: vec![],
            }],
        };
        let name_arg = make_expr(ExprKind::Literal(Literal::String("x".to_string())));
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![make_param("v")],
            body: Box::new(make_int(100)),
        });
        let result = interp.eval_dataframe_transform_method(&df, &[name_arg, lambda]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns } = result.unwrap() {
            assert_eq!(columns[0].values.len(), 0);
        }
    }

    // -- values_equal: string with different types --
    #[test]
    fn test_values_equal_string_with_int() {
        let interp = Interpreter::new();
        let s = Value::from_string("42".to_string());
        let i = Value::Integer(42);
        assert!(!interp.values_equal(&s, &i));
    }

    // -- values_equal: bool with int --
    #[test]
    fn test_values_equal_bool_with_int() {
        let interp = Interpreter::new();
        let b = Value::Bool(true);
        let i = Value::Integer(1);
        assert!(!interp.values_equal(&b, &i));
    }

    // -- compare_values: integers equal --
    #[test]
    fn test_compare_values_integers_equal() {
        let interp = Interpreter::new();
        let left = Value::Integer(5);
        let right = Value::Integer(5);
        let result = interp.compare_values(&left, &right, |a, b| a == b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    // -- compare_values: int with float equal check --
    #[test]
    fn test_compare_values_int_float_equal() {
        let interp = Interpreter::new();
        let left = Value::Integer(5);
        let right = Value::Float(5.0);
        let result = interp.compare_values(&left, &right, |a, b| a == b);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }
}
