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
    use tempfile::{NamedTempFile, TempDir};

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

    // ===== EXTREME TDD Round 148 - Helpers Tests =====

    #[test]
    fn test_should_print_result_empty_string() {
        assert!(should_print_result(""));
    }

    #[test]
    fn test_should_print_result_whitespace() {
        assert!(should_print_result(" "));
        assert!(should_print_result("\n"));
        assert!(should_print_result("\t"));
    }

    #[test]
    fn test_should_print_result_various_values() {
        assert!(should_print_result("true"));
        assert!(should_print_result("false"));
        assert!(should_print_result("0"));
        assert!(should_print_result("-1"));
        assert!(should_print_result("3.14159"));
        assert!(should_print_result("[1, 2, 3]"));
        assert!(should_print_result("{\"key\": \"value\"}"));
    }

    #[test]
    fn test_should_print_result_unit_variations() {
        // Only exact matches should be filtered
        assert!(should_print_result("unit"));
        assert!(should_print_result("UNIT"));
        assert!(should_print_result("( )"));
        assert!(should_print_result("(  )"));
    }

    #[test]
    fn test_read_file_with_context_success() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "test content").unwrap();
        let result = read_file_with_context(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test content");
    }

    #[test]
    fn test_read_file_with_context_unicode() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "こんにちは世界").unwrap();
        let result = read_file_with_context(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "こんにちは世界");
    }

    #[test]
    fn test_read_file_with_context_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "").unwrap();
        let result = read_file_with_context(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_create_repl_returns_ok() {
        let result = create_repl();
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_repl_multiple_times() {
        // Should be able to create multiple REPL instances
        let repl1 = create_repl();
        let repl2 = create_repl();
        assert!(repl1.is_ok());
        assert!(repl2.is_ok());
    }

    #[test]
    fn test_log_command_output_verbose_false() {
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: b"test error".to_vec(),
        };
        // Should not panic
        log_command_output(&output, false);
    }

    #[test]
    fn test_log_command_output_verbose_true() {
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: b"test error output".to_vec(),
        };
        // Should print but not panic
        log_command_output(&output, true);
    }

    #[test]
    fn test_log_command_output_empty() {
        let output = std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        };
        log_command_output(&output, true);
        log_command_output(&output, false);
    }

    #[test]
    fn test_write_file_with_context_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("output.txt");
        let result = write_file_with_context(&file_path, b"test content");
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "test content");
    }

    #[test]
    fn test_write_file_with_context_overwrite() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "original").unwrap();
        let result = write_file_with_context(temp_file.path(), b"new content");
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(temp_file.path()).unwrap(), "new content");
    }

    #[test]
    fn test_write_file_with_context_unicode() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("unicode.txt");
        let result = write_file_with_context(&file_path, "こんにちは".as_bytes());
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "こんにちは");
    }

    #[test]
    fn test_write_file_with_context_empty() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        let result = write_file_with_context(&file_path, b"");
        assert!(result.is_ok());
        assert_eq!(std::fs::read_to_string(&file_path).unwrap(), "");
    }

    #[test]
    fn test_write_file_with_context_error() {
        let result = write_file_with_context(Path::new("/nonexistent/dir/file.txt"), b"test");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("/nonexistent/dir/file.txt"));
    }
}
