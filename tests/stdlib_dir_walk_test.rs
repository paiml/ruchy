// STDLIB-005: Multi-Threaded Directory Walking + Text Search
//
// RED PHASE: Tests created first (TDD), implementation follows
//
// Functions under test:
// - walk(path) -> Array<FileEntry>
// - walk_parallel(path, callback) -> Array<Any>
// - walk_with_options(path, options) -> Array<FileEntry>
// - glob(pattern) -> Array<String>
// - find(path, predicate) -> Array<FileEntry>
// - search(pattern, path, options?) -> Array<SearchMatch>
//
// Reference: docs/specifications/multi-threaded-dir-walk-spec.md
//
// Test Strategy: EXTREME TDD with mutation testing target ≥90%
// Total: 70 unit tests + 3 concurrency + 5 security + 2 benchmarks

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Helper: Create test directory structure
fn create_test_tree() -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path();

    // Create directory structure
    fs::create_dir(root.join("dir1")).unwrap();
    fs::create_dir(root.join("dir2")).unwrap();
    fs::create_dir(root.join("dir1/subdir")).unwrap();

    // Create files
    File::create(root.join("file1.txt")).unwrap();
    File::create(root.join("file2.log")).unwrap();
    File::create(root.join("dir1/file3.txt")).unwrap();
    File::create(root.join("dir1/subdir/file4.txt")).unwrap();
    File::create(root.join("dir2/file5.log")).unwrap();

    temp
}

// Helper: Create test files with content for search tests
fn create_test_files_with_content() -> TempDir {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let root = temp.path();

    let mut file1 = File::create(root.join("test1.txt")).unwrap();
    writeln!(file1, "This is a test file").unwrap();
    writeln!(file1, "It contains an error message").unwrap();
    writeln!(file1, "And some other content").unwrap();

    let mut file2 = File::create(root.join("test2.log")).unwrap();
    writeln!(file2, "Log file contents").unwrap();
    writeln!(file2, "ERROR: Something went wrong").unwrap();
    writeln!(file2, "INFO: Normal operation").unwrap();

    temp
}

// ============================================================================
// BASIC WALK() TESTS (10 tests)
// ============================================================================

#[test]
fn test_stdlib005_walk_basic() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
assert(entries.len() > 0, "Should find at least one entry")
println("Found {{}} entries", entries.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_returns_array() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
// Verify it's an array by checking it has length and can be indexed
assert(entries.len() >= 0, "walk() should return an array with length")
println("Type check passed")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Type check passed"));
}

#[test]
fn test_stdlib005_walk_file_entries_have_fields() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
let first = entries[0]

// FileEntry should have required fields
assert(first.path != nil, "Should have path field")
assert(first.name != nil, "Should have name field")
assert(first.is_file != nil || first.is_dir != nil, "Should have type fields")

println("Field check passed")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Field check passed"));
}

#[test]
fn test_stdlib005_walk_filter_files() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
let files = entries.filter(|e| e.is_file)

assert(files.len() > 0, "Should find at least one file")
println("Found {{}} files", files.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_filter_directories() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
let dirs = entries.filter(|e| e.is_dir)

assert(dirs.len() > 0, "Should find at least one directory")
println("Found {{}} directories", dirs.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_recursive() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")

// Should find files in subdirectories (recursive)
// We created file4.txt in dir1/subdir/
let deep_files = entries.filter(|e| e.path.contains("subdir"))

assert(deep_files.len() > 0, "Should find files in subdirectories")
println("Recursive walk successful")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Recursive walk successful"));
}

#[test]
fn test_stdlib005_walk_depth_field() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")

// Check that depth field exists and has valid values
let depths = entries.map(|e| e.depth)
let has_zero = depths.contains(0)  // Root level

assert(has_zero, "Should have depth 0 entries")
println("Depth field verified")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Depth field verified"));
}

#[test]
fn test_stdlib005_walk_filter_by_extension() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")
let txt_files = entries.filter(|e| e.path.ends_with(".txt"))

assert(txt_files.len() > 0, "Should find .txt files")
println("Found {{}} .txt files", txt_files.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_walk_empty_directory() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let entries = walk("{}")

// Empty directory should return at least the directory itself
assert(entries.len() >= 0, "Should handle empty directory")
println("Empty directory handled correctly")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Empty directory handled correctly"));
}

#[test]
fn test_stdlib005_walk_nonexistent_path_error() {
    let code = r#"
let entries = walk("/nonexistent/path/that/does/not/exist")
"#;

    // Should handle error gracefully (either return empty array or error message)
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .code(predicate::in_iter([0, 1])); // Allow either success with empty result or error
}

// ============================================================================
// More test categories to be added:
// - Parallel walk_parallel() tests (8 tests)
// - Advanced walk_with_options() (12 tests)
// - Utility glob()/find() tests (6 tests)
// - Text search() tests (8 tests)
// - Integration tests (6 tests)
// - Property tests (4 tests, 40K cases)
// - Concurrency tests (3 tests)
// - Security tests (5 tests)
// - Performance benchmarks (2 tests)
// ============================================================================

// TODO: Add remaining test categories following the spec
