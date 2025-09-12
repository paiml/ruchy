//! Helper functions for test command
//! Extracted to maintain ≤10 complexity per function
use anyhow::{Context, Result};
use colored::Colorize;
use ruchy::utils::read_file_with_context;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use walkdir::WalkDir;
/// Test result information
pub struct TestResult {
    pub file: PathBuf,
    pub success: bool,
    pub duration: Duration,
    pub error: Option<String>,
}
/// Discover .ruchy test files in a path
pub fn discover_test_files(path: &Path, filter: Option<&str>, verbose: bool) -> Result<Vec<PathBuf>> {
    if verbose {
        println!("🔍 Discovering .ruchy test files in {}", path.display());
    }
    let mut test_files = Vec::new();
    if path.is_file() {
        validate_and_add_file(path, &mut test_files)?;
    } else if path.is_dir() {
        discover_files_in_directory(path, filter, &mut test_files)?;
    } else {
        return Err(anyhow::anyhow!("Path {} does not exist", path.display()));
    }
    Ok(test_files)
}
/// Validate and add a single file
fn validate_and_add_file(path: &Path, test_files: &mut Vec<PathBuf>) -> Result<()> {
    if path.extension().is_some_and(|ext| ext == "ruchy") {
        test_files.push(path.to_path_buf());
    } else {
        return Err(anyhow::anyhow!("File {} is not a .ruchy file", path.display()));
    }
    Ok(())
}
/// Discover files in a directory
fn discover_files_in_directory(
    path: &Path,
    filter: Option<&str>,
    test_files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if should_include_file(&entry, filter) {
            test_files.push(entry.path().to_path_buf());
        }
    }
    Ok(())
}
/// Check if file should be included based on filter
fn should_include_file(entry: &walkdir::DirEntry, filter: Option<&str>) -> bool {
    if entry.path().extension().is_none_or(|ext| ext != "ruchy") {
        return false;
    }
    if let Some(filter_pattern) = filter {
        let file_name = entry.path().file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        file_name.contains(filter_pattern)
    } else {
        true
    }
}
/// Run a single .ruchy test file
pub fn run_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    use ruchy::runtime::repl::Repl;
    let test_content = read_file_with_context(test_file)?;
    if verbose {
        println!("   📖 Parsing test file...");
        println!("   🏃 Executing test...");
    }
    let mut repl = Repl::new()?;
    let result = repl.evaluate_expr_str(&test_content, None)
        .with_context(|| format!("Test execution failed for: {}", test_file.display()))?;
    if verbose {
        println!("   📤 Test result: {:?}", result);
    }
    Ok(())
}
/// Execute all test files
pub fn execute_tests(
    test_files: &[PathBuf],
    verbose: bool,
) -> Vec<TestResult> {
    let mut test_results = Vec::new();
    for test_file in test_files {
        if verbose {
            println!("📄 Testing: {}", test_file.display());
        }
        let test_start = Instant::now();
        let result = run_test_file(test_file, verbose);
        let test_duration = test_start.elapsed();
        handle_test_result(
            test_file,
            result,
            test_duration,
            verbose,
            &mut test_results,
        );
    }
    test_results
}
/// Handle a single test result
fn handle_test_result(
    test_file: &Path,
    result: Result<()>,
    duration: Duration,
    verbose: bool,
    test_results: &mut Vec<TestResult>,
) {
    match result {
        Ok(()) => {
            if verbose {
                println!("   ✅ {} ({:.2}ms)", 
                    test_file.file_name().unwrap().to_str().expect("Failed to convert to str"), 
                    duration.as_secs_f64() * 1000.0);
            } else {
                print!(".");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            test_results.push(TestResult {
                file: test_file.to_path_buf(),
                success: true,
                duration,
                error: None,
            });
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            if verbose {
                println!("   ❌ {} ({:.2}ms): {}", 
                    test_file.file_name().unwrap().to_str().expect("Failed to convert to str"), 
                    duration.as_secs_f64() * 1000.0, 
                    error_msg);
            } else {
                print!("F");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            test_results.push(TestResult {
                file: test_file.to_path_buf(),
                success: false,
                duration,
                error: Some(error_msg),
            });
        }
    }
}
/// Print test summary
pub fn print_test_summary(
    test_results: &[TestResult],
    total_duration: Duration,
    verbose: bool,
) {
    if !verbose {
        println!(); // New line after dots/F's
    }
    let passed = test_results.iter().filter(|r| r.success).count();
    let failed = test_results.len() - passed;
    println!("\n📊 Test Results:");
    println!("   Total: {}", test_results.len());
    println!("   Passed: {}", passed.to_string().green());
    if failed > 0 {
        println!("   Failed: {}", failed.to_string().red());
        print_failed_tests(test_results, verbose);
    }
    println!("   Duration: {:.2}s", total_duration.as_secs_f64());
}
/// Print details of failed tests
fn print_failed_tests(test_results: &[TestResult], verbose: bool) {
    if verbose {
        return; // Already printed during execution
    }
    println!("\n❌ Failed Tests:");
    for result in test_results {
        if !result.success {
            println!("   {} - {}", 
                result.file.display(), 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }
    }
}
/// Generate JSON output for test results
pub fn generate_json_output(
    test_results: &[TestResult],
    total_duration: Duration,
) -> Result<String> {
    let passed = test_results.iter().filter(|r| r.success).count();
    let failed = test_results.len() - passed;
    let json_output = serde_json::json!({
        "total": test_results.len(),
        "passed": passed,
        "failed": failed,
        "duration_seconds": total_duration.as_secs_f64(),
        "results": test_results.iter().map(|r| {
            serde_json::json!({
                "file": r.file.display().to_string(),
                "success": r.success,
                "duration_ms": r.duration.as_secs_f64() * 1000.0,
                "error": r.error
            })
        }).collect::<Vec<_>>()
    });
    Ok(serde_json::to_string_pretty(&json_output)?)
}
/// Handle coverage reporting
pub fn generate_coverage_report(
    test_files: &[PathBuf],
    test_results: &[TestResult],
    coverage_format: &str,
    threshold: f64,
) -> Result<()> {
    use ruchy::quality::ruchy_coverage::RuchyCoverageCollector;
    let mut collector = RuchyCoverageCollector::new();
    // Analyze test files
    for test_file in test_files {
        if let Err(e) = collector.analyze_file(test_file) {
            eprintln!("Warning: Failed to analyze {}: {}", test_file.display(), e);
        }
    }
    // Collect runtime coverage for successful tests
    for result in test_results {
        if result.success {
            if let Err(e) = collector.execute_with_coverage(&result.file) {
                eprintln!("Warning: Failed to collect runtime coverage for {}: {}", 
                    result.file.display(), e);
            }
        }
    }
    // Generate and output report
    output_coverage_report(&collector, coverage_format)?;
    // Check threshold
    check_coverage_threshold(&collector, threshold);
    Ok(())
}
/// Output coverage report in requested format
fn output_coverage_report(
    collector: &ruchy::quality::ruchy_coverage::RuchyCoverageCollector,
    format: &str,
) -> Result<()> {
    let report = match format {
        "json" => collector.generate_json_report(),
        "html" => {
            let html_report = collector.generate_html_report();
            save_html_report(&html_report)?
        }
        _ => collector.generate_text_report(),
    };
    println!("{}", report);
    Ok(())
}
/// Save HTML coverage report to file
fn save_html_report(html_report: &str) -> Result<String> {
    let coverage_dir = Path::new("target/coverage");
    fs::create_dir_all(coverage_dir)?;
    let html_path = coverage_dir.join("index.html");
    fs::write(&html_path, html_report)?;
    Ok(format!("\n📈 HTML Coverage Report written to: {}", html_path.display()))
}
/// Check if coverage meets threshold
fn check_coverage_threshold(
    collector: &ruchy::quality::ruchy_coverage::RuchyCoverageCollector,
    threshold: f64,
) {
    if threshold > 0.0 {
        if collector.meets_threshold(threshold) {
            println!("\n✅ Coverage meets threshold of {:.1}%", threshold);
        } else {
            eprintln!("\n❌ Coverage below threshold of {:.1}%", threshold);
            std::process::exit(1);
        }
    }
}