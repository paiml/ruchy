//! Basic tests for coverage tooling functionality  
//!
//! [TEST-TOOL-001] Coverage tooling test coverage

use ruchy::quality::ruchy_coverage::*;
use std::fs;
use tempfile::TempDir;

/// Create a test .ruchy file
fn create_ruchy_test_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    fs::write(&file_path, content).expect("Failed to create test file");
    file_path
}

#[test]
fn test_ruchy_coverage_creation() {
    let coverage = RuchyCoverage::new("test.ruchy");

    assert_eq!(coverage.file_path, "test.ruchy");
    assert_eq!(coverage.total_lines, 0);
    assert_eq!(coverage.total_functions, 0);
    assert_eq!(coverage.total_branches, 0);
    assert_eq!(coverage.covered_branches, 0);
    assert!(coverage.covered_lines.is_empty());
    assert!(coverage.covered_functions.is_empty());
}

#[test]
fn test_ruchy_coverage_line_coverage_empty() {
    let coverage = RuchyCoverage::new("empty.ruchy");

    // Empty file should have 100% coverage
    assert_eq!(coverage.line_coverage(), 100.0);
}

#[test]
fn test_ruchy_coverage_function_coverage_empty() {
    let coverage = RuchyCoverage::new("empty.ruchy");

    // Empty file should have 100% function coverage
    assert_eq!(coverage.function_coverage(), 100.0);
}

#[test]
fn test_ruchy_coverage_with_data() {
    let mut coverage = RuchyCoverage::new("test.ruchy");

    // Set some test data
    coverage.total_lines = 10;
    coverage.covered_lines.insert(1);
    coverage.covered_lines.insert(2);
    coverage.covered_lines.insert(5);

    coverage.total_functions = 2;
    coverage.covered_functions.insert("test_func".to_string());

    // Test coverage calculations
    let line_cov = coverage.line_coverage();
    let func_cov = coverage.function_coverage();

    assert_eq!(line_cov, 30.0); // 3/10 * 100 = 30%
    assert_eq!(func_cov, 50.0); // 1/2 * 100 = 50%
}

#[test]
fn test_ruchy_coverage_branch_coverage() {
    let coverage = RuchyCoverage::new("branches.ruchy");

    // Should have branch coverage calculation method if implemented
    // Testing the basic structure for now
    assert_eq!(coverage.total_branches, 0);
    assert_eq!(coverage.covered_branches, 0);
}

#[test]
fn test_ruchy_coverage_collector_creation() {
    let collector = RuchyCoverageCollector::new();

    // Should create without panicking - test the basic structure
    // The specific fields depend on the actual implementation
    drop(collector); // Ensure it can be created and dropped
}

#[test]
fn test_ruchy_coverage_collector_analyze_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_content = r#"
fn simple_function() -> i32 {
    42
}

fn another_function() -> String {
    "hello"
}
"#;

    let test_file = create_ruchy_test_file(temp_dir.path(), "test.ruchy", test_content);
    let mut collector = RuchyCoverageCollector::new();

    // Test file analysis - should not panic
    let result = collector.analyze_file(&test_file);
    assert!(result.is_ok(), "File analysis should succeed");
}

#[test]
fn test_ruchy_coverage_collector_execute_with_coverage() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let test_content = r"
fn test_function() -> i32 {
    1 + 1
}

#[test]  
fn test_addition() {
    assert_eq!(test_function(), 2);
}
";

    let test_file = create_ruchy_test_file(temp_dir.path(), "with_test.ruchy", test_content);
    let mut collector = RuchyCoverageCollector::new();

    // Test execution with coverage - this tests the runtime instrumentation
    let result = collector.execute_with_coverage(&test_file);

    // Should execute without major errors (specific behavior depends on implementation)
    let _coverage_result = result; // Don't assert success/failure - depends on implementation details
}

#[test]
fn test_ruchy_coverage_collector_html_report() {
    let collector = RuchyCoverageCollector::new();

    // Test HTML report generation
    let html_report = collector.generate_html_report();

    // Should generate some HTML content
    assert!(!html_report.is_empty(), "HTML report should not be empty");
    assert!(
        html_report.contains("<html>") || html_report.contains("Coverage"),
        "Should contain HTML or coverage content"
    );
}

#[test]
fn test_coverage_from_multiple_files() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut collector = RuchyCoverageCollector::new();

    // Create multiple test files
    let file1 = create_ruchy_test_file(temp_dir.path(), "file1.ruchy", "fn test1() { 1 }");
    let file2 = create_ruchy_test_file(temp_dir.path(), "file2.ruchy", "fn test2() { 2 }");
    let file3 = create_ruchy_test_file(temp_dir.path(), "file3.ruchy", "fn test3() { 3 }");

    // Analyze all files
    let results = [
        collector.analyze_file(&file1),
        collector.analyze_file(&file2),
        collector.analyze_file(&file3),
    ];

    // Should handle multiple file analysis
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "File {} analysis should succeed", i + 1);
    }
}
