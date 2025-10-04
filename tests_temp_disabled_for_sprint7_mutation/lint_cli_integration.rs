/// CLI integration tests for lint command
/// Tests all flags and options work correctly
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_lint_basic_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(&file_path, "fn test() { let x = 1; }").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("unused variable: x"));
}

#[test]
fn test_lint_json_format() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(&file_path, "fn test() { let unused = 1; }").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap(), "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"issues\""))
        .stdout(predicate::str::contains("\"type\""))
        .stdout(predicate::str::contains("\"name\""));
}

#[test]
fn test_lint_strict_mode() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Complex function that should trigger complexity warning
    let complex_code = r"
fn complex() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        let x = 1;
                    }
                }
            }
        }
    }
}
";

    fs::write(&file_path, complex_code).unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap(), "--strict"])
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_lint_rules_filter() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(
        &file_path,
        r"
fn test() {
    let unused = 1;
    println(undefined);
}
",
    )
    .unwrap();

    // Only check for undefined variables
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap(), "--rules", "undefined"])
        .assert()
        .success()
        .stdout(predicate::str::contains("undefined variable"))
        .stdout(predicate::str::contains("unused variable").not());
}

#[test]
fn test_lint_verbose_output() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(&file_path, "fn test() { let x = 1; }").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap(), "--verbose"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Suggestion"));
}

#[test]
fn test_lint_clean_file() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    // Clean code with no issues
    fs::write(&file_path, "fn test() { let x = 1; println(x); }").unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("No issues found"));
}

#[test]
fn test_lint_multiple_issues() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(
        &file_path,
        r"
fn test() {
    let unused1 = 1;
    let unused2 = 2;
    let x = 3;
    let x = 4;  // Shadowing
    println(x);
}
",
    )
    .unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 3 issues"));
}

#[test]
fn test_lint_errors_and_warnings() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");

    fs::write(
        &file_path,
        r"
fn test() {
    let unused = 1;  // Warning
    println(undefined);  // Error
}
",
    )
    .unwrap();

    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", file_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 Error"))
        .stdout(predicate::str::contains("1 Warning"));
}

#[test]
fn test_lint_help() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--strict"))
        .stdout(predicate::str::contains("--rules"));
}

#[test]
fn test_lint_nonexistent_file() {
    Command::cargo_bin("ruchy")
        .unwrap()
        .args(["lint", "/nonexistent/file.ruchy"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to read file"));
}
