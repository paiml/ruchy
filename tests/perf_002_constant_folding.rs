// PERF-002-A: Constant Folding Optimization
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// Target: 10-20% speedup on BENCH-003, BENCH-007

use assert_cmd::Command;
use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until constant folder is implemented
/// Acceptance: Constant expressions evaluated at compile-time, not runtime

// ============================================================================
// TEST GROUP 1: Basic Arithmetic Constant Folding (Core Functionality)
// ============================================================================

#[test]
fn test_perf_002a_fold_simple_arithmetic() {
    // Pattern: 2 + 3 → 5 (compile-time)
    let code = r#"
        let x = 2 + 3;
        println(x);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 5")); // Folded!
}

#[test]
fn test_perf_002a_fold_operator_precedence() {
    // Pattern: 2 + 3 * 4 → 2 + 12 → 14 (respects precedence)
    let code = r#"
        let result = 2 + 3 * 4;
        println(result);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let result = 14"));
}

#[test]
fn test_perf_002a_fold_nested_expressions() {
    // Pattern: (10 - 2) * (3 + 1) → 8 * 4 → 32
    let code = r#"
        let x = (10 - 2) * (3 + 1);
        println(x);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = 32"));
}

// ============================================================================
// TEST GROUP 2: Boolean Constant Folding (Comparison Ops)
// ============================================================================

#[test]
fn test_perf_002a_fold_comparison_true() {
    // Pattern: 10 > 5 → true (compile-time)
    let code = r#"
        let is_greater = 10 > 5;
        if is_greater {
            println("yes");
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let is_greater = true"));
}

#[test]
fn test_perf_002a_fold_comparison_false() {
    // Pattern: 3 <= 2 → false (compile-time)
    let code = r#"
        let x = 3 <= 2;
        println(x);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let x = false"));
}

// ============================================================================
// TEST GROUP 3: Dead Branch Elimination (Control Flow)
// ============================================================================

#[test]
#[ignore] // TODO: Dead branch elimination not yet implemented (PERF-002-B)
fn test_perf_002a_eliminate_dead_if_branch() {
    // Pattern: if false { ... } → eliminated entirely
    let code = r#"
        if false {
            expensive_computation();
        }
        println("done");
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")  // Read from stdin
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("expensive_computation").not()); // Dead code removed!
}

// ============================================================================
// PROPERTY TEST 1: Constant folding preserves semantics
// ============================================================================

#[test]
#[ignore] // Run with: cargo test --test perf_002_constant_folding -- --ignored
fn property_constant_folding_preserves_semantics() {
    use proptest::prelude::*;

    proptest!(|(a in 0..100i32, b in 1..100i32)| {
        let code = format!(r#"
            let x = {} + {};
            println(x);
        "#, a, b);

        // Evaluate original expression
        let expected = a + b;

        // Run transpiled code and verify folded constant equals original result
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.arg("transpile")
            .arg("-")  // Read from stdin
            .write_stdin(code.clone())
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("let x = {}", expected)));
    });
}
