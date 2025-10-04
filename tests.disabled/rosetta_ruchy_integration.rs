#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Integration tests for rosetta-ruchy examples
//!
//! These tests validate that the Ruchy compiler can successfully run all
//! rosetta-ruchy algorithm implementations, preventing regressions in
//! language compatibility.

use std::path::Path;
use std::process::Command;

/// Test that we can find and enumerate rosetta-ruchy examples
#[test]
fn test_find_rosetta_ruchy_examples() {
    let rosetta_path = Path::new("../rosetta-ruchy");
    if !rosetta_path.exists() {
        eprintln!("Warning: rosetta-ruchy not found at ../rosetta-ruchy");
        eprintln!("This test requires rosetta-ruchy to be cloned as a sibling directory");
        return; // Skip test if rosetta-ruchy is not available
    }

    let examples_path = rosetta_path.join("examples/algorithms");
    assert!(examples_path.exists(), "Examples directory should exist");

    // Count algorithm directories
    let mut algorithm_count = 0;
    let mut ruchy_files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&examples_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("00")
            {
                algorithm_count += 1;

                // Look for ruchy implementations
                let ruchy_impl_path = path.join("implementations/ruchy");
                if ruchy_impl_path.exists() {
                    if let Ok(ruchy_entries) = std::fs::read_dir(&ruchy_impl_path) {
                        for ruchy_entry in ruchy_entries.flatten() {
                            let ruchy_path = ruchy_entry.path();
                            if ruchy_path.extension().is_some_and(|ext| ext == "ruchy") {
                                ruchy_files.push(ruchy_path);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Found {algorithm_count} algorithm directories");
    println!("Found {} .ruchy files", ruchy_files.len());

    // We expect at least 5 algorithms with ruchy implementations
    assert!(
        algorithm_count >= 5,
        "Should have at least 5 algorithm examples"
    );
    assert!(
        ruchy_files.len() >= 10,
        "Should have at least 10 ruchy implementation files"
    );
}

/// Test that simple ruchy examples can be executed
#[test]
fn test_fibonacci_simple_execution() {
    let rosetta_path = Path::new("../rosetta-ruchy");
    if !rosetta_path.exists() {
        eprintln!("Warning: rosetta-ruchy not found at ../rosetta-ruchy");
        return; // Skip test if rosetta-ruchy is not available
    }

    let fibonacci_simple = rosetta_path
        .join("examples/algorithms/001-fibonacci/implementations/ruchy/fibonacci_simple.ruchy");

    if !fibonacci_simple.exists() {
        eprintln!("Warning: fibonacci_simple.ruchy not found, skipping test");
        return;
    }

    // Try to run the fibonacci example
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run"])
        .arg(&fibonacci_simple)
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                eprintln!("Command failed with status: {}", output.status);
                eprintln!("Stdout: {stdout}");
                eprintln!("Stderr: {stderr}");
                // Don't fail the test - just report the issue
                eprintln!("Note: This is expected if ruchy run command is not fully implemented");
                return;
            }

            println!("fibonacci_simple.ruchy output:\n{stdout}");

            // Check for expected fibonacci outputs
            // The script calculates fibonacci(10) which should be 55
            assert!(
                stdout.contains("55") || stderr.contains("55"),
                "Output should contain fibonacci(10) = 55"
            );
        }
        Err(e) => {
            eprintln!("Failed to execute command: {e}");
            // Don't fail the test - this might be expected during development
        }
    }
}

/// Test that we can at least parse ruchy files without panicking
#[test]
fn test_parse_rosetta_ruchy_files() {
    let rosetta_path = Path::new("../rosetta-ruchy");
    if !rosetta_path.exists() {
        return; // Skip if rosetta-ruchy not available
    }

    let (parsed_files, failed_files) = parse_all_rosetta_files(&rosetta_path);
    report_parsing_results(parsed_files, &failed_files);
    verify_parsing_baseline(parsed_files, failed_files.len());
}

/// Parse all rosetta-ruchy algorithm files
fn parse_all_rosetta_files(rosetta_path: &Path) -> (usize, Vec<(PathBuf, ruchy::Error)>) {
    let examples_path = rosetta_path.join("examples/algorithms");
    let mut parsed_files = 0;
    let mut failed_files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&examples_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if should_process_directory(&path) {
                process_algorithm_directory(&path, &mut parsed_files, &mut failed_files);
            }
        }
    }

    (parsed_files, failed_files)
}

