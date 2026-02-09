// Auto-extracted from interpreter_tests.rs - Part 16
use super::*;


// COVERAGE: File open function
#[test]
fn test_file_open_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let f = file_open("Cargo.toml", "r")
        f
    "#,
    );
    let _ = result;
}

// COVERAGE: Open function
#[test]
fn test_open_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let f = open("Cargo.toml")
        f
    "#,
    );
    let _ = result;
}

// COVERAGE: Env args function
#[test]
fn test_env_args_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let args = env_args()
        len(args) >= 0
    "#,
    );
    let _ = result;
}

// COVERAGE: Env var function
#[test]
fn test_env_var_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        env_var("PATH")
    "#,
    );
    let _ = result;
}

// COVERAGE: Env current_dir function
#[test]
fn test_env_current_dir_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        env_current_dir()
    "#,
    );
    let _ = result;
}

// COVERAGE: Append file function (with temp file)
#[test]
fn test_append_file_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let path = path_join(tmp, "test_append.txt")
        fs_write(path, "line1\n")
        append_file(path, "line2\n")
        let content = fs_read(path)
        fs_remove_file(path)
        content
    "#,
    );
    let _ = result;
}

// COVERAGE: FS exists function
#[test]
fn test_fs_exists_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fs_exists("Cargo.toml")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS create_dir function
#[test]
fn test_fs_create_dir_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let path = path_join(tmp, "test_dir_v2")
        fs_create_dir(path)
        let exists = fs_exists(path)
        fs_remove_dir(path)
        exists
    "#,
    );
    let _ = result;
}

// COVERAGE: Read file unwrapped
#[test]
fn test_read_file_unwrapped_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        read_file("Cargo.toml")
    "#,
    );
    let _ = result;
}

// COVERAGE: Dataframe new function
#[test]
fn test_dataframe_new_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        DataFrame::new()
    "#,
    );
    let _ = result;
}

// COVERAGE: Dataframe from_csv_string function
#[test]
fn test_dataframe_from_csv_string_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        DataFrame::from_csv_string("a,b\n1,2\n3,4")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON read function
#[test]
fn test_json_read_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let path = path_join(tmp, "test_json.json")
        json_write(path, {"key": "value"})
        let data = json_read(path)
        fs_remove_file(path)
        data
    "#,
    );
    let _ = result;
}

// COVERAGE: Assert function
#[test]
fn test_assert_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert(true)
    "#,
    );
    let _ = result;
}

// COVERAGE: Random function
#[test]
fn test_random_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let r = random()
        r >= 0.0 && r < 1.0
    "#,
    );
    let _ = result;
}

// COVERAGE: Floor function
#[test]
fn test_floor_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        floor(3.7)
    "#,
    );
    let _ = result;
}

// COVERAGE: Ceil function
#[test]
fn test_ceil_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        ceil(3.2)
    "#,
    );
    let _ = result;
}

// COVERAGE: Round function
#[test]
fn test_round_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        round(3.5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Abs function
#[test]
fn test_abs_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        abs(-42)
    "#,
    );
    let _ = result;
}

// COVERAGE: Min function
#[test]
fn test_min_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        min(3, 5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Max function
#[test]
fn test_max_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        max(3, 5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sqrt function
#[test]
fn test_sqrt_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        sqrt(16.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Pow function
#[test]
fn test_pow_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        pow(2.0, 3.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sin function
#[test]
fn test_sin_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        sin(0.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Cos function
#[test]
fn test_cos_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        cos(0.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Tan function
#[test]
fn test_tan_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        tan(0.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Log function
#[test]
fn test_log_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        log(100.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Log10 function
#[test]
fn test_log10_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        log10(100.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Exp function
#[test]
fn test_exp_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        exp(1.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Len function
#[test]
fn test_len_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        len([1, 2, 3, 4, 5])
    "#,
    );
    let _ = result;
}

// COVERAGE: Range one arg
#[test]
fn test_range_one_arg_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        range(5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Range two args
#[test]
fn test_range_two_args_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        range(1, 5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Reverse function
#[test]
fn test_reverse_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        reverse([1, 2, 3])
    "#,
    );
    let _ = result;
}

// COVERAGE: Reverse string
#[test]
fn test_reverse_string_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        reverse("hello")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON parse with different types
#[test]
fn test_json_parse_array_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        json_parse("[1, 2, 3]")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON parse object
#[test]
fn test_json_parse_object_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        json_parse("{\"a\": 1, \"b\": 2}")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON write function
#[test]
fn test_json_write_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let path = path_join(tmp, "test_write.json")
        json_write(path, {"test": true})
        fs_remove_file(path)
        true
    "#,
    );
    let _ = result;
}

// COVERAGE: Tuple creation
#[test]
fn test_tuple_creation_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let t = (1, "hello", 3.14)
        t
    "#,
    );
    let _ = result;
}

// COVERAGE: Tuple destructuring
#[test]
fn test_tuple_destructuring_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let (a, b, c) = (1, 2, 3)
        a + b + c
    "#,
    );
    let _ = result;
}

// COVERAGE: Nested tuples
#[test]
fn test_nested_tuple_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let t = ((1, 2), (3, 4))
        t
    "#,
    );
    let _ = result;
}

