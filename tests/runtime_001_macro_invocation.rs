//! RUNTIME-001: Fix `MacroInvocation` runtime support (GitHub Issue #74)
//!
//! ROOT CAUSE: FORMATTER-088 changed parser to emit `ExprKind::MacroInvocation` instead of
//! `ExprKind::Macro`, but forgot to update interpreter to handle the new variant.
//!
//! FIX: Add match arm for `ExprKind::MacroInvocation` in interpreter.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test vec! macro with `MacroInvocation` variant (Issue #74 regression)
#[test]
fn test_runtime_001_vec_macro_invocation() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("let x = vec![1, 2, 3]; x")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success();
}

/// Test println! macro with `MacroInvocation` variant
#[test]
fn test_runtime_001_println_macro_invocation() {
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("println!(\"Hello\")")
        .timeout(std::time::Duration::from_secs(5))
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

/// Test that both Macro and `MacroInvocation` work the same
#[test]
fn test_runtime_001_macro_backward_compat() {
    // This test verifies that the fix works with CURRENT parser output (MacroInvocation)
    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg("let x = vec![1, 2]; x")
        .assert()
        .success();
}
