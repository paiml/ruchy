#![allow(missing_docs)]
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

/// Parser state for markdown extraction
struct ParserState {
    in_code_block: bool,
    in_expected_block: bool,
    current_code: String,
    current_expected: Option<String>,
    expecting_output_next: bool,
}

impl ParserState {
    fn new() -> Self {
        Self {
            in_code_block: false,
            in_expected_block: false,
            current_code: String::new(),
            current_expected: None,
            expecting_output_next: false,
        }
    }

    /// Check if we have unsaved code to finalize
    fn has_pending_code(&self) -> bool {
        !self.current_code.trim().is_empty()
    }

    /// Finalize current example and reset state
    fn finalize_example(&mut self) -> Option<(String, Option<String>)> {
        if self.has_pending_code() {
            let example = (self.current_code.clone(), self.current_expected.clone());
            self.current_code.clear();
            self.current_expected = None;
            self.expecting_output_next = false;
            Some(example)
        } else {
            None
        }
    }
}

/// Extract code examples from a markdown file
/// Returns: Vec<(code, `expected_output`)>
fn extract_examples(md_path: &Path) -> Vec<(String, Option<String>)> {
    let content = fs::read_to_string(md_path).expect("Failed to read MD file");
    let mut examples = Vec::new();
    let mut state = ParserState::new();

    for line in content.lines() {
        if process_ruchy_code_start(line, &mut state) {
            continue;
        }

        if process_code_block_end(line, &mut state, &mut examples) {
            continue;
        }

        if process_expected_output_marker(line, &mut state) {
            continue;
        }

        if process_section_boundary(line, &mut state, &mut examples) {
            continue;
        }

        collect_line_content(line, &mut state);
    }

    // Save any remaining code
    if let Some(example) = state.finalize_example() {
        examples.push(example);
    }

    examples
}

/// Process ruchy code block start marker
fn process_ruchy_code_start(line: &str, state: &mut ParserState) -> bool {
    if line.starts_with("```ruchy") {
        state.in_code_block = true;
        state.current_code.clear();
        return true;
    }
    false
}

/// Process code block end markers (``` not followed by ruchy)
fn process_code_block_end(
    line: &str,
    state: &mut ParserState,
    examples: &mut Vec<(String, Option<String>)>,
) -> bool {
    if !line.starts_with("```") || line.starts_with("```ruchy") {
        return false;
    }

    if state.in_code_block {
        // Ruchy code block ended - wait for possible expected output
        state.in_code_block = false;
        return true;
    }

    if state.in_expected_block {
        // Expected output block ended - finalize example
        state.in_expected_block = false;
        if let Some(example) = state.finalize_example() {
            examples.push(example);
        }
        return true;
    }

    if state.expecting_output_next {
        // Start of expected output block
        state.in_expected_block = true;
        state.expecting_output_next = false;
        state.current_expected = Some(String::new());
        return true;
    }

    false
}

/// Process "Expected Output:" marker
fn process_expected_output_marker(line: &str, state: &mut ParserState) -> bool {
    if line.starts_with("**Expected Output**:") {
        state.expecting_output_next = true;
        return true;
    }
    false
}

/// Process section boundary (###) - save pending code without expected output
fn process_section_boundary(
    line: &str,
    state: &mut ParserState,
    examples: &mut Vec<(String, Option<String>)>,
) -> bool {
    if line.starts_with("###") && state.has_pending_code() {
        if let Some(example) = state.finalize_example() {
            examples.push(example);
        }
        return true;
    }
    false
}

/// Collect line content into current code or expected output
fn collect_line_content(line: &str, state: &mut ParserState) {
    if state.in_code_block {
        state.current_code.push_str(line);
        state.current_code.push('\n');
    } else if state.in_expected_block {
        if let Some(ref mut expected) = state.current_expected {
            expected.push_str(line);
            expected.push('\n');
        }
    }
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
                    eprintln!("Example {i} failed to execute: {e}");
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
                    eprintln!("Example {i} output mismatch:");
                    eprintln!("  Code: {}", code.trim());
                    eprintln!("  Expected: {expected}");
                    eprintln!("  Got: {actual}");
                    failed += 1;
                }
            } else {
                // No expected output specified, just verify it didn't error
                passed += 1;
            }
        }

        let total = passed + failed;
        let pass_rate = if total > 0 {
            (f64::from(passed) / f64::from(total)) * 100.0
        } else {
            0.0
        };

        println!("\nChapter: 01-literals");
        println!("  Passed: {passed}/{total} ({pass_rate:.1}%)");
        println!("  Failed: {failed}");

        // Target: ≥90% passing
        assert!(
            pass_rate >= 90.0,
            "Pass rate {pass_rate:.1}% below 90% target"
        );
    }
}