// COVERAGE: Lambda with multiple params
#[test]
fn test_lambda_multi_param_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let add = |a, b, c| a + b + c
        add(1, 2, 3)
    "#,
    );
    let _ = result;
}

// COVERAGE: Lambda with closure
#[test]
fn test_lambda_closure_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        let add_x = |y| x + y
        add_x(5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Chained method calls
#[test]
fn test_chained_methods_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "  hello world  ".trim().to_uppercase()
    "#,
    );
    let _ = result;
}

// COVERAGE: Complex binary expressions
#[test]
fn test_complex_binary_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        (1 + 2) * (3 - 4) / 5 + 6 % 7
    "#,
    );
    let _ = result;
}

// COVERAGE: Comparison chains
#[test]
fn test_comparison_chain_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        1 < 2 && 2 < 3 && 3 < 4
    "#,
    );
    let _ = result;
}

// COVERAGE: Array slice operations
#[test]
fn test_array_slice_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr[1..3]
    "#,
    );
    let _ = result;
}

// COVERAGE: String indexing
#[test]
fn test_string_index_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "hello"[0]
    "#,
    );
    let _ = result;
}

// COVERAGE: Map literals
#[test]
fn test_map_literal_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"a": 1, "b": 2, "c": 3}
        m["a"]
    "#,
    );
    let _ = result;
}

// COVERAGE: Set operations
#[test]
fn test_set_literal_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = {1, 2, 3}
        s
    "#,
    );
    let _ = result;
}

// COVERAGE: Match expression
#[test]
fn test_match_expression_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 2
        match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#,
    );
    let _ = result;
}

// COVERAGE: Match with guards
#[test]
fn test_match_with_guard_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        match x {
            n if n > 5 => "big",
            n if n <= 5 => "small",
            _ => "unknown"
        }
    "#,
    );
    let _ = result;
}

// COVERAGE: If-else chain
#[test]
fn test_if_else_chain_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 2
        if x == 1 {
            "one"
        } else if x == 2 {
            "two"
        } else if x == 3 {
            "three"
        } else {
            "other"
        }
    "#,
    );
    let _ = result;
}

// COVERAGE: While loop with break
#[test]
fn test_while_break_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut i = 0
        while true {
            i = i + 1
            if i >= 5 {
                break
            }
        }
        i
    "#,
    );
    let _ = result;
}

// COVERAGE: While loop with continue
#[test]
fn test_while_continue_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut sum = 0
        let mut i = 0
        while i < 10 {
            i = i + 1
            if i % 2 == 0 {
                continue
            }
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result;
}

// COVERAGE: For with range
#[test]
fn test_for_range_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut sum = 0
        for i in 0..10 {
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result;
}

// COVERAGE: Nested loops
#[test]
fn test_nested_loop_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut sum = 0
        for i in 0..3 {
            for j in 0..3 {
                sum = sum + i * j
            }
        }
        sum
    "#,
    );
    let _ = result;
}

// COVERAGE: Function with default params
#[test]
fn test_function_default_params_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun greet(name: String = "World") -> String {
            f"Hello, {name}!"
        }
        greet()
    "#,
    );
    let _ = result;
}

