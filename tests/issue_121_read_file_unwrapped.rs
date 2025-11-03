// Issue #121: read_file() should return unwrapped string (not Result enum)
// EXTREME TDD Protocol: RED → GREEN → REFACTOR → VALIDATE
// Blocking: BENCH-006 (file-processing), BENCH-009 (json-parsing)
//
// ROOT CAUSE: eval_fs_read() returns Result::Ok(string) but benchmarks expect plain string
// FIX: Create read_file() alias that unwraps Result automatically

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_issue_121_read_file_returns_string() {
    // RED: This test WILL FAIL until read_file() returns unwrapped string
    let mut file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "Hello from file").unwrap();

    let code = format!(
        r#"
        let contents = read_file("{}")
        println(contents)
        "#,
        file.path().display()
    );

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello from file"));
}

#[test]
fn test_issue_121_read_file_with_json_parsing() {
    // RED: Benchmark pattern - read file then parse JSON
    let mut file = NamedTempFile::new().unwrap();
    fs::write(file.path(), r#"{"name": "Alice", "age": 30}"#).unwrap();

    let code = format!(
        r#"
        let contents = read_file("{}")
        let data = parse_json(contents)
        println(data["name"])
        "#,
        file.path().display()
    );

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));
}

#[test]
fn test_issue_121_read_file_string_operations() {
    // RED: Verify string methods work on result
    let mut file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "The quick brown fox").unwrap();

    let code = format!(
        r#"
        let contents = read_file("{}")
        println(len(contents))
        "#,
        file.path().display()
    );

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("19"));
}

#[test]
fn test_issue_121_read_file_multiline() {
    // RED: Test multiline file reading
    let mut file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "line 1\nline 2\nline 3").unwrap();

    let code = format!(
        r#"
        let contents = read_file("{}")
        println(contents)
        "#,
        file.path().display()
    );

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("line 1"))
        .stdout(predicate::str::contains("line 2"))
        .stdout(predicate::str::contains("line 3"));
}

#[test]
fn test_issue_121_bench_006_pattern() {
    // RED: BENCH-006 file processing pattern
    let mut file = NamedTempFile::new().unwrap();
    fs::write(file.path(), "apple\nbanana\ncherry\napricot").unwrap();

    let code = format!(
        r#"
        let contents = read_file("{}")
        let lines = contents.split("\n")
        let mut count = 0
        let mut i = 0
        while i < len(lines) {{
            let line = lines[i]
            if line[0] == 'a' {{
                count = count + 1
            }}
            i = i + 1
        }}
        println(count)
        "#,
        file.path().display()
    );

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(&code)
        .assert()
        .success()
        .stdout(predicate::str::contains("2")); // apple, apricot
}

#[test]
fn test_issue_121_fs_read_still_returns_result() {
    // ENSURE: fs_read() still returns Result (for error handling use cases)
    let code = r#"
        let result = fs_read("/nonexistent/file.txt")
        match result {
            Ok(contents) => println("Got contents"),
            Err(msg) => println("Got error")
        }
    "#;

    Command::cargo_bin("ruchy")
        .unwrap()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("Got error"));
}
