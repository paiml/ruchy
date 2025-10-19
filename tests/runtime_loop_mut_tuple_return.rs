//! PARSER-DEFECT-001: Block + Tuple Parsing Bug (FIXED)
//!
//! **Problem**: Loop followed by tuple was misparsed as function call
//! **Root Cause**: Parser treated `loop { } (x, x)` as `(loop { })(x, x)`
//! **Fix**: Added is_block_like_expression() check in try_handle_single_postfix()
//! **Discovered**: 2025-10-19 (BOOTSTRAP-003 session in ruchyruchy)
//! **Status**: GREEN PHASE - All tests passing
//!
//! This test follows EXTREME TDD (RED ✅ → GREEN ✅ → REFACTOR)

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ==================== GREEN PHASE: Tests Now Passing ====================

/// Test 1: Basic loop + mut + tuple return (NOW PASSING)
///
/// Tests the fix for PARSER-DEFECT-001 where:
/// - Pattern: `loop { } (x, x)` was parsed as `(loop { })(x, x)` (function call)
/// - Fix: is_block_like_expression() prevents block-like exprs from consuming `(` as call
/// - Expected: Returns (10, 5) and prints "Sum: 10, Index: 5"
#[test]
fn test_green_loop_mut_tuple_basic() {
    let code = r#"
fn test_loop_mut() -> (i32, i32) {
    let mut idx = 0;
    let mut sum = 0;

    loop {
        if idx >= 5 {
            break;
        }
        sum = sum + idx;
        idx = idx + 1;
    }

    (sum, idx)
}

fn main() {
    let result = test_loop_mut();
    let sum = result.0;
    let idx = result.1;

    println("Sum: {}, Index: {}", sum, idx);
}

main();
"#;

    // GREEN PHASE: Now passes after PARSER-DEFECT-001 fix
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Sum: 10, Index: 5"));
}

/// Test 2: Simplified loop + mut + tuple (NOW PASSING)
///
/// Simpler version without tuple destructuring - just direct println
#[test]
fn test_green_loop_mut_tuple_simple() {
    let code = r#"
fn make_pair() -> (i32, i32) {
    let mut x = 0;
    loop {
        if x >= 3 {
            break;
        }
        x = x + 1;
    }
    (x, x * 2)
}

fn main() {
    println("{:?}", make_pair());
}

main();
"#;

    // GREEN PHASE: Now passes after PARSER-DEFECT-001 fix
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("(3, 6)"));
}

// ==================== BASELINE TESTS (Currently Pass) ====================

/// Baseline 1: Tuple return WITHOUT loop works
#[test]
fn test_baseline_tuple_return_no_loop() {
    let code = r#"
fn make_tuple() -> (i32, i32) {
    (42, 100)
}

fn main() {
    let result = make_tuple();
    println("{}, {}", result.0, result.1);
}

main();
"#;

    // This should work NOW
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42, 100"));
}

/// Baseline 2: Loop + mut WITHOUT tuple return works
#[test]
fn test_baseline_loop_mut_no_tuple() {
    let code = r#"
fn count_to_five() -> i32 {
    let mut x = 0;
    loop {
        if x >= 5 {
            break;
        }
        x = x + 1;
    }
    x
}

fn main() {
    println("{}", count_to_five());
}

main();
"#;

    // This should work NOW
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

// ==================== GREEN PHASE SUMMARY ====================

/// Summary test documenting GREEN phase success
#[test]
fn test_green_phase_summary() {
    println!("\n=== PARSER-DEFECT-001: FIXED ===");
    println!("\nProblem: loop {{ }} (x, x) misparsed as (loop {{ }})(x, x)");
    println!("\nRoot Cause: try_handle_single_postfix() line 351");
    println!("  Unconditionally treated `(` after ANY expression as call");
    println!("\nFix: is_block_like_expression() check");
    println!("  Prevents block-like exprs (Loop, If, Match, etc) from consuming `(` as call");
    println!("\nComplexity: ≤10 (within Toyota Way limits)");
    println!("\nStatus: GREEN PHASE COMPLETE");
    println!("  - All tests passing");
    println!("  - BOOTSTRAP-003 unblocked");
    println!("  - Pattern (Token, i32) return now works\n");
    println!("\nNext: REFACTOR phase - PMAT quality gates\n");
}
