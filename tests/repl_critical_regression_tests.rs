//! Critical REPL regression tests to prevent future command handler bypass bugs
//!
//! These tests specifically verify that piped input to REPL commands
//! is handled correctly and not bypassed to `eval()` mode.

#![allow(clippy::unwrap_used)] // Tests are allowed to unwrap

use std::io::Write;
use std::process::Command;

/// Test that REPL commands work when piped
#[test]
fn test_repl_quit_command_piped() {
    let mut output = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(b":quit\n").unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();

    // Command should succeed (not fail to parse)
    // This is the regression test for the critical bug
    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
        "REPL quit command should not cause parse error"
    );
}

/// Test that REPL arithmetic works when piped
#[test]
fn test_repl_arithmetic_piped() {
    let mut output = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(b"2 + 3\n:quit\n").unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();

    assert!(stdout.contains('5'), "Should contain arithmetic result");
    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
        "Should not have parse errors"
    );
}

/// Test that REPL help command works when piped
#[test]
fn test_repl_help_command_piped() {
    let mut output = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(b":help\n:quit\n").unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();

    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
        "Should not have parse errors"
    );
    // Help command should not cause parse errors
}

/// Test that REPL println works when piped  
#[test]
fn test_repl_println_piped() {
    let mut output = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(b"println(\"Hello\")\n:quit\n").unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();

    assert!(stdout.contains("Hello"), "Should contain println output");
    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
        "Should not have parse errors"
    );
}

/// Property test: All valid REPL commands should work when piped
#[test]
fn test_repl_all_commands_piped() {
    let commands = vec![":quit", ":q", ":help", ":h", ":history", ":clear"];

    for cmd in commands {
        let mut output = Command::new("target/release/ruchy")
            .arg("repl")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdin = output.stdin.take().unwrap();
        stdin.write_all(format!("{cmd}\n").as_bytes()).unwrap();
        if cmd != ":quit" && cmd != ":q" {
            stdin.write_all(b":quit\n").unwrap();
        }
        drop(stdin);

        let result = output.wait_with_output().unwrap();

        assert!(
            !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
            "Command '{cmd}' should not cause parse error"
        );
    }
}

/// Test that one-liner mode still works correctly (should not be affected)
#[test]
fn test_oneliner_still_works() {
    let output = Command::new("target/release/ruchy")
        .arg("-e")
        .arg("2 + 3")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.trim() == "5", "One-liner should still work");
}

/// Test that piped input to non-REPL commands still works  
#[test]
fn test_piped_input_non_repl_still_works() {
    let mut output = Command::new("target/release/ruchy")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(b"2 + 3\n").unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();

    assert!(stdout.trim() == "5", "Piped input to default should work");
}

/// Integration test: Complex REPL session should work
#[test]
fn test_repl_complex_session_piped() {
    let session = r#"
let x = 10
x + 5
println("Result: {}", x * 2)
2 + 3 * 4
:history
:quit
"#;

    let mut output = Command::new("target/release/ruchy")
        .arg("repl")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = output.stdin.take().unwrap();
    stdin.write_all(session.as_bytes()).unwrap();
    drop(stdin);

    let result = output.wait_with_output().unwrap();

    assert!(
        !String::from_utf8_lossy(&result.stderr).contains("Failed to parse input"),
        "Complex REPL session should not have parse errors"
    );
}
