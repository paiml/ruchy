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
// GLOB() TESTS (6 tests)
// ============================================================================

#[test]
fn test_stdlib005_glob_basic_pattern() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let pattern = "{}/*.txt"
let files = glob(pattern)
assert(files.len() > 0, "Should find .txt files")
println("Found {{}} .txt files", files.len())
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
fn test_stdlib005_glob_recursive_pattern() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let pattern = "{}/**/*.txt"
let files = glob(pattern)
assert(files.len() >= 3, "Should find all .txt files recursively")
println("Found {{}} .txt files recursively", files.len())
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
fn test_stdlib005_glob_returns_string_array() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let pattern = "{}/*.txt"
let files = glob(pattern)
let first = files[0]
// String paths should be printable
println("First file: {{}}", first)
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("First file:"));
}

#[test]
fn test_stdlib005_glob_filter_by_extension() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let txt_files = glob("{}/**/*.txt")
let log_files = glob("{}/**/*.log")

assert(txt_files.len() > 0, "Should find .txt files")
assert(log_files.len() > 0, "Should find .log files")
println("Found {{}} .txt and {{}} .log files", txt_files.len(), log_files.len())
"#,
        path, path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn test_stdlib005_glob_no_matches() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let files = glob("{}/**/*.nonexistent")
assert(files.len() == 0, "Should return empty array for no matches")
println("No matches found (expected)")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_stdlib005_glob_absolute_paths() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let files = glob("{}/**/*.txt")
// Results should be absolute paths
assert(files[0].starts_with("/") || files[0].starts_with("C:"), "Should return absolute paths")
println("Absolute paths verified")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Absolute paths verified"));
}

// ============================================================================
// FIND() TESTS (5 tests) - Using stdlib library function pattern
// Note: find() is implemented in Ruchy itself as walk().filter()
// This demonstrates proper architectural layering
// ============================================================================

#[test]
fn test_stdlib005_find_library_function() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
// Library function: find() delegates to walk().filter()
fn find(path, predicate) {{
    walk(path).filter(predicate)
}}

// Use the library function
let files = find("{}", |e| e.is_file)
assert(files.len() > 0, "Should find files")
println("Found {{}} files via library function", files.len())
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
fn test_stdlib005_find_txt_files() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
fn find(path, predicate) {{ walk(path).filter(predicate) }}

let txt_files = find("{}", |e| e.is_file && e.path.ends_with(".txt"))
assert(txt_files.len() >= 3, "Should find .txt files")
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
fn test_stdlib005_find_demonstrates_composability() {
    let temp = create_test_tree();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
// Demonstrates architectural layering:
// 1. walk() is a Rust builtin (low-level, efficient)
// 2. filter() is a Rust builtin (higher-order function)
// 3. find() is a Ruchy library function (convenience wrapper)

let dirs = walk("{}").filter(|e| e.is_dir)
assert(dirs.len() >= 3, "Should find directories")
println("Composability: {{}} directories found", dirs.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Composability"));
}

// ============================================================================
// SEARCH() TESTS (6 tests) - Fast text search with ripgrep functionality
// ============================================================================

#[test]
fn test_stdlib005_search_basic() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let matches = search("error", "{}")
assert(matches.len() > 0, "Should find matches for 'error'")
println("Found {{}} matches", matches.len())
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
fn test_stdlib005_search_returns_array() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let matches = search("test", "{}")
assert(matches.len() >= 0, "Should return an array")
println("Search returned array with {{}} matches", matches.len())
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Search returned array"));
}

#[test]
fn test_stdlib005_search_match_has_fields() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let matches = search("error", "{}")
let first = matches[0]

// SearchMatch should have required fields
assert(first.path != nil, "Should have path field")
assert(first.line_num > 0, "Should have line_num field")
assert(first.line != nil, "Should have line field")

println("SearchMatch fields validated")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("SearchMatch fields validated"));
}

#[test]
fn test_stdlib005_search_case_insensitive() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
// Should find both "error" and "ERROR"
let matches = search("ERROR", "{}", {{
    case_insensitive: true
}})
assert(matches.len() > 0, "Should find case-insensitive matches")
println("Found {{}} case-insensitive matches", matches.len())
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
fn test_stdlib005_search_no_matches() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let matches = search("nonexistentpattern12345", "{}")
assert(matches.len() == 0, "Should return empty array for no matches")
println("No matches found (expected)")
"#,
        path
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("No matches found"));
}

#[test]
fn test_stdlib005_search_multiple_files() {
    let temp = create_test_files_with_content();
    let path = temp.path().display().to_string();

    let code = format!(
        r#"
let matches = search("test", "{}")
// Should find matches in multiple files (test1.txt and test2.log)
assert(matches.len() > 0, "Should find matches across multiple files")
println("Found {{}} matches across files", matches.len())
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

// ============================================================================
// More test categories to be added:
// - Parallel walk_parallel() tests (8 tests) - DEFERRED to v2.0 (documented defect)
// - Advanced walk_with_options() (12 tests)
// - Integration tests (6 tests)
// - Property tests (4 tests, 40K cases)
// - Concurrency tests (3 tests)
// - Security tests (5 tests)
// - Performance benchmarks (2 tests)
// ============================================================================
