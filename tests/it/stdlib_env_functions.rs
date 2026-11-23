#![allow(missing_docs)]
//! STDLIB Phase 1: Remaining Environment Functions Tests
//!
//! **Task**: Implement remaining 6 environment functions
//! **Priority**: HIGH (Phase 1 of `STDLIB_ACCESS_PLAN`)
//! **Pattern**: Three-layer builtin function (proven from `env_args` and `env_var`)
//!
//! Functions:
//! 1. `env_set_var(key`: String, value: String) -> Result<()>
//! 2. `env_remove_var(key`: String) -> Result<()>
//! 3. `env_vars()` -> `HashMap`<String, String>
//! 4. `env_current_dir()` -> Result<String>
//! 5. `env_set_current_dir(path`: String) -> Result<()>
//! 6. `env_temp_dir()` -> String
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== env_set_var() Tests ====================

#[test]
fn test_env_set_var_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    env_set_var("RUCHY_TEST", "hello");
    let value = env_var("RUCHY_TEST");
    println(value);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_env_set_var_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    env_set_var("TEST_KEY", "test_value");
    println("Environment variable set");
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

// ==================== env_remove_var() Tests ====================

#[test]
fn test_env_remove_var_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    env_set_var("RUCHY_REMOVE_TEST", "value");
    env_remove_var("RUCHY_REMOVE_TEST");
    println("Variable removed");
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== env_vars() Tests ====================

#[test]
fn test_env_vars_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let vars = env_vars();
    println("Environment variables loaded");
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_env_vars_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let all_vars = env_vars();
    println("Got all environment variables");
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

// ==================== env_current_dir() Tests ====================

#[test]
fn test_env_current_dir_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let dir = env_current_dir();
    println("Current directory:");
    println(dir);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_env_current_dir_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let cwd = env_current_dir();
    println("Working directory found");
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

// ==================== env_set_current_dir() Tests ====================

#[test]
fn test_env_set_current_dir_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let original = env_current_dir();
    env_set_current_dir("/tmp");
    let new_dir = env_current_dir();
    println("Changed directory");
    env_set_current_dir(original);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

// ==================== env_temp_dir() Tests ====================

#[test]
fn test_env_temp_dir_basic() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let tmp = env_temp_dir();
    println("Temp directory:");
    println(tmp);
}
"#;

    fs::write(&source, code).expect("Failed to write test file");

    ruchy_cmd().arg("run").arg(&source).assert().success();
}

#[test]
fn test_env_temp_dir_compile() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r#"
fun main() {
    let temp_path = env_temp_dir();
    println("Temp directory available");
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

// ==================== Summary Test ====================

#[test]
fn test_env_functions_summary() {
    println!("STDLIB Phase 1: Remaining Environment Functions");
    println!("1. env_set_var(key, value) - Set environment variable");
    println!("2. env_remove_var(key) - Remove environment variable");
    println!("3. env_vars() - Get all environment variables as HashMap");
    println!("4. env_current_dir() - Get current working directory");
    println!("5. env_set_current_dir(path) - Change current directory");
    println!("6. env_temp_dir() - Get system temp directory");
    println!();
    println!("Three-Layer Implementation Required for each:");
    println!("1. Runtime: builtin_* in builtins.rs");
    println!("2. Transpiler: case in try_transpile_environment_function()");
    println!("3. Environment: eval_* in eval_builtin.rs + builtin_init.rs");
}
