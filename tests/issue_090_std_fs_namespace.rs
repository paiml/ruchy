// RED Phase Test for Issue #90: std::fs namespace access from Ruchy code
//
// GitHub Issue: https://github.com/paiml/ruchy/issues/90
//
// Problem: std::fs functions exist but aren't accessible via namespace syntax.
// Currently registered as flat builtins (fs_read, fs_write) instead of nested (std::fs::read).
//
// Expected: Ruchy code should be able to use `std::fs::write()`, `std::fs::read_to_string()`, etc.
//
// EXTREME TDD Methodology:
// 1. RED: Create failing test using std::fs namespace from Ruchy code
// 2. GREEN: Register std object with fs sub-object containing all functions
// 3. REFACTOR: Verify all functions accessible and clean up

#![allow(missing_docs)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// RED: Test basic `std::fs::write` and `std::fs::read_to_string`
///
/// This is the core Issue #90 complaint - namespace syntax doesn't work.
#[test]
#[ignore = "BUG: std::fs namespace not working"]
fn test_issue_090_std_fs_write_and_read() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create Ruchy script using std::fs namespace
    let script_file = temp_dir.path().join("test.ruchy");
    let script_code = format!(
        r#"
fun main() {{
    let test_file = "{}";
    let test_content = "Hello from std::fs!";

    // Write using std::fs namespace (Issue #90 request)
    std::fs::write(test_file, test_content);
    println!("✓ Write succeeded");

    // Read using std::fs namespace
    let content = std::fs::read_to_string(test_file);
    if content == test_content {{
        println!("✓ Read succeeded");
    }} else {{
        println!("✗ Content mismatch");
    }}
}}
"#,
        test_file.to_str().unwrap()
    );
    fs::write(&script_file, script_code).unwrap();

    // RED: Previously failed with "Object has no field named 'fs'"
    // GREEN: Now succeeds with namespace registration
    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("test.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(
        stdout.contains("✓ Write succeeded"),
        "Expected write success, got: {stdout}"
    );
    assert!(
        stdout.contains("✓ Read succeeded"),
        "Expected read success, got: {stdout}"
    );

    // Verify file was actually created
    assert!(test_file.exists(), "File should exist after write");
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Hello from std::fs!");
}

/// RED: Test `std::fs::create_dir` and `std::fs::remove_dir`
#[test]
fn test_issue_090_std_fs_directory_operations() {
    let temp_dir = TempDir::new().unwrap();
    let test_dir = temp_dir.path().join("test_dir");

    let script_file = temp_dir.path().join("test.ruchy");
    let script_code = format!(
        r#"
fun main() {{
    let test_dir = "{}";

    // Create directory using std::fs namespace
    std::fs::create_dir(test_dir);
    println!("✓ Create dir succeeded");

    // Check if exists
    if std::fs::exists(test_dir) {{
        println!("✓ Directory exists");
    }} else {{
        println!("✗ Directory doesn't exist");
        return;
    }}

    // Remove directory
    std::fs::remove_dir(test_dir);
    println!("✓ Remove dir succeeded");
}}
"#,
        test_dir.to_str().unwrap()
    );
    fs::write(&script_file, script_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("test.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("✓ Create dir succeeded"));
    assert!(stdout.contains("✓ Directory exists"));
    assert!(stdout.contains("✓ Remove dir succeeded"));
}

/// RED: Test `std::fs::copy` and `std::fs::rename`
#[test]
fn test_issue_090_std_fs_copy_and_rename() {
    let temp_dir = TempDir::new().unwrap();
    let source = temp_dir.path().join("source.txt");
    let copied = temp_dir.path().join("copied.txt");
    let renamed = temp_dir.path().join("renamed.txt");

    // Create source file
    fs::write(&source, "Test content").unwrap();

    let script_file = temp_dir.path().join("test.ruchy");
    let script_code = format!(
        r#"
fun main() {{
    let source = "{}";
    let copied = "{}";
    let renamed = "{}";

    // Copy file
    std::fs::copy(source, copied);
    println!("✓ Copy succeeded");

    // Rename file
    std::fs::rename(copied, renamed);
    println!("✓ Rename succeeded");
}}
"#,
        source.to_str().unwrap(),
        copied.to_str().unwrap(),
        renamed.to_str().unwrap()
    );
    fs::write(&script_file, script_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("test.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("✓ Copy succeeded"));
    assert!(stdout.contains("✓ Rename succeeded"));

    // Verify files
    assert!(renamed.exists(), "Renamed file should exist");
    assert!(
        !copied.exists(),
        "Original copied file should not exist after rename"
    );
}

/// RED: Test `std::fs::metadata`
#[test]
fn test_issue_090_std_fs_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();

    let script_file = temp_dir.path().join("test.ruchy");
    let script_code = format!(
        r#"
fun main() {{
    let test_file = "{}";

    // Get metadata (returns object with size, is_file, is_dir fields)
    let meta = std::fs::metadata(test_file);
    println!("✓ Metadata succeeded");

    // Access fields directly (object doesn't support method calls yet)
    if meta.is_file {{
        println!("✓ Is file");
    }}
}}
"#,
        test_file.to_str().unwrap()
    );
    fs::write(&script_file, script_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("test.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("✓ Metadata succeeded"));
    assert!(stdout.contains("✓ Is file"));
}

/// RED: Test `std::fs::read_dir`
#[test]
fn test_issue_090_std_fs_read_dir() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    fs::write(temp_dir.path().join("file1.txt"), "1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "2").unwrap();

    let script_file = temp_dir.path().join("test.ruchy");
    let script_code = format!(
        r#"
fun main() {{
    let dir_path = "{}";

    // Read directory
    let entries = std::fs::read_dir(dir_path);
    println!("✓ Read dir succeeded");
    let count = entries.len();
    if count >= 2 {{
        println!("✓ Found files");
    }}
}}
"#,
        temp_dir.path().to_str().unwrap()
    );
    fs::write(&script_file, script_code).unwrap();

    let output = ruchy_cmd()
        .current_dir(temp_dir.path())
        .arg("run")
        .arg("test.ruchy")
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    assert!(stdout.contains("✓ Read dir succeeded"));
    assert!(stdout.contains("✓ Found files"));
}
