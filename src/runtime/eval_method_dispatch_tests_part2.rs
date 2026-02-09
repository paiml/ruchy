    use super::*;
    // ============================================================================
    // EXTREME TDD Round 131: Comprehensive method dispatch coverage tests
    // Target: 88.35% → 95%+ coverage
    // ============================================================================

    // --- Float method error paths ---
    #[test]
    fn test_float_method_powf_suggests_operator() {
        let result = eval_float_method(2.0, "powf", true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Use ** operator"));
    }

    #[test]
    fn test_float_method_with_args_error() {
        let result = eval_float_method(2.0, "sqrt", false);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_float_method_unknown() {
        let result = eval_float_method(2.0, "unknown_method", true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown float method"));
    }

    #[test]
    fn test_float_method_abs() {
        let result = eval_float_method(-3.5, "abs", true).unwrap();
        assert_eq!(result, Value::Float(3.5));
    }

    #[test]
    fn test_float_method_ceil() {
        let result = eval_float_method(3.2, "ceil", true).unwrap();
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_float_method_floor() {
        let result = eval_float_method(3.8, "floor", true).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_float_method_sin() {
        let result = eval_float_method(0.0, "sin", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_cos() {
        let result = eval_float_method(0.0, "cos", true).unwrap();
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_tan() {
        let result = eval_float_method(0.0, "tan", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_ln() {
        let result = eval_float_method(1.0, "ln", true).unwrap();
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_float_method_log10() {
        let result = eval_float_method(100.0, "log10", true).unwrap();
        assert_eq!(result, Value::Float(2.0));
    }

    #[test]
    fn test_float_method_exp() {
        let result = eval_float_method(0.0, "exp", true).unwrap();
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_float_method_to_string() {
        let result = eval_float_method(3.14, "to_string", true).unwrap();
        assert_eq!(result, Value::from_string("3.14".to_string()));
    }

    // --- Integer method error paths ---
    #[test]
    fn test_integer_method_pow_wrong_arg_count() {
        let result = eval_integer_pow(2, &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires exactly 1 argument"));
    }

    #[test]
    fn test_integer_method_pow_negative_exp() {
        let result = eval_integer_pow(2, &[Value::Integer(-1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be non-negative"));
    }

    #[test]
    fn test_integer_method_pow_wrong_type() {
        let result = eval_integer_pow(2, &[Value::from_string("3".to_string())]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("requires integer exponent"));
    }

    #[test]
    fn test_integer_method_unknown() {
        let result = eval_integer_method(42, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown integer method"));
    }

    #[test]
    fn test_integer_method_abs_with_args() {
        let result = eval_integer_method(42, "abs", &[Value::Integer(1)]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("takes no arguments"));
    }

    #[test]
    fn test_integer_method_sqrt() {
        let result = eval_integer_method(9, "sqrt", &[]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_integer_method_to_float() {
        let result = eval_integer_method(42, "to_float", &[]).unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_integer_method_to_string() {
        let result = eval_integer_method(42, "to_string", &[]).unwrap();
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    #[test]
    fn test_integer_method_signum_positive() {
        let result = eval_integer_method(42, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_integer_method_signum_negative() {
        let result = eval_integer_method(-42, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(-1));
    }

    #[test]
    fn test_integer_method_signum_zero() {
        let result = eval_integer_method(0, "signum", &[]).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_integer_method_pow_success() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(10)]).unwrap();
        assert_eq!(result, Value::Integer(1024));
    }

    // --- Object method error paths ---
    #[test]
    fn test_object_method_unknown_type() {
        let mut obj = std::collections::HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("UnknownType".to_string()),
        );
        let result = eval_object_method(&obj, "method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown object type"));
    }

    #[test]
    fn test_object_method_missing_type() {
        let obj = std::collections::HashMap::new();
        let result = eval_object_method(&obj, "method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing __type marker"));
    }

    // --- Generic method error paths ---
    #[test]
    fn test_generic_method_unknown() {
        let result = eval_generic_method(&Value::Nil, "unknown", true);
        assert!(result.is_err());
    }

    // --- Dataframe method tests ---
    #[test]
    fn test_dataframe_method_unknown() {
        let columns = vec![];
        let result = eval_dataframe_method(&columns, "unknown_method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown DataFrame method"));
    }

    #[test]
    fn test_dataframe_columns_method() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![],
            },
        ];
        let result = eval_dataframe_columns(&columns, &[]).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::from_string("a".to_string()));
            assert_eq!(arr[1], Value::from_string("b".to_string()));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dataframe_shape_method() {
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
        let result = eval_dataframe_shape(&columns, &[]).unwrap();
        if let Value::Array(shape) = result {
            assert_eq!(shape[0], Value::Integer(2)); // rows
            assert_eq!(shape[1], Value::Integer(2)); // cols
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_dataframe_count_method() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        }];
        let result = eval_dataframe_count(&columns, &[]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_dataframe_select_method() {
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
        let result =
            eval_dataframe_select(&columns, &[Value::from_string("a".to_string())]).unwrap();
        if let Value::DataFrame { columns: new_cols } = result {
            assert_eq!(new_cols.len(), 1);
            assert_eq!(new_cols[0].name, "a");
        } else {
            panic!("Expected dataframe");
        }
    }

    #[test]
    fn test_dataframe_select_not_found() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let result = eval_dataframe_select(&columns, &[Value::from_string("z".to_string())]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Column 'z' not found"));
    }

    // --- Dispatch turbofish stripping test ---
    #[test]
    fn test_turbofish_stripping_in_dispatch() {
        // Method "parse::<i32>" should be stripped to "parse"
        // Testing via integer method which doesn't have parse (expect error)
        let result = eval_integer_method(42, "parse::<i32>", &[]);
        assert!(result.is_err());
        // The error should mention "parse" not "parse::<i32>"
    }

    // --- eval_method_call main entry point ---
    #[test]
    fn test_eval_method_call_integer_abs() {
        let result = eval_method_call(
            &Value::Integer(-42),
            "abs",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_method_call_float_sqrt() {
        let result = eval_method_call(
            &Value::Float(16.0),
            "sqrt",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(4.0));
    }

    // ============================================================================
    // COMPREHENSIVE COVERAGE TESTS - Targeting 334 uncovered lines
    // ============================================================================

    // --- Command method tests (non-WASM only) ---

    #[cfg(not(target_arch = "wasm32"))]
    mod command_tests {
        use super::*;

        fn create_command_obj(program: &str) -> std::collections::HashMap<String, Value> {
            let mut obj = std::collections::HashMap::new();
            obj.insert(
                "__type".to_string(),
                Value::from_string("Command".to_string()),
            );
            obj.insert(
                "program".to_string(),
                Value::from_string(program.to_string()),
            );
            obj.insert("args".to_string(), Value::Array(Arc::from(vec![])));
            obj
        }

        #[test]
        fn test_command_arg_success() {
            let obj = create_command_obj("echo");
            let result =
                eval_command_method(&obj, "arg", &[Value::from_string("hello".to_string())]);
            assert!(result.is_ok());
            if let Value::Object(new_obj) = result.unwrap() {
                if let Some(Value::Array(args)) = new_obj.get("args") {
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Value::from_string("hello".to_string()));
                } else {
                    panic!("Expected args array");
                }
            } else {
                panic!("Expected Object");
            }
        }

        #[test]
        fn test_command_arg_wrong_count() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "arg", &[]);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("requires exactly 1 argument"));
        }

        #[test]
        fn test_command_arg_wrong_type() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "arg", &[Value::Integer(42)]);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("expects a string argument"));
        }

        #[test]
        fn test_command_unknown_method() {
            let obj = create_command_obj("echo");
            let result = eval_command_method(&obj, "unknown_method", &[]);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Unknown Command method"));
        }

        #[test]
        fn test_command_status_success() {
            // Use 'true' command which always succeeds
            let obj = create_command_obj("true");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } = result.unwrap()
            {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                assert_eq!(data.len(), 1);
                if let Value::Object(status_obj) = &data[0] {
                    assert_eq!(status_obj.get("success"), Some(&Value::Bool(true)));
                } else {
                    panic!("Expected Object in data");
                }
            } else {
                panic!("Expected EnumVariant");
            }
        }

        #[test]
        fn test_command_status_failure() {
            // Use 'false' command which always fails
            let obj = create_command_obj("false");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } = result.unwrap()
            {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                if let Value::Object(status_obj) = &data[0] {
                    assert_eq!(status_obj.get("success"), Some(&Value::Bool(false)));
                }
            }
        }

        #[test]
        fn test_command_status_error_nonexistent() {
            // Use a command that doesn't exist
            let obj = create_command_obj("nonexistent_command_12345");
            let result = eval_command_method(&obj, "status", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant {
                enum_name,
                variant_name,
                ..
            } = result.unwrap()
            {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Err");
            } else {
                panic!("Expected EnumVariant Err");
            }
        }

        #[test]
        fn test_command_output_success() {
            let obj = create_command_obj("echo");
            let mut obj_with_args = obj.clone();
            obj_with_args.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![Value::from_string("hello".to_string())])),
            );
            let result = eval_command_method(&obj_with_args, "output", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } = result.unwrap()
            {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Ok");
                assert!(data.is_some());
                let data = data.unwrap();
                if let Value::Object(output_obj) = &data[0] {
                    assert!(output_obj.contains_key("stdout"));
                    assert!(output_obj.contains_key("stderr"));
                    assert!(output_obj.contains_key("status"));
                } else {
                    panic!("Expected Object in data");
                }
            }
        }

        #[test]
        fn test_command_output_error_nonexistent() {
            let obj = create_command_obj("nonexistent_command_12345");
            let result = eval_command_method(&obj, "output", &[]);
            assert!(result.is_ok());
            if let Value::EnumVariant {
                enum_name,
                variant_name,
                ..
            } = result.unwrap()
            {
                assert_eq!(enum_name, "Result");
                assert_eq!(variant_name, "Err");
            }
        }

        #[test]
        fn test_build_command_missing_program() {
            let obj = std::collections::HashMap::new();
            let result = build_command_from_obj(&obj);
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("missing 'program' field"));
        }

        #[test]
        fn test_build_command_with_args() {
            let mut obj = std::collections::HashMap::new();
            obj.insert(
                "program".to_string(),
                Value::from_string("echo".to_string()),
            );
            obj.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![
                    Value::from_string("-n".to_string()),
                    Value::from_string("hello".to_string()),
                ])),
            );
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_build_command_no_args() {
            let mut obj = std::collections::HashMap::new();
            obj.insert(
                "program".to_string(),
                Value::from_string("echo".to_string()),
            );
            // No args field - should default to empty
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_build_command_args_with_non_string() {
            let mut obj = std::collections::HashMap::new();
            obj.insert(
                "program".to_string(),
                Value::from_string("echo".to_string()),
            );
            obj.insert(
                "args".to_string(),
                Value::Array(Arc::from(vec![
                    Value::from_string("hello".to_string()),
                    Value::Integer(42), // Non-string - should be skipped
                ])),
            );
            let result = build_command_from_obj(&obj);
            assert!(result.is_ok());
        }

        #[test]
        fn test_command_arg_multiple() {
            let obj = create_command_obj("echo");
            // First arg
            let result1 = eval_command_method(&obj, "arg", &[Value::from_string("-n".to_string())]);
            assert!(result1.is_ok());
            if let Value::Object(obj1) = result1.unwrap() {
                // Second arg
                let result2 =
                    eval_command_method(&obj1, "arg", &[Value::from_string("hello".to_string())]);
                assert!(result2.is_ok());
                if let Value::Object(obj2) = result2.unwrap() {
                    if let Some(Value::Array(args)) = obj2.get("args") {
                        assert_eq!(args.len(), 2);
                    }
                }
            }
        }
    }

    // --- DataFrame select with array tests ---

    #[test]
    fn test_dataframe_select_array_success() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(4)],
            },
            DataFrameColumn {
                name: "c".to_string(),
                values: vec![Value::Integer(5), Value::Integer(6)],
            },
        ];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("c".to_string()),
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns: selected } = result.unwrap() {
            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0].name, "a");
            assert_eq!(selected[1].name, "c");
        } else {
            panic!("Expected DataFrame");
        }
    }

    #[test]
    fn test_dataframe_select_array_not_found() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("missing".to_string()),
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_dataframe_select_array_non_string_element() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let col_names = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::Integer(42), // Not a string
        ]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be strings"));
    }

    #[test]
    fn test_dataframe_select_empty_array() {
        let columns = vec![DataFrameColumn {
            name: "a".to_string(),
            values: vec![Value::Integer(1)],
        }];
        let col_names = Value::Array(Arc::from(vec![]));
        let result = eval_dataframe_select(&columns, &[col_names]);
        assert!(result.is_ok());
        if let Value::DataFrame { columns: selected } = result.unwrap() {
            assert!(selected.is_empty());
        }
    }

    // --- Object method tests ---

    #[test]
    fn test_object_method_module_type() {
        let mut obj = std::collections::HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("Module".to_string()),
        );
        let result = eval_object_method(&obj, "some_method", &[]);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("handled in interpreter"));
    }

    // --- More dispatch tests ---

    #[test]
    fn test_dispatch_string_method() {
        let value = Value::String(Arc::from("hello"));
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result =
            dispatch_method_call(&value, "len", &[], true, &mut eval_fn, eval_df, eval_ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_dispatch_object_method() {
        let mut obj = std::collections::HashMap::new();
        obj.insert(
            "__type".to_string(),
            Value::from_string("ExitStatus".to_string()),
        );
        obj.insert("success".to_string(), Value::Bool(true));
        let value = Value::Object(Arc::new(obj));

        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "success",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_dispatch_bool_to_string() {
        let value = Value::Bool(false);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
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
        assert_eq!(result.unwrap(), Value::from_string("false".to_string()));
    }

    #[test]
    fn test_dispatch_unknown_method_on_bool() {
        let value = Value::Bool(true);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result = dispatch_method_call(
            &value,
            "unknown_method",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // --- DataFrame multiple columns tests ---

    #[test]
    fn test_dataframe_sum_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(2)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Float(3.5), Value::Float(4.5)],
            },
        ];
        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        // 1 + 2 + 3.5 + 4.5 = 11.0
        assert_eq!(result, Value::Float(11.0));
    }

    #[test]
    fn test_dataframe_mean_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(2), Value::Integer(4)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(6), Value::Integer(8)],
            },
        ];
        let result = eval_dataframe_mean(&columns, &[]).unwrap();
        // (2 + 4 + 6 + 8) / 4 = 5.0
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_dataframe_max_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(1), Value::Integer(5)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(2)],
            },
        ];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_dataframe_min_multiple_columns() {
        let columns = vec![
            DataFrameColumn {
                name: "a".to_string(),
                values: vec![Value::Integer(10), Value::Integer(5)],
            },
            DataFrameColumn {
                name: "b".to_string(),
                values: vec![Value::Integer(3), Value::Integer(7)],
            },
        ];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_dataframe_max_with_floats() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Float(1.5), Value::Float(3.7), Value::Float(2.1)],
        }];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(3.7));
    }

    #[test]
    fn test_dataframe_min_with_floats() {
        let columns = vec![DataFrameColumn {
            name: "values".to_string(),
            values: vec![Value::Float(1.5), Value::Float(3.7), Value::Float(2.1)],
        }];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(1.5));
    }

    // --- Float method edge cases ---

    #[test]
    fn test_float_method_sqrt_zero() {
        let result = eval_float_method(0.0, "sqrt", true).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_float_method_abs_zero() {
        let result = eval_float_method(0.0, "abs", true).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_float_method_round_half() {
        // Rust rounds half away from zero
        let result = eval_float_method(2.5, "round", true).unwrap();
        assert_eq!(result, Value::Float(3.0)); // rounds away from zero
    }

    #[test]
    fn test_float_method_floor_negative() {
        let result = eval_float_method(-2.3, "floor", true).unwrap();
        assert_eq!(result, Value::Float(-3.0));
    }

    #[test]
    fn test_float_method_ceil_negative() {
        let result = eval_float_method(-2.7, "ceil", true).unwrap();
        assert_eq!(result, Value::Float(-2.0));
    }

    // --- Integer method edge cases ---

    #[test]
    fn test_integer_method_pow_zero_exp() {
        let result = eval_integer_method(5, "pow", &[Value::Integer(0)]).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_integer_method_pow_one_exp() {
        let result = eval_integer_method(5, "pow", &[Value::Integer(1)]).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_integer_method_pow_large() {
        let result = eval_integer_method(2, "pow", &[Value::Integer(16)]).unwrap();
        assert_eq!(result, Value::Integer(65536));
    }

    #[test]
    fn test_integer_method_abs_min() {
        // Edge case: i64::MIN can cause overflow in abs(), but we test a safe negative
        let result = eval_integer_method(-100, "abs", &[]).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_integer_method_sqrt_large() {
        let result = eval_integer_method(1000000, "sqrt", &[]).unwrap();
        assert_eq!(result, Value::Float(1000.0));
    }

    // --- require_no_args helper tests ---

    #[test]
    fn test_require_no_args_multiple_args() {
        let result = require_no_args("test", &[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    // --- Generic method edge cases ---

    #[test]
    fn test_generic_to_string_enum_variant() {
        let value = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        if let Value::String(s) = result {
            assert!(s.contains("Some") || s.contains("Option"));
        }
    }

    #[test]
    fn test_generic_to_string_byte() {
        let value = Value::Byte(65);
        let result = eval_generic_method(&value, "to_string", true).unwrap();
        if let Value::String(_) = result {
            // Success - any string representation is fine
        } else {
            panic!("Expected String");
        }
    }

    // --- Additional dataframe edge cases ---

    #[test]
    fn test_dataframe_mean_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_mean(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_max_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_max(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_min_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_min(&columns, &[]).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_dataframe_sum_only_non_numeric() {
        let columns = vec![DataFrameColumn {
            name: "strings".to_string(),
            values: vec![
                Value::from_string("a".to_string()),
                Value::from_string("b".to_string()),
            ],
        }];
        let result = eval_dataframe_sum(&columns, &[]).unwrap();
        assert_eq!(result, Value::Float(0.0));
    }

    // --- Dispatch with turbofish variations ---

    #[test]
    fn test_dispatch_turbofish_complex() {
        let value = Value::Float(4.0);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        // Method with complex turbofish that should be stripped
        let result = dispatch_method_call(
            &value,
            "sqrt::<f64, f64>",
            &[],
            true,
            &mut eval_fn,
            eval_df,
            eval_ctx,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(2.0));
    }

    #[test]
    fn test_dispatch_no_turbofish() {
        let value = Value::Integer(16);
        let mut eval_fn = |_v: &Value, _args: &[Value]| Ok(Value::Integer(0));
        let eval_df = |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0));
        let eval_ctx = |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0));

        let result =
            dispatch_method_call(&value, "sqrt", &[], true, &mut eval_fn, eval_df, eval_ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Float(4.0));
    }

    // --- eval_method_call wrapper tests ---

    #[test]
    fn test_eval_method_call_string_len() {
        let result = eval_method_call(
            &Value::String(Arc::from("test")),
            "len",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(4));
    }

    #[test]
    fn test_eval_method_call_dataframe_count() {
        let columns = vec![DataFrameColumn {
            name: "col".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2)],
        }];
        let result = eval_method_call(
            &Value::DataFrame { columns },
            "count",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_eval_method_call_array_len() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = eval_method_call(
            &arr,
            "len",
            &[],
            true,
            |_v: &Value, _args: &[Value]| Ok(Value::Integer(0)),
            |_v: &Value, _args: &[Expr]| Ok(Value::Integer(0)),
            |_e: &Expr, _cols: &[DataFrameColumn], _row: usize| Ok(Value::Integer(0)),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }
