// Auto-extracted from interpreter_tests.rs - Part 14
use super::*;

#[test]
fn test_string_escape_sequences_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = "line1\nline2\ttab"
        s.len()
    "#,
    );
    let _ = result;
}

#[test]
fn test_raw_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = r"no\nescape"
        s
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_interpolation_nested_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        let y = 10
        "{x} + {y} = {x + y}"
    "#,
    );
    let _ = result;
}

#[test]
fn test_multiline_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = "line 1
line 2
line 3"
        s.lines()
    "#,
    );
    let _ = result;
}

#[test]
fn test_float_operations_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 3.14
        let b = 2.71
        a * b + a / b - a
    "#,
    );
    let _ = result;
}

#[test]
fn test_float_comparison_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 1.0 / 3.0
        x > 0.33 && x < 0.34
    "#,
    );
    let _ = result;
}

#[test]
fn test_negative_numbers_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = -5
        let b = -3.14
        a + b
    "#,
    );
    let _ = result;
}

#[test]
fn test_unary_not_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = true
        !a
    "#,
    );
    let _ = result;
}

#[test]
fn test_conditional_assignment_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = if true { 1 } else { 0 }
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_block_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = {
            let a = 1
            let b = 2
            a + b
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_empty_array_methods_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = []
        arr.is_empty()
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_push_pop_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        arr.push(4)
        arr.pop()
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_reverse_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3].reverse()
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_sort_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [3, 1, 2].sort()
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_zip_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2].zip([3, 4])
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_enumerate_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        ["a", "b", "c"].enumerate()
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_split_csv_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "a,b,c".split(",")
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_replace_word_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "hello world".replace("world", "universe")
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_starts_ends_with_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = "hello"
        s.starts_with("he") && s.ends_with("lo")
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_repeat_times_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "ab".repeat(3)
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_pad_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "5".pad_start(3, "0")
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_find_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        "hello".find("ll")
    "#,
    );
    let _ = result;
}

#[test]
fn test_math_pow_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        pow(2.0, 10.0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_math_sqrt_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        sqrt(16.0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_math_exp_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        exp(1.0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_math_log_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        log(10.0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_math_log10_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        log10(100.0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_of_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type_of(42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_of_string_literal_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type_of("hello")
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_of_array_literal_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type_of([1, 2, 3])
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_of_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type_of(|x| x)
    "#,
    );
    let _ = result;
}

#[test]
fn test_assert_eq_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert_eq(1, 1)
    "#,
    );
    let _ = result;
}

#[test]
fn test_assert_ne_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert_ne(1, 2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_dbg_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        dbg(42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_env_get_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        env_get("PATH")
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_default_values_init_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Config {
            debug: bool = false,
            level: i64 = 1
        }
        let c = Config {}
        c.debug
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Counter { count: i64 }
        impl Counter {
            fun new() { Counter { count: 0 } }
            fun increment(self) { self.count = self.count + 1 }
        }
        let c = Counter::new()
        c.increment()
        c.count
    "#,
    );
    let _ = result;
}

#[test]
fn test_enum_with_fields_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Message {
            Text(String),
            Number(i64)
        }
        let m = Message::Text("hello")
        m
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_enum_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Color { Red, Green, Blue }
        let c = Color::Red
        match c {
            Color::Red => "red",
            Color::Green => "green",
            Color::Blue => "blue"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_for_in_map_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"a": 1, "b": 2}
        let sum = 0
        for (k, v) in m {
            sum = sum + v
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_while_with_assignment_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let i = 0
        while (i = i + 1) < 5 {
            i
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_loop_with_counter_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let i = 0
        loop {
            i = i + 1
            if i >= 3 { break }
        }
        i
    "#,
    );
    let _ = result;
}

#[test]
fn test_lambda_with_multiple_params_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let add = |a, b, c| a + b + c
        add(1, 2, 3)
    "#,
    );
    let _ = result;
}

#[test]
fn test_lambda_no_params_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let get_val = || 42
        get_val()
    "#,
    );
    let _ = result;
}

