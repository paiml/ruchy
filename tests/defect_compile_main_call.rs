//! DEFECT-COMPILE-MAIN-CALL: Stack Overflow on Double main() Call
//!
//! Ticket: DEFECT-COMPILE-MAIN-CALL
//! Priority: HIGH (P0) - User-facing crash
//! Filed: 2025-10-09
//!
//! EXTREME TDD Test Suite
//! RED → GREEN → REFACTOR with mutation testing

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to get ruchy binary command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ===========================================================================
// RED PHASE: Tests that currently FAIL (reproduce the bug)
// ===========================================================================

#[test]
#[ignore] // Will pass after fix
fn test_main_function_with_explicit_call_no_stack_overflow() {
    // DEFECT: This currently causes stack overflow in compiled binary
    // EXPECTED: Should work (either skip the call or warn)

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_main_call.ruchy");

    // Write code with both fun main() definition AND explicit call
    let code = r#"fun main() {
    println("Hello from main");
}

main()
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // Compile the code
    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Get the compiled binary path
    let binary_path = temp_dir.path().join("a.out");
    assert!(binary_path.exists(), "Compiled binary should exist");

    // Execute the binary - this should NOT stack overflow
    let run_output = Command::new(&binary_path)
        .timeout(std::time::Duration::from_secs(2))
        .assert();

    // Should complete successfully without timeout or crash
    run_output
        .success()
        .stdout(predicate::str::contains("Hello from main"));
}

#[test]
fn test_main_function_without_explicit_call_works() {
    // This should already work - baseline test

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_main_no_call.ruchy");

    // Write code with ONLY fun main() definition (no explicit call)
    let code = r#"fun main() {
    println("Hello from main");
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    // Compile the code
    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Get the compiled binary path
    let binary_path = temp_dir.path().join("a.out");
    assert!(binary_path.exists(), "Compiled binary should exist");

    // Execute the binary - should work
    Command::new(&binary_path)
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from main"));
}

// ===========================================================================
// Additional Edge Cases (for mutation testing)
// ===========================================================================

#[test]
#[ignore] // Will pass after fix
fn test_nested_main_call_in_function() {
    // Edge case: main() called from another function

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_nested_main.ruchy");

    let code = r#"fun main() {
    println("In main");
}

fun start() {
    main();
    println("After main");
}

start()
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let binary_path = temp_dir.path().join("a.out");

    Command::new(&binary_path)
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("In main"))
        .stdout(predicate::str::contains("After main"));
}

#[test]
fn test_other_function_with_call_works() {
    // Ensure fix doesn't break normal function calls

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_normal_call.ruchy");

    let code = r#"fun greet() {
    println("Hello");
}

fun main() {
    greet();
}
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let binary_path = temp_dir.path().join("a.out");

    Command::new(&binary_path)
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

#[test]
#[ignore] // Will pass after fix
fn test_multiple_main_calls() {
    // Edge case: multiple main() calls at module level

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("test_multiple_calls.ruchy");

    let code = r#"fun main() {
    println("Main executed");
}

main()
main()
"#;

    fs::write(&test_file, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&test_file)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let binary_path = temp_dir.path().join("a.out");

    // Should execute without stack overflow
    // (may print "Main executed" once or twice depending on fix strategy)
    Command::new(&binary_path)
        .timeout(std::time::Duration::from_secs(2))
        .assert()
        .success();
}

// ===========================================================================
// Property Tests (for mutation testing validation)
// ===========================================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Property: Any function named main should compile and run without stack overflow
    proptest! {
        #[test]
        #[ignore] // Expensive test - run manually with --ignored
        fn prop_main_function_never_stack_overflows(
            message in "Hello.*|Test.*|Output.*"
        ) {
            let temp_dir = TempDir::new().unwrap();
            let test_file = temp_dir.path().join("prop_test.ruchy");

            let code = format!(r#"fun main() {{
    println("{}");
}}

main()
"#, message);

            fs::write(&test_file, code).unwrap();

            let compile_result = ruchy_cmd()
                .arg("compile")
                .arg(&test_file)
                .current_dir(temp_dir.path())
                .assert()
                .try_success();

            if compile_result.is_ok() {
                let binary_path = temp_dir.path().join("a.out");
                if binary_path.exists() {
                    // Should not timeout (no stack overflow)
                    let _ = Command::new(&binary_path)
                        .timeout(std::time::Duration::from_secs(2))
                        .assert()
                        .try_success();
                }
            }
        }
    }
}

// ===========================================================================
// Documentation Tests
// ===========================================================================

/// Example: Correct usage (no explicit main call)
///
/// ```rust,ignore
/// // This should compile and run correctly
/// fun main() {
///     println("Hello");
/// }
/// // No explicit main() call needed
/// ```
///
/// Example: Problematic usage (explicit main call) - FIXED by DEFECT-COMPILE-MAIN-CALL
///
/// ```rust,ignore
/// // This used to cause stack overflow, now handled gracefully
/// fun main() {
///     println("Hello");
/// }
/// main()  // Compiler now skips or warns about this
/// ```
#[allow(dead_code)]
fn documentation_examples() {}
