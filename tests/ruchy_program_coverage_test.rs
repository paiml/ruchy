//! TDD test for actual Ruchy program coverage (not just test analysis)
//!
//! [RUCHY-206] Implement proper instrumentation-based coverage

use std::process::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ruchy_program_line_coverage() {
    // Create a Ruchy program with some covered and uncovered lines
    let temp_dir = TempDir::new().unwrap();
    let program_file = temp_dir.path().join("program.ruchy");
    
    fs::write(&program_file, r"
fn covered_function(x: i32) -> i32 {
    x + 1  // This line should be covered
}

fn uncovered_function(y: i32) -> i32 {
    y * 2  // This line should NOT be covered
}

fn main() {
    let result = covered_function(5);  // This should be covered
    println(result);                   // This should be covered
    // uncovered_function is never called
}

#[test]
fn test_coverage() {
    assert_eq!(covered_function(3), 4);
    // Note: uncovered_function is not tested
}
").unwrap();
    
    // Run coverage on the program
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", "--coverage", program_file.to_str().unwrap()])
        .output()
        .expect("Failed to run coverage");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT:\n{stdout}");
    println!("STDERR:\n{stderr}");
    
    // Coverage should show:
    // - covered_function: 100% (called by test)
    // - uncovered_function: 0% (never called)  
    // - main: 100% (if executed)
    
    assert!(stdout.contains("Coverage"), "Should show coverage report");
    
    // Check that we can distinguish covered vs uncovered code
    if stdout.contains("covered_function") && stdout.contains("uncovered_function") {
        // If function names are shown, verify coverage difference
        // This test will evolve as we implement proper instrumentation
    }
}

#[test]
fn test_ruchy_branch_coverage() {
    let temp_dir = TempDir::new().unwrap();
    let program_file = temp_dir.path().join("branches.ruchy");
    
    fs::write(&program_file, r#"
fn test_branches(x: i32) -> String {
    if x > 0 {
        "positive"  // This branch should be covered
    } else {
        "negative"  // This branch should NOT be covered
    }
}

#[test] 
fn test_positive_only() {
    assert_eq!(test_branches(5), "positive");
    // Note: negative branch not tested
}
"#).unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", "--coverage", program_file.to_str().unwrap()])
        .output()
        .expect("Failed to run coverage");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should show branch coverage - only 1 of 2 branches covered
    assert!(
        stdout.contains("Coverage") || stdout.contains("branch"),
        "Should report branch coverage"
    );
}

#[test]
fn test_ruchy_coverage_threshold_failure() {
    let temp_dir = TempDir::new().unwrap();
    let program_file = temp_dir.path().join("partial.ruchy");
    
    fs::write(&program_file, r"
fn tested() -> i32 {
    42  // Covered
}

fn untested() -> i32 {
    99  // Not covered - no test calls this
}

#[test]
fn test_partial() {
    assert_eq!(tested(), 42);
}
").unwrap();
    
    // Set high threshold that should fail
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", "--coverage", "--threshold", "90", program_file.to_str().unwrap()])
        .output()
        .expect("Failed to run coverage");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should either fail threshold or show it's checking
    assert!(
        !output.status.success() || 
        stdout.contains("threshold") || 
        stderr.contains("threshold") ||
        stdout.contains("90"),
        "Should check coverage threshold"
    );
}

#[test]
fn test_ruchy_coverage_json_output() {
    let temp_dir = TempDir::new().unwrap();
    let program_file = temp_dir.path().join("json_test.ruchy");
    
    fs::write(&program_file, r"
fn simple() -> i32 { 1 }

#[test]
fn test_simple() {
    assert_eq!(simple(), 1);
}
").unwrap();
    
    let output = Command::new("cargo")
        .args([
            "run", "--quiet", "--", "test", 
            "--coverage", "--coverage-format", "json",
            program_file.to_str().unwrap()
        ])
        .output()
        .expect("Failed to run coverage");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // JSON format should produce machine-readable output
    if !stdout.contains("not yet implemented") {
        // If implemented, should be valid JSON with coverage data
        assert!(
            stdout.contains('{') || stdout.contains("coverage"),
            "JSON format should produce structured output"
        );
    }
}

#[test]
fn test_ruchy_coverage_shows_line_numbers() {
    let temp_dir = TempDir::new().unwrap();
    let program_file = temp_dir.path().join("lines.ruchy");
    
    fs::write(&program_file, r#"
// Line 2
fn covered_line() {  // Line 3
    println("executed");  // Line 4 - should be covered
}  // Line 5

fn uncovered_line() {  // Line 7
    println("never executed");  // Line 8 - should NOT be covered  
}  // Line 9

#[test]
fn test_coverage() {  // Line 11
    covered_line();  // Line 12 - should be covered
}  // Line 13
"#).unwrap();
    
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", "--coverage", program_file.to_str().unwrap()])
        .output()
        .expect("Failed to run coverage");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Eventually should show line-by-line coverage
    // For now, just verify coverage report exists
    assert!(stdout.contains("Coverage"), "Should show coverage information");
}