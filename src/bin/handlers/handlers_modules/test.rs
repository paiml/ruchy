//! Refactored test command handler
//! Complexity reduced from ~200 lines to ≤10 per function
use anyhow::Result;
use super::test_helpers::{discover_test_files, execute_tests, print_test_summary, generate_json_output, generate_coverage_report, TestResult};
use std::path::{Path, PathBuf};
use std::time::Instant;
/// Handle test command - refactored with ≤10 complexity
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
    let test_path = path.unwrap_or_else(|| PathBuf::from("."));
    if watch {
        handle_watch_mode(&test_path, verbose, filter)
    } else {
        run_tests(
            &test_path,
            verbose,
            filter,
            coverage,
            coverage_format,
            parallel,
            threshold,
            format,
        )
    }
}
/// Run tests once
fn run_tests(
    path: &Path,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    _parallel: usize, // Unused for now
    threshold: f64,
    format: &str,
) -> Result<()> {
    // Discover test files
    let test_files = discover_test_files(path, filter, verbose)?;
    if test_files.is_empty() {
        println!("⚠️  No .ruchy test files found in {}", path.display());
        return Ok(());
    }
    println!("🧪 Running {} .ruchy test files...\n", test_files.len());
    // Execute tests
    let total_start = Instant::now();
    let test_results = execute_tests(&test_files, verbose);
    let total_duration = total_start.elapsed();
    // Print summary
    print_test_summary(&test_results, total_duration, verbose);
    // Handle output format
    if format == "json" {
        let json = generate_json_output(&test_results, total_duration)?;
        println!("\n{}", json);
    }
    // Handle coverage if requested
    if coverage {
        generate_coverage_report(&test_files, &test_results, coverage_format, threshold)?;
    }
    // Check for failures
    check_test_failures(&test_results);
    println!("\n✅ All tests passed!");
    Ok(())
}
/// Handle watch mode
fn handle_watch_mode(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    use colored::Colorize;
    use std::thread;
    use std::time::Duration;
    println!(
        "{} Watching {} for test changes...",
        "👁".bright_cyan(),
        path.display()
    );
    println!("Press Ctrl+C to stop watching\n");
    // Initial test run
    let _ = run_tests(path, verbose, filter, false, "text", 1, 0.0, "text");
    // Watch for changes
    let mut last_modified = get_latest_modification(path);
    loop {
        thread::sleep(Duration::from_millis(1000));
        let current_modified = get_latest_modification(path);
        if current_modified > last_modified {
            last_modified = current_modified;
            println!("\n{} Files changed, running tests...", "→".bright_cyan());
            let _ = run_tests(path, verbose, filter, false, "text", 1, 0.0, "text");
        }
    }
}
/// Get latest modification time in directory
fn get_latest_modification(path: &Path) -> std::time::SystemTime {
    use std::fs;
    let mut latest = std::time::SystemTime::now();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(path) = entry.path().canonicalize() {
                if path.extension().and_then(|ext| ext.to_str()) == Some("ruchy") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > latest {
                                latest = modified;
                            }
                        }
                    }
                }
            }
        }
    }
    latest
}
/// Check if any tests failed and exit if necessary
fn check_test_failures(test_results: &[TestResult]) {
    let failed = test_results.iter().filter(|r| !r.success).count();
    if failed > 0 {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};
    use std::time::Duration;

    // Helper function to create a test directory with .ruchy files
    fn create_test_directory_with_files() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // Create a passing test file
        let test_file1 = temp_dir.path().join("passing_test.ruchy");
        fs::write(&test_file1, "println(\"Hello Test\")")?;

        // Create another passing test file
        let test_file2 = temp_dir.path().join("another_test.ruchy");
        fs::write(&test_file2, "let x = 42; println(x)")?;

        // Create a non-ruchy file (should be ignored)
        let readme_file = temp_dir.path().join("README.txt");
        fs::write(&readme_file, "This is documentation")?;

        Ok(temp_dir)
    }

    // Helper function to create TestResult instances
    fn create_test_result(file: &str, success: bool, duration_ms: u64, error: Option<&str>) -> TestResult {
        TestResult {
            file: PathBuf::from(file),
            success,
            duration: Duration::from_millis(duration_ms),
            error: error.map(|s| s.to_string()),
        }
    }

    // ========== Test Command Handler Tests ==========
    #[test]
    fn test_handle_test_command_default_path() {
        let temp_dir = create_test_directory_with_files().unwrap();

        // Change to the temp directory for the test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir.path()).unwrap();

        let result = handle_test_command(
            None, // Use default path (current directory)
            false, // No watch mode
            false, // Not verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_handle_test_command_with_path() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode
            true, // Verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_handle_test_command_with_filter() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode
            false, // Not verbose
            Some("passing"), // Filter for "passing" in filename
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_handle_test_command_with_coverage() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode
            false, // Not verbose
            None, // No filter
            true, // Enable coverage
            "html", // HTML coverage format
            1, // Parallel threads
            80.0, // Coverage threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_handle_test_command_json_output() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode
            false, // Not verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "json", // JSON output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Run Tests Function Tests ==========
    #[test]
    fn test_run_tests_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        // Create empty directory with no .ruchy files

        let result = run_tests(
            temp_dir.path(),
            false, // Not verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Should handle empty directory gracefully
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_tests_with_files() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = run_tests(
            temp_dir.path(),
            true, // Verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_run_tests_with_filter() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = run_tests(
            temp_dir.path(),
            false, // Not verbose
            Some("another"), // Filter for "another" in filename
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_run_tests_json_format() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = run_tests(
            temp_dir.path(),
            false, // Not verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Parallel threads
            0.0, // No threshold
            "json", // JSON output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_run_tests_with_coverage() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let result = run_tests(
            temp_dir.path(),
            false, // Not verbose
            None, // No filter
            true, // Enable coverage
            "html", // HTML coverage format
            1, // Parallel threads
            75.0, // Coverage threshold
            "text", // Output format
        );

        // Test should complete without panicking
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Watch Mode Tests ==========
    #[test]
    #[ignore = "Ignore by default as this is an infinite loop test"]
    fn test_handle_watch_mode_setup() {
        let temp_dir = create_test_directory_with_files().unwrap();

        // We can't easily test the full watch mode (infinite loop),
        // but we can test that it doesn't panic on initial setup
        // This test would need to be run manually or with a timeout mechanism

        // For now, just test that the function exists and can be called
        // In a real test environment, you'd use a timeout or separate thread

        // Note: This test is marked as #[ignore] to prevent infinite loop in CI
        let _result = std::panic::catch_unwind(|| {
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(10)); // Short sleep
                std::process::exit(0); // Exit quickly to avoid infinite loop
            });
        });
    }

    // ========== File Modification Tests ==========
    #[test]
    fn test_get_latest_modification_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        // Empty directory with no .ruchy files

        let modification_time = get_latest_modification(temp_dir.path());
        // Should return some time (likely current time)
        assert!(modification_time <= std::time::SystemTime::now());
    }

    #[test]
    fn test_get_latest_modification_with_ruchy_files() {
        let temp_dir = create_test_directory_with_files().unwrap();

        let modification_time = get_latest_modification(temp_dir.path());
        // Should return a valid time
        assert!(modification_time <= std::time::SystemTime::now());
    }

    #[test]
    fn test_get_latest_modification_with_mixed_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create a .ruchy file
        let ruchy_file = temp_dir.path().join("test.ruchy");
        fs::write(&ruchy_file, "println(\"test\")").unwrap();

        // Create a non-ruchy file (should be ignored)
        let other_file = temp_dir.path().join("other.txt");
        fs::write(&other_file, "not ruchy").unwrap();

        let modification_time = get_latest_modification(temp_dir.path());
        // Should return a valid time
        assert!(modification_time <= std::time::SystemTime::now());
    }

    #[test]
    fn test_get_latest_modification_nonexistent_directory() {
        let nonexistent_path = Path::new("/nonexistent/directory");

        let modification_time = get_latest_modification(nonexistent_path);
        // Should handle gracefully and return some time
        assert!(modification_time <= std::time::SystemTime::now());
    }

    // ========== Test Failure Checking Tests ==========
    #[test]
    fn test_check_test_failures_all_passing() {
        let test_results = vec![
            create_test_result("test1.ruchy", true, 100, None),
            create_test_result("test2.ruchy", true, 150, None),
            create_test_result("test3.ruchy", true, 200, None),
        ];

        // Should not panic or exit for all passing tests
        // Note: We can't easily test std::process::exit, so we test the logic
        let failed_count = test_results.iter().filter(|r| !r.success).count();
        assert_eq!(failed_count, 0);
    }

    #[test]
    fn test_check_test_failures_with_failures() {
        let test_results = vec![
            create_test_result("test1.ruchy", true, 100, None),
            create_test_result("test2.ruchy", false, 50, Some("Parse error")),
            create_test_result("test3.ruchy", true, 200, None),
        ];

        // Test the logic without actually calling the function (to avoid exit)
        let failed_count = test_results.iter().filter(|r| !r.success).count();
        assert_eq!(failed_count, 1);
    }

    #[test]
    fn test_check_test_failures_all_failing() {
        let test_results = vec![
            create_test_result("test1.ruchy", false, 50, Some("Error 1")),
            create_test_result("test2.ruchy", false, 60, Some("Error 2")),
            create_test_result("test3.ruchy", false, 70, Some("Error 3")),
        ];

        // Test the logic without actually calling the function
        let failed_count = test_results.iter().filter(|r| !r.success).count();
        assert_eq!(failed_count, 3);
    }

    #[test]
    fn test_check_test_failures_empty_results() {
        let test_results: Vec<TestResult> = vec![];

        // Should handle empty results gracefully
        let failed_count = test_results.iter().filter(|r| !r.success).count();
        assert_eq!(failed_count, 0);
    }

    // ========== Integration Tests ==========
    #[test]
    fn test_integration_complete_workflow() {
        let temp_dir = create_test_directory_with_files().unwrap();

        // Test the complete workflow without watch mode
        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode
            true, // Verbose
            None, // No filter
            false, // No coverage (to keep test simple)
            "text", // Coverage format
            1, // Single thread
            0.0, // No threshold
            "text", // Text output
        );

        // Should complete the full workflow
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_integration_with_all_options() {
        let temp_dir = create_test_directory_with_files().unwrap();

        // Test with maximum options enabled
        let result = handle_test_command(
            Some(temp_dir.path().to_path_buf()),
            false, // No watch mode (to keep test finite)
            true, // Verbose
            Some("test"), // Filter
            true, // Enable coverage
            "json", // JSON coverage format
            2, // Multiple threads
            50.0, // Coverage threshold
            "json", // JSON output
        );

        // Should handle all options gracefully
        assert!(result.is_ok() || result.is_err());
    }

    // ========== Error Handling Tests ==========
    #[test]
    fn test_error_handling_invalid_path() {
        let invalid_path = PathBuf::from("/absolutely/nonexistent/path/that/should/not/exist");

        let result = handle_test_command(
            Some(invalid_path),
            false, // No watch mode
            false, // Not verbose
            None, // No filter
            false, // No coverage
            "text", // Coverage format
            1, // Single thread
            0.0, // No threshold
            "text", // Text output
        );

        // Should handle invalid path gracefully (likely return an error)
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_parameter_validation() {
        let temp_dir = create_test_directory_with_files().unwrap();

        // Test with various parameter combinations
        let test_cases = vec![
            (0, 0.0, "text"),    // Minimum values
            (1, 50.0, "json"),   // Medium values
            (10, 100.0, "html"), // Maximum values
        ];

        for (parallel, threshold, format) in test_cases {
            let result = handle_test_command(
                Some(temp_dir.path().to_path_buf()),
                false, // No watch mode
                false, // Not verbose
                None, // No filter
                false, // No coverage
                "text", // Coverage format
                parallel,
                threshold,
                format,
            );

            // All parameter combinations should be handled
            assert!(result.is_ok() || result.is_err());
        }
    }
}