#[test]
fn test_nested_function_calls_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun a(x) { x + 1 }
        fun b(x) { a(x) * 2 }
        fun c(x) { b(x) - 1 }
        c(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_default_function_params_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun greet(name = "World") { "Hello " + name }
        greet()
    "#,
    );
    let _ = result;
}

#[test]
fn test_variadic_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun sum(...nums) {
            let total = 0
            for n in nums { total = total + n }
            total
        }
        sum(1, 2, 3, 4, 5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_expression_in_condition_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        if arr.len() > 2 { "long" } else { "short" }
    "#,
    );
    let _ = result;
}

#[test]
fn test_chained_comparisons_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        1 < x && x < 10
    "#,
    );
    let _ = result;
}

#[test]
fn test_complex_arithmetic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        (1 + 2) * (3 - 4) / (5 + 1) % 3
    "#,
    );
    let _ = result;
}

#[test]
fn test_assignment_operators_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        x += 5
        x -= 3
        x *= 2
        x /= 4
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_deeply_nested_arrays_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [[[1, 2], [3, 4]], [[5, 6], [7, 8]]]
        arr[0][1][0]
    "#,
    );
    let _ = result;
}

#[test]
fn test_map_nested_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = {"outer": {"inner": 42}}
        m["outer"]["inner"]
    "#,
    );
    let _ = result;
}

#[test]
fn test_mixed_collection_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let data = [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]
        data[0]["name"]
    "#,
    );
    let _ = result;
}

#[test]
fn test_iterator_fold_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3, 4].fold(0, |acc, x| acc + x)
    "#,
    );
    let _ = result;
}

#[test]
fn test_iterator_scan_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3].scan(0, |acc, x| acc + x)
    "#,
    );
    let _ = result;
}

#[test]
fn test_iterator_partition_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3, 4, 5].partition(|x| x % 2 == 0)
    "#,
    );
    let _ = result;
}

#[test]
fn test_iterator_group_by_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        [1, 2, 3, 4, 5, 6].group_by(|x| x % 3)
    "#,
    );
    let _ = result;
}

#[test]
fn test_try_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun may_fail() { throw "error" }
        try { may_fail() } catch e { "caught" }
    "#,
    );
    let _ = result;
}

#[test]
fn test_try_finally_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let cleanup = false
        try { 1 } finally { cleanup = true }
        cleanup
    "#,
    );
    let _ = result;
}

// ============================================================================
// COVERAGE IMPROVEMENT: Parser and Collection Edge Cases
// ============================================================================

#[test]
fn test_set_literal_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let s = {1, 2, 3, 2, 1}
        s.len()
    "#,
    );
    let _ = result;
}

#[test]
fn test_set_operations_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = {1, 2, 3}
        let b = {2, 3, 4}
        a.contains(2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_tuple_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let t = (1, "hello", true)
        match t {
            (x, s, b) => x
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4]
        match arr {
            [first, ..rest] => first,
            _ => 0
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: i64, y: i64 }
        let p = Point { x: 10, y: 20 }
        match p {
            Point { x: 10, y } => y,
            _ => 0
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_guard_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        match x {
            n if n > 10 => "big",
            n if n > 0 => "positive",
            _ => "other"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_or_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 2
        match x {
            1 | 2 | 3 => "small",
            _ => "other"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_range_patterns_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        match x {
            0..=5 => "low",
            6..=10 => "mid",
            _ => "high"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_async_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        async fun fetch_data() {
            42
        }
        fetch_data()
    "#,
    );
    let _ = result;
}

#[test]
fn test_await_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        async fun get_value() { 100 }
        async fun main() {
            let v = await get_value()
            v
        }
        main()
    "#,
    );
    let _ = result;
}

#[test]
fn test_use_statement_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        use std::collections::HashMap
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_type_alias_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type IntList = [i64]
        let x: IntList = [1, 2, 3]
        x.len()
    "#,
    );
    let _ = result;
}

#[test]
fn test_const_declaration_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        const PI = 3.14159
        const MAX = 100
        PI + MAX
    "#,
    );
    let _ = result;
}

#[test]
fn test_static_declaration_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        static COUNTER = 0
        COUNTER
    "#,
    );
    let _ = result;
}

