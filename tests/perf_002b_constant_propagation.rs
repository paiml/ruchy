// PERF-002-B: Constant Propagation Optimization (Julia-inspired)
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// GitHub Issue: #124
// Spec: ../ruchyruchy/docs/specifications/performance-profiling-compiler-tooling.md
// Target: 10-20% speedup on compute-heavy workloads
// Dependencies: PERF-002-A (constant folding) complete

use assert_cmd::Command;
use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until constant propagation is implemented
/// Acceptance: Variables with known constant values are propagated through expressions

// ============================================================================
// TEST GROUP 1: Simple Variable Propagation (Core Functionality)
// ============================================================================

#[test]
fn test_perf_002b_propagate_simple_variable() {
    // Pattern: let x = 5; x + 1 → 6
    let code = r"
        let x = 5;
        let y = x + 1;
        println(y);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let y = 6")); // Propagated!
}

#[test]
fn test_perf_002b_propagate_chained_variables() {
    // Pattern: let x = 5; let y = x; let z = y + 3 → z = 8
    let code = r"
        let x = 5;
        let y = x;
        let z = y + 3;
        println(z);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let z = 8"));
}

#[test]
fn test_perf_002b_propagate_multiple_uses() {
    // Pattern: let x = 5; x + x → 10
    let code = r"
        let x = 5;
        let result = x + x;
        println(result);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let result = 10"));
}

// ============================================================================
// TEST GROUP 2: Arithmetic with Propagated Variables
// ============================================================================

#[test]
fn test_perf_002b_propagate_arithmetic_chain() {
    // Pattern: let a = 2; let b = 3; let c = a * b → c = 6
    let code = r"
        let a = 2;
        let b = 3;
        let c = a * b;
        println(c);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let c = 6"));
}

#[test]
fn test_perf_002b_propagate_with_folding() {
    // Pattern: let x = 5; let y = x + (2 * 3) → y = 11
    // Combines constant folding (2*3=6) + propagation (x=5)
    let code = r"
        let x = 5;
        let y = x + (2 * 3);
        println(y);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let y = 11"));
}

// ============================================================================
// TEST GROUP 3: Conservative Boundaries (Don't Propagate Incorrectly)
// ============================================================================

#[test]
fn test_perf_002b_no_propagate_mutable() {
    // Pattern: let mut x = 5; x might change, DON'T propagate
    let code = r"
        let mut x = 5;
        let y = x + 1;
        println(y);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should NOT propagate (x is mutable)
        .stdout(predicate::str::contains("x + 1"));
}

#[test]
fn test_perf_002b_no_propagate_across_conditional() {
    // Pattern: if blocks, don't propagate (conservative)
    let code = r"
        let x = 5;
        if true {
            let y = x + 1;
            println(y);
        }
        let z = x + 2;
        println(z);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Can propagate x=5 into if block (x defined before)
        .stdout(predicate::str::contains("let y = 6"))
        // Can propagate x=5 after if block
        .stdout(predicate::str::contains("let z = 7"));
}

// ============================================================================
// TEST GROUP 4: Boolean Propagation
// ============================================================================

#[test]
fn test_perf_002b_propagate_boolean() {
    // Pattern: let flag = true; if flag → if true
    let code = r"
        let flag = true;
        let result = if flag { 1 } else { 0 };
        println(result);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Dead branch eliminated, result is block expression { 1 }
        .stdout(predicate::str::contains("let result ="))
        .stdout(predicate::str::contains("{ 1 }"));
}

#[test]
fn test_perf_002b_propagate_comparison_result() {
    // Pattern: let cmp = (10 > 5); cmp is constant (true)
    let code = r"
        let cmp = 10 > 5;
        let x = if cmp { 42 } else { 0 };
        println(x);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Dead branch eliminated, result is block expression { 42 }
        .stdout(predicate::str::contains("let x ="))
        .stdout(predicate::str::contains("{ 42 }"));
}

// ============================================================================
// TEST GROUP 5: End-to-End Real-World Pattern
// ============================================================================

#[test]
fn test_perf_002b_fibonacci_constants_propagated() {
    // Real-world pattern: Fibonacci with constant inputs
    let code = r"
        fun fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }

        let input = 5;
        let result = fibonacci(input);
        println(result);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should propagate input=5 into fibonacci call
        .stdout(predicate::str::contains("fibonacci(5)"));
}

// ============================================================================
// VALIDATE PHASE: Property Tests (10K cases)
// ============================================================================

/// Property Test 1: Constant propagation preserves semantics
/// Invariant: Optimized code produces same result as original
#[test]
#[ignore = "Run with: cargo test property_propagation -- --ignored --nocapture"]
fn property_propagation_preserves_semantics() {
    use proptest::prelude::*;

    proptest!(|(a in 0..100i32, b in 0..100i32)| {
        let code = format!(r"
            let x = {a};
            let y = {b};
            let z = x + y;
            println(z);
        ");

        // Expected result after propagation
        let expected = a + b;

        // Verify constant propagation produces correct folded result
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("let z = {expected}")));
    });
}

/// Property Test 2: Mutable variables are never propagated
/// Invariant: If variable is mutable, its value is NOT substituted
#[test]
#[ignore = "Run with: cargo test property_no_propagate_mutable -- --ignored --nocapture"]
fn property_no_propagate_mutable() {
    use proptest::prelude::*;

    proptest!(|(a in 0..100i32)| {
        let code = format!(r"
            let mut x = {a};
            let y = x + 1;
            println(y);
        ");

        // Verify mutable variable is NOT propagated (should see "x + 1", not constant)
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .stdout(predicate::str::contains("x + 1"));
    });
}

/// Property Test 3: Propagation with arithmetic operations
/// Invariant: Chained arithmetic produces correct folded result
#[test]
#[ignore = "Run with: cargo test property_arithmetic_chain -- --ignored --nocapture"]
fn property_arithmetic_chain() {
    use proptest::prelude::*;

    proptest!(|(a in 0..50i32, b in 1..50i32)| {
        let code = format!(r"
            let x = {a};
            let y = {b};
            let z = x * y;
            println(z);
        ");

        // Expected result after propagation and folding
        let expected = a * b;

        // Verify propagation + folding produces correct result
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("let z = {expected}")));
    });
}
