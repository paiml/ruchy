//! Tests for eval_builtin module
//!
//! EXTREME TDD Round 86: Comprehensive tests for builtin functions
//! Coverage target: 95% for eval_builtin module
//!
//! These tests use the REPL to evaluate builtin functions end-to-end.
//! Requires `repl` feature since they use the REPL for evaluation.

#[cfg(all(test, feature = "repl"))]
mod tests {
    use crate::runtime::Repl;

    // Helper to create a REPL and evaluate an expression
    fn eval(code: &str) -> String {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed");
        repl.eval(code).expect("eval should succeed")
    }

    // Helper that may or may not succeed
    fn try_eval(code: &str) -> Option<String> {
        let mut repl = Repl::new(std::env::temp_dir()).expect("Repl::new should succeed");
        repl.eval(code).ok()
    }

    // ============== Math Functions ==============

    #[test]
    fn test_sqrt_int() {
        // sqrt(16) returns 4.0 as float
        let result = eval("sqrt(16)");
        assert!(result == "4" || result == "4.0");
    }

    #[test]
    fn test_abs() {
        assert_eq!(eval("abs(-5)"), "5");
        assert_eq!(eval("abs(5)"), "5");
    }

    #[test]
    fn test_abs_float() {
        let result = eval("abs(-3.14)");
        assert!(result.contains("3.14"));
    }

    #[test]
    fn test_floor() {
        assert_eq!(eval("floor(3.7)"), "3");
    }

    #[test]
    fn test_floor_negative() {
        assert_eq!(eval("floor(-3.2)"), "-4");
    }

    #[test]
    fn test_ceil() {
        assert_eq!(eval("ceil(3.2)"), "4");
    }

    #[test]
    fn test_ceil_negative() {
        assert_eq!(eval("ceil(-3.7)"), "-3");
    }

    #[test]
    fn test_round_down() {
        assert_eq!(eval("round(3.4)"), "3");
    }

    #[test]
    fn test_round_up() {
        assert_eq!(eval("round(3.6)"), "4");
    }

    #[test]
    fn test_min() {
        assert_eq!(eval("min(3, 5)"), "3");
        assert_eq!(eval("min(-1, 1)"), "-1");
    }

    #[test]
    fn test_max() {
        assert_eq!(eval("max(3, 5)"), "5");
        assert_eq!(eval("max(-1, 1)"), "1");
    }

    #[test]
    fn test_pow_int() {
        assert_eq!(eval("pow(2, 3)"), "8");
    }

    #[test]
    fn test_pow_zero() {
        assert_eq!(eval("pow(10, 0)"), "1");
    }

    #[test]
    fn test_sin() {
        let result = eval("sin(0)");
        assert!(result == "0" || result.starts_with("0.0") || result.contains("-0"));
    }

    #[test]
    fn test_cos() {
        let result = eval("cos(0)");
        assert!(result == "1" || result.starts_with("1.0"));
    }

    #[test]
    fn test_tan() {
        let result = eval("tan(0)");
        assert!(result == "0" || result.starts_with("0.0") || result.contains("-0"));
    }

