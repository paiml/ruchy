#![allow(missing_docs)]
// RUNTIME-038: Variable Collision in Nested Function Calls with Tuple Unpacking
//
// Bug: Variable names in outer scope collide with variable names in nested call stacks,
// causing runtime type corruption. A String variable can be replaced with an i32 value
// from a completely different function scope.
//
// GitHub Issue: https://github.com/paiml/ruchy/issues/38
// Priority: HIGH - Type safety violation
// Discovered: 2025-10-19 during RuchyRuchy bootstrap compiler testing
//
// FIX: interpreter.rs:1821 - env_set() now creates bindings ONLY in current scope (shadowing)
// Complexity: Reduced from 4 to 1 (within Toyota Way limits)

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property test: Nested functions with same variable names should not leak
    /// Tests 10K+ random combinations of nested calls
    #[test]
    #[ignore = "Run with: cargo test property_tests -- --ignored"]
    fn prop_nested_variables_isolated() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for iteration in 0..10000 {
            let var_name = format!("var_{}", rng.gen::<u8>() % 10); // Common variable names
            let outer_value = rng.gen::<i32>();
            let inner_value = rng.gen::<i32>();

            let code = format!(
                r"
fun inner() -> (i32, i32) {{
    let {var_name} = {inner_value};
    ({inner_value}, {var_name})
}}

fun main() {{
    let {var_name} = {outer_value};
    let result = inner();
    assert_eq({var_name}, {outer_value});  // Outer variable should NOT change!
}}
"
            );

            let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
            temp_file
                .write_all(code.as_bytes())
                .expect("Failed to write temp file");

            assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                .arg("run")
                .arg(temp_file.path())
                .assert()
                .success()
                .stderr(predicate::str::is_empty());

            if iteration % 1000 == 0 {
                println!("Property test iteration: {iteration}/10000");
            }
        }

        println!("✅ Property test passed: 10,000 scope isolation tests");
    }
}

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Minimal reproduction from Issue #38
///
/// Expected: Variable 'a' in `main()` should be a String
/// Actual: Variable 'a' corrupted to integer (1103515245) from nested function
#[test]
fn test_runtime038_minimal_reproduction() {
    let code = r#"
fun next_random(seed: i32) -> i32 {
    let a = 1103515245;
    let c = 12345;
    let m = 2147483647;

    let temp = a * seed + c;
    if temp < 0 {
        (temp + m) % m
    } else {
        temp % m
    }
}

fun random_in_range(seed: i32, max: i32) -> (i32, i32) {
    let new_seed = next_random(seed);
    let value = if max > 0 {
        if new_seed < 0 {
            ((new_seed + 2147483647) % max)
        } else {
            new_seed % max
        }
    } else {
        0
    };
    (value, new_seed)
}

fun random_string(seed: i32, max_len: i32) -> (String, i32) {
    let result = random_in_range(seed, 100);
    let num = result.0;
    let new_seed = result.1;

    if num < 10 {
        ("x".to_string(), new_seed)
    } else if num < 20 {
        ("xy".to_string(), new_seed)
    } else {
        ("hello".to_string(), new_seed)
    }
}

fun main() {
    let r1 = random_string(42, 5);
    let a = r1.0;
    let seed1 = r1.1;

    let r2 = random_string(seed1, 5);
    let b = r2.0;

    println("a = {}", a);
    println("b = {}", b);

    let result = a + b;
    println("result = {}", result);
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_file.path();

    // This should succeed but currently fails with type corruption
    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("a = \"hello\"")) // String with quotes
        .stdout(predicate::str::contains("b = \"hello\""))
        .stdout(predicate::str::contains("result = \"hellohello\"")); // Concatenation works!
}

/// Regression test for variable collision bug (Issue #38)
///
/// Test that inner function variables do NOT affect outer scope
#[test]
fn test_runtime038_regression_simple() {
    let code = r#"
fun inner() -> (String, i32) {
    let a = 1103515245;
    ("test".to_string(), a)
}

fun main() {
    let result = inner();
    let a = result.0;
    let num = result.1;

    assert_eq(a, "test");
    assert_eq(num, 1103515245);
    println("PASS: Variable 'a' correctly preserved as String");
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

/// Test interpreter mode with same bug
#[test]
fn test_runtime038_interpreter_mode() {
    let code = r#"
fun inner() -> (String, i32) {
    let a = 999;
    ("string_value".to_string(), a)
}

let result = inner()
let a = result.0
let num = result.1

assert_eq(a, "string_value")
assert_eq(num, 999)
println("Interpreter PASS")
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Interpreter PASS"));
}

/// Test with common variable names (a, b, c, i, x, y, z)
/// These are likely to collide in real code
#[test]
fn test_runtime038_common_variable_names() {
    let code = r#"
fun nested_a() -> (String, i32) {
    let a = 100;
    ("value_a".to_string(), a)
}

fun nested_b() -> (String, i32) {
    let b = 200;
    ("value_b".to_string(), b)
}

fun main() {
    let r1 = nested_a();
    let a = r1.0;
    let num_a = r1.1;

    let r2 = nested_b();
    let b = r2.0;
    let num_b = r2.1;

    assert_eq(a, "value_a");
    assert_eq(num_a, 100);
    assert_eq(b, "value_b");
    assert_eq(num_b, 200);

    println("PASS: All variables preserved correctly");
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

/// Test deeply nested function calls (3+ levels)
#[test]
fn test_runtime038_deeply_nested() {
    let code = r#"
fun level3() -> (String, i32) {
    let a = 300;
    ("level3".to_string(), a)
}

fun level2() -> (String, i32) {
    let result = level3();
    let a = result.0;
    let num = result.1;
    (a, num)
}

fun level1() -> (String, i32) {
    let result = level2();
    let a = result.0;
    let num = result.1;
    (a, num)
}

fun main() {
    let result = level1();
    let a = result.0;
    let num = result.1;

    assert_eq(a, "level3");
    assert_eq(num, 300);
    println("PASS: Deep nesting preserved scope");
}
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_file.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}
