// ISSUE-116: File object methods (.read_line(), .close())
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// GitHub Issue: https://github.com/paiml/ruchy/issues/116

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// RED PHASE: These tests WILL FAIL until file methods are implemented
/// Current Error: File objects don't have .`read_line()` or .`close()` methods
/// Expected: Support `file.read_line()` and `file.close()` like Python/Ruby

// ============================================================================
// TEST GROUP 1: read_line() Method
// ============================================================================

#[test]
fn test_issue_116_read_line_basic() {
    // Create test file
    fs::write("/tmp/test_read_line.txt", "line1\nline2\nline3\n").unwrap();

    let code = r#"
        let file = File.open("/tmp/test_read_line.txt");
        let line = file.read_line();
        println(line);
        file.close();
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("line1"));
}

#[test]
fn test_issue_116_read_line_multiple() {
    // Create test file
    fs::write("/tmp/test_read_multiple.txt", "first\nsecond\nthird\n").unwrap();

    let code = r#"
        let file = File.open("/tmp/test_read_multiple.txt");
        let line1 = file.read_line();
        let line2 = file.read_line();
        println(line1);
        println(line2);
        file.close();
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("first\nsecond"));
}

#[test]
fn test_issue_116_read_line_eof() {
    // Create test file with one line
    fs::write("/tmp/test_eof.txt", "only_line\n").unwrap();

    let code = r#"
        let file = File.open("/tmp/test_eof.txt");
        let line1 = file.read_line();
        let line2 = file.read_line();  // Should return None or empty
        println(line1);
        file.close();
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("only_line"));
}

// ============================================================================
// TEST GROUP 2: close() Method
// ============================================================================

#[test]
fn test_issue_116_close_explicit() {
    fs::write("/tmp/test_close.txt", "data\n").unwrap();

    let code = r#"
        let file = File.open("/tmp/test_close.txt");
        let content = file.read();
        file.close();  // Explicit close
        println(content);
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("data"));
}

#[test]
fn test_issue_116_read_after_close_error() {
    fs::write("/tmp/test_read_after_close.txt", "data\n").unwrap();

    let code = r#"
        let file = File.open("/tmp/test_read_after_close.txt");
        file.close();
        let content = file.read_line();  // Should error or return None
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("-e")
        .arg(code)
        .assert()
        .failure();  // Should fail or handle gracefully
}

// ============================================================================
// TEST GROUP 3: Transpile Validation
// ============================================================================

#[test]
fn test_issue_116_transpile_file_methods() {
    let code = r#"
        let file = File.open("test.txt");
        let line = file.read_line();
        file.close();
    "#;

    let mut cmd = Command::cargo_bin("ruchy").unwrap();
    cmd.arg("transpile")
        .arg("-")
        .write_stdin(code.to_string())
        .assert()
        .success()
        .stdout(predicate::str::contains("read_line").and(predicate::str::contains("close")));
}

// ============================================================================
// PROPERTY TEST: read_line() reads full lines
// ============================================================================

#[test]
#[ignore] // Run with: cargo test --test issue_116_file_methods -- --ignored
fn property_read_line_preserves_content() {
    use proptest::prelude::*;

    proptest!(|(lines in prop::collection::vec(".*", 1..10))| {
        let content = lines.join("\n") + "\n";
        fs::write("/tmp/test_property.txt", &content).unwrap();

        let code = r#"
            let file = File.open("/tmp/test_property.txt");
            let mut count = 0;
            while true {
                let line = file.read_line();
                if line == "" { break; }
                count = count + 1;
            }
            file.close();
            println(count);
        "#.to_string();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        let output = cmd.arg("-e").arg(&code).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should read same number of lines as written
        assert!(stdout.contains(&lines.len().to_string()));
    });
}
