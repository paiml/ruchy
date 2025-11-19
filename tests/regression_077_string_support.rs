#![allow(missing_docs)]
// REGRESSION-077: String::new() and String::from() support
// GitHub Issue: https://github.com/paiml/ruchy/issues/77
//
// ROOT CAUSE: v3.147.1 didn't have String module registered as builtin
// SOLUTION: Added String to parser whitelist + runtime handlers
//
// SCOPE: This test validates String::new() and String::from() work correctly
// OUT OF SCOPE: impl methods (non-new), Option enum (separate issues)

use assert_cmd::Command;
use predicates::prelude::*;

/// Test Case 1: `String::new()` creates empty string
#[test]
fn test_regression_077_string_new() {
    let script = r#"
let s = String::new();
println!("Success");
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 2: `String::from()` converts value to string
#[test]
fn test_regression_077_string_from() {
    let script = r#"
let s = String::from("test");
println!("Success");
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 3: String in struct field initialization
#[test]
fn test_regression_077_string_in_struct() {
    let script = r#"
struct Logger {
    prefix: String,
}

impl Logger {
    fun new() -> Logger {
        Logger {
            prefix: String::new(),
        }
    }
}

let logger = Logger::new();
println!("Success");
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 4: Multiple String operations
#[test]
fn test_regression_077_multiple_strings() {
    let script = r#"
struct Schema {
    table_name: String,
    column_name: String,
}

impl Schema {
    fun new() -> Schema {
        Schema {
            table_name: String::from("users"),
            column_name: String::from("id"),
        }
    }
}

let schema = Schema::new();
println!("Success");
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 5: `Vec::new()` should still work (no regression from v3.147.1)
#[test]
fn test_regression_077_vec_new_still_works() {
    let script = r#"
let mut vec = Vec::new();
let mut i = 0;
while i < 10 {
    vec.push(1.0);
    i += 1;
}
println!("Success: {} elements", vec.len());
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}
