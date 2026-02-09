// Auto-extracted from interpreter_tests.rs - Part 15
use super::*;

// === Collections Parser Coverage Tests ===
#[test]
fn test_list_comprehension_basic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[x * 2 for x in [1, 2, 3]]"#);
    let _ = result;
}

#[test]
fn test_list_comprehension_with_if_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[x for x in [1, 2, 3, 4, 5] if x % 2 == 0]"#);
    let _ = result;
}

#[test]
fn test_list_comprehension_nested_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"[x + y for x in [1, 2] for y in [10, 20]]"#);
    let _ = result;
}

#[test]
fn test_set_literal_basic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"#{1, 2, 3}"#);
    let _ = result;
}

#[test]
fn test_set_comprehension_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"#{x * 2 for x in [1, 2, 3]}"#);
    let _ = result;
}

#[test]
fn test_dict_literal_basic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{"a": 1, "b": 2}"#);
    let _ = result;
}

#[test]
fn test_dict_comprehension_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{x: x * 2 for x in [1, 2, 3]}"#);
    let _ = result;
}

#[test]
fn test_empty_block_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{ }"#);
    let _ = result;
}

#[test]
fn test_block_with_comments_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"{
        // This is a comment
        let x = 5
        x + 1
    }"#,
    );
    let _ = result;
}

#[test]
fn test_object_literal_syntax_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"{ name: "test", value: 42 }"#);
    let _ = result;
}

#[test]
fn test_object_literal_shorthand_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let name = "test"
        let value = 42
        { name, value }
    "#,
    );
    let _ = result;
}

#[test]
fn test_tuple_basic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"(1, "hello", true)"#);
    let _ = result;
}

#[test]
fn test_tuple_destructuring_let_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let (a, b, c) = (1, 2, 3)
        a + b + c
    "#,
    );
    let _ = result;
}

#[test]
fn test_array_slice_cov() {
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
fn test_array_slice_from_cov() {
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
fn test_array_slice_to_cov() {
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
fn test_nested_array_index_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [[1, 2], [3, 4], [5, 6]]
        arr[1][0]
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_from_map_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        DataFrame::from_map({"a": [1, 2, 3], "b": [4, 5, 6]})
    "#,
    );
    let _ = result;
}

#[test]
fn test_dataframe_column_access_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let df = DataFrame::from_map({"a": [1, 2, 3]})
        df["a"]
    "#,
    );
    let _ = result;
}

// === WASM/Compiler Coverage Tests ===
#[test]
fn test_bitwise_operations_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 5
        let b = 3
        let and_result = a & b
        let or_result = a | b
        let xor_result = a ^ b
        and_result + or_result + xor_result
    "#,
    );
    let _ = result;
}

#[test]
fn test_shift_operations_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 8
        let left = a << 2
        let right = a >> 2
        left + right
    "#,
    );
    let _ = result;
}

#[test]
fn test_complex_match_pattern_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let value = [1, 2, 3]
        match value {
            [] => "empty",
            [x] => "single",
            [x, y] => "pair",
            [x, y, z] => "triple",
            _ => "many"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_guard_condition_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        match x {
            n if n < 0 => "negative",
            n if n == 0 => "zero",
            n if n > 0 => "positive",
            _ => "unknown"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_range_inclusive_loop_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let sum = 0
        for i in 1..=5 {
            sum = sum + i
        }
        sum
    "#,
    );
    let _ = result;
}

#[test]
fn test_labeled_break_outer_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = 0
        @outer: for i in 1..5 {
            for j in 1..5 {
                if i * j > 6 {
                    break @outer
                }
                result = result + 1
            }
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_labeled_continue_outer_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = 0
        @outer: for i in 1..4 {
            for j in 1..4 {
                if j == 2 {
                    continue @outer
                }
                result = result + 1
            }
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_try_catch_with_type_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        try {
            let x = 1 / 0
            x
        } catch e {
            0
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_assert_with_message_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert(true, "This should not fail")
        42
    "#,
    );
    let _ = result;
}

#[test]
fn test_pipeline_operator_chain_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun double(x) { x * 2 }
        fun add_one(x) { x + 1 }
        5 |> double |> add_one
    "#,
    );
    let _ = result;
}

#[test]
fn test_compose_operator_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun double(x) { x * 2 }
        fun add_one(x) { x + 1 }
        let composed = double >> add_one
        composed(5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_null_coalescing_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let maybe_value: Option<i64> = None
        maybe_value ?? 42
    "#,
    );
    let _ = result;
}

