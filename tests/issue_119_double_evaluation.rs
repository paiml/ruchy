// ISSUE-119: Double-evaluation bug in println() builtin function
// ROOT CAUSE: Function arguments with side-effects evaluated twice
//
// EXTREME TDD: RED → GREEN → REFACTOR → VALIDATE

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_issue_119_println_side_effects_evaluated_once() {
    // RED: This test MUST fail - println() evaluates arguments TWICE

    let script = r"
let mut counter = 0

fun increment() {
    counter = counter + 1
    counter
}

// Each println(increment()) should call increment() EXACTLY ONCE
println(increment())  // Expect: 1, Actual: 2 ❌
println(increment())  // Expect: 2, Actual: 4 ❌
println(increment())  // Expect: 3, Actual: 6 ❌
println(counter)      // Expect: 3, Actual: 6 ❌
";

    std::fs::write("/tmp/issue_119_double_eval.ruchy", script).unwrap();

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(script)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Expected output if side-effects evaluated ONCE
    let expected_lines = ["1", "2", "3", "3"];

    // Actual buggy output (side-effects evaluated TWICE)
    let actual_lines: Vec<&str> = stdout.trim().lines().collect();

    assert!(
        actual_lines != ["2", "4", "6", "6"],
        "BUG DETECTED: Side-effects evaluated TWICE!\n\
             Expected: {expected_lines:?} (each increment() called once)\n\
             Actual: {actual_lines:?} (each increment() called twice)\n\
             \n\
             Full output:\n{stdout}\n"
    );

    // Test passes when output is correct
    assert_eq!(
        actual_lines, expected_lines,
        "increment() should be called exactly once per println()"
    );
}

#[test]
fn test_issue_119_variable_assignment_no_double_eval() {
    // Baseline: Variable assignment should NOT double-evaluate

    let script = r"
let mut counter = 0

fun increment() {
    counter = counter + 1
    counter
}

let result = increment()  // Should call increment() once
println(result)
println(counter)
";

    let output = Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(script)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Variable assignment should work correctly (no double-eval)
    assert!(
        stdout.contains("1\n1"),
        "Variable assignment should not double-evaluate, got: {stdout}"
    );
}
