#![allow(missing_docs)]
// STDLIB-003: Advanced File I/O Functions
//
// Implementing: append_file, file_exists, delete_file
// Pattern: Zero-cost abstraction (wrapping Rust std::fs methods)
// Tests: Both interpreter (-e flag) and transpiler (run command) modes
//
// Reference: docs/specifications/stdlib1.20-spec.md - File I/O section

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// file_exists() - Check if file exists
// ============================================================================

#[test]
fn test_stdlib003_file_exists_true() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("exists.txt");
    std::fs::write(&test_file, "content").expect("Failed to create test file");

    let code = format!(
        r#"
let exists = file_exists("{}")
assert_eq(exists, true)
println("File exists: {{}}", exists)
"#,
        test_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("File exists: true"));
}

#[test]
fn test_stdlib003_file_exists_false() {
    let code = r#"
let exists = file_exists("/nonexistent/path/file.txt")
assert_eq(exists, false)
println("File exists: {}", exists)
"#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("File exists: false"));
}

// ============================================================================
// append_file() - Append content to file
// ============================================================================

#[test]
fn test_stdlib003_append_file_new() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("append.txt");

    let code = format!(
        r#"
append_file("{}", "first line\n")
append_file("{}", "second line\n")
let content = read_file("{}")
println("Content: {{}}", content)
"#,
        test_file.display(),
        test_file.display(),
        test_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("first line"))
        .stdout(predicate::str::contains("second line"));
}

#[test]
fn test_stdlib003_append_file_existing() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("append_existing.txt");
    std::fs::write(&test_file, "initial content\n").expect("Failed to create test file");

    let code = format!(
        r#"
append_file("{}", "appended content\n")
let content = read_file("{}")
assert(content.contains("initial content"))
assert(content.contains("appended content"))
println("Content: {{}}", content)
"#,
        test_file.display(),
        test_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("initial content"))
        .stdout(predicate::str::contains("appended content"));
}

// ============================================================================
// delete_file() - Delete file from filesystem
// ============================================================================

#[test]
fn test_stdlib003_delete_file_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("delete_me.txt");
    std::fs::write(&test_file, "content").expect("Failed to create test file");

    let code = format!(
        r#"
let before = file_exists("{}")
assert_eq(before, true)
delete_file("{}")
let after = file_exists("{}")
assert_eq(after, false)
println("Deleted successfully")
"#,
        test_file.display(),
        test_file.display(),
        test_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted successfully"));
}

#[test]
fn test_stdlib003_delete_file_nonexistent() {
    let code = r#"
delete_file("/nonexistent/file.txt")
println("Delete attempted")
"#;

    // Should either succeed silently or return error - depends on implementation
    ruchy_cmd().arg("-e").arg(code).assert();
    // Don't assert success/failure - implementation may vary
}

// ============================================================================
// Transpiler Mode Tests (compile to binary)
// ============================================================================

#[test]
fn test_stdlib003_transpiler_file_exists() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("transpiler_exists.txt");
    std::fs::write(&test_file, "content").expect("Failed to create test file");

    let code = format!(
        r#"
fn main() {{
    let exists = file_exists("{}")
    assert_eq(exists, true)

    let not_exists = file_exists("/nonexistent.txt")
    assert_eq(not_exists, false)

    println("file_exists() works!")
}}
"#,
        test_file.display()
    );

    let mut temp_source = NamedTempFile::new().expect("Failed to create temp file");
    temp_source
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_source.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("file_exists() works!"));
}

#[test]
fn test_stdlib003_transpiler_append_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("transpiler_append.txt");

    let code = format!(
        r#"
fn main() {{
    append_file("{}", "line 1\n")
    append_file("{}", "line 2\n")

    let content = read_file("{}")
    assert(content.contains("line 1"))
    assert(content.contains("line 2"))

    println("append_file() works!")
}}
"#,
        test_file.display(),
        test_file.display(),
        test_file.display()
    );

    let mut temp_source = NamedTempFile::new().expect("Failed to create temp file");
    temp_source
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_source.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("append_file() works!"));
}

#[test]
fn test_stdlib003_transpiler_delete_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_file = temp_dir.path().join("transpiler_delete.txt");
    std::fs::write(&test_file, "content").expect("Failed to create test file");

    let code = format!(
        r#"
fn main() {{
    let before = file_exists("{}")
    assert_eq(before, true)

    delete_file("{}")

    let after = file_exists("{}")
    assert_eq(after, false)

    println("delete_file() works!")
}}
"#,
        test_file.display(),
        test_file.display(),
        test_file.display()
    );

    let mut temp_source = NamedTempFile::new().expect("Failed to create temp file");
    temp_source
        .write_all(code.as_bytes())
        .expect("Failed to write temp file");
    let temp_path = temp_source.path();

    ruchy_cmd()
        .arg("run")
        .arg(temp_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("delete_file() works!"));
}

// ============================================================================
// Integration Tests - Real-world scenarios
// ============================================================================

#[test]
fn test_stdlib003_integration_log_rotation() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_file = temp_dir.path().join("app.log");

    let code = format!(
        r#"
// Simulate log rotation
append_file("{}", "2025-01-01 INFO Starting app\n")
append_file("{}", "2025-01-01 DEBUG Processing request\n")
append_file("{}", "2025-01-01 INFO Request completed\n")

let log = read_file("{}")
assert(log.contains("Starting app"))
assert(log.contains("Processing request"))
assert(log.contains("Request completed"))

println("Log rotation test passed")
"#,
        log_file.display(),
        log_file.display(),
        log_file.display(),
        log_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Log rotation test passed"));
}

#[test]
fn test_stdlib003_integration_temp_file_cleanup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_file = temp_dir.path().join("temp_data.txt");

    let code = format!(
        r#"
// Write temp data
write_file("{}", "temporary content")

// Verify it exists
let exists = file_exists("{}")
assert_eq(exists, true)

// Clean up
delete_file("{}")

// Verify cleanup
let after = file_exists("{}")
assert_eq(after, false)

println("Temp file cleanup test passed")
"#,
        temp_file.display(),
        temp_file.display(),
        temp_file.display(),
        temp_file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Temp file cleanup test passed"));
}
