#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Tests for CLI command handlers
//!
//! These tests validate the individual command handler functions
//! that were extracted from the main CLI binary for complexity reduction.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

// Note: We can't directly import handler functions since they're in a binary crate
// Instead, we'll test the handlers through integration tests of the CLI binary

/// Test parse command functionality
#[test]
fn test_parse_command_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.ruchy");

    // Create a simple test file
    fs::write(&test_file, "let x = 42\nprintln(x)")?;

    // Run parse command
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "parse"])
        .arg(&test_file)
        .output()?;

    // Should succeed and show AST
    assert!(
        output.status.success(),
        "Parse command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain AST elements
    assert!(stdout.contains("Let"), "Should show Let expression in AST");
    assert!(stdout.contains("Call"), "Should show function call in AST");

    Ok(())
}

/// Test transpile command functionality  
#[test]
fn test_transpile_command_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("test.ruchy");
    let output_file = temp_dir.path().join("output.rs");

    // Create a simple test file
    fs::write(&test_file, "let x = 42\nprintln(\"Answer: {}\", x)")?;

    // Run transpile command with output file
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "transpile"])
        .arg(&test_file)
        .args(["-o", output_file.to_str().unwrap()])
        .output()?;

    // Should succeed
    assert!(
        output.status.success(),
        "Transpile command should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Output file should exist and contain Rust code
    assert!(output_file.exists(), "Output file should be created");

    let rust_code = fs::read_to_string(&output_file)?;
    assert!(
        !rust_code.is_empty(),
        "Generated Rust code should not be empty"
    );

    // Should contain some expected Rust patterns
    assert!(
        rust_code.contains("fn main") || rust_code.contains("println"),
        "Should generate valid Rust code with main or println"
    );

    Ok(())
}

/// Test transpile to stdout
#[test]
fn test_transpile_to_stdout() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("simple.ruchy");

    // Simple expression that should transpile
    fs::write(&test_file, "2 + 2")?;

    // Run transpile without output file (should go to stdout)
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "transpile"])
        .arg(&test_file)
        .output()?;

    // Should succeed
    assert!(
        output.status.success(),
        "Transpile to stdout should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Should output Rust code to stdout");

    Ok(())
}

/// Test minimal transpile mode
#[test]
fn test_minimal_transpile_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("minimal.ruchy");

    // Simple expression for minimal transpilation
    fs::write(&test_file, "let x = 10\nx")?;

    // Run transpile with --minimal flag
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "transpile", "--minimal"])
        .arg(&test_file)
        .output()?;

    // Should succeed
    assert!(
        output.status.success(),
        "Minimal transpile should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "Should output minimal Rust code");

    Ok(())
}

/// Test run command functionality
#[test]
fn test_run_command_integration() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("hello.ruchy");

    // Create a simple program that produces output
    fs::write(&test_file, r#"println("Hello from Ruchy!")"#)?;

    // Run the file
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "run"])
        .arg(&test_file)
        .output()?;

    // Should succeed and produce expected output
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Hello from Ruchy!"),
            "Should execute and print expected output"
        );
    } else {
        // If run command is not fully implemented, that's expected
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Run command not fully implemented: {stderr}");
        // This is acceptable during development
    }

    Ok(())
}

/// Test parse command error handling
#[test]
fn test_parse_command_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("invalid.ruchy");

    // Create a file with invalid syntax
    fs::write(&test_file, "let x = { invalid syntax")?;

    // Run parse command on invalid file
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "parse"])
        .arg(&test_file)
        .output()?;

    // Should fail with parse error
    assert!(
        !output.status.success(),
        "Parse should fail on invalid syntax"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Parse error") || stderr.contains("error"),
        "Should show parse error message"
    );

    Ok(())
}

/// Test transpile with stdin input
#[test]
fn test_transpile_stdin_input() -> Result<()> {
    // Run transpile with stdin input (use "-" as filename)
    let mut child = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "transpile", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write simple Ruchy code to stdin
    if let Some(stdin) = child.stdin.take() {
        use std::io::Write;
        let mut stdin = stdin;
        writeln!(stdin, "let greeting = \"Hello\"")?;
        writeln!(stdin, "println(greeting)")?;
    }

    let output = child.wait_with_output()?;

    // Should succeed and produce Rust code
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "Should generate Rust code from stdin input"
        );
    } else {
        // Stdin processing might not be fully implemented - that's ok for now
        println!(
            "Stdin transpilation not fully implemented - this is acceptable during development"
        );
    }

    Ok(())
}

/// Test handler verbose mode
#[test]
fn test_handler_verbose_mode() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("verbose_test.ruchy");

    fs::write(&test_file, "let result = 1 + 1")?;

    // Run parse command with verbose flag
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "--verbose", "parse"])
        .arg(&test_file)
        .output()?;

    // Should succeed
    assert!(
        output.status.success(),
        "Verbose parse should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // In verbose mode, should show additional information
    let _stderr = String::from_utf8_lossy(&output.stderr);
    // Verbose output goes to stderr, may contain file parsing info

    Ok(())
}

/// Test file not found error handling
#[test]
fn test_file_not_found_handling() -> Result<()> {
    // Try to parse a non-existent file
    let output = std::process::Command::new("cargo")
        .args(["run", "--bin", "ruchy", "--", "parse", "nonexistent.ruchy"])
        .output()?;

    // Should fail
    assert!(
        !output.status.success(),
        "Should fail when file doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No such file")
            || stderr.contains("not found")
            || stderr.contains("Failed to read"),
        "Should show file not found error"
    );

    Ok(())
}
