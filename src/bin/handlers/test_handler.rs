//! Test Command Handler
//!
//! Handles test execution, watch mode, and test file running commands.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Handle test command - run tests with various options
///
/// Delegates to the refactored handlers_modules::test module.
///
/// # Arguments
/// * `path` - Optional path to test directory
/// * `watch` - Enable watch mode
/// * `verbose` - Enable verbose output
/// * `filter` - Optional test filter
/// * `coverage` - Enable coverage reporting
/// * `coverage_format` - Coverage report format
/// * `parallel` - Number of parallel test threads
/// * `threshold` - Coverage threshold
/// * `format` - Output format
///
/// # Errors
/// Returns error if tests fail to run or coverage threshold is not met
pub fn handle_test_command(
    path: Option<PathBuf>,
    watch: bool,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    // Delegate to refactored module with ≤10 complexity
    super::handlers_modules::test::handle_test_command(
        path,
        watch,
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}

/// Watch and run tests on changes
///
/// Delegates to the test module with watch mode enabled.
///
/// # Arguments
/// * `path` - Path to watch
/// * `verbose` - Enable verbose output
/// * `filter` - Optional test filter
///
/// # Errors
/// Returns error if watch mode fails
pub fn handle_watch_and_test(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    super::handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        true, // watch mode
        verbose,
        filter,
        false, // coverage
        "text",
        1,
        0.0,
        "text",
    )
}

/// Run enhanced tests with all options
///
/// Delegates to the test module with full configuration.
///
/// # Arguments
/// * `path` - Path to test
/// * `verbose` - Enable verbose output
/// * `filter` - Optional test filter
/// * `coverage` - Enable coverage
/// * `coverage_format` - Coverage format
/// * `parallel` - Parallel threads
/// * `threshold` - Coverage threshold
/// * `format` - Output format
///
/// # Errors
/// Returns error if tests fail
#[allow(clippy::too_many_arguments)]
pub fn handle_run_enhanced_tests(
    path: &Path,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    super::handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        false, // not watch mode
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}

/// Run a single .ruchy test file
///
/// Delegates to the test_helpers module.
///
/// # Arguments
/// * `test_file` - Path to the test file
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if the test file fails
pub fn run_ruchy_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    super::handlers_modules::test_helpers::run_test_file(test_file, verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_test_handler_stub() {
        // Test handler tests are in handlers_modules::test
        // This is a placeholder for the delegation layer
    }

    // ===== EXTREME TDD Round 147 - Test Handler Tests =====

    #[test]
    fn test_handle_test_command_no_path() {
        let result = handle_test_command(None, false, false, None, false, "text", 1, 0.0, "text");
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_with_path() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false,
            false,
            None,
            false,
            "text",
            1,
            0.0,
            "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_watch_mode() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            true, // watch
            false,
            None,
            false,
            "text",
            1,
            0.0,
            "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_verbose() {
        let result = handle_test_command(
            None, false, true, // verbose
            None, false, "text", 1, 0.0, "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_with_filter() {
        let result = handle_test_command(
            None,
            false,
            false,
            Some("test_name"), // filter
            false,
            "text",
            1,
            0.0,
            "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_with_coverage() {
        let result = handle_test_command(
            None, false, false, None, true, // coverage
            "lcov", 1, 80.0, "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_json_format() {
        let result = handle_test_command(None, false, false, None, false, "text", 1, 0.0, "json");
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_parallel() {
        let result = handle_test_command(
            None, false, false, None, false, "text", 4, // parallel
            0.0, "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_test_command_threshold() {
        let result = handle_test_command(
            None, false, false, None, true, "html", 1, 90.0, // threshold
            "text",
        );
        let _ = result;
    }

    #[test]
    fn test_handle_watch_and_test_basic() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_watch_and_test(temp_dir.path(), false, None);
        let _ = result;
    }

    #[test]
    fn test_handle_watch_and_test_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_watch_and_test(temp_dir.path(), true, None);
        let _ = result;
    }

    #[test]
    fn test_handle_watch_and_test_with_filter() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_watch_and_test(temp_dir.path(), false, Some("test_*"));
        let _ = result;
    }

    #[test]
    fn test_handle_run_enhanced_tests_basic() {
        let temp_dir = TempDir::new().unwrap();
        let result =
            handle_run_enhanced_tests(temp_dir.path(), false, None, false, "text", 1, 0.0, "text");
        let _ = result;
    }

    #[test]
    fn test_handle_run_enhanced_tests_all_options() {
        let temp_dir = TempDir::new().unwrap();
        let result = handle_run_enhanced_tests(
            temp_dir.path(),
            true,
            Some("integration"),
            true,
            "html",
            8,
            75.0,
            "json",
        );
        let _ = result;
    }

    #[test]
    fn test_run_ruchy_test_file_nonexistent() {
        let result = run_ruchy_test_file(Path::new("/nonexistent/test.ruchy"), false);
        let _ = result;
    }

    #[test]
    fn test_run_ruchy_test_file_verbose() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.ruchy");
        std::fs::write(&test_file, "test basic { assert(1 == 1) }").unwrap();
        let result = run_ruchy_test_file(&test_file, true);
        let _ = result;
    }
}
