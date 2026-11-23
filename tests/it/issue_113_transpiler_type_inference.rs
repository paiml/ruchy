//! ISSUE-113: Critical transpiler type inference bugs
//!
//! GitHub Issue: <https://github.com/paiml/ruchy/issues/113>
//! Severity: CRITICAL - Blocks transpile and compile modes for real-world code
//!
//! Bugs discovered through BENCH-008 (Prime Generation) scientific benchmarking:
//! 1. Boolean return types incorrectly inferred as `i32`
//! 2. Integer parameters incorrectly inferred as `&str`
//! 3. Vector return types incorrectly inferred as `i32`

use predicates::prelude::*;

/// Bug 1: Boolean return type inference
/// Expected: `fn is_prime(n: i32) -> bool`
/// Actual: `fn is_prime(n: i32) -> i32`
/// Error: E0308 - mismatched types (expected i32, found bool)
#[test]
fn test_issue_113_bug_1_boolean_return_type() {
    let input = r"
fun is_prime(n) {
    if n < 2 {
        return false
    }
    if n == 2 {
        return true
    }
    if n % 2 == 0 {
        return false
    }
    true
}
";

    // Transpile and check the generated Rust code
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn is_prime(n: i32) -> bool"))
        .stdout(predicate::str::contains("-> i32").not()); // Should NOT infer i32
}

/// Bug 1 variant: Simple boolean function
#[test]
fn test_issue_113_bug_1_simple_boolean() {
    let input = r"
fun is_even(n) {
    n % 2 == 0
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn is_even(n: i32) -> bool"));
}

/// Bug 2: Integer parameter type inference
/// Expected: `fn generate_primes(count: i32) -> Vec<i32>`
/// Actual: `fn generate_primes(count: &str) -> i32`
/// Error: E0308 - cannot compare usize with &str
#[test]
fn test_issue_113_bug_2_integer_parameter_in_comparison() {
    let input = r"
fun count_up_to(limit) {
    let mut i = 0
    while i < limit {
        i = i + 1
    }
    i
}
";

    // Parameter `limit` is used in comparison (i < limit), should be i32
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn count_up_to(limit: i32)"))
        .stdout(predicate::str::contains("limit: &str").not()); // Should NOT infer &str
}

/// Bug 2 variant: Parameter used in `Vec.len()` comparison
#[test]
fn test_issue_113_bug_2_vec_len_comparison() {
    let input = r"
fun fill_array(count) {
    let mut arr = []
    while arr.len() < count {
        arr.push(1)
    }
    arr
}
";

    // Parameter `count` is compared with arr.len() (usize), should infer i32/usize
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn fill_array(count: i32)"))
        .stdout(predicate::str::contains("count: &str").not());
}

/// Bug 3: Vector return type inference
/// Expected: `fn generate_primes(...) -> Vec<i32>`
/// Actual: `fn generate_primes(...) -> i32`
/// Error: E0308 - expected i32, found Vec<i32>
#[test]
fn test_issue_113_bug_3_vector_return_type() {
    let input = r"
fun make_array() {
    let mut arr = []
    arr.push(1)
    arr.push(2)
    arr.push(3)
    arr
}
";

    // Function returns array literal with .push(), should be Vec<i32>
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn make_array() -> Vec<i32>"))
        .stdout(predicate::str::contains("-> i32").count(0)); // Should NOT be -> i32
}

/// Bug 3 variant: Empty array return
#[test]
fn test_issue_113_bug_3_empty_array_return() {
    let input = r"
fun empty_array() {
    []
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn empty_array() -> Vec"));
}

/// Integration test: Full BENCH-008 prime generation reproducer
#[test]
fn test_issue_113_full_reproducer_bench_008() {
    let input = r"
fun is_prime(n) {
    if n < 2 {
        return false
    }
    if n == 2 {
        return true
    }
    if n % 2 == 0 {
        return false
    }

    let mut i = 3
    while i * i <= n {
        if n % i == 0 {
            return false
        }
        i = i + 2
    }
    true
}

fun generate_primes(count) {
    let mut primes = []
    let mut candidate = 2

    while primes.len() < count {
        if is_prime(candidate) {
            primes.push(candidate)
        }
        candidate = candidate + 1
    }

    primes
}

fun main() {
    let primes = generate_primes(100)
}
";

    // Full transpilation should succeed with correct types
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn is_prime(n: i32) -> bool"))
        .stdout(predicate::str::contains(
            "fn generate_primes(count: i32) -> Vec<i32>",
        ));
}

/// Compilation test: Transpiled code should compile with rustc
#[test]
fn test_issue_113_transpiled_code_compiles() {
    use std::fs;
    use std::process::Command as StdCommand;

    let input = r"
fun is_prime(n) {
    if n < 2 { return false }
    if n == 2 { return true }
    true
}

fun main() {
    let result = is_prime(7)
}
";

    // Transpile to Rust
    let output = assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(output).unwrap();

    // Write to temp file
    let temp_file = "/tmp/issue_113_test.rs";
    fs::write(temp_file, rust_code).unwrap();

    // Attempt to compile with rustc
    let compile_result = StdCommand::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg(temp_file)
        .arg("-o")
        .arg("/tmp/issue_113_test")
        .output()
        .unwrap();

    // Cleanup
    let _ = fs::remove_file(temp_file);
    let _ = fs::remove_file("/tmp/issue_113_test");

    // Should compile without errors
    assert!(
        compile_result.status.success(),
        "Transpiled code failed to compile:\n{}",
        String::from_utf8_lossy(&compile_result.stderr)
    );
}
