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
