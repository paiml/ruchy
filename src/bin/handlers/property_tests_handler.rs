//! Property Tests Command Handler
//!
//! Handles property-based testing for Ruchy files.

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Run property test suite via cargo test
/// Complexity: 3 (Toyota Way: <10)
fn run_property_test_suite(
    cases: usize,
    seed: Option<u64>,
    verbose: bool,
) -> Result<std::process::Output> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["test", "--test", "lang_comp_suite", "--", "--nocapture"])
        .env("PROPTEST_CASES", cases.to_string());

    if let Some(s) = seed {
        cmd.env("PROPTEST_SEED", s.to_string());
    }

    let output_result = cmd.output()?;
    super::log_command_output(&output_result, verbose);

    Ok(output_result)
}

/// Write property test summary report
fn write_property_test_summary(
    format: &str,
    output: Option<&Path>,
    cases: usize,
    stdout: &str,
) -> Result<()> {
    if format == "json" {
        write_property_test_json(output, cases, stdout)
    } else {
        write_property_test_text(output, cases, stdout)
    }
}

/// Write JSON property test report
fn write_property_test_json(output: Option<&Path>, cases: usize, stdout: &str) -> Result<()> {
    let report = serde_json::json!({
        "status": "passed",
        "cases": cases,
        "output": stdout
    });
    let json_output = serde_json::to_string_pretty(&report)?;
    if let Some(out_path) = output {
        fs::write(out_path, json_output)?;
    } else {
        println!("{}", json_output);
    }
    Ok(())
}

/// Write text property test report
fn write_property_test_text(output: Option<&Path>, cases: usize, stdout: &str) -> Result<()> {
    println!("Property Test Report");
    println!("====================");
    println!("Status: PASSED");
    println!("Test cases: {}", cases);
    if let Some(out_path) = output {
        super::write_file_with_context(out_path, stdout.as_bytes())?;
    } else {
        println!("\n{}", stdout);
    }
    Ok(())
}

/// Handle property tests command - single file or test suite
pub fn handle_property_tests_command(
    path: &Path,
    cases: usize,
    format: &str,
    output: Option<&Path>,
    seed: Option<u64>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        eprintln!("Running property tests on: {}", path.display());
        eprintln!("Test cases per property: {}", cases);
    }

    if !path.exists() {
        anyhow::bail!("{}: File or directory not found", path.display());
    }

    if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ruchy") {
        return handle_property_tests_single_file(path, cases, format, output, seed, verbose);
    }

    let output_result = run_property_test_suite(cases, seed, verbose)?;
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    let stderr = String::from_utf8_lossy(&output_result.stderr);

    if output_result.status.success() {
        write_property_test_summary(format, output, cases, &stdout)?;
        Ok(())
    } else {
        anyhow::bail!("Property tests failed:\n{}", stderr)
    }
}

/// Compile Ruchy file for property testing
pub(crate) fn compile_for_property_testing(path: &Path, verbose: bool) -> Result<PathBuf> {
    use super::{compile_rust_code, prepare_compilation, transpile_for_execution};
    use crate::handlers::transpile_handler::parse_source;

    if verbose {
        eprintln!("Compiling file once for property testing...");
    }

    let source = super::read_file_with_context(path)?;
    let ast = parse_source(&source)?;
    let rust_code = transpile_for_execution(&ast, path)?;
    let (temp_source, binary_path) = prepare_compilation(&rust_code, verbose)?;
    compile_rust_code(temp_source.path(), &binary_path)?;

    if verbose {
        eprintln!("Binary compiled: {}", binary_path.display());
    }

    Ok(binary_path)
}

/// Run panic property tests (executes binary N times)
fn run_panic_property_tests(
    binary_path: &Path,
    cases: usize,
    verbose: bool,
) -> Result<(usize, Vec<String>)> {
    if verbose {
        eprintln!("Property 1: Testing {} executions for panics...", cases);
    }

    let mut failures = Vec::new();
    for i in 0..cases {
        let result = std::process::Command::new(binary_path).output()?;

        if !result.status.success() {
            failures.push(format!(
                "Iteration {}: FAILED - {}",
                i,
                String::from_utf8_lossy(&result.stderr)
            ));
            if verbose {
                eprintln!("  Iteration {}: FAILED", i);
            }
        }
    }

    Ok((cases - failures.len(), failures))
}

