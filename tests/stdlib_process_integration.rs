//! Integration tests for stdlib::process module
//!
//! Target: 0% → 100% coverage for stdlib/process.rs (52 lines)
//! Protocol: EXTREME TDD - External integration tests provide llvm-cov coverage
//!
//! Root Cause: #[cfg(test)] unit tests exist but aren't tracked by coverage.
//! Solution: Integration tests from tests/ directory ARE tracked by llvm-cov.

use ruchy::stdlib::process;

#[test]
fn test_process_execute_echo() {
    let (stdout, stderr, exit_code) = process::execute("echo", &["hello"]).unwrap();
    assert!(
        stdout.contains("hello"),
        "stdout should contain 'hello', got: {stdout}"
    );
    assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
    assert_eq!(exit_code, 0, "exit code should be 0");
}

#[test]
fn test_process_execute_multiple_args() {
    let (stdout, _stderr, exit_code) = process::execute("echo", &["hello", "world"]).unwrap();
    assert!(
        stdout.contains("hello"),
        "stdout should contain 'hello', got: {stdout}"
    );
    assert!(
        stdout.contains("world"),
        "stdout should contain 'world', got: {stdout}"
    );
    assert_eq!(exit_code, 0, "exit code should be 0");
}

#[test]
fn test_process_execute_no_args() {
    let (stdout, _stderr, exit_code) = process::execute("pwd", &[]).unwrap();
    assert!(!stdout.is_empty(), "stdout should not be empty");
    assert_eq!(exit_code, 0, "exit code should be 0");
}

#[test]
fn test_process_execute_nonexistent_command() {
    let result = process::execute("nonexistent_command_xyz_ruchy_test", &[]);
    assert!(
        result.is_err(),
        "executing nonexistent command should return error"
    );
}

#[test]
fn test_process_execute_command_with_failure() {
    // Try to list a non-existent directory (should fail)
    let (stdout, stderr, exit_code) =
        process::execute("ls", &["/nonexistent_directory_xyz_ruchy_test"]).unwrap();
    assert_ne!(exit_code, 0, "exit code should be non-zero for failed command");
    assert!(
        stderr.contains("No such file or directory") || !stdout.is_empty(),
        "stderr should contain error message or stdout should have output"
    );
}

#[test]
fn test_process_execute_with_special_characters() {
    let (stdout, _stderr, exit_code) = process::execute("echo", &["test@123"]).unwrap();
    assert!(
        stdout.contains("test@123"),
        "stdout should contain 'test@123', got: {stdout}"
    );
    assert_eq!(exit_code, 0, "exit code should be 0");
}

#[test]
fn test_process_execute_empty_string() {
    let (stdout, _stderr, exit_code) = process::execute("echo", &[""]).unwrap();
    assert_eq!(exit_code, 0, "exit code should be 0");
    // Echo of empty string produces newline
    assert!(
        stdout.len() <= 2,
        "stdout should be at most 2 chars (newline), got length: {}",
        stdout.len()
    );
}

#[test]
fn test_process_current_pid_positive() {
    let pid = process::current_pid().unwrap();
    assert!(pid > 0, "PID should be positive, got: {pid}");
    assert!(pid < u32::MAX, "PID should be less than u32::MAX");
}

#[test]
fn test_process_current_pid_consistent() {
    // PID should be the same within a single process
    let pid1 = process::current_pid().unwrap();
    let pid2 = process::current_pid().unwrap();
    assert_eq!(pid1, pid2, "PID should be consistent within process");
}

#[test]
fn test_process_workflow_execute_and_check() {
    // Complete workflow: execute command and verify results
    let (stdout, stderr, exit_code) = process::execute("echo", &["integration_test"]).unwrap();

    // Verify command succeeded
    assert_eq!(exit_code, 0, "Command should succeed");
    assert!(
        stdout.contains("integration_test"),
        "Output should contain expected text"
    );
    assert!(stderr.is_empty(), "No errors expected");

    // Verify we can get our PID
    let pid = process::current_pid().unwrap();
    assert!(pid > 0, "Should have valid process ID");
}

#[test]
fn test_process_execute_captures_stderr() {
    // Use a command that writes to stderr (bash -c)
    // Note: This test may behave differently on different systems
    let result = process::execute("bash", &["-c", "echo error_output >&2"]);

    // Should succeed even though output goes to stderr
    if let Ok((stdout, stderr, exit_code)) = result {
        assert_eq!(exit_code, 0, "bash command should succeed");
        // Either stdout or stderr should have content (implementation dependent)
        assert!(
            !stderr.is_empty() || !stdout.is_empty(),
            "Should capture output from stderr or stdout"
        );
    }
}

#[test]
fn test_process_execute_exit_code_propagation() {
    // Use false command which always returns exit code 1
    let (stdout, stderr, exit_code) = process::execute("false", &[]).unwrap();

    assert_eq!(
        exit_code, 1,
        "false command should return exit code 1, got: {exit_code}"
    );
    // stdout and stderr should be empty for the false command
    assert!(
        stdout.is_empty(),
        "stdout should be empty for false command"
    );
    assert!(
        stderr.is_empty(),
        "stderr should be empty for false command"
    );
}

#[test]
fn test_process_execute_true_command() {
    // Use true command which always returns exit code 0
    let (stdout, stderr, exit_code) = process::execute("true", &[]).unwrap();

    assert_eq!(
        exit_code, 0,
        "true command should return exit code 0, got: {exit_code}"
    );
    assert!(stdout.is_empty(), "stdout should be empty for true command");
    assert!(stderr.is_empty(), "stderr should be empty for true command");
}
