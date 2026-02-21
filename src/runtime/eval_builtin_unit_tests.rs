
use super::*;

#[test]
fn test_eval_sqrt() {
    let args = vec![Value::Integer(16)];
    let result = eval_sqrt(&args).expect("eval_sqrt should succeed in test");
    assert_eq!(result, Value::Float(4.0));

    let args = vec![Value::Float(9.0)];
    let result = eval_sqrt(&args).expect("eval_sqrt should succeed in test");
    assert_eq!(result, Value::Float(3.0));
}

#[test]
fn test_eval_pow() {
    let args = vec![Value::Integer(2), Value::Integer(3)];
    let result = eval_pow(&args).expect("eval_pow should succeed in test");
    assert_eq!(result, Value::Integer(8));

    let args = vec![Value::Float(2.0), Value::Float(3.0)];
    let result = eval_pow(&args).expect("eval_pow should succeed in test");
    assert_eq!(result, Value::Float(8.0));
}

#[test]
fn test_eval_abs() {
    let args = vec![Value::Integer(-42)];
    let result = eval_abs(&args).expect("eval_abs should succeed in test");
    assert_eq!(result, Value::Integer(42));

    let args = vec![Value::Float(-3.15)];
    let result = eval_abs(&args).expect("eval_abs should succeed in test");
    assert_eq!(result, Value::Float(3.15));
}

#[test]
fn test_eval_min_max() {
    let args = vec![Value::Integer(5), Value::Integer(3)];
    let min_result = eval_min(&args).expect("eval_min should succeed in test");
    assert_eq!(min_result, Value::Integer(3));

    let max_result = eval_max(&args).expect("eval_max should succeed in test");
    assert_eq!(max_result, Value::Integer(5));
}

#[test]
fn test_eval_len() {
    let args = vec![Value::from_string("hello".to_string())];
    let result = eval_len(&args).expect("eval_len should succeed in test");
    assert_eq!(result, Value::Integer(5));

    let args = vec![Value::Array(Arc::from(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]))];
    let result = eval_len(&args).expect("eval_len should succeed in test");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_eval_type() {
    let args = vec![Value::Integer(42)];
    let result = eval_type(&args).expect("eval_type should succeed in test");
    assert_eq!(result, Value::from_string("integer".to_string()));

    let args = vec![Value::Float(3.15)];
    let result = eval_type(&args).expect("eval_type should succeed in test");
    assert_eq!(result, Value::from_string("float".to_string()));
}

#[test]
fn test_eval_range() {
    let args = vec![Value::Integer(3)];
    let result = eval_range(&args).expect("eval_range should succeed in test");
    if let Value::Array(arr) = result {
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::Integer(0));
        assert_eq!(arr[1], Value::Integer(1));
        assert_eq!(arr[2], Value::Integer(2));
    } else {
        panic!("Expected array result");
    }
}

#[test]
fn test_eval_reverse() {
    let args = vec![Value::Array(Arc::from(vec![
        Value::Integer(1),
        Value::Integer(2),
        Value::Integer(3),
    ]))];
    let result = eval_reverse(&args).expect("eval_reverse should succeed in test");
    if let Value::Array(arr) = result {
        assert_eq!(arr[0], Value::Integer(3));
        assert_eq!(arr[1], Value::Integer(2));
        assert_eq!(arr[2], Value::Integer(1));
    } else {
        panic!("Expected array result");
    }

    let args = vec![Value::from_string("hello".to_string())];
    let result = eval_reverse(&args).expect("eval_reverse should succeed in test");
    assert_eq!(result, Value::from_string("olleh".to_string()));
}

// ============================================================================
// EXTREME TDD: Comprehensive Builtin Function Testing (QUALITY-008)
// Coverage Target: 16.83% → 70%+
// ============================================================================

// --------------------------------------------------------------------------
// Math Functions (floor, ceil, round, sin, cos, tan)
// --------------------------------------------------------------------------

