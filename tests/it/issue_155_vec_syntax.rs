//! EXTREME TDD Tests for Issue #155: vec! syntax (comma vs semicolon)
//!
//! Bug: Transpiler generates `vec![0f64, n]` instead of `vec![0f64; n]`
//! Root cause: `MacroInvocation` doesn't distinguish repeat pattern from element list
//! Fix: Add `VecRepeat` variant to AST and handle in transpiler

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

/// Helper to create ruchy command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// RED PHASE: These tests should FAIL before the fix
// ============================================================================

#[test]
fn test_issue_155_01_vec_repeat_basic() {
    // Basic vec repeat pattern: vec![0.0; 5]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(&file_path, "fun main() { let v = vec![0.0; 5]; }").unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // MUST use semicolon, not comma
    assert!(
        output.contains("vec![0f64; 5]") || output.contains("vec![0.0; 5]"),
        "Expected vec![0.0; 5] but got: {output}"
    );
    assert!(
        !output.contains("vec![0f64, 5]") && !output.contains("vec![0.0, 5]"),
        "Should NOT contain comma syntax: {output}"
    );
}

#[test]
fn test_issue_155_02_vec_repeat_with_variable() {
    // Vec repeat with variable size: vec![0.0; n]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        "fun main() { let n = 10; let v = vec![0.0; n]; }",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // MUST use semicolon with variable
    assert!(
        output.contains("; n]") || output.contains("; n }"),
        "Expected semicolon before variable n, but got: {output}"
    );
}

#[test]
fn test_issue_155_03_nested_vec_repeat() {
    // Nested vec repeat for matrix: vec![vec![0.0; n]; m]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        "fun main() { let n = 5; let m = 3; let matrix = vec![vec![0.0; n]; m]; }",
    )
    .unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // Both inner and outer must use semicolon
    assert!(
        output.contains("; n]") && output.contains("; m]"),
        "Expected nested vec with semicolons, but got: {output}"
    );
}

#[test]
fn test_issue_155_04_vec_element_list_still_works() {
    // Normal element list should still use commas: vec![1, 2, 3]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(&file_path, "fun main() { let v = vec![1, 2, 3]; }").unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    // MUST use commas for element list
    assert!(
        output.contains("vec![1, 2, 3]") || output.contains("vec![1i64, 2i64, 3i64]"),
        "Expected vec![1, 2, 3] with commas, but got: {output}"
    );
}

#[test]
fn test_issue_155_05_vec_repeat_compiles() {
    // The transpiled code must actually compile with rustc
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(
        &file_path,
        r#"
fun main() {
    let size = 128;
    let a = vec![vec![0.0; size]; size];
    println!("{}", a.len());
}
"#,
    )
    .unwrap();

    // Transpile
    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    // Try to compile with rustc (should succeed)
    // DEFECT-RACE-CONDITION FIX: Use unique output path to avoid parallel test collisions
    let binary_path = dir.path().join("output_binary");
    ruchy_cmd()
        .arg("compile")
        .arg(&file_path)
        .arg("-o")
        .arg(&binary_path)
        .assert()
        .success();
}

#[test]
fn test_issue_155_06_matmul_example() {
    // The original bug report example (simplified)
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("matmul.ruchy");
    fs::write(
        &file_path,
        r#"
fun matmul(a: Vec<Vec<f64>>, b: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n = a.len();
    let mut c = vec![vec![0.0; n]; n];
    c
}

fun main() {
    let size = 4;
    let a = vec![vec![0.0; size]; size];
    let b = vec![vec![0.0; size]; size];
    let c = matmul(a, b);
    println!("{}", c.len());
}
"#,
    )
    .unwrap();

    // Should transpile successfully
    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();

    // Verify correct syntax with semicolons
    assert!(
        output.contains("; n]") || output.contains("; size]"),
        "Expected semicolon in vec repeat, but got: {output}"
    );

    // Should NOT have the buggy comma syntax
    assert!(
        !output.contains("vec![vec![0f64, n], n]") && !output.contains("vec![vec![0.0, n], n]"),
        "Should NOT have comma syntax bug: {output}"
    );
}

#[test]
fn test_issue_155_07_single_element_vec() {
    // Single element vec should still work: vec![42]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(&file_path, "fun main() { let v = vec![42]; }").unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    assert!(
        output.contains("vec![42]") || output.contains("vec![42i64]"),
        "Single element vec should work: {output}"
    );
}

#[test]
fn test_issue_155_08_empty_vec() {
    // Empty vec should still work: vec![]
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    fs::write(&file_path, "fun main() { let v: Vec<i32> = vec![]; }").unwrap();

    let output_path = dir.path().join("output.rs");
    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    assert!(
        output.contains("vec![]") || output.contains("Vec::new()"),
        "Empty vec should work: {output}"
    );
}
