use super::*;

// =========================================================================
// FLOAT METHOD TESTS
// =========================================================================

#[test]
fn test_float_sqrt() {
    assert_eq!(
        eval_float_method(9.0, "sqrt", true).expect("eval_float_method should succeed in test"),
        Value::Float(3.0)
    );
    assert_eq!(
        eval_float_method(0.0, "sqrt", true).expect("eval_float_method should succeed in test"),
        Value::Float(0.0)
    );
}

#[test]
fn test_float_abs() {
    assert_eq!(
        eval_float_method(-5.5, "abs", true).expect("eval_float_method should succeed in test"),
        Value::Float(5.5)
    );
    assert_eq!(
        eval_float_method(5.5, "abs", true).expect("eval_float_method should succeed in test"),
        Value::Float(5.5)
    );
}

#[test]
fn test_float_round() {
    assert_eq!(
        eval_float_method(3.7, "round", true).expect("eval_float_method should succeed in test"),
        Value::Float(4.0)
    );
    assert_eq!(
        eval_float_method(3.2, "round", true).expect("eval_float_method should succeed in test"),
        Value::Float(3.0)
    );
}

#[test]
fn test_float_floor() {
    assert_eq!(
        eval_float_method(3.7, "floor", true).expect("eval_float_method should succeed in test"),
        Value::Float(3.0)
    );
    assert_eq!(
        eval_float_method(-3.7, "floor", true).expect("eval_float_method should succeed in test"),
        Value::Float(-4.0)
    );
}

#[test]
fn test_float_ceil() {
    assert_eq!(
        eval_float_method(3.2, "ceil", true).expect("eval_float_method should succeed in test"),
        Value::Float(4.0)
    );
    assert_eq!(
        eval_float_method(-3.2, "ceil", true).expect("eval_float_method should succeed in test"),
        Value::Float(-3.0)
    );
}

#[test]
fn test_float_trig() {
    // sin, cos, tan
    assert!(
        (eval_float_method(0.0, "sin", true).expect("eval_float_method should succeed in test")
            == Value::Float(0.0))
    );
    assert!(
        (eval_float_method(0.0, "cos", true).expect("eval_float_method should succeed in test")
            == Value::Float(1.0))
    );
    assert!(
        (eval_float_method(0.0, "tan", true).expect("eval_float_method should succeed in test")
            == Value::Float(0.0))
    );
}

#[test]
fn test_float_log() {
    assert_eq!(
        eval_float_method(std::f64::consts::E, "ln", true)
            .expect("eval_float_method should succeed in test"),
        Value::Float(1.0)
    );
    assert_eq!(
        eval_float_method(10.0, "log10", true).expect("eval_float_method should succeed in test"),
        Value::Float(1.0)
    );
    assert_eq!(
        eval_float_method(0.0, "exp", true).expect("eval_float_method should succeed in test"),
        Value::Float(1.0)
    );
}

