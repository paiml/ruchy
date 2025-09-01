/// P0-BOOK-002: File Operations TDD Tests
/// These tests define the expected behavior for file I/O operations in transpiled code

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_basic_file_write() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    let output_file = dir.path().join("output.txt");
    
    // Basic file write operation (using supported import syntax)
    let code = format!(r#"
import std::fs::write_file
write_file("{}", "Hello, World!")
"#, output_file.to_string_lossy());
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .success();
    
    // Verify file was created with correct content
    assert!(output_file.exists());
    let content = fs::read_to_string(&output_file).unwrap();
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_basic_file_read() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    let input_file = dir.path().join("input.txt");
    
    // Create input file
    fs::write(&input_file, "Hello from file!").unwrap();
    
    // Read and print file content (using supported import syntax)
    let code = format!(r#"
import std::fs::read_file
let content = read_file("{}")
println(content)
"#, input_file.to_string_lossy());
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from file!"));
}

#[test]
fn test_file_write_and_read_chain() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    let temp_file = dir.path().join("temp.txt");
    
    // Write then read in same program (using supported import syntax)
    let code = format!(r#"
import std::fs
write_file("{}", "Chain test")
let content = read_file("{}")
println(content)
"#, temp_file.to_string_lossy(), temp_file.to_string_lossy());
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Chain test"));
}

#[test]
fn test_import_wildcard_from_std_fs() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    let output_file = dir.path().join("wildcard.txt");
    
    // Import everything from std::fs (using supported syntax)
    let code = format!(r#"
import std::fs
write_file("{}", "Wildcard import works")
let content = read_file("{}")
println(content)
"#, output_file.to_string_lossy(), output_file.to_string_lossy());
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Wildcard import works"));
}

#[test]
fn test_file_operations_without_import() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Using file operations without import should fail gracefully
    let code = r#"
let content = read_file("nonexistent.txt")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot find function `read_file`"));
}

#[test]
fn test_import_parsing_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // This should not panic the transpiler (using supported syntax)
    let code = r"
import std::fs::read_file
";
    
    fs::write(&file_path, code).unwrap();
    
    // Should not panic, even if compilation fails
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .code(predicate::in_iter(vec![0, 1])); // Success or failure, but not panic
}

#[test] 
fn test_error_handling_file_not_found() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Reading non-existent file should produce proper error
    let code = r#"
import std::fs::read_file
let content = read_file("nonexistent.txt")
"#;
    
    fs::write(&file_path, code).unwrap();
    
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["run", file_path.to_str().unwrap()])
        .assert()
        .failure();
}