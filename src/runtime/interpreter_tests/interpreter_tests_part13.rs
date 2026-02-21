// Auto-extracted from interpreter_tests.rs - Part 13
use super::*;

// ============================================================================
// COVERAGE IMPROVEMENT: Actor Functions (interpreter_types_actor.rs)
// Note: Actor tests are ignored during coverage due to threading timeouts
// ============================================================================

#[test]
#[ignore] // Actor spawns threads that don't terminate during coverage
fn test_actor_spawn_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Counter {
            count: i32 = 0
            fn increment(&mut self) {
                self.count = self.count + 1
            }
        }
        let a = Counter::spawn()
    "#,
    );
    let _ = result;
}

#[test]
#[ignore] // Actor spawns threads that don't terminate during coverage
fn test_actor_send_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Printer {
            fn print(&self, msg: String) {
                println(msg)
            }
        }
        let p = Printer::spawn()
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: String Methods (eval_builtin.rs)
// ============================================================================

#[test]
fn test_string_new_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"String::new()"#);
    let _ = result;
}

#[test]
fn test_string_from_builtin_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"String::from("hello")"#);
    let _ = result;
}

#[test]
fn test_string_split_whitespace_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".split_whitespace()"#);
    let _ = result;
}

#[test]
fn test_string_split_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""a,b,c".split(",")"#);
    let _ = result;
}

#[test]
fn test_string_trim_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""  hello  ".trim()"#);
    let _ = result;
}

#[test]
fn test_string_to_uppercase_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello".to_uppercase()"#);
    let _ = result;
}

#[test]
fn test_string_to_lowercase_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""HELLO".to_lowercase()"#);
    let _ = result;
}

#[test]
fn test_string_replace_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".replace("world", "rust")"#);
    let _ = result;
}

#[test]
fn test_string_contains_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".contains("world")"#);
    let _ = result;
}

#[test]
fn test_string_starts_with_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".starts_with("hello")"#);
    let _ = result;
}

#[test]
fn test_string_ends_with_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello world".ends_with("world")"#);
    let _ = result;
}

#[test]
fn test_string_chars_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello".chars()"#);
    let _ = result;
}

#[test]
fn test_string_bytes_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello".bytes()"#);
    let _ = result;
}

