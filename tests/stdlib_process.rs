//! EXTREME TDD Coverage Tests for stdlib::process Module
//!
//! Target: 0% → 80% coverage (+11 lines)
//! Protocol: RED → GREEN → REFACTOR → VALIDATE
//! Quality: Property tests + mutation tests ≥75%

use ruchy::stdlib::process;

// ============================================================================
// UNIT TESTS (Basic Function Coverage)
// ============================================================================

#[test]
fn test_execute_echo_success() {
    // Execute simple echo command
    let result = process::execute("echo", &["hello", "world"]);

    // Verify
    assert!(result.is_ok());
    let (stdout, stderr, exit_code) = result.unwrap();

    assert!(stdout.contains("hello"));
    assert!(stdout.contains("world"));
    assert!(stderr.is_empty() || stderr.trim().is_empty());
    assert_eq!(exit_code, 0);
}

#[test]
fn test_execute_no_args() {
    // Execute command without arguments
    let result = process::execute("pwd", &[]);

    // Verify
    assert!(result.is_ok());
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(!stdout.is_empty());
    assert_eq!(exit_code, 0);
}

#[test]
fn test_execute_stderr() {
    // Execute command that writes to stderr (ls nonexistent file)
    let result = process::execute("ls", &["/nonexistent/file/xyz123"]);

    // Verify
    assert!(result.is_ok());
    let (stdout, stderr, exit_code) = result.unwrap();

    assert!(stdout.is_empty() || stdout.trim().is_empty());
    assert!(!stderr.is_empty()); // Should have error message
    assert_ne!(exit_code, 0); // Non-zero exit code
}

#[test]
fn test_execute_exit_code_nonzero() {
    // Execute false command (always exits with 1)
    let result = process::execute("false", &[]);

    // Verify
    assert!(result.is_ok());
    let (_stdout, _stderr, exit_code) = result.unwrap();

    assert_eq!(exit_code, 1);
}

#[test]
fn test_execute_invalid_command() {
    // Execute non-existent command
    let result = process::execute("nonexistent_command_xyz_999", &[]);

    // Verify error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("No such file") ||
            err.contains("not found") ||
            err.contains("NotFound"));
}

#[test]
fn test_current_pid_positive() {
    // Execute
    let result = process::current_pid();

    // Verify
    assert!(result.is_ok());
    let pid = result.unwrap();
    assert!(pid > 0);
}

#[test]
fn test_current_pid_consistent() {
    // Execute twice
    let pid1 = process::current_pid().unwrap();
    let pid2 = process::current_pid().unwrap();

    // Verify: Same process should have same PID
    assert_eq!(pid1, pid2);
}

// ============================================================================
// PROPERTY-BASED TESTS (High Coverage per Test)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn property_echo_roundtrip(
        input in "[a-zA-Z0-9]+[ a-zA-Z0-9]*" // At least one non-whitespace
    ) {
        // Property: echo should return what we give it
        let result = process::execute("echo", &[&input]).unwrap();
        let (stdout, _stderr, exit_code) = result;

        prop_assert!(stdout.contains(&input.trim()));
        prop_assert_eq!(exit_code, 0);
    }

    #[test]
    fn property_current_pid_always_positive(
        _dummy in 0..100i32
    ) {
        // Property: PID is always > 0
        let pid = process::current_pid().unwrap();
        prop_assert!(pid > 0);
    }

    #[test]
    fn property_execute_args_order(
        arg1 in "[a-z]{3,10}",
        arg2 in "[a-z]{3,10}",
        arg3 in "[a-z]{3,10}"
    ) {
        // Property: Arguments appear in order
        let result = process::execute("echo", &[&arg1, &arg2, &arg3]).unwrap();
        let (stdout, _, _) = result;

        let stdout_clean = stdout.trim();
        let pos1 = stdout_clean.find(&arg1);
        let pos2 = stdout_clean.find(&arg2);
        let pos3 = stdout_clean.find(&arg3);

        prop_assert!(pos1.is_some());
        prop_assert!(pos2.is_some());
        prop_assert!(pos3.is_some());

        // Verify order
        prop_assert!(pos1.unwrap() < pos2.unwrap());
        prop_assert!(pos2.unwrap() < pos3.unwrap());
    }
}

// ============================================================================
// EDGE CASES & ERROR HANDLING
// ============================================================================

#[test]
fn test_execute_empty_output() {
    // Execute command with no output (true command)
    let result = process::execute("true", &[]);

    // Verify
    assert!(result.is_ok());
    let (stdout, stderr, exit_code) = result.unwrap();

    assert!(stdout.is_empty() || stdout.trim().is_empty());
    assert!(stderr.is_empty() || stderr.trim().is_empty());
    assert_eq!(exit_code, 0);
}

#[test]
fn test_execute_unicode_args() {
    // Execute with Unicode arguments
    let result = process::execute("echo", &["Hello", "世界", "🌍"]);

    // Verify
    assert!(result.is_ok());
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(stdout.contains("Hello"));
    assert!(stdout.contains("世界"));
    assert!(stdout.contains("🌍"));
    assert_eq!(exit_code, 0);
}

#[test]
fn test_execute_special_chars() {
    // Execute with special characters (properly escaped)
    let result = process::execute("echo", &["test!@#$%"]);

    // Verify
    assert!(result.is_ok());
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(stdout.contains("test"));
    assert_eq!(exit_code, 0);
}

#[test]
fn test_execute_long_output() {
    // Execute command with long output (ls -lR /usr/bin)
    // Using a safer command that won't hang
    let result = process::execute("echo", &["x".repeat(1000).as_str()]);

    // Verify
    assert!(result.is_ok());
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(stdout.len() >= 1000);
    assert_eq!(exit_code, 0);
}

// ============================================================================
// INTEGRATION TESTS (Multiple Functions Together)
// ============================================================================

#[test]
fn test_process_workflow() {
    // Step 1: Get current PID
    let pid = process::current_pid().unwrap();
    assert!(pid > 0);

    // Step 2: Execute command
    let (stdout, _stderr, exit_code) = process::execute("echo", &["workflow"]).unwrap();
    assert!(stdout.contains("workflow"));
    assert_eq!(exit_code, 0);

    // Step 3: PID should still be same
    let pid2 = process::current_pid().unwrap();
    assert_eq!(pid, pid2);
}

#[test]
fn test_multiple_executions() {
    // Execute same command multiple times
    for i in 0..5 {
        let arg = format!("test{}", i);
        let result = process::execute("echo", &[&arg]);

        assert!(result.is_ok());
        let (stdout, _stderr, exit_code) = result.unwrap();
        assert!(stdout.contains(&arg));
        assert_eq!(exit_code, 0);
    }
}
