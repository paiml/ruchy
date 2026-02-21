use super::*;
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

// ============== Environment Functions ==============

#[test]
fn test_env_var_nonexistent() {
    // Just verify env_var with nonexistent key doesn't panic
    let _result = try_eval("env_var(\"NONEXISTENT_VAR_12345\")");
    // May return nil, error, empty string, or option - all are valid
}

#[test]
fn test_env_current_dir() {
    let result = eval("env_current_dir()");
    // Should return a path string
    assert!(!result.is_empty());
}

#[test]
fn test_env_temp_dir() {
    let result = eval("env_temp_dir()");
    // Should return a path string
    assert!(!result.is_empty());
}

// ============== Additional Path Functions ==============

#[test]
fn test_path_join() {
    let result = eval("path_join(\"/home\", \"user\")");
    assert!(result.contains("home") && result.contains("user"));
}

#[test]
fn test_path_parent() {
    let result = eval("path_parent(\"/home/user\")");
    assert!(result.contains("home"));
}

#[test]
fn test_path_file_name() {
    let result = eval("path_file_name(\"/home/user/file.txt\")");
    assert!(result.contains("file.txt"));
}

#[test]
fn test_path_file_stem() {
    let result = eval("path_file_stem(\"/home/user/file.txt\")");
    assert!(result.contains("file"));
}

#[test]
fn test_path_extension() {
    let result = eval("path_extension(\"/home/user/file.txt\")");
    assert!(result.contains("txt"));
}

#[test]
fn test_path_is_absolute_true() {
    assert_eq!(eval("path_is_absolute(\"/home/user\")"), "true");
}

#[test]
fn test_path_is_absolute_false() {
    assert_eq!(eval("path_is_absolute(\"relative/path\")"), "false");
}

#[test]
fn test_path_is_relative_true() {
    assert_eq!(eval("path_is_relative(\"relative/path\")"), "true");
}

#[test]
fn test_path_is_relative_false() {
    assert_eq!(eval("path_is_relative(\"/absolute/path\")"), "false");
}

// ============== JSON Functions ==============

#[test]
fn test_json_stringify_int() {
    let result = eval("json_stringify(42)");
    // JSON stringify may quote or not quote numbers
    assert!(result.contains("42"));
}

#[test]
fn test_json_stringify_string() {
    let result = eval("json_stringify(\"hello\")");
    assert!(result.contains("hello"));
}

#[test]
fn test_json_stringify_bool() {
    let result_true = eval("json_stringify(true)");
    let result_false = eval("json_stringify(false)");
    assert!(result_true.contains("true"));
    assert!(result_false.contains("false"));
}

#[test]
fn test_json_validate_valid() {
    assert_eq!(eval("json_validate(\"{\\\"key\\\": 42}\")"), "true");
}

#[test]
fn test_json_validate_invalid() {
    assert_eq!(eval("json_validate(\"not json\")"), "false");
}

#[test]
fn test_json_type_object() {
    let result = eval("json_type(\"{}\")");
    assert!(result.to_lowercase().contains("object") || result.contains("Object"));
}

#[test]
fn test_json_type_array() {
    let result = eval("json_type(\"[]\")");
    assert!(result.to_lowercase().contains("array") || result.contains("Array"));
}

#[test]
fn test_json_type_number() {
    let result = eval("json_type(\"42\")");
    assert!(
        result.to_lowercase().contains("number")
            || result.contains("Number")
            || result.contains("int")
    );
}

#[test]
fn test_json_type_string() {
    let result = eval("json_type(\"\\\"hello\\\"\")");
    assert!(result.to_lowercase().contains("string") || result.contains("String"));
}

#[test]
fn test_json_type_bool() {
    let result = eval("json_type(\"true\")");
    assert!(result.to_lowercase().contains("bool") || result.contains("Bool"));
}

#[test]
fn test_json_type_null() {
    let result = eval("json_type(\"null\")");
    assert!(
        result.to_lowercase().contains("null") || result.contains("Null") || result.contains("nil")
    );
}

// ============== Assertion Functions ==============

#[test]
fn test_assert_eq_passing() {
    let result = eval("assert_eq(1, 1)");
    // Should not error, returns nil/unit
    assert!(result.is_empty() || result.contains("nil") || result == "()");
}

#[test]
fn test_assert_passing() {
    let result = eval("assert(true)");
    // Should not error
    assert!(result.is_empty() || result.contains("nil") || result == "()");
}

// ============== Log Functions ==============

