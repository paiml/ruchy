//! TDD Test for ruchy prove subcommand (ECOSYSTEM-003)
//! 
//! Tests that the ruchy prove command works as expected for formal verification

use std::process::Command;
use tempfile::NamedTempFile;
use std::fs;

#[test]
fn test_ruchy_prove_command_basic_functionality() {
    // RED: Write test first - expect it to fail initially
    
    // Create a simple Ruchy file with a property to prove
    let prove_file = NamedTempFile::with_suffix(".ruchy").expect("Create temp file");
    fs::write(prove_file.path(), r#"
// Simple property to prove: addition is commutative
fun add_commutative(a: i32, b: i32) -> bool {
    a + b == b + a
}

// Property assertion
assert add_commutative(3, 5)
assert add_commutative(10, 20)
"#).expect("Write test file");
    
    // TDD: Test the prove command with --check flag (non-interactive)
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("prove")
        .arg(prove_file.path())
        .arg("--check")
        .arg("--backend")
        .arg("z3")
        .output()
        .expect("Execute ruchy prove command");
    
    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));
    
    // Expected behavior: Should verify the simple properties
    assert!(output.status.success(), "Prove command should succeed for simple properties");
    
    let stdout = String::from_utf8(output.stdout).expect("Valid UTF-8 stdout");
    assert!(stdout.contains("✅") || stdout.contains("Proof successful") || stdout.contains("verified"), 
            "Should indicate successful proof verification: {}", stdout);
}

#[test]
fn test_ruchy_prove_command_json_output() {
    // TDD: Test JSON output format for automation
    
    let prove_file = NamedTempFile::with_suffix(".ruchy").expect("Create temp file");
    fs::write(prove_file.path(), r#"
// Simple tautology
fun always_true() -> bool {
    true
}

assert always_true()
"#).expect("Write test file");
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("prove")
        .arg(prove_file.path())
        .arg("--check")
        .arg("--format")
        .arg("json")
        .output()
        .expect("Execute ruchy prove with JSON output");
    
    assert!(output.status.success(), "Prove command should succeed");
    
    let stdout = String::from_utf8(output.stdout).expect("Valid UTF-8 stdout");
    
    // Should output valid JSON
    let json_result: serde_json::Value = serde_json::from_str(&stdout)
        .expect(&format!("Should output valid JSON: {}", stdout));
    
    // Should have proof results
    assert!(json_result.get("proofs").is_some() || 
            json_result.get("verified").is_some() ||
            json_result.get("status").is_some(),
            "JSON should contain proof results: {}", stdout);
}

#[test] 
fn test_ruchy_prove_command_with_counterexample() {
    // TDD: Test counterexample generation for false properties
    
    let prove_file = NamedTempFile::with_suffix(".ruchy").expect("Create temp file");
    fs::write(prove_file.path(), r#"
// False property that should generate counterexample
fun false_property(x: i32) -> bool {
    x > 0 && x < 0  // This is always false
}

assert false_property(5)  // This should fail
"#).expect("Write test file");
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("prove") 
        .arg(prove_file.path())
        .arg("--check")
        .arg("--counterexample")
        .output()
        .expect("Execute ruchy prove with counterexample");
    
    // Should fail but provide counterexample
    let stdout = String::from_utf8(output.stdout).expect("Valid UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr).expect("Valid UTF-8 stderr");
    
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    // Should indicate proof failure and provide counterexample
    assert!(stdout.contains("❌") || stdout.contains("Proof failed") || 
            stderr.contains("counterexample") || stdout.contains("counterexample"),
            "Should indicate proof failure with counterexample: stdout={}, stderr={}", stdout, stderr);
}

#[test]
fn test_ruchy_prove_command_timeout() {
    // TDD: Test timeout functionality
    
    let prove_file = NamedTempFile::with_suffix(".ruchy").expect("Create temp file");
    fs::write(prove_file.path(), r#"
// Complex property that might timeout
fun complex_property(n: i32) -> bool {
    // Recursive fibonacci-like property that's hard to verify
    n <= 0 || (complex_property(n-1) && complex_property(n-2))
}

assert complex_property(10)
"#).expect("Write test file");
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("prove")
        .arg(prove_file.path()) 
        .arg("--check")
        .arg("--timeout")
        .arg("100")  // Very short timeout
        .output()
        .expect("Execute ruchy prove with timeout");
    
    let stdout = String::from_utf8(output.stdout).expect("Valid UTF-8 stdout");
    let stderr = String::from_utf8(output.stderr).expect("Valid UTF-8 stderr");
    
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    // Should either succeed quickly or timeout gracefully
    assert!(output.status.success() || 
            stdout.contains("timeout") || stderr.contains("timeout") ||
            stdout.contains("⏰") || stderr.contains("⏰"),
            "Should handle timeout gracefully: stdout={}, stderr={}", stdout, stderr);
}

#[test]
fn test_ruchy_prove_command_help() {
    // TDD: Test that help works (basic sanity check)
    
    let output = Command::new(env!("CARGO_BIN_EXE_ruchy"))
        .arg("prove")
        .arg("--help")
        .output()
        .expect("Execute ruchy prove --help");
    
    assert!(output.status.success(), "Help should work");
    
    let stdout = String::from_utf8(output.stdout).expect("Valid UTF-8 stdout");
    
    // Should contain expected help content
    assert!(stdout.contains("prove"), "Help should mention 'prove'");
    assert!(stdout.contains("SMT"), "Help should mention SMT backend");
    assert!(stdout.contains("theorem"), "Help should mention theorem proving");
}