//! DF-003: `DataFrame` Aggregation Functions (EXTREME TDD)
//!
//! **CRITICAL**: RED → GREEN → REFACTOR cycle with comprehensive test coverage.
//!
//! **Toyota Way**: Write failing tests FIRST, then implement to pass.

use ruchy::runtime::{DataFrameColumn, Value};
use ruchy::runtime::eval_dataframe_ops::eval_dataframe_method;

/// Test helper to create simple numeric `DataFrameColumn`
fn numeric_column(name: &str, values: Vec<i64>) -> DataFrameColumn {
    DataFrameColumn {
        name: name.to_string(),
        values: values.into_iter().map(Value::Integer).collect(),
    }
}

/// Test helper to create float `DataFrameColumn`
fn float_column(name: &str, values: Vec<f64>) -> DataFrameColumn {
    DataFrameColumn {
        name: name.to_string(),
        values: values.into_iter().map(Value::Float).collect(),
    }
}

#[cfg(test)]
mod df003_std_tests {
    use super::*;

    /// DF-003: GREEN phase - `std()` calculates standard deviation
    #[test]
    fn test_df003_std_basic_integers() {
        let columns = vec![numeric_column("values", vec![1, 2, 3, 4, 5])];

        // Expected: std([1,2,3,4,5]) = sqrt(var) = sqrt(2) ≈ 1.414
        let result = eval_dataframe_method(&columns, "std", &[]);
        assert!(result.is_ok(), "std() should work on numeric columns");

        if let Ok(Value::Float(std_val)) = result {
            assert!((std_val - 1.414).abs() < 0.01, "std([1,2,3,4,5]) ≈ 1.414, got {std_val}");
        } else {
            panic!("Expected Float result from std()");
        }
    }

    #[test]
    fn test_df003_std_single_value() {
        let columns = vec![numeric_column("val", vec![42])];

        let result = eval_dataframe_method(&columns, "std", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(std_val)) = result {
            assert_eq!(std_val, 0.0, "std of single value should be 0");
        }
    }

    #[test]
    fn test_df003_std_floats() {
        let columns = vec![float_column("data", vec![1.0, 2.0, 3.0, 4.0])];

        // Expected: std([1,2,3,4]) = sqrt((0.25+0.25+0.25+0.25)/4) ≈ 1.118
        let result = eval_dataframe_method(&columns, "std", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(std_val)) = result {
            assert!((std_val - 1.118).abs() < 0.01, "std([1,2,3,4]) ≈ 1.118, got {std_val}");
        }
    }

    #[test]
    fn test_df003_std_empty_dataframe() {
        let columns: Vec<DataFrameColumn> = vec![];

        let result = eval_dataframe_method(&columns, "std", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(std_val)) = result {
            assert_eq!(std_val, 0.0, "std of empty DataFrame should be 0");
        }
    }

    #[test]
    fn test_df003_std_with_args_error() {
        let columns = vec![numeric_column("val", vec![1, 2, 3])];
        let args = vec![Value::Integer(1)];

        let result = eval_dataframe_method(&columns, "std", &args);
        assert!(result.is_err(), "std() should not accept arguments");
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[test]
    fn test_df003_std_multiple_columns() {
        let columns = vec![
            numeric_column("a", vec![1, 2, 3]),
            numeric_column("b", vec![4, 5, 6]),
        ];

        // Expected: std([1,2,3,4,5,6]) ≈ 1.707
        let result = eval_dataframe_method(&columns, "std", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(std_val)) = result {
            assert!((std_val - 1.707).abs() < 0.01, "std of all values ≈ 1.707, got {std_val}");
        }
    }
}

#[cfg(test)]
mod df003_var_tests {
    use super::*;

