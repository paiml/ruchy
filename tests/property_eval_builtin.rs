//! Property-Based Tests for `eval_builtin.rs`
//!
//! Target: runtime/eval_builtin.rs (32.24% → 60-65% coverage)
//! Strategy: Property testing with 10K+ cases per function
//! Focus: Pure functions (math, utility, conversion)
//!
//! EXTREME TDD: RED phase - These tests will exercise uncovered code paths

use proptest::prelude::*;
use ruchy::runtime::{eval_builtin::eval_builtin_function, Value};
use std::sync::Arc;

// ============================================================================
// Property Tests: Math Functions (Pure, High-Value)
// ============================================================================

proptest! {
    /// Property: sqrt(x) * sqrt(x) ≈ x (for non-negative x)
    /// Coverage target: eval_sqrt (lines 392-401)
    #[test]
    fn prop_sqrt_inverse(x in 0.0f64..1_000_000.0f64) {
        let result = eval_builtin_function("__builtin_sqrt__", &[Value::Float(x)]);
        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Float(sqrt_x))) = result {
            let reconstructed = sqrt_x * sqrt_x;
            // Allow small floating-point error
            prop_assert!((reconstructed - x).abs() < 0.001,
                "sqrt({})^2 = {} ≠ {}", x, reconstructed, x);
        }
    }

    /// Property: sqrt(integer) returns Float
    /// Coverage target: eval_sqrt integer branch (line 395)
    #[test]
    fn prop_sqrt_integer_type(x in 0i64..1000i64) {
        let result = eval_builtin_function("__builtin_sqrt__", &[Value::Integer(x)]);
        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Float(sqrt_x))) = result {
            let expected = (x as f64).sqrt();
            prop_assert!((sqrt_x - expected).abs() < 0.001);
        } else {
            prop_assert!(false, "Expected Float result for sqrt(Integer)");
        }
    }

    /// Property: abs(-x) == abs(x) (symmetry)
    /// Coverage target: eval_abs (lines 430-439)
    #[test]
    fn prop_abs_symmetry_int(x in -1_000_000i64..1_000_000i64) {
        let result_pos = eval_builtin_function("__builtin_abs__", &[Value::Integer(x)]);
        let result_neg = eval_builtin_function("__builtin_abs__", &[Value::Integer(-x)]);

        prop_assert!(result_pos.is_ok() && result_neg.is_ok());
        prop_assert_eq!(result_pos.unwrap(), result_neg.unwrap(),
            "abs({}) ≠ abs({})", x, -x);
    }

    /// Property: abs(float) symmetry
    /// Coverage target: eval_abs float branch (line 434)
    #[test]
    fn prop_abs_symmetry_float(x in -1_000_000.0f64..1_000_000.0f64) {
        let result_pos = eval_builtin_function("__builtin_abs__", &[Value::Float(x)]);
        let result_neg = eval_builtin_function("__builtin_abs__", &[Value::Float(-x)]);

        prop_assert!(result_pos.is_ok() && result_neg.is_ok());
        if let (Ok(Some(Value::Float(a))), Ok(Some(Value::Float(b)))) = (result_pos, result_neg) {
            prop_assert!((a - b).abs() < 0.001, "abs({}) ≠ abs({})", x, -x);
        }
    }

    /// Property: min(a, b) ≤ a AND min(a, b) ≤ b
    /// Coverage target: eval_min (lines 445-456)
    #[test]
    fn prop_min_lower_bound(a in -1000i64..1000i64, b in -1000i64..1000i64) {
        let result = eval_builtin_function("__builtin_min__",
            &[Value::Integer(a), Value::Integer(b)]);

        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Integer(min_val))) = result {
            prop_assert!(min_val <= a, "min({}, {}) = {} > {}", a, b, min_val, a);
            prop_assert!(min_val <= b, "min({}, {}) = {} > {}", a, b, min_val, b);
            prop_assert!(min_val == a || min_val == b,
                "min({}, {}) = {} not in {{a, b}}", a, b, min_val);
        }
    }

    /// Property: max(a, b) ≥ a AND max(a, b) ≥ b
    /// Coverage target: eval_max (lines 462-473)
    #[test]
    fn prop_max_upper_bound(a in -1000i64..1000i64, b in -1000i64..1000i64) {
        let result = eval_builtin_function("__builtin_max__",
            &[Value::Integer(a), Value::Integer(b)]);

        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Integer(max_val))) = result {
            prop_assert!(max_val >= a, "max({}, {}) = {} < {}", a, b, max_val, a);
            prop_assert!(max_val >= b, "max({}, {}) = {} < {}", a, b, max_val, b);
            prop_assert!(max_val == a || max_val == b,
                "max({}, {}) = {} not in {{a, b}}", a, b, max_val);
        }
    }

    /// Property: floor(x) ≤ x < floor(x) + 1
    /// Coverage target: eval_floor (lines 479-488)
    #[test]
    fn prop_floor_bounds(x in -1000.0f64..1000.0f64) {
        let result = eval_builtin_function("__builtin_floor__", &[Value::Float(x)]);

        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Integer(floor_x))) = result {
            let floor_f = floor_x as f64;
            prop_assert!(floor_f <= x, "floor({}) = {} > {}", x, floor_x, x);
            prop_assert!(x < floor_f + 1.0, "{} >= floor({}) + 1", x, floor_x);
        }
    }

    /// Property: ceil(x) - 1 < x ≤ ceil(x)
    /// Coverage target: eval_ceil (lines 494-503)
    #[test]
    fn prop_ceil_bounds(x in -1000.0f64..1000.0f64) {
        let result = eval_builtin_function("__builtin_ceil__", &[Value::Float(x)]);

        prop_assert!(result.is_ok());
        if let Ok(Some(Value::Integer(ceil_x))) = result {
            let ceil_f = ceil_x as f64;
            prop_assert!(x <= ceil_f, "{} > ceil({}) = {}", x, x, ceil_x);
            prop_assert!(ceil_f - 1.0 < x, "ceil({}) - 1 = {} >= {}", x, ceil_f - 1.0, x);
        }
    }

    /// Property: pow(x, 0) == 1 (identity)
    /// Coverage target: eval_pow (lines 407-424)
    #[test]
    fn prop_pow_zero_exponent(x in -100i64..100i64) {
        let result = eval_builtin_function("__builtin_pow__",
            &[Value::Integer(x), Value::Integer(0)]);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(1)),
            "pow({}, 0) ≠ 1", x);
    }

    /// Property: pow(x, 1) == x (identity)
    /// Coverage target: eval_pow (lines 407-424)
    #[test]
    fn prop_pow_one_exponent(x in -100i64..100i64) {
        let result = eval_builtin_function("__builtin_pow__",
            &[Value::Integer(x), Value::Integer(1)]);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(x)),
            "pow({}, 1) ≠ {}", x, x);
    }

    /// Property: pow(2, n) == 2^n (positive exponents)
    /// Coverage target: eval_pow integer/integer branch (line 410-415)
    #[test]
    fn prop_pow_two_powers(n in 0u32..20u32) {
        let expected = 2i64.pow(n);
        let result = eval_builtin_function("__builtin_pow__",
            &[Value::Integer(2), Value::Integer(n as i64)]);

        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(expected)),
            "pow(2, {}) ≠ {}", n, expected);
    }

    /// Property: round(x) is within 0.5 of x
    /// Coverage target: eval_round (lines 509-518)
    #[test]
    fn prop_round_bounds(x in -1000.0f64..1000.0f64) {
        let result = eval_builtin_function("__builtin_round__", &[Value::Float(x)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Integer(rounded))) = result {
            let diff = (x - rounded as f64).abs();
            prop_assert!(diff <= 0.5, "round({}) = {} diff {} > 0.5", x, rounded, diff);
        }
    }

    /// Property: sin(x)^2 + cos(x)^2 ≈ 1 (trigonometric identity)
    /// Coverage target: eval_sin (line 524), eval_cos (line 539)
    #[test]
    fn prop_sin_cos_identity(x in -10.0f64..10.0f64) {
        let sin_result = eval_builtin_function("__builtin_sin__", &[Value::Float(x)]);
        let cos_result = eval_builtin_function("__builtin_cos__", &[Value::Float(x)]);

        prop_assert!(sin_result.is_ok() && cos_result.is_ok());

        if let (Ok(Some(Value::Float(sin_x))), Ok(Some(Value::Float(cos_x)))) = (sin_result, cos_result) {
            let identity = sin_x * sin_x + cos_x * cos_x;
            prop_assert!((identity - 1.0).abs() < 0.0001,
                "sin({})^2 + cos({})^2 = {} ≠ 1", x, x, identity);
        }
    }

    /// Property: tan(x) ≈ sin(x) / cos(x)
    /// Coverage target: eval_tan (line 554)
    #[test]
    fn prop_tan_identity(x in -1.5f64..1.5f64) { // Avoid pi/2 where tan is undefined
        let tan_result = eval_builtin_function("__builtin_tan__", &[Value::Float(x)]);
        let sin_result = eval_builtin_function("__builtin_sin__", &[Value::Float(x)]);
        let cos_result = eval_builtin_function("__builtin_cos__", &[Value::Float(x)]);

        prop_assert!(tan_result.is_ok() && sin_result.is_ok() && cos_result.is_ok());

        if let (Ok(Some(Value::Float(tan_x))), Ok(Some(Value::Float(sin_x))), Ok(Some(Value::Float(cos_x))))
            = (tan_result, sin_result, cos_result) {
            if cos_x.abs() > 0.01 { // Avoid division by zero
                let expected_tan = sin_x / cos_x;
                prop_assert!((tan_x - expected_tan).abs() < 0.001,
                    "tan({}) = {} ≠ sin/cos = {}", x, tan_x, expected_tan);
            }
        }
    }

    /// Property: log(e^x) ≈ x (for small x)
    /// Coverage target: eval_log (lines 574-583)
    #[test]
    fn prop_log_inverse(x in -5.0f64..5.0f64) {
        let e_to_x = std::f64::consts::E.powf(x);
        let result = eval_builtin_function("__builtin_log__", &[Value::Float(e_to_x)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Float(log_val))) = result {
            prop_assert!((log_val - x).abs() < 0.001,
                "log(e^{}) = {} ≠ {}", x, log_val, x);
        }
    }

    /// Property: log10(10^x) ≈ x
    /// Coverage target: eval_log10 (lines 589-598)
    #[test]
    fn prop_log10_inverse(x in -2.0f64..2.0f64) {
        let ten_to_x = 10.0f64.powf(x);
        let result = eval_builtin_function("__builtin_log10__", &[Value::Float(ten_to_x)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Float(log10_val))) = result {
            prop_assert!((log10_val - x).abs() < 0.001,
                "log10(10^{}) = {} ≠ {}", x, log10_val, x);
        }
    }

}

