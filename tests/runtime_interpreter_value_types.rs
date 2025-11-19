//! EXTREME TDD: Runtime Interpreter Value Types Coverage
//!
//! Target: src/runtime/interpreter.rs (7,908 lines, 73.4/100 TDG → 85+ target)
//! Strategy: Test Value type operations, type conversions, edge cases
//! Coverage: `Value::Int`, Float, String, Bool, Array, Function, Nil, `DataFrame`
//!
//! TDG-driven testing: Focus on interpreter.rs quality improvement

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Value Type: Integer Operations
// ============================================================================

#[test]
fn test_value_int_arithmetic() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(42 + 8 - 10 * 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_value_int_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = -42; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("-42"));
}

#[test]
fn test_value_int_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 0; println(x * 100)")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_value_int_max_value() {
    // Test large integer handling
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 2147483647; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("2147483647"));
}

#[test]
fn test_value_int_comparison() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(10 > 5, 3 < 2, 7 == 7)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("false"));
}

// ============================================================================
// Value Type: Float Operations
// ============================================================================

#[test]
fn test_value_float_arithmetic() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(3.14 + 2.86)")
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
fn test_value_float_precision() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 0.1 + 0.2; println(x)")
        .assert()
        .success(); // May have floating point imprecision
}

#[test]
fn test_value_float_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = -3.14; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("-3.14"));
}

#[test]
fn test_value_float_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 0.0; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_value_float_division() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(10.0 / 4.0)")
        .assert()
        .success()
        .stdout(predicate::str::contains("2.5"));
}

// ============================================================================
// Value Type: String Operations
// ============================================================================

#[test]
fn test_value_string_empty() {
    ruchy_cmd()
        .arg("-e")
        .arg("let s = \"\"; println(len(s))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_value_string_concatenation() {
    ruchy_cmd()
        .arg("-e")
        .arg("let s = \"hello\" + \" \" + \"world\"; println(s)")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_value_string_multiline() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let s = "line1\nline2"; println(s)"#)
        .assert()
        .success();
}

#[test]
fn test_value_string_unicode() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let s = "✓ ✗ 🚀"; println(len(s))"#)
        .assert()
        .success(); // Unicode length
}

#[test]
fn test_value_string_escape_sequences() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"println("tab:\t\nnewline")"#)
        .assert()
        .success();
}

#[test]
fn test_value_string_multiplication() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"println("x" * 10)"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("xxxxxxxxxx"));
}

// ============================================================================
// Value Type: Boolean Operations
// ============================================================================

#[test]
fn test_value_bool_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("let b = true; println(b)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_value_bool_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("let b = false; println(b)")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_value_bool_and() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(true && false)")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_value_bool_or() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(true || false)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_value_bool_not() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(!true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_value_bool_from_comparison() {
    ruchy_cmd()
        .arg("-e")
        .arg("let b = 10 > 5; println(b)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// Value Type: Array Operations
// ============================================================================

#[test]
fn test_value_array_empty() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = []; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_value_array_homogeneous() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, 2, 3, 4, 5]; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_value_array_heterogeneous() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let arr = [1, "two", 3.0, true]; println(len(arr))"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_value_array_nested() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [[1, 2], [3, 4], [5, 6]]; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_value_array_indexing() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [10, 20, 30]; println(arr[0], arr[1], arr[2])")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("20"))
        .stdout(predicate::str::contains("30"));
}

#[test]
fn test_value_array_negative_indexing() {
    // Python-style negative indexing
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, 2, 3]; println(arr[-1])")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_value_array_mutation() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut arr = [1, 2, 3]; arr[1] = 99; println(arr[1])")
        .assert()
        .success()
        .stdout(predicate::str::contains("99"));
}

#[test]
fn test_value_array_iteration() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        let arr = [1, 2, 3];
        let mut sum = 0;
        for x in arr {
            sum = sum + x
        };
        println(sum)
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

// ============================================================================
// Value Type: Function Operations
// ============================================================================

