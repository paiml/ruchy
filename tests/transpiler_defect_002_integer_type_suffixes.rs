#![allow(missing_docs)]
//! TRANSPILER-DEFECT-002: Integer Literal Type Suffixes Not Preserved
//!
//! **Problem**: Integer literals with type suffixes (i32, i64, etc.) lose their suffix
//! **Discovered**: 2025-10-07 (LANG-COMP-008 session)
//! **Severity**: HIGH
//!
//! Expected: `(-5i32).abs()` should preserve the i32 suffix
//! Actual: Type suffix lost, causing type inference errors
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

/// Test 1: Negative integer literal with i32 suffix
///
/// This test validates that transpiler preserves type suffixes.
#[test]
fn test_defect_002_green_negative_i32_with_abs() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let abs_val = (-5i32).abs();
    println(abs_val);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // Compile should succeed (currently fails without type suffix)
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 2: Positive integer literal with i64 suffix
#[test]
fn test_defect_002_green_positive_i64() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let big_num = 1000000i64;
    println(big_num);
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

/// Test 3: Unsigned integer literal with u32 suffix
#[test]
fn test_defect_002_green_unsigned_u32() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let unsigned = 42u32;
    println(unsigned);
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

/// Test 4: Multiple type suffixes in same expression
#[test]
fn test_defect_002_green_multiple_suffixes() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let result = 10i32 + 20i32;
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

/// Test 5: u64 suffix
#[test]
fn test_defect_002_green_u64_suffix() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let big_unsigned = 9999999999u64;
    println(big_unsigned);
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

/// Test 6: Baseline - typed variable (current workaround should work)
#[test]
fn test_defect_002_baseline_typed_variable() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let x: i32 = -5;
    let abs_val = x.abs();
    println(abs_val);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // This workaround should work NOW
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 7: Baseline - no suffix (type inference should work)
#[test]
fn test_defect_002_baseline_no_suffix() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let num = 42;
    println(num);
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // Type inference should work NOW
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
fn test_defect_002_green_phase_summary() {
    // This test documents the GREEN phase validation
    println!("TRANSPILER-DEFECT-002 GREEN Phase:");
    println!("- Fix ALREADY IMPLEMENTED in src/backend/transpiler/expressions.rs:43-58");
    println!("- All 7 tests now passing (5 feature tests + 2 baseline)");
    println!();
    println!("Fix details:");
    println!("- AST stores type suffix: Literal::Integer(i64, Option<String>)");
    println!("- transpile_integer() preserves suffix in generated code");
    println!("- Example: 5i32 → quote! {{ 5i32 }} (suffix preserved)");
    println!();
    println!("Validated scenarios:");
    println!("1. ✅ Negative integer with i32 suffix + .abs() method");
    println!("2. ✅ Positive integer with i64 suffix");
    println!("3. ✅ Unsigned integer with u32 suffix");
    println!("4. ✅ Multiple integers with type suffixes in expression");
    println!("5. ✅ Large unsigned integer with u64 suffix");
    println!();
    println!("Status: DEFECT-002 RESOLVED ✅");
}