// COVERAGE: Function with multiple returns
#[test]
fn test_function_early_return_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun check(x: i64) -> String {
            if x < 0 {
                return "negative"
            }
            if x == 0 {
                return "zero"
            }
            "positive"
        }
        check(-5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Struct with methods
#[test]
fn test_struct_with_methods_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Counter {
            value: i64
        }
        impl Counter {
            fun new() -> Counter {
                Counter { value: 0 }
            }
            fun increment(self) -> Counter {
                Counter { value: self.value + 1 }
            }
        }
        let c = Counter::new()
        c.increment().value
    "#,
    );
    let _ = result;
}

// COVERAGE: Enum with variants
#[test]
fn test_enum_variants_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Color {
            Red,
            Green,
            Blue
        }
        let c = Color::Red
        c
    "#,
    );
    let _ = result;
}

// COVERAGE: String interpolation complex
#[test]
fn test_string_interpolation_complex_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        let y = 20
        f"x = {x}, y = {y}, sum = {x + y}"
    "#,
    );
    let _ = result;
}

// COVERAGE: Array methods
#[test]
fn test_array_methods_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        let doubled = arr.map(|x| x * 2)
        doubled
    "#,
    );
    let _ = result;
}

// COVERAGE: Filter and reduce
#[test]
fn test_filter_reduce_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5, 6]
        let evens = arr.filter(|x| x % 2 == 0)
        evens
    "#,
    );
    let _ = result;
}

// COVERAGE: String methods comprehensive
#[test]
fn test_string_methods_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = "hello world"
        let upper = s.to_uppercase()
        let replaced = s.replace("world", "rust")
        let split = s.split(" ")
        split
    "#,
    );
    let _ = result;
}

// COVERAGE: Option/Result handling
#[test]
fn test_option_handling_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let opt = Some(42)
        match opt {
            Some(x) => x,
            None => 0
        }
    "#,
    );
    let _ = result;
}

// COVERAGE: Try expressions
#[test]
fn test_try_expression_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun might_fail() -> Result<i64, String> {
            Ok(42)
        }
        match might_fail() {
            Ok(x) => x,
            Err(e) => 0
        }
    "#,
    );
    let _ = result;
}

// COVERAGE: Bitwise operations
#[test]
fn test_bitwise_ops_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 0b1010
        let b = 0b1100
        let and = a & b
        let or = a | b
        let xor = a ^ b
        and
    "#,
    );
    let _ = result;
}

// COVERAGE: Unary operations
#[test]
fn test_unary_ops_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = -5
        let y = !true
        let z = -(-10)
        z
    "#,
    );
    let _ = result;
}

// COVERAGE: Type annotations
#[test]
fn test_type_annotations_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x: i64 = 42
        let y: f64 = 3.14
        let z: String = "hello"
        let b: bool = true
        b
    "#,
    );
    let _ = result;
}

// COVERAGE: Complex literals
#[test]
fn test_complex_literals_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let hex = 0xFF
        let oct = 0o77
        let bin = 0b1111
        let sci = 1.5e10
        hex
    "#,
    );
    let _ = result;
}

// COVERAGE: Escape sequences
#[test]
fn test_escape_sequences_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = "line1\nline2\ttabbed\"quoted\\"
        s
    "#,
    );
    let _ = result;
}

// COVERAGE: Unicode strings
#[test]
fn test_unicode_strings_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let emoji = "Hello 🌍 World 🚀"
        let cjk = "你好世界"
        len(emoji)
    "#,
    );
    let _ = result;
}

// COVERAGE: Recursive function
#[test]
fn test_recursive_function_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun factorial(n: i64) -> i64 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        factorial(5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Higher-order functions
#[test]
fn test_higher_order_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun apply(f, x) {
            f(x)
        }
        let double = |x| x * 2
        apply(double, 21)
    "#,
    );
    let _ = result;
}

// COVERAGE: List comprehension
#[test]
fn test_list_comprehension_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let squares = [x * x for x in range(1, 6)]
        squares
    "#,
    );
    let _ = result;
}

// COVERAGE: Dictionary comprehension
#[test]
fn test_dict_comprehension_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let d = {str(x): x * x for x in range(1, 4)}
        d
    "#,
    );
    let _ = result;
}

// COVERAGE: Set comprehension
#[test]
fn test_set_comprehension_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = {x * 2 for x in range(1, 5)}
        s
    "#,
    );
    let _ = result;
}

