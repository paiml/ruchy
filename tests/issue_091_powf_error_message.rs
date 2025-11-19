// Issue #91: Improve .powf() error message to suggest ** operator
//
// Problem: Current error "Float method 'powf' takes no arguments" is misleading
//          because it suggests powf exists but takes no args, when actually
//          powf doesn't exist at all and ** operator should be used
//
// Root Cause: Generic error message in eval_float_method() doesn't special-case
//             common Rust methods that don't exist in Ruchy
//
// Solution: Add special handling for 'powf' to suggest ** operator instead
//
// Test Strategy: E2E tests verifying helpful error message with actionable suggestion

use assert_cmd::Command;
use predicates::prelude::*;

/// Test 1: .`powf()` with arguments shows helpful error (not misleading "takes no arguments")
///
/// Current (BAD): "Float method 'powf' takes no arguments"
/// Target (GOOD): "Float method 'powf' not available. Use ** operator for exponentiation"
#[test]
fn test_issue_091_powf_with_args_helpful_error() {
    let script_code = r#"
fun main() {
    let result = (2.0).powf(3.0);
    println!("Result: {}", result);
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("powf"))
        .stderr(predicate::str::contains("**"))
        .stderr(predicate::str::contains("exponentiation").or(predicate::str::contains("power")));
}

/// Test 2: .`powf()` without arguments also shows helpful error
#[test]
fn test_issue_091_powf_no_args_helpful_error() {
    let script_code = r#"
fun main() {
    let result = (2.0).powf();
    println!("Result: {}", result);
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("powf"))
        .stderr(predicate::str::contains("**").or(predicate::str::contains("operator")));
}

/// Test 3: Verify ** operator works correctly (documentation)
#[test]
fn test_issue_091_pow_operator_works() {
    let script_code = r#"
fun main() {
    let result = 2.0 ** 3.0;
    println!("Result: {}", result);

    if result == 8.0 {
        println!("✓ Power operator works correctly");
    }
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Result: 8"))
        .stdout(predicate::str::contains("✓ Power operator works correctly"));
}

/// Test 4: Error message should NOT say "takes no arguments" (misleading)
#[test]
fn test_issue_091_no_misleading_takes_no_arguments() {
    let script_code = r"
fun main() {
    let result = (2.0).powf(3.0);
}
";

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_code);

    // Should NOT contain the misleading phrase "takes no arguments"
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("takes no arguments").not());
}

/// Test 5: Other float methods still work and have correct error messages
#[test]
fn test_issue_091_other_float_methods_unchanged() {
    // Test that .sqrt() still works
    let script_sqrt = r#"
fun main() {
    let result = (9.0).sqrt();
    println!("sqrt(9.0) = {}", result);
}
"#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e").arg(script_sqrt);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sqrt(9.0) = 3"));

    // Test that unknown method still gives appropriate error
    let script_unknown = r"
fun main() {
    let result = (2.0).nonexistent();
}
";

    let mut cmd2 = Command::cargo_bin("ruchy").unwrap();
    cmd2.arg("-e").arg(script_unknown);
    cmd2.assert().failure().stderr(
        predicate::str::contains("Unknown float method")
            .or(predicate::str::contains("nonexistent")),
    );
}