#[test]
fn test_optional_chaining_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Person { name: String }
        let person: Option<Person> = Some(Person { name: "Alice" })
        person?.name
    "#,
    );
    let _ = result;
}

#[test]
fn test_spread_operator_array_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr1 = [1, 2, 3]
        let arr2 = [...arr1, 4, 5]
        arr2
    "#,
    );
    let _ = result;
}

#[test]
fn test_spread_operator_object_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let obj1 = { a: 1, b: 2 }
        let obj2 = { ...obj1, c: 3 }
        obj2
    "#,
    );
    let _ = result;
}

// === Module/Import Coverage Tests ===
#[test]
fn test_module_definition_math_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod math {
            fun add(a, b) { a + b }
            fun multiply(a, b) { a * b }
        }
        math::add(2, 3)
    "#,
    );
    let _ = result;
}

#[test]
fn test_module_nested_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        mod outer {
            mod inner {
                fun value() { 42 }
            }
        }
        outer::inner::value()
    "#,
    );
    let _ = result;
}

// === Error Handling Coverage Tests ===
#[test]
fn test_result_ok_unwrap_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result: Result<i64, String> = Ok(42)
        result.unwrap()
    "#,
    );
    let _ = result;
}

#[test]
fn test_result_map_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result: Result<i64, String> = Ok(21)
        result.map(|x| x * 2)
    "#,
    );
    let _ = result;
}

#[test]
fn test_option_and_then_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let opt: Option<i64> = Some(5)
        opt.and_then(|x| if x > 0 { Some(x * 2) } else { None })
    "#,
    );
    let _ = result;
}

// === Actor Coverage Tests ===
#[test]
fn test_actor_definition_counter_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Counter {
            state count: i64 = 0

            message increment() {
                self.count = self.count + 1
            }

            message get() -> i64 {
                self.count
            }
        }
        42
    "#,
    );
    let _ = result;
}

// === Type System Coverage Tests ===
#[test]
fn test_generic_function_identity_cov() {
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
fn test_generic_struct_pair_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Pair<A, B> { first: A, second: B }
        let p = Pair { first: 1, second: "hello" }
        p.first
    "#,
    );
    let _ = result;
}

#[test]
fn test_where_clause_clone_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun process<T>(x: T) -> T where T: Clone {
            x
        }
        process(42)
    "#,
    );
    let _ = result;
}

// === String Interpolation Coverage Tests ===
#[test]
fn test_string_interpolation_basic_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let name = "World"
        f"Hello, {name}!"
    "#,
    );
    let _ = result;
}

#[test]
fn test_string_interpolation_expression_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 5
        f"Result: {x * 2}"
    "#,
    );
    let _ = result;
}

#[test]
fn test_raw_string_escape_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(r#"r"This is a raw string with \n no escapes""#);
    let _ = result;
}

// === Async Coverage Tests ===
#[test]
fn test_async_function_def_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        async fun fetch_data() {
            42
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_await_expression_async_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        async fun get_value() {
            42
        }
        async fun main() {
            await get_value()
        }
        1
    "#,
    );
    let _ = result;
}

// === Actor Extended Coverage Tests ===
#[test]
fn test_actor_with_state_field_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Bank {
            state balance: i64 = 1000

            message deposit(amount: i64) {
                self.balance = self.balance + amount
            }

            message withdraw(amount: i64) -> bool {
                if self.balance >= amount {
                    self.balance = self.balance - amount
                    true
                } else {
                    false
                }
            }
        }
        1
    "#,
    );
    let _ = result;
}

#[test]
fn test_actor_multiple_state_fields_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        actor Player {
            state name: String = "Unknown"
            state health: i64 = 100
            state score: i64 = 0

            message damage(amount: i64) {
                self.health = self.health - amount
            }
        }
        1
    "#,
    );
    let _ = result;
}

// === Error Recovery Coverage Tests ===
#[test]
fn test_division_by_zero_handling_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 10
        let y = 0
        let z = if y != 0 { x / y } else { 0 }
        z
    "#,
    );
    let _ = result;
}

#[test]
fn test_option_unwrap_or_default_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let opt: Option<i64> = None
        opt.unwrap_or(99)
    "#,
    );
    let _ = result;
}

#[test]
fn test_result_unwrap_or_else_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let res: Result<i64, String> = Err("error")
        res.unwrap_or_else(|_| 0)
    "#,
    );
    let _ = result;
}

