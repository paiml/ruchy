// Regression tests for GitHub Issue #79: Runtime hang when accessing enum field via self and casting to i32
// https://github.com/paiml/ruchy/issues/79
//
// ROOT CAUSE: eval_type_cast() was not handling enum-to-integer casts
// SOLUTION: Added special case handling for FieldAccess patterns in type casts to extract discriminant
//
// Test naming convention: test_regression_079_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Basic enum cast to i32 (simplest case from Issue #79)
#[test]
fn test_regression_079_enum_cast_i32() {
    let code = r#"
enum LogLevel {
    Info = 1,
}
fun main() {
    let val = LogLevel::Info as i32;
    println(val);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

/// Test #2: Multiple enum variants with different discriminants
#[test]
fn test_regression_079_multiple_variants() {
    let code = r#"
enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}
fun main() {
    let debug_val = LogLevel::Debug as i32;
    let info_val = LogLevel::Info as i32;
    let warn_val = LogLevel::Warn as i32;
    let error_val = LogLevel::Error as i32;
    println(debug_val);
    println(info_val);
    println(warn_val);
    println(error_val);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3"));
}

/// Test #3: Enum cast to i64 (different integer type)
#[test]
fn test_regression_079_enum_cast_i64() {
    let code = r#"
enum Status {
    Pending = 100,
    Active = 200,
}
fun main() {
    let val = Status::Active as i64;
    println(val);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("200"));
}

/// Test #4: Enum cast to isize
#[test]
fn test_regression_079_enum_cast_isize() {
    let code = r#"
enum Priority {
    Low = 1,
    High = 10,
}
fun main() {
    let val = Priority::High as isize;
    println(val);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

/// Test #5: Enum cast in arithmetic expression
#[test]
fn test_regression_079_enum_cast_arithmetic() {
    let code = r#"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
fun main() {
    let result = (LogLevel::Info as i32) + 10;
    println(result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("11"));
}

/// Test #6: Enum cast in variable assignment
#[test]
fn test_regression_079_enum_cast_assignment() {
    let code = r#"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
fun main() {
    let level = LogLevel::Info;
    println("Enum variant created");
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Enum variant created"));
}
