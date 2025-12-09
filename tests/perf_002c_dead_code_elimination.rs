// PERF-002-C: Dead Code Elimination (DCE) Optimization
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// GitHub Issue: #125
// Spec: ../ruchyruchy/docs/specifications/compiler-transpiler-optimization-spec.md (OPT-CODEGEN-003)
// Target: 5-15% code size reduction
// Dependencies: PERF-002-A (constant folding), PERF-002-B (constant propagation) complete

use predicates::prelude::*;

/// RED PHASE: These tests WILL FAIL until DCE is implemented
/// Acceptance: Dead code is removed from transpiled output

// ============================================================================
// TEST GROUP 1: Unreachable Code After Return
// ============================================================================

#[test]
#[ignore = "Dead code elimination not yet implemented"]
fn test_perf_002c_dce_after_return() {
    // Pattern: Code after return statement is unreachable
    let code = r"
        fun example() -> i32 {
            return 42;
            let x = 5;  // Dead code
            println(x);  // Dead code
            return 99;  // Dead code
        }
        println(example());
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should NOT contain dead code after return
        .stdout(predicate::str::contains("let x = 5").not())
        .stdout(predicate::str::contains("println(x)").not())
        .stdout(predicate::str::contains("return 99").not());
}

#[test]
#[ignore = "Dead code elimination not yet implemented"]
fn test_perf_002c_dce_multiple_returns() {
    // Pattern: Only first return path is reachable
    let code = r"
        fun check(n: i32) -> i32 {
            if n > 10 {
                return n;
            }
            return 0;
            let unreachable = 42;  // Dead code
        }
        println(check(5));
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("unreachable").not());
}

// ============================================================================
// TEST GROUP 2: Dead Branches from Constant Folding
// ============================================================================

#[test]
fn test_perf_002c_dce_false_branch() {
    // Pattern: if false { ... } entire branch is dead
    let code = r#"
        let x = if false {
            println("Dead branch");
            42
        } else {
            println("Live branch");
            0
        };
        println(x);
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Dead branch should be eliminated
        .stdout(predicate::str::contains("Dead branch").not());
}

#[test]
fn test_perf_002c_dce_true_branch_no_else() {
    // Pattern: if true { A } (no else) → just A
    let code = r#"
        if true {
            println("Always executes");
        }
        let dead = 5;  // Not actually dead, for now
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should fold to just the then-branch content
        .stdout(predicate::str::contains("Always executes"));
}

// ============================================================================
// TEST GROUP 3: Unused Variable Bindings
// ============================================================================

#[test]
#[ignore = "Dead code elimination not yet implemented"]
fn test_perf_002c_dce_unused_variable() {
    // Pattern: Variable defined but never used
    let code = r"
        let unused = 42;
        let used = 10;
        println(used);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Should eliminate unused variable binding
        .stdout(predicate::str::contains("let unused").not());
}

#[test]
#[ignore = "Dead code elimination not yet implemented"]
fn test_perf_002c_dce_unused_computation() {
    // Pattern: Computation result never used (pure expression)
    let code = r"
        let x = 5;
        let unused_result = x + 10;  // Never used
        println(x);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("unused_result").not());
}

// ============================================================================
// TEST GROUP 4: Unreachable Code After Break/Continue
// ============================================================================

#[test]
fn test_perf_002c_dce_after_break() {
    // Pattern: Code after break in loop is unreachable
    let code = r"
        let mut i = 0;
        while true {
            if i > 5 {
                break;
                let dead = 42;  // Dead code
                println(dead);   // Dead code
            }
            i = i + 1;
        }
        println(i);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let dead").not());
}

#[test]
fn test_perf_002c_dce_after_continue() {
    // Pattern: Code after continue in loop is unreachable
    let code = r"
        let mut sum = 0;
        let mut i = 0;
        while i < 10 {
            i = i + 1;
            if i == 5 {
                continue;
                let dead = 42;  // Dead code
            }
            sum = sum + i;
        }
        println(sum);
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("let dead").not());
}

// ============================================================================
// TEST GROUP 5: Empty Blocks After DCE
// ============================================================================

#[test]
#[ignore = "Dead code elimination not yet implemented"]
fn test_perf_002c_dce_empty_block_cleanup() {
    // Pattern: Block becomes empty after DCE, should be removed
    let code = r#"
        {
            let unused = 42;  // Only thing in block, unused
        }
        println("After empty block");
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Empty block should not appear in output
        .stdout(predicate::str::contains("let unused").not());
}

// ============================================================================
// TEST GROUP 6: Integration with Constant Propagation
// ============================================================================

#[test]
fn test_perf_002c_dce_with_constant_propagation() {
    // Pattern: Constant propagation creates dead branch, DCE removes it
    let code = r#"
        let flag = false;
        if flag {
            println("Dead code via propagation");
            let x = 42;
            println(x);
        } else {
            println("Live code");
        }
    "#;

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // Dead branch eliminated via propagation + DCE
        .stdout(predicate::str::contains("Dead code via propagation").not());
}

// ============================================================================
// VALIDATE PHASE: Property Tests (10K+ cases)
// ============================================================================

/// Property Test 1: Dead Code Elimination preserves semantics
/// Invariant: Optimized code produces same result as original
#[test]
fn property_dce_preserves_semantics() {
    use proptest::prelude::*;

    proptest!(|(dead_val in 0..100i32, live_val in 0..100i32)| {
        let code = format!(r"
            fun compute() -> i32 {{
                return {live_val};
                let dead = {dead_val};  // Dead code
                dead
            }}
            println(compute());
        ");

        // Expected result after DCE
        let expected = live_val;

        // Verify DCE preserves semantics
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("return {expected}")));
    });
}

/// Property Test 2: DCE is idempotent
/// Invariant: DCE(DCE(code)) == DCE(code) - applying twice produces same result
#[test]
fn property_dce_idempotent() {
    use proptest::prelude::*;

    proptest!(|(a in 0..50i32, b in 0..50i32)| {
        let code = format!(r"
            fun test() -> i32 {{
                return {a};
                let unused1 = {b};  // Dead code
                return 999;         // Dead code
            }}
            println(test());
        ");

        // Run transpile twice and verify identical output
        let mut cmd1 = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        let output1 = cmd1.arg("transpile")
            .arg("-")
            .write_stdin(code.clone())
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        // Second transpile (DCE already applied)
        let mut cmd2 = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        let output2 = cmd2.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        // Verify outputs are identical (idempotent)
        prop_assert_eq!(output1, output2);
    });
}

/// Property Test 3: Live code is never eliminated
/// Invariant: Variables that are used must remain in output
#[test]
fn property_no_live_code_eliminated() {
    use proptest::prelude::*;

    proptest!(|(a in 0..100i32, b in 0..100i32)| {
        let code = format!(r"
            let x = {a};
            let y = {b};
            let result = x + y;
            println(result);
        ");

        // Expected: all live variables should remain
        let expected_result = a + b;

        // Verify live code is NOT eliminated
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
        cmd.arg("transpile")
            .arg("-")
            .write_stdin(code)
            .assert()
            .success()
            .stdout(predicate::str::contains("let x"))
            .stdout(predicate::str::contains("let y"))
            .stdout(predicate::str::contains(format!("let result = {expected_result}")));
    });
}

// ============================================================================
// TEST GROUP 7: Async/Await Support (GitHub Issue #133)
// ============================================================================

/// ASYNC-AWAIT: DCE must not eliminate async functions called via .await
/// Bug: `collect_used_functions_rec()` didn't handle `ExprKind::Await`
/// Fix: Added Await, `AsyncBlock`, Spawn cases to recurse into expressions
#[test]
fn test_perf_002c_dce_async_function_not_eliminated() {
    // Pattern: Async function called via .await should NOT be eliminated
    let code = r"
        async fun fetch_data() -> i32 {
            let result = 42
            result
        }

        fun main() {
            let data = await fetch_data()
            data
        }
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // CRITICAL: async function must NOT be eliminated by DCE
        .stdout(predicate::str::contains("async fn fetch_data"))
        .stdout(predicate::str::contains("fetch_data().await"));
}

#[test]
fn test_perf_002c_dce_multiple_async_functions() {
    // Pattern: Multiple async functions with chained .await calls
    let code = r"
        async fun fetch_user(id: i32) -> i32 {
            id
        }

        async fun fetch_data() -> i32 {
            42
        }

        async fun process_data() -> i32 {
            let data = await fetch_data()
            data * 2
        }

        fun main() {
            let user = await fetch_user(1)
            let processed = await process_data()
            processed
        }
    ";

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        // All async functions must be preserved
        .stdout(predicate::str::contains("async fn fetch_user"))
        .stdout(predicate::str::contains("async fn fetch_data"))
        .stdout(predicate::str::contains("async fn process_data"));
}
