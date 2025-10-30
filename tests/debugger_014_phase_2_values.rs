//! Tests for DEBUGGER-014 Phase 2: Enhanced tracing with argument and return values
//! GitHub Issue: <https://github.com/paiml/ruchy/issues/84>
//!
//! Phase 2 enhances Phase 1 basic tracing (function entry/exit) with:
//! - Argument values: TRACE: → fibonacci(3)
//! - Return values: TRACE: ← fibonacci = 2
//!
//! Test naming convention: `test_debugger_014_phase_2`_<feature>_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Trace function call with single argument value
///
/// This tests that --trace outputs argument values when calling functions
#[test]
fn test_debugger_014_phase_2_trace_single_argument() {
    let code = r#"
fun square(x) {
    return x * x;
}

fun main() {
    let result = square(5);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → square(5: integer)"))
        .stdout(predicate::str::contains("TRACE: ← square = 25: integer"));
}

/// Test #2: Trace function call with multiple arguments
///
/// This tests that --trace handles multiple arguments correctly
#[test]
fn test_debugger_014_phase_2_trace_multiple_arguments() {
    let code = r#"
fun add(a, b) {
    return a + b;
}

fun main() {
    let result = add(10, 20);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → add(10: integer, 20: integer)"))
        .stdout(predicate::str::contains("TRACE: ← add = 30: integer"));
}

/// Test #3: Trace recursive function with argument values
///
/// This tests that --trace shows argument values in recursive calls
#[test]
fn test_debugger_014_phase_2_trace_recursive_arguments() {
    let code = r#"
fun factorial(n) {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}

fun main() {
    let result = factorial(3);
    println("Result: {}", result);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → factorial(3: integer)"))
        .stdout(predicate::str::contains("TRACE: → factorial(2: integer)"))
        .stdout(predicate::str::contains("TRACE: → factorial(1: integer)"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 1: integer"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 2: integer"))
        .stdout(predicate::str::contains("TRACE: ← factorial = 6: integer"));
}

/// Test #4: Trace function with string arguments
///
/// This tests that --trace handles string values correctly
#[test]
fn test_debugger_014_phase_2_trace_string_arguments() {
    let code = r#"
fun greet(name) {
    return "Hello, " + name;
}

fun main() {
    let msg = greet("Alice");
    println("{}", msg);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → greet(\"Alice\": string)"))
        .stdout(predicate::str::contains("TRACE: ← greet = \"Hello, Alice\": string"));
}

/// Test #5: Trace function with no arguments
///
/// This tests that --trace handles zero-argument functions correctly
#[test]
fn test_debugger_014_phase_2_trace_no_arguments() {
    let code = r#"
fun get_answer() {
    return 42;
}

fun main() {
    let answer = get_answer();
    println("Answer: {}", answer);
}
"#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → get_answer()"))
        .stdout(predicate::str::contains("TRACE: ← get_answer = 42"));
}

/// Test #6: Backward compatibility - Phase 1 tests still pass
///
/// This ensures Phase 2 doesn't break existing Phase 1 trace output
#[test]
fn test_debugger_014_phase_2_backward_compatible() {
    let code = r"
fun test_func() {
    return 42;
}

fun main() {
    test_func();
}
";

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        // Phase 1 format still works (contains function name)
        .stdout(predicate::str::contains("TRACE: → test_func"))
        .stdout(predicate::str::contains("TRACE: ← test_func"));
}
