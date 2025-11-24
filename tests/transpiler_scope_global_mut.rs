//! TRANSPILER-SCOPE: Top-level let mut should transpile to module-level static mut
//!
//! RED PHASE - These tests MUST fail initially

use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

/// Test 1: Top-level mutable variable used by function
/// MUST transpile to static mut at module level, NOT inside `main()`
#[test]
fn test_transpiler_scope_global_mut_used_by_function() {
    let code = r#"
let mut global_state = 0;

fn modify_global(value: i32) {
    global_state = value;
}

modify_global(42);
println!("{}", global_state);
"#;

    // Transpile
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    let output = cmd
        .arg("transpile")
        .arg("-")
        .write_stdin(code)
        .assert()
        .success();

    let transpiled = String::from_utf8_lossy(&output.get_output().stdout);

    // Should NOT have let mut inside main()
    assert!(
        !transpiled.contains("fn main() {") || !transpiled.contains("let mut global_state"),
        "Should not have 'let mut global_state' inside main(), got: {transpiled}"
    );

    // Write to temp file and compile
    let temp_file = NamedTempFile::new().unwrap();
    let rust_path = temp_file.path().with_extension("rs");
    fs::write(&rust_path, transpiled.as_ref()).unwrap();

    // CRITICAL: Must compile successfully
    let compile_result = std::process::Command::new("rustc")
        .arg(&rust_path)
        .arg("--crate-type")
        .arg("bin")
        .arg("--crate-name")
        .arg("transpiler_scope_test")
        .arg("-o")
        .arg(temp_file.path().with_extension("exe"))
        .output()
        .unwrap();

    assert!(
        compile_result.status.success(),
        "Transpiled code must compile! rustc errors:\n{}",
        String::from_utf8_lossy(&compile_result.stderr)
    );

    // Clean up
    let _ = fs::remove_file(&rust_path);
    let _ = fs::remove_file(temp_file.path().with_extension("exe"));
}

/// Test 2: Compile via ruchy compile command
#[test]
fn test_transpiler_scope_compile_command() {
    let code = r#"
let mut counter = 0;

fn increment() {
    counter = counter + 1;
}

increment();
increment();
println!("{}", counter);
"#;

    let temp_file = NamedTempFile::new().unwrap();
    let ruchy_path = temp_file.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // CRITICAL: Must compile successfully
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("compile").arg(&ruchy_path).assert().success(); // ❌ This will FAIL until we fix the bug

    // Clean up
    let _ = fs::remove_file(&ruchy_path);
}

/// Test 3: Full execution via ruchy run command
#[test]
fn test_transpiler_scope_run_command() {
    let code = r#"
let mut total = 0;

fn add(x: i32) {
    total = total + x;
}

add(10);
add(20);
add(12);
println!("{}", total);
"#;

    let temp_file = NamedTempFile::new().unwrap();
    let ruchy_path = temp_file.path().with_extension("ruchy");
    fs::write(&ruchy_path, code).unwrap();

    // CRITICAL: Must execute and output "42"
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ruchy");
    cmd.arg("run")
        .arg(&ruchy_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));

    // Clean up
    let _ = fs::remove_file(&ruchy_path);
}