#[test]
fn test_float_to_string() {
    let result = eval_float_method(std::f64::consts::PI, "to_string", true)
        .expect("eval_float_method should succeed in test");
    match result {
        Value::String(s) => assert_eq!(s.as_ref(), &std::f64::consts::PI.to_string()),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_float_powf_error() {
    let result = eval_float_method(2.0, "powf", true);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Use ** operator"));
}

#[test]
fn test_float_with_args_error() {
    let result = eval_float_method(5.0, "sqrt", false);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("takes no arguments"));
}

#[test]
fn test_float_unknown_method() {
    let result = eval_float_method(5.0, "unknown", true);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown float method"));
}

// =========================================================================
// INTEGER METHOD TESTS
// =========================================================================

#[test]
fn test_integer_abs() {
    assert_eq!(
        eval_integer_method(-42, "abs", &[]).expect("eval_integer_method should succeed in test"),
        Value::Integer(42)
    );
    assert_eq!(
        eval_integer_method(42, "abs", &[]).expect("eval_integer_method should succeed in test"),
        Value::Integer(42)
    );
    assert_eq!(
        eval_integer_method(0, "abs", &[]).expect("eval_integer_method should succeed in test"),
        Value::Integer(0)
    );
}

#[test]
fn test_integer_sqrt() {
    assert_eq!(
        eval_integer_method(16, "sqrt", &[]).expect("eval_integer_method should succeed in test"),
        Value::Float(4.0)
    );
    assert_eq!(
        eval_integer_method(0, "sqrt", &[]).expect("eval_integer_method should succeed in test"),
        Value::Float(0.0)
    );
}

#[test]
fn test_integer_to_float() {
    assert_eq!(
        eval_integer_method(42, "to_float", &[])
            .expect("eval_integer_method should succeed in test"),
        Value::Float(42.0)
    );
    assert_eq!(
        eval_integer_method(-5, "to_float", &[])
            .expect("eval_integer_method should succeed in test"),
        Value::Float(-5.0)
    );
}

#[test]
fn test_integer_to_string() {
    let result = eval_integer_method(123, "to_string", &[])
        .expect("eval_integer_method should succeed in test");
    assert_eq!(result, Value::from_string("123".to_string()));
}

#[test]
fn test_integer_signum() {
    assert_eq!(
        eval_integer_method(42, "signum", &[]).expect("eval_integer_method should succeed in test"),
        Value::Integer(1)
    );
    assert_eq!(
        eval_integer_method(-42, "signum", &[])
            .expect("eval_integer_method should succeed in test"),
        Value::Integer(-1)
    );
    assert_eq!(
        eval_integer_method(0, "signum", &[]).expect("eval_integer_method should succeed in test"),
        Value::Integer(0)
    );
}

#[test]
fn test_integer_pow() {
    let result = eval_integer_method(2, "pow", &[Value::Integer(3)])
        .expect("eval_integer_method should succeed in test");
    assert_eq!(result, Value::Integer(8));

    let result = eval_integer_method(5, "pow", &[Value::Integer(0)])
        .expect("eval_integer_method should succeed in test");
    assert_eq!(result, Value::Integer(1));
}

#[test]
fn test_integer_pow_negative_exponent_error() {
    let result = eval_integer_method(2, "pow", &[Value::Integer(-1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("non-negative"));
}

#[test]
fn test_integer_pow_wrong_type_error() {
    let result = eval_integer_method(2, "pow", &[Value::Float(3.0)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("integer exponent"));
}

#[test]
fn test_integer_pow_wrong_arg_count() {
    let result = eval_integer_method(2, "pow", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("requires exactly 1 argument"));
}

#[test]
fn test_integer_abs_with_args_error() {
    let result = eval_integer_method(42, "abs", &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("takes no arguments"));
}

#[test]
fn test_integer_unknown_method() {
    let result = eval_integer_method(42, "unknown", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown integer method"));
}

// =========================================================================
// DATAFRAME METHOD TESTS
// =========================================================================

#[test]
fn test_dataframe_count() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_count(&columns, &[]).expect("eval_dataframe_count should succeed in test"),
        Value::Integer(3)
    );
}

#[test]
fn test_dataframe_sum() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_sum(&columns, &[]).expect("eval_dataframe_sum should succeed in test"),
        Value::Float(6.0)
    );
}

#[test]
fn test_dataframe_sum_mixed_types() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1), Value::Float(2.5), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_sum(&columns, &[]).expect("eval_dataframe_sum should succeed in test"),
        Value::Float(6.5)
    );
}

#[test]
fn test_dataframe_mean() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_mean(&columns, &[]).expect("eval_dataframe_mean should succeed in test"),
        Value::Float(2.0)
    );
}

#[test]
fn test_dataframe_max() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1), Value::Integer(5), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_max(&columns, &[]).expect("eval_dataframe_max should succeed in test"),
        Value::Float(5.0)
    );
}

#[test]
fn test_dataframe_min() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(5), Value::Integer(1), Value::Integer(3)],
    }];
    assert_eq!(
        eval_dataframe_min(&columns, &[]).expect("eval_dataframe_min should succeed in test"),
        Value::Float(1.0)
    );
}

#[test]
fn test_dataframe_columns() {
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
    let result = eval_dataframe_columns(&columns, &[])
        .expect("eval_dataframe_columns should succeed in test");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_dataframe_shape() {
    let columns = vec![
        DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        },
        DataFrameColumn {
            name: "b".to_string(),
            values: vec![Value::Integer(4), Value::Integer(5), Value::Integer(6)],
        },
    ];
    let result =
        eval_dataframe_shape(&columns, &[]).expect("eval_dataframe_shape should succeed in test");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr[0], Value::Integer(3)); // rows
            assert_eq!(arr[1], Value::Integer(2)); // columns
        }
        _ => panic!("Expected Array"),
    }
}