/// Test output determinism (run twice, compare)
fn test_output_determinism(binary_path: &Path, verbose: bool) -> Result<bool> {
    if verbose {
        eprintln!("Property 2: Testing output determinism...");
    }

    let run1 = std::process::Command::new(binary_path).output()?;
    let run2 = std::process::Command::new(binary_path).output()?;

    Ok(run1.stdout == run2.stdout)
}

/// Generate property test report (JSON or text)
fn generate_property_test_report(
    path: &Path,
    format: &str,
    output: Option<&Path>,
    cases: usize,
    passed: usize,
    failed: usize,
    deterministic: bool,
    test_results: &[String],
) -> Result<()> {
    let total_tests = cases + 1;
    let success = failed == 0;

    match format {
        "json" => write_json_property_report(
            path,
            output,
            success,
            total_tests,
            passed,
            failed,
            cases,
            deterministic,
            test_results,
        ),
        _ => write_text_property_report(
            path,
            output,
            success,
            total_tests,
            passed,
            failed,
            cases,
            deterministic,
            test_results,
        ),
    }
}

/// Write JSON format property test report
#[allow(clippy::too_many_arguments)]
fn write_json_property_report(
    path: &Path,
    output: Option<&Path>,
    success: bool,
    total_tests: usize,
    passed: usize,
    failed: usize,
    cases: usize,
    deterministic: bool,
    test_results: &[String],
) -> Result<()> {
    let report = serde_json::json!({
        "status": if success { "passed" } else { "failed" },
        "file": path.display().to_string(),
        "total_tests": total_tests,
        "passed": passed,
        "failed": failed,
        "properties": {
            "no_panic": { "iterations": cases, "passed": cases - (test_results.len()) },
            "deterministic": deterministic
        },
        "failures": test_results
    });
    let json_output = serde_json::to_string_pretty(&report)?;

    if let Some(out_path) = output {
        super::write_file_with_context(out_path, json_output.as_bytes())?;
    } else {
        println!("{}", json_output);
    }
    Ok(())
}

/// Write text format property test report
#[allow(clippy::too_many_arguments)]
fn write_text_property_report(
    path: &Path,
    output: Option<&Path>,
    success: bool,
    total_tests: usize,
    passed: usize,
    failed: usize,
    cases: usize,
    deterministic: bool,
    test_results: &[String],
) -> Result<()> {
    println!("Property Test Report");
    println!("====================");
    println!("File: {}", path.display());
    println!("Status: {}", if success { "PASSED" } else { "FAILED" });
    println!("Total tests: {}", total_tests);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("\nProperties Tested:");
    println!("  1. No panics: {} iterations", cases);
    println!(
        "  2. Deterministic output: {}",
        if deterministic { "YES" } else { "NO" }
    );

    if !test_results.is_empty() {
        println!("\nFailures:");
        for failure in test_results {
            println!("  - {}", failure);
        }
    }

    if let Some(out_path) = output {
        let report = format!(
            "Property Test Report\nFile: {}\nPassed: {}/{}\n",
            path.display(),
            passed,
            total_tests
        );
        fs::write(out_path, report)?;
    }

    Ok(())
}