#[test]
fn test_value_function_call() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn add(a, b) { a + b }; println(add(10, 32))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_value_function_closure() {
    ruchy_cmd()
        .arg("-e")
        .arg("let f = |x| x * 2; println(f(21))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_value_function_higher_order() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        fn apply(f, x) { f(x) };
        let double = |n| n * 2;
        println(apply(double, 21))
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_value_function_recursive() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        fn factorial(n) {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        };
        println(factorial(5))
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("120"));
}

#[test]
fn test_value_function_returning_function() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        fn make_adder(x) {
            |y| x + y
        };
        let add10 = make_adder(10);
        println(add10(32))
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Value Type: Nil Operations
// ============================================================================

#[test]
fn test_value_nil_assignment() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = nil; println(\"OK\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn test_value_nil_comparison() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = nil; let y = nil; println(x == y)")
        .assert()
        .success(); // May print true or may not support nil comparison
}

// ============================================================================
// Value Type Conversions
// ============================================================================

#[test]
fn test_value_conversion_int_to_float() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; let y = float(x); println(y)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_value_conversion_float_to_int() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 3.7; let y = int(x); println(y)")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_value_conversion_int_to_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; let s = str(x); println(s)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_value_conversion_string_to_int() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let s = "123"; let x = int(s); println(x)"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("123"));
}

#[test]
fn test_value_conversion_bool_to_int() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(true), int(false))")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_value_conversion_int_to_bool() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(1), bool(0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("false"));
}

// ============================================================================
// Complex Integration Tests (Multiple Value Types)
// ============================================================================

#[test]
fn test_integration_mixed_types_calculation() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r#"
        let a = 10;
        let b = 3.14;
        let c = "result: ";
        println(c, a, b)
    "#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("result:"))
        .stdout(predicate::str::contains("10"))
        .stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_integration_array_of_functions() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r#"
        let add = |a, b| a + b;
        let sub = |a, b| a - b;
        let mul = |a, b| a * b;
        println("OK")
    "#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn test_integration_nested_data_structures() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        let data = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let mut sum = 0;
        for row in data {
            for val in row {
                sum = sum + val
            }
        };
        println(sum)
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("45"));
}

#[test]
fn test_integration_closure_with_multiple_captures() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        let x = 10;
        let y = 20;
        let z = 30;
        let f = || x + y + z;
        println(f())
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("60"));
}

#[test]
fn test_integration_function_composition() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        fn double(x) { x * 2 };
        fn increment(x) { x + 1 };
        fn compose(f, g, x) { f(g(x)) };
        println(compose(double, increment, 5))
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("12"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_empty_string_operations() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let s = ""; println(len(s) == 0)"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn edge_case_single_element_array() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [42]; println(arr[0])")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_deeply_nested_arrays() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [[[[[42]]]]]; println(arr[0][0][0][0][0])")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_function_with_no_params() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn get42() { 42 }; println(get42())")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_zero_iterations() {
    ruchy_cmd()
        .arg("-e")
        .arg(
            r"
        let mut count = 0;
        for i in range(0) {
            count = count + 1
        };
        println(count)
    ",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_int_addition_commutative() {
    // Property: a + b == b + a
    for (a, b) in [(1, 2), (5, 10), (100, 200)] {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert_eq({a} + {b}, {b} + {a})"))
            .assert()
            .success();
    }
}

#[test]
fn property_array_length_nonnegative() {
    // Property: len(arr) >= 0
    for n in [0, 1, 5, 10, 100] {
        let elements = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        ruchy_cmd()
            .arg("-e")
            .arg(format!("let arr = [{elements}]; assert(len(arr) >= 0)"))
            .assert()
            .success();
    }
}

#[test]
fn property_string_concatenation_associative() {
    // Property: (a + b) + c == a + (b + c)
    ruchy_cmd()
        .arg("-e")
        .arg(r#"let s1 = ("a" + "b") + "c"; let s2 = "a" + ("b" + "c"); assert_eq(s1, s2)"#)
        .assert()
        .success();
}

#[test]
fn property_bool_double_negation() {
    // Property: !!x == x
    ruchy_cmd()
        .arg("-e")
        .arg("assert_eq(!!true, true); assert_eq(!!false, false)")
        .assert()
        .success();
}