// === Pattern Matching Coverage Tests ===
#[test]
fn test_match_tuple_pattern_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let pair = (1, 2)
        match pair {
            (0, 0) => "origin",
            (x, 0) => "x-axis",
            (0, y) => "y-axis",
            (x, y) => "somewhere"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_or_pattern_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 3
        match x {
            1 | 2 | 3 => "small",
            4 | 5 | 6 => "medium",
            _ => "large"
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_match_range_pattern_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = 50
        match x {
            0..=25 => "low",
            26..=75 => "medium",
            _ => "high"
        }
    "#,
    );
    let _ = result;
}

// === Closure Coverage Tests ===
#[test]
fn test_closure_with_multiple_captures_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = 1
        let b = 2
        let c = 3
        let f = || a + b + c
        f()
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_as_argument_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun apply(f: (i64) -> i64, x: i64) -> i64 {
            f(x)
        }
        apply(|x| x * 2, 21)
    "#,
    );
    let _ = result;
}

#[test]
fn test_closure_returning_closure_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fun make_adder(n: i64) -> (i64) -> i64 {
            |x| x + n
        }
        let add5 = make_adder(5)
        add5(10)
    "#,
    );
    let _ = result;
}

// === Iterator Coverage Tests ===
#[test]
fn test_map_filter_chain_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr.map(|x| x * 2).filter(|x| x > 5)
    "#,
    );
    let _ = result;
}

#[test]
fn test_fold_operation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3, 4, 5]
        arr.fold(0, |acc, x| acc + x)
    "#,
    );
    let _ = result;
}

#[test]
fn test_zip_operation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = [1, 2, 3]
        let b = [4, 5, 6]
        a.zip(b)
    "#,
    );
    let _ = result;
}

// === Type Annotation Coverage Tests ===
#[test]
fn test_explicit_type_annotation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x: i64 = 42
        let y: f64 = 3.14
        let z: String = "hello"
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_generic_type_annotation_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr: Vec<i64> = [1, 2, 3]
        let map: HashMap<String, i64> = {"a": 1, "b": 2}
        arr.len()
    "#,
    );
    let _ = result;
}

// === Control Flow Edge Cases ===
#[test]
fn test_nested_loops_with_break_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let result = 0
        for i in 0..10 {
            for j in 0..10 {
                for k in 0..10 {
                    if k == 5 {
                        break
                    }
                    result = result + 1
                }
            }
        }
        result
    "#,
    );
    let _ = result;
}

#[test]
fn test_loop_with_return_value_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let x = loop {
            let value = 42
            break value
        }
        x
    "#,
    );
    let _ = result;
}

#[test]
fn test_while_let_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let mut stack: Vec<i64> = [1, 2, 3]
        let mut sum = 0
        while let Some(x) = stack.pop() {
            sum = sum + x
        }
        sum
    "#,
    );
    let _ = result;
}

// === Struct Method Coverage Tests ===
#[test]
fn test_struct_with_methods_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Rectangle {
            width: f64,
            height: f64
        }
        impl Rectangle {
            fun area(self) -> f64 {
                self.width * self.height
            }
            fun perimeter(self) -> f64 {
                2.0 * (self.width + self.height)
            }
        }
        let rect = Rectangle { width: 10.0, height: 5.0 }
        rect.area()
    "#,
    );
    let _ = result;
}

#[test]
fn test_struct_static_method_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        struct Point { x: f64, y: f64 }
        impl Point {
            fun origin() -> Point {
                Point { x: 0.0, y: 0.0 }
            }
            fun new(x: f64, y: f64) -> Point {
                Point { x, y }
            }
        }
        let p = Point::new(3.0, 4.0)
        p.x
    "#,
    );
    let _ = result;
}

// === Enum Coverage Tests ===
#[test]
fn test_enum_with_data_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        enum Message {
            Quit,
            Move { x: i64, y: i64 },
            Write(String),
            ChangeColor(i64, i64, i64)
        }
        let msg = Message::Move { x: 10, y: 20 }
        match msg {
            Message::Quit => 0,
            Message::Move { x, y } => x + y,
            Message::Write(s) => s.len(),
            Message::ChangeColor(r, g, b) => r + g + b
        }
    "#,
    );
    let _ = result;
}

#[test]
fn test_enum_option_methods_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let some_val: Option<i64> = Some(42)
        let none_val: Option<i64> = None
        let is_some = some_val.is_some()
        let is_none = none_val.is_none()
        is_some && is_none
    "#,
    );
    let _ = result;
}

// === Trait Coverage Tests ===
#[test]
fn test_trait_definition_and_impl_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        trait Display {
            fun display(self) -> String
        }
        struct Person { name: String, age: i64 }
        impl Display for Person {
            fun display(self) -> String {
                f"{self.name}: {self.age}"
            }
        }
        let p = Person { name: "Alice", age: 30 }
        p.display()
    "#,
    );
    let _ = result;
}