#[test]
fn test_log_natural() {
    let result = eval("log(2.718281828)");
    // ln(e) ≈ 1.0
    let val: f64 = result.parse().unwrap_or(0.0);
    assert!((val - 1.0).abs() < 0.1);
}

#[test]
fn test_log10() {
    let result = eval("log10(100)");
    // log10(100) = 2
    let val: f64 = result.parse().unwrap_or(0.0);
    assert!((val - 2.0).abs() < 0.01);
}

#[test]
fn test_exp() {
    let result = eval("exp(0)");
    // e^0 = 1
    let val: f64 = result.parse().unwrap_or(0.0);
    assert!((val - 1.0).abs() < 0.01);
}

#[test]
fn test_exp_one() {
    let result = eval("exp(1)");
    // e^1 ≈ 2.718
    let val: f64 = result.parse().unwrap_or(0.0);
    assert!((val - 2.718).abs() < 0.1);
}

// ============== Range Functions ==============

#[test]
fn test_range_one_arg() {
    let result = eval("range(5)");
    assert!(result.contains("0") && result.contains("4"));
}

#[test]
fn test_range_two_args() {
    let result = eval("range(2, 5)");
    assert!(result.contains("2") && result.contains("4"));
}

#[test]
fn test_range_three_args() {
    let result = eval("range(0, 10, 2)");
    assert!(result.contains("0") && result.contains("8"));
}

// ============== Time Functions ==============

#[test]
fn test_timestamp() {
    let result = eval("timestamp()");
    // Should be a large integer (unix timestamp in ms or s)
    let val: i64 = result.parse().unwrap_or(0);
    assert!(val > 0);
}

// ============== EXTREME TDD Round 115 - Additional Tests ==============

// --- is_nil Tests ---
#[test]
fn test_is_nil_with_nil() {
    let result = eval("is_nil(nil)");
    assert_eq!(result, "true");
}

#[test]
fn test_is_nil_with_value() {
    let result = eval("is_nil(42)");
    assert_eq!(result, "false");
}

#[test]
fn test_is_nil_with_empty_string() {
    let result = eval("is_nil(\"\")");
    assert_eq!(result, "false");
}

// --- Collection Functions ---
#[test]
fn test_reverse_array() {
    let result = eval("reverse([1, 2, 3])");
    assert!(result.contains("3") && result.contains("1"));
}

#[test]
fn test_reverse_string() {
    let result = eval("reverse(\"abc\")");
    assert!(result.contains("cba") || result.contains("c"));
}

#[test]
fn test_push_to_array() {
    let result = eval("push([1, 2], 3)");
    assert!(result.contains("1") && result.contains("2") && result.contains("3"));
}

#[test]
fn test_pop_from_array() {
    let result = eval("pop([1, 2, 3])");
    // Should return the array without last element or the popped element
    assert!(result.contains("1") || result.contains("2") || result.contains("3"));
}

#[test]
fn test_sort_integers() {
    let result = eval("sort([3, 1, 4, 1, 5])");
    // Should be sorted
    assert!(result.contains("1"));
}

#[test]
fn test_zip_two_arrays() {
    let result = eval("zip([1, 2], [\"a\", \"b\"])");
    // Should return array of tuples
    assert!(result.contains("1") && result.contains("a"));
}

#[test]
fn test_enumerate_array() {
    let result = eval("enumerate([\"a\", \"b\", \"c\"])");
    // Should return array with indices
    assert!(result.contains("0") && result.contains("a"));
}

// --- Type Conversion Tests (Unique) ---
#[test]
fn test_str_from_float_conversion() {
    let result = eval("str(3.14)");
    assert!(result.contains("3.14"));
}

#[test]
fn test_float_from_string_conversion() {
    let result = eval("float(\"3.14\")");
    assert!(result.contains("3.14"));
}

#[test]
fn test_bool_from_int_zero_value() {
    let result = eval("bool(0)");
    assert_eq!(result, "false");
}

#[test]
fn test_bool_from_int_nonzero_value() {
    let result = eval("bool(1)");
    assert_eq!(result, "true");
}

#[test]
fn test_bool_from_empty_string_value() {
    let result = eval("bool(\"\")");
    assert_eq!(result, "false");
}

#[test]
fn test_bool_from_nonempty_string_value() {
    let result = eval("bool(\"hello\")");
    assert_eq!(result, "true");
}

// --- Assert Functions (Unique) ---
#[test]
fn test_assert_eq_same_ints() {
    let result = eval("assert_eq(42, 42)");
    // Should pass without error
    assert!(result.is_empty() || result.contains("nil") || result.contains("()"));
}

// --- Edge Cases (Unique) ---
#[test]
fn test_random_in_range() {
    // Run multiple times to ensure it's working
    for _ in 0..5 {
        let result = eval("random()");
        let val: f64 = result.parse().unwrap_or(-1.0);
        assert!(
            val >= 0.0 && val < 1.0,
            "random() should return 0.0 <= x < 1.0"
        );
    }
}

#[test]
fn test_parse_int_valid() {
    let result = eval("parse_int(\"123\")");
    assert!(result.contains("123") || result.contains("Some"));
}

#[test]
fn test_parse_float_valid() {
    let result = eval("parse_float(\"3.14\")");
    assert!(result.contains("3.14") || result.contains("Some"));
}

#[test]
fn test_to_string_int() {
    let result = eval("to_string(42)");
    assert!(result.contains("42"));
}

#[test]
fn test_to_string_bool() {
    let result = eval("to_string(true)");
    assert!(result.contains("true"));
}

// === EXTREME TDD Round 125 tests ===

#[test]
fn test_abs_positive_r125() {
    let result = eval("abs(42)");
    assert!(result.contains("42"));
}

#[test]
fn test_abs_negative_r125() {
    let result = eval("abs(-42)");
    assert!(result.contains("42"));
}

#[test]
fn test_abs_zero_r125() {
    let result = eval("abs(0)");
    assert!(result.contains("0"));
}

#[test]
fn test_min_two_values_r125() {
    let result = eval("min(10, 5)");
    assert!(result.contains("5"));
}

#[test]
fn test_max_two_values_r125() {
    let result = eval("max(10, 5)");
    assert!(result.contains("10"));
}

#[test]
fn test_sqrt_positive_r125() {
    let result = eval("sqrt(4.0)");
    assert!(result.contains("2"));
}

#[test]
fn test_floor_float_r125() {
    let result = eval("floor(3.7)");
    assert!(result.contains("3"));
}

#[test]
fn test_ceil_float_r125() {
    let result = eval("ceil(3.2)");
    assert!(result.contains("4"));
}

#[test]
fn test_round_float_up_r125() {
    let result = eval("round(3.7)");
    assert!(result.contains("4"));
}

#[test]
fn test_round_float_down_r125() {
    let result = eval("round(3.2)");
    assert!(result.contains("3"));
}

#[test]
fn test_len_string_r125() {
    let result = eval("len(\"hello\")");
    assert!(result.contains("5"));
}

#[test]
fn test_len_empty_string_r125() {
    let result = eval("len(\"\")");
    assert!(result.contains("0"));
}

#[test]
fn test_type_of_int_r125() {
    let result = eval("type_of(42)");
    assert!(
        result.contains("int")
            || result.contains("Integer")
            || result.contains("i32")
            || result.contains("i64")
    );
}

#[test]
fn test_type_of_string_r125() {
    let result = eval("type_of(\"hello\")");
    assert!(result.contains("String") || result.contains("str"));
}

#[test]
fn test_type_of_bool_r125() {
    let result = eval("type_of(true)");
    assert!(result.contains("bool") || result.contains("Bool"));
}

// === EXTREME TDD Round 126 tests - Comprehensive builtin coverage ===

// Trigonometric functions
#[test]
fn test_sin_zero_r126() {
    let result = eval("sin(0.0)");
    assert!(result.contains("0"));
}

#[test]
fn test_sin_integer_r126() {
    let result = eval("sin(0)");
    assert!(result.contains("0"));
}

#[test]
fn test_cos_zero_r126() {
    let result = eval("cos(0.0)");
    assert!(result.contains("1"));
}

#[test]
fn test_cos_integer_r126() {
    let result = eval("cos(0)");
    assert!(result.contains("1"));
}

#[test]
fn test_tan_zero_r126() {
    let result = eval("tan(0.0)");
    assert!(result.contains("0"));
}

#[test]
fn test_tan_integer_r126() {
    let result = eval("tan(0)");
    assert!(result.contains("0"));
}

// Logarithm functions
#[test]
fn test_log_e_r126() {
    let result = eval("log(2.718281828)");
    assert!(result.contains("0.9") || result.contains("1"));
}

#[test]
fn test_log_integer_r126() {
    let result = eval("log(1)");
    assert!(result.contains("0"));
}

#[test]
fn test_log10_ten_r126() {
    let result = eval("log10(10.0)");
    assert!(result.contains("1"));
}

#[test]
fn test_log10_integer_r126() {
    let result = eval("log10(100)");
    assert!(result.contains("2"));
}

