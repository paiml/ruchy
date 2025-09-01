//! Test for v1.29.0 coverage threshold regression bug
//! GitHub issue #12: Threshold detection broken

use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_coverage_threshold_100_percent() {
    // Create a simple Ruchy program that should have 100% coverage
    let mut temp_file = NamedTempFile::with_suffix(".ruchy").expect("Failed to create temp file");
    writeln!(temp_file, r#"println("Hello, World!")"#).expect("Failed to write to temp file");
    
    // Run coverage with 100% threshold
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "test", "--coverage"])
        .arg(temp_file.path())
        .arg("--threshold")
        .arg("100")
        .output()
        .expect("Failed to run ruchy test command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT: {stdout}");
    println!("STDERR: {stderr}");
    
    // The bug: it should report "Coverage meets threshold of 100.0%" 
    // but instead reports "Coverage meets threshold of 70.0%"
    
    // This test will FAIL initially (reproducing the bug)
    assert!(
        stdout.contains("Coverage meets threshold of 100.0%"),
        "Expected to see 100.0% threshold but got: {stdout}{stderr}"
    );
    
    // Also verify the command succeeded
    assert!(output.status.success(), "Command should succeed");
}

#[test]
fn test_coverage_threshold_80_percent() {
    // Test another threshold value to verify the bug affects all thresholds
    let mut temp_file = NamedTempFile::with_suffix(".ruchy").expect("Failed to create temp file");
    writeln!(temp_file, r#"
let score = 85
let grade = if score >= 90 {{ "A" }} else if score >= 80 {{ "B" }} else {{ "C" }}
println(f"Grade: {{grade}}")
    "#).expect("Failed to write to temp file");
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "test", "--coverage"])
        .arg(temp_file.path())
        .arg("--threshold")
        .arg("80")
        .output()
        .expect("Failed to run ruchy test command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should report 80.0% threshold, not 70.0%
    assert!(
        stdout.contains("Coverage meets threshold of 80.0%"),
        "Expected 80.0% threshold but got: {stdout}"
    );
}

#[test] 
fn test_coverage_threshold_default_behavior() {
    // Test without threshold to ensure default behavior works
    let mut temp_file = NamedTempFile::with_suffix(".ruchy").expect("Failed to create temp file");
    writeln!(temp_file, r#"println("Test without threshold")"#).expect("Failed to write to temp file");
    
    let output = Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "test", "--coverage"])
        .arg(temp_file.path())
        // No --threshold argument
        .output()
        .expect("Failed to run ruchy test command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Without threshold, should not mention threshold checking
    assert!(
        !stdout.contains("Coverage meets threshold"),
        "Should not mention threshold when none specified: {stdout}"
    );
    
    // Should still show coverage report
    assert!(
        stdout.contains("Coverage:") || stdout.contains('%'),
        "Should show coverage information: {stdout}"
    );
}