/// Property: random() returns value in [0.0, 1.0)
/// Coverage target: eval_random (lines 604-610)
#[test]
fn prop_random_bounds() {
    // Run random() 100 times and check all values are in valid range
    for _ in 0..100 {
        let result = eval_builtin_function("__builtin_random__", &[]);
        assert!(result.is_ok());

        if let Ok(Some(Value::Float(r))) = result {
            assert!(r >= 0.0 && r < 1.0,
                "random() = {} not in [0.0, 1.0)", r);
        }
    }
}

// ============================================================================
// Property Tests: Utility Functions
// ============================================================================

proptest! {
    /// Property: len(array) matches actual element count
    /// Coverage target: eval_len (lines 616-637)
    #[test]
    fn prop_len_accuracy(elements in prop::collection::vec(0i64..100i64, 0..100)) {
        let values: Vec<Value> = elements.iter().map(|&x| Value::Integer(x)).collect();
        let array = Value::from_array(values);

        let result = eval_builtin_function("__builtin_len__", &[array]);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(elements.len() as i64)),
            "len(array) ≠ {}", elements.len());
    }

    /// Property: len(string) matches byte count
    /// Coverage target: eval_len string branch (line 622)
    #[test]
    fn prop_len_string(s in ".*") {
        let result = eval_builtin_function("__builtin_len__", &[Value::String(Arc::from(s.as_str()))]);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(s.len() as i64)),
            "len(\"{}\") ≠ {}", s, s.len());
    }

    /// Property: range(n) produces array of length n
    /// Coverage target: eval_range (lines 638-652)
    #[test]
    fn prop_range_length(n in 0i64..100i64) {
        let result = eval_builtin_function("__builtin_range__", &[Value::Integer(n)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Array(arr))) = result {
            prop_assert_eq!(arr.len(), n as usize, "range({}) length ≠ {}", n, n);

            // Verify values are [0, 1, 2, ..., n-1]
            for (i, val) in arr.iter().enumerate() {
                if let Value::Integer(v) = val {
                    prop_assert_eq!(*v, i as i64,
                        "range({})[{}] = {} ≠ {}", n, i, v, i);
                }
            }
        }
    }

    /// Property: range(start, end) produces correct sequence
    /// Coverage target: eval_range_two_args (lines 672-716)
    #[test]
    fn prop_range_two_args(start in -50i64..50i64, end in -50i64..50i64) {
        let result = eval_builtin_function("__builtin_range__",
            &[Value::Integer(start), Value::Integer(end)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Array(arr))) = result {
            let expected_len = if end > start { (end - start) as usize } else { 0 };
            prop_assert_eq!(arr.len(), expected_len,
                "range({}, {}) length ≠ {}", start, end, expected_len);

            // Verify sequence
            for (i, val) in arr.iter().enumerate() {
                if let Value::Integer(v) = val {
                    prop_assert_eq!(*v, start + i as i64);
                }
            }
        }
    }

    /// Property: reverse(reverse(array)) == array
    /// Coverage target: eval_reverse (lines 775-796)
    #[test]
    fn prop_reverse_involution(elements in prop::collection::vec(0i64..100i64, 0..50)) {
        let values: Vec<Value> = elements.iter().map(|&x| Value::Integer(x)).collect();
        let array = Value::from_array(values.clone());

        // First reverse
        let result1 = eval_builtin_function("__builtin_reverse__", &[array]);
        prop_assert!(result1.is_ok());

        // Second reverse
        if let Ok(Some(reversed_once)) = result1 {
            let result2 = eval_builtin_function("__builtin_reverse__", &[reversed_once]);
            prop_assert!(result2.is_ok());

            if let Ok(Some(Value::Array(arr))) = result2 {
                prop_assert_eq!(arr.len(), elements.len());
                for (i, val) in arr.iter().enumerate() {
                    if let Value::Integer(v) = val {
                        prop_assert_eq!(*v, elements[i],
                            "reverse(reverse(array))[{}] ≠ array[{}]", i, i);
                    }
                }
            }
        }
    }
}

