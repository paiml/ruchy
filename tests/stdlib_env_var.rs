#![allow(missing_docs)]
//! STDLIB Phase 1: `env_var()` Implementation Tests
//!
//! **Task**: Implement `env_var(key`: String) -> Result<String>
//! **Priority**: HIGH (Phase 1 of `STDLIB_ACCESS_PLAN`)
//! **Pattern**: Three-layer builtin function (proven from `env_args`)
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

/// Test 1: Basic `env_var()` with existing variable
#[test]
fn test_env_var_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    // PATH should exist in all environments
    let path = env_var("PATH");
    println("PATH exists");
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should run successfully
    ruchy_cmd().arg("run").arg(&source).assert().success();
}

/// Test 2: `env_var()` in compiled mode
#[test]
fn test_env_var_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let home = env_var("HOME");
    println("Home directory found");
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

/// Test 3: `env_var()` with custom variable
#[test]
fn test_env_var_custom() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let value = env_var("TEST_RUCHY_VAR");
    println(value);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Set custom environment variable
    ruchy_cmd()
        .arg("run")
        .arg(&source)
        .env("TEST_RUCHY_VAR", "test_value_123")
        .assert()
        .success();
}

/// Test 4: `env_var()` error handling for missing variable
#[test]
fn test_env_var_missing() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let value = env_var("NONEXISTENT_VAR_XYZ");
    println(value);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    // Should fail with error about missing variable
    ruchy_cmd().arg("run").arg(&source).assert().failure();
}

/// Test 5: `env_var()` with wrong number of arguments
#[test]
fn test_env_var_wrong_args() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun main() {
    let value = env_var();  // Missing argument
}
";

    fs::write(&source, code).expect("Failed to write test file");

    // Should fail during compilation or runtime
    ruchy_cmd().arg("run").arg(&source).assert().failure();
}

// ==================== GREEN PHASE VERIFICATION ====================

/// Summary test documenting what needs to be implemented
#[test]
fn test_env_var_summary() {
    println!("STDLIB Phase 1: env_var() Implementation");
    println!("- Function: env_var(key: String) -> Result<String>");
    println!("- Retrieves environment variable by key");
    println!("- Returns error if variable doesn't exist");
    println!();
    println!("Three-Layer Implementation Required:");
    println!("1. Runtime: builtin_env_var() in builtins.rs");
    println!("2. Transpiler: env_var case in try_transpile_environment_function()");
    println!("3. Environment: eval_env_var() in eval_builtin.rs");
}