#[test]
fn test_trait_definition_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        trait Printable {
            fun print(self)
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_impl_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: i64, y: i64 }
        trait Display {
            fun display(self) -> String
        }
        impl Display for Point {
            fun display(self) -> String {
                "{self.x}, {self.y}"
            }
        }
        let p = Point { x: 1, y: 2 }
        p.display()
    "#,
    );
    let _ = result;
}

#[test]
fn test_generic_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun identity<T>(x: T) -> T { x }
        identity(42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_generic_struct_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Box<T> { value: T }
        let b = Box { value: 42 }
        b.value
    "#,
    );
    let _ = result;
}

#[test]
fn test_where_clause_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun process<T>(x: T) -> T where T: Clone { x }
        process(1)
    "#,
    );
    let _ = result;
}

#[test]
fn test_lifetime_annotation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun longest<'a>(x: &'a str, y: &'a str) -> &'a str {
            if x.len() > y.len() { x } else { y }
        }
        longest("hello", "world")
    "#,
    );
    let _ = result;
}

#[test]
fn test_slice_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr[1..3]
    "#,
    );
    let _ = result;
}

#[test]
fn test_slice_from_start_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr[..3]
    "#,
    );
    let _ = result;
}

#[test]
fn test_slice_to_end_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr[2..]
    "#,
    );
    let _ = result;
}

#[test]
fn test_reference_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 42
        let r = &x
        *r
    "#,
    );
    let _ = result;
}

#[test]
fn test_mutable_reference_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 42
        let r = &mut x
        *r = 100
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_box_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let b = Box::new(42)
        *b
    "#,
    );
    let _ = result;
}

#[test]
fn test_rc_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let r = Rc::new(42)
        Rc::strong_count(&r)
    "#,
    );
    let _ = result;
}

#[test]
fn test_arc_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = Arc::new(42)
        Arc::strong_count(&a)
    "#,
    );
    let _ = result;
}

#[test]
fn test_cell_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let c = Cell::new(42)
        c.get()
    "#,
    );
    let _ = result;
}

#[test]
fn test_refcell_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let r = RefCell::new(42)
        *r.borrow()
    "#,
    );
    let _ = result;
}

#[test]
fn test_mutex_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let m = Mutex::new(42)
        *m.lock()
    "#,
    );
    let _ = result;
}

#[test]
fn test_rwlock_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let rw = RwLock::new(42)
        *rw.read()
    "#,
    );
    let _ = result;
}

#[test]
fn test_channel_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let (tx, rx) = channel()
        tx.send(42)
        rx.recv()
    "#,
    );
    let _ = result;
}

#[test]
fn test_labeled_loop_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        'outer: loop {
            'inner: loop {
                break 'outer
            }
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_labeled_while_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let i = 0
        'outer: while i < 10 {
            let j = 0
            while j < 10 {
                if j == 5 { break 'outer }
                j = j + 1
            }
            i = i + 1
        }
        i
    "#,
    );
    let _ = result;
}

#[test]
fn test_labeled_for_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let sum = 0
        'outer: for i in range(0, 10) {
            for j in range(0, 10) {
                if i + j > 10 { break 'outer }
                sum = sum + 1
            }
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_ternary_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        let y = if x > 0 { "positive" } else { "non-positive" }
        y
    "#,
    );
    let _ = result;
}

#[test]
fn test_elvis_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = None
        x ?? 42
    "#,
    );
    let _ = result;
}

#[test]
fn test_safe_navigation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let obj = Some({"name": "Alice"})
        obj?.name
    "#,
    );
    let _ = result;
}

#[test]
fn test_not_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = false
        !x && !false
    "#,
    );
    let _ = result;
}

#[test]
fn test_bitwise_not_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        ~0xFF
    "#,
    );
    let _ = result;
}

#[test]
fn test_xor_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        0b1010 ^ 0b1100
    "#,
    );
    let _ = result;
}

#[test]
fn test_compound_xor_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 0b1010
        x ^= 0b1100
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_compound_shift_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 4
        x <<= 2
        x >>= 1
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_compound_and_or_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 0xFF
        x &= 0x0F
        x |= 0xF0
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_compound_mod_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 17
        x %= 5
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_import_multiple_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        use {std::io, std::fs}
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_import_alias_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        use std::collections::HashMap as Map
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_import_glob_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        use std::io::*
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_module_definition_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod utils {
            fun helper() { 42 }
        }
        utils::helper()
    "#,
    );
    let _ = result;
}

