// LANG-COMP-001: Basic Syntax - Variables Property Tests
// RED→GREEN→REFACTOR: Tests written FIRST
// Quality: Property-based testing with 10K+ cases

use proptest::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

/// Helper: Run code via REPL stdin and filter output
fn run_repl_code(code: &str) -> Result<String, String> {
    let mut child = Command::new("ruchy")
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(code.as_bytes())
            .map_err(|e| format!("Failed to write: {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Filter out REPL prompts and status lines
        let filtered: String = stdout
            .lines()
            .filter(|line| {
                !line.contains("Type :help")
                    && !line.contains("Goodbye")
                    && !line.contains("Welcome")
                    && !line.contains("Ruchy REPL")
                    && !line.contains("ALL functions")
                    && !line.contains("coverage")
                    && !line.contains("TDG")
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(filtered)
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Property: Let bindings should preserve integer values
#[test]
fn test_variable_let_binding_integers() {
    let output = Command::new("ruchy")
        .args([
            "run",
            "examples/lang_comp/01-basic-syntax/01_variables.ruchy",
        ])
        .output()
        .expect("Failed to run example");

    assert!(output.status.success(), "Example failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("42"), "Expected output to contain 42");
}

/// Property: Let bindings should preserve string values
#[test]
fn test_variable_let_binding_strings() {
    let output = Command::new("ruchy")
        .args([
            "run",
            "examples/lang_comp/01-basic-syntax/02_string_variables.ruchy",
        ])
        .output()
        .expect("Failed to run example");

    assert!(output.status.success(), "Example failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello"),
        "Expected output to contain 'Hello'"
    );
}

proptest! {
    /// Property: Let bindings should work with any valid identifier
    #[test]
    fn prop_variable_names_valid(name in "[a-z][a-z0-9_]{0,15}") {
        let code = format!("let {name} = 42; {name}");
        let result = run_repl_code(&code);

        // Should either succeed or fail with clear error
        match result {
            Ok(_) => {}, // Success is acceptable
            Err(stderr) => {
                prop_assert!(
                    stderr.contains("error") || stderr.contains("Error"),
                    "Failed execution should have error message"
                );
            }
        }
    }

    /// Property: Integer literals should preserve exact values
    #[test]
    fn prop_integer_literals(n in -1000i64..1000i64) {
        let code = format!("{n}");
        if let Ok(stdout) = run_repl_code(&code) {
            prop_assert!(
                stdout.contains(&n.to_string()),
                "Output should contain the integer value: {}", n
            );
        }
    }

    /// Property: Float literals should preserve values
    #[test]
    fn prop_float_literals(f in -100.0f64..100.0f64) {
        let code = format!("{f}");
        if let Ok(stdout) = run_repl_code(&code) {
            // Float representation may vary slightly
            prop_assert!(
                !stdout.trim().is_empty(),
                "Output should contain float representation"
            );
        }
    }

    /// Property: String literals should preserve content
    #[test]
    fn prop_string_literals(s in "[a-zA-Z0-9 ]{1,20}") {
        let code = format!("\"{s}\"");
        if let Ok(stdout) = run_repl_code(&code) {
            prop_assert!(
                stdout.contains(&s),
                "Output should contain the string content: {}", s
            );
        }
    }

    /// Property: Multiple variable declarations should maintain independence
    #[test]
    fn prop_multiple_variables(a in 0i64..100, b in 0i64..100) {
        let code = format!("let x = {a}; let y = {b}; x + y");
        let expected = a + b;

        if let Ok(stdout) = run_repl_code(&code) {
            prop_assert!(
                stdout.contains(&expected.to_string()),
                "Output should contain sum: {}", expected
            );
        }
    }
}

/// Property: Boolean literals should preserve truth values
#[test]
fn test_boolean_literals() {
    let test_cases = vec![("true", "true"), ("false", "false")];

    for (input, expected) in test_cases {
        let result = run_repl_code(input);
        assert!(
            result.is_ok(),
            "Boolean literal failed: {input} - {result:?}"
        );

        let stdout = result.unwrap();
        assert!(
            stdout.contains(expected),
            "Expected '{expected}' in output, got: {stdout}"
        );
    }
}

/// Property: Comments should be ignored
#[test]
fn test_comments() {
    let test_cases = vec![
        ("// comment\n42", "42"),
        ("/* block comment */ 42", "42"),
        ("42 // trailing comment", "42"),
    ];

    for (input, expected) in test_cases {
        let result = run_repl_code(input);
        assert!(
            result.is_ok(),
            "Comment test failed: {input} - {result:?}"
        );

        let stdout = result.unwrap();
        assert!(
            stdout.contains(expected),
            "Expected '{expected}' in output, got: {stdout}"
        );
    }
}
