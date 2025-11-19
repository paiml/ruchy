#![allow(missing_docs)]
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
    let code = r"
enum LogLevel {
    Info = 1,
}
fun main() {
    let val = LogLevel::Info as i32;
    println(val);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
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
    let code = r"
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
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
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
    let code = r"
enum Status {
    Pending = 100,
    Active = 200,
}
fun main() {
    let val = Status::Active as i64;
    println(val);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
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
    let code = r"
enum Priority {
    Low = 1,
    High = 10,
}
fun main() {
    let val = Priority::High as isize;
    println(val);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
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
    let code = r"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
fun main() {
    let result = (LogLevel::Info as i32) + 10;
    println(result);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("11"));
}

/// Test #6: Enum variable cast (v3.147.4 - RUNTIME-092)
/// This test verifies that enum values stored in variables can be cast to integers.
/// Previous versions (v3.147.3) only supported direct enum literal casts.
#[test]
fn test_regression_079_enum_variable_cast() {
    let code = r"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
fun main() {
    let level = LogLevel::Debug;
    let val = level as i32;
    println(val);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

/// Test #7: Enum struct field cast (original Issue #79 case)
/// This test reproduces the original bug report: accessing enum field via self and casting to i32.
/// This was the exact scenario reported in GitHub Issue #79.
///
/// CURRENTLY IGNORED: Blocked by separate runtime bug - custom struct method dispatch fails.
/// Error: "Method 'test' not found for type struct"
/// This is NOT an enum cast bug - it's a struct method lookup issue.
/// Once the method dispatch bug is fixed, this test should pass.
#[test]
fn test_regression_079_enum_field_cast() {
    let code = r"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
struct Logger {
    level: LogLevel,
}
impl Logger {
    fun test(&self) {
        let val = self.level as i32;
        println(val);
    }
}
fun main() {
    let logger = Logger { level: LogLevel::Info };
    logger.test();
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

/// Test #8: Multiple variable casts (v3.147.4 - RUNTIME-092)
/// This test verifies that multiple enum variables can be cast without interference.
/// Tests both Debug (0) and Info (1) discriminant values.
#[test]
fn test_regression_079_multiple_variable_casts() {
    let code = r"
enum LogLevel {
    Debug = 0,
    Info = 1,
}
fun main() {
    let debug = LogLevel::Debug;
    let info = LogLevel::Info;
    let debug_val = debug as i32;
    let info_val = info as i32;
    println(debug_val);
    println(info_val);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("0"))
        .stdout(predicate::str::contains("1"));
}