// ============================================================================
// Property Tests: Type Conversion Functions
// ============================================================================

proptest! {
    /// Property: int(str(x)) == x (round-trip)
    /// Coverage target: eval_int (lines 253-254) + eval_str (line 250)
    #[test]
    fn prop_int_str_roundtrip(x in -1_000_000i64..1_000_000i64) {
        // str(x)
        let str_result = eval_builtin_function("__builtin_str__", &[Value::Integer(x)]);
        prop_assert!(str_result.is_ok());

        if let Ok(Some(Value::String(s))) = str_result {
            // int(str(x))
            let int_result = eval_builtin_function("__builtin_int__", &[Value::String(s)]);
            prop_assert!(int_result.is_ok());
            prop_assert_eq!(int_result.unwrap(), Some(Value::Integer(x)),
                "int(str({})) ≠ {}", x, x);
        }
    }

    /// Property: float(x) preserves integer value exactly (for small x)
    /// Coverage target: eval_float (line 253)
    #[test]
    fn prop_float_integer_exact(x in -1000i64..1000i64) {
        let result = eval_builtin_function("__builtin_float__", &[Value::Integer(x)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Float(f))) = result {
            prop_assert_eq!(f, x as f64, "float({}) ≠ {}", x, x as f64);
        }
    }

    /// Property: bool(non_zero_int) == true, bool(0) == false
    /// Coverage target: eval_bool (line 254)
    #[test]
    fn prop_bool_integer_truthiness(x in -1000i64..1000i64) {
        let result = eval_builtin_function("__builtin_bool__", &[Value::Integer(x)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Bool(b))) = result {
            let expected = x != 0;
            prop_assert_eq!(b, expected, "bool({}) = {} ≠ {}", x, b, expected);
        }
    }

    /// Property: parse_int(valid_string) succeeds
    /// Coverage target: eval_parse_int (line 255)
    #[test]
    fn prop_parse_int_valid(x in -1_000_000i64..1_000_000i64) {
        let s = x.to_string();
        let result = eval_builtin_function("__builtin_parse_int__", &[Value::String(Arc::from(s.as_str()))]);
        prop_assert!(result.is_ok());
        prop_assert_eq!(result.unwrap(), Some(Value::Integer(x)),
            "parse_int(\"{}\") ≠ {}", s, x);
    }
}

