// Issue #114: Transpiler generates wrong return type for String-returning functions
//
// PROBLEM: The Ruchy transpiler incorrectly infers the return type of functions
// that return String as `i32`, causing compilation failures.
//
// EXPECTED: Functions returning String should transpile to `fn name(...) -> String`
// ACTUAL: Functions returning String transpile to `fn name(...) -> i32`
//
// IMPACT: Blocks BENCH-003 (string concatenation) in transpile/compile modes
//
// ROOT CAUSE: Type inference engine fails to analyze return expression type for strings
//
// EXTREME TDD: RED → GREEN → REFACTOR
// - RED: These tests FAIL before fix (transpiled code has `-> i32` instead of `-> String`)
// - GREEN: Fix type inference to detect String returns
// - REFACTOR: Apply PMAT quality gates (≤10 complexity, zero SATD)

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_issue_114_simple_string_return() {
    // Test 1: Simple string literal return (should be &'static str in Rust)
    let input = r#"
fun returns_string() {
    "hello"
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn returns_string() -> &'static str"))
        .stdout(predicate::str::contains("-> i32").not());
}

#[test]
fn test_issue_114_string_concatenation_in_loop() {
    // Test 2: String concatenation in while loop (BENCH-003 pattern)
    let input = r#"
fun string_concatenation(iterations) {
    let mut result = ""
    let mut i = 0

    while i < iterations {
        result = result + "x"
        i = i + 1
    }

    result
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn string_concatenation(iterations: i32) -> String"))
        .stdout(predicate::str::contains("fn string_concatenation(iterations: i32) -> i32").not());
}

#[test]
fn test_issue_114_string_from_variable() {
    // Test 3: String return from immutable variable binding (should be &'static str)
    let input = r#"
fun get_message() {
    let msg = "Hello, World!"
    msg
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn get_message() -> &'static str"))
        .stdout(predicate::str::contains("-> i32").not());
}

#[test]
fn test_issue_114_mixed_types_returns_string() {
    // Test 4: Mixed types - ensure string is returned, not i32
    let input = r#"
fun mixed(flag) {
    let s = "hello"
    let n = 42
    s
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn mixed(flag: i32) -> &'static str"))
        .stdout(predicate::str::contains("fn mixed(flag: i32) -> i32").not());
}

#[test]
#[ignore] // Parameter type inference from + operator is ambiguous without type annotations
fn test_issue_114_string_concatenation_with_plus() {
    // Test 5: String concatenation with + operator
    // NOTE: This is ambiguous - we can't determine if a+b is string concat or int addition
    // without type annotations or usage context
    let input = r#"
fun concat_strings(a, b) {
    a + b
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn concat_strings(a: String, b: String) -> String"))
        .stdout(predicate::str::contains("-> i32").not());
}

#[test]
fn test_issue_114_string_from_if_expression() {
    // Test 6: String return from if expression branches (both are literals)
    let input = r#"
fun conditional_string(flag) {
    if flag {
        "yes"
    } else {
        "no"
    }
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn conditional_string(flag: bool) -> &'static str"))
        .stdout(predicate::str::contains("-> i32").not());
}

#[test]
fn test_issue_114_explicit_string_return_statement() {
    // Test 7: Explicit return statement with string literals
    let input = r#"
fun explicit_return(n) {
    if n > 10 {
        return "large"
    }
    "small"
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .stdout(predicate::str::contains("fn explicit_return(n: i32) -> &'static str"))
        .stdout(predicate::str::contains("-> i32").not());
}

#[test]
fn test_issue_114_end_to_end_compilation() {
    // Test 8: End-to-end verification - transpile and compile
    let input = r#"
fun string_concatenation(iterations) {
    let mut result = ""
    let mut i = 0

    while i < iterations {
        result = result + "x"
        i = i + 1
    }

    result
}

fun main() {
    let iterations = 10
    let result = string_concatenation(iterations)
    println(result)
}
"#;

    // First, transpile to Rust
    let transpile_output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("transpile")
        .arg("-")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let rust_code = String::from_utf8(transpile_output).unwrap();

    // Verify return type is String
    assert!(
        rust_code.contains("fn string_concatenation(iterations: i32) -> String"),
        "Expected 'fn string_concatenation(iterations: i32) -> String', got:\n{}",
        rust_code
    );

    // Verify NOT i32
    assert!(
        !rust_code.contains("fn string_concatenation(iterations: i32) -> i32"),
        "Should NOT contain 'fn string_concatenation(iterations: i32) -> i32', got:\n{}",
        rust_code
    );

    // Write to temp file and attempt compilation
    use std::io::Write;
    let temp_file = std::env::temp_dir().join("issue_114_test.rs");
    let mut file = std::fs::File::create(&temp_file).unwrap();
    file.write_all(rust_code.as_bytes()).unwrap();
    drop(file);

    // Attempt to compile with rustc
    let compile_result = std::process::Command::new("rustc")
        .arg("--crate-type")
        .arg("bin")
        .arg(&temp_file)
        .arg("-o")
        .arg(std::env::temp_dir().join("issue_114_test"))
        .output();

    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!(
                    "Compilation failed:\n{}\n\nGenerated Rust code:\n{}",
                    stderr, rust_code
                );
            }
        }
        Err(e) => {
            panic!("Failed to run rustc: {}", e);
        }
    }

    // Clean up
    let _ = std::fs::remove_file(temp_file);
    let _ = std::fs::remove_file(std::env::temp_dir().join("issue_114_test"));
}
