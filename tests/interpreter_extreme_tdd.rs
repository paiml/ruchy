// EXTREME TDD: Interpreter Coverage with Quality Gates
// Requirements:
// - Cyclomatic Complexity: <10 per function
// - Property tests: 10,000+ iterations
// - Big O validation
// - Zero SATD
// - 100% provability score

use ruchy::runtime::interpreter::Interpreter;
use ruchy::runtime::repl::Value;
use std::rc::Rc;
use proptest::prelude::*;

// ============================================================================
// SECTION 1: Test Helpers with Complexity <5
// ============================================================================

fn create_interpreter() -> Interpreter {
    Interpreter::new()
}

fn parse_and_eval(code: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut interpreter = create_interpreter();
    interpreter.eval_string(code)
}

fn assert_eval_eq(code: &str, expected: Value) {
    let result = parse_and_eval(code).expect("Evaluation failed");
    assert_eq!(result, expected);
}

fn assert_eval_error(code: &str) {
    let result = parse_and_eval(code);
    assert!(result.is_err(), "Expected error but got: {:?}", result);
}

// ============================================================================
// SECTION 2: Arithmetic Operations - O(1) complexity
// ============================================================================

#[test]
fn test_arithmetic_add() {
    assert_eval_eq("1 + 2", Value::Integer(3));
    assert_eval_eq("1.5 + 2.5", Value::Float(4.0));
}

#[test]
fn test_arithmetic_subtract() {
    assert_eval_eq("5 - 3", Value::Integer(2));
    assert_eval_eq("5.5 - 2.5", Value::Float(3.0));
}

#[test]
fn test_arithmetic_multiply() {
    assert_eval_eq("3 * 4", Value::Integer(12));
    assert_eval_eq("2.5 * 2.0", Value::Float(5.0));
}

#[test]
fn test_arithmetic_divide() {
    assert_eval_eq("10 / 2", Value::Integer(5));
    assert_eval_eq("7.5 / 2.5", Value::Float(3.0));
}

#[test]
fn test_arithmetic_modulo() {
    assert_eval_eq("10 % 3", Value::Integer(1));
    assert_eval_eq("7 % 2", Value::Integer(1));
}

#[test]
fn test_arithmetic_power() {
    assert_eval_eq("2 ** 3", Value::Integer(8));
    assert_eval_eq("2.0 ** 3.0", Value::Float(8.0));
}

// ============================================================================
// SECTION 3: Comparison Operations - O(1) complexity
// ============================================================================

#[test]
fn test_comparison_equal() {
    assert_eval_eq("1 == 1", Value::Bool(true));
    assert_eval_eq("1 == 2", Value::Bool(false));
    assert_eval_eq("\"hello\" == \"hello\"", Value::Bool(true));
}

#[test]
fn test_comparison_not_equal() {
    assert_eval_eq("1 != 2", Value::Bool(true));
    assert_eval_eq("1 != 1", Value::Bool(false));
}

#[test]
fn test_comparison_less_than() {
    assert_eval_eq("1 < 2", Value::Bool(true));
    assert_eval_eq("2 < 1", Value::Bool(false));
}

#[test]
fn test_comparison_greater_than() {
    assert_eval_eq("2 > 1", Value::Bool(true));
    assert_eval_eq("1 > 2", Value::Bool(false));
}

#[test]
fn test_comparison_less_equal() {
    assert_eval_eq("1 <= 2", Value::Bool(true));
    assert_eval_eq("2 <= 2", Value::Bool(true));
    assert_eval_eq("3 <= 2", Value::Bool(false));
}

#[test]
fn test_comparison_greater_equal() {
    assert_eval_eq("2 >= 1", Value::Bool(true));
    assert_eval_eq("2 >= 2", Value::Bool(true));
    assert_eval_eq("1 >= 2", Value::Bool(false));
}

// ============================================================================
// SECTION 4: Logical Operations - O(1) complexity with short-circuit
// ============================================================================

#[test]
fn test_logical_and() {
    assert_eval_eq("true && true", Value::Bool(true));
    assert_eval_eq("true && false", Value::Bool(false));
    assert_eval_eq("false && true", Value::Bool(false));
}

#[test]
fn test_logical_or() {
    assert_eval_eq("true || false", Value::Bool(true));
    assert_eval_eq("false || true", Value::Bool(true));
    assert_eval_eq("false || false", Value::Bool(false));
}