/// Handle property tests for a single file
fn handle_property_tests_single_file(
    path: &Path,
    cases: usize,
    format: &str,
    output: Option<&Path>,
    _seed: Option<u64>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        eprintln!(
            "Generating property tests for single file: {}",
            path.display()
        );
    }

    let binary_path = match compile_for_property_testing(path, verbose) {
        Ok(bp) => bp,
        Err(e) => {
            anyhow::bail!("{}: {}", path.display(), e);
        }
    };

    let (panic_passed, mut test_results) = run_panic_property_tests(&binary_path, cases, verbose)?;
    let deterministic = test_output_determinism(&binary_path, verbose)?;

    let passed = panic_passed + usize::from(deterministic);
    let failed = (cases - panic_passed) + usize::from(!deterministic);

    if !deterministic {
        test_results.push("Determinism test: FAILED - outputs differ".to_string());
    }

    let _ = fs::remove_file(&binary_path);

    generate_property_test_report(
        path,
        format,
        output,
        cases,
        passed,
        failed,
        deterministic,
        &test_results,
    )?;

    if failed == 0 {
        Ok(())
    } else {
        anyhow::bail!(
            "Property tests failed: {}/{} tests passed",
            passed,
            cases + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_handle_property_tests_nonexistent() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_property_tests_command(path, 10, "text", None, None, false);
        assert!(result.is_err());
    }

    // ===== EXTREME TDD Round 144 - Property Tests Handler =====

    #[test]
    fn test_handle_property_tests_nonexistent_verbose() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_property_tests_command(path, 10, "text", None, None, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_property_tests_json_format() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_property_tests_command(path, 10, "json", None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_write_property_test_json_basic() {
        let result = write_property_test_json(None, 100, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_property_test_json_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_property_test_json(Some(temp.path()), 50, "data");
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("50"));
        assert!(content.contains("passed"));
    }

    #[test]
    fn test_write_property_test_text_basic() {
        let result = write_property_test_text(None, 100, "test output");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_property_test_text_to_file() {
        let temp = NamedTempFile::new().unwrap();
        let result = write_property_test_text(Some(temp.path()), 50, "data");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_property_test_summary_json() {
        let result = write_property_test_summary("json", None, 100, "out");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_property_test_summary_text() {
        let result = write_property_test_summary("text", None, 100, "out");
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_property_report_success() {
        let path = Path::new("/test/file.ruchy");
        let result = write_json_property_report(path, None, true, 10, 10, 0, 100, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_property_report_failure() {
        let path = Path::new("/test/file.ruchy");
        let failures = vec!["Test failed".to_string()];
        let result = write_json_property_report(path, None, false, 10, 8, 2, 100, false, &failures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_json_property_report_to_file() {
        let path = Path::new("/test/file.ruchy");
        let temp = NamedTempFile::new().unwrap();
        let result =
            write_json_property_report(path, Some(temp.path()), true, 5, 5, 0, 50, true, &[]);
        assert!(result.is_ok());
        let content = std::fs::read_to_string(temp.path()).unwrap();
        assert!(content.contains("passed"));
    }

    #[test]
    fn test_write_text_property_report_success() {
        let path = Path::new("/test/file.ruchy");
        let result = write_text_property_report(path, None, true, 10, 10, 0, 100, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_text_property_report_failure() {
        let path = Path::new("/test/file.ruchy");
        let failures = vec![
            "Panic at iteration 5".to_string(),
            "Determinism failed".to_string(),
        ];
        let result = write_text_property_report(path, None, false, 10, 8, 2, 100, false, &failures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_write_text_property_report_to_file() {
        let path = Path::new("/test/file.ruchy");
        let temp = NamedTempFile::new().unwrap();
        let result =
            write_text_property_report(path, Some(temp.path()), true, 5, 5, 0, 50, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_property_test_report_json() {
        let path = Path::new("/test/file.ruchy");
        let result = generate_property_test_report(path, "json", None, 100, 100, 0, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_property_test_report_text() {
        let path = Path::new("/test/file.ruchy");
        let result = generate_property_test_report(path, "text", None, 100, 100, 0, true, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_property_test_report_with_failures() {
        let path = Path::new("/test/file.ruchy");
        let failures = vec!["Failure 1".to_string()];
        let result =
            generate_property_test_report(path, "text", None, 100, 95, 5, false, &failures);
        assert!(result.is_ok());
    }

    #[test]
    fn test_property_tests_with_seed() {
        let path = Path::new("/nonexistent/file.ruchy");
        let result = handle_property_tests_command(path, 10, "text", None, Some(12345), false);
        assert!(result.is_err()); // File doesn't exist
    }

    #[test]
    fn test_property_tests_with_different_cases() {
        let path = Path::new("/nonexistent/file.ruchy");
        // Test with 1 case
        let result = handle_property_tests_command(path, 1, "text", None, None, false);
        assert!(result.is_err());
        // Test with many cases
        let result = handle_property_tests_command(path, 1000, "text", None, None, false);
        assert!(result.is_err());
    }
}