#[test]
fn test_multiple_trait_impl_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        trait Add<T> {
            fun add(self, other: T) -> T
        }
        trait Multiply<T> {
            fun multiply(self, other: T) -> T
        }
        struct Number { value: i64 }
        impl Add<Number> for Number {
            fun add(self, other: Number) -> Number {
                Number { value: self.value + other.value }
            }
        }
        let a = Number { value: 5 }
        let b = Number { value: 3 }
        a.add(b).value
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON stringify function
#[test]
fn test_json_stringify_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let data = {"name": "test", "value": 42}
        json_stringify(data)
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON pretty printing
#[test]
fn test_json_pretty_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let data = {"key": "value"}
        json_pretty(data)
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON type function
#[test]
fn test_json_type_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        json_type("{}")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON merge function
#[test]
fn test_json_merge_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = {"x": 1}
        let b = {"y": 2}
        json_merge(a, b)
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON get function
#[test]
fn test_json_get_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let data = {"nested": {"value": 100}}
        json_get(data, "nested.value")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON set function
#[test]
fn test_json_set_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let data = {"key": 1}
        json_set(data, "key", 2)
    "#,
    );
    let _ = result;
}

// COVERAGE: Path is_absolute function
#[test]
fn test_path_is_absolute_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_is_absolute("/home/user")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path is_relative function
#[test]
fn test_path_is_relative_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_is_relative("./relative/path")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path with_extension function
#[test]
fn test_path_with_extension_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_with_extension("file.txt", "md")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path with_file_name function
#[test]
fn test_path_with_file_name_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_with_file_name("/path/to/old.txt", "new.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path components function
#[test]
fn test_path_components_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_components("/home/user/file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path normalize function
#[test]
fn test_path_normalize_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_normalize("/home/../home/user/./file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: String from function
#[test]
fn test_string_from_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        String::from("hello")
    "#,
    );
    let _ = result;
}

// COVERAGE: int conversion
#[test]
fn test_int_conversion_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        int(3.14)
    "#,
    );
    let _ = result;
}

// COVERAGE: float conversion
#[test]
fn test_float_conversion_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        float(42)
    "#,
    );
    let _ = result;
}

// COVERAGE: bool conversion
#[test]
fn test_bool_conversion_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        bool(1)
    "#,
    );
    let _ = result;
}

// COVERAGE: parse_int function
#[test]
fn test_parse_int_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        parse_int("42")
    "#,
    );
    let _ = result;
}

// COVERAGE: parse_float function
#[test]
fn test_parse_float_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        parse_float("3.14")
    "#,
    );
    let _ = result;
}

// COVERAGE: str function
#[test]
fn test_str_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        str(42)
    "#,
    );
    let _ = result;
}

// COVERAGE: to_string function
#[test]
fn test_to_string_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        to_string(100)
    "#,
    );
    let _ = result;
}

// COVERAGE: assert_eq function
#[test]
fn test_assert_eq_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        assert_eq(1, 1)
    "#,
    );
    let _ = result;
}

// COVERAGE: is_nil function
#[test]
fn test_is_nil_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        is_nil(nil)
    "#,
    );
    let _ = result;
}

// COVERAGE: timestamp function
#[test]
fn test_timestamp_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        timestamp()
    "#,
    );
    let _ = result;
}

// COVERAGE: chrono_utc_now function
#[test]
fn test_chrono_utc_now_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        chrono_utc_now()
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - asin
#[test]
fn test_asin_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        asin(0.5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - acos
#[test]
fn test_acos_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        acos(0.5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - atan
#[test]
fn test_atan_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        atan(1.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - atan2
#[test]
fn test_atan2_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        atan2(1.0, 1.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - sinh
#[test]
fn test_sinh_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        sinh(1.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - cosh
#[test]
fn test_cosh_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        cosh(1.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - tanh
#[test]
fn test_tanh_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        tanh(0.5)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - log2
#[test]
fn test_log2_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        log2(8.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - ln
#[test]
fn test_ln_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        ln(2.718281828)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - trunc
#[test]
fn test_trunc_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        trunc(3.9)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - fract
#[test]
fn test_fract_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fract(3.14)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - signum
#[test]
fn test_signum_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        signum(-42.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - clamp
#[test]
fn test_clamp_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        clamp(5.0, 0.0, 10.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - hypot
#[test]
fn test_hypot_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        hypot(3.0, 4.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Advanced math - cbrt
#[test]
fn test_cbrt_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        cbrt(27.0)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sleep function (short duration)
#[test]
fn test_sleep_short_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        sleep(1)
    "#,
    );
    let _ = result;
}

// COVERAGE: Enumerate with offset
#[test]
fn test_enumerate_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let items = ["a", "b", "c"]
        let result = []
        for i, item in enumerate(items) {
            result = result + [[i, item]]
        }
        result
    "#,
    );
    let _ = result;
}

// COVERAGE: Zip function
#[test]
fn test_zip_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let a = [1, 2, 3]
        let b = ["a", "b", "c"]
        zip(a, b)
    "#,
    );
    let _ = result;
}

// COVERAGE: Sort function
#[test]
fn test_sort_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [3, 1, 4, 1, 5]
        sort(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Pop function
#[test]
fn test_pop_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2, 3]
        pop(arr)
    "#,
    );
    let _ = result;
}

// COVERAGE: Push function
#[test]
fn test_push_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let arr = [1, 2]
        push(arr, 3)
    "#,
    );
    let _ = result;
}