#[test]
fn test_logical_not() {
    assert_eval_eq("!true", Value::Bool(false));
    assert_eval_eq("!false", Value::Bool(true));
}

// ============================================================================
// SECTION 5: Bitwise Operations - O(1) complexity
// ============================================================================

#[test]
fn test_bitwise_and() {
    assert_eval_eq("5 & 3", Value::Integer(1));
    assert_eval_eq("12 & 7", Value::Integer(4));
}

#[test]
fn test_bitwise_or() {
    assert_eval_eq("5 | 3", Value::Integer(7));
    assert_eval_eq("8 | 4", Value::Integer(12));
}

#[test]
fn test_bitwise_xor() {
    assert_eval_eq("5 ^ 3", Value::Integer(6));
    assert_eval_eq("12 ^ 7", Value::Integer(11));
}

#[test]
fn test_bitwise_shift_left() {
    assert_eval_eq("2 << 3", Value::Integer(16));
    assert_eval_eq("1 << 8", Value::Integer(256));
}

#[test]
fn test_bitwise_shift_right() {
    assert_eval_eq("16 >> 2", Value::Integer(4));
    assert_eval_eq("256 >> 8", Value::Integer(1));
}

// ============================================================================
// SECTION 6: Variable Operations - O(1) hash lookup
// ============================================================================

#[test]
fn test_variable_assignment() {
    assert_eval_eq("x = 5; x", Value::Integer(5));
    assert_eval_eq("x = \"hello\"; x", Value::String(Rc::new("hello".to_string())));
}

#[test]
fn test_variable_reassignment() {
    assert_eval_eq("x = 5; x = 10; x", Value::Integer(10));
}

#[test]
fn test_compound_assignment_add() {
    assert_eval_eq("x = 5; x += 3; x", Value::Integer(8));
}

#[test]
fn test_compound_assignment_subtract() {
    assert_eval_eq("x = 10; x -= 3; x", Value::Integer(7));
}

#[test]
fn test_compound_assignment_multiply() {
    assert_eval_eq("x = 4; x *= 3; x", Value::Integer(12));
}

#[test]
fn test_compound_assignment_divide() {
    assert_eval_eq("x = 20; x /= 4; x", Value::Integer(5));
}

// ============================================================================
// SECTION 7: Control Flow - O(n) where n is branch count
// ============================================================================

#[test]
fn test_if_true_branch() {
    assert_eval_eq("if true { 1 } else { 2 }", Value::Integer(1));
}

#[test]
fn test_if_false_branch() {
    assert_eval_eq("if false { 1 } else { 2 }", Value::Integer(2));
}

#[test]
fn test_if_without_else() {
    assert_eval_eq("if false { 1 }", Value::Nil);
}

#[test]
fn test_nested_if() {
    let code = "if true { if false { 1 } else { 2 } } else { 3 }";
    assert_eval_eq(code, Value::Integer(2));
}

// ============================================================================
// SECTION 8: Loop Operations - O(n*m) where n=iterations, m=body complexity
// ============================================================================

#[test]
fn test_while_loop() {
    let code = "x = 0; while x < 3 { x = x + 1 }; x";
    assert_eval_eq(code, Value::Integer(3));
}

#[test]
fn test_for_loop_array() {
    let code = "sum = 0; for x in [1, 2, 3] { sum = sum + x }; sum";
    assert_eval_eq(code, Value::Integer(6));
}

#[test]
fn test_for_loop_range() {
    let code = "sum = 0; for x in 1..4 { sum = sum + x }; sum";
    assert_eval_eq(code, Value::Integer(6));
}

#[test]
fn test_loop_break() {
    let code = "x = 0; loop { x = x + 1; if x == 3 { break } }; x";
    assert_eval_eq(code, Value::Integer(3));
}

#[test]
fn test_loop_continue() {
    let code = "sum = 0; for x in 1..6 { if x == 3 { continue }; sum = sum + x }; sum";
    assert_eval_eq(code, Value::Integer(12)); // 1+2+4+5 = 12
}

// ============================================================================
// SECTION 9: Collection Operations - O(n) for most operations
// ============================================================================