#[test]
fn test_pub_modifier_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod inner {
            pub fun visible() { 1 }
            fun private() { 2 }
        }
        inner::visible()
    "#,
    );
    let _ = result;
}

#[test]
fn test_extern_block_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        extern "C" {
            fun printf(format: *const i8) -> i32
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_attribute_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        #[derive(Debug)]
        struct Point { x: i64, y: i64 }
        Point { x: 1, y: 2 }
    "#,
    );
    let _ = result;
}

#[test]
fn test_inner_attribute_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        #![allow(unused)]
        let x = 42
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_macro_invocation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        vec![1, 2, 3]
    "#,
    );
    let _ = result;
}

#[test]
fn test_format_macro_string_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        format!("Hello, {}!", "World")
    "#,
    );
    let _ = result;
}

#[test]
fn test_println_macro_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        println!("Test: {}", 42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_panic_macro_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun may_panic() {
            panic!("Something went wrong")
        }
        try { may_panic() } catch e { "caught" }
    "#,
    );
    let _ = result;
}

#[test]
fn test_assert_macro_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert!(true)
        assert!(1 == 1, "Numbers should be equal")
    "#,
    );
    let _ = result;
}

#[test]
fn test_debug_assert_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        debug_assert!(true)
    "#,
    );
    let _ = result;
}

#[test]
fn test_unreachable_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun check(x: i64) -> String {
            if x > 0 { "positive" }
            else if x < 0 { "negative" }
            else { "zero" }
        }
        check(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_todo_macro_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun unimplemented() {
            todo!("Implement later")
        }
        try { unimplemented() } catch e { "caught" }
    "#,
    );
    let _ = result;
}

#[test]
fn test_method_receiver_self_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Counter { value: i64 }
        impl Counter {
            fun get(self) -> i64 { self.value }
            fun increment(&mut self) { self.value += 1 }
        }
        let c = Counter { value: 0 }
        c.get()
    "#,
    );
    let _ = result;
}

#[test]
fn test_associated_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: i64, y: i64 }
        impl Point {
            fun origin() -> Point { Point { x: 0, y: 0 } }
            fun new(x: i64, y: i64) -> Point { Point { x, y } }
        }
        Point::origin()
    "#,
    );
    let _ = result;
}

#[test]
fn test_method_chaining_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Builder { value: String }
        impl Builder {
            fun new() -> Builder { Builder { value: "" } }
            fun add(self, s: String) -> Builder {
                Builder { value: self.value + s }
            }
            fun build(self) -> String { self.value }
        }
        Builder::new().add("hello").add(" ").add("world").build()
    "#,
    );
    let _ = result;
}

#[test]
fn test_operator_overload_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Vec2 { x: f64, y: f64 }
        impl Add for Vec2 {
            fun add(self, other: Vec2) -> Vec2 {
                Vec2 { x: self.x + other.x, y: self.y + other.y }
            }
        }
        let a = Vec2 { x: 1.0, y: 2.0 }
        let b = Vec2 { x: 3.0, y: 4.0 }
        a + b
    "#,
    );
    let _ = result;
}

#[test]
fn test_index_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct MyArray { data: [i64] }
        impl Index for MyArray {
            fun index(self, i: usize) -> i64 { self.data[i] }
        }
        let arr = MyArray { data: [1, 2, 3] }
        arr[1]
    "#,
    );
    let _ = result;
}

#[test]
fn test_iterator_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Counter { count: i64 }
        impl Iterator for Counter {
            fun next(self) -> Option<i64> {
                if self.count < 5 {
                    self.count += 1
                    Some(self.count)
                } else {
                    None
                }
            }
        }
        let c = Counter { count: 0 }
        c.collect()
    "#,
    );
    let _ = result;
}

#[test]
fn test_from_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Wrapper { value: i64 }
        impl From<i64> for Wrapper {
            fun from(v: i64) -> Wrapper { Wrapper { value: v } }
        }
        Wrapper::from(42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_into_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Num { v: i64 }
        impl Into<i64> for Num {
            fun into(self) -> i64 { self.v }
        }
        let n = Num { v: 42 }
        n.into()
    "#,
    );
    let _ = result;
}

#[test]
fn test_clone_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Data { value: i64 }
        impl Clone for Data {
            fun clone(self) -> Data { Data { value: self.value } }
        }
        let d = Data { value: 42 }
        d.clone()
    "#,
    );
    let _ = result;
}

#[test]
fn test_default_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Config { level: i64, debug: bool }
        impl Default for Config {
            fun default() -> Config { Config { level: 1, debug: false } }
        }
        Config::default()
    "#,
    );
    let _ = result;
}

#[test]
fn test_drop_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Resource { id: i64 }
        impl Drop for Resource {
            fun drop(self) { println!("Dropping {}", self.id) }
        }
        let r = Resource { id: 1 }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_deref_trait_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct SmartPtr<T> { value: T }
        impl Deref for SmartPtr<i64> {
            fun deref(self) -> i64 { self.value }
        }
        let p = SmartPtr { value: 42 }
        *p
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_once_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun run_once<F: FnOnce() -> i64>(f: F) -> i64 { f() }
        run_once(|| 42)
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_mut_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun run_mut<F: FnMut() -> i64>(f: &mut F) -> i64 { f() }
        let count = 0
        let f = || { count += 1; count }
        run_mut(&mut f)
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_fn_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun run<F: Fn() -> i64>(f: &F) -> i64 { f() }
        let f = || 42
        run(&f)
    "#,
    );
    let _ = result;
}

#[test]
fn test_higher_order_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun apply_twice<F: Fn(i64) -> i64>(f: F, x: i64) -> i64 {
            f(f(x))
        }
        apply_twice(|x| x * 2, 3)
    "#,
    );
    let _ = result;
}

#[test]
fn test_currying_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun add(a: i64) -> (i64) -> i64 {
            |b| a + b
        }
        let add5 = add(5)
        add5(10)
    "#,
    );
    let _ = result;
}

#[test]
fn test_compose_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun compose<F, G>(f: F, g: G) -> (i64) -> i64
            where F: Fn(i64) -> i64, G: Fn(i64) -> i64
        {
            |x| f(g(x))
        }
        let double_then_add1 = compose(|x| x + 1, |x| x * 2)
        double_then_add1(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_partial_application_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun multiply(a: i64, b: i64) -> i64 { a * b }
        let double = |x| multiply(2, x)
        double(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_memoize_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let cache = {}
        fun fib(n: i64) -> i64 {
            if cache.contains_key(n) { return cache[n] }
            let result = if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
            cache[n] = result
            result
        }
        fib(10)
    "#,
    );
    let _ = result;
}

#[test]
fn test_lazy_evaluation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Lazy<T> { value: Option<T>, compute: () -> T }
        impl Lazy<i64> {
            fun get(self) -> i64 {
                if self.value.is_none() {
                    self.value = Some((self.compute)())
                }
                self.value.unwrap()
            }
        }
        let lazy = Lazy { value: None, compute: || 42 }
        lazy.get()
    "#,
    );
    let _ = result;
}

#[test]
fn test_either_type_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Either<L, R> {
            Left(L),
            Right(R)
        }
        let x: Either<i64, String> = Either::Left(42)
        match x {
            Either::Left(n) => n,
            Either::Right(s) => 0
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_state_monad_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct State<S, A> { run: (S) -> (A, S) }
        fun get<S>() -> State<S, S> {
            State { run: |s| (s, s) }
        }
        fun put<S>(s: S) -> State<S, ()> {
            State { run: |_| ((), s) }
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_io_monad_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct IO<A> { action: () -> A }
        impl IO<i64> {
            fun run(self) -> i64 { (self.action)() }
            fun map<B>(self, f: (i64) -> B) -> IO<B> {
                IO { action: || f(self.run()) }
            }
        }
        let io = IO { action: || 42 }
        io.run()
    "#,
    );
    let _ = result;
}

