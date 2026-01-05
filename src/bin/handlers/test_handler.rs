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
    #[test]
    fn test_test_handler_stub() {
        // Test handler tests are in handlers_modules::test
        // This is a placeholder for the delegation layer
    }
}