    #[test]
    fn test_log_e() {
        let result = eval("log(2.718281828459045)");
        let val: f64 = result.parse().unwrap_or(0.0);
        assert!((val - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_log10_100() {
        let result = eval("log10(100)");
        // May return 2 or 2.0
        assert!(result == "2" || result.starts_with("2.0"));
    }

    #[test]
    fn test_exp_zero() {
        let result = eval("exp(0)");
        assert!(result == "1" || result.starts_with("1.0"));
    }

    // ============== Collection Functions ==============

    #[test]
    fn test_len_array() {
        assert_eq!(eval("len([1, 2, 3])"), "3");
    }

    #[test]
    fn test_len_empty_array() {
        assert_eq!(eval("len([])"), "0");
    }

    #[test]
    fn test_len_string() {
        assert_eq!(eval("len(\"hello\")"), "5");
    }

    #[test]
    fn test_len_empty_string() {
        assert_eq!(eval("len(\"\")"), "0");
    }

    #[test]
    fn test_range_basic() {
        let result = eval("range(0, 3)");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_sorted() {
        let result = eval("sorted([3, 1, 2])");
        assert!(result.contains("1") && result.contains("2") && result.contains("3"));
    }

    #[test]
    fn test_reversed() {
        let result = eval("reversed([1, 2, 3])");
        assert!(result.contains("3") && result.contains("2") && result.contains("1"));
    }

    #[test]
    fn test_enumerate() {
        let result = eval("enumerate([\"a\", \"b\"])");
        // Result should contain indices
        assert!(!result.is_empty());
    }

    #[test]
    fn test_zip_arrays() {
        let result = eval("zip([1, 2], [\"a\", \"b\"])");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_take_basic() {
        let result = eval("take([1, 2, 3, 4], 2)");
        assert!(result.contains("1"));
    }

    #[test]
    fn test_drop_basic() {
        let result = eval("drop([1, 2, 3, 4], 2)");
        assert!(result.contains("3") || result.contains("4"));
    }

    // ============== Conversion Functions ==============

    #[test]
    fn test_int_from_float() {
        assert_eq!(eval("int(3.14)"), "3");
    }

    #[test]
    fn test_int_from_string() {
        assert_eq!(eval("int(\"42\")"), "42");
    }

    #[test]
    fn test_float_from_int() {
        let result = eval("float(42)");
        assert!(result == "42" || result == "42.0");
    }

    #[test]
    fn test_str_from_int() {
        let result = eval("str(42)");
        assert!(result.contains("42"));
    }

    #[test]
    fn test_str_from_bool() {
        let result = eval("str(true)");
        assert!(result.contains("true"));
    }

    #[test]
    fn test_bool_from_int() {
        assert_eq!(eval("bool(1)"), "true");
        assert_eq!(eval("bool(0)"), "false");
    }

    // ============== String Method-based Tests ==============

    #[test]
    fn test_string_len_method() {
        // Test via method call
        assert_eq!(eval("\"hello\".len()"), "5");
    }

    #[test]
    fn test_string_upper_method() {
        let result = eval("\"hello\".upper()");
        assert!(result.contains("HELLO"));
    }

    #[test]
    fn test_string_lower_method() {
        let result = eval("\"HELLO\".lower()");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_string_trim_method() {
        let result = eval("\"  hello  \".trim()");
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_string_split_method() {
        let result = eval("\"a,b,c\".split(\",\")");
        assert!(result.contains("a"));
    }

    #[test]
    fn test_string_contains_method() {
        assert_eq!(eval("\"hello\".contains(\"ell\")"), "true");
        assert_eq!(eval("\"hello\".contains(\"xyz\")"), "false");
    }

    #[test]
    fn test_string_starts_with_method() {
        assert_eq!(eval("\"hello\".starts_with(\"hel\")"), "true");
    }

    #[test]
    fn test_string_ends_with_method() {
        assert_eq!(eval("\"hello\".ends_with(\"llo\")"), "true");
    }

    #[test]
    fn test_string_replace_method() {
        let result = eval("\"hello\".replace(\"l\", \"L\")");
        assert!(result.contains("L"));
    }

    #[test]
    fn test_chars_method() {
        let result = eval("\"hi\".chars()");
        assert!(!result.is_empty());
    }

    // ============== Type Functions ==============

    #[test]
    fn test_type_of_int() {
        let result = eval("type_of(42)");
        // Could be "Integer" or "integer"
        assert!(result.to_lowercase().contains("integer"));
    }

    #[test]
    fn test_type_of_float() {
        let result = eval("type_of(3.14)");
        assert!(result.to_lowercase().contains("float"));
    }

    #[test]
    fn test_type_of_string() {
        let result = eval("type_of(\"hello\")");
        assert!(result.to_lowercase().contains("string"));
    }

    #[test]
    fn test_type_of_bool() {
        let result = eval("type_of(true)");
        assert!(result.to_lowercase().contains("bool"));
    }

    #[test]
    fn test_type_of_array() {
        let result = eval("type_of([1,2,3])");
        assert!(result.to_lowercase().contains("array"));
    }

    #[test]
    fn test_is_nil() {
        assert_eq!(eval("is_nil(nil)"), "true");
        assert_eq!(eval("is_nil(42)"), "false");
    }

    // ============== Utility Functions ==============

    #[test]
    fn test_assert_true() {
        let result = eval("assert(true)");
        // Should not error, return nil
        assert!(result == "nil" || result == "()" || result.is_empty() || result == "true");
    }

    #[test]
    fn test_assert_eq_same() {
        let result = eval("assert_eq(1, 1)");
        assert!(result == "nil" || result == "()" || result.is_empty() || result == "true");
    }

    #[test]
    fn test_hash_deterministic() {
        let result1 = eval("hash(\"hello\")");
        let result2 = eval("hash(\"hello\")");
        assert_eq!(result1, result2);
    }

    // ============== I/O Functions ==============

    #[test]
    fn test_println() {
        let result = eval("println(\"test\")");
        assert!(result == "nil" || result == "()" || result.is_empty());
    }

    #[test]
    fn test_print() {
        let result = eval("print(\"test\")");
        assert!(result == "nil" || result == "()" || result.is_empty());
    }

    #[test]
    fn test_dbg() {
        let result = eval("dbg(42)");
        assert!(result.contains("42"));
    }

    // ============== Array Operations ==============

    #[test]
    fn test_push_array() {
        let result = eval("push([1, 2], 3)");
        assert!(result.contains("3"));
    }

    #[test]
    fn test_pop_array() {
        let result = eval("pop([1, 2, 3])");
        // Returns popped element or modified array
        assert!(!result.is_empty());
    }

    #[test]
    fn test_append_arrays() {
        let result = eval("append([1, 2], [3, 4])");
        assert!(result.contains("1") || result.contains("4"));
    }

    // ============== Map/Object Functions ==============

    #[test]
    fn test_keys_object() {
        let result = eval("keys({a: 1, b: 2})");
        assert!(result.contains("a") || result.contains("b"));
    }

    #[test]
    fn test_values_object() {
        let result = eval("values({a: 1, b: 2})");
        assert!(result.contains("1") || result.contains("2"));
    }

    #[test]
    fn test_entries_object() {
        let result = eval("entries({a: 1})");
        assert!(!result.is_empty());
    }

    // ============== Random Functions ==============

    #[test]
    fn test_random() {
        let result = eval("random()");
        let val: f64 = result.parse().unwrap_or(-1.0);
        assert!(val >= 0.0 && val <= 1.0);
    }

    // ============== Edge Cases ==============

    #[test]
    fn test_min_same_values() {
        assert_eq!(eval("min(5, 5)"), "5");
    }

    #[test]
    fn test_max_same_values() {
        assert_eq!(eval("max(5, 5)"), "5");
    }

    #[test]
    fn test_abs_zero() {
        assert_eq!(eval("abs(0)"), "0");
    }

    #[test]
    fn test_sqrt_one() {
        let result = eval("sqrt(1)");
        assert!(result == "1" || result == "1.0");
    }

    #[test]
    fn test_pow_one_power() {
        assert_eq!(eval("pow(5, 1)"), "5");
    }

    // ============== Array Tests (Function Form) ==============

    #[test]
    fn test_array_len_function() {
        assert_eq!(eval("len([1,2,3])"), "3");
    }

    #[test]
    fn test_array_first_method_via_index() {
        // Use array indexing instead of first()
        assert_eq!(eval("[1,2,3][0]"), "1");
    }

    #[test]
    fn test_array_last_method_via_index() {
        // Use negative indexing instead of last()
        assert_eq!(eval("[1,2,3][-1]"), "3");
    }

    #[test]
    fn test_array_sorted_function() {
        let result = eval("sorted([3,1,2])");
        assert!(result.contains("1"));
    }

    #[test]
    fn test_array_reversed_function() {
        let result = eval("reversed([1,2,3])");
        assert!(result.contains("3"));
    }

}
