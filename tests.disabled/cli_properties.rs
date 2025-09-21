//! Property tests for CLI commands
//! Toyota Way: Mathematical invariants that must always hold

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Property: fmt is idempotent - format(format(x)) == format(x)
#[test]
fn prop_fmt_idempotent() {
    let test_cases = [
        "fun test() -> i32 { 42 }",
        "fun add(x: i32, y: i32) -> i32 { x + y }",
        "fun cond(x: i32) -> i32 { if x > 0 { x } else { 0 } }",
        "fun complex(a: i32, b: i32) -> i32 { if a > b { a * 2 } else { b * 2 } }",
    ];

    for code in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");

        // Write initial code
        fs::write(&test_file, code).unwrap();

        // Format once
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();

        let first_format = fs::read_to_string(&test_file).unwrap();

        // Format again
        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();

        let second_format = fs::read_to_string(&test_file).unwrap();

        // Property: format(format(x)) == format(x)
        assert_eq!(
            first_format, second_format,
            "Idempotency failed for: {code}"
        );
    }
}

/// Property: fmt preserves function names
#[test]
fn prop_fmt_preserves_function_names() {
    let test_cases = [
        ("test", "fun test() -> i32 { 42 }"),
        ("add", "fun add(x: i32, y: i32) -> i32 { x + y }"),
        ("multiply", "fun multiply(a: i32) -> i32 { a * 2 }"),
    ];

    for (name, code) in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");

        fs::write(&test_file, code).unwrap();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();

        let formatted = fs::read_to_string(&test_file).unwrap();

        // Property: function name is preserved
        assert!(
            formatted.contains(&format!("fun {name}")),
            "Function name not preserved in: {formatted}"
        );
    }
}

/// Property: fmt preserves semantic structure (operators)
#[test]
fn prop_fmt_preserves_operators() {
    let test_cases = [
        ("fun test() -> i32 { 1 + 2 }", "+"),
        ("fun test() -> i32 { 5 - 3 }", "-"),
        ("fun test() -> i32 { 4 * 6 }", "*"),
        ("fun test() -> bool { 10 > 5 }", ">"),
        ("fun test() -> bool { 3 == 3 }", "=="),
    ];

    for (code, op) in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");

        fs::write(&test_file, code).unwrap();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();

        let formatted = fs::read_to_string(&test_file).unwrap();

        // Property: operator is preserved
        assert!(
            formatted.contains(op),
            "Operator {op} not preserved in: {formatted}"
        );
    }
}

/// Property: fmt is deterministic - same input produces same output
#[test]
fn prop_fmt_deterministic() {
    let test_cases = [
        "fun test() -> i32 { 42 }",
        "fun add(x: i32, y: i32) -> i32 { x + y }",
        "fun cond(x: i32) -> i32 { if x > 0 { x } else { 0 } }",
    ];

    for code in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let test_file1 = temp_dir.path().join("test1.ruchy");
        let test_file2 = temp_dir.path().join("test2.ruchy");

        // Write same code to two files
        fs::write(&test_file1, code).unwrap();
        fs::write(&test_file2, code).unwrap();

        // Format both
        let mut cmd1 = Command::cargo_bin("ruchy").unwrap();
        cmd1.args(["fmt", test_file1.to_str().unwrap()])
            .assert()
            .success();

        let mut cmd2 = Command::cargo_bin("ruchy").unwrap();
        cmd2.args(["fmt", test_file2.to_str().unwrap()])
            .assert()
            .success();

        let format1 = fs::read_to_string(&test_file1).unwrap();
        let format2 = fs::read_to_string(&test_file2).unwrap();

        // Property: same input produces identical output
        assert_eq!(format1, format2, "Determinism failed for: {code}");
    }
}

/// Property: fmt preserves if-else structure
#[test]
fn prop_fmt_preserves_control_flow() {
    let test_cases = [
        "fun test(x: i32) -> i32 { if x > 0 { 1 } else { 0 } }",
        "fun nested(x: i32) -> i32 { if x > 10 { if x > 20 { 2 } else { 1 } } else { 0 } }",
    ];

    for code in test_cases {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");

        fs::write(&test_file, code).unwrap();

        let mut cmd = Command::cargo_bin("ruchy").unwrap();
        cmd.args(["fmt", test_file.to_str().unwrap()])
            .assert()
            .success();

        let formatted = fs::read_to_string(&test_file).unwrap();

        // Property: control flow keywords are preserved
        assert!(formatted.contains("if"), "if keyword not preserved");
        assert!(formatted.contains("else"), "else keyword not preserved");
    }
}
