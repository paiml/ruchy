#![allow(missing_docs)]
//! CLI Contract Tests: `ruchy compile`
//!
//! **Purpose**: Validate user-facing contract (exit codes, stdio, binary creation)
//! **Layer 4**: CLI expectation testing (black-box validation)
//!
//! **Contract Specification**:
//! - Exit code 0: Successful compilation
//! - Exit code 1: Compilation error OR file not found OR rustc not available
//! - stdout: Success message with binary path
//! - stderr: Error messages (compilation errors, missing rustc)
//! - Binary creation: Output file created and executable
//!
//! **Reference**: docs/specifications/15-tool-improvement-spec.md (v4.0)
//! **TICR**: docs/testing/TICR-ANALYSIS.md (compile: 0.2 → target 0.4)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// CLI CONTRACT TESTS: EXIT CODES
// ============================================================================

#[test]
fn cli_compile_valid_program_exits_zero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "hello.ruchy", "println(\"Hello, World!\")\n");
    let output = temp.path().join("hello");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success(); // Exit code 0
}

#[test]
fn cli_compile_syntax_error_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "invalid.ruchy", "let x = \n");
    let output = temp.path().join("invalid");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure(); // Exit code != 0
}

#[test]
fn cli_compile_missing_file_exits_nonzero() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output");

    ruchy_cmd()
        .arg("compile")
        .arg("nonexistent_xyz.ruchy")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure(); // Exit code != 0
}

// ============================================================================
// CLI CONTRACT TESTS: STDOUT (success messages)
// ============================================================================

#[test]
fn cli_compile_success_writes_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "simple.ruchy", "let x = 42\n");
    let output = temp.path().join("simple");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully compiled"));
}

#[test]
fn cli_compile_output_path_in_stdout() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "program.ruchy", "println(\"test\")\n");
    let output = temp.path().join("my_program");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("my_program"));
}

// ============================================================================
// CLI CONTRACT TESTS: STDERR (error messages)
// ============================================================================

#[test]
fn cli_compile_syntax_error_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "bad_syntax.ruchy", "fun f( { }\n");
    let output = temp.path().join("bad");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not()); // stderr NOT empty
}

#[test]
fn cli_compile_missing_file_writes_stderr() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("output");

    ruchy_cmd()
        .arg("compile")
        .arg("missing.ruchy")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("No such file"))
                .or(predicate::str::contains("does not exist")),
        );
}

// ============================================================================
// CLI CONTRACT TESTS: BINARY CREATION
// ============================================================================

#[test]
fn cli_compile_creates_output_binary() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "create_bin.ruchy", "println(\"binary test\")\n");
    let output = temp.path().join("test_binary");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Verify binary was created
    assert!(output.exists(), "Binary should be created at output path");
}

#[test]
fn cli_compile_binary_is_executable() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "exec_test.ruchy", "println(\"executable\")\n");
    let output = temp.path().join("exec_binary");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    // Verify binary has executable permissions (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&output).expect("Binary should exist");
        let permissions = metadata.permissions();
        assert!(
            permissions.mode() & 0o111 != 0,
            "Binary should be executable"
        );
    }
}

#[test]
fn cli_compile_default_output_name() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "default.ruchy", "println(\"default name\")\n");

    // Change to temp directory so a.out is created there
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp.path()).unwrap();

    ruchy_cmd().arg("compile").arg(&file).assert().success();

    // Verify a.out was created (default name)
    let default_output = temp.path().join("a.out");
    assert!(
        default_output.exists(),
        "Default output 'a.out' should be created"
    );

    // Restore original directory
    std::env::set_current_dir(current_dir).unwrap();
}

// ============================================================================
// CLI CONTRACT TESTS: OPTIMIZATION FLAGS
// ============================================================================

#[test]
fn cli_compile_with_optimization_level() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "optimized.ruchy", "println(\"optimized\")\n");
    let output = temp.path().join("optimized");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .arg("-O")
        .arg("3") // Max optimization
        .assert()
        .success();

    assert!(output.exists(), "Optimized binary should be created");
}

#[test]
fn cli_compile_with_strip_flag() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "stripped.ruchy", "println(\"stripped\")\n");
    let output = temp.path().join("stripped");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .arg("--strip") // Strip debug symbols
        .assert()
        .success();

    assert!(output.exists(), "Stripped binary should be created");
}

// ============================================================================
// CLI CONTRACT TESTS: EDGE CASES
// ============================================================================

#[test]
fn cli_compile_empty_file_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "empty.ruchy", "");
    let output = temp.path().join("empty");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure(); // Empty file should fail
}

#[test]
fn cli_compile_comment_only_fails() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "comments.ruchy", "// Just a comment\n");
    let output = temp.path().join("comments");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .failure(); // Comment-only should fail
}

#[test]
fn cli_compile_complex_program() {
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "complex.ruchy",
        r"
fun fibonacci(n) {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

println(fibonacci(10))
",
    );
    let output = temp.path().join("complex");

    ruchy_cmd()
        .arg("compile")
        .arg(&file)
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    assert!(
        output.exists(),
        "Complex program should compile successfully"
    );
}
