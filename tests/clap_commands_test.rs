//! TDD test for all clap commands - ensure every command is accessible
//! This prevents regressions like the Coverage command not showing up

use std::process::Command;

#[test]
fn test_help_shows_all_commands() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "--help"])
        .output()
        .expect("Failed to run ruchy --help");
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    
    // Every command MUST appear in help
    let required_commands = vec![
        "repl", "parse", "transpile", "run", "compile", "check", "test",
        "coverage",  // The one that was missing!
        "ast", "provability", "runtime", "score", "quality-gate",
        "fmt", "lint", "prove", "doc", "bench", "add", "publish",
        "mcp", "optimize", "actor:observe", "dataflow:debug", "wasm"
    ];
    
    for cmd in &required_commands {
        assert!(
            help_text.contains(cmd),
            "Command '{}' not found in help output. Help text:\n{}",
            cmd,
            help_text
        );
    }
}

#[test]
fn test_coverage_command_exists() {
    // Specific test for the Coverage command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "coverage", "--help"])
        .output()
        .expect("Failed to run ruchy coverage --help");
    
    assert!(
        output.status.success(),
        "Coverage command should exist and show help. Got stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    
    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("Generate coverage report"));
    assert!(help_text.contains("--threshold"));
    assert!(help_text.contains("--format"));
}

#[test]
fn test_all_commands_have_help() {
    let commands = vec![
        "repl", "parse", "transpile", "run", "compile", "check", "test",
        "coverage", "ast", "provability", "runtime", "score", "quality-gate",
        "fmt", "lint", "prove", "doc", "bench", "add", "publish",
        "mcp", "optimize", "actor:observe", "dataflow:debug", "wasm"
    ];
    
    for cmd in &commands {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "--", cmd, "--help"])
            .output()
            .expect(&format!("Failed to run ruchy {} --help", cmd));
        
        assert!(
            output.status.success(),
            "Command '{}' should have --help. Status: {}, stderr: {}",
            cmd,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_coverage_command_with_file() {
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    // Create a test file
    let mut temp_file = NamedTempFile::with_suffix(".ruchy").expect("Failed to create temp file");
    writeln!(temp_file, r#"println("Coverage test")"#).expect("Failed to write to temp file");
    temp_file.flush().expect("Failed to flush");
    
    // Run coverage command
    let output = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "coverage"])
        .arg(temp_file.path())
        .output()
        .expect("Failed to run ruchy coverage");
    
    // Coverage command should work
    assert!(
        output.status.success(),
        "Coverage command should work. Status: {}, stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Coverage") || stdout.contains("%"),
        "Coverage output should contain coverage information. Got: {}",
        stdout
    );
}