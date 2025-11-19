#![allow(missing_docs)]
// STDLIB-005: compute_hash() Tests - MD5 File Hashing for Duplicate Detection
//
// Architecture: Simple composable primitive that computes MD5 hash of a file
// Users compose with walk_parallel() for duplicate detection:
//
//   let files = walk_parallel("/data")
//       .filter(fn(e) { e.is_file })
//       .map(fn(e) { { path: e.path, hash: compute_hash(e.path) } })
//
//   let duplicates = files.group_by(fn(e) { e.hash }).filter(fn(g) { g.len() > 1 })
//
// WHY THIS IS PERFECT:
// - Single responsibility: Just computes hash
// - Composable: Users chain with walk_parallel, filter, map, group_by
// - Fast: Uses MD5 (optimized for speed, not cryptographic security)
// - Memory efficient: Reads file once, streams through MD5

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// Test 1: Basic hash computation
#[test]
fn test_compute_hash_basic() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("test.txt");
    fs::write(&file, "hello world").unwrap();

    let code = format!(
        r#"
        let hash = compute_hash("{}")
        println("Hash: {{}}", hash)
        assert(hash != nil, "Hash should not be nil")
        assert(hash.len() == 32, "MD5 hash should be 32 hex characters")
    "#,
        file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hash: "));
}

// Test 2: Identical files produce identical hashes
#[test]
fn test_compute_hash_identical_files() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    fs::write(&file1, "identical content").unwrap();
    fs::write(&file2, "identical content").unwrap();

    let code = format!(
        r#"
        let hash1 = compute_hash("{}")
        let hash2 = compute_hash("{}")

        println("Hash1: {{}}", hash1)
        println("Hash2: {{}}", hash2)
        assert(hash1 == hash2, "Identical files should have identical hashes")
    "#,
        file1.display(),
        file2.display()
    );

    ruchy_cmd().arg("-e").arg(&code).assert().success();
}

// Test 3: Different files produce different hashes
#[test]
fn test_compute_hash_different_files() {
    let temp = TempDir::new().unwrap();
    let file1 = temp.path().join("file1.txt");
    let file2 = temp.path().join("file2.txt");
    fs::write(&file1, "content A").unwrap();
    fs::write(&file2, "content B").unwrap();

    let code = format!(
        r#"
        let hash1 = compute_hash("{}")
        let hash2 = compute_hash("{}")

        println("Hash1: {{}}", hash1)
        println("Hash2: {{}}", hash2)
        assert(hash1 != hash2, "Different files should have different hashes")
    "#,
        file1.display(),
        file2.display()
    );

    ruchy_cmd().arg("-e").arg(&code).assert().success();
}

// Test 4: Compose with walk_parallel for duplicate detection
#[test]
fn test_compute_hash_with_walk_parallel() {
    let temp = TempDir::new().unwrap();

    // Create duplicate files
    fs::write(temp.path().join("dup1.txt"), "duplicate content").unwrap();
    fs::write(temp.path().join("dup2.txt"), "duplicate content").unwrap();
    fs::write(temp.path().join("unique.txt"), "unique content").unwrap();

    let code = format!(
        r#"
        let entries = walk_parallel("{}")
            .filter(fn(e) {{ e.is_file }})

        println("Found {{}} files", entries.len())
        assert(entries.len() == 3, "Should find 3 files")

        # Compute hashes for all files
        let with_hashes = entries.map(fn(e) {{
            let h = compute_hash(e.path)
            println("File: {{}} Hash: {{}}", e.name, h)
            h
        }})

        assert(with_hashes.len() == 3, "Should have 3 hashes")
    "#,
        temp.path().display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 3 files"));
}

// Test 5: Empty file handling
#[test]
fn test_compute_hash_empty_file() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("empty.txt");
    fs::write(&file, "").unwrap();

    let code = format!(
        r#"
        let hash = compute_hash("{}")
        println("Empty file hash: {{}}", hash)
        # MD5 of empty string: d41d8cd98f00b204e9800998ecf8427e
        assert(hash == "d41d8cd98f00b204e9800998ecf8427e", "Empty file should have known MD5")
    "#,
        file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("d41d8cd98f00b204e9800998ecf8427e"));
}

// Test 6: Nonexistent file error handling
#[test]
fn test_compute_hash_nonexistent_file() {
    let code = r#"
        let hash = compute_hash("/nonexistent/file/xyz123.txt")
        println("Hash: {}", hash)
    "#;

    ruchy_cmd().arg("-e").arg(code).assert().failure(); // Should error on nonexistent file
}

// Test 7: Known MD5 hash verification (hello world)
#[test]
fn test_compute_hash_known_md5() {
    let temp = TempDir::new().unwrap();
    let file = temp.path().join("hello.txt");
    fs::write(&file, "hello world").unwrap();

    let code = format!(
        r#"
        let hash = compute_hash("{}")
        println("Hash: {{}}", hash)
        # MD5 of "hello world": 5eb63bbbe01eeed093cb22bb8f5acdc3
        assert(hash == "5eb63bbbe01eeed093cb22bb8f5acdc3", "Should match known MD5")
    "#,
        file.display()
    );

    ruchy_cmd()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("5eb63bbbe01eeed093cb22bb8f5acdc3"));
}