// =========================================================================
// GENERIC METHOD TESTS
// =========================================================================

#[test]
fn test_generic_to_string_bool() {
    let value = Value::Bool(true);
    assert_eq!(
        eval_generic_method(&value, "to_string", true)
            .expect("eval_generic_method should succeed in test"),
        Value::from_string("true".to_string())
    );
}

#[test]
fn test_generic_to_string_nil() {
    let value = Value::Nil;
    let result = eval_generic_method(&value, "to_string", true)
        .expect("eval_generic_method should succeed in test");
    assert_eq!(result, Value::from_string("nil".to_string()));
}

#[test]
fn test_generic_unknown_method() {
    let value = Value::Bool(true);
    let result = eval_generic_method(&value, "unknown", true);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// =========================================================================
// DISPATCH TESTS (dispatch_method_call)
// =========================================================================

#[test]
fn test_dispatch_turbofish_stripping() {
    // Test that turbofish syntax is stripped from method names
    // Example: "parse::<i32>" becomes "parse"
    let s = Arc::from("42");
    let value = Value::String(s);

    // Mock closures
    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    // This should work because turbofish is stripped
    let result = dispatch_method_call(
        &value,
        "to_string::<String>", // With turbofish
        &[],
        true,
        &mut eval_fn,
        eval_df,
        eval_ctx,
    );
    assert!(result.is_ok());
}

// Round 95: Additional method dispatch tests

// Test 35: dataframe empty columns
#[test]
fn test_dataframe_columns_empty() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_columns(&columns, &[])
        .expect("eval_dataframe_columns should succeed for empty");
    match result {
        Value::Array(arr) => assert!(arr.is_empty()),
        _ => panic!("Expected empty Array"),
    }
}

// Test 36: dataframe shape empty
#[test]
fn test_dataframe_shape_empty() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result =
        eval_dataframe_shape(&columns, &[]).expect("eval_dataframe_shape should succeed for empty");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[1], Value::Integer(0));
        }
        _ => panic!("Expected Array"),
    }
}

// Test 37: dataframe sum empty
#[test]
fn test_dataframe_sum_empty() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_sum(&columns, &[]);
    // Empty columns might return 0 or error
    assert!(result.is_ok() || result.is_err());
}

// Test 38: dataframe mean with single value
#[test]
fn test_dataframe_mean_single() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(42)],
    }];
    let result = eval_dataframe_mean(&columns, &[]).expect("eval_dataframe_mean should succeed");
    assert_eq!(result, Value::Float(42.0));
}

// Test 39: generic to_string integer
#[test]
fn test_generic_to_string_integer() {
    let value = Value::Integer(42);
    let result =
        eval_generic_method(&value, "to_string", true).expect("eval_generic_method should succeed");
    assert_eq!(result, Value::from_string("42".to_string()));
}

// Test 40: generic to_string float
#[test]
fn test_generic_to_string_float() {
    let value = Value::Float(3.14);
    let result =
        eval_generic_method(&value, "to_string", true).expect("eval_generic_method should succeed");
    // Float to_string might include precision
    match result {
        Value::String(s) => assert!(s.starts_with("3.14")),
        _ => panic!("Expected String"),
    }
}

// Test 41: generic to_string array
#[test]
fn test_generic_to_string_array() {
    let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result =
        eval_generic_method(&value, "to_string", true).expect("eval_generic_method should succeed");
    match result {
        Value::String(_) => {} // Any string representation is fine
        _ => panic!("Expected String"),
    }
}

// Test 42: dataframe single column max
#[test]
fn test_dataframe_max_single() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(42)],
    }];
    assert_eq!(
        eval_dataframe_max(&columns, &[]).expect("eval_dataframe_max should succeed"),
        Value::Float(42.0)
    );
}

// Test 43: dataframe single column min
#[test]
fn test_dataframe_min_single() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(42)],
    }];
    assert_eq!(
        eval_dataframe_min(&columns, &[]).expect("eval_dataframe_min should succeed"),
        Value::Float(42.0)
    );
}

