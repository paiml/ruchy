#![allow(missing_docs)]
//! STDLIB-DEFECT-002: `String.split()` Returns Internal Rust Type
//!
//! **Problem**: .`split()` returns `std::str::Split` iterator instead of Vec<String>
//! **Discovered**: 2025-10-13 (Book compatibility investigation)
//! **Severity**: MEDIUM
//!
//! Expected: `"a,b,c".split(",")` should return ["a", "b", "c"] (Vec<String>)
//! Actual: Returns Split(SplitInternal { ... }) in transpiled code
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

/// Test 1: Basic split with .`len()` access
#[test]
fn test_stdlib_defect_002_green_split_with_len() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "a,b,c";
    let parts = text.split(",");
    println(parts.len());
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should run successfully
    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 2: Split with iteration
#[test]
fn test_stdlib_defect_002_green_split_with_iteration() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "hello,world,test";
    let parts = text.split(",");
    for part in parts {
        println(part);
    }
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 3: Split with indexing
#[test]
fn test_stdlib_defect_002_green_split_with_index() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "first,second,third";
    let parts = text.split(",");
    println(parts[0]);
    println(parts[1]);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 4: Split in compiled binary
#[test]
fn test_stdlib_defect_002_green_split_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let csv = "name,age,city";
    let fields = csv.split(",");
    println("Fields:");
    println(fields.len());
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should compile successfully
    ruchy_cmd()
        .arg("compile")
        .arg(&source)
        .arg("-o")
        .arg(temp.path().join("test_binary"))
        .assert()
        .success();
}

/// Test 5: Split with empty string
#[test]
fn test_stdlib_defect_002_green_split_empty() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "";
    let parts = text.split(",");
    println(parts.len());
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 6: Split with different delimiters
#[test]
fn test_stdlib_defect_002_green_split_various_delims() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text1 = "a;b;c";
    let parts1 = text1.split(";");
    println(parts1.len());

    let text2 = "one two three";
    let parts2 = text2.split(" ");
    println(parts2.len());
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 7: Baseline - builtin functions work
#[test]
fn test_stdlib_defect_002_baseline_builtins() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let text = "hello";
    let upper = text.to_upper();
    println(upper);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Other string methods should work NOW
    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== GREEN PHASE SUMMARY ====================

/// Summary test to document what needs to be fixed
#[test]
fn test_stdlib_defect_002_summary() {
    println!("STDLIB-DEFECT-002: .split() Returns Internal Rust Type");
    println!("- .split() returns std::str::Split iterator");
    println!("- Should return Vec<String>");
    println!("- Error: cannot call .len() on iterator");
    println!();
    println!("Root Cause:");
    println!("- Transpiler emits raw .split() call");
    println!("- Doesn't collect() iterator into Vec");
    println!();
    println!("Solution Needed:");
    println!("- Change transpiler to emit .split().collect::<Vec<_>>()");
    println!("- Similar to how .substring() collects into String");
}
