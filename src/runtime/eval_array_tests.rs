    use super::*;
    use std::sync::Arc;

    // ============================================================================
    // Coverage tests for eval_array_nth (25 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_array_nth_valid_index() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
        ]);
        let result = eval_array_nth(&arr, &Value::Integer(1))
            .expect("nth should succeed for valid index");
        match result {
            Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } => {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant_name, "Some");
                assert_eq!(data, Some(vec![Value::Integer(20)]));
            }
            _ => panic!("Expected EnumVariant"),
        }
    }

    #[test]
    fn test_eval_array_nth_first_element() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::from_string("hello".to_string())]);
        let result = eval_array_nth(&arr, &Value::Integer(0))
            .expect("nth should succeed for index 0");
        match result {
            Value::EnumVariant {
                variant_name, data, ..
            } => {
                assert_eq!(variant_name, "Some");
                assert_eq!(data, Some(vec![Value::from_string("hello".to_string())]));
            }
            _ => panic!("Expected EnumVariant"),
        }
    }

    #[test]
    fn test_eval_array_nth_out_of_bounds() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = eval_array_nth(&arr, &Value::Integer(5))
            .expect("nth should return None for out of bounds");
        match result {
            Value::EnumVariant {
                enum_name,
                variant_name,
                data,
            } => {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant_name, "None");
                assert_eq!(data, None);
            }
            _ => panic!("Expected EnumVariant None"),
        }
    }

    #[test]
    fn test_eval_array_nth_negative_index() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let result = eval_array_nth(&arr, &Value::Integer(-1))
            .expect("nth should return None for negative index");
        match result {
            Value::EnumVariant {
                variant_name, data, ..
            } => {
                assert_eq!(variant_name, "None");
                assert_eq!(data, None);
            }
            _ => panic!("Expected EnumVariant None"),
        }
    }

    #[test]
    fn test_eval_array_nth_non_integer_index() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let result = eval_array_nth(&arr, &Value::from_string("bad".to_string()));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("nth expects integer index"));
    }

    #[test]
    fn test_eval_array_nth_empty_array() {
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let result = eval_array_nth(&arr, &Value::Integer(0))
            .expect("nth on empty array should return None");
        match result {
            Value::EnumVariant {
                variant_name, data, ..
            } => {
                assert_eq!(variant_name, "None");
                assert_eq!(data, None);
            }
            _ => panic!("Expected EnumVariant None"),
        }
    }

    // ============================================================================
    // Coverage tests for eval_array_product (21 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_array_product_integers() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let result = eval_array_product(&arr).expect("product of integers should succeed");
        assert_eq!(result, Value::Integer(24));
    }

    #[test]
    fn test_eval_array_product_empty() {
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let result = eval_array_product(&arr).expect("product of empty should be 1");
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_array_product_single_integer() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(7)]);
        let result = eval_array_product(&arr).expect("product of single integer");
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_eval_array_product_floats() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Float(2.0),
            Value::Float(3.0),
        ]);
        let result = eval_array_product(&arr).expect("product of floats should succeed");
        if let Value::Float(f) = result {
            assert!((f - 6.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_eval_array_product_mixed_int_float() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(3),
            Value::Float(2.5),
        ]);
        let result = eval_array_product(&arr).expect("product of mixed types");
        if let Value::Float(f) = result {
            assert!((f - 7.5).abs() < f64::EPSILON);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_eval_array_product_non_numeric_error() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(2),
            Value::from_string("bad".to_string()),
        ]);
        let result = eval_array_product(&arr);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("numeric"));
    }

    #[test]
    fn test_eval_array_product_with_zero() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(5),
            Value::Integer(0),
            Value::Integer(3),
        ]);
        let result = eval_array_product(&arr).expect("product with zero");
        assert_eq!(result, Value::Integer(0));
    }

    // ============================================================================
    // Coverage tests for eval_array_union (20 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_array_union_basic() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let other = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]));
        let result = eval_array_union(&arr, &other).expect("union should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 5);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_union_no_overlap() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)]));
        let result = eval_array_union(&arr, &other).expect("union should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 4);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_union_all_duplicates() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_array_union(&arr, &other).expect("union should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_union_empty_arrays() {
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let other = Value::Array(Arc::from(vec![]));
        let result = eval_array_union(&arr, &other).expect("union of empty arrays");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_union_with_duplicates_in_first() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(2),
        ]);
        let other = Value::Array(Arc::from(vec![Value::Integer(2), Value::Integer(3)]));
        let result = eval_array_union(&arr, &other).expect("union with dedup");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 3); // [1, 2, 3]
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_union_non_array_arg_error() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let other = Value::Integer(42);
        let result = eval_array_union(&arr, &other);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("union()"));
    }

    // ============================================================================
    // Coverage tests for eval_array_intersection (17 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_array_intersection_basic() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let other = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
            Value::Integer(6),
        ]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
            assert_eq!(result_arr[0], Value::Integer(3));
            assert_eq!(result_arr[1], Value::Integer(4));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_no_common() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_all_common() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_empty_first() {
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let other = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_empty_second() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let other = Value::Array(Arc::from(vec![]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_duplicates_in_first() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        let other = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            // Deduplication: only unique elements
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_intersection_non_array_arg_error() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let other = Value::Integer(42);
        let result = eval_array_intersection(&arr, &other);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("intersection()"));
    }

    #[test]
    fn test_eval_array_intersection_with_strings() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
            Value::from_string("c".to_string()),
        ]);
        let other = Value::Array(Arc::from(vec![
            Value::from_string("b".to_string()),
            Value::from_string("d".to_string()),
        ]));
        let result = eval_array_intersection(&arr, &other).expect("intersection should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 1);
        } else {
            panic!("Expected Array");
        }
    }

    // ============================================================================
    // Coverage tests for eval_array_difference (17 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_eval_array_difference_basic() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        let other = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
            Value::Integer(6),
        ]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
            assert_eq!(result_arr[0], Value::Integer(1));
            assert_eq!(result_arr[1], Value::Integer(2));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_no_overlap() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_all_removed() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_empty_first() {
        let arr: Arc<[Value]> = Arc::from(vec![]);
        let other = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_empty_second() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_duplicates_in_first() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(1),
            Value::Integer(2),
        ]);
        let other = Value::Array(Arc::from(vec![Value::Integer(3)]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            // Deduplication: only unique elements retained
            assert_eq!(result_arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_array_difference_non_array_arg_error() {
        let arr: Arc<[Value]> = Arc::from(vec![Value::Integer(1)]);
        let other = Value::from_string("not an array".to_string());
        let result = eval_array_difference(&arr, &other);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("difference()"));
    }

    #[test]
    fn test_eval_array_difference_with_strings() {
        let arr: Arc<[Value]> = Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
            Value::from_string("c".to_string()),
        ]);
        let other = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("c".to_string()),
        ]));
        let result = eval_array_difference(&arr, &other).expect("difference should succeed");
        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 1);
        } else {
            panic!("Expected Array");
        }
    }
