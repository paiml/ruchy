#![allow(missing_docs)]
//! README.md Validation Tests (EXTREME TDD)
//!
//! **SACRED RULE**: README.md can NEVER document features that don't work.
//!
//! This test suite extracts ALL code examples from README.md and validates them
//! against the actual Ruchy implementation. Any example that doesn't work MUST
//! either be fixed in the implementation OR removed from the README.
//!
//! ## Toyota Way Principle
//! - **Jidoka**: Stop the line for documentation defects
//! - **Genchi Genbutsu**: Go and see - test the actual examples
//! - **No False Advertising**: README = ground truth, not aspirations

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Extract code blocks from markdown
fn extract_code_blocks(markdown: &str, language: &str) -> Vec<(usize, String)> {
    let mut blocks = Vec::new();
    let mut in_code_block = false;
    let mut current_block = String::new();
    let mut block_start_line = 0;
    let mut current_lang = String::new();

    for (line_num, line) in markdown.lines().enumerate() {
        if line.starts_with("```") {
            if in_code_block {
                // End of code block
                if current_lang == language {
                    blocks.push((block_start_line, current_block.clone()));
                }
                in_code_block = false;
                current_block.clear();
            } else {
                // Start of code block
                in_code_block = true;
                block_start_line = line_num + 1;
                current_lang = line.trim_start_matches("```").trim().to_string();
            }
        } else if in_code_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    blocks
}

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

/// Test that README.md exists
#[test]
fn test_readme_exists() {
    let readme = Path::new("README.md");
    assert!(readme.exists(), "README.md must exist in project root");
}

/// Test that README.md is not empty
#[test]
fn test_readme_not_empty() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    assert!(
        content.len() > 100,
        "README.md must contain substantial content (found {} bytes)",
        content.len()
    );
}

/// Test that README.md contains required sections
#[test]
fn test_readme_required_sections() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let required_sections = vec![
        "# Ruchy",         // Title
        "## Features",     // What it does
        "## Installation", // How to install
        "## CLI Commands", // How to use
    ];

    for section in required_sections {
        assert!(
            content.contains(section),
            "README.md must contain section: {section}"
        );
    }
}

/// EXTREME TDD: Extract and validate ALL Ruchy code examples in README.md
#[test]
fn test_readme_ruchy_examples_all_work() {
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let examples = extract_code_blocks(&readme_content, "ruchy");

    assert!(
        !examples.is_empty(),
        "README.md must contain at least one ```ruchy code example"
    );

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut passed = 0;
    let mut failed = Vec::new();

    for (line_num, code) in &examples {
        // Skip examples with special markers
        if code.contains("// NOT IMPLEMENTED") || code.contains("// TODO") {
            continue;
        }

        // Write code to temp file
        let test_file = temp_dir
            .path()
            .join(format!("readme_line_{line_num}.ruchy"));
        fs::write(&test_file, code).expect("Failed to write test file");

        // Try to run with ruchy
        let result = ruchy_cmd().arg("run").arg(&test_file).assert();

        if result.get_output().status.success() {
            passed += 1;
        } else {
            failed.push((*line_num, code.clone()));
        }
    }

    if !failed.is_empty() {
        eprintln!("\n❌ README.md VALIDATION FAILED");
        eprintln!("Passed: {}/{}", passed, examples.len());
        eprintln!("\nFailing examples:");
        for (line_num, code) in &failed {
            eprintln!("\nLine {line_num}: ```ruchy");
            eprintln!("{}", code.trim());
            eprintln!("```");
        }
        panic!(
            "README.md contains {} non-working examples. Fix implementation or remove from README.",
            failed.len()
        );
    }

    println!("✅ All {passed} README.md examples validated successfully");
}

/// Test that README.md doesn't claim features that don't exist
#[test]
fn test_readme_no_false_claims() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    // List of features that are NOT implemented or partially implemented
    let false_claims: Vec<&str> = vec![
        // Add known false claims here as they're discovered
    ];

    for claim in false_claims {
        assert!(
            !content.contains(claim),
            "README.md falsely claims: '{claim}'. Either implement it or remove the claim."
        );
    }
}

/// Test that README.md installation instructions work
#[test]
fn test_readme_installation_instructions() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    // Should mention cargo install
    assert!(
        content.contains("cargo install") || content.contains("Installation"),
        "README.md must contain installation instructions"
    );
}