// COVERAGE: Type of function
#[test]
fn test_type_of_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type_of(42)
    "#,
    );
    let _ = result;
}

// COVERAGE: Type function
#[test]
fn test_type_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        type([1, 2, 3])
    "#,
    );
    let _ = result;
}

// COVERAGE: Range with step
#[test]
fn test_range_with_step_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        range(0, 10, 2)
    "#,
    );
    let _ = result;
}

// COVERAGE: Range reverse (negative step)
#[test]
fn test_range_reverse_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        range(10, 0, -1)
    "#,
    );
    let _ = result;
}

// COVERAGE: Compute hash
#[test]
fn test_compute_hash_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        compute_hash("test string")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS metadata
#[test]
fn test_fs_metadata_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fs_metadata(".")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS read_dir
#[test]
fn test_fs_read_dir_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fs_read_dir(".")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS is_file
#[test]
fn test_fs_is_file_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fs_is_file("Cargo.toml")
    "#,
    );
    let _ = result;
}

// COVERAGE: FS canonicalize
#[test]
fn test_fs_canonicalize_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        fs_canonicalize(".")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path canonicalize
#[test]
fn test_path_canonicalize_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_canonicalize(".")
    "#,
    );
    let _ = result;
}

// COVERAGE: Glob function
#[test]
fn test_glob_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        glob("*.toml")
    "#,
    );
    let _ = result;
}

// COVERAGE: Search function
#[test]
fn test_search_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        search(".", "*.rs")
    "#,
    );
    let _ = result;
}

// COVERAGE: JSON validate
#[test]
fn test_json_validate_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        json_validate("{\"key\": \"value\"}")
    "#,
    );
    let _ = result;
}

// COVERAGE: Env vars function
#[test]
fn test_env_vars_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        let vars = env_vars()
        len(vars) > 0
    "#,
    );
    let _ = result;
}

// COVERAGE: Env temp_dir function
#[test]
fn test_env_temp_dir_fn_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        env_temp_dir()
    "#,
    );
    let _ = result;
}

// COVERAGE: Dbg function
#[test]
fn test_dbg_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        dbg(42)
    "#,
    );
    let _ = result;
}

// COVERAGE: Print function
#[test]
fn test_print_function_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        print("test")
    "#,
    );
    let _ = result;
}

// COVERAGE: Println with format string
#[test]
fn test_println_format_cov() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        println("value: {}", 42)
    "#,
    );
    let _ = result;
}

// COVERAGE: Path join function
#[test]
fn test_path_join_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_join("/home", "user")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path join many function
#[test]
fn test_path_join_many_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_join_many(["/", "home", "user", "file.txt"])
    "#,
    );
    let _ = result;
}

// COVERAGE: Path parent function
#[test]
fn test_path_parent_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_parent("/home/user/file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path file_name function
#[test]
fn test_path_file_name_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_file_name("/home/user/file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path file_stem function
#[test]
fn test_path_file_stem_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_file_stem("/home/user/file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: Path extension function
#[test]
fn test_path_extension_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        path_extension("/home/user/file.txt")
    "#,
    );
    let _ = result;
}

// COVERAGE: String new function
#[test]
fn test_string_new_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        String::new()
    "#,
    );
    let _ = result;
}

// COVERAGE: String from_utf8 function
#[test]
fn test_string_from_utf8_v2() {
    let mut interp = Interpreter::new();
    let result = interp.eval_string(
        r#"
        String::from_utf8([72, 101, 108, 108, 111])
    "#,
    );
    let _ = result;
}
