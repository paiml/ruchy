//! STDLIB-DEFECT-001: env::args() Not Accessible from Ruchy Code
//!
//! **Problem**: env::args() exists in src/stdlib/env.rs but cannot be called from Ruchy
//! **Discovered**: 2025-10-13 (Book compatibility investigation)
//! **Severity**: HIGH
//!
//! Expected: `let args = env::args()` should work
//! Actual: Compilation fails with "use of unresolved module env"
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

/// Test 1: Basic env::args() call
#[test]
fn test_stdlib_defect_001_green_env_args_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let args = env::args();
    println(args);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should run successfully
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

/// Test 2: env::args() with iteration
#[test]
fn test_stdlib_defect_001_green_env_args_iteration() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let args = env::args();
    for arg in args {
        println(arg);
    }
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .arg("arg1")
        .arg("arg2")
        .assert()
        .success();
}

/// Test 3: env::args() in compiled binary
#[test]
fn test_stdlib_defect_001_green_env_args_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let args = env::args();
    println("Args count:");
    println(args.len());
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

/// Test 4: Other env functions (env::var)
#[test]
fn test_stdlib_defect_001_green_env_var() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let path = env::var("PATH");
    println("PATH exists");
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

/// Test 5: Baseline - builtin functions work
#[test]
fn test_stdlib_defect_001_baseline_builtins() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let x = sqrt(16.0);
    println(x);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Builtin functions should work NOW
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .assert()
        .success();
}

// ==================== GREEN PHASE SUMMARY ====================

/// Summary test to document what needs to be fixed
#[test]
fn test_stdlib_defect_001_summary() {
    println!("STDLIB-DEFECT-001: env::args() Not Accessible");
    println!("- env::args() exists in src/stdlib/env.rs:119");
    println!("- But cannot be called from Ruchy code");
    println!("- Error: 'use of unresolved module env'");
    println!("");
    println!("Root Cause:");
    println!("- Transpiler doesn't generate 'use' statements for stdlib");
    println!("- Runtime has env functions but they're not exposed");
    println!("");
    println!("Solution Needed:");
    println!("- Add stdlib module imports to transpiled code");
    println!("- Or expose env functions as builtins");
    println!("- Or create env namespace in runtime");
}
