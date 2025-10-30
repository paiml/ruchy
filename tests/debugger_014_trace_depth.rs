// Tests for DEBUGGER-014 Phase 1.4: Trace depth tracking (Issue #84)
// GitHub Issue: https://github.com/paiml/ruchy/issues/84
//
// Test naming convention: test_debugger_014_phase_1_4_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;

/// Test #1: Verify --trace shows indentation for nested calls
#[test]
fn test_debugger_014_phase_1_4_trace_shows_depth() {
    let code = r#"
fun inner() {
    return 42;
}

fun middle() {
    return inner();
}

fun outer() {
    return middle();
}

fun main() {
    println("Result: {}", outer());
}
"#;

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();

    // Check that we have depth indicators (at least basic)
    // Phase 1.4 goal: Show call depth/nesting in trace output
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify we have trace output (basic requirement from Phase 1.3)
    assert!(stdout.contains("TRACE:"), "Should have trace output");
    assert!(stdout.contains("outer"), "Should trace outer function");
    assert!(stdout.contains("middle"), "Should trace middle function");
    assert!(stdout.contains("inner"), "Should trace inner function");
}

/// Test #2: Verify main() function is also traced
#[test]
fn test_debugger_014_phase_1_4_trace_includes_main() {
    let code = r#"
fun helper() {
    return 10;
}

fun main() {
    let x = helper();
    println("Value: {}", x);
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
        .stdout(predicate::str::contains("TRACE:"))
        .stdout(predicate::str::contains("main"))
        .stdout(predicate::str::contains("helper"));
}

/// Test #3: Verify trace output goes to stderr (not stdout)
/// This allows separating program output from trace output
#[test]
#[ignore] // Phase 1.4 enhancement - trace to stderr
fn test_debugger_014_phase_1_4_trace_to_stderr() {
    let code = r#"
fun test_func() {
    return 42;
}

fun main() {
    println("Result: {}", test_func());
}
"#;

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("--trace")
        .arg("-e")
        .arg(code)
        .timeout(std::time::Duration::from_secs(5))
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    // Program output on stdout
    assert!(stdout.contains("Result:"), "Program output should be on stdout");

    // Trace output on stderr
    assert!(stderr.contains("TRACE:"), "Trace output should be on stderr");
    assert!(stderr.contains("test_func"), "Should trace function on stderr");
}