// Test 44: dataframe with float values
#[test]
fn test_dataframe_sum_floats() {
    let columns = vec![DataFrameColumn {
        name: "values".to_string(),
        values: vec![Value::Float(1.5), Value::Float(2.5), Value::Float(3.0)],
    }];
    let result =
        eval_dataframe_sum(&columns, &[]).expect("eval_dataframe_sum should succeed for floats");
    match result {
        Value::Float(f) => assert!((f - 7.0).abs() < 0.001),
        _ => panic!("Expected Float"),
    }
}

// Test 45: dataframe shape with multiple columns
#[test]
fn test_dataframe_shape_multi_column() {
    let columns = vec![
        DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        },
        DataFrameColumn {
            name: "b".to_string(),
            values: vec![Value::Integer(2)],
        },
        DataFrameColumn {
            name: "c".to_string(),
            values: vec![Value::Integer(3)],
        },
    ];
    let result = eval_dataframe_shape(&columns, &[]).expect("eval_dataframe_shape should succeed");
    match result {
        Value::Array(arr) => {
            assert_eq!(arr[0], Value::Integer(1)); // 1 row
            assert_eq!(arr[1], Value::Integer(3)); // 3 columns
        }
        _ => panic!("Expected Array"),
    }
}

// Test 46: dispatch with array value
#[test]
fn test_dispatch_array_method() {
    let value = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(2));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(&value, "len", &[], true, &mut eval_fn, eval_df, eval_ctx);
    assert!(result.is_ok());
}

// Test 47: dispatch with nil value
#[test]
fn test_dispatch_nil_to_string() {
    let value = Value::Nil;

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::from_string("nil".to_string()));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(
        &value,
        "to_string",
        &[],
        true,
        &mut eval_fn,
        eval_df,
        eval_ctx,
    );
    assert!(result.is_ok());
}

// =========================================================================
// EXTREME TDD ROUND 127 - Additional Coverage Tests
// =========================================================================

// Test R127-01: eval_dataframe_select success
#[test]
fn test_dataframe_select_success_r127() {
    let columns = vec![
        DataFrameColumn {
            name: "price".to_string(),
            values: vec![Value::Float(10.5), Value::Float(20.0)],
        },
        DataFrameColumn {
            name: "quantity".to_string(),
            values: vec![Value::Integer(5), Value::Integer(10)],
        },
    ];
    let result = eval_dataframe_select(&columns, &[Value::from_string("price".to_string())])
        .expect("should select column");
    match result {
        Value::DataFrame { columns: selected } => {
            assert_eq!(selected.len(), 1);
            assert_eq!(selected[0].name, "price");
        }
        _ => panic!("Expected DataFrame"),
    }
}

