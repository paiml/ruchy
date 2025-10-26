// STDLIB-005: walk_parallel() Tests - Perfect Composable API
//
// Architecture: walk_parallel() does parallel I/O, returns FileEntry array
// Users compose transformations via .filter(), .map(), etc
//
// WHY THIS IS PERFECT:
// - Parallel I/O (directory walking is I/O-bound - biggest bottleneck)
// - Clean separation: builtins don't execute closures
// - Composable: users chain array methods for transformations
// - Memory efficient: rayon's work-stealing handles millions of files

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// Test 1: Basic parallel walk
#[test]
fn test_walk_parallel_basic() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp.path().join("file2.txt"), "content2").unwrap();

    let code = format!(r#"
        let entries = walk_parallel("{}")
        let files = entries.filter(fn(e) {{ e.is_file }})

        println("Found {{}} files", files.len())
        assert(files.len() == 2, "Expected 2 files")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 2 files"));
}

// Test 2: Composable transformations (filter + map)
#[test]
fn test_walk_parallel_composable() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("a.txt"), "aaa").unwrap();
    fs::write(temp.path().join("b.txt"), "bbbbb").unwrap();

    let code = format!(r#"
        let file_sizes = walk_parallel("{}")
            .filter(fn(e) {{ e.is_file }})
            .map(fn(e) {{ e.size }})

        println("Sizes: {{:?}}", file_sizes)
        assert(file_sizes.len() == 2, "Expected 2 sizes")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success();
}

// Test 3: Returns proper FileEntry objects
#[test]
fn test_walk_parallel_file_entry_structure() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("test.txt"), "content").unwrap();

    let code = format!(r#"
        let entries = walk_parallel("{}")
        let first = entries[0]

        // Verify FileEntry has all required fields
        assert(first.path != nil, "Should have path field")
        assert(first.name != nil, "Should have name field")
        assert(first.is_file != nil, "Should have is_file field")
        assert(first.is_dir != nil, "Should have is_dir field")
        assert(first.size != nil, "Should have size field")
        assert(first.depth != nil, "Should have depth field")

        println("FileEntry structure validated")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("FileEntry structure validated"));
}

// Test 4: Empty directory handling
#[test]
fn test_walk_parallel_empty_directory() {
    let temp = TempDir::new().unwrap();

    let code = format!(r#"
        let entries = walk_parallel("{}")
        println("Entries: {{}} (root dir)", entries.len())
        assert(entries.len() >= 1, "Should have at least root entry")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success();
}

// Test 5: Nonexistent path (graceful handling)
#[test]
fn test_walk_parallel_nonexistent_path() {
    let code = r#"
        let entries = walk_parallel("/nonexistent/path/xyz123")
        println("Entries: {}", entries.len())
    "#;

    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success(); // Should return empty array or handle gracefully
}

// Test 6: Parallel processing (100 files)
#[test]
#[ignore] // Run with: cargo test -- --ignored
fn test_walk_parallel_performance() {
    let temp = TempDir::new().unwrap();

    // Create 100 files
    for i in 0..100 {
        fs::write(temp.path().join(format!("file{}.txt", i)), "test").unwrap();
    }

    let code = format!(r#"
        let files = walk_parallel("{}")
            .filter(fn(e) {{ e.is_file }})

        println("Processed {{}} files in parallel", files.len())
        assert(files.len() == 100, "Expected 100 files")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Processed 100 files"));
}

// Test 7: Complex filtering
#[test]
fn test_walk_parallel_complex_filter() {
    let temp = TempDir::new().unwrap();
    fs::write(temp.path().join("small.txt"), "ab").unwrap();
    fs::write(temp.path().join("large.txt"), "abcdefghij").unwrap();

    let code = format!(r#"
        let large_files = walk_parallel("{}")
            .filter(fn(e) {{ e.is_file && e.size > 5 }})

        println("Found {{}} large files", large_files.len())
        assert(large_files.len() == 1, "Only 1 file > 5 bytes")
    "#, temp.path().display());

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1 large files"));
}
