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
    check_test_failures(&test_results)?;
    
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
fn check_test_failures(test_results: &[TestResult]) -> Result<()> {
    let failed = test_results.iter().filter(|r| !r.success).count();
    if failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}