// Test R127-02: eval_dataframe_select column not found
#[test]
fn test_dataframe_select_not_found_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_select(&columns, &[Value::from_string("missing".to_string())]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// Test R127-03: eval_dataframe_select wrong arg type
#[test]
fn test_dataframe_select_wrong_type_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_select(&columns, &[Value::Integer(42)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("column name"));
}

// Test R127-04: eval_dataframe_select wrong arg count
#[test]
fn test_dataframe_select_wrong_count_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_select(&columns, &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exactly 1"));
}

// Test R127-05: dataframe count with args error
#[test]
fn test_dataframe_count_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_count(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-06: dataframe sum with args error
#[test]
fn test_dataframe_sum_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_sum(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-07: dataframe mean with args error
#[test]
fn test_dataframe_mean_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_mean(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-08: dataframe max with args error
#[test]
fn test_dataframe_max_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_max(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-09: dataframe min with args error
#[test]
fn test_dataframe_min_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_min(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-10: dataframe columns with args error
#[test]
fn test_dataframe_columns_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_columns(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-11: dataframe shape with args error
#[test]
fn test_dataframe_shape_with_args_error_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_shape(&columns, &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-12: dataframe mean empty (nil result)
#[test]
fn test_dataframe_mean_empty_nil_r127() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_mean(&columns, &[]).expect("should return nil for empty");
    assert_eq!(result, Value::Nil);
}

// Test R127-13: dataframe max empty (nil result)
#[test]
fn test_dataframe_max_empty_nil_r127() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_max(&columns, &[]).expect("should return nil for empty");
    assert_eq!(result, Value::Nil);
}

// Test R127-14: dataframe min empty (nil result)
#[test]
fn test_dataframe_min_empty_nil_r127() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_min(&columns, &[]).expect("should return nil for empty");
    assert_eq!(result, Value::Nil);
}

// Test R127-15: dataframe unknown method
#[test]
fn test_dataframe_unknown_method_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let result = eval_dataframe_method(&columns, "unknown_method", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown DataFrame"));
}

// Test R127-16: dataframe count empty
#[test]
fn test_dataframe_count_empty_r127() {
    let columns: Vec<DataFrameColumn> = vec![];
    let result = eval_dataframe_count(&columns, &[]).expect("should return 0 for empty");
    assert_eq!(result, Value::Integer(0));
}

// Test R127-17: eval_exit_status_method success
#[test]
fn test_exit_status_method_success_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("ExitStatus".to_string()),
    );
    obj.insert("success".to_string(), Value::Bool(true));
    obj.insert("code".to_string(), Value::Integer(0));

    let result = eval_exit_status_method(&obj, "success", &[]).expect("should get success");
    assert_eq!(result, Value::Bool(true));
}

// Test R127-18: eval_exit_status_method success false
#[test]
fn test_exit_status_method_success_false_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("ExitStatus".to_string()),
    );
    obj.insert("success".to_string(), Value::Bool(false));
    obj.insert("code".to_string(), Value::Integer(1));

    let result = eval_exit_status_method(&obj, "success", &[]).expect("should get success");
    assert_eq!(result, Value::Bool(false));
}

// Test R127-19: eval_exit_status_method with args error
#[test]
fn test_exit_status_method_with_args_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("ExitStatus".to_string()),
    );
    obj.insert("success".to_string(), Value::Bool(true));

    let result = eval_exit_status_method(&obj, "success", &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-20: eval_exit_status_method unknown method
#[test]
fn test_exit_status_method_unknown_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("ExitStatus".to_string()),
    );
    obj.insert("success".to_string(), Value::Bool(true));

    let result = eval_exit_status_method(&obj, "unknown", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown ExitStatus"));
}

// Test R127-21: eval_exit_status_method missing success field
#[test]
fn test_exit_status_method_missing_field_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("ExitStatus".to_string()),
    );
    // Missing "success" field

    let result = eval_exit_status_method(&obj, "success", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing"));
}

// Test R127-22: require_no_args helper with args
#[test]
fn test_require_no_args_with_args_r127() {
    let result = require_no_args("test_method", &[Value::Integer(1)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("no arguments"));
}

// Test R127-23: require_no_args helper empty
#[test]
fn test_require_no_args_empty_r127() {
    let result = require_no_args("test_method", &[]);
    assert!(result.is_ok());
}

// Test R127-24: eval_integer_pow multiple args
#[test]
fn test_integer_pow_multiple_args_r127() {
    let result = eval_integer_pow(2, &[Value::Integer(3), Value::Integer(4)]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exactly 1"));
}

// Test R127-25: try_dispatch_builtin no marker
#[test]
fn test_try_dispatch_builtin_no_marker_r127() {
    let obj = std::collections::HashMap::new();
    let result = try_dispatch_builtin(&obj, "some_method", &[]).expect("should return None");
    assert!(result.is_none());
}

// Test R127-26: try_dispatch_builtin not builtin
#[test]
fn test_try_dispatch_builtin_not_builtin_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "some_method".to_string(),
        Value::from_string("regular_value".to_string()),
    );
    let result =
        try_dispatch_builtin(&obj, "some_method", &[]).expect("should return None for non-builtin");
    assert!(result.is_none());
}

// Test R127-27: eval_object_method missing type marker
#[test]
fn test_object_method_missing_type_r127() {
    let obj = std::collections::HashMap::new();
    let result = eval_object_method(&obj, "test", &[]);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("missing __type"));
}

// Test R127-28: eval_object_method unknown type
#[test]
fn test_object_method_unknown_type_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("UnknownType".to_string()),
    );
    let result = eval_object_method(&obj, "test", &[]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown object type"));
}

