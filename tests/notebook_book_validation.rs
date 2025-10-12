//! NOTEBOOK-008 Phase 2: MD Book Integration Testing
//!
//! This test suite validates ALL code examples from the Ruchy MD Book
//! by executing them through the notebook API and verifying outputs.
//!
//! Pattern: Hybrid Testing (Toyota Way)
//! - Unit Level: Rust tests hitting API directly (fast feedback)
//! - Integration Level: Playwright tests for UI validation (realistic scenarios)
//!
//! Target: ≥90% book examples passing

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct ExecuteRequest {
    source: String,
}

#[derive(Debug, Deserialize)]
struct ExecuteResponse {
    output: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// Extract code examples from a markdown file
/// Returns: Vec<(code, expected_output)>
fn extract_examples(md_path: &Path) -> Vec<(String, Option<String>)> {
    let content = fs::read_to_string(md_path).expect("Failed to read MD file");
    let mut examples = Vec::new();
    let mut in_code_block = false;
    let mut in_expected_block = false;
    let mut current_code = String::new();
    let mut current_expected: Option<String> = None;
    let mut expecting_output_next = false;

    for line in content.lines() {
        // Check for ruchy code block start
        if line.starts_with("```ruchy") {
            in_code_block = true;
            current_code.clear();
            continue;
        }

        // Check for code block end (either ruchy or expected output)
        if line.starts_with("```") && !line.starts_with("```ruchy") {
            if in_code_block {
                // Ruchy code block ended
                in_code_block = false;
                // Don't save yet - wait to see if there's an expected output
                continue;
            } else if in_expected_block {
                // Expected output block ended
                in_expected_block = false;
                // Now save the code + expected output pair
                if !current_code.trim().is_empty() {
                    examples.push((current_code.clone(), current_expected.clone()));
                    current_code.clear();
                    current_expected = None;
                }
                continue;
            } else if expecting_output_next {
                // This is the start of the expected output block
                in_expected_block = true;
                expecting_output_next = false;
                current_expected = Some(String::new());
                continue;
            }
        }

        // Check for "Expected Output:" marker
        if line.starts_with("**Expected Output**:") {
            expecting_output_next = true;
            continue;
        }

        // If we're in a new section and have unsaved code (no expected output followed)
        if line.starts_with("###") && !current_code.trim().is_empty() {
            // Save code without expected output
            examples.push((current_code.clone(), None));
            current_code.clear();
            current_expected = None;
            expecting_output_next = false;
            continue;
        }

        // Collect code or expected output
        if in_code_block {
            current_code.push_str(line);
            current_code.push('\n');
        } else if in_expected_block {
            if let Some(ref mut expected) = current_expected {
                expected.push_str(line);
                expected.push('\n');
            }
        }
    }

    // Save any remaining code
    if !current_code.trim().is_empty() {
        examples.push((current_code, current_expected));
    }

    examples
}

/// Execute code through notebook API
fn execute_notebook_code(client: &Client, code: &str) -> Result<ExecuteResponse, reqwest::Error> {
    let request = ExecuteRequest {
        source: code.to_string(),
    };

    client
        .post("http://127.0.0.1:8080/api/execute")
        .json(&request)
        .timeout(Duration::from_secs(10))
        .send()?
        .json::<ExecuteResponse>()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper: Start notebook server and return client
    fn setup_notebook() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client")
    }

    #[test]
    fn test_extract_examples_from_literals() {
        let path = Path::new("docs/notebook/book/src/01-basic-syntax/01-literals.md");
        let examples = extract_examples(path);

        // Should extract multiple examples from literals chapter
        assert!(
            examples.len() >= 5,
            "Expected at least 5 examples, got {}",
            examples.len()
        );

        // First example should be integer literal
        assert_eq!(examples[0].0.trim(), "42");
        assert_eq!(examples[0].1.as_ref().map(|s| s.trim()), Some("42"));
    }

    #[test]
    #[ignore = "Requires running notebook server at http://127.0.0.1:8080"]
    fn test_notebook_api_integer_literal() {
        let client = setup_notebook();
        let result = execute_notebook_code(&client, "42").expect("API call failed");

        assert!(result.success, "Execution failed: {:?}", result.error);
        assert_eq!(result.output.trim(), "42");
    }

    #[test]
    #[ignore = "Requires running notebook server at http://127.0.0.1:8080"]
    fn test_notebook_api_float_literal() {
        let client = setup_notebook();
        let result = execute_notebook_code(&client, "3.14").expect("API call failed");

        assert!(result.success, "Execution failed: {:?}", result.error);
        assert_eq!(result.output.trim(), "3.14");
    }

    #[test]
    #[ignore = "Requires running notebook server at http://127.0.0.1:8080"]
    fn test_notebook_api_string_literal() {
        let client = setup_notebook();
        let result = execute_notebook_code(&client, r#""Hello, Ruchy!""#).expect("API call failed");

        assert!(result.success, "Execution failed: {:?}", result.error);
        assert_eq!(result.output.trim(), r#""Hello, Ruchy!""#);
    }

    #[test]
    #[ignore = "Requires running notebook server at http://127.0.0.1:8080"]
    fn test_notebook_api_println() {
        let client = setup_notebook();
        let result = execute_notebook_code(&client, r#"println("Test")"#).expect("API call failed");

        assert!(result.success, "Execution failed: {:?}", result.error);
        assert_eq!(result.output.trim(), r#""Test""#);
    }

    #[test]
    #[ignore = "Requires running notebook server at http://127.0.0.1:8080"]
    fn test_all_basic_syntax_literals() {
        let client = setup_notebook();
        let path = Path::new("docs/notebook/book/src/01-basic-syntax/01-literals.md");
        let examples = extract_examples(path);

        let mut passed = 0;
        let mut failed = 0;

        for (i, (code, expected)) in examples.iter().enumerate() {
            let result = match execute_notebook_code(&client, code) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Example {} failed to execute: {}", i, e);
                    failed += 1;
                    continue;
                }
            };

            if !result.success {
                eprintln!("Example {} execution error: {:?}", i, result.error);
                failed += 1;
                continue;
            }

            if let Some(expected_output) = expected {
                let actual = result.output.trim();
                let expected = expected_output.trim();

                if actual == expected {
                    passed += 1;
                } else {
                    eprintln!("Example {} output mismatch:", i);
                    eprintln!("  Code: {}", code.trim());
                    eprintln!("  Expected: {}", expected);
                    eprintln!("  Got: {}", actual);
                    failed += 1;
                }
            } else {
                // No expected output specified, just verify it didn't error
                passed += 1;
            }
        }

        let total = passed + failed;
        let pass_rate = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        println!("\nChapter: 01-literals");
        println!("  Passed: {}/{} ({:.1}%)", passed, total, pass_rate);
        println!("  Failed: {}", failed);

        // Target: ≥90% passing
        assert!(
            pass_rate >= 90.0,
            "Pass rate {:.1}% below 90% target",
            pass_rate
        );
    }
}
