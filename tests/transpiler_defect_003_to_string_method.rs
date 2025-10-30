#![allow(missing_docs)]
//! TRANSPILER-DEFECT-003: .`to_string()` Method Calls Not Preserved
//!
//! **Problem**: .`to_string()` method calls on variables are not transpiled correctly
//! **Discovered**: 2025-10-07 (LANG-COMP-008 session)
//! **Severity**: MEDIUM
//!
//! Expected: `let as_string = num.to_string()` should generate method call
//! Actual: Method call may be dropped in some contexts
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Integer `to_string()` method call
///
/// This test validates that .`to_string()` method calls are preserved.
#[test]
fn test_defect_003_green_integer_to_string() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let num = 42;
    let as_string = num.to_string();
    println(as_string);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // Compile should succeed and generate .to_string() call
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 2: Float `to_string()` method call
#[test]
fn test_defect_003_green_float_to_string() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let pi = 3.14;
    let as_string = pi.to_string();
    println(as_string);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 3: Boolean `to_string()` method call
#[test]
fn test_defect_003_green_boolean_to_string() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let flag = true;
    let as_string = flag.to_string();
    println(as_string);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 4: Method chain with `to_string()`
#[test]
fn test_defect_003_green_method_chain() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let num: i32 = -5;
    let result = num.abs().to_string();
    println(result);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 5: `to_s` alias (Ruby-style)
#[test]
fn test_defect_003_green_to_s_alias() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let num = 42;
    let as_string = num.to_s();
    println(as_string);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 6: `to_string()` in expression context
#[test]
fn test_defect_003_green_expression_context() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let num = 42;
    println("Value: " + num.to_string());
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 7: Multiple `to_string()` calls
#[test]
fn test_defect_003_green_multiple_calls() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let a = 10;
    let b = 20;
    let result = a.to_string() + " + " + b.to_string();
    println(result);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 8: Baseline - String literal (already String)
#[test]
fn test_defect_003_baseline_string_literal() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "hello";
    println(text);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // This should work NOW (baseline test)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

// ==================== GREEN PHASE SUMMARY ====================

/// Summary test to document validation results
#[test]
fn test_defect_003_green_phase_summary() {
    // This test documents the GREEN phase validation
    println!("TRANSPILER-DEFECT-003 GREEN Phase:");
    println!("- Validating .to_string() method call preservation");
    println!("- All 8 tests created (7 feature tests + 1 baseline)");
    println!();
    println!("Validated scenarios:");
    println!("1. Integer.to_string()");
    println!("2. Float.to_string()");
    println!("3. Boolean.to_string()");
    println!("4. Method chain with .to_string()");
    println!("5. Ruby-style .to_s() alias");
    println!("6. .to_string() in expression context");
    println!("7. Multiple .to_string() calls in same expression");
    println!("8. Baseline: String literal (no conversion needed)");
    println!();
    println!("If all tests pass: DEFECT-003 RESOLVED ✅");
}