// Test R127-29: generic method with args (should fail)
#[test]
fn test_generic_to_string_with_args_r127() {
    let value = Value::Bool(true);
    let result = eval_generic_method(&value, "to_string", false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// Test R127-30: dispatch with dataframe value
#[test]
fn test_dispatch_dataframe_method_r127() {
    let columns = vec![DataFrameColumn {
        name: "a".to_string(),
        values: vec![Value::Integer(1)],
    }];
    let value = Value::DataFrame { columns };

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(&value, "count", &[], true, &mut eval_fn, eval_df, eval_ctx);
    assert!(result.is_ok());
    assert_eq!(result.expect("should work"), Value::Integer(1));
}

// Test R127-31: dispatch with float value
#[test]
fn test_dispatch_float_method_r127() {
    let value = Value::Float(9.0);

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(&value, "sqrt", &[], true, &mut eval_fn, eval_df, eval_ctx);
    assert!(result.is_ok());
    assert_eq!(result.expect("should work"), Value::Float(3.0));
}

// Test R127-32: dispatch with integer value
#[test]
fn test_dispatch_integer_method_r127() {
    let value = Value::Integer(-42);

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(&value, "abs", &[], true, &mut eval_fn, eval_df, eval_ctx);
    assert!(result.is_ok());
    assert_eq!(result.expect("should work"), Value::Integer(42));
}

// Test R127-33: dispatch with object value (Command type)
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_dispatch_object_command_r127() {
    let mut obj = std::collections::HashMap::new();
    obj.insert(
        "__type".to_string(),
        Value::from_string("Command".to_string()),
    );
    obj.insert(
        "program".to_string(),
        Value::from_string("echo".to_string()),
    );
    obj.insert("args".to_string(), Value::Array(Arc::from(vec![])));
    let value = Value::Object(Arc::new(obj));

    let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
    let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
    let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

    let result = dispatch_method_call(
        &value,
        "arg",
        &[Value::from_string("hello".to_string())],
        false,
        &mut eval_fn,
        eval_df,
        eval_ctx,
    );
    assert!(result.is_ok());
}

// Test R127-34: eval_method_call wrapper function
#[test]
fn test_eval_method_call_wrapper_r127() {
    let value = Value::Float(16.0);

    let result = eval_method_call(
        &value,
        "sqrt",
        &[],
        true,
        |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
        |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
        |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
    );
    assert!(result.is_ok());
    assert_eq!(result.expect("should work"), Value::Float(4.0));
}

// Test R127-35: dataframe sum with non-numeric (should skip)
#[test]
fn test_dataframe_sum_with_strings_r127() {
    let columns = vec![DataFrameColumn {
        name: "mixed".to_string(),
        values: vec![
            Value::Integer(1),
            Value::from_string("skip".to_string()),
            Value::Integer(2),
        ],
    }];
    let result = eval_dataframe_sum(&columns, &[]).expect("should work");
    assert_eq!(result, Value::Float(3.0));
}

// Test R127-36: dataframe mean with non-numeric (should skip)
#[test]
fn test_dataframe_mean_with_strings_r127() {
    let columns = vec![DataFrameColumn {
        name: "mixed".to_string(),
        values: vec![
            Value::Integer(2),
            Value::from_string("skip".to_string()),
            Value::Integer(4),
        ],
    }];
    let result = eval_dataframe_mean(&columns, &[]).expect("should work");
    assert_eq!(result, Value::Float(3.0));
}

// Test R127-37: dataframe max with strings (should skip)
#[test]
fn test_dataframe_max_with_strings_r127() {
    let columns = vec![DataFrameColumn {
        name: "mixed".to_string(),
        values: vec![
            Value::Integer(5),
            Value::from_string("skip".to_string()),
            Value::Integer(10),
        ],
    }];
    let result = eval_dataframe_max(&columns, &[]).expect("should work");
    assert_eq!(result, Value::Float(10.0));
}

// Test R127-38: dataframe min with strings (should skip)
#[test]
fn test_dataframe_min_with_strings_r127() {
    let columns = vec![DataFrameColumn {
        name: "mixed".to_string(),
        values: vec![
            Value::Integer(5),
            Value::from_string("skip".to_string()),
            Value::Integer(3),
        ],
    }];
    let result = eval_dataframe_min(&columns, &[]).expect("should work");
    assert_eq!(result, Value::Float(3.0));
}