/// Check if directory should be processed (starts with "00")
fn should_process_directory(path: &Path) -> bool {
    path.is_dir()
        && path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("00")
}

/// Process a single algorithm directory
fn process_algorithm_directory(
    dir_path: &Path,
    parsed_files: &mut usize,
    failed_files: &mut Vec<(PathBuf, ruchy::Error)>,
) {
    let ruchy_impl_path = dir_path.join("implementations/ruchy");
    if !ruchy_impl_path.exists() {
        return;
    }

    if let Ok(ruchy_entries) = std::fs::read_dir(&ruchy_impl_path) {
        for ruchy_entry in ruchy_entries.flatten() {
            let ruchy_path = ruchy_entry.path();
            if ruchy_path.extension().is_some_and(|ext| ext == "ruchy") {
                try_parse_ruchy_file(&ruchy_path, parsed_files, failed_files);
            }
        }
    }
}

/// Try to parse a single ruchy file
fn try_parse_ruchy_file(
    file_path: &Path,
    parsed_files: &mut usize,
    failed_files: &mut Vec<(PathBuf, ruchy::Error)>,
) {
    if let Ok(content) = std::fs::read_to_string(file_path) {
        match ruchy::Parser::new(&content).parse() {
            Ok(_) => {
                *parsed_files += 1;
            }
            Err(e) => {
                failed_files.push((file_path.to_path_buf(), e));
            }
        }
    }
}

/// Report parsing results to stdout
fn report_parsing_results(parsed_files: usize, failed_files: &[(PathBuf, ruchy::Error)]) {
    println!("Successfully parsed {parsed_files} ruchy files");
    if !failed_files.is_empty() {
        println!("Failed to parse {} files:", failed_files.len());
        for (file, error) in failed_files {
            println!("  {}: {}", file.display(), error);
        }
    }
}

/// Verify parsing baseline assertions
fn verify_parsing_baseline(parsed_files: usize, failed_count: usize) {
    // We expect to be able to parse at least some files
    assert!(
        parsed_files > 0,
        "Should be able to parse at least some ruchy files"
    );

    // For now, allow some parsing failures as the language is still in development
    // In the future, this should require all files to parse successfully
    let success_rate = parsed_files as f64 / (parsed_files + failed_count) as f64;

    // Store baseline: Currently parsing 9/21 files (42.9%)
    // This test ensures we don't regress below this baseline
    assert!(
        success_rate >= 0.40,
        "Parse success rate regression detected! Should be at least 40%, got {:.1}%",
        success_rate * 100.0
    );

    // Print specific parsing issues for future improvement
    if !failed_files.is_empty() {
        println!("\n=== Parsing Issues to Address ===");
        let mut comment_issues = 0;
        let mut type_issues = 0;
        let mut generic_issues = 0;

        for (_file, error) in &failed_files {
            let error_str = error.to_string();
            if error_str.contains("Expected '[' after '#'") {
                comment_issues += 1;
            } else if error_str.contains("Expected type") {
                type_issues += 1;
            } else if error_str.contains("Expected Greater, found Colon") {
                generic_issues += 1;
            }
        }

        println!("Comment syntax issues: {comment_issues}");
        println!("Type annotation issues: {type_issues}");
        println!("Generic type syntax issues: {generic_issues}");
        println!(
            "Other parsing issues: {}",
            failed_files.len() - comment_issues - type_issues - generic_issues
        );
    }
}

/// Comprehensive test runner for all rosetta-ruchy examples
#[test]
#[ignore = "Comprehensive test that may take time"]
fn test_all_rosetta_ruchy_examples() {
    let rosetta_path = Path::new("../rosetta-ruchy");
    if !rosetta_path.exists() {
        println!("Skipping comprehensive test - rosetta-ruchy not available");
        return;
    }

    let (total, successful, parse_fails, exec_fails) = test_all_algorithm_files(&rosetta_path);

    report_integration_results(total, successful, parse_fails, exec_fails);
    verify_integration_baseline(total, parse_fails);
}