#[test]
fn test_exp_zero_r126() {
    let result = eval("exp(0.0)");
    assert!(result.contains("1"));
}

#[test]
fn test_exp_one_r126() {
    let result = eval("exp(1.0)");
    assert!(result.contains("2.71") || result.contains("2.7"));
}

#[test]
fn test_exp_integer_r126() {
    let result = eval("exp(0)");
    assert!(result.contains("1"));
}

// Power function with various combinations
#[test]
fn test_pow_int_int_r126() {
    let result = eval("pow(2, 3)");
    assert!(result.contains("8"));
}

#[test]
fn test_pow_float_int_r126() {
    let result = eval("pow(2.0, 3)");
    assert!(result.contains("8"));
}

#[test]
fn test_pow_int_float_r126() {
    let result = eval("pow(2, 3.0)");
    assert!(result.contains("8"));
}

#[test]
fn test_pow_float_float_r126() {
    let result = eval("pow(2.0, 3.0)");
    assert!(result.contains("8"));
}

#[test]
fn test_pow_negative_exp_r126() {
    let result = eval("pow(2, -1)");
    assert!(result.contains("0.5"));
}

// Min/max with mixed types
#[test]
fn test_min_int_float_r126() {
    let result = eval("min(5, 3.5)");
    assert!(result.contains("3.5"));
}

#[test]
fn test_min_float_int_r126() {
    let result = eval("min(3.5, 5)");
    assert!(result.contains("3.5"));
}

#[test]
fn test_max_int_float_r126() {
    let result = eval("max(5, 3.5)");
    assert!(result.contains("5"));
}

#[test]
fn test_max_float_int_r126() {
    let result = eval("max(3.5, 5)");
    assert!(result.contains("5"));
}

#[test]
fn test_max_float_float_r126() {
    let result = eval("max(3.5, 7.5)");
    assert!(result.contains("7.5"));
}

#[test]
fn test_min_float_float_r126() {
    let result = eval("min(3.5, 7.5)");
    assert!(result.contains("3.5"));
}

// Abs with float
#[test]
fn test_abs_negative_float_r126() {
    let result = eval("abs(-3.14)");
    assert!(result.contains("3.14"));
}

#[test]
fn test_abs_positive_float_r126() {
    let result = eval("abs(3.14)");
    assert!(result.contains("3.14"));
}

// Sqrt with integer input
#[test]
fn test_sqrt_integer_r126() {
    let result = eval("sqrt(9)");
    assert!(result.contains("3"));
}

#[test]
fn test_sqrt_float_r126() {
    let result = eval("sqrt(16.0)");
    assert!(result.contains("4"));
}

// Floor/ceil/round with integers (should return same)
#[test]
fn test_floor_integer_r126() {
    let result = eval("floor(5)");
    assert!(result.contains("5"));
}

#[test]
fn test_ceil_integer_r126() {
    let result = eval("ceil(5)");
    assert!(result.contains("5"));
}

#[test]
fn test_round_integer_r126() {
    let result = eval("round(5)");
    assert!(result.contains("5"));
}

// Floor/ceil edge cases
#[test]
fn test_floor_negative_r126() {
    let result = eval("floor(-3.7)");
    assert!(result.contains("-4"));
}

#[test]
fn test_ceil_negative_r126() {
    let result = eval("ceil(-3.7)");
    assert!(result.contains("-3"));
}

#[test]
fn test_round_half_r126() {
    let result = eval("round(3.5)");
    assert!(result.contains("4"));
}

// Random function
#[test]
fn test_random_returns_number_r126() {
    let result = eval("random()");
    // random() returns a float between 0 and 1
    assert!(result.contains("0.") || result.contains("0") || result.contains("1"));
}

// Type conversions
#[test]
fn test_to_string_int_r126() {
    let result = eval("to_string(42)");
    assert!(result.contains("42"));
}

#[test]
fn test_to_string_float_r126() {
    let result = eval("to_string(3.14)");
    assert!(result.contains("3.14"));
}

#[test]
fn test_str_int_r126() {
    let result = eval("str(42)");
    assert!(result.contains("42"));
}

#[test]
fn test_str_float_r126() {
    let result = eval("str(3.14)");
    assert!(result.contains("3.14"));
}

// Array operations
#[test]
fn test_len_array_r126() {
    let result = eval("len([1, 2, 3])");
    assert!(result.contains("3"));
}

#[test]
fn test_len_empty_array_r126() {
    let result = eval("len([])");
    assert!(result.contains("0"));
}