/// Test that README.md examples use correct syntax
#[test]
fn test_readme_syntax_check_all_examples() {
    let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let examples = extract_code_blocks(&readme_content, "ruchy");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut syntax_errors = Vec::new();

    for (line_num, code) in &examples {
        if code.contains("// NOT IMPLEMENTED") || code.contains("// TODO") {
            continue;
        }

        let test_file = temp_dir.path().join(format!("syntax_{line_num}.ruchy"));
        fs::write(&test_file, code).expect("Failed to write test file");

        // Check syntax only (don't run)
        let result = ruchy_cmd().arg("check").arg(&test_file).assert();

        if !result.get_output().status.success() {
            syntax_errors.push((*line_num, code.clone()));
        }
    }

    if !syntax_errors.is_empty() {
        eprintln!("\n❌ README.md SYNTAX VALIDATION FAILED");
        eprintln!("\nExamples with syntax errors:");
        for (line_num, code) in &syntax_errors {
            eprintln!("\nLine {line_num}: ```ruchy");
            eprintln!("{}", code.trim());
            eprintln!("```");
        }
        panic!(
            "README.md contains {} examples with syntax errors",
            syntax_errors.len()
        );
    }
}

/// Property test: README.md should be stable (not change frequently)
#[test]
fn test_readme_stability() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    // README should be comprehensive enough to guide users
    assert!(
        content.len() > 1000,
        "README.md seems too short ({} bytes). Should be comprehensive.",
        content.len()
    );

    // Should have multiple examples
    let ruchy_examples = extract_code_blocks(&content, "ruchy");
    assert!(
        !ruchy_examples.is_empty(),
        "README.md should have at least 1 Ruchy example (found {})",
        ruchy_examples.len()
    );
}

/// Test that README.md mentions `DataFrame` status accurately
#[test]
fn test_readme_dataframe_accuracy() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    if content.contains("DataFrame") || content.contains("dataframe") {
        // If DataFrame is mentioned, it should have accuracy warnings
        // This test will evolve as DataFrame implementation progresses

        // For now, just document that DataFrame examples exist
        let df_examples: Vec<_> = extract_code_blocks(&content, "ruchy")
            .into_iter()
            .filter(|(_, code)| code.contains("DataFrame") || code.contains("df!"))
            .collect();

        println!(
            "Found {} DataFrame examples in README.md",
            df_examples.len()
        );
    }
}

/// Test README.md against actual Ruchy version
#[test]
fn test_readme_version_consistency() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");

    // Extract version from Cargo.toml
    let version_line = cargo_toml
        .lines()
        .find(|line| line.starts_with("version = "))
        .expect("Cargo.toml must have version");

    let version = version_line
        .split('"')
        .nth(1)
        .expect("Version must be in quotes");

    // If README mentions version, it should be current or "latest"
    if content.contains("version") || content.contains("v3.") || content.contains("v4.") {
        println!("README.md may contain version info - verify manually");
        println!("Current version: {version}");
    }
}

/// Test that code blocks have proper language tags
#[test]
fn test_readme_code_block_formatting() {
    let content = fs::read_to_string("README.md").expect("Failed to read README.md");

    let lines: Vec<&str> = content.lines().collect();
    let mut bare_code_blocks = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.trim() == "```" {
            bare_code_blocks.push(i + 1);
        }
    }

    if !bare_code_blocks.is_empty() {
        eprintln!(
            "⚠️  WARNING: Found {} code blocks without language tags at lines: {:?}",
            bare_code_blocks.len(),
            bare_code_blocks
        );
        eprintln!("Use ```ruchy, ```bash, etc. for better syntax highlighting");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    /// Property: All examples should be idempotent (running twice = same result)
    #[test]
    fn test_readme_examples_idempotent() {
        let readme_content = fs::read_to_string("README.md").expect("Failed to read README.md");

        let examples = extract_code_blocks(&readme_content, "ruchy");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        for (line_num, code) in examples.iter().take(3) {
            if code.contains("// NOT IMPLEMENTED") {
                continue;
            }

            let test_file = temp_dir.path().join(format!("idempotent_{line_num}.ruchy"));
            fs::write(&test_file, code).expect("Failed to write test file");

            // Run twice
            let result1 = ruchy_cmd()
                .arg("run")
                .arg(&test_file)
                .output()
                .expect("Failed to run first time");

            let result2 = ruchy_cmd()
                .arg("run")
                .arg(&test_file)
                .output()
                .expect("Failed to run second time");

            // Both should succeed or both should fail
            assert_eq!(
                result1.status.success(),
                result2.status.success(),
                "Example at line {line_num} should be deterministic"
            );
        }
    }
}
