//! Property tests for Issue #116 - File `open()` builtin function
//!
//! EXTREME TDD: Verify File object behavior across various inputs

use proptest::prelude::*;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::TempDir;

/// Property: `open()` successfully opens valid files
#[test]
fn prop_open_valid_files() {
    proptest!(|(line_count in 1usize..100, content in "[a-zA-Z0-9 ]{1,50}")| {
        // Create temp file with N lines
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        {
            let mut file = File::create(&file_path).expect("Failed to create file");
            for i in 0..line_count {
                writeln!(file, "{content} {i}").expect("Failed to write");
            }
        }

        let script = format!(r#"
let file = open("{}", "r")
let lines = 0
while !file.at_end() {{
    let line = file.read_line()
    lines = lines + 1
}}
file.close()
println(lines)
"#, file_path.to_str().unwrap());

        let output = Command::new("target/release/ruchy")
            .arg("-e")
            .arg(&script)
            .output()
            .expect("ruchy execution failed");

        // Property: Execution succeeds
        prop_assert!(output.status.success(), "open() failed on valid file");

        // Property: Line count matches file content
        let lines_read: usize = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .parse()
            .expect("Not an integer");

        prop_assert_eq!(
            lines_read,
            line_count,
            "Expected {} lines, read {}",
            line_count,
            lines_read
        );
    });
}

/// Property: `open()` with invalid mode returns error
#[test]
fn prop_open_invalid_mode_fails() {
    let invalid_modes = ["w", "a", "rw", "x", "invalid"];

    proptest!(|(mode_idx in 0..invalid_modes.len())| {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).expect("Failed to create file");

        let mode = invalid_modes[mode_idx];
        let script = format!(r#"
let file = open("{}", "{}")
"#, file_path.to_str().unwrap(), mode);

        let output = Command::new("target/release/ruchy")
            .arg("-e")
            .arg(&script)
            .output()
            .expect("ruchy execution failed");

        // Property: Invalid modes cause runtime error
        prop_assert!(
            !output.status.success() || String::from_utf8_lossy(&output.stderr).contains("not supported"),
            "open() should reject invalid mode '{}'",
            mode
        );
    });
}

/// Property: File methods work after `open()`
#[test]
fn prop_file_methods_functional() {
    proptest!(|(lines_content in prop::collection::vec("[a-zA-Z0-9 ]{1,30}", 1..20))| {
        // Create temp file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.txt");

        {
            let mut file = File::create(&file_path).expect("Failed to create file");
            for content in &lines_content {
                writeln!(file, "{content}").expect("Failed to write");
            }
        }

        let script = format!(r#"
let file = open("{}", "r")
let first_line = file.read_line()
println(len(first_line) > 0)
file.close()
"#, file_path.to_str().unwrap());

        let output = Command::new("target/release/ruchy")
            .arg("-e")
            .arg(&script)
            .output()
            .expect("ruchy execution failed");

        // Property: File methods return valid data
        let result = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8")
            .trim()
            .to_lowercase();

        prop_assert_eq!(result, "true", "read_line() should return non-empty string");
    });
}

/// Property: `open()` handles non-existent files gracefully
#[test]
fn prop_open_nonexistent_file() {
    proptest!(|(random_path in "[a-z]{10,20}\\.txt")| {
        let script = format!(r#"
let file = open("/tmp/nonexistent_{random_path}", "r")
"#);

        let output = Command::new("target/release/ruchy")
            .arg("-e")
            .arg(&script)
            .output()
            .expect("ruchy execution failed");

        // Property: Non-existent files cause error (not panic)
        prop_assert!(
            !output.status.success() || String::from_utf8_lossy(&output.stderr).contains("Failed to open"),
            "open() should handle non-existent files gracefully"
        );
    });
}

/// Property: Multiple `open()` calls work independently
#[test]
fn prop_multiple_open_calls_independent() {
    proptest!(|(content1 in "[a-z]{5,15}", content2 in "[A-Z]{5,15}")| {
        // Create two temp files
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");

        File::create(&file1_path)
            .and_then(|mut f| writeln!(f, "{content1}"))
            .expect("Failed to create file1");

        File::create(&file2_path)
            .and_then(|mut f| writeln!(f, "{content2}"))
            .expect("Failed to create file2");

        let script = format!(r#"
let file1 = open("{}", "r")
let file2 = open("{}", "r")
let line1 = file1.read_line()
let line2 = file2.read_line()
file1.close()
file2.close()
println(line1)
println(line2)
"#, file1_path.to_str().unwrap(), file2_path.to_str().unwrap());

        let output = Command::new("target/release/ruchy")
            .arg("-e")
            .arg(&script)
            .output()
            .expect("ruchy execution failed");

        let stdout = String::from_utf8(output.stdout)
            .expect("Invalid UTF-8");
        let lines: Vec<&str> = stdout.trim().lines().collect();

        // Property: Both files read independently
        prop_assert_eq!(lines.len(), 2, "Should read from both files");
        prop_assert!(lines[0].contains(&content1), "File1 content mismatch");
        prop_assert!(lines[1].contains(&content2), "File2 content mismatch");
    });
}
