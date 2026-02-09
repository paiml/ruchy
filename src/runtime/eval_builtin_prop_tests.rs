    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(50))]

        // sqrt of non-negative values is non-negative
        #[test]
        fn prop_sqrt_non_negative(x in 0.0f64..1000.0) {
            let result = eval_sqrt(&[Value::Float(x)]).expect("sqrt should succeed");
            if let Value::Float(f) = result {
                prop_assert!(f >= 0.0, "sqrt({}) = {} should be >= 0", x, f);
            }
        }

        // sqrt then square recovers original (approximately)
        #[test]
        fn prop_sqrt_square_inverse(x in 0.0f64..1000.0) {
            let sqrt_result = eval_sqrt(&[Value::Float(x)]).expect("sqrt");
            if let Value::Float(s) = sqrt_result {
                let squared = s * s;
                prop_assert!((squared - x).abs() < 1e-9, "sqrt({})² = {} should ≈ {}", x, squared, x);
            }
        }

        // abs always returns non-negative
        #[test]
        fn prop_abs_always_non_negative(x in -1000i64..1000) {
            let result = eval_abs(&[Value::Integer(x)]).expect("abs should succeed");
            if let Value::Integer(a) = result {
                prop_assert!(a >= 0, "abs({}) = {} should be >= 0", x, a);
            }
        }

        // abs(x) = abs(-x)
        #[test]
        fn prop_abs_symmetric(x in -1000i64..1000) {
            let pos_result = eval_abs(&[Value::Integer(x)]).expect("abs(x)");
            let neg_result = eval_abs(&[Value::Integer(-x)]).expect("abs(-x)");
            prop_assert_eq!(pos_result, neg_result, "abs({}) should == abs({})", x, -x);
        }

        // min(a, b) <= a and min(a, b) <= b
        #[test]
        fn prop_min_less_than_both(a in -1000i64..1000, b in -1000i64..1000) {
            let result = eval_min(&[Value::Integer(a), Value::Integer(b)]).expect("min");
            if let Value::Integer(m) = result {
                prop_assert!(m <= a, "min({}, {}) = {} should be <= {}", a, b, m, a);
                prop_assert!(m <= b, "min({}, {}) = {} should be <= {}", a, b, m, b);
            }
        }

        // max(a, b) >= a and max(a, b) >= b
        #[test]
        fn prop_max_greater_than_both(a in -1000i64..1000, b in -1000i64..1000) {
            let result = eval_max(&[Value::Integer(a), Value::Integer(b)]).expect("max");
            if let Value::Integer(m) = result {
                prop_assert!(m >= a, "max({}, {}) = {} should be >= {}", a, b, m, a);
                prop_assert!(m >= b, "max({}, {}) = {} should be >= {}", a, b, m, b);
            }
        }

        // min(a, a) == a
        #[test]
        fn prop_min_same_value(a in -1000i64..1000) {
            let result = eval_min(&[Value::Integer(a), Value::Integer(a)]).expect("min");
            prop_assert_eq!(result, Value::Integer(a), "min({}, {}) should == {}", a, a, a);
        }

        // max(a, a) == a
        #[test]
        fn prop_max_same_value(a in -1000i64..1000) {
            let result = eval_max(&[Value::Integer(a), Value::Integer(a)]).expect("max");
            prop_assert_eq!(result, Value::Integer(a), "max({}, {}) should == {}", a, a, a);
        }

        // floor(x) <= x
        #[test]
        fn prop_floor_less_than_or_equal(x in -1000.0f64..1000.0) {
            let result = eval_floor(&[Value::Float(x)]).expect("floor");
            if let Value::Integer(f) = result {
                let f_as_f64 = f as f64;
                prop_assert!(f_as_f64 <= x, "floor({}) = {} should be <= {}", x, f, x);
            }
        }

        // ceil(x) >= x
        #[test]
        fn prop_ceil_greater_than_or_equal(x in -1000.0f64..1000.0) {
            let result = eval_ceil(&[Value::Float(x)]).expect("ceil");
            if let Value::Integer(c) = result {
                let c_as_f64 = c as f64;
                prop_assert!(c_as_f64 >= x, "ceil({}) = {} should be >= {}", x, c, x);
            }
        }

        // range(n) has length n for non-negative n
        #[test]
        fn prop_range_length(n in 0i64..100) {
            let result = eval_range(&[Value::Integer(n)]).expect("range");
            if let Value::Array(arr) = result {
                prop_assert_eq!(arr.len() as i64, n, "range({}) should have length {}", n, n);
            }
        }

        // range(n) starts at 0 and ends at n-1
        #[test]
        fn prop_range_boundaries(n in 1i64..100) {
            let result = eval_range(&[Value::Integer(n)]).expect("range");
            if let Value::Array(arr) = result {
                prop_assert_eq!(arr[0].clone(), Value::Integer(0), "range({}) should start at 0", n);
                let last_idx = arr.len() - 1;
                prop_assert_eq!(arr[last_idx].clone(), Value::Integer(n - 1), "range({}) should end at {}", n, n - 1);
            }
        }

        // len of string equals number of chars
        #[test]
        fn prop_len_string(s in "[a-zA-Z0-9]{0,50}") {
            let result = eval_len(&[Value::from_string(s.clone())]).expect("len");
            if let Value::Integer(len) = result {
                prop_assert_eq!(len as usize, s.chars().count(), "len('{}') should be {}", s, s.chars().count());
            }
        }

        // type always returns a non-empty string
        #[test]
        fn prop_type_non_empty(x in -1000i64..1000) {
            let result = eval_type(&[Value::Integer(x)]).expect("type");
            if let Value::String(s) = result {
                prop_assert!(!s.is_empty(), "type should return non-empty string");
            }
        }

        // reverse twice recovers original
        #[test]
        fn prop_reverse_involutive(
            a in -100i64..100,
            b in -100i64..100,
            c in -100i64..100
        ) {
            let arr = Value::Array(Arc::from(vec![
                Value::Integer(a),
                Value::Integer(b),
                Value::Integer(c),
            ]));
            let reversed = eval_reverse(std::slice::from_ref(&arr)).expect("reverse once");
            let double_reversed = eval_reverse(&[reversed]).expect("reverse twice");
            prop_assert_eq!(double_reversed, arr, "reverse(reverse(arr)) should == arr");
        }

        // pow(x, 0) == 1 for non-zero x
        #[test]
        fn prop_pow_zero_exponent(x in 1i64..100) {
            let result = eval_pow(&[Value::Integer(x), Value::Integer(0)]).expect("pow");
            prop_assert_eq!(result, Value::Integer(1), "{}^0 should == 1", x);
        }

        // pow(x, 1) == x
        #[test]
        fn prop_pow_one_exponent(x in -100i64..100) {
            let result = eval_pow(&[Value::Integer(x), Value::Integer(1)]).expect("pow");
            prop_assert_eq!(result, Value::Integer(x), "{}^1 should == {}", x, x);
        }

        // sin²(x) + cos²(x) = 1
        #[test]
        fn prop_trig_identity(x in -10.0f64..10.0) {
            let sin_result = eval_sin(&[Value::Float(x)]).expect("sin");
            let cos_result = eval_cos(&[Value::Float(x)]).expect("cos");
            if let (Value::Float(s), Value::Float(c)) = (sin_result, cos_result) {
                let identity = s * s + c * c;
                prop_assert!((identity - 1.0).abs() < 1e-10, "sin²({}) + cos²({}) = {} should ≈ 1", x, x, identity);
            }
        }

        // tan(x) = sin(x) / cos(x) (for small x where cos(x) != 0)
        #[test]
        fn prop_tan_definition(x in -1.0f64..1.0) {
            let sin_result = eval_sin(&[Value::Float(x)]).expect("sin");
            let cos_result = eval_cos(&[Value::Float(x)]).expect("cos");
            let tan_result = eval_tan(&[Value::Float(x)]).expect("tan");
            if let (Value::Float(s), Value::Float(c), Value::Float(t)) = (sin_result, cos_result, tan_result) {
                if c.abs() > 0.01 {
                    let expected = s / c;
                    prop_assert!((t - expected).abs() < 1e-10, "tan({}) = {} should ≈ sin/cos = {}", x, t, expected);
                }
            }
        }

        // log(exp(x)) ≈ x for reasonable x
        #[test]
        fn prop_log_exp_inverse(x in -10.0f64..10.0) {
            let exp_result = eval_exp(&[Value::Float(x)]).expect("exp");
            if let Value::Float(e) = exp_result {
                if e > 0.0 && e.is_finite() {
                    let log_result = eval_log(&[Value::Float(e)]).expect("log");
                    if let Value::Float(l) = log_result {
                        prop_assert!((l - x).abs() < 1e-9, "log(exp({})) = {} should ≈ {}", x, l, x);
                    }
                }
            }
        }
    }

    // ============================================================================
    // COVERAGE-95%: Additional Comprehensive Tests
    // ============================================================================

    #[test]
    fn test_eval_log() {
        let args = vec![Value::Float(std::f64::consts::E)];
        let result = eval_log(&args).expect("eval_log should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "ln(e) should be ~1");
        } else {
            panic!("Expected float result");
        }

        let args = vec![Value::Integer(1)];
        let result = eval_log(&args).expect("eval_log should succeed");
        if let Value::Float(v) = result {
            assert!((v - 0.0).abs() < 1e-10, "ln(1) should be ~0");
        }
    }

    #[test]
    fn test_eval_log10() {
        let args = vec![Value::Float(100.0)];
        let result = eval_log10(&args).expect("eval_log10 should succeed");
        if let Value::Float(v) = result {
            assert!((v - 2.0).abs() < 1e-10, "log10(100) should be ~2");
        }

        let args = vec![Value::Integer(10)];
        let result = eval_log10(&args).expect("eval_log10 should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "log10(10) should be ~1");
        }
    }

    #[test]
    fn test_eval_exp() {
        let args = vec![Value::Float(0.0)];
        let result = eval_exp(&args).expect("eval_exp should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "exp(0) should be ~1");
        }

        let args = vec![Value::Integer(0)];
        let result = eval_exp(&args).expect("eval_exp should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "exp(0) should be ~1");
        }
    }

    #[test]
    fn test_eval_random() {
        let args = vec![];
        let result = eval_random(&args).expect("eval_random should succeed");
        if let Value::Float(v) = result {
            assert!(v >= 0.0 && v < 1.0, "random() should be in [0, 1)");
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_eval_type_of() {
        let args = vec![Value::Integer(42)];
        let result = eval_type_of(&args).expect("eval_type_of should succeed");
        assert_eq!(result, Value::from_string("integer".to_string()));

        let args = vec![Value::Nil];
        let result = eval_type_of(&args).expect("eval_type_of should succeed");
        assert_eq!(result, Value::from_string("nil".to_string()));

        let args = vec![Value::Bool(true)];
        let result = eval_type_of(&args).expect("eval_type_of should succeed");
        assert_eq!(result, Value::from_string("boolean".to_string()));
    }

    #[test]
    fn test_eval_is_nil() {
        let args = vec![Value::Nil];
        let result = eval_is_nil(&args).expect("eval_is_nil should succeed");
        assert_eq!(result, Value::Bool(true));

        let args = vec![Value::Integer(0)];
        let result = eval_is_nil(&args).expect("eval_is_nil should succeed");
        assert_eq!(result, Value::Bool(false));

        let args = vec![Value::from_string("".to_string())];
        let result = eval_is_nil(&args).expect("eval_is_nil should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_push() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let args = vec![arr, Value::Integer(3)];
        let result = eval_push(&args).expect("eval_push should succeed");
        if let Value::Array(new_arr) = result {
            assert_eq!(new_arr.len(), 3);
            assert_eq!(new_arr[2], Value::Integer(3));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_pop() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let args = vec![arr];
        let result = eval_pop(&args).expect("eval_pop should succeed");
        // eval_pop returns the popped value directly
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_sort() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::Integer(1),
            Value::Integer(2),
        ]));
        let args = vec![arr];
        let result = eval_sort(&args).expect("eval_sort should succeed");
        if let Value::Array(sorted) = result {
            assert_eq!(sorted[0], Value::Integer(1));
            assert_eq!(sorted[1], Value::Integer(2));
            assert_eq!(sorted[2], Value::Integer(3));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_zip() {
        let arr1 = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let arr2 = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let args = vec![arr1, arr2];
        let result = eval_zip(&args).expect("eval_zip should succeed");
        if let Value::Array(zipped) = result {
            assert_eq!(zipped.len(), 2);
            if let Value::Tuple(first) = &zipped[0] {
                assert_eq!(first[0], Value::Integer(1));
            }
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_enumerate() {
        let arr = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let args = vec![arr];
        let result = eval_enumerate(&args).expect("eval_enumerate should succeed");
        if let Value::Array(enumerated) = result {
            assert_eq!(enumerated.len(), 2);
            if let Value::Tuple(first) = &enumerated[0] {
                assert_eq!(first[0], Value::Integer(0));
            }
            if let Value::Tuple(second) = &enumerated[1] {
                assert_eq!(second[0], Value::Integer(1));
            }
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_assert_eq_success() {
        let args = vec![Value::Integer(42), Value::Integer(42)];
        let result = eval_assert_eq(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_assert_eq_failure() {
        let args = vec![Value::Integer(42), Value::Integer(43)];
        let result = eval_assert_eq(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_success() {
        let args = vec![Value::Bool(true)];
        let result = eval_assert(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_assert_failure() {
        let args = vec![Value::Bool(false)];
        let result = eval_assert(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_with_message() {
        let args = vec![
            Value::Bool(false),
            Value::from_string("custom error".to_string()),
        ];
        let result = eval_assert(&args);
        assert!(result.is_err());
        if let Err(InterpreterError::AssertionFailed(msg)) = result {
            // Message may include "Assertion failed:" prefix from format!("{}", Value)
            assert!(
                msg.contains("custom error"),
                "Message should contain custom error text"
            );
        }
    }

    #[test]
    fn test_eval_range_two_args() {
        let args = vec![Value::Integer(2), Value::Integer(5)];
        let result = eval_range(&args).expect("eval_range should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2));
            assert_eq!(arr[1], Value::Integer(3));
            assert_eq!(arr[2], Value::Integer(4));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_range_three_args() {
        let args = vec![Value::Integer(0), Value::Integer(10), Value::Integer(2)];
        let result = eval_range(&args).expect("eval_range should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[4], Value::Integer(8));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_range_negative_step() {
        let args = vec![Value::Integer(5), Value::Integer(0), Value::Integer(-1)];
        let result = eval_range(&args).expect("eval_range should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(5));
            assert_eq!(arr[4], Value::Integer(1));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_min_floats() {
        let args = vec![Value::Float(5.5), Value::Float(3.3)];
        let result = eval_min(&args).expect("eval_min should succeed");
        assert_eq!(result, Value::Float(3.3));
    }

    #[test]
    fn test_eval_max_floats() {
        let args = vec![Value::Float(5.5), Value::Float(3.3)];
        let result = eval_max(&args).expect("eval_max should succeed");
        assert_eq!(result, Value::Float(5.5));
    }

    #[test]
    fn test_eval_len_tuple() {
        let tuple = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let args = vec![tuple];
        let result = eval_len(&args).expect("eval_len should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_type_string() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("string".to_string()));
    }

    #[test]
    fn test_eval_type_array() {
        let args = vec![Value::Array(Arc::from(vec![]))];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("array".to_string()));
    }

    #[test]
    fn test_eval_type_nil() {
        let args = vec![Value::Nil];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("nil".to_string()));
    }

    #[test]
    fn test_eval_type_bool() {
        let args = vec![Value::Bool(true)];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("boolean".to_string()));
    }

    #[test]
    fn test_eval_cos() {
        let args = vec![Value::Float(0.0)];
        let result = eval_cos(&args).expect("eval_cos should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "cos(0) should be ~1");
        }

        let args = vec![Value::Integer(0)];
        let result = eval_cos(&args).expect("eval_cos should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10, "cos(0) should be ~1");
        }
    }

    #[test]
    fn test_eval_tan() {
        let args = vec![Value::Float(0.0)];
        let result = eval_tan(&args).expect("eval_tan should succeed");
        if let Value::Float(v) = result {
            assert!((v - 0.0).abs() < 1e-10, "tan(0) should be ~0");
        }

        let args = vec![Value::Integer(0)];
        let result = eval_tan(&args).expect("eval_tan should succeed");
        if let Value::Float(v) = result {
            assert!((v - 0.0).abs() < 1e-10, "tan(0) should be ~0");
        }
    }

    #[test]
    fn test_eval_str() {
        let args = vec![Value::Integer(42)];
        let result = eval_str(&args).expect("eval_str should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));

        let args = vec![Value::Float(3.14)];
        let result = eval_str(&args).expect("eval_str should succeed");
        if let Value::String(s) = result {
            assert!(s.starts_with("3.14"));
        }

        let args = vec![Value::Bool(true)];
        let result = eval_str(&args).expect("eval_str should succeed");
        assert_eq!(result, Value::from_string("true".to_string()));
    }

    #[test]
    fn test_eval_to_string() {
        let args = vec![Value::Integer(42)];
        let result = eval_to_string(&args).expect("eval_to_string should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    #[test]
    fn test_eval_int() {
        let args = vec![Value::Float(3.9)];
        let result = eval_int(&args).expect("eval_int should succeed");
        assert_eq!(result, Value::Integer(3));

        let args = vec![Value::from_string("42".to_string())];
        let result = eval_int(&args).expect("eval_int should succeed");
        assert_eq!(result, Value::Integer(42));

        let args = vec![Value::Bool(true)];
        let result = eval_int(&args).expect("eval_int should succeed");
        assert_eq!(result, Value::Integer(1));

        let args = vec![Value::Bool(false)];
        let result = eval_int(&args).expect("eval_int should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_float() {
        let args = vec![Value::Integer(42)];
        let result = eval_float(&args).expect("eval_float should succeed");
        assert_eq!(result, Value::Float(42.0));

        let args = vec![Value::from_string("3.14".to_string())];
        let result = eval_float(&args).expect("eval_float should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.14).abs() < 0.001);
        }
    }

    #[test]
    fn test_eval_bool() {
        let args = vec![Value::Integer(0)];
        let result = eval_bool(&args).expect("eval_bool should succeed");
        assert_eq!(result, Value::Bool(false));

        let args = vec![Value::Integer(1)];
        let result = eval_bool(&args).expect("eval_bool should succeed");
        assert_eq!(result, Value::Bool(true));

        let args = vec![Value::from_string("".to_string())];
        let result = eval_bool(&args).expect("eval_bool should succeed");
        assert_eq!(result, Value::Bool(false));

        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_bool(&args).expect("eval_bool should succeed");
        assert_eq!(result, Value::Bool(true));

        let args = vec![Value::Nil];
        let result = eval_bool(&args).expect("eval_bool should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_parse_int() {
        let args = vec![Value::from_string("42".to_string())];
        let result = eval_parse_int(&args).expect("eval_parse_int should succeed");
        assert_eq!(result, Value::Integer(42));

        let args = vec![Value::from_string("-123".to_string())];
        let result = eval_parse_int(&args).expect("eval_parse_int should succeed");
        assert_eq!(result, Value::Integer(-123));
    }

    #[test]
    fn test_eval_parse_float() {
        let args = vec![Value::from_string("3.14".to_string())];
        let result = eval_parse_float(&args).expect("eval_parse_float should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.14).abs() < 0.001);
        }
    }

    #[test]
    fn test_eval_timestamp() {
        let args = vec![];
        let result = eval_timestamp(&args).expect("eval_timestamp should succeed");
        if let Value::Integer(ts) = result {
            assert!(ts > 0, "timestamp should be positive");
        } else {
            panic!("Expected integer result");
        }
    }

    #[test]
    fn test_try_eval_io_function_unknown() {
        let result = try_eval_io_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_math_function_unknown() {
        let result = try_eval_math_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_utility_function_unknown() {
        let result = try_eval_utility_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_collection_function_unknown() {
        let result = try_eval_collection_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_conversion_function_unknown() {
        let result = try_eval_conversion_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_time_function_unknown() {
        let result = try_eval_time_function("unknown", &[]).expect("should not error");
        assert!(result.is_none());
    }

    #[test]
    fn test_eval_builtin_function_unknown() {
        let result =
            eval_builtin_function("unknown_function", &[]).expect("should not error for unknown");
        assert!(result.is_none());
    }

    #[test]
    fn test_eval_builtin_function_println() {
        let result = eval_builtin_function("__builtin_println__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_sqrt() {
        let result = eval_builtin_function("__builtin_sqrt__", &[Value::Integer(16)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Float(4.0)));
    }

    #[test]
    fn test_eval_builtin_function_len() {
        let result = eval_builtin_function(
            "__builtin_len__",
            &[Value::from_string("hello".to_string())],
        )
        .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(5)));
    }

    #[test]
    fn test_eval_builtin_function_range() {
        let result = eval_builtin_function("__builtin_range__", &[Value::Integer(3)])
            .expect("should succeed");
        assert!(result.is_some());
        if let Some(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 3);
        }
    }

    #[test]
    fn test_eval_builtin_function_type() {
        let result = eval_builtin_function("__builtin_type__", &[Value::Integer(42)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::from_string("integer".to_string())));
    }

    #[test]
    fn test_eval_builtin_function_str() {
        let result = eval_builtin_function("__builtin_str__", &[Value::Integer(42)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::from_string("42".to_string())));
    }

    #[test]
    fn test_eval_builtin_function_int() {
        let result =
            eval_builtin_function("__builtin_int__", &[Value::Float(3.9)]).expect("should succeed");
        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[test]
    fn test_eval_builtin_function_float() {
        let result = eval_builtin_function("__builtin_float__", &[Value::Integer(42)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Float(42.0)));
    }

    #[test]
    fn test_eval_builtin_function_bool() {
        let result = eval_builtin_function("__builtin_bool__", &[Value::Integer(1)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_eval_sort_strings() {
        let arr = Value::Array(Arc::from(vec![
            Value::from_string("c".to_string()),
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let args = vec![arr];
        let result = eval_sort(&args).expect("eval_sort should succeed");
        if let Value::Array(sorted) = result {
            assert_eq!(sorted[0], Value::from_string("a".to_string()));
            assert_eq!(sorted[1], Value::from_string("b".to_string()));
            assert_eq!(sorted[2], Value::from_string("c".to_string()));
        }
    }

    #[test]
    fn test_eval_sort_floats() {
        let arr = Value::Array(Arc::from(vec![
            Value::Float(3.3),
            Value::Float(1.1),
            Value::Float(2.2),
        ]));
        let args = vec![arr];
        let result = eval_sort(&args).expect("eval_sort should succeed");
        if let Value::Array(sorted) = result {
            assert_eq!(sorted[0], Value::Float(1.1));
            assert_eq!(sorted[1], Value::Float(2.2));
            assert_eq!(sorted[2], Value::Float(3.3));
        }
    }

    #[test]
    fn test_eval_sqrt_from_integer() {
        let args = vec![Value::Integer(25)];
        let result = eval_sqrt(&args).expect("eval_sqrt should succeed");
        if let Value::Float(v) = result {
            assert!((v - 5.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_pow_with_floats() {
        let args = vec![Value::Float(2.0), Value::Float(3.0)];
        let result = eval_pow(&args).expect("eval_pow should succeed");
        if let Value::Float(v) = result {
            assert!((v - 8.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_min_with_floats() {
        let args = vec![Value::Float(5.5), Value::Float(3.3)];
        let result = eval_min(&args).expect("eval_min should succeed");
        assert_eq!(result, Value::Float(3.3));
    }

    #[test]
    fn test_eval_max_with_floats() {
        let args = vec![Value::Float(5.5), Value::Float(3.3)];
        let result = eval_max(&args).expect("eval_max should succeed");
        assert_eq!(result, Value::Float(5.5));
    }

    #[test]
    fn test_eval_floor_with_negative() {
        let args = vec![Value::Float(-3.2)];
        let result = eval_floor(&args).expect("eval_floor should succeed");
        assert_eq!(result, Value::Integer(-4));
    }

    #[test]
    fn test_eval_ceil_with_negative() {
        let args = vec![Value::Float(-3.7)];
        let result = eval_ceil(&args).expect("eval_ceil should succeed");
        assert_eq!(result, Value::Integer(-3));
    }

    #[test]
    fn test_eval_round_down_case() {
        let args = vec![Value::Float(3.4)];
        let result = eval_round(&args).expect("eval_round should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_range_single_arg() {
        let args = vec![Value::Integer(5)];
        let result = eval_range(&args).expect("eval_range should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[4], Value::Integer(4));
        }
    }

    #[test]
    fn test_eval_reverse_on_array() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let args = vec![arr];
        let result = eval_reverse(&args).expect("eval_reverse should succeed");
        if let Value::Array(reversed) = result {
            assert_eq!(reversed[0], Value::Integer(3));
            assert_eq!(reversed[1], Value::Integer(2));
            assert_eq!(reversed[2], Value::Integer(1));
        }
    }

    #[test]
    fn test_eval_reverse_on_string() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_reverse(&args).expect("eval_reverse should succeed");
        assert_eq!(result, Value::from_string("olleh".to_string()));
    }

    #[test]
    fn test_eval_is_nil_returns_true() {
        let args = vec![Value::Nil];
        let result = eval_is_nil(&args).expect("eval_is_nil should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_is_nil_returns_false() {
        let args = vec![Value::Integer(0)];
        let result = eval_is_nil(&args).expect("eval_is_nil should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_push_element_to_array() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let args = vec![arr, Value::Integer(2)];
        let result = eval_push(&args).expect("eval_push should succeed");
        if let Value::Array(new_arr) = result {
            assert_eq!(new_arr.len(), 2);
        }
    }

    #[test]
    fn test_eval_pop_element_from_array() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let args = vec![arr];
        let result = eval_pop(&args).expect("eval_pop should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_pop_from_empty_array() {
        let arr = Value::Array(Arc::from(vec![]));
        let args = vec![arr];
        let result = eval_pop(&args).expect("eval_pop should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_type_for_integer_value() {
        let args = vec![Value::Integer(42)];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("integer".to_string()));
    }

    #[test]
    fn test_eval_type_for_float_value() {
        let args = vec![Value::Float(3.14)];
        let result = eval_type(&args).expect("eval_type should succeed");
        assert_eq!(result, Value::from_string("float".to_string()));
    }

    #[test]
    fn test_eval_abs_on_negative_integer() {
        let args = vec![Value::Integer(-42)];
        let result = eval_abs(&args).expect("eval_abs should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_abs_on_negative_float() {
        let args = vec![Value::Float(-3.14)];
        let result = eval_abs(&args).expect("eval_abs should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.14).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_len_on_string() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_len(&args).expect("eval_len should succeed");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_eval_len_on_array() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let args = vec![arr];
        let result = eval_len(&args).expect("eval_len should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_len_on_empty_string() {
        let args = vec![Value::from_string("".to_string())];
        let result = eval_len(&args).expect("eval_len should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_len_on_empty_array() {
        let arr = Value::Array(Arc::from(vec![]));
        let args = vec![arr];
        let result = eval_len(&args).expect("eval_len should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    // ============================================================================
    // EXTREME TDD Round 130: Comprehensive builtin coverage tests
    // Target: 55.41% → 90%+ coverage
    // ============================================================================

    // --- Math error path tests ---
    #[test]
    fn test_eval_sqrt_error_on_string() {
        let args = vec![Value::from_string("not a number".to_string())];
        let result = eval_sqrt(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expects a number"));
    }

    #[test]
    fn test_eval_sqrt_error_on_wrong_arg_count() {
        let args = vec![];
        let result = eval_sqrt(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pow_error_on_string() {
        let args = vec![Value::from_string("a".to_string()), Value::Integer(2)];
        let result = eval_pow(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("expects two numbers"));
    }

    #[test]
    fn test_eval_pow_negative_exponent() {
        let args = vec![Value::Integer(2), Value::Integer(-2)];
        let result = eval_pow(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 0.25).abs() < 1e-10);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_eval_pow_mixed_int_float() {
        let args = vec![Value::Integer(2), Value::Float(3.0)];
        let result = eval_pow(&args).expect("should succeed");
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_eval_pow_float_int() {
        let args = vec![Value::Float(2.0), Value::Integer(3)];
        let result = eval_pow(&args).expect("should succeed");
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_eval_abs_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_abs(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_min_error_on_string() {
        let args = vec![Value::from_string("a".to_string()), Value::Integer(1)];
        let result = eval_min(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_min_mixed_int_float() {
        let args = vec![Value::Integer(5), Value::Float(3.3)];
        let result = eval_min(&args).expect("should succeed");
        assert_eq!(result, Value::Float(3.3));
    }

    #[test]
    fn test_eval_min_float_int() {
        let args = vec![Value::Float(2.5), Value::Integer(3)];
        let result = eval_min(&args).expect("should succeed");
        assert_eq!(result, Value::Float(2.5));
    }

    #[test]
    fn test_eval_max_error_on_string() {
        let args = vec![Value::from_string("a".to_string()), Value::Integer(1)];
        let result = eval_max(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_max_mixed_int_float() {
        let args = vec![Value::Integer(5), Value::Float(3.3)];
        let result = eval_max(&args).expect("should succeed");
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_eval_max_float_int() {
        let args = vec![Value::Float(2.5), Value::Integer(3)];
        let result = eval_max(&args).expect("should succeed");
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_eval_floor_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_floor(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_floor_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_floor(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_ceil_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_ceil(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_ceil_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_ceil(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_round_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_round(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_round_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_round(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    // --- Trigonometric function tests ---
    #[test]
    fn test_eval_sin_on_integer() {
        let args = vec![Value::Integer(0)];
        let result = eval_sin(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_sin_on_float() {
        let args = vec![Value::Float(0.0)];
        let result = eval_sin(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_sin_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_sin(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_cos_on_integer() {
        let args = vec![Value::Integer(0)];
        let result = eval_cos(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_cos_on_float() {
        let args = vec![Value::Float(0.0)];
        let result = eval_cos(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_cos_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_cos(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_tan_on_integer() {
        let args = vec![Value::Integer(0)];
        let result = eval_tan(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_tan_on_float() {
        let args = vec![Value::Float(0.0)];
        let result = eval_tan(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_tan_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_tan(&args);
        assert!(result.is_err());
    }

    // --- Logarithmic and exponential function tests ---
    #[test]
    fn test_eval_log_on_integer() {
        let args = vec![Value::Integer(1)];
        let result = eval_log(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v.abs() < 1e-10); // ln(1) = 0
        }
    }

    #[test]
    fn test_eval_log_on_float() {
        use std::f64::consts::E;
        let args = vec![Value::Float(E)];
        let result = eval_log(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10); // ln(e) = 1
        }
    }

    #[test]
    fn test_eval_log_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_log(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_log10_on_integer() {
        let args = vec![Value::Integer(100)];
        let result = eval_log10(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 2.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_log10_on_float() {
        let args = vec![Value::Float(1000.0)];
        let result = eval_log10(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.0).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_log10_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_log10(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_exp_on_integer() {
        let args = vec![Value::Integer(0)];
        let result = eval_exp(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 1.0).abs() < 1e-10); // e^0 = 1
        }
    }

    #[test]
    fn test_eval_exp_on_float() {
        let args = vec![Value::Float(1.0)];
        let result = eval_exp(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - std::f64::consts::E).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_exp_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_exp(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_random_returns_float_in_range() {
        let args = vec![];
        let result = eval_random(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!(v >= 0.0 && v < 1.0);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_eval_random_error_on_args() {
        let args = vec![Value::Integer(1)];
        let result = eval_random(&args);
        assert!(result.is_err());
    }

    // --- Range function tests ---
    #[test]
    fn test_eval_range_two_args_start_end() {
        let args = vec![Value::Integer(2), Value::Integer(5)];
        let result = eval_range(&args).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(4));
        }
    }

    #[test]
    fn test_eval_range_three_args_positive_step() {
        let args = vec![Value::Integer(0), Value::Integer(10), Value::Integer(2)];
        let result = eval_range(&args).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[4], Value::Integer(8));
        }
    }

    #[test]
    fn test_eval_range_three_args_negative_step() {
        let args = vec![Value::Integer(10), Value::Integer(0), Value::Integer(-2)];
        let result = eval_range(&args).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
            assert_eq!(arr[0], Value::Integer(10));
            assert_eq!(arr[4], Value::Integer(2));
        }
    }

    #[test]
    fn test_eval_range_zero_step_error() {
        let args = vec![Value::Integer(0), Value::Integer(10), Value::Integer(0)];
        let result = eval_range(&args);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("step cannot be zero"));
    }

    #[test]
    fn test_eval_range_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_range(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_range_two_args_error_on_string() {
        let args = vec![Value::from_string("a".to_string()), Value::Integer(5)];
        let result = eval_range(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_range_three_args_error_on_string() {
        let args = vec![
            Value::Integer(0),
            Value::from_string("x".to_string()),
            Value::Integer(1),
        ];
        let result = eval_range(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_range_invalid_arg_count() {
        let args = vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ];
        let result = eval_range(&args);
        assert!(result.is_err());
    }

    // --- Collection function tests ---
    #[test]
    fn test_eval_len_on_tuple() {
        let tuple = Value::Tuple(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let args = vec![tuple];
        let result = eval_len(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_len_error_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_len(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_reverse_error_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_reverse(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_push_error_on_non_array() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = eval_push(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pop_error_on_non_array() {
        let args = vec![Value::Integer(1)];
        let result = eval_pop(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sort_error_on_non_array() {
        let args = vec![Value::Integer(1)];
        let result = eval_sort(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sort_mixed_types() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::from_string("b".to_string()),
            Value::Integer(1),
        ]));
        let args = vec![arr];
        // Should not error, just use default ordering
        let result = eval_sort(&args);
        assert!(result.is_ok());
    }

    // --- Zip and Enumerate tests ---
    #[test]
    fn test_eval_zip_success() {
        let a = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let b = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let args = vec![a, b];
        let result = eval_zip(&args).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
            if let Value::Tuple(t) = &arr[0] {
                assert_eq!(t[0], Value::Integer(1));
            }
        }
    }

    #[test]
    fn test_eval_zip_error_on_non_arrays() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = eval_zip(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_enumerate_success() {
        let arr = Value::Array(Arc::from(vec![
            Value::from_string("a".to_string()),
            Value::from_string("b".to_string()),
        ]));
        let args = vec![arr];
        let result = eval_enumerate(&args).expect("should succeed");
        if let Value::Array(enumerated) = result {
            assert_eq!(enumerated.len(), 2);
            if let Value::Tuple(t) = &enumerated[0] {
                assert_eq!(t[0], Value::Integer(0));
                assert_eq!(t[1], Value::from_string("a".to_string()));
            }
        }
    }

    #[test]
    fn test_eval_enumerate_error_on_non_array() {
        let args = vec![Value::Integer(1)];
        let result = eval_enumerate(&args);
        assert!(result.is_err());
    }

    // --- Type inspection tests ---
    #[test]
    fn test_eval_type_of_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_type_of(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("integer".to_string()));
    }

    #[test]
    fn test_eval_type_of_string() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_type_of(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("string".to_string()));
    }

    #[test]
    fn test_eval_type_for_bool() {
        let args = vec![Value::Bool(true)];
        let result = eval_type(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("boolean".to_string()));
    }

    #[test]
    fn test_eval_type_for_nil() {
        let args = vec![Value::Nil];
        let result = eval_type(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("nil".to_string()));
    }

    #[test]
    fn test_eval_type_for_array() {
        let arr = Value::Array(Arc::from(vec![]));
        let args = vec![arr];
        let result = eval_type(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("array".to_string()));
    }

    // --- Conversion function tests ---
    #[test]
    fn test_eval_str_on_bool() {
        let args = vec![Value::Bool(true)];
        let result = eval_str(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("true".to_string()));
    }

    #[test]
    fn test_eval_str_on_nil() {
        let args = vec![Value::Nil];
        let result = eval_str(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("nil".to_string()));
    }

    #[test]
    fn test_eval_to_string_on_integer() {
        let args = vec![Value::Integer(42)];
        let result = eval_to_string(&args).expect("should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));
    }

    #[test]
    fn test_eval_int_from_string() {
        let args = vec![Value::from_string("42".to_string())];
        let result = eval_int(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_int_from_bool() {
        let args = vec![Value::Bool(true)];
        let result = eval_int(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_int_from_bool_false() {
        let args = vec![Value::Bool(false)];
        let result = eval_int(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_int_error_on_invalid_string() {
        let args = vec![Value::from_string("not a number".to_string())];
        let result = eval_int(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_float_from_string() {
        let args = vec![Value::from_string("3.14".to_string())];
        let result = eval_float(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.14).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_float_from_bool() {
        let args = vec![Value::Bool(true)];
        let result = eval_float(&args).expect("should succeed");
        assert_eq!(result, Value::Float(1.0));
    }

    #[test]
    fn test_eval_float_error_on_invalid_string() {
        let args = vec![Value::from_string("not a number".to_string())];
        let result = eval_float(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_int_success() {
        let args = vec![Value::from_string("123".to_string())];
        let result = eval_parse_int(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(123));
    }

    #[test]
    fn test_eval_parse_int_error() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_parse_int(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_int_on_non_string() {
        let args = vec![Value::Integer(42)];
        let result = eval_parse_int(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_float_success() {
        let args = vec![Value::from_string("3.14".to_string())];
        let result = eval_parse_float(&args).expect("should succeed");
        if let Value::Float(v) = result {
            assert!((v - 3.14).abs() < 1e-10);
        }
    }

    #[test]
    fn test_eval_parse_float_error() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_parse_float(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_float_on_non_string() {
        let args = vec![Value::Integer(42)];
        let result = eval_parse_float(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_bool_from_integer_zero() {
        let args = vec![Value::Integer(0)];
        let result = eval_bool(&args).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_bool_from_nil() {
        let args = vec![Value::Nil];
        let result = eval_bool(&args).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_bool_from_empty_string() {
        let args = vec![Value::from_string("".to_string())];
        let result = eval_bool(&args).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_bool_from_nonempty_string() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_bool(&args).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_bool_from_empty_array() {
        let arr = Value::Array(Arc::from(vec![]));
        let args = vec![arr];
        let result = eval_bool(&args).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    // --- IO function tests ---
    #[test]
    fn test_eval_print_empty_args() {
        let args = vec![];
        let result = eval_print(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_println_empty_args() {
        let args = vec![];
        let result = eval_println(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_println_format_interpolation() {
        let args = vec![
            Value::from_string("Hello, {}!".to_string()),
            Value::from_string("World".to_string()),
        ];
        let result = eval_println(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_dbg_single_value() {
        let args = vec![Value::Integer(42)];
        let result = eval_dbg(&args).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_dbg_multiple_values() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = eval_dbg(&args).expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected array");
        }
    }

    // --- Time function tests ---
    #[test]
    fn test_eval_timestamp_returns_integer() {
        let args = vec![];
        let result = eval_timestamp(&args).expect("should succeed");
        if let Value::Integer(v) = result {
            assert!(v > 0);
        } else {
            panic!("Expected integer");
        }
    }

    #[test]
    fn test_eval_timestamp_error_on_args() {
        let args = vec![Value::Integer(1)];
        let result = eval_timestamp(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_chrono_utc_now_returns_string() {
        let args = vec![];
        let result = eval_chrono_utc_now(&args).expect("should succeed");
        if let Value::String(s) = result {
            // RFC3339 format contains 'T'
            assert!(s.contains('T') || s.contains(':'));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_eval_chrono_utc_now_error_on_args() {
        let args = vec![Value::Integer(1)];
        let result = eval_chrono_utc_now(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sleep_with_float() {
        let args = vec![Value::Float(1.0)]; // 1ms
        let result = eval_sleep(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_sleep_error_on_string() {
        let args = vec![Value::from_string("abc".to_string())];
        let result = eval_sleep(&args);
        assert!(result.is_err());
    }

    // --- Assertion function tests (Round 130) ---
    #[test]
    fn test_eval_assert_eq_equals_values() {
        let args = vec![Value::Integer(1), Value::Integer(1)];
        let result = eval_assert_eq(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_assert_eq_not_equals_error() {
        let args = vec![Value::Integer(1), Value::Integer(2)];
        let result = eval_assert_eq(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_truthy_passes() {
        let args = vec![Value::Bool(true)];
        let result = eval_assert(&args).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_assert_falsy_fails() {
        let args = vec![Value::Bool(false)];
        let result = eval_assert(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_custom_message_included() {
        let args = vec![
            Value::Bool(false),
            Value::from_string("custom message".to_string()),
        ];
        let result = eval_assert(&args);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("custom message"));
    }

    // --- Helper function tests ---
    #[test]
    fn test_format_value_for_println_string() {
        let val = Value::from_string("hello".to_string());
        let result = format_value_for_println(&val);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_value_for_println_integer() {
        let val = Value::Integer(42);
        let result = format_value_for_println(&val);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_with_interpolation_multiple() {
        let result = format_with_interpolation(
            "{} + {} = {}",
            &[Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        assert_eq!(result, "1 + 2 = 3");
    }

    #[test]
    fn test_join_values_empty() {
        let result = join_values(&[]);
        assert_eq!(result, "");
    }

    #[test]
    fn test_join_values_multiple() {
        let result = join_values(&[Value::Integer(1), Value::Integer(2)]);
        assert_eq!(result, "1 2");
    }

    #[test]
    fn test_format_println_output_empty() {
        let result = format_println_output(&[]);
        assert_eq!(result, "\n");
    }

    #[test]
    fn test_format_println_output_no_format() {
        let result = format_println_output(&[Value::Integer(1), Value::Integer(2)]);
        assert_eq!(result, "1 2\n");
    }

    // --- Range helper tests ---
    #[test]
    fn test_generate_range_forward() {
        let result = generate_range_forward(0, 5, 2);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Value::Integer(0));
        assert_eq!(result[1], Value::Integer(2));
        assert_eq!(result[2], Value::Integer(4));
    }

    #[test]
    fn test_generate_range_backward() {
        let result = generate_range_backward(10, 0, -2);
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], Value::Integer(10));
        assert_eq!(result[4], Value::Integer(2));
    }

    // --- Dispatcher tests for coverage ---
    #[test]
    fn test_try_eval_io_function_print() {
        let result = try_eval_io_function("__builtin_print__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_io_function_dbg() {
        let result =
            try_eval_io_function("__builtin_dbg__", &[Value::Integer(42)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_io_function_returns_none_for_unknown() {
        let result = try_eval_io_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_basic_math_part1_sqrt() {
        let result = try_eval_basic_math_part1("__builtin_sqrt__", &[Value::Integer(4)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_basic_math_part1_pow() {
        let result =
            try_eval_basic_math_part1("__builtin_pow__", &[Value::Integer(2), Value::Integer(3)])
                .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_basic_math_part2_min() {
        let result =
            try_eval_basic_math_part2("__builtin_min__", &[Value::Integer(1), Value::Integer(2)])
                .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_basic_math_part2_max() {
        let result =
            try_eval_basic_math_part2("__builtin_max__", &[Value::Integer(1), Value::Integer(2)])
                .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part1_floor() {
        let result = try_eval_advanced_math_part1("__builtin_floor__", &[Value::Float(3.5)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part1_ceil() {
        let result = try_eval_advanced_math_part1("__builtin_ceil__", &[Value::Float(3.5)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part1_round() {
        let result = try_eval_advanced_math_part1("__builtin_round__", &[Value::Float(3.5)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part2_sin() {
        let result = try_eval_advanced_math_part2("__builtin_sin__", &[Value::Integer(0)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part2_cos() {
        let result = try_eval_advanced_math_part2("__builtin_cos__", &[Value::Integer(0)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part2_tan() {
        let result = try_eval_advanced_math_part2("__builtin_tan__", &[Value::Integer(0)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part3_log() {
        let result = try_eval_advanced_math_part3("__builtin_log__", &[Value::Integer(1)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part3_log10() {
        let result = try_eval_advanced_math_part3("__builtin_log10__", &[Value::Integer(10)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part3_exp() {
        let result = try_eval_advanced_math_part3("__builtin_exp__", &[Value::Integer(0)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_advanced_math_part3_random() {
        let result =
            try_eval_advanced_math_part3("__builtin_random__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part1_len() {
        let result = try_eval_utility_part1(
            "__builtin_len__",
            &[Value::from_string("hello".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part1_range() {
        let result = try_eval_utility_part1("__builtin_range__", &[Value::Integer(5)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_type() {
        let result = try_eval_utility_part2("__builtin_type__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_type_of() {
        let result = try_eval_utility_part2("__builtin_type_of__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_is_nil() {
        let result =
            try_eval_utility_part2("__builtin_is_nil__", &[Value::Nil]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_reverse() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result = try_eval_utility_part2("__builtin_reverse__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_assert_eq() {
        let result = try_eval_utility_part2(
            "__builtin_assert_eq__",
            &[Value::Integer(1), Value::Integer(1)],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_assert() {
        let result = try_eval_utility_part2("__builtin_assert__", &[Value::Bool(true)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_zip() {
        let a = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let b = Value::Array(Arc::from(vec![Value::Integer(2)]));
        let result = try_eval_utility_part2("__builtin_zip__", &[a, b]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_utility_part2_enumerate() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result =
            try_eval_utility_part2("__builtin_enumerate__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_collection_function_push() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result = try_eval_collection_function("__builtin_push__", &[arr, Value::Integer(2)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_collection_function_pop() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result =
            try_eval_collection_function("__builtin_pop__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_collection_function_sort() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(2), Value::Integer(1)]));
        let result =
            try_eval_collection_function("__builtin_sort__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_str() {
        let result = try_eval_conversion_function("__builtin_str__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_to_string() {
        let result = try_eval_conversion_function("__builtin_to_string__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_int() {
        let result = try_eval_conversion_function("__builtin_int__", &[Value::Float(3.5)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_float() {
        let result = try_eval_conversion_function("__builtin_float__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_bool() {
        let result = try_eval_conversion_function("__builtin_bool__", &[Value::Integer(1)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_parse_int() {
        let result = try_eval_conversion_function(
            "__builtin_parse_int__",
            &[Value::from_string("42".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_conversion_function_parse_float() {
        let result = try_eval_conversion_function(
            "__builtin_parse_float__",
            &[Value::from_string("3.14".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_time_function_sleep() {
        let result = try_eval_time_function("__builtin_sleep__", &[Value::Integer(1)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_time_function_timestamp() {
        let result = try_eval_time_function("__builtin_timestamp__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_time_function_chrono_utc_now() {
        let result =
            try_eval_time_function("__builtin_chrono_utc_now__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    // --- Main dispatcher tests ---
    #[test]
    fn test_eval_builtin_function_abs() {
        let result = eval_builtin_function("__builtin_abs__", &[Value::Integer(-5)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(5)));
    }

    #[test]
    fn test_eval_builtin_function_min() {
        let result =
            eval_builtin_function("__builtin_min__", &[Value::Integer(3), Value::Integer(5)])
                .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[test]
    fn test_eval_builtin_function_max() {
        let result =
            eval_builtin_function("__builtin_max__", &[Value::Integer(3), Value::Integer(5)])
                .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(5)));
    }

    #[test]
    fn test_eval_builtin_function_floor() {
        let result = eval_builtin_function("__builtin_floor__", &[Value::Float(3.7)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(3)));
    }

    #[test]
    fn test_eval_builtin_function_ceil() {
        let result = eval_builtin_function("__builtin_ceil__", &[Value::Float(3.2)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(4)));
    }

    #[test]
    fn test_eval_builtin_function_round() {
        let result = eval_builtin_function("__builtin_round__", &[Value::Float(3.5)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(4)));
    }

    #[test]
    fn test_eval_builtin_function_reverse() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_builtin_function("__builtin_reverse__", &[arr]).expect("should succeed");
        if let Some(Value::Array(reversed)) = result {
            assert_eq!(reversed[0], Value::Integer(2));
        }
    }

    #[test]
    fn test_eval_builtin_function_push() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result = eval_builtin_function("__builtin_push__", &[arr, Value::Integer(2)])
            .expect("should succeed");
        if let Some(Value::Array(pushed)) = result {
            assert_eq!(pushed.len(), 2);
        }
    }

    #[test]
    fn test_eval_builtin_function_pop() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = eval_builtin_function("__builtin_pop__", &[arr]).expect("should succeed");
        assert_eq!(result, Some(Value::Integer(2)));
    }

    #[test]
    fn test_eval_builtin_function_is_nil() {
        let result =
            eval_builtin_function("__builtin_is_nil__", &[Value::Nil]).expect("should succeed");
        assert_eq!(result, Some(Value::Bool(true)));
    }

    #[test]
    fn test_eval_builtin_function_type_of() {
        let result = eval_builtin_function("__builtin_type_of__", &[Value::Float(3.14)])
            .expect("should succeed");
        assert_eq!(result, Some(Value::from_string("float".to_string())));
    }

    // ============================================================================
    // COMPREHENSIVE COVERAGE TESTS - Error Cases and Edge Cases
    // ============================================================================

    // --- Error Cases: Wrong argument counts ---

    #[test]
    fn test_eval_sqrt_wrong_arg_count() {
        let result = eval_sqrt(&[]);
        assert!(result.is_err());
        if let Err(InterpreterError::RuntimeError(msg)) = result {
            assert!(msg.contains("expects"));
        }
    }

    #[test]
    fn test_eval_sqrt_wrong_type() {
        let result = eval_sqrt(&[Value::from_string("not a number".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pow_wrong_arg_count() {
        let result = eval_pow(&[Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pow_wrong_type() {
        let result = eval_pow(&[Value::from_string("a".to_string()), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_abs_wrong_arg_count() {
        let result = eval_abs(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_abs_wrong_type() {
        let result = eval_abs(&[Value::from_string("abc".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_min_wrong_arg_count() {
        let result = eval_min(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_min_wrong_type() {
        let result = eval_min(&[Value::from_string("a".to_string()), Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_max_wrong_arg_count() {
        let result = eval_max(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_max_wrong_type() {
        let result = eval_max(&[Value::Bool(true), Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_floor_wrong_type() {
        let result = eval_floor(&[Value::from_string("3.5".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_ceil_wrong_type() {
        let result = eval_ceil(&[Value::from_string("3.5".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_round_wrong_type() {
        let result = eval_round(&[Value::Bool(true)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sin_wrong_type() {
        let result = eval_sin(&[Value::from_string("0".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_cos_wrong_type() {
        let result = eval_cos(&[Value::from_string("0".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_tan_wrong_type() {
        let result = eval_tan(&[Value::Nil]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_log_wrong_type() {
        let result = eval_log(&[Value::from_string("e".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_log10_wrong_type() {
        let result = eval_log10(&[Value::Bool(true)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_exp_wrong_type() {
        let result = eval_exp(&[Value::from_string("0".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_random_wrong_arg_count() {
        let result = eval_random(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_len_wrong_arg_count() {
        let result = eval_len(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_len_wrong_type() {
        let result = eval_len(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_len_dataframe_empty() {
        let df = Value::DataFrame { columns: vec![] };
        let result = eval_len(&[df]).expect("should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_len_dataframe_with_data() {
        let col = crate::runtime::DataFrameColumn {
            name: "col1".to_string(),
            values: vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        };
        let df = Value::DataFrame { columns: vec![col] };
        let result = eval_len(&[df]).expect("should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_range_wrong_arg_count() {
        let result = eval_range(&[]);
        assert!(result.is_err());

        let result = eval_range(&[
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_range_wrong_type() {
        let result = eval_range(&[Value::from_string("5".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_range_zero_step() {
        let result = eval_range(&[Value::Integer(0), Value::Integer(10), Value::Integer(0)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_type_wrong_arg_count() {
        let result = eval_type(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_type_of_wrong_arg_count() {
        let result = eval_type_of(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_is_nil_wrong_arg_count() {
        let result = eval_is_nil(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_reverse_wrong_arg_count() {
        let result = eval_reverse(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_reverse_wrong_type() {
        let result = eval_reverse(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_push_wrong_arg_count() {
        let result = eval_push(&[Value::Array(Arc::from(vec![]))]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_push_wrong_type() {
        let result = eval_push(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pop_wrong_arg_count() {
        let result = eval_pop(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pop_wrong_type() {
        let result = eval_pop(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_pop_empty_array() {
        let result = eval_pop(&[Value::Array(Arc::from(vec![]))]).expect("should succeed");
        assert_eq!(result, Value::nil());
    }

    #[test]
    fn test_eval_sort_wrong_arg_count() {
        let result = eval_sort(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sort_wrong_type() {
        let result = eval_sort(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_zip_wrong_arg_count() {
        let result = eval_zip(&[Value::Array(Arc::from(vec![]))]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_zip_wrong_type() {
        let result = eval_zip(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_enumerate_wrong_arg_count() {
        let result = eval_enumerate(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_enumerate_wrong_type() {
        let result = eval_enumerate(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_eq_wrong_arg_count() {
        let result = eval_assert_eq(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_assert_wrong_arg_count() {
        let result = eval_assert(&[]);
        assert!(result.is_err());
    }

    // --- Type conversion error cases ---

    #[test]
    fn test_eval_str_wrong_arg_count() {
        let result = eval_str(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_to_string_wrong_arg_count() {
        let result = eval_to_string(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_int_wrong_arg_count() {
        let result = eval_int(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_int_invalid_string() {
        let result = eval_int(&[Value::from_string("not a number".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_int_already_int() {
        let result = eval_int(&[Value::Integer(42)]).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_float_wrong_arg_count() {
        let result = eval_float(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_float_invalid_string() {
        let result = eval_float(&[Value::from_string("not a number".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_float_already_float() {
        let result = eval_float(&[Value::Float(3.14)]).expect("should succeed");
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_bool_wrong_arg_count() {
        let result = eval_bool(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_bool_already_bool() {
        let result = eval_bool(&[Value::Bool(true)]).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_bool_float_zero() {
        let result = eval_bool(&[Value::Float(0.0)]).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_bool_float_nonzero() {
        let result = eval_bool(&[Value::Float(1.5)]).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_parse_int_wrong_arg_count() {
        let result = eval_parse_int(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_int_wrong_type() {
        let result = eval_parse_int(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_int_invalid_string() {
        let result = eval_parse_int(&[Value::from_string("not a number".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_float_wrong_arg_count() {
        let result = eval_parse_float(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_float_wrong_type() {
        let result = eval_parse_float(&[Value::Float(3.14)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_parse_float_invalid_string() {
        let result = eval_parse_float(&[Value::from_string("not a number".to_string())]);
        assert!(result.is_err());
    }

    // --- Time function error cases ---

    #[test]
    fn test_eval_sleep_wrong_arg_count() {
        let result = eval_sleep(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sleep_wrong_type() {
        let result = eval_sleep(&[Value::from_string("100".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_sleep_float() {
        let result = eval_sleep(&[Value::Float(1.0)]).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_timestamp_wrong_arg_count() {
        let result = eval_timestamp(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_chrono_utc_now() {
        let result = eval_chrono_utc_now(&[]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains("T"), "Should be RFC3339 format");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_eval_chrono_utc_now_wrong_arg_count() {
        let result = eval_chrono_utc_now(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    // --- DataFrame function tests ---

    #[test]
    fn test_eval_dataframe_new() {
        let result = eval_dataframe_new(&[]).expect("should succeed");
        if let Value::Object(obj) = result {
            assert!(obj.get("__type").is_some());
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_eval_dataframe_new_wrong_arg_count() {
        let result = eval_dataframe_new(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_dataframe_from_csv_string() {
        let csv = "name,age\nAlice,30\nBob,25";
        let result = eval_dataframe_from_csv_string(&[Value::from_string(csv.to_string())])
            .expect("should succeed");
        if let Value::DataFrame { columns } = result {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].name, "name");
            assert_eq!(columns[1].name, "age");
        } else {
            panic!("Expected DataFrame result");
        }
    }

    #[test]
    fn test_eval_dataframe_from_csv_string_empty() {
        let result = eval_dataframe_from_csv_string(&[Value::from_string("".to_string())])
            .expect("should succeed");
        if let Value::DataFrame { columns } = result {
            assert!(columns.is_empty());
        }
    }

    #[test]
    fn test_eval_dataframe_from_csv_string_wrong_type() {
        let result = eval_dataframe_from_csv_string(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_dataframe_from_json() {
        let json = r#"[{"name": "Alice", "age": 30}]"#;
        let result = eval_dataframe_from_json(&[Value::from_string(json.to_string())])
            .expect("should succeed");
        if let Value::DataFrame { columns } = result {
            assert!(!columns.is_empty());
        } else {
            panic!("Expected DataFrame result");
        }
    }

    #[test]
    fn test_eval_dataframe_from_json_empty() {
        let result = eval_dataframe_from_json(&[Value::from_string("[]".to_string())])
            .expect("should succeed");
        if let Value::DataFrame { columns } = result {
            assert!(columns.is_empty());
        }
    }

    #[test]
    fn test_eval_dataframe_from_json_invalid() {
        let result = eval_dataframe_from_json(&[Value::from_string("not json".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_dataframe_from_json_wrong_type() {
        let result = eval_dataframe_from_json(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    // --- Environment function tests ---

    #[test]
    fn test_eval_env_args() {
        let result = eval_env_args(&[]).expect("should succeed");
        if let Value::Array(_) = result {
            // Success - args returned as array
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_env_args_wrong_arg_count() {
        let result = eval_env_args(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_var_not_found() {
        let result = eval_env_var(&[Value::from_string("__NONEXISTENT_VAR_12345__".to_string())])
            .expect("should succeed");
        if let Value::EnumVariant { variant_name, .. } = result {
            assert_eq!(variant_name, "Err");
        } else {
            panic!("Expected EnumVariant result");
        }
    }

    #[test]
    fn test_eval_env_var_wrong_type() {
        let result = eval_env_var(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_set_var_and_get() {
        let key = "__TEST_ENV_VAR_RUCHY__";
        let value = "test_value";

        // Set the variable
        eval_env_set_var(&[
            Value::from_string(key.to_string()),
            Value::from_string(value.to_string()),
        ])
        .expect("set should succeed");

        // Get it back
        let result =
            eval_env_var(&[Value::from_string(key.to_string())]).expect("get should succeed");
        if let Value::EnumVariant {
            variant_name, data, ..
        } = result
        {
            assert_eq!(variant_name, "Ok");
            if let Some(d) = data {
                assert_eq!(d[0], Value::from_string(value.to_string()));
            }
        }

        // Clean up
        eval_env_remove_var(&[Value::from_string(key.to_string())]).expect("remove should succeed");
    }

    #[test]
    fn test_eval_env_set_var_wrong_type() {
        let result = eval_env_set_var(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_remove_var_wrong_type() {
        let result = eval_env_remove_var(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_vars() {
        let result = eval_env_vars(&[]).expect("should succeed");
        if let Value::Object(_) = result {
            // Success
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_eval_env_vars_wrong_arg_count() {
        let result = eval_env_vars(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_current_dir() {
        let result = eval_env_current_dir(&[]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(!s.is_empty());
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_eval_env_current_dir_wrong_arg_count() {
        let result = eval_env_current_dir(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_env_temp_dir() {
        let result = eval_env_temp_dir(&[]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(!s.is_empty());
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_eval_env_temp_dir_wrong_arg_count() {
        let result = eval_env_temp_dir(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    // --- Path function tests ---

    #[test]
    fn test_eval_path_join() {
        let result = eval_path_join(&[
            Value::from_string("/home".to_string()),
            Value::from_string("user".to_string()),
        ])
        .expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains("home") && s.contains("user"));
        }
    }

    #[test]
    fn test_eval_path_join_wrong_type() {
        let result = eval_path_join(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_path_parent() {
        let result = eval_path_parent(&[Value::from_string("/home/user/file.txt".to_string())])
            .expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains("user"));
        }
    }

    #[test]
    fn test_eval_path_parent_root() {
        let result =
            eval_path_parent(&[Value::from_string("/".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_path_parent_wrong_type() {
        let result = eval_path_parent(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_path_file_name() {
        let result = eval_path_file_name(&[Value::from_string("/home/user/file.txt".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::from_string("file.txt".to_string()));
    }

    #[test]
    fn test_eval_path_file_name_no_file() {
        let result =
            eval_path_file_name(&[Value::from_string("/".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_path_file_stem() {
        let result = eval_path_file_stem(&[Value::from_string("/home/user/file.txt".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::from_string("file".to_string()));
    }

    #[test]
    fn test_eval_path_extension() {
        let result = eval_path_extension(&[Value::from_string("/home/user/file.txt".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::from_string("txt".to_string()));
    }

    #[test]
    fn test_eval_path_extension_none() {
        let result = eval_path_extension(&[Value::from_string("/home/user/file".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_path_is_absolute() {
        let result = eval_path_is_absolute(&[Value::from_string("/home/user".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(true));

        let result = eval_path_is_absolute(&[Value::from_string("relative/path".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_path_is_relative() {
        let result = eval_path_is_relative(&[Value::from_string("relative/path".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(true));

        let result = eval_path_is_relative(&[Value::from_string("/absolute/path".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_path_with_extension() {
        let result = eval_path_with_extension(&[
            Value::from_string("/home/file.txt".to_string()),
            Value::from_string("md".to_string()),
        ])
        .expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.ends_with(".md"));
        }
    }

    #[test]
    fn test_eval_path_with_file_name() {
        let result = eval_path_with_file_name(&[
            Value::from_string("/home/old.txt".to_string()),
            Value::from_string("new.txt".to_string()),
        ])
        .expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.ends_with("new.txt"));
        }
    }

    #[test]
    fn test_eval_path_components() {
        let result = eval_path_components(&[Value::from_string("/home/user/file".to_string())])
            .expect("should succeed");
        if let Value::Array(arr) = result {
            assert!(arr.len() >= 3);
        }
    }

    #[test]
    fn test_eval_path_normalize() {
        let result =
            eval_path_normalize(&[Value::from_string("/home/../home/user/./file".to_string())])
                .expect("should succeed");
        if let Value::String(s) = result {
            assert!(!s.contains(".."));
            assert!(!s.contains("./"));
        }
    }

    #[test]
    fn test_eval_path_join_many() {
        let components = Value::Array(Arc::from(vec![
            Value::from_string("/home".to_string()),
            Value::from_string("user".to_string()),
            Value::from_string("file.txt".to_string()),
        ]));
        let result = eval_path_join_many(&[components]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains("home") && s.contains("user") && s.contains("file.txt"));
        }
    }

    #[test]
    fn test_eval_path_join_many_wrong_type() {
        let result = eval_path_join_many(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_path_join_many_invalid_component() {
        let components = Value::Array(Arc::from(vec![
            Value::from_string("/home".to_string()),
            Value::Integer(42), // Invalid!
        ]));
        let result = eval_path_join_many(&[components]);
        assert!(result.is_err());
    }

    // --- JSON function tests ---

    #[test]
    fn test_eval_json_parse_object() {
        let result = eval_json_parse(&[Value::from_string(r#"{"key": "value"}"#.to_string())])
            .expect("should succeed");
        if let Value::Object(obj) = result {
            assert!(obj.get("key").is_some());
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_eval_json_parse_array() {
        let result = eval_json_parse(&[Value::from_string("[1, 2, 3]".to_string())])
            .expect("should succeed");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_json_parse_primitives() {
        let result =
            eval_json_parse(&[Value::from_string("null".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Nil);

        let result =
            eval_json_parse(&[Value::from_string("true".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Bool(true));

        let result =
            eval_json_parse(&[Value::from_string("42".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Integer(42));

        let result =
            eval_json_parse(&[Value::from_string("3.14".to_string())]).expect("should succeed");
        if let Value::Float(f) = result {
            assert!((f - 3.14).abs() < 0.001);
        }

        let result = eval_json_parse(&[Value::from_string(r#""hello""#.to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_json_parse_invalid() {
        let result = eval_json_parse(&[Value::from_string("not valid json".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_json_parse_wrong_type() {
        let result = eval_json_parse(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_json_stringify() {
        let obj = Value::Object(Arc::new({
            let mut map = HashMap::new();
            map.insert("key".to_string(), Value::from_string("value".to_string()));
            map
        }));
        let result = eval_json_stringify(&[obj]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains("key") && s.contains("value"));
        }
    }

    #[test]
    fn test_eval_json_stringify_primitives() {
        let result = eval_json_stringify(&[Value::Integer(42)]).expect("should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));

        let result = eval_json_stringify(&[Value::Bool(true)]).expect("should succeed");
        assert_eq!(result, Value::from_string("true".to_string()));

        let result = eval_json_stringify(&[Value::Nil]).expect("should succeed");
        assert_eq!(result, Value::from_string("null".to_string()));
    }

    #[test]
    fn test_eval_json_pretty() {
        let obj = Value::Object(Arc::new({
            let mut map = HashMap::new();
            map.insert("key".to_string(), Value::from_string("value".to_string()));
            map
        }));
        let result = eval_json_pretty(&[obj]).expect("should succeed");
        if let Value::String(s) = result {
            assert!(s.contains('\n'), "Pretty print should have newlines");
        }
    }

    #[test]
    fn test_eval_json_validate_valid() {
        let result = eval_json_validate(&[Value::from_string(r#"{"key": "value"}"#.to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_json_validate_invalid() {
        let result = eval_json_validate(&[Value::from_string("not json".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_json_validate_wrong_type() {
        let result = eval_json_validate(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_json_type() {
        let result =
            eval_json_type(&[Value::from_string("null".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("null".to_string()));

        let result =
            eval_json_type(&[Value::from_string("true".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("boolean".to_string()));

        let result =
            eval_json_type(&[Value::from_string("42".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("number".to_string()));

        let result = eval_json_type(&[Value::from_string(r#""hello""#.to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::from_string("string".to_string()));

        let result =
            eval_json_type(&[Value::from_string("[]".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("array".to_string()));

        let result =
            eval_json_type(&[Value::from_string("{}".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("object".to_string()));
    }

    #[test]
    fn test_eval_json_type_invalid() {
        let result = eval_json_type(&[Value::from_string("not json".to_string())]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_json_merge() {
        let obj1 = Value::Object(Arc::new({
            let mut map = HashMap::new();
            map.insert("a".to_string(), Value::Integer(1));
            map
        }));
        let obj2 = Value::Object(Arc::new({
            let mut map = HashMap::new();
            map.insert("b".to_string(), Value::Integer(2));
            map
        }));
        let result = eval_json_merge(&[obj1, obj2]).expect("should succeed");
        if let Value::Object(obj) = result {
            assert!(obj.get("a").is_some());
            assert!(obj.get("b").is_some());
        } else {
            panic!("Expected object result");
        }
    }

    #[test]
    fn test_eval_json_get() {
        let obj = Value::Object(Arc::new({
            let mut map = HashMap::new();
            let mut nested = HashMap::new();
            nested.insert("inner".to_string(), Value::Integer(42));
            map.insert("outer".to_string(), Value::Object(Arc::new(nested)));
            map
        }));
        let result = eval_json_get(&[obj, Value::from_string("outer.inner".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_json_get_not_found() {
        let obj = Value::Object(Arc::new(HashMap::new()));
        let result = eval_json_get(&[obj, Value::from_string("nonexistent".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    // --- String function tests ---

    #[test]
    fn test_eval_string_new() {
        let result = eval_string_new(&[]).expect("should succeed");
        assert_eq!(result, Value::from_string("".to_string()));
    }

    #[test]
    fn test_eval_string_new_wrong_arg_count() {
        let result = eval_string_new(&[Value::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_string_from() {
        let result = eval_string_from(&[Value::Integer(42)]).expect("should succeed");
        assert_eq!(result, Value::from_string("42".to_string()));

        let result =
            eval_string_from(&[Value::from_string("hello".to_string())]).expect("should succeed");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_string_from_wrong_arg_count() {
        let result = eval_string_from(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_string_from_utf8_valid() {
        let bytes = Value::Array(Arc::from(vec![
            Value::Byte(72),  // H
            Value::Byte(105), // i
        ]));
        let result = eval_string_from_utf8(&[bytes]).expect("should succeed");
        if let Value::EnumVariant {
            variant_name, data, ..
        } = result
        {
            assert_eq!(variant_name, "Ok");
            if let Some(d) = data {
                assert_eq!(d[0], Value::from_string("Hi".to_string()));
            }
        } else {
            panic!("Expected EnumVariant result");
        }
    }

    #[test]
    fn test_eval_string_from_utf8_invalid() {
        let bytes = Value::Array(Arc::from(vec![Value::Byte(0xFF), Value::Byte(0xFE)]));
        let result = eval_string_from_utf8(&[bytes]).expect("should succeed");
        if let Value::EnumVariant { variant_name, .. } = result {
            assert_eq!(variant_name, "Err");
        } else {
            panic!("Expected EnumVariant result");
        }
    }

    #[test]
    fn test_eval_string_from_utf8_wrong_type() {
        let result = eval_string_from_utf8(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_string_from_utf8_non_byte_array() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(72)])); // Not Byte type
        let result = eval_string_from_utf8(&[arr]);
        assert!(result.is_err());
    }

    // --- File system function tests ---

    #[test]
    fn test_eval_fs_exists_true() {
        let result =
            eval_fs_exists(&[Value::from_string(".".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_fs_exists_false() {
        let result =
            eval_fs_exists(&[Value::from_string("__nonexistent_path_12345__".to_string())])
                .expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_fs_exists_wrong_type() {
        let result = eval_fs_exists(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_is_file() {
        let result = eval_fs_is_file(&[Value::from_string("Cargo.toml".to_string())])
            .expect("should succeed");
        assert_eq!(result, Value::Bool(true));

        let result =
            eval_fs_is_file(&[Value::from_string(".".to_string())]).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_fs_is_file_wrong_type() {
        let result = eval_fs_is_file(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_read_nonexistent() {
        let result = eval_fs_read(&[Value::from_string("__nonexistent_file__".to_string())])
            .expect("should succeed");
        if let Value::EnumVariant { variant_name, .. } = result {
            assert_eq!(variant_name, "Err");
        }
    }

    #[test]
    fn test_eval_fs_read_wrong_type() {
        let result = eval_fs_read(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_write_wrong_type() {
        let result = eval_fs_write(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_create_dir_wrong_type() {
        let result = eval_fs_create_dir(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_remove_file_nonexistent() {
        let result = eval_fs_remove_file(&[Value::from_string("__nonexistent_file__".to_string())])
            .expect("should succeed");
        // Should be Ok (idempotent)
        if let Value::EnumVariant { variant_name, .. } = result {
            assert_eq!(variant_name, "Ok");
        }
    }

    #[test]
    fn test_eval_fs_remove_file_wrong_type() {
        let result = eval_fs_remove_file(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_remove_dir_wrong_type() {
        let result = eval_fs_remove_dir(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_copy_wrong_type() {
        let result = eval_fs_copy(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_rename_wrong_type() {
        let result = eval_fs_rename(&[Value::Integer(1), Value::Integer(2)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_metadata_wrong_type() {
        let result = eval_fs_metadata(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_read_dir_wrong_type() {
        let result = eval_fs_read_dir(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_fs_canonicalize_wrong_type() {
        let result = eval_fs_canonicalize(&[Value::Integer(42)]);
        assert!(result.is_err());
    }

    // --- Dispatcher coverage tests ---

    #[test]
    fn test_try_eval_dataframe_function_new() {
        let result =
            try_eval_dataframe_function("__builtin_dataframe_new__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_dataframe_function_from_csv() {
        let result = try_eval_dataframe_function(
            "__builtin_dataframe_from_csv_string__",
            &[Value::from_string("a,b\n1,2".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_dataframe_function_from_json() {
        let result = try_eval_dataframe_function(
            "__builtin_dataframe_from_json__",
            &[Value::from_string("[]".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_dataframe_function_unknown() {
        let result = try_eval_dataframe_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_environment_function_args() {
        let result =
            try_eval_environment_function("__builtin_env_args__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_environment_function_vars() {
        let result =
            try_eval_environment_function("__builtin_env_vars__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_environment_function_current_dir() {
        let result = try_eval_environment_function("__builtin_env_current_dir__", &[])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_environment_function_temp_dir() {
        let result =
            try_eval_environment_function("__builtin_env_temp_dir__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_environment_function_unknown() {
        let result = try_eval_environment_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_fs_function_read() {
        let result = try_eval_fs_function(
            "__builtin_fs_read__",
            &[Value::from_string("__nonexistent__".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_fs_function_exists() {
        let result = try_eval_fs_function(
            "__builtin_fs_exists__",
            &[Value::from_string(".".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_fs_function_is_file() {
        let result = try_eval_fs_function(
            "__builtin_fs_is_file__",
            &[Value::from_string("Cargo.toml".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_fs_function_unknown() {
        let result = try_eval_fs_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_path_function_join() {
        let result = try_eval_path_function(
            "__builtin_path_join__",
            &[
                Value::from_string("/home".to_string()),
                Value::from_string("user".to_string()),
            ],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_parent() {
        let result = try_eval_path_function(
            "__builtin_path_parent__",
            &[Value::from_string("/home/user".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_file_name() {
        let result = try_eval_path_function(
            "__builtin_path_file_name__",
            &[Value::from_string("/home/file.txt".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_extension() {
        let result = try_eval_path_function(
            "__builtin_path_extension__",
            &[Value::from_string("file.txt".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_is_absolute() {
        let result = try_eval_path_function(
            "__builtin_path_is_absolute__",
            &[Value::from_string("/home".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_components() {
        let result = try_eval_path_function(
            "__builtin_path_components__",
            &[Value::from_string("/home/user".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_normalize() {
        let result = try_eval_path_function(
            "__builtin_path_normalize__",
            &[Value::from_string("/home/../user".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_path_function_unknown() {
        let result = try_eval_path_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_json_function_parse() {
        let result = try_eval_json_function(
            "__builtin_json_parse__",
            &[Value::from_string("{}".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_json_function_stringify() {
        let result = try_eval_json_function("__builtin_json_stringify__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_json_function_pretty() {
        let result = try_eval_json_function("__builtin_json_pretty__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_json_function_validate() {
        let result = try_eval_json_function(
            "__builtin_json_validate__",
            &[Value::from_string("{}".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_json_function_type() {
        let result = try_eval_json_function(
            "__builtin_json_type__",
            &[Value::from_string("42".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_json_function_unknown() {
        let result = try_eval_json_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_string_function_new() {
        let result =
            try_eval_string_function("__builtin_String_new__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_string_function_from() {
        let result = try_eval_string_function("__builtin_String_from__", &[Value::Integer(42)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_eval_string_function_unknown() {
        let result = try_eval_string_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    #[test]
    fn test_try_eval_file_function_file_open() {
        // This will fail because the file doesn't exist, but the dispatcher should work
        let result = try_eval_file_function(
            "File_open",
            &[Value::from_string("__nonexistent__".to_string())],
        );
        assert!(result.is_err()); // Error because file doesn't exist
    }

    #[test]
    fn test_try_eval_file_function_unknown() {
        let result = try_eval_file_function("unknown", &[]).expect("should succeed");
        assert!(result.is_none());
    }

    // --- Edge cases for format functions ---

    #[test]
    fn test_format_println_output_interpolation() {
        let output = format_println_output(&[
            Value::from_string("Hello {}!".to_string()),
            Value::from_string("World".to_string()),
        ]);
        assert_eq!(output, "Hello World!\n");
    }

    #[test]
    fn test_format_println_output_multiple_placeholders() {
        let output = format_println_output(&[
            Value::from_string("{} + {} = {}".to_string()),
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        assert_eq!(output, "1 + 2 = 3\n");
    }

    #[test]
    fn test_format_println_output_no_placeholders() {
        let output = format_println_output(&[
            Value::from_string("Hello".to_string()),
            Value::from_string("World".to_string()),
        ]);
        assert_eq!(output, "Hello World\n");
    }

    #[test]
    fn test_format_println_output_non_string_first() {
        let output = format_println_output(&[Value::Integer(42), Value::Integer(43)]);
        assert_eq!(output, "42 43\n");
    }

    #[test]
    fn test_format_value_for_println() {
        assert_eq!(
            format_value_for_println(&Value::from_string("hello".to_string())),
            "hello"
        );
        assert_eq!(format_value_for_println(&Value::Integer(42)), "42");
        assert_eq!(format_value_for_println(&Value::Float(3.14)), "3.14");
        assert_eq!(format_value_for_println(&Value::Bool(true)), "true");
        assert_eq!(format_value_for_println(&Value::Nil), "nil");
    }

    // --- Helper function coverage ---

    #[test]
    fn test_infer_value_type_integer() {
        let result = infer_value_type("42");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_infer_value_type_float() {
        let result = infer_value_type("3.14");
        if let Value::Float(f) = result {
            assert!((f - 3.14).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_infer_value_type_string() {
        let result = infer_value_type("hello");
        assert_eq!(result, Value::from_string("hello".to_string()));
    }

    // --- More builtin function dispatcher tests ---

    #[test]
    fn test_eval_builtin_function_sort() {
        let arr = Value::Array(Arc::from(vec![
            Value::Integer(3),
            Value::Integer(1),
            Value::Integer(2),
        ]));
        let result = eval_builtin_function("__builtin_sort__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_zip() {
        let a = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let b = Value::Array(Arc::from(vec![Value::Integer(2)]));
        let result = eval_builtin_function("__builtin_zip__", &[a, b]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_enumerate() {
        let arr = Value::Array(Arc::from(vec![Value::Integer(1)]));
        let result =
            eval_builtin_function("__builtin_enumerate__", &[arr]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_sin() {
        let result =
            eval_builtin_function("__builtin_sin__", &[Value::Float(0.0)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_cos() {
        let result =
            eval_builtin_function("__builtin_cos__", &[Value::Float(0.0)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_tan() {
        let result =
            eval_builtin_function("__builtin_tan__", &[Value::Float(0.0)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_log() {
        let result =
            eval_builtin_function("__builtin_log__", &[Value::Float(1.0)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_log10() {
        let result = eval_builtin_function("__builtin_log10__", &[Value::Float(10.0)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_exp() {
        let result =
            eval_builtin_function("__builtin_exp__", &[Value::Float(0.0)]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_random() {
        let result = eval_builtin_function("__builtin_random__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_timestamp() {
        let result = eval_builtin_function("__builtin_timestamp__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_chrono_utc_now() {
        let result =
            eval_builtin_function("__builtin_chrono_utc_now__", &[]).expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_assert_eq() {
        let result = eval_builtin_function(
            "__builtin_assert_eq__",
            &[Value::Integer(1), Value::Integer(1)],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_assert() {
        let result = eval_builtin_function("__builtin_assert__", &[Value::Bool(true)])
            .expect("should succeed");
        assert!(result.is_some());
    }

    #[test]
    fn test_eval_builtin_function_pow() {
        let result =
            eval_builtin_function("__builtin_pow__", &[Value::Integer(2), Value::Integer(3)])
                .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(8)));
    }

    #[test]
    fn test_eval_builtin_function_parse_int() {
        let result = eval_builtin_function(
            "__builtin_parse_int__",
            &[Value::from_string("42".to_string())],
        )
        .expect("should succeed");
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_eval_builtin_function_parse_float() {
        let result = eval_builtin_function(
            "__builtin_parse_float__",
            &[Value::from_string("3.14".to_string())],
        )
        .expect("should succeed");
        assert!(result.is_some());
    }