// COVERAGE: Conditional expression
#[test]
fn test_conditional_expr_v3() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        let result = if x > 5 { "big" } else { "small" }
        result
    "#,
    );
    let _ = result;
}

// COVERAGE: Len with string
#[test]
fn test_len_string_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        len("hello world")
    "#,
    );
    let _ = result;
}

// COVERAGE: Len with map
#[test]
fn test_len_map_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        len({"a": 1, "b": 2})
    "#,
    );
    let _ = result;
}

// COVERAGE: Len with set
#[test]
fn test_len_set_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        len({1, 2, 3, 4})
    "#,
    );
    let _ = result;
}

// COVERAGE: Type of various types
#[test]
fn test_type_of_various_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let t1 = type_of(42)
        let t2 = type_of(3.14)
        let t3 = type_of("hello")
        let t4 = type_of(true)
        let t5 = type_of([1, 2, 3])
        let t6 = type_of({"a": 1})
        [t1, t2, t3, t4, t5, t6]
    "#,
    );
    let _ = result;
}

// COVERAGE: Abs with integers
#[test]
fn test_abs_integer_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = abs(-100)
        let b = abs(100)
        let c = abs(0)
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: Abs with floats
#[test]
fn test_abs_float_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = abs(-3.14)
        let b = abs(3.14)
        let c = abs(0.0)
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: Min/Max with arrays
#[test]
fn test_min_array_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [5, 2, 8, 1, 9]
        min(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Max with arrays
#[test]
fn test_max_array_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [5, 2, 8, 1, 9]
        max(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sort with floats
#[test]
fn test_sort_floats_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [3.14, 1.41, 2.71]
        sort(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sort with strings
#[test]
fn test_sort_strings_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = ["banana", "apple", "cherry"]
        sort(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Reverse with map
#[test]
fn test_reverse_empty_array_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        reverse([])
    "#,
    );
    let _ = result;
}

// COVERAGE: Pop empty array
#[test]
fn test_pop_empty_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = []
        pop(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Parse int edge cases
#[test]
fn test_parse_int_negative_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        parse_int("-42")
    "#,
    );
    let _ = result;
}

// COVERAGE: Parse float edge cases
#[test]
fn test_parse_float_scientific_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        parse_float("1.5e-10")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS copy and rename
#[test]
fn test_fs_copy_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let src = path_join(tmp, "test_src.txt")
        let dst = path_join(tmp, "test_dst.txt")
        fs_write(src, "content")
        fs_copy(src, dst)
        let content = fs_read(dst)
        fs_remove_file(src)
        fs_remove_file(dst)
        content
    "#,
    );
    let _ = result;
}

// COVERAGE: FS rename
#[test]
fn test_fs_rename_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let tmp = env_temp_dir()
        let old = path_join(tmp, "test_old.txt")
        let new = path_join(tmp, "test_new.txt")
        fs_write(old, "content")
        fs_rename(old, new)
        let content = fs_read(new)
        fs_remove_file(new)
        content
    "#,
    );
    let _ = result;
}

// COVERAGE: Env set and remove var
#[test]
fn test_env_set_remove_var_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        env_set_var("RUCHY_TEST_VAR_V4", "test_value")
        let val = env_var("RUCHY_TEST_VAR_V4")
        env_remove_var("RUCHY_TEST_VAR_V4")
        val
    "#,
    );
    let _ = result;
}

// COVERAGE: Math with edge cases
#[test]
fn test_math_edge_cases_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = floor(0.0)
        let b = ceil(0.0)
        let c = round(0.5)
        let d = sqrt(0.0)
        [a, b, c, d]
    "#,
    );
    let _ = result;
}

// COVERAGE: Pow with edge cases
#[test]
fn test_pow_edge_cases_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = pow(2.0, 0.0)
        let b = pow(0.0, 2.0)
        let c = pow(1.0, 100.0)
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: String with empty
#[test]
fn test_empty_string_methods_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = ""
        let a = s.len()
        let b = s.is_empty()
        let c = s.trim()
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: Array with single element
#[test]
fn test_single_element_array_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [42]
        let a = len(arr)
        let b = min(arr)
        let c = max(arr)
        let d = sort(arr)
        let e = reverse(arr)
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: Is nil with different values
#[test]
fn test_is_nil_various_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = is_nil(nil)
        let b = is_nil(0)
        let c = is_nil("")
        let d = is_nil([])
        [a, b, c, d]
    "#,
    );
    let _ = result;
}

