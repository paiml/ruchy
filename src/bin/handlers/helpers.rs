//! Common Helper Functions for CLI Handlers
//!
//! Shared utility functions used across multiple command handlers.
//! All functions have complexity ≤5 (Toyota Way: <10).

use anyhow::{Context, Result};
use std::path::Path;

/// Check if a result should be printed (filters out Unit values)
/// Complexity: 2 (Toyota Way: <10)
pub fn should_print_result(result: &str) -> bool {
    result != "Unit" && result != "()"
}

/// Read file contents with detailed error context
/// Complexity: 2 (Toyota Way: <10)
pub fn read_file_with_context(file: &Path) -> Result<String> {
    std::fs::read_to_string(file).map_err(|e| {
        // Include the OS error message (e.g., "No such file or directory")
        anyhow::anyhow!("{}: {}", file.display(), e)
    })
}

/// Create a REPL instance with temp directory
/// Complexity: 1 (Toyota Way: <10)
pub fn create_repl() -> Result<ruchy::runtime::Repl> {
    ruchy::runtime::Repl::new(std::env::temp_dir())
}

/// Log command output if verbose mode is enabled
/// Complexity: 2 (Toyota Way: <10)
pub fn log_command_output(output: &std::process::Output, verbose: bool) {
    if verbose {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command output:\n{}", stderr);
    }
}

/// Write file with detailed error context
/// Complexity: 2 (Toyota Way: <10)
pub fn write_file_with_context(path: &Path, content: &[u8]) -> Result<()> {
    std::fs::write(path, content)
        .with_context(|| format!("Failed to write file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_print_result_filters_unit() {
        assert!(!should_print_result("Unit"));
        assert!(!should_print_result("()"));
        assert!(should_print_result("42"));
        assert!(should_print_result("hello"));
    }

    #[test]
    fn test_read_file_with_context_error_message() {
        let result = read_file_with_context(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("/nonexistent/file.txt"));
    }
}