#[test]
fn test_eval_floor() {
    let args = vec![Value::Float(3.7)];
    let result = eval_floor(&args).expect("eval_floor should succeed in test");
    assert_eq!(result, Value::Integer(3));

    let args = vec![Value::Float(-2.3)];
    let result = eval_floor(&args).expect("eval_floor should succeed in test");
    assert_eq!(result, Value::Integer(-3));

    let args = vec![Value::Integer(5)];
    let result = eval_floor(&args).expect("eval_floor should succeed in test");
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_eval_ceil() {
    let args = vec![Value::Float(3.2)];
    let result = eval_ceil(&args).expect("eval_ceil should succeed in test");
    assert_eq!(result, Value::Integer(4));

    let args = vec![Value::Float(-2.7)];
    let result = eval_ceil(&args).expect("eval_ceil should succeed in test");
    assert_eq!(result, Value::Integer(-2));

    let args = vec![Value::Integer(5)];
    let result = eval_ceil(&args).expect("eval_ceil should succeed in test");
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_eval_round() {
    let args = vec![Value::Float(3.5)];
    let result = eval_round(&args).expect("eval_round should succeed in test");
    assert_eq!(result, Value::Integer(4));

    let args = vec![Value::Float(3.4)];
    let result = eval_round(&args).expect("eval_round should succeed in test");
    assert_eq!(result, Value::Integer(3));

    // Note: Rust's round() uses banker's rounding (round half to even)
    let args = vec![Value::Float(-2.5)];
    let result = eval_round(&args).expect("eval_round should succeed in test");
    assert_eq!(result, Value::Integer(-3));

    let args = vec![Value::Integer(7)];
    let result = eval_round(&args).expect("eval_round should succeed in test");
    assert_eq!(result, Value::Integer(7));
}

#[test]
fn test_eval_sin() {
    use std::f64::consts::PI;

    let args = vec![Value::Float(0.0)];
    let result = eval_sin(&args).expect("eval_sin should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 0.0).abs() < 1e-10, "sin(0) should be ~0");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Float(PI / 2.0)];
    let result = eval_sin(&args).expect("eval_sin should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 1.0).abs() < 1e-10, "sin(π/2) should be ~1");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Integer(0)];
    let result = eval_sin(&args).expect("eval_sin should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 0.0).abs() < 1e-10);
    } else {
        panic!("Expected float result");
    }
}

#[test]
fn test_eval_cos() {
    use std::f64::consts::PI;

    let args = vec![Value::Float(0.0)];
    let result = eval_cos(&args).expect("eval_cos should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 1.0).abs() < 1e-10, "cos(0) should be ~1");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Float(PI)];
    let result = eval_cos(&args).expect("eval_cos should succeed in test");
    if let Value::Float(v) = result {
        assert!((v + 1.0).abs() < 1e-10, "cos(π) should be ~-1");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Integer(0)];
    let result = eval_cos(&args).expect("eval_cos should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 1.0).abs() < 1e-10);
    } else {
        panic!("Expected float result");
    }
}

#[test]
fn test_eval_tan() {
    use std::f64::consts::PI;

    let args = vec![Value::Float(0.0)];
    let result = eval_tan(&args).expect("eval_tan should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 0.0).abs() < 1e-10, "tan(0) should be ~0");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Float(PI / 4.0)];
    let result = eval_tan(&args).expect("eval_tan should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 1.0).abs() < 1e-10, "tan(π/4) should be ~1");
    } else {
        panic!("Expected float result");
    }

    let args = vec![Value::Integer(0)];
    let result = eval_tan(&args).expect("eval_tan should succeed in test");
    if let Value::Float(v) = result {
        assert!((v - 0.0).abs() < 1e-10);
    } else {
        panic!("Expected float result");
    }
}

// --------------------------------------------------------------------------
// Assertion Functions (assert, assert_eq)
// --------------------------------------------------------------------------

#[test]
fn test_eval_assert_true() {
    let args = vec![Value::Bool(true)];
    let result = eval_assert(&args);
    assert!(result.is_ok(), "assert(true) should succeed");
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::Nil
    );
}

#[test]
fn test_eval_assert_false() {
    let args = vec![Value::Bool(false)];
    let result = eval_assert(&args);
    assert!(result.is_err(), "assert(false) should fail");
}

#[test]
fn test_eval_assert_with_message() {
    let args = vec![
        Value::Bool(false),
        Value::from_string("Custom error".to_string()),
    ];
    let result = eval_assert(&args);
    assert!(result.is_err(), "assert(false, msg) should fail");
    if let Err(InterpreterError::AssertionFailed(msg)) = result {
        assert!(
            msg.contains("Custom error"),
            "Should include custom message"
        );
    } else {
        panic!("Expected AssertionFailed error");
    }
}

#[test]
fn test_eval_assert_non_boolean() {
    let args = vec![Value::Integer(1)];
    let result = eval_assert(&args);
    assert!(result.is_err(), "assert(non-bool) should fail");
}

#[test]
fn test_eval_assert_eq_equal() {
    let args = vec![Value::Integer(42), Value::Integer(42)];
    let result = eval_assert_eq(&args);
    assert!(result.is_ok(), "assert_eq(42, 42) should succeed");
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::Nil
    );
}

#[test]
fn test_eval_assert_eq_not_equal() {
    let args = vec![Value::Integer(42), Value::Integer(43)];
    let result = eval_assert_eq(&args);
    assert!(result.is_err(), "assert_eq(42, 43) should fail");
}

#[test]
fn test_eval_assert_eq_strings() {
    let args = vec![
        Value::from_string("hello".to_string()),
        Value::from_string("hello".to_string()),
    ];
    let result = eval_assert_eq(&args);
    assert!(result.is_ok(), "assert_eq strings should succeed");

    let args = vec![
        Value::from_string("hello".to_string()),
        Value::from_string("world".to_string()),
    ];
    let result = eval_assert_eq(&args);
    assert!(result.is_err(), "assert_eq different strings should fail");
}

// --------------------------------------------------------------------------
// Core I/O Functions (println, print, dbg)
// Note: These functions have side effects (stdout), so we test they don't panic
// --------------------------------------------------------------------------

#[test]
fn test_eval_println_basic() {
    let args = vec![Value::from_string("Hello, World!".to_string())];
    let result = eval_println(&args);
    assert!(result.is_ok(), "println should not panic");
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::Nil
    );
}

#[test]
fn test_eval_println_multiple_args() {
    let args = vec![
        Value::from_string("Hello".to_string()),
        Value::from_string("World".to_string()),
    ];
    let result = eval_println(&args);
    assert!(
        result.is_ok(),
        "println with multiple args should not panic"
    );
}

#[test]
fn test_eval_println_no_args() {
    let args = vec![];
    let result = eval_println(&args);
    assert!(result.is_ok(), "println with no args should print newline");
}

#[test]
fn test_eval_print_basic() {
    let args = vec![Value::from_string("Test".to_string())];
    let result = eval_print(&args);
    assert!(result.is_ok(), "print should not panic");
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::Nil
    );
}

#[test]
fn test_eval_print_integers() {
    let args = vec![Value::Integer(42)];
    let result = eval_print(&args);
    assert!(result.is_ok(), "print(42) should not panic");
}

#[test]
fn test_eval_dbg_basic() {
    let args = vec![Value::Integer(123)];
    let result = eval_dbg(&args);
    assert!(result.is_ok(), "dbg should not panic");
    // dbg returns the value, not Nil
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::Integer(123)
    );
}

#[test]
fn test_eval_dbg_string() {
    let args = vec![Value::from_string("debug".to_string())];
    let result = eval_dbg(&args);
    assert!(result.is_ok(), "dbg should not panic");
    assert_eq!(
        result.expect("operation should succeed in test"),
        Value::from_string("debug".to_string())
    );
}

// --------------------------------------------------------------------------
// Property Tests (Mathematical Invariants)
// --------------------------------------------------------------------------

#[test]
fn prop_floor_ceil_relationship() {
    // Property: floor(x) <= x <= ceil(x)
    let test_values = vec![3.1, 3.9, -2.3, -2.9, 0.0, 5.0];

    for val in test_values {
        let floor_result =
            eval_floor(&[Value::Float(val)]).expect("eval_floor should succeed in test");
        let ceil_result =
            eval_ceil(&[Value::Float(val)]).expect("eval_ceil should succeed in test");

        if let (Value::Integer(floor), Value::Integer(ceil)) = (floor_result, ceil_result) {
            let floor_f = floor as f64;
            let ceil_f = ceil as f64;
            assert!(floor_f <= val, "floor({val}) should be <= {val}");
            assert!(ceil_f >= val, "ceil({val}) should be >= {val}");
            assert!(floor_f <= ceil_f, "floor({val}) <= ceil({val})");
        }
    }
}

#[test]
fn prop_trig_pythagorean_identity() {
    // Property: sin²(x) + cos²(x) = 1
    use std::f64::consts::PI;
    let test_angles = vec![0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0];

    for angle in test_angles {
        let sin_val = eval_sin(&[Value::Float(angle)]).expect("eval_sin should succeed in test");
        let cos_val = eval_cos(&[Value::Float(angle)]).expect("eval_cos should succeed in test");

        if let (Value::Float(s), Value::Float(c)) = (sin_val, cos_val) {
            let identity = s * s + c * c;
            assert!(
                (identity - 1.0).abs() < 1e-10,
                "sin²({angle}) + cos²({angle}) should = 1, got {identity}"
            );
        }
    }
}

#[test]
fn prop_abs_non_negative() {
    // Property: abs(x) >= 0 for all x
    let test_values = vec![
        Value::Integer(-100),
        Value::Integer(0),
        Value::Integer(100),
        Value::Float(-3.15),
        Value::Float(0.0),
        Value::Float(2.71),
    ];

    for val in test_values {
        let result = eval_abs(&[val]).expect("eval_abs should succeed in test");
        match result {
            Value::Integer(i) => assert!(i >= 0, "abs should be non-negative"),
            Value::Float(f) => assert!(f >= 0.0, "abs should be non-negative"),
            _ => panic!("abs should return number"),
        }
    }
}

// Standalone test (was outside mod tests in original)
#[test]
fn test_println_string_no_quotes() {
    // DEFECT: println should print strings WITHOUT quotes
    // Expected: "Hello Ruchy" → Hello Ruchy (no quotes)
    // Actual: "Hello Ruchy" → "Hello Ruchy" (with quotes)
    let fmt = Value::from_string("Name: {}".to_string());
    let arg = Value::from_string("Ruchy".to_string());
    let output = format_println_output(&[fmt, arg]);

    // Should NOT contain quotes around Ruchy
    assert!(
        !output.contains("\"Ruchy\""),
        "println should not print quotes around strings, got: {output}"
    );
    assert!(
        output.contains("Name: Ruchy"),
        "Expected 'Name: Ruchy' without quotes, got: {output}"
    );
}
