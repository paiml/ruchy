//! EXTREME TDD: Runtime Interpreter Comprehensive Coverage
//!
//! Target: runtime/interpreter.rs execution paths (high-impact module)
//! Strategy: Exercise interpreter via eval, covering expression/statement evaluation
//! Coverage: Control flow, variable scoping, closures, error handling

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Variable Scoping & Environment
// ============================================================================

#[test]
fn test_global_scope() {
    ruchy_cmd().arg("-e").arg("let x = 10; println(x)")
        .assert().success().stdout(predicate::str::contains("10"));
}

#[test]
fn test_local_scope_function() {
    ruchy_cmd().arg("-e").arg("fn test() { let x = 20; x }; println(test())")
        .assert().success().stdout(predicate::str::contains("20"));
}

#[test]
fn test_shadowing() {
    ruchy_cmd().arg("-e").arg("let x = 10; let x = 20; println(x)")
        .assert().success().stdout(predicate::str::contains("20"));
}

#[test]
fn test_closure_captures_environment() {
    ruchy_cmd().arg("-e").arg("let x = 42; let f = || x; println(f())")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_nested_scopes() {
    ruchy_cmd().arg("-e").arg("let x = 1; fn outer() { let x = 2; fn inner() { let x = 3; x }; inner() }; println(outer())")
        .assert().success().stdout(predicate::str::contains("3"));
}

// ============================================================================
// Expression Evaluation
// ============================================================================

#[test]
fn test_eval_literal_integer() {
    ruchy_cmd().arg("-e").arg("println(42)")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_literal_float() {
    ruchy_cmd().arg("-e").arg("println(3.14)")
        .assert().success().stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_eval_literal_string() {
    ruchy_cmd().arg("-e").arg("println(\"test\")")
        .assert().success().stdout(predicate::str::contains("test"));
}

#[test]
fn test_eval_literal_boolean() {
    ruchy_cmd().arg("-e").arg("println(true, false)")
        .assert().success()
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_eval_arithmetic_expression() {
    ruchy_cmd().arg("-e").arg("println(10 + 5 * 2)")
        .assert().success().stdout(predicate::str::contains("20"));
}

#[test]
fn test_eval_comparison_expression() {
    ruchy_cmd().arg("-e").arg("println(10 > 5)")
        .assert().success().stdout(predicate::str::contains("true"));
}

#[test]
fn test_eval_logical_expression() {
    ruchy_cmd().arg("-e").arg("println(true && false)")
        .assert().success().stdout(predicate::str::contains("false"));
}

// ============================================================================
// Control Flow Evaluation
// ============================================================================

#[test]
fn test_eval_if_true_branch() {
    ruchy_cmd().arg("-e").arg("if true { println(\"yes\") }")
        .assert().success().stdout(predicate::str::contains("yes"));
}

#[test]
fn test_eval_if_false_branch() {
    ruchy_cmd().arg("-e").arg("if false { println(\"no\") } else { println(\"yes\") }")
        .assert().success().stdout(predicate::str::contains("yes"));
}

#[test]
fn test_eval_nested_if() {
    ruchy_cmd().arg("-e").arg("if true { if true { println(\"nested\") } }")
        .assert().success().stdout(predicate::str::contains("nested"));
}

#[test]
fn test_eval_match_literal() {
    ruchy_cmd().arg("-e").arg("let x = match 2 { 1 => 10, 2 => 20, _ => 30 }; println(x)")
        .assert().success().stdout(predicate::str::contains("20"));
}

#[test]
fn test_eval_match_default() {
    ruchy_cmd().arg("-e").arg("let x = match 5 { 1 => 10, 2 => 20, _ => 30 }; println(x)")
        .assert().success().stdout(predicate::str::contains("30"));
}

#[test]
fn test_eval_for_loop() {
    ruchy_cmd().arg("-e").arg("let mut sum = 0; for i in range(5) { sum = sum + i }; println(sum)")
        .assert().success().stdout(predicate::str::contains("10"));
}

#[test]
fn test_eval_while_loop() {
    ruchy_cmd().arg("-e").arg("let mut i = 0; while i < 5 { i = i + 1 }; println(i)")
        .assert().success().stdout(predicate::str::contains("5"));
}

#[test]
fn test_eval_loop_with_break() {
    ruchy_cmd().arg("-e").arg("let mut i = 0; loop { i = i + 1; if i > 3 { break } }; println(i)")
        .assert().success().stdout(predicate::str::contains("4"));
}

// ============================================================================
// Function Calls & Returns
// ============================================================================

#[test]
fn test_eval_function_call_no_args() {
    ruchy_cmd().arg("-e").arg("fn get42() { 42 }; println(get42())")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_function_call_with_args() {
    ruchy_cmd().arg("-e").arg("fn add(a, b) { a + b }; println(add(10, 32))")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_function_early_return() {
    ruchy_cmd().arg("-e").arg("fn test() { return 42; 99 }; println(test())")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_recursive_function() {
    ruchy_cmd().arg("-e").arg("fn fact(n) { if n <= 1 { 1 } else { n * fact(n-1) } }; println(fact(5))")
        .assert().success().stdout(predicate::str::contains("120"));
}

#[test]
fn test_eval_closure() {
    ruchy_cmd().arg("-e").arg("let add5 = |x| x + 5; println(add5(10))")
        .assert().success().stdout(predicate::str::contains("15"));
}

#[test]
fn test_eval_higher_order_function() {
    ruchy_cmd().arg("-e").arg("fn apply(f, x) { f(x) }; let double = |n| n * 2; println(apply(double, 21))")
        .assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Array Operations
// ============================================================================

#[test]
fn test_eval_array_creation() {
    ruchy_cmd().arg("-e").arg("let arr = [1, 2, 3]; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("3"));
}

#[test]
fn test_eval_array_index() {
    ruchy_cmd().arg("-e").arg("let arr = [10, 20, 30]; println(arr[1])")
        .assert().success().stdout(predicate::str::contains("20"));
}

#[test]
fn test_eval_array_mutation() {
    ruchy_cmd().arg("-e").arg("let mut arr = [1, 2, 3]; arr[1] = 99; println(arr[1])")
        .assert().success().stdout(predicate::str::contains("99"));
}

#[test]
fn test_eval_nested_array() {
    ruchy_cmd().arg("-e").arg("let arr = [[1, 2], [3, 4]]; println(arr[0][1])")
        .assert().success().stdout(predicate::str::contains("2"));
}

// ============================================================================
// String Interpolation
// ============================================================================

#[test]
fn test_eval_f_string_single_var() {
    ruchy_cmd().arg("-e").arg("let x = 42; println(f\"value: {x}\")")
        .assert().success()
        .stdout(predicate::str::contains("value:"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_f_string_multiple_vars() {
    ruchy_cmd().arg("-e").arg("let a = 1; let b = 2; println(f\"{a} + {b} = {a+b}\")")
        .assert().success();
}

#[test]
fn test_eval_f_string_with_expression() {
    ruchy_cmd().arg("-e").arg("let n = 5; println(f\"{n} squared is {n*n}\")")
        .assert().success()
        .stdout(predicate::str::contains("5"))
        .stdout(predicate::str::contains("25"));
}

// ============================================================================
// Error Handling Paths
// ============================================================================

#[test]
fn test_eval_undefined_variable() {
    ruchy_cmd().arg("-e").arg("println(nonexistent)")
        .assert().failure().stderr(predicate::str::contains("Undefined variable"));
}

#[test]
fn test_eval_division_by_zero() {
    ruchy_cmd().arg("-e").arg("println(10 / 0)")
        .assert().failure();
}

#[test]
fn test_eval_index_out_of_bounds() {
    ruchy_cmd().arg("-e").arg("let arr = [1, 2, 3]; println(arr[10])")
        .assert().failure();
}

#[test]
fn test_eval_string_multiplication() {
    // String multiplication IS supported (not invalid)
    ruchy_cmd().arg("-e").arg("println(\"hello\" * 5)")
        .assert().success().stdout(predicate::str::contains("hellohellohellohellohello"));
}

// ============================================================================
// Type Coercion & Conversions
// ============================================================================

#[test]
fn test_eval_int_to_float() {
    ruchy_cmd().arg("-e").arg("let x = 42; let y = float(x); println(y)")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn test_eval_string_to_int() {
    ruchy_cmd().arg("-e").arg("let s = \"123\"; let n = int(s); println(n)")
        .assert().success().stdout(predicate::str::contains("123"));
}

#[test]
fn test_eval_bool_to_int() {
    ruchy_cmd().arg("-e").arg("println(int(true), int(false))")
        .assert().success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("0"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_variable_assignment() {
    // Property: Assigned values are retrievable
    for val in [0, 42, -100, 999] {
        ruchy_cmd().arg("-e").arg(format!("let x = {val}; assert_eq(x, {val})"))
            .assert().success();
    }
}

#[test]
fn property_function_return_values() {
    // Property: Functions return correct values
    for val in [1, 10, 100] {
        ruchy_cmd().arg("-e").arg(format!("fn f() {{ {val} }}; assert_eq(f(), {val})"))
            .assert().success();
    }
}

#[test]
fn property_array_length() {
    // Property: Array length matches element count
    for n in 0..10 {
        let elements = (0..n).map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
        ruchy_cmd().arg("-e").arg(format!("let arr = [{elements}]; assert_eq(len(arr), {n})"))
            .assert().success();
    }
}

#[test]
fn property_closure_captures() {
    // Property: Closures capture environment variables
    for val in [5, 10, 20] {
        ruchy_cmd().arg("-e").arg(format!("let x = {val}; let f = || x; assert_eq(f(), {val})"))
            .assert().success();
    }
}

// ============================================================================
// Integration Tests (Complex Scenarios)
// ============================================================================

#[test]
fn integration_factorial_iterative() {
    ruchy_cmd().arg("-e").arg(r"
        fn factorial(n) {
            let mut result = 1;
            let mut i = 1;
            while i <= n {
                result = result * i;
                i = i + 1
            };
            result
        };
        println(factorial(6))
    ").assert().success().stdout(predicate::str::contains("720"));
}

#[test]
fn integration_fibonacci_recursive() {
    ruchy_cmd().arg("-e").arg(r"
        fn fib(n) {
            if n <= 1 {
                n
            } else {
                fib(n-1) + fib(n-2)
            }
        };
        println(fib(10))
    ").assert().success().stdout(predicate::str::contains("55"));
}

#[test]
fn integration_array_sum() {
    ruchy_cmd().arg("-e").arg(r"
        let arr = [1, 2, 3, 4, 5];
        let mut sum = 0;
        for x in arr {
            sum = sum + x
        };
        println(sum)
    ").assert().success().stdout(predicate::str::contains("15"));
}

#[test]
fn integration_nested_functions() {
    ruchy_cmd().arg("-e").arg(r"
        fn outer(x) {
            fn inner(y) {
                x + y
            };
            inner(10)
        };
        println(outer(32))
    ").assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn integration_closure_with_state() {
    ruchy_cmd().arg("-e").arg(r"
        let x = 10;
        let add_x = |y| x + y;
        println(add_x(32))
    ").assert().success().stdout(predicate::str::contains("42"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_zero_iterations() {
    ruchy_cmd().arg("-e").arg("for i in range(0) { println(\"never\") }; println(\"done\")")
        .assert().success().stdout(predicate::str::contains("done"));
}

#[test]
fn edge_case_deeply_nested_calls() {
    ruchy_cmd().arg("-e").arg("fn a() { b() }; fn b() { c() }; fn c() { 42 }; println(a())")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_empty_array() {
    ruchy_cmd().arg("-e").arg("let arr = []; println(len(arr))")
        .assert().success().stdout(predicate::str::contains("0"));
}

#[test]
fn edge_case_single_element_array() {
    ruchy_cmd().arg("-e").arg("let arr = [42]; println(arr[0])")
        .assert().success().stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_recursion_depth_limit() {
    // Recursion depth limit is 100, so count(100) fails
    // Test with count(50) which should succeed
    ruchy_cmd().arg("-e").arg("fn count(n) { if n == 0 { 0 } else { 1 + count(n-1) } }; println(count(50))")
        .assert().success().stdout(predicate::str::contains("50"));
}

#[test]
fn edge_case_recursion_limit_exceeded() {
    // Recursion depth 100 should fail
    ruchy_cmd().arg("-e").arg("fn count(n) { if n == 0 { 0 } else { 1 + count(n-1) } }; println(count(100))")
        .assert().failure().stderr(predicate::str::contains("Recursion limit"));
}
