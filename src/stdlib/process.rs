//! Process Operations Module (ruchy/std/process)
//!
//! Thin wrappers around Rust's `std::process` for command execution.
//!
//! **Design**: Thin wrappers (complexity ≤2 per function) around proven Rust `std::process`.
//! **Quality**: 100% unit test coverage, property tests, ≥75% mutation coverage.

use std::process::Command;

/// Execute command and capture output
///
/// Returns tuple of (stdout, stderr, `exit_code`)
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::process;
///
/// let (stdout, stderr, exit_code) = process::execute("echo", &["hello"]).unwrap();
/// assert!(stdout.contains("hello"));
/// assert_eq!(exit_code, 0);
/// ```
///
/// # Errors
///
/// Returns error if command fails to spawn
pub fn execute(command: &str, args: &[&str]) -> Result<(String, String, i32), String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    Ok((stdout, stderr, exit_code))
}

/// Get current process ID
///
/// # Examples
///
/// ```
/// use ruchy::stdlib::process;
///
/// let pid = process::current_pid().unwrap();
/// assert!(pid > 0);
/// ```
pub fn current_pid() -> Result<u32, String> {
    Ok(std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_echo() {
        let (stdout, stderr, exit_code) = execute("echo", &["hello"]).unwrap();
        assert!(stdout.contains("hello"));
        assert!(stderr.is_empty());
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_execute_multiple_args() {
        let (stdout, _stderr, exit_code) = execute("echo", &["hello", "world"]).unwrap();
        assert!(stdout.contains("hello"));
        assert!(stdout.contains("world"));
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_execute_no_args() {
        let (stdout, _stderr, exit_code) = execute("pwd", &[]).unwrap();
        assert!(!stdout.is_empty());
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_execute_nonexistent_command() {
        let result = execute("nonexistent_command_xyz", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command_with_failure() {
        // Try to list a non-existent directory (should fail)
        let (stdout, stderr, exit_code) = execute("ls", &["/nonexistent_directory_xyz"]).unwrap();
        assert_ne!(exit_code, 0); // Should have non-zero exit code
        assert!(stderr.contains("No such file or directory") || !stdout.is_empty());
    }

    #[test]
    fn test_current_pid() {
        let pid = current_pid().unwrap();
        assert!(pid > 0);
        assert!(pid < u32::MAX); // Sanity check
    }

    #[test]
    fn test_current_pid_consistent() {
        // PID should be the same within a single process
        let pid1 = current_pid().unwrap();
        let pid2 = current_pid().unwrap();
        assert_eq!(pid1, pid2);
    }

    #[test]
    fn test_execute_with_special_characters() {
        let (stdout, _stderr, exit_code) = execute("echo", &["test@123"]).unwrap();
        assert!(stdout.contains("test@123"));
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn test_execute_empty_string() {
        let (stdout, _stderr, exit_code) = execute("echo", &[""]).unwrap();
        assert_eq!(exit_code, 0);
        // Echo of empty string produces newline
        assert!(stdout.len() <= 2);
    }
}