#[test]
fn test_array_literal() {
    let mut interp = create_interpreter();
    let result = interp.eval_string("[1, 2, 3]").unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_indexing() {
    assert_eval_eq("[1, 2, 3][0]", Value::Integer(1));
    assert_eval_eq("[1, 2, 3][2]", Value::Integer(3));
}

#[test]
fn test_array_length() {
    assert_eval_eq("[1, 2, 3].len()", Value::Integer(3));
    assert_eval_eq("[].length()", Value::Integer(0));
}

#[test]
fn test_array_push() {
    let code = "arr = [1, 2]; arr.push(3); arr.len()";
    assert_eval_eq(code, Value::Integer(3));
}

#[test]
fn test_array_pop() {
    let code = "arr = [1, 2, 3]; arr.pop()";
    assert_eval_eq(code, Value::Integer(3));
}

// ============================================================================
// SECTION 10: String Operations - O(n) for most operations
// ============================================================================

#[test]
fn test_string_concatenation() {
    assert_eval_eq("\"hello\" + \" world\"", Value::String(Rc::new("hello world".to_string())));
}

#[test]
fn test_string_length() {
    assert_eval_eq("\"hello\".len()", Value::Integer(5));
}

#[test]
fn test_string_to_upper() {
    assert_eval_eq("\"hello\".to_upper()", Value::String(Rc::new("HELLO".to_string())));
}

#[test]
fn test_string_to_lower() {
    assert_eval_eq("\"HELLO\".to_lower()", Value::String(Rc::new("hello".to_string())));
}

#[test]
fn test_string_trim() {
    assert_eval_eq("\"  hello  \".trim()", Value::String(Rc::new("hello".to_string())));
}

#[test]
fn test_string_contains() {
    assert_eval_eq("\"hello world\".contains(\"world\")", Value::Bool(true));
    assert_eval_eq("\"hello world\".contains(\"foo\")", Value::Bool(false));
}

// ============================================================================
// SECTION 11: Function Operations - O(1) definition, O(n) execution
// ============================================================================

#[test]
fn test_function_definition_and_call() {
    let code = "fun add(x, y) { x + y }; add(3, 4)";
    assert_eval_eq(code, Value::Integer(7));
}

#[test]
fn test_recursive_function() {
    let code = "fun fact(n) { if n <= 1 { 1 } else { n * fact(n - 1) } }; fact(5)";
    assert_eval_eq(code, Value::Integer(120));
}

#[test]
fn test_closure() {
    let code = "x = 10; fun add_x(y) { x + y }; add_x(5)";
    assert_eval_eq(code, Value::Integer(15));
}

#[test]
fn test_lambda_expression() {
    let code = "add = (x, y) => x + y; add(3, 4)";
    assert_eval_eq(code, Value::Integer(7));
}

// ============================================================================
// SECTION 12: Higher-Order Functions - O(n*m) where n=collection size, m=function complexity
// ============================================================================

#[test]
fn test_array_map() {
    let code = "[1, 2, 3].map(x => x * 2)";
    let mut interp = create_interpreter();
    let result = interp.eval_string(code).unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(2));
            assert_eq!(arr[1], Value::Integer(4));
            assert_eq!(arr[2], Value::Integer(6));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_filter() {
    let code = "[1, 2, 3, 4, 5].filter(x => x > 2)";
    let mut interp = create_interpreter();
    let result = interp.eval_string(code).unwrap();
    match result {
        Value::Array(arr) => {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(3));
            assert_eq!(arr[1], Value::Integer(4));
            assert_eq!(arr[2], Value::Integer(5));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_array_reduce() {
    let code = "[1, 2, 3, 4].reduce((acc, x) => acc + x, 0)";
    assert_eval_eq(code, Value::Integer(10));
}

#[test]
fn test_array_any() {
    assert_eval_eq("[1, 2, 3].any(x => x > 2)", Value::Bool(true));
    assert_eval_eq("[1, 2, 3].any(x => x > 5)", Value::Bool(false));
}

#[test]
fn test_array_all() {
    assert_eval_eq("[2, 4, 6].all(x => x % 2 == 0)", Value::Bool(true));
    assert_eval_eq("[2, 3, 4].all(x => x % 2 == 0)", Value::Bool(false));
}

#[test]
fn test_array_find() {
    assert_eval_eq("[1, 2, 3].find(x => x > 1)", Value::Integer(2));
    assert_eval_eq("[1, 2, 3].find(x => x > 5)", Value::Nil);
}

// ============================================================================
// SECTION 13: Error Handling - O(1) for error creation
// ============================================================================

#[test]
fn test_undefined_variable() {
    assert_eval_error("undefined_var");
}

#[test]
fn test_division_by_zero() {
    assert_eval_error("1 / 0");
}

#[test]
fn test_type_mismatch_add() {
    assert_eval_error("1 + \"hello\"");
}

#[test]
fn test_array_out_of_bounds() {
    assert_eval_error("[1, 2, 3][10]");
}

#[test]
fn test_invalid_method_call() {
    assert_eval_error("123.undefined_method()");
}

// ============================================================================
// SECTION 14: Property-Based Tests - 10,000+ iterations
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_arithmetic_commutative_add(a: i32, b: i32) {
        let code1 = format!("{} + {}", a, b);
        let code2 = format!("{} + {}", b, a);
        let result1 = parse_and_eval(&code1);
        let result2 = parse_and_eval(&code2);
        prop_assert_eq!(result1.ok(), result2.ok());
    }

    #[test]
    fn prop_arithmetic_commutative_multiply(a: i32, b: i32) {
        let code1 = format!("{} * {}", a, b);
        let code2 = format!("{} * {}", b, a);
        let result1 = parse_and_eval(&code1);
        let result2 = parse_and_eval(&code2);
        prop_assert_eq!(result1.ok(), result2.ok());
    }

    #[test]
    fn prop_arithmetic_associative_add(a: i16, b: i16, c: i16) {
        let code1 = format!("({} + {}) + {}", a, b, c);
        let code2 = format!("{} + ({} + {})", a, b, c);
        let result1 = parse_and_eval(&code1);
        let result2 = parse_and_eval(&code2);
        prop_assert_eq!(result1.ok(), result2.ok());
    }

    #[test]
    fn prop_boolean_double_negation(b: bool) {
        let code = format!("!!{}", b);
        let result = parse_and_eval(&code).unwrap();
        prop_assert_eq!(result, Value::Bool(b));
    }

    #[test]
    fn prop_comparison_reflexive(n: i32) {
        let code = format!("{} == {}", n, n);
        let result = parse_and_eval(&code).unwrap();
        prop_assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn prop_comparison_antisymmetric(a: i32, b: i32) {
        let code1 = format!("{} < {}", a, b);
        let code2 = format!("{} > {}", b, a);
        let result1 = parse_and_eval(&code1);
        let result2 = parse_and_eval(&code2);
        prop_assert_eq!(result1.ok(), result2.ok());
    }

    #[test]
    fn prop_string_concat_associative(a in "[a-z]{0,10}", b in "[a-z]{0,10}", c in "[a-z]{0,10}") {
        let code1 = format!("(\"{}\" + \"{}\") + \"{}\"", a, b, c);
        let code2 = format!("\"{}\" + (\"{}\" + \"{}\")", a, b, c);
        let result1 = parse_and_eval(&code1).unwrap();
        let result2 = parse_and_eval(&code2).unwrap();
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_array_length_after_push(size in 0usize..100) {
        let elements = (0..size).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        let code = format!("arr = [{}]; arr.push(999); arr.len()", elements);
        let result = parse_and_eval(&code).unwrap();
        prop_assert_eq!(result, Value::Integer((size + 1) as i64));
    }

    #[test]
    fn prop_variable_assignment_preserves_value(n: i32) {
        let code = format!("x = {}; x", n);
        let result = parse_and_eval(&code).unwrap();
        prop_assert_eq!(result, Value::Integer(n as i64));
    }

    #[test]
    fn prop_if_else_exhaustive(condition: bool, a: i32, b: i32) {
        let code = format!("if {} {{ {} }} else {{ {} }}", condition, a, b);
        let result = parse_and_eval(&code).unwrap();
        let expected = if condition { a } else { b };
        prop_assert_eq!(result, Value::Integer(expected as i64));
    }
}

// ============================================================================
// SECTION 15: Big O Complexity Validation Tests
// ============================================================================

#[test]
fn test_arithmetic_is_o1() {
    // Arithmetic operations should take constant time regardless of input size
    use std::time::Instant;

    let small_calc = "1 + 1";
    let large_calc = "999999999 + 999999999";

    let start = Instant::now();
    parse_and_eval(small_calc).unwrap();
    let small_time = start.elapsed();

    let start = Instant::now();
    parse_and_eval(large_calc).unwrap();
    let large_time = start.elapsed();

    // Time should be roughly the same (within 10x tolerance for system variance)
    assert!(large_time.as_nanos() < small_time.as_nanos() * 10);
}

#[test]
fn test_array_map_is_on() {
    // Array map should be O(n) - linear with array size
    use std::time::Instant;

    let small_array = "[1, 2, 3].map(x => x * 2)";
    let large_array = format!("[{}].map(x => x * 2)",
        (1..100).map(|i| i.to_string()).collect::<Vec<_>>().join(", "));

    let start = Instant::now();
    parse_and_eval(small_array).unwrap();
    let small_time = start.elapsed();

    let start = Instant::now();
    parse_and_eval(&large_array).unwrap();
    let large_time = start.elapsed();

    // Time should scale roughly linearly (within reasonable bounds)
    // 100 elements vs 3 elements = ~33x, allow up to 100x for overhead
    let ratio = large_time.as_nanos() / small_time.as_nanos().max(1);
    assert!(ratio < 100, "Map operation not O(n): ratio = {}", ratio);
}

// ============================================================================
// SECTION 16: Match Expression Tests - O(n) where n is number of arms
// ============================================================================

#[test]
fn test_match_literal_patterns() {
    let code = r#"
        match 2 {
            1 => "one",
            2 => "two",
            3 => "three",
            _ => "other"
        }
    "#;
    assert_eval_eq(code, Value::String(Rc::new("two".to_string())));
}

#[test]
fn test_match_with_guards() {
    let code = r#"
        x = 15;
        match x {
            n if n < 10 => "small",
            n if n < 20 => "medium",
            _ => "large"
        }
    "#;
    assert_eval_eq(code, Value::String(Rc::new("medium".to_string())));
}

// ============================================================================
// SECTION 17: Object/HashMap Operations - O(1) average, O(n) worst
// ============================================================================

#[test]
fn test_object_literal() {
    let code = r#"{ "name": "Alice", "age": 30 }"#;
    let mut interp = create_interpreter();
    let result = interp.eval_string(code).unwrap();
    match result {
        Value::Object(map) => {
            assert_eq!(map.get("name"), Some(&Value::String(Rc::new("Alice".to_string()))));
            assert_eq!(map.get("age"), Some(&Value::Integer(30)));
        }
        _ => panic!("Expected object"),
    }
}

#[test]
fn test_object_field_access() {
    let code = r#"obj = { "x": 10 }; obj.x"#;
    assert_eval_eq(code, Value::Integer(10));
}

// ============================================================================
// SECTION 18: Range Operations - O(1) for creation, O(n) for iteration
// ============================================================================

#[test]
fn test_range_exclusive() {
    let code = "sum = 0; for i in 0..3 { sum = sum + i }; sum";
    assert_eval_eq(code, Value::Integer(3)); // 0+1+2
}

#[test]
fn test_range_inclusive() {
    let code = "sum = 0; for i in 1..=3 { sum = sum + i }; sum";
    assert_eval_eq(code, Value::Integer(6)); // 1+2+3
}

// ============================================================================
// SECTION 19: Tuple Operations - O(n) where n is tuple size
// ============================================================================

#[test]
fn test_tuple_literal() {
    let code = "(1, \"hello\", true)";
    let mut interp = create_interpreter();
    let result = interp.eval_string(code).unwrap();
    match result {
        Value::Tuple(elements) => {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], Value::Integer(1));
            assert_eq!(elements[1], Value::String(Rc::new("hello".to_string())));
            assert_eq!(elements[2], Value::Bool(true));
        }
        _ => panic!("Expected tuple"),
    }
}

// ============================================================================
// SECTION 20: String Interpolation - O(n) where n is number of parts
// ============================================================================

#[test]
fn test_string_interpolation() {
    let code = r#"name = "World"; f"Hello, {name}!""#;
    assert_eval_eq(code, Value::String(Rc::new("Hello, World!".to_string())));
}

#[test]
fn test_string_interpolation_with_expressions() {
    let code = r#"f"2 + 2 = {2 + 2}""#;
    assert_eval_eq(code, Value::String(Rc::new("2 + 2 = 4".to_string())));
}

// ============================================================================
// SECTION 21: Module Operations - O(1) for lookup
// ============================================================================

#[test]
fn test_module_definition() {
    let code = "module math { fun add(x, y) { x + y } }";
    // Module definition returns nil
    assert_eval_eq(code, Value::Nil);
}

// ============================================================================
// QUALITY METRICS VALIDATION
// All functions have complexity <10 (verified by structure)
// No SATD comments present
// Property tests run 10,000 iterations
// Big O complexity documented and tested
// ============================================================================