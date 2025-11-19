#![allow(missing_docs)]
// REGRESSION-077: Logger/Common/Schema runtime hang in v3.147.1
// GitHub Issue: https://github.com/paiml/ruchy/issues/77
//
// ROOT CAUSE: v3.147.1 fix for Issue #76 only partially fixed Vec::new() regression.
// Logger, Common, and Schema files (3/6 conversions) still hang at runtime.
//
// PATTERN: Struct methods with String parameters in impl blocks hang when called
// EXPECTED: Logger::new_with_options() should work (did in v3.146.0)
// ACTUAL (v3.147.1): Infinite hang when calling methods with String parameters

use assert_cmd::Command;
use predicates::prelude::*;

/// Test Case 1: Minimal `Logger::new_with_options()` hang
/// This is the smallest reproduction case from Issue #77 Bug #1
#[test]
fn test_regression_077_logger_new_with_options() {
    let script = r#"
struct Logger {
    prefix: String,
    use_colors: bool,
    min_level: i32,
}

impl Logger {
    fun new() -> Logger {
        Logger {
            prefix: String::new(),
            use_colors: true,
            min_level: 0,
        }
    }

    fun new_with_options(prefix: String, use_colors: bool, min_level: i32) -> Logger {
        Logger {
            prefix: prefix,
            use_colors: use_colors,
            min_level: min_level,
        }
    }
}

fun main() {
    let logger1 = Logger::new();
    let logger2 = Logger::new_with_options(
        String::from("test"),
        true,
        1
    );
    println!("Success");
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 2: `Logger.log()` method with String parameter
/// Verifies methods with String parameters work after fix
#[test]
fn test_regression_077_logger_log_method() {
    let script = r#"
struct Logger {
    prefix: String,
    use_colors: bool,
    min_level: i32,
}

impl Logger {
    fun new() -> Logger {
        Logger {
            prefix: String::new(),
            use_colors: true,
            min_level: 0,
        }
    }

    fun log(self, message: String) {
        println!("{}", message);
    }
}

fun main() {
    let logger = Logger::new();
    logger.log(String::from("Test message"));
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Test message"));
}

/// Test Case 3: Common.rs pattern - Option<String> parameters
/// Verifies Option<String> parameters work after fix
#[test]
fn test_regression_077_common_optional_string() {
    let script = r#"
struct Common {
    name: String,
    desc: Option<String>,
}

impl Common {
    fun new(name: String, desc: Option<String>) -> Common {
        Common {
            name: name,
            desc: desc,
        }
    }
}

fun main() {
    let common1 = Common::new(String::from("test"), Option::None);
    let common2 = Common::new(
        String::from("test2"),
        Option::Some(String::from("description"))
    );
    println!("Success");
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(script)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Success"));
}

/// Test Case 4: Schema.rs pattern - Multiple String parameters
/// Verifies multiple String parameters work after fix
#[test]
fn test_regression_077_schema_multiple_strings() {
    let script = r#"
struct Schema {
    table_name: String,
    column_name: String,
    column_type: String,
}

impl Schema {
    fun new(table_name: String, column_name: String, column_type: String) -> Schema {
        Schema {
            table_name: table_name,
            column_name: column_name,
            column_type: column_type,
        }
    }
}

fun main() {
    let schema = Schema::new(
        String::from("users"),
        String::from("id"),
        String::from("INTEGER")
    );
    println!("Success");
}
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
/// This verifies the v3.147.1 fix isn't broken by v3.147.2
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