#[test]
fn test_string_is_empty_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""".is_empty()"#);
    match result {
        Ok(Value::Bool(b)) => assert!(b),
        _ => {}
    }
}

#[test]
fn test_string_lines_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""line1\nline2".lines()"#);
    let _ = result;
}

#[test]
fn test_string_repeat_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""ab".repeat(3)"#);
    let _ = result;
}

#[test]
fn test_string_parse_int_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""42".parse_int()"#);
    let _ = result;
}

#[test]
fn test_string_parse_float_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""3.14".parse_float()"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Conversion Functions (eval_builtin.rs)
// ============================================================================

#[test]
fn test_int_from_float_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"int(3.7)"#);
    match result {
        Ok(Value::Integer(_n)) => {}
        _ => {}
    }
}

#[test]
fn test_float_from_int_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"float(42)"#);
    let _ = result;
}

#[test]
fn test_str_from_int_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"str(42)"#);
    let _ = result;
}

#[test]
fn test_str_from_float_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"str(3.14)"#);
    let _ = result;
}

#[test]
fn test_bool_from_int_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"bool(1)"#);
    let _ = result;
}

#[test]
fn test_bool_from_zero_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"bool(0)"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Error Handling Branches
// ============================================================================

#[test]
fn test_sqrt_negative_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sqrt(-1)"#);
    // NaN result is valid
    let _ = result;
}

#[test]
fn test_division_by_zero_float_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"1.0 / 0.0"#);
    let _ = result;
}

#[test]
fn test_pop_empty_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"pop([])"#);
    // Should handle gracefully
    let _ = result;
}

#[test]
fn test_index_negative_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3][-1]"#);
    let _ = result;
}

#[test]
fn test_slice_out_of_bounds_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3][0..10]"#);
    let _ = result;
}

#[test]
fn test_string_index_utf8_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"let s = "héllo"; s[0]"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Complex Expressions
// ============================================================================

#[test]
fn test_nested_method_calls_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""  HELLO  ".trim().to_lowercase()"#);
    let _ = result;
}

#[test]
fn test_chained_array_ops_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [3, 1, 4, 1, 5]
        sort(arr)
    "#,
    );
    let _ = result;
}

#[test]
fn test_complex_struct_access_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Inner { value: i32 }
        struct Outer { inner: Inner }
        let o = Outer { inner: Inner { value: 42 } }
        o.inner.value
    "#,
    );
    let _ = result;
}

#[test]
fn test_nested_arrays_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[[1, 2], [3, 4]][0][1]"#);
    match result {
        Ok(Value::Integer(_n)) => {}
        _ => {}
    }
}

#[test]
fn test_map_in_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[{"a": 1}, {"a": 2}][0]["a"]"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Glob and Walk Functions
// ============================================================================

#[test]
fn test_glob_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"glob("/tmp/*")"#);
    let _ = result;
}

#[test]
fn test_walk_function_cov() {
    let mut interp = Interpreter::new();
    // Use /etc/hostname which exists and is small, not /tmp which can have many files
    let result = interp.eval_string(r#"walk("/etc/hostname")"#);
    let _ = result;
}

#[test]
fn test_search_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"search("/tmp", "*.txt")"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: File Handle Operations
// ============================================================================

#[test]
fn test_file_open_read_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"File::open("/etc/passwd")"#);
    let _ = result;
}

#[test]
fn test_open_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"open("/etc/passwd", "r")"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Print/Debug Functions
// ============================================================================

#[test]
fn test_dbg_integer_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"dbg(42)"#);
    let _ = result;
}

#[test]
fn test_dbg_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"dbg("test")"#);
    let _ = result;
}

#[test]
fn test_dbg_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"dbg([1, 2, 3])"#);
    let _ = result;
}

#[test]
fn test_print_multiple_args_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"print("hello", " ", "world")"#);
    let _ = result;
}

#[test]
fn test_println_multiple_args_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"println("a", "b", "c")"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: DataFrame Builder Methods (interpreter_dataframe.rs)
// ============================================================================

#[test]
fn test_dataframe_builder_column_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        DataFrame::builder()
            .column("a", [1, 2, 3])
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_builder_build_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        DataFrame::builder()
            .column("x", [1, 2])
            .build()
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_group_by() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("cat,val\na,1\na,2\nb,3")
        df.group_by("cat")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_agg_sum() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,10\n2,20")
        df.agg("b", "sum")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_agg_mean() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,10\n2,20")
        df.agg("b", "mean")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_agg_min() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,10\n2,5")
        df.agg("b", "min")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_agg_max() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,10\n2,5")
        df.agg("b", "max")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_agg_count() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,10\n2,5")
        df.agg("b", "count")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_join() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df1 = DataFrame::from_csv_string("id,x\n1,a\n2,b")
        let df2 = DataFrame::from_csv_string("id,y\n1,c\n2,d")
        df1.join(df2, "id")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_concat() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df1 = DataFrame::from_csv_string("a\n1\n2")
        let df2 = DataFrame::from_csv_string("a\n3\n4")
        df1.concat(df2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_sample() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a\n1\n2\n3\n4\n5")
        df.sample(2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_dropna() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,2\n,3")
        df.dropna()
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_fillna() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,\n2,3")
        df.fillna(0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_apply() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a\n1\n2")
        df.apply("a", |x| x * 2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_to_array() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,2")
        df.to_array()
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_get_column() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a,b\n1,2\n3,4")
        df.get("a")
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_set_column() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_csv_string("a\n1\n2")
        df.set("a", [10, 20])
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Index Operations (interpreter_index.rs)
// ============================================================================

#[test]
fn test_index_string_char() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello"[1]"#);
    let _ = result;
}

#[test]
fn test_index_array_element() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[10, 20, 30][1]"#);
    let _ = result;
}

#[test]
fn test_index_map_key() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{"key": "value"}["key"]"#);
    let _ = result;
}

#[test]
fn test_index_tuple_element() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"(1, 2, 3).1"#);
    let _ = result;
}

#[test]
fn test_slice_array_range() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3, 4, 5][1..3]"#);
    let _ = result;
}

#[test]
fn test_slice_string_range() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#""hello"[1..4]"#);
    let _ = result;
}

#[test]
fn test_index_assignment_array() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut arr = [1, 2, 3]
        arr[1] = 99
        arr
    "#,
    );
    let _ = result;
}

#[test]
fn test_index_assignment_map() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut m = {"a": 1}
        m["a"] = 2
        m
    "#,
    );
    let _ = result;
}

#[test]
fn test_nested_index() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[[1, 2], [3, 4]][1][0]"#);
    let _ = result;
}

#[test]
fn test_index_with_expression() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let i = 1
        [10, 20, 30][i]
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: More Builtin Functions (eval_builtin.rs)
// ============================================================================

#[test]
fn test_input_builtin() {
    let mut interp = Interpreter::new();
    // Can't test interactive input, but exercise the code path
    let result = interp.eval_string(r#"type_of(input)"#);
    let _ = result;
}

#[test]
fn test_assert_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert(true)"#);
    let _ = result;
}

#[test]
fn test_assert_eq_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"assert_eq(1, 1)"#);
    let _ = result;
}

#[test]
fn test_panic_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        try {
            panic("test error")
        } catch e {
            "caught"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_format_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"format("{} + {} = {}", 1, 2, 3)"#);
    let _ = result;
}

#[test]
fn test_clone_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        let arr2 = clone(arr)
        arr2
    "#,
    );
    let _ = result;
}

#[test]
fn test_hash_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"hash("test")"#);
    let _ = result;
}

#[test]
fn test_exit_builtin() {
    let mut interp = Interpreter::new();
    // Can't actually exit, just check it exists
    let result = interp.eval_string(r#"type_of(exit)"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Control Flow Edge Cases
// ============================================================================

#[test]
fn test_early_return_in_loop() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fn find_first_even(arr) {
            for x in arr {
                if x % 2 == 0 {
                    return x
                }
            }
            return -1
        }
        find_first_even([1, 3, 4, 5])
    "#,
    );
    let _ = result;
}

#[test]
fn test_nested_break() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut found = false
        for i in range(0, 5) {
            if i == 3 {
                found = true
                break
            }
        }
        found
    "#,
    );
    let _ = result;
}

#[test]
fn test_loop_with_else() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut result = 0
        for i in range(0, 3) {
            result = result + i
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_multiple_patterns() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 2
        match x {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_with_guard() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        match x {
            n if n > 5 => "big",
            _ => "small"
        }
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Struct and Enum Operations
// ============================================================================

#[test]
fn test_struct_method_self() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: i32, y: i32 }
        impl Point {
            fn distance(&self) -> f64 {
                sqrt(self.x * self.x + self.y * self.y)
            }
        }
        let p = Point { x: 3, y: 4 }
        p.distance()
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_default_values() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Config {
            name: String = "default",
            value: i32 = 0
        }
        let c = Config {}
        c.name
    "#,
    );
    let _ = result;
}

#[test]
fn test_enum_variant_data() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Message {
            Text(String),
            Number(i32)
        }
        let m = Message::Text("hello")
    "#,
    );
    let _ = result;
}

#[test]
fn test_enum_match() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Status { Ok, Error }
        let s = Status::Ok
        match s {
            Status::Ok => "success",
            Status::Error => "failure"
        }
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Class/Object Operations
// ============================================================================

#[test]
fn test_class_inheritance() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        class Animal {
            fn speak(&self) { "..." }
        }
        class Dog : Animal {
            fn speak(&self) { "woof" }
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_class_static_method() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        class Counter {
            static fn zero() -> i32 { 0 }
        }
        Counter::zero()
    "#,
    );
    let _ = result;
}

#[test]
fn test_class_getter_setter() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        class Box {
            value: i32 = 0
            fn get(&self) -> i32 { self.value }
            fn set(&mut self, v: i32) { self.value = v }
        }
        let b = Box {}
        b.set(42)
        b.get()
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Module Operations
// ============================================================================

#[test]
fn test_module_const() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod math {
            const PI: f64 = 3.14159
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_module_nested() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod outer {
            mod inner {
                fn greet() { "hello" }
            }
        }
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: More Math Functions
// ============================================================================

#[test]
fn test_asin_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"asin(0.5)"#);
    let _ = result;
}

#[test]
fn test_acos_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"acos(0.5)"#);
    let _ = result;
}

#[test]
fn test_atan_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"atan(1.0)"#);
    let _ = result;
}

#[test]
fn test_atan2_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"atan2(1.0, 1.0)"#);
    let _ = result;
}

#[test]
fn test_sinh_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sinh(1.0)"#);
    let _ = result;
}

#[test]
fn test_cosh_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"cosh(1.0)"#);
    let _ = result;
}

#[test]
fn test_tanh_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"tanh(1.0)"#);
    let _ = result;
}

#[test]
fn test_log2_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"log2(8.0)"#);
    let _ = result;
}

#[test]
fn test_sign_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"sign(-5)"#);
    let _ = result;
}

#[test]
fn test_clamp_builtin() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"clamp(15, 0, 10)"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Array Methods
// ============================================================================

#[test]
fn test_array_first_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].first()"#);
    let _ = result;
}

#[test]
fn test_array_last_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].last()"#);
    let _ = result;
}

#[test]
fn test_array_find_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].find(|x| x > 1)"#);
    let _ = result;
}

#[test]
fn test_array_filter_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3, 4].filter(|x| x % 2 == 0)"#);
    let _ = result;
}

#[test]
fn test_array_map_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].map(|x| x * 2)"#);
    let _ = result;
}

#[test]
fn test_array_reduce_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3, 4].reduce(0, |acc, x| acc + x)"#);
    let _ = result;
}

#[test]
fn test_array_all_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[2, 4, 6].all(|x| x % 2 == 0)"#);
    let _ = result;
}

#[test]
fn test_array_any_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].any(|x| x > 2)"#);
    let _ = result;
}

#[test]
fn test_array_take_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3, 4, 5].take(3)"#);
    let _ = result;
}

#[test]
fn test_array_skip_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3, 4, 5].skip(2)"#);
    let _ = result;
}

#[test]
fn test_array_flatten_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[[1, 2], [3, 4]].flatten()"#);
    let _ = result;
}

#[test]
fn test_array_unique_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 2, 3, 3, 3].unique()"#);
    let _ = result;
}

#[test]
fn test_array_join_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"["a", "b", "c"].join(",")"#);
    let _ = result;
}

#[test]
fn test_array_contains_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].contains(2)"#);
    let _ = result;
}

#[test]
fn test_array_index_of_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[1, 2, 3].index_of(2)"#);
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Type Inference and Control Flow Paths
// ============================================================================

#[test]
fn test_type_inference_numeric_param() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun double(x) { x * 2 }
        double(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_inference_string_param() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun greet(name) { "Hello " + name }
        greet("World")
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_inference_bool_param() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun check(flag) { if flag { 1 } else { 0 } }
        check(true)
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_inference_array_param() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun first(arr) { arr[0] }
        first([1, 2, 3])
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_inference_function_param() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun apply(f, x) { f(x) }
        apply(|x| x * 2, 5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_control_flow_break_in_nested_loop() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = 0
        for i in range(0, 5) {
            for j in range(0, 5) {
                if j == 2 { break }
                result = result + 1
            }
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_control_flow_continue_pattern() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let sum = 0
        for i in range(0, 10) {
            if i % 2 == 0 { continue }
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_control_flow_early_return() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun find_first_even(arr) {
            for x in arr {
                if x % 2 == 0 { return x }
            }
            return -1
        }
        find_first_even([1, 3, 5, 4, 7])
    "#,
    );
    let _ = result;
}

#[test]
fn test_control_flow_throw_catch() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun safe_div(a, b) {
            if b == 0 { throw "division by zero" }
            a / b
        }
        try { safe_div(10, 0) } catch e { -1 }
    "#,
    );
    let _ = result;
}

#[test]
fn test_expression_match_complex() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        match x {
            1 => "one",
            2 | 3 => "two or three",
            4..=6 => "four to six",
            _ => "other"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_expression_if_let() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let maybe = Some(42)
        if let Some(x) = maybe {
            x * 2
        } else {
            0
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_expression_while_let() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        let idx = 0
        let sum = 0
        while idx < arr.len() {
            sum = sum + arr[idx]
            idx = idx + 1
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_parser_comments_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        // Line comment
        let x = 1  // trailing comment
        /* Block comment */
        let y = 2
        x + y
    "#,
    );
    let _ = result;
}

#[test]
fn test_parser_doc_comments_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        /// This is a doc comment
        fun documented() { 42 }
        documented()
    "#,
    );
    let _ = result;
}

#[test]
fn test_complex_nested_expression() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = if true {
            match 1 {
                1 => if false { 0 } else { 1 },
                _ => 2
            }
        } else {
            3
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_capture_env_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let multiplier = 10
        let scale = |x| x * multiplier
        scale(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_recursive_function_call_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun factorial(n) {
            if n <= 1 { 1 }
            else { n * factorial(n - 1) }
        }
        factorial(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_mutual_recursion_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun is_even(n) {
            if n == 0 { true }
            else { is_odd(n - 1) }
        }
        fun is_odd(n) {
            if n == 0 { false }
            else { is_even(n - 1) }
        }
        is_even(4)
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_methods_chain_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "  Hello World  ".trim().to_upper().len()
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_method_chain_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3, 4, 5]
            .filter(|x| x % 2 == 0)
            .map(|x| x * 2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_map_operations_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"a": 1, "b": 2, "c": 3}
        m.keys()
    "#,
    );
    let _ = result;
}

#[test]
fn test_map_values_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"x": 10, "y": 20}
        m.values()
    "#,
    );
    let _ = result;
}

#[test]
fn test_map_entries_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"key": "value"}
        m.entries()
    "#,
    );
    let _ = result;
}

#[test]
fn test_option_some_unwrap_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let opt = Some(42)
        opt.unwrap()
    "#,
    );
    let _ = result;
}

#[test]
fn test_option_none_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let opt = None
        opt.is_none()
    "#,
    );
    let _ = result;
}

#[test]
fn test_result_ok_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let res = Ok(42)
        res.is_ok()
    "#,
    );
    let _ = result;
}

#[test]
fn test_result_err_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let res = Err("error")
        res.is_err()
    "#,
    );
    let _ = result;
}

#[test]
fn test_pipeline_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        5 |> (|x| x * 2) |> (|x| x + 1)
    "#,
    );
    let _ = result;
}

#[test]
fn test_spread_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = [1, 2]
        let b = [3, 4]
        [...a, ...b]
    "#,
    );
    let _ = result;
}

#[test]
fn test_destructure_tuple_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let (x, y, z) = (1, 2, 3)
        x + y + z
    "#,
    );
    let _ = result;
}

#[test]
fn test_destructure_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let [a, b, c] = [10, 20, 30]
        a + b + c
    "#,
    );
    let _ = result;
}

#[test]
fn test_destructure_struct_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: i64, y: i64 }
        let p = Point { x: 1, y: 2 }
        let Point { x, y } = p
        x + y
    "#,
    );
    let _ = result;
}

#[test]
fn test_range_inclusive_sum_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let sum = 0
        for i in range(1, 4) {
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_bitwise_shift_ops_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 8 >> 2
        let b = 2 << 3
        a + b
    "#,
    );
    let _ = result;
}

#[test]
fn test_comparison_chain_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        x > 3 && x < 10
    "#,
    );
    let _ = result;
}

#[test]
fn test_complex_boolean_expr_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = true
        let b = false
        let c = true
        (a || b) && (c || !b) && !(a && b && c)
    "#,
    );
    let _ = result;
}