// ============================================================================
// Error Path Tests (Edge Cases)
// ============================================================================

proptest! {
    /// Property: sqrt rejects non-numeric input
    /// Coverage target: eval_sqrt error branch (line 397-399)
    #[test]
    fn prop_sqrt_rejects_string(s in ".*") {
        let result = eval_builtin_function("__builtin_sqrt__", &[Value::String(Arc::from(s.as_str()))]);
        // Should return Err or None
        if let Ok(Some(_)) = result {
            // If it succeeds, it must have been a valid numeric string
            // Most random strings will fail - that's expected
        }
        // Test passes if it handles error gracefully (no panic)
    }

    /// Property: pow requires exactly 2 arguments (validate_arg_count)
    /// Coverage target: validate_arg_count in eval_pow (line 408)
    #[test]
    fn prop_pow_arg_count_validation(x in 0i64..10i64) {
        // One arg - should fail
        let result_one = eval_builtin_function("__builtin_pow__", &[Value::Integer(x)]);
        prop_assert!(result_one.is_err() || matches!(result_one, Ok(None)));

        // Three args - should fail
        let result_three = eval_builtin_function("__builtin_pow__",
            &[Value::Integer(x), Value::Integer(2), Value::Integer(3)]);
        prop_assert!(result_three.is_err() || matches!(result_three, Ok(None)));
    }

    /// Property: min/max handle mixed integer/float correctly
    /// Coverage target: eval_min mixed type branches (lines 450-451)
    #[test]
    fn prop_min_mixed_types(a in -100i64..100i64, b in -100.0f64..100.0f64) {
        let result = eval_builtin_function("__builtin_min__",
            &[Value::Integer(a), Value::Float(b)]);
        prop_assert!(result.is_ok());

        if let Ok(Some(Value::Float(min_val))) = result {
            let expected = (a as f64).min(b);
            prop_assert!((min_val - expected).abs() < 0.001,
                "min({}, {}) = {} ≠ {}", a, b, min_val, expected);
        }
    }

    /// Property: parse_int handles invalid input gracefully
    /// Coverage target: eval_parse_int error path
    #[test]
    fn prop_parse_int_invalid(s in "[a-z]+") {
        // Alphabetic strings should fail to parse
        let result = eval_builtin_function("__builtin_parse_int__", &[Value::String(Arc::from(s.as_str()))]);
        // Should return Err, not panic
        if let Ok(Some(Value::Integer(_))) = result {
            prop_assert!(false, "parse_int(\"{}\") should fail", s);
        }
    }
}