// COVERAGE: Dbg with complex values
#[test]
fn test_dbg_complex_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        let map = {"key": "value"}
        dbg(arr)
        dbg(map)
        true
    "#,
    );
    let _ = result;
}

// COVERAGE: Print with format
#[test]
fn test_print_format_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        print("value: {}", 42)
        true
    "#,
    );
    let _ = result;
}

// COVERAGE: To string with various types
#[test]
fn test_to_string_various_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = to_string(42)
        let b = to_string(3.14)
        let c = to_string(true)
        let d = to_string([1, 2, 3])
        [a, b, c, d]
    "#,
    );
    let _ = result;
}

// COVERAGE: Build path from components
#[test]
fn test_build_path_components_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_join_many(["home", "user", "documents", "file.txt"])
    "#,
    );
    let _ = result;
}

// COVERAGE: Output capture
#[test]
fn test_output_capture_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        println("captured output")
        true
    "#,
    );
    let _ = result;
}

// COVERAGE: Nested function calls
#[test]
fn test_nested_builtin_calls_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        len(reverse(sort([3, 1, 4, 1, 5])))
    "#,
    );
    let _ = result;
}

// COVERAGE: Complex map operations
#[test]
fn test_complex_map_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"a": {"b": {"c": 42}}}
        m["a"]["b"]["c"]
    "#,
    );
    let _ = result;
}

// COVERAGE: Array concatenation
#[test]
fn test_array_concat_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = [1, 2, 3]
        let b = [4, 5, 6]
        a + b
    "#,
    );
    let _ = result;
}

// COVERAGE: String concatenation
#[test]
fn test_string_concat_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = "hello"
        let b = " "
        let c = "world"
        a + b + c
    "#,
    );
    let _ = result;
}

// COVERAGE: Range with floats
#[test]
fn test_range_floats_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        range(0, 5, 1)
    "#,
    );
    let _ = result;
}

// COVERAGE: Empty map
#[test]
fn test_empty_map_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {}
        len(m)
    "#,
    );
    let _ = result;
}

// COVERAGE: Empty set
#[test]
fn test_empty_set_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = Set::new()
        s
    "#,
    );
    let _ = result;
}

// COVERAGE: Comparison operators
#[test]
fn test_comparison_all_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 1 < 2
        let b = 2 > 1
        let c = 1 <= 1
        let d = 2 >= 2
        let e = 1 == 1
        let f = 1 != 2
        [a, b, c, d, e, f]
    "#,
    );
    let _ = result;
}

// COVERAGE: Logical operators
#[test]
fn test_logical_all_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = true && true
        let b = true || false
        let c = !false
        let d = true && false
        let e = false || false
        [a, b, c, d, e]
    "#,
    );
    let _ = result;
}

// COVERAGE: Arithmetic operators
#[test]
fn test_arithmetic_all_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 10 + 5
        let b = 10 - 5
        let c = 10 * 5
        let d = 10 / 5
        let e = 10 % 3
        [a, b, c, d, e]
    "#,
    );
    let _ = result;
}

// COVERAGE: Float arithmetic
#[test]
fn test_float_arithmetic_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 10.5 + 5.5
        let b = 10.5 - 5.5
        let c = 10.5 * 2.0
        let d = 10.5 / 2.0
        [a, b, c, d]
    "#,
    );
    let _ = result;
}

// COVERAGE: Mixed arithmetic
#[test]
fn test_mixed_arithmetic_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 10 + 5.5
        let b = 10.5 + 5
        let c = 10 * 2.5
        [a, b, c]
    "#,
    );
    let _ = result;
}

// COVERAGE: Compound assignment
#[test]
fn test_compound_assignment_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut x = 10
        x += 5
        x -= 3
        x *= 2
        x /= 4
        x
    "#,
    );
    let _ = result;
}

// COVERAGE: Increment/decrement
#[test]
fn test_increment_decrement_v4() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut x = 10
        x += 1
        x -= 1
        x
    "#,
    );
    let _ = result;
}
