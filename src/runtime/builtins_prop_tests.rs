
use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_abs_idempotent(n: i64) {
        let val = Value::Integer(n);
        let result1 = builtin_abs(&[val])
    .expect("builtin function should succeed in test");
        let result2 = builtin_abs(std::slice::from_ref(&result1))
    .expect("builtin function should succeed in test");
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn test_min_max_consistency(a: i64, b: i64) {
        let min_result = builtin_min(&[Value::Integer(a), Value::Integer(b)])
    .expect("builtin function should succeed in test");
        let max_result = builtin_max(&[Value::Integer(a), Value::Integer(b)])
    .expect("builtin function should succeed in test");

        // min and max should return one of the inputs
        prop_assert!(min_result == Value::Integer(a) || min_result == Value::Integer(b));
        prop_assert!(max_result == Value::Integer(a) || max_result == Value::Integer(b));

        // min should be <= max
        match (min_result, max_result) {
            (Value::Integer(min), Value::Integer(max)) => prop_assert!(min <= max),
            _ => prop_assert!(false),
        }
    }

    #[test]
    fn test_to_string_parse_roundtrip(n: i64) {
        let val = Value::Integer(n);
        let str_val = builtin_to_string(&[val])
    .expect("builtin function should succeed in test");
        let parsed = builtin_parse_int(&[str_val])
    .expect("builtin function should succeed in test");
        prop_assert_eq!(parsed, Value::Integer(n));
    }
}
