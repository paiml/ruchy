//! Check Command Handler
//!
//! Handles syntax checking of Ruchy source files.

use anyhow::Result;
use colored::Colorize;
use ruchy::Parser as RuchyParser;
use std::fs;
use std::path::{Path, PathBuf};

/// Handle check command - validate syntax of Ruchy files
///
/// # Arguments
/// * `files` - List of files to check
/// * `watch` - Enable watch mode for continuous checking
///
/// # Errors
/// Returns error if files cannot be read or have syntax errors
pub fn handle_check_command(files: &[PathBuf], watch: bool) -> Result<()> {
    // FIX CLI-CONTRACT-CHECK-003: Support checking multiple files
    validate_file_list(files)?;

    if watch {
        check_watch_mode(files)
    } else if files.len() == 1 {
        // Single file - return error directly for better error messages
        handle_check_syntax(&files[0])
    } else {
        check_multiple_files(files)
    }
}

/// Validate that file list is not empty (complexity: 1)
fn validate_file_list(files: &[PathBuf]) -> Result<()> {
    if files.is_empty() {
        anyhow::bail!("No files specified for checking");
    }
    Ok(())
}

/// Handle watch mode for check command (complexity: 2)
fn check_watch_mode(files: &[PathBuf]) -> Result<()> {
    if files.len() > 1 {
        anyhow::bail!("Watch mode only supports checking a single file");
    }
    handle_watch_and_check(&files[0])
}

/// Check multiple files sequentially (complexity: 4)
fn check_multiple_files(files: &[PathBuf]) -> Result<()> {
    let mut all_valid = true;
    for file in files {
        if let Err(e) = handle_check_syntax(file) {
            all_valid = false;
            eprintln!("{e}");
        }
    }
    if all_valid {
        Ok(())
    } else {
        anyhow::bail!("Some files have syntax errors")
    }
}

/// Check syntax of a single file
pub fn handle_check_syntax(file: &Path) -> Result<()> {
    let source = super::read_file_with_context(file)?;
    let mut parser = RuchyParser::new(&source);
    match parser.parse() {
        Ok(_) => {
            println!("{}", "✓ Syntax is valid".green());
            Ok(())
        }
        Err(e) => {
            // FIX CLI-CONTRACT-CHECK-001: Include filename in error message
            // FIX CLI-CONTRACT-CHECK-002: Include line number in error message
            let filename = file.display();
            let line_info = estimate_error_line(&source, &e.to_string());
            let error_location = if let Some(line) = line_info {
                format!("{filename}:{line}")
            } else {
                format!("{filename}")
            };
            eprintln!("{}", format!("✗ {error_location}: Syntax error: {e}").red());
            Err(anyhow::anyhow!("{error_location}: Syntax error: {}", e))
        }
    }
}

/// Estimate the line number where a parse error occurred (complexity: 5)
///
/// This is a heuristic that counts newlines in the source code to find the approximate
/// error location. Ideally, the parser would include precise span information in errors,
/// but that requires significant parser refactoring.
pub fn estimate_error_line(source: &str, _error_msg: &str) -> Option<usize> {
    // Heuristic: Most parse errors occur near the end of the source that was successfully
    // tokenized. Count total lines and report the last non-empty line as the error location.
    // This is not perfect but better than no line number at all.
    let lines: Vec<&str> = source.lines().collect();

    // Find the last non-empty, non-comment line
    for (idx, line) in lines.iter().enumerate().rev() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            return Some(idx + 1); // Line numbers are 1-indexed
        }
    }

    // If all lines are empty/comments, return the last line
    if lines.is_empty() {
        None
    } else {
        Some(lines.len())
    }
}

/// Watch a file and check syntax on changes
fn handle_watch_and_check(file: &Path) -> Result<()> {
    use std::thread;
    use std::time::Duration;
    println!(
        "{} Watching {} for changes...",
        "👁".bright_cyan(),
        file.display()
    );
    println!("Press Ctrl+C to stop watching\n");
    // Initial check
    handle_check_syntax(file)?;
    // Simple file watching using polling
    let mut last_modified = fs::metadata(file)?.modified()?;
    loop {
        thread::sleep(Duration::from_millis(500));
        let Ok(metadata) = fs::metadata(file) else {
            continue; // File might be temporarily unavailable
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if modified != last_modified {
            last_modified = modified;
            println!("\n{} File changed, checking...", "→".bright_cyan());
            let _ = handle_check_syntax(file); // Don't exit on error, keep watching
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_file_list_empty() {
        let result = validate_file_list(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_list_non_empty() {
        let files = vec![PathBuf::from("test.ruchy")];
        let result = validate_file_list(&files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_error_line_simple() {
        let source = "line1\nline2\nline3";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(3));
    }

    #[test]
    fn test_estimate_error_line_with_comments() {
        let source = "line1\n// comment\n";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(1));
    }

    #[test]
    fn test_estimate_error_line_empty() {
        let source = "";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, None);
    }

    #[test]
    fn test_estimate_error_line_only_empty() {
        let source = "\n\n\n";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(3)); // Returns last line
    }

    #[test]
    fn test_check_watch_mode_multiple_files() {
        let files = vec![PathBuf::from("a.ruchy"), PathBuf::from("b.ruchy")];
        let result = check_watch_mode(&files);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("only supports checking a single file"));
    }

    // ===== EXTREME TDD Round 152 - Check Handler Tests =====

    #[test]
    fn test_estimate_error_line_single_line() {
        let source = "let x = 42";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(1));
    }

    #[test]
    fn test_estimate_error_line_all_comments() {
        let source = "// comment1\n// comment2\n// comment3";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(3)); // Returns last line when all comments
    }

    #[test]
    fn test_estimate_error_line_mixed_content() {
        let source = "let x = 1\n// comment\nlet y = 2\n// trailing";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(3)); // Last non-comment line
    }

    #[test]
    fn test_handle_check_command_empty_files() {
        let result = handle_check_command(&[], false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No files specified"));
    }

    #[test]
    fn test_handle_check_command_nonexistent_file() {
        let files = vec![PathBuf::from("/nonexistent/file.ruchy")];
        let result = handle_check_command(&files, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_multiple_files_all_nonexistent() {
        let files = vec![
            PathBuf::from("/nonexistent/a.ruchy"),
            PathBuf::from("/nonexistent/b.ruchy"),
        ];
        let result = check_multiple_files(&files);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_list_multiple_files() {
        let files = vec![
            PathBuf::from("a.ruchy"),
            PathBuf::from("b.ruchy"),
            PathBuf::from("c.ruchy"),
        ];
        let result = validate_file_list(&files);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_error_line_whitespace_only() {
        let source = "   \n   \n   ";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(3));
    }

    #[test]
    fn test_handle_check_syntax_valid_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid.ruchy");
        fs::write(&file_path, "let x = 42").unwrap();

        let result = handle_check_syntax(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_check_syntax_invalid_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.ruchy");
        fs::write(&file_path, "let x = {").unwrap();

        let result = handle_check_syntax(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_check_command_single_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");
        fs::write(&file_path, "42").unwrap();

        let files = vec![file_path];
        let result = handle_check_command(&files, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_estimate_error_line_code_then_comment() {
        let source = "fun foo() { }\n// end of file";
        let line = estimate_error_line(source, "error");
        assert_eq!(line, Some(1));
    }

    #[test]
    fn test_check_watch_mode_single_file() {
        // Note: We can't actually test watch mode since it loops forever
        // Just test the early validation
        let files = vec![PathBuf::from("/nonexistent/file.ruchy")];
        let result = check_watch_mode(&files);
        // This will fail on file read, not watch mode validation
        assert!(result.is_err());
    }
}