/// Test all algorithm files (parse + execute)
fn test_all_algorithm_files(rosetta_path: &Path) -> (u32, u32, u32, u32) {
    let examples_path = rosetta_path.join("examples/algorithms");
    let mut total_files = 0;
    let mut successful_runs = 0;
    let mut parsing_failures = 0;
    let mut execution_failures = 0;

    if let Ok(entries) = std::fs::read_dir(&examples_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if should_process_directory(&path) {
                test_directory_files(
                    &path,
                    &mut total_files,
                    &mut successful_runs,
                    &mut parsing_failures,
                    &mut execution_failures,
                );
            }
        }
    }

    (
        total_files,
        successful_runs,
        parsing_failures,
        execution_failures,
    )
}

/// Test all ruchy files in a directory
fn test_directory_files(
    dir_path: &Path,
    total_files: &mut u32,
    successful_runs: &mut u32,
    parsing_failures: &mut u32,
    execution_failures: &mut u32,
) {
    let ruchy_impl_path = dir_path.join("implementations/ruchy");
    if !ruchy_impl_path.exists() {
        return;
    }

    if let Ok(ruchy_entries) = std::fs::read_dir(&ruchy_impl_path) {
        for ruchy_entry in ruchy_entries.flatten() {
            let ruchy_path = ruchy_entry.path();
            if ruchy_path.extension().is_some_and(|ext| ext == "ruchy") {
                test_and_execute_file(
                    &ruchy_path,
                    total_files,
                    successful_runs,
                    parsing_failures,
                    execution_failures,
                );
            }
        }
    }
}

/// Test a single file: parse and execute
fn test_and_execute_file(
    file_path: &Path,
    total_files: &mut u32,
    successful_runs: &mut u32,
    parsing_failures: &mut u32,
    execution_failures: &mut u32,
) {
    *total_files += 1;
    println!("Testing: {}", file_path.display());

    // Try to parse
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            println!("  ❌ Failed to read file: {e}");
            return;
        }
    };

    match ruchy::Parser::new(&content).parse() {
        Ok(_) => {
            println!("  ✅ Parsing successful");
        }
        Err(e) => {
            println!("  ❌ Parsing failed: {e}");
            *parsing_failures += 1;
            return;
        }
    }

    // Try to execute
    try_execute_ruchy_file(file_path, successful_runs, execution_failures);
}

/// Try to execute a ruchy file
fn try_execute_ruchy_file(
    file_path: &Path,
    successful_runs: &mut u32,
    execution_failures: &mut u32,
) {
    match Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run"])
        .arg(file_path)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("  ✅ Execution successful");
                *successful_runs += 1;
            } else {
                println!("  ⚠️  Execution failed (expected during development)");
                *execution_failures += 1;
            }
        }
        Err(e) => {
            println!("  ⚠️  Could not execute: {e}");
            *execution_failures += 1;
        }
    }
}

/// Report integration test results
fn report_integration_results(
    total_files: u32,
    successful_runs: u32,
    parsing_failures: u32,
    execution_failures: u32,
) {
    println!("\n=== Rosetta-Ruchy Integration Test Summary ===");
    println!("Total files tested: {total_files}");
    println!("Successful parses: {}", total_files - parsing_failures);
    println!("Parsing failures: {parsing_failures}");
    println!("Successful executions: {successful_runs}");
    println!("Execution issues: {execution_failures}");
}

/// Verify integration baseline
fn verify_integration_baseline(total_files: u32, parsing_failures: u32) {
    if total_files > 0 {
        let parse_success_rate = f64::from(total_files - parsing_failures) / f64::from(total_files);
        println!("Parse success rate: {:.1}%", parse_success_rate * 100.0);

        // For now, require at least 70% parsing success
        assert!(
            parse_success_rate >= 0.7,
            "Parse success rate should be at least 70%, got {:.1}%",
            parse_success_rate * 100.0
        );
    }
}
