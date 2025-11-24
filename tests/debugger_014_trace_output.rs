#![allow(missing_docs)]
// Tests for DEBUGGER-014 Phase 1.3: Function tracing output (Issue #84)
// GitHub Issue: https://github.com/paiml/ruchy/issues/84
//
// Test naming convention: test_debugger_014_phase_1_3_<scenario>

use predicates::prelude::*;

/// Test #1: Verify --trace outputs function entry/exit for simple function
#[test]
fn test_debugger_014_phase_1_3_trace_outputs_function_calls() {
    let code = r#"
fun fibonacci(n) {
    if n <= 1 {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

fun main() {
    println("Result: {}", fibonacci(3));
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → fibonacci"))
        .stdout(predicate::str::contains("TRACE: ← fibonacci"));
}

/// Test #2: Verify --trace shows nested function calls
#[test]
fn test_debugger_014_phase_1_3_trace_shows_nesting() {
    let code = r#"
fun inner() {
    return 42;
}

fun outer() {
    return inner();
}

fun main() {
    println("Result: {}", outer());
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE: → outer"))
        .stdout(predicate::str::contains("TRACE: → inner"))
        .stdout(predicate::str::contains("TRACE: ← inner"))
        .stdout(predicate::str::contains("TRACE: ← outer"));
}

/// Test #3: Verify --trace is disabled by default (no trace output)
#[test]
fn test_debugger_014_phase_1_3_trace_disabled_by_default() {
    let code = r#"
fun test_func() {
    return 42;
}

fun main() {
    println("Result: {}", test_func());
}
"#;

    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("TRACE:").not());
}
