//! Issue #163: Windows line ending support (CRLF)
//!
//! Tests that files with Windows line endings (\r\n) parse correctly.

use assert_cmd::Command;
use std::fs;
use tempfile::tempdir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

#[test]
fn test_issue_163_01_crlf_hello_world() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    // Write file with Windows line endings (CRLF)
    fs::write(
        &file_path,
        "fun main() {\r\n    println!(\"Hello, World!\")\r\n}\r\n",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, World!"),
        "Should run CRLF file: stdout={}, stderr={}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_issue_163_02_crlf_function_definition() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    // Function with CRLF line endings
    fs::write(
        &file_path,
        "fun add(a: i64, b: i64) -> i64 {\r\n    a + b\r\n}\r\n\r\nfun main() {\r\n    println!(\"{}\", add(2, 3))\r\n}\r\n",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("5"),
        "Should output 5: stdout={}, stderr={}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_issue_163_03_crlf_transpile() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    let output_path = dir.path().join("output.rs");
    // CRLF file for transpilation
    fs::write(
        &file_path,
        "fun main() {\r\n    let x = 42;\r\n    println!(\"{}\", x)\r\n}\r\n",
    )
    .unwrap();

    ruchy_cmd()
        .arg("transpile")
        .arg(&file_path)
        .arg("-o")
        .arg(&output_path)
        .assert()
        .success();

    let output = fs::read_to_string(&output_path).unwrap();
    assert!(
        output.contains("fn main"),
        "Should transpile CRLF file: {}",
        output
    );
}

#[test]
fn test_issue_163_04_crlf_with_comments() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    // CRLF with comments
    fs::write(
        &file_path,
        "// Comment with CRLF\r\nfun main() {\r\n    // Another comment\r\n    println!(\"test\")\r\n}\r\n",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test"),
        "Should handle comments with CRLF: stdout={}, stderr={}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_issue_163_05_mixed_line_endings() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    // Mixed line endings (some CRLF, some LF)
    fs::write(
        &file_path,
        "fun main() {\r\n    let x = 1;\n    let y = 2;\r\n    println!(\"{}\", x + y)\n}\r\n",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("3"),
        "Should handle mixed line endings: stdout={}, stderr={}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_issue_163_06_crlf_if_expression() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.ruchy");
    // If expression with CRLF
    fs::write(
        &file_path,
        "fun main() {\r\n    let result = if true {\r\n        42\r\n    } else {\r\n        0\r\n    };\r\n    println!(\"{}\", result)\r\n}\r\n",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("run")
        .arg(&file_path)
        .output()
        .expect("Failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("42"),
        "Should handle if expression with CRLF: stdout={}, stderr={}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}
