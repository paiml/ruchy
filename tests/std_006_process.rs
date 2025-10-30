//! STD-006: Process Module Tests (ruchy/std/process)
//!
//! Test suite for process operations module.
//! Thin wrappers around Rust's `std::process` with Ruchy-friendly API.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

#[test]
fn test_std_006_execute_success() {
    // STD-006: Test executing command successfully

    // Call ruchy::stdlib::process::execute
    let result = ruchy::stdlib::process::execute("echo", &["hello", "world"]);

    assert!(result.is_ok(), "execute should succeed");
    let (stdout, stderr, exit_code) = result.unwrap();

    assert!(!stdout.is_empty(), "stdout must not be empty");
    assert!(stdout.contains("hello"), "stdout must contain 'hello'");
    assert!(stdout.contains("world"), "stdout must contain 'world'");
    assert_eq!(exit_code, 0, "exit code must be 0 for success");
    assert!(
        stderr.is_empty() || stderr.trim().is_empty(),
        "stderr should be empty for success"
    );
}

#[test]
fn test_std_006_execute_with_output() {
    // STD-006: Test capturing command output

    let result = ruchy::stdlib::process::execute("echo", &["test output"]);

    assert!(result.is_ok(), "execute should succeed");
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(stdout.contains("test"), "stdout must contain 'test'");
    assert!(stdout.contains("output"), "stdout must contain 'output'");
    assert_eq!(exit_code, 0, "exit code must be 0");
    assert!(stdout.len() > 5, "stdout length must be > 5");
}

#[test]
fn test_std_006_execute_nonzero_exit() {
    // STD-006: Test command with non-zero exit code

    // Use 'false' command which always exits with 1
    let result = ruchy::stdlib::process::execute("false", &[]);

    assert!(
        result.is_ok(),
        "execute should succeed even with non-zero exit"
    );
    let (_stdout, _stderr, exit_code) = result.unwrap();

    assert_ne!(
        exit_code, 0,
        "exit code must be non-zero for 'false' command"
    );
    assert_eq!(exit_code, 1, "exit code must be 1 for 'false' command");
}

#[test]
fn test_std_006_execute_with_multiple_args() {
    // STD-006: Test executing command with multiple arguments

    let result = ruchy::stdlib::process::execute("echo", &["one", "two", "three"]);

    assert!(result.is_ok(), "execute should succeed");
    let (stdout, _stderr, exit_code) = result.unwrap();

    assert!(stdout.contains("one"), "stdout must contain 'one'");
    assert!(stdout.contains("two"), "stdout must contain 'two'");
    assert!(stdout.contains("three"), "stdout must contain 'three'");
    assert_eq!(exit_code, 0, "exit code must be 0");
}

#[test]
fn test_std_006_execute_command_not_found() {
    // STD-006: Test executing non-existent command returns error

    let result = ruchy::stdlib::process::execute("nonexistent_command_xyz", &[]);

    assert!(
        result.is_err(),
        "execute should fail for non-existent command"
    );
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "error message must not be empty");
}

#[test]
fn test_std_006_execute_empty_args() {
    // STD-006: Test executing command with no arguments

    let result = ruchy::stdlib::process::execute("echo", &[]);

    assert!(result.is_ok(), "execute should succeed with empty args");
    let (stdout, _stderr, exit_code) = result.unwrap();

    // echo with no args prints newline
    assert!(!stdout.is_empty() || exit_code == 0, "Should succeed");
    assert_eq!(exit_code, 0, "exit code must be 0");
}

#[test]
fn test_std_006_execute_stderr_output() {
    // STD-006: Test capturing stderr output

    // Use 'ls' on non-existent file to generate stderr
    let result = ruchy::stdlib::process::execute("ls", &["/nonexistent/path/xyz"]);

    assert!(result.is_ok(), "execute should succeed even with stderr");
    let (_stdout, stderr, exit_code) = result.unwrap();

    assert!(!stderr.is_empty(), "stderr must not be empty for error");
    assert_ne!(exit_code, 0, "exit code must be non-zero for error");
}

#[test]
fn test_std_006_current_pid() {
    // STD-006: Test getting current process ID

    // Call ruchy::stdlib::process::current_pid
    let result = ruchy::stdlib::process::current_pid();

    assert!(result.is_ok(), "current_pid should succeed");
    let pid = result.unwrap();

    // PIDs must be positive and less than system pid_max (typically 4M on Linux)
    assert!(pid > 0, "PID must be positive");
    assert!(
        pid < 5000000,
        "PID must be less than reasonable system maximum"
    );

    // Call again should return same PID
    let pid2 = ruchy::stdlib::process::current_pid().unwrap();
    assert_eq!(pid, pid2, "PID should be consistent");
}

#[test]
fn test_std_006_execute_captures_output_completely() {
    // STD-006: Test that execute captures all output

    let result = ruchy::stdlib::process::execute("echo", &["line1\nline2\nline3"]);

    assert!(result.is_ok(), "execute should succeed");
    let (stdout, _stderr, exit_code) = result.unwrap();

    // Should capture all lines
    assert!(stdout.contains("line1"), "Must contain line1");
    assert!(stdout.contains("line2"), "Must contain line2");
    assert!(stdout.contains("line3"), "Must contain line3");
    assert_eq!(exit_code, 0, "exit code must be 0");
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_006_execute_never_panics(arg in "[a-zA-Z0-9]{0,20}") {
            // Property: execute should never panic, even with random args

            let _ = ruchy::stdlib::process::execute("echo", &[&arg]);
            // Should not panic
        }

        #[test]
        fn test_std_006_echo_roundtrip(text in "[a-zA-Z0-9 ]{1,50}") {
            // Property: Echoing text should return that text in stdout

            let result = ruchy::stdlib::process::execute("echo", &[&text]);

            if let Ok((stdout, _stderr, exit_code)) = result {
                assert_eq!(exit_code, 0, "echo should always succeed");
                assert!(stdout.contains(&text), "stdout must contain echoed text");
            }
        }

        #[test]
        fn test_std_006_exit_code_consistency(n in 0u8..3) {
            // Property: exit code should be consistent across runs

            // Run 'true' n times, should always get exit code 0
            for _ in 0..n {
                let result = ruchy::stdlib::process::execute("true", &[]);
                if let Ok((_stdout, _stderr, exit_code)) = result {
                    assert_eq!(exit_code, 0, "true should always exit 0");
                }
            }
        }
    }
}