    /// DF-003: GREEN phase - `var()` calculates variance
    #[test]
    fn test_df003_var_basic_integers() {
        let columns = vec![numeric_column("values", vec![1, 2, 3, 4, 5])];

        // Expected: var([1,2,3,4,5]) = 2.0 (population variance)
        let result = eval_dataframe_method(&columns, "var", &[]);
        assert!(result.is_ok(), "var() should work on numeric columns");

        if let Ok(Value::Float(var_val)) = result {
            assert!((var_val - 2.0).abs() < 0.01, "var([1,2,3,4,5]) = 2.0, got {var_val}");
        } else {
            panic!("Expected Float result from var()");
        }
    }

    #[test]
    fn test_df003_var_single_value() {
        let columns = vec![numeric_column("val", vec![42])];

        let result = eval_dataframe_method(&columns, "var", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(var_val)) = result {
            assert_eq!(var_val, 0.0, "var of single value should be 0");
        }
    }

    #[test]
    fn test_df003_var_floats() {
        let columns = vec![float_column("data", vec![2.0, 4.0, 6.0, 8.0])];

        // Expected: mean=5, var=((9+1+1+9)/4) = 5.0
        let result = eval_dataframe_method(&columns, "var", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(var_val)) = result {
            assert!((var_val - 5.0).abs() < 0.01, "var([2,4,6,8]) = 5.0, got {var_val}");
        }
    }

    #[test]
    fn test_df003_var_empty_dataframe() {
        let columns: Vec<DataFrameColumn> = vec![];

        let result = eval_dataframe_method(&columns, "var", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(var_val)) = result {
            assert_eq!(var_val, 0.0, "var of empty DataFrame should be 0");
        }
    }

    #[test]
    fn test_df003_var_with_args_error() {
        let columns = vec![numeric_column("val", vec![1, 2, 3])];
        let args = vec![Value::Integer(1)];

        let result = eval_dataframe_method(&columns, "var", &args);
        assert!(result.is_err(), "var() should not accept arguments");
        assert!(result.unwrap_err().to_string().contains("takes no arguments"));
    }

    #[test]
    fn test_df003_var_all_same_values() {
        let columns = vec![numeric_column("same", vec![5, 5, 5, 5])];

        let result = eval_dataframe_method(&columns, "var", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Float(var_val)) = result {
            assert_eq!(var_val, 0.0, "var of identical values should be 0");
        }
    }

    #[test]
    fn test_df003_var_relationship_with_std() {
        let columns = vec![numeric_column("vals", vec![10, 20, 30, 40, 50])];

        let var_result = eval_dataframe_method(&columns, "var", &[]);
        let std_result = eval_dataframe_method(&columns, "std", &[]);

        assert!(var_result.is_ok() && std_result.is_ok());

        if let (Ok(Value::Float(var_val)), Ok(Value::Float(std_val))) = (var_result, std_result) {
            // Mathematical invariant: var = std²
            let expected_std = var_val.sqrt();
            assert!((std_val - expected_std).abs() < 0.01, "std² should equal var");
        }
    }
}

#[cfg(test)]
mod df003_integration_tests {
    use super::*;

    /// DF-003: Test that existing aggregations still work
    #[test]
    fn test_df003_existing_mean_still_works() {
        let columns = vec![numeric_column("vals", vec![1, 2, 3, 4, 5])];

        let result = eval_dataframe_method(&columns, "mean", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Integer(mean_val)) = result {
            assert_eq!(mean_val, 3, "mean([1,2,3,4,5]) = 3");
        }
    }

    #[test]
    fn test_df003_existing_min_still_works() {
        let columns = vec![numeric_column("vals", vec![5, 2, 8, 1, 9])];

        let result = eval_dataframe_method(&columns, "min", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Integer(min_val)) = result {
            assert_eq!(min_val, 1, "min([5,2,8,1,9]) = 1");
        }
    }

    #[test]
    fn test_df003_existing_max_still_works() {
        let columns = vec![numeric_column("vals", vec![5, 2, 8, 1, 9])];

        let result = eval_dataframe_method(&columns, "max", &[]);
        assert!(result.is_ok());

        if let Ok(Value::Integer(max_val)) = result {
            assert_eq!(max_val, 9, "max([5,2,8,1,9]) = 9");
        }
    }
}
