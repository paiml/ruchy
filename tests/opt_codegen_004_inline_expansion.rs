// OPT-CODEGEN-004: Inline Expansion Optimization (Risk Class 1)
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE (8 phases)
// GitHub Issue: #126
// Spec: ../ruchyruchy/docs/specifications/compiler-transpiler-optimization-spec.md (line 372)
// Target: 10-25% runtime speedup
// Dependencies: PERF-002-A (constant folding), PERF-002-B (constant propagation) complete

use assert_cmd::Command;
use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until inline expansion is implemented
/// Acceptance: Small functions are inlined at call sites

// ============================================================================
// TEST GROUP 1: Simple Function Inlining
// ============================================================================

#[test]
fn test_opt_codegen_004_inline_simple_function() {
    // Pattern: Small helper function should be inlined
    let code = r#"
        fun add_one(x: i32) -> i32 {
            x + 1
        }

        fun main() -> i32 {
            let result = add_one(5);
            result
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should inline: let result = 5 + 1
        .stdout(predicate::str::contains("add_one").not());
}

#[test]
fn test_opt_codegen_004_inline_multi_use() {
    // Pattern: Function called multiple times should be inlined at each call site
    let code = r#"
        fun double(n: i32) -> i32 {
            n * 2
        }

        fun main() -> i32 {
            let a = double(3);
            let b = double(5);
            let c = double(7);
            a + b + c
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // All 3 calls should be inlined
        .stdout(predicate::str::contains("double").not());
}

#[test]
#[ignore] // TODO(OPT-CODEGEN-004-B): Requires recursive inlining + constant folding
fn test_opt_codegen_004_inline_with_constants() {
    // Pattern: Inlining + constant folding should work together
    let code = r#"
        fun square(x: i32) -> i32 {
            x * x
        }

        fun main() -> i32 {
            square(4)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should inline AND fold: 4 * 4 → 16
        .stdout(predicate::str::contains("square").not())
        .stdout(predicate::str::contains("16"));
}

// ============================================================================
// TEST GROUP 2: Size Threshold Heuristics
// ============================================================================

#[test]
fn test_opt_codegen_004_no_inline_large_function() {
    // Pattern: Large functions (>10 LOC) should NOT be inlined
    let code = r#"
        fun large_computation(n: i32) -> i32 {
            let a = n + 1;
            let b = a * 2;
            let c = b - 3;
            let d = c / 2;
            let e = d + 5;
            let f = e * 3;
            let g = f - 7;
            let h = g / 2;
            let i = h + 9;
            let j = i * 4;
            j
        }

        fun main() -> i32 {
            large_computation(10)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should NOT inline - function is too large
        .stdout(predicate::str::contains("large_computation"));
}

#[test]
fn test_opt_codegen_004_inline_small_threshold() {
    // Pattern: Functions at threshold (≤10 LOC) SHOULD be inlined
    let code = r#"
        fun at_threshold(x: i32) -> i32 {
            let a = x + 1;
            let b = a * 2;
            let c = b - 3;
            c
        }

        fun main() -> i32 {
            at_threshold(5)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should inline - within threshold
        .stdout(predicate::str::contains("at_threshold").not());
}

// ============================================================================
// TEST GROUP 3: Recursive Functions (Safety Check)
// ============================================================================

#[test]
fn test_opt_codegen_004_no_inline_recursive() {
    // Pattern: Recursive functions should NEVER be inlined (correctness risk)
    let code = r#"
        fun factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }

        fun main() -> i32 {
            factorial(5)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should NOT inline recursive function
        .stdout(predicate::str::contains("factorial"));
}

#[test]
fn test_opt_codegen_004_no_inline_mutually_recursive() {
    // Pattern: Mutually recursive functions should NOT be inlined
    let code = r#"
        fun is_even(n: i32) -> bool {
            if n == 0 {
                true
            } else {
                is_odd(n - 1)
            }
        }

        fun is_odd(n: i32) -> bool {
            if n == 0 {
                false
            } else {
                is_even(n - 1)
            }
        }

        fun main() -> bool {
            is_even(4)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Both functions should remain (not inlined)
        .stdout(predicate::str::contains("is_even"))
        .stdout(predicate::str::contains("is_odd"));
}

// ============================================================================
// TEST GROUP 4: Nested Inlining (Chain)
// ============================================================================

#[test]
#[ignore] // TODO(OPT-CODEGEN-004-B): Implement recursive/nested inlining (see issue)
fn test_opt_codegen_004_inline_chain() {
    // Pattern: A calls B, B calls C - all should inline
    let code = r#"
        fun add_two(x: i32) -> i32 {
            x + 2
        }

        fun add_four(x: i32) -> i32 {
            add_two(add_two(x))
        }

        fun main() -> i32 {
            add_four(10)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // All functions should be inlined
        .stdout(predicate::str::contains("add_two").not())
        .stdout(predicate::str::contains("add_four").not());
}

// ============================================================================
// TEST GROUP 5: Integration with Other Optimizations
// ============================================================================

#[test]
fn test_opt_codegen_004_inline_after_dce() {
    // Pattern: Inline expansion works after dead code elimination
    let code = r#"
        fun helper(x: i32) -> i32 {
            return x * 2;
            let dead = 99;  // DCE should remove this
            dead
        }

        fun main() -> i32 {
            helper(5)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should inline AND eliminate dead code
        .stdout(predicate::str::contains("helper").not())
        .stdout(predicate::str::contains("dead").not());
}

#[test]
#[ignore] // TODO(OPT-CODEGEN-004-B): Requires recursive inlining + constant propagation
fn test_opt_codegen_004_inline_with_propagation() {
    // Pattern: Inline + constant propagation + folding all work together
    let code = r#"
        fun compute(a: i32, b: i32) -> i32 {
            a + b
        }

        fun main() -> i32 {
            let x = 10;
            let y = 20;
            compute(x, y)
        }
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should inline AND propagate AND fold: 10 + 20 → 30
        .stdout(predicate::str::contains("compute").not())
        .stdout(predicate::str::contains("30"));
}

// ============================================================================
// Property Test Placeholder (VALIDATE phase)
// ============================================================================

// Note: Property tests will be added in VALIDATE phase
// - prop_inline_preserves_semantics: verify original == optimized behavior (25,000+ cases per spec)
// - prop_no_inline_recursive: verify recursive functions never inlined
// - prop_inline_size_threshold: verify large functions not inlined

// Note: Fuzz tests will be added in VALIDATE phase (250,000+ cases per spec)
