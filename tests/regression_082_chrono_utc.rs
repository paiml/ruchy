#![allow(missing_docs)]
// Regression tests for GitHub Issue #82: Runtime error when using `use chrono::Utc;`
// https://github.com/paiml/ruchy/issues/82
//
// REGRESSION INFO:
// - Working Version: v3.147.6 ✅
// - Broken Versions: v3.147.7, v3.147.8 ❌
// - Error: "Runtime error: Undefined variable: Utc"
// - Type: Standard library regression
//
// ROOT CAUSE: chrono::Utc was never implemented (NOT a regression - missing feature)
//   - No chrono namespace in global environment
//   - No Utc module with now() method
//   - ImportAll didn't navigate nested objects for module paths
//   - println! macro didn't support format strings with {:?}
//   - String values lacked .timestamp() method for datetime conversion
//   - String values lacked .to_rfc3339() method (added in v3.147.10)
//
// SOLUTION: Implemented chrono support with EXTREME TDD
//   - v3.147.9: Core chrono::Utc implementation (3/4 tests)
//     - Added add_chrono_namespace() to builtin_init.rs (line 466)
//     - Implemented eval_chrono_utc_now() in eval_builtin.rs (line 841)
//     - Enhanced ImportAll in interpreter.rs to navigate module paths (line 1150)
//     - Added .timestamp() method for RFC3339 strings in eval_string_methods.rs (line 414)
//     - Updated println! to support {:?} debug formatting in interpreter.rs (lines 1216, 1358)
//   - v3.147.10: Complete chrono::Utc support (4/4 tests) - 100% FUNCTIONALITY ✅
//     - Added .to_rfc3339() method in eval_string_methods.rs (line 50)
//   - All 4 regression tests now pass ✅
//
// Test naming convention: test_regression_082_<scenario>

use predicates::prelude::*;

/// Test #1: Basic `chrono::Utc` import (minimal reproduction from Issue #82)
/// This is the exact test case reported in the GitHub issue.
#[test]
fn test_regression_082_chrono_utc_basic_import() {
    let code = r"
use chrono::Utc;

fun main() {
    let now = Utc::now();
    println(now);
}
";

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("202")); // Check for year 202x
}

/// Test #2: `Chrono::Utc` with timestamp formatting
/// Verifies that Utc type works in more complex scenarios
#[test]
fn test_regression_082_chrono_utc_with_formatting() {
    let code = r#"
use chrono::Utc;

fun main() {
    let now = Utc::now();
    let timestamp = now.timestamp();
    println!("Timestamp: {}", timestamp);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Timestamp:"));
}

/// Test #3: Multiple chrono types imported together
/// Verifies that Utc works alongside other chrono types
#[test]
fn test_regression_082_multiple_chrono_imports() {
    let code = r#"
use chrono::Utc;
use chrono::DateTime;

fun main() {
    let now: DateTime<Utc> = Utc::now();
    println!("DateTime: {:?}", now);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("DateTime:"));
}

/// Test #4: .`to_rfc3339()` method on datetime strings
/// Verifies that RFC3339 datetime strings have .`to_rfc3339()` method
#[test]
fn test_regression_082_to_rfc3339_method() {
    let code = r#"
use chrono::Utc;

fun main() {
    let now = Utc::now();
    let rfc3339 = now.to_rfc3339();
    println!("RFC3339: {}", rfc3339);
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("RFC3339:"))
        .stdout(predicate::str::contains("202")); // Check for year 202x
}
