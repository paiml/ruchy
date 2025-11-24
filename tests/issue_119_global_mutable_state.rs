// Issue #119: Global mutable state not persisting across function calls
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// Blocking: BENCH-002 (Matrix Multiplication)
//
// ROOT CAUSE: Double-clone in function definition + function call
// - Line 31 (eval_func.rs): env: Arc::new(current_env.clone())
// - Line 274 (eval_function.rs): let mut call_env = closure.captured_env.clone();
// FIX: Pass mutable reference to parent scope instead of cloning
//
// HYPOTHESIS CONFIRMED: Scope Copy
// - Functions work with cloned environment (snapshot at definition time)
// - Modifications inside function don't propagate back to parent scope
// - Each function call gets fresh copy of captured environment

use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_issue_119_01_simple_global_counter() {
    // RED: This test WILL FAIL until global mutable state is fixed
    let code = r"
        let mut global_counter = 0

        fun increment() {
            global_counter = global_counter + 1
        }

        increment()
        println(global_counter)  // Expected: 1
        increment()
        println(global_counter)  // Expected: 2
        increment()
        println(global_counter)  // Expected: 3
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_issue_119_02_multiple_functions_same_global() {
    // RED: Multiple functions accessing same global variable
    let code = r"
        let mut counter = 0

        fun increment() {
            counter = counter + 1
        }

        fun decrement() {
            counter = counter - 1
        }

        increment()
        println(counter)  // Expected: 1
        increment()
        println(counter)  // Expected: 2
        decrement()
        println(counter)  // Expected: 1
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_issue_119_03_nested_function_calls() {
    // RED: Nested functions with global modifications
    let code = r"
        let mut total = 0

        fun add_five() {
            total = total + 5
        }

        fun add_ten() {
            add_five()
            add_five()
        }

        add_ten()
        println(total)  // Expected: 10
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn test_issue_119_04_global_array_mutation() {
    // RED: Global array mutations (needed for BENCH-002)
    let code = r"
        let mut results = []

        fun append_value(val) {
            results = results + [val]
        }

        append_value(10)
        append_value(20)
        append_value(30)
        println(len(results))  // Expected: 3
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_issue_119_05_global_state_across_loop() {
    // RED: Global state maintained across loop iterations
    let code = r"
        let mut sum = 0

        fun add(n) {
            sum = sum + n
        }

        let mut i = 1
        while i <= 5 {
            add(i)
            i = i + 1
        }

        println(sum)  // Expected: 15 (1+2+3+4+5)
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));
}

#[test]
fn test_issue_119_06_recursive_function_global_state() {
    // RED: Recursive functions with global accumulator
    let code = r"
        let mut factorial_result = 1

        fun factorial(n) {
            if n <= 1 {
                factorial_result
            } else {
                factorial_result = factorial_result * n
                factorial(n - 1)
            }
        }

        factorial(5)
        println(factorial_result)  // Expected: 120
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("120"));
}

#[test]
fn test_issue_119_07_bench_002_matrix_pattern() {
    // RED: BENCH-002 pattern - matrix multiplication requires global state
    let file = NamedTempFile::new().unwrap();
    let code = r"
// Simplified BENCH-002 pattern
let mut matrix_result = []

fun process_cell(i, j, a, b) {
    let mut sum = 0
    let mut k = 0
    while k < 2 {
        sum = sum + (a[i][k] * b[k][j])
        k = k + 1
    }
    matrix_result = matrix_result + [sum]
}

let a = [[1, 2], [3, 4]]
let b = [[5, 6], [7, 8]]

// Process single cell
process_cell(0, 0, a, b)
println(matrix_result[0])  // Expected: 19 (1*5 + 2*7)
    ";
    fs::write(file.path(), code).unwrap();

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("19"));
}

#[test]
fn test_issue_119_08_multiple_globals() {
    // RED: Multiple global variables with independent state
    let code = r"
        let mut x = 0
        let mut y = 0

        fun update_x() {
            x = x + 1
        }

        fun update_y() {
            y = y + 2
        }

        update_x()
        update_y()
        println(x)  // Expected: 1
        println(y)  // Expected: 2
    ";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

// ============================================================================
// Property-Based Tests (VALIDATE Phase)
// ============================================================================

#[cfg(test)]
mod property_tests {

    use proptest::prelude::*;

    // Property 1: Global mutations are visible after function returns
    proptest! {
        #[test]
        fn prop_global_mutation_visible(initial_value in 0i32..100, increment in 1i32..10) {
            let code = format!(
                r"
                let mut counter = {initial_value}
                fun add_value() {{
                    counter = counter + {increment}
                }}
                add_value()
                println(counter)
                "
            );

            let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("-e")
                .arg(&code)
                .output()
                .unwrap();

            let stdout = String::from_utf8_lossy(&output.stdout);
            let expected = initial_value + increment;
            prop_assert!(
                stdout.contains(&expected.to_string()),
                "Expected {} but got: {}",
                expected,
                stdout
            );
        }
    }

    // Property 2: Multiple function calls accumulate correctly
    proptest! {
        #[test]
        fn prop_multiple_calls_accumulate(num_calls in 1usize..10, increment_per_call in 1i32..5) {
            let code = format!(
                r"
                let mut sum = 0
                fun add() {{
                    sum = sum + {}
                }}
                {}
                println(sum)
                ",
                increment_per_call,
                "add()\n".repeat(num_calls)
            );

            let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("-e")
                .arg(&code)
                .output()
                .unwrap();

            let stdout = String::from_utf8_lossy(&output.stdout);
            let expected = (num_calls as i32) * increment_per_call;
            prop_assert!(
                stdout.contains(&expected.to_string()),
                "Expected {} ({}×{}) but got: {}",
                expected,
                num_calls,
                increment_per_call,
                stdout
            );
        }
    }

    // Property 3: State persists across arbitrary sequences of operations
    proptest! {
        #[test]
        fn prop_state_persistence(operations in prop::collection::vec(0i32..10, 1..5)) {
            let calls = operations
                .iter()
                .map(|op| format!("add({op})"))
                .collect::<Vec<_>>()
                .join("\n");

            let code = format!(
                r"
                let mut total = 0
                fun add(n) {{
                    total = total + n
                }}
                {calls}
                println(total)
                "
            );

            let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("-e")
                .arg(&code)
                .output()
                .unwrap();

            let stdout = String::from_utf8_lossy(&output.stdout);
            let expected: i32 = operations.iter().sum();
            prop_assert!(
                stdout.contains(&expected.to_string()),
                "Expected {} (sum of {:?}) but got: {}",
                expected,
                operations,
                stdout
            );
        }
    }
}
