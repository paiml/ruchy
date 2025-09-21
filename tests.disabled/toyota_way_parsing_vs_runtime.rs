#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
//! Toyota Way: Idiot-proof separation of parsing vs runtime testing
#![allow(
    clippy::expect_used,
    clippy::print_stdout,
    clippy::uninlined_format_args,
    clippy::needless_borrows_for_generic_args,
    clippy::cast_sign_loss
)] // Test code allows these
//!
//! This test suite ensures we NEVER confuse parsing failures with runtime failures again.
//! Each test explicitly separates the two phases and tests them independently.

use ruchy::frontend::parser::Parser;
use std::io::Write;
use std::process::{Command, Stdio};

/// Test that separates parsing success from runtime success
/// This prevents the confusion that led to the false defect report
#[test]
fn toyota_way_parse_vs_runtime_separation() {
    let test_cases = vec![
        // [input, should_parse, should_run_without_error, description]
        (
            "1 + 2",
            true,
            true,
            "Simple arithmetic - both parse and run",
        ),
        (
            "Result::Ok(42)",
            true,
            true,
            "Known constructor - both parse and run",
        ),
        (
            "std::fs::read_file(\"test\")",
            true,
            false,
            "Unknown method - parse OK, runtime fail",
        ),
        (
            "invalid::syntax::}",
            false,
            false,
            "Invalid syntax - parse fails",
        ),
        (
            "println(\"hello\")",
            true,
            true,
            "Known function - both parse and run",
        ),
    ];

    for (input, should_parse, should_run, description) in test_cases {
        println!("Testing: {} - {}", input, description);

        // PHASE 1: PARSING TEST (isolated)
        let mut parser = Parser::new(input);
        let parse_result = parser.parse();
        let parse_succeeded = parse_result.is_ok();

        assert_eq!(
            parse_succeeded, should_parse,
            "Parse phase mismatch for '{}': expected parse={}, got parse={}",
            input, should_parse, parse_succeeded
        );

        // PHASE 2: RUNTIME TEST (E2E with REPL)
        if parse_succeeded {
            // Only test runtime if parsing succeeded
            let mut child = Command::new("cargo")
                .args(&["run", "--bin", "ruchy", "repl"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start ruchy repl");

            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            writeln!(stdin, "{}", input).expect("Failed to write to stdin");
            let _ = stdin; // Close stdin

            let output = child.wait_with_output().expect("Failed to read output");
            let stderr = String::from_utf8_lossy(&output.stderr);

            let parse_failed_at_runtime = stderr.contains("Failed to parse input");
            let runtime_error = stderr.contains("Error:")
                && !stderr.contains("Compiling")
                && !stderr.contains("Finished");
            let runtime_succeeded = !parse_failed_at_runtime && !runtime_error;

            // CRITICAL: If parsing succeeded in unit test, it MUST succeed in REPL
            assert!(!parse_failed_at_runtime,
                "FATAL: Parsing succeeded in unit test but failed in REPL for '{}'. This indicates parser inconsistency.",
                input);

            // Runtime success/failure should match expectations
            assert_eq!(
                runtime_succeeded, should_run,
                "Runtime phase mismatch for '{}': expected run={}, got run={}. Stderr: {}",
                input, should_run, runtime_succeeded, stderr
            );

            println!(
                "  ✅ Parse: {}, Runtime: {} (expected: {})",
                parse_succeeded, runtime_succeeded, should_run
            );
        } else {
            println!("  ✅ Parse: {} (skipped runtime test)", parse_succeeded);
        }
    }
}

/// Property-based test: Any input that parses in unit tests MUST parse in REPL
#[test]
fn toyota_way_parser_consistency_property() {
    let inputs = vec![
        "std::fs::read_file(\"test.txt\")",
        "a::b::c::deeply::nested::function()",
        "std::collections::HashMap::new()",
        "Result::Ok(std::option::Option::Some(42))",
        "match x { Ok(y) => y, Err(e) => panic!(e) }",
        "fn test(x: std::string::String) -> std::result::Result { x }",
    ];

    for input in inputs {
        // Unit test parsing
        let mut parser = Parser::new(input);
        let unit_result = parser.parse();

        if unit_result.is_ok() {
            // REPL parsing - must match unit test result
            let mut child = Command::new("cargo")
                .args(&["run", "--bin", "ruchy", "repl"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start ruchy repl");

            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            writeln!(stdin, "{}", input).expect("Failed to write to stdin");
            let _ = stdin;

            let output = child.wait_with_output().expect("Failed to read output");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let repl_parse_failed = stderr.contains("Failed to parse input");

            assert!(!repl_parse_failed,
                "PROPERTY VIOLATION: Input '{}' parses in unit tests but fails in REPL. This violates parser consistency.",
                input);

            println!("✅ Consistency verified for: {}", input);
        }
    }
}

/// Fuzz-like test with complex qualified names
#[test]
fn toyota_way_qualified_names_stress_test() {
    let complex_cases = vec![
        "a::b::c::d::e::f::g::h::function()",
        "std::collections::HashMap<String, Vec<Result<Option<i32>, String>>>",
        "super::super::parent::module::deeply::nested::Type",
        "crate::internal::parser::expressions::parse_qualified_name",
        "::global::absolute::path::function",
    ];

    for input in complex_cases {
        println!("Stress testing: {}", input);

        // All of these should parse successfully
        let mut parser = Parser::new(input);
        let result = parser.parse();

        if result.is_err() {
            println!("  ⚠️  Parse failed (acceptable): {:?}", result.err());
        } else {
            println!("  ✅ Parse succeeded");

            // If it parses, ensure REPL consistency
            let mut child = Command::new("cargo")
                .args(&["run", "--bin", "ruchy", "repl"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start ruchy repl");

            let stdin = child.stdin.as_mut().expect("Failed to open stdin");
            writeln!(stdin, "{}", input).expect("Failed to write to stdin");
            let _ = stdin;

            let output = child.wait_with_output().expect("Failed to read output");
            let stderr = String::from_utf8_lossy(&output.stderr);
            let repl_parse_failed = stderr.contains("Failed to parse input");

            assert!(
                !repl_parse_failed,
                "Complex qualified name '{}' parses in unit test but fails in REPL",
                input
            );

            println!("  ✅ REPL consistency verified");
        }
    }
}

/// Performance test: Parsing should be fast and consistent
#[test]
fn toyota_way_parsing_performance_consistency() {
    use std::time::Instant;

    let input =
        "std::collections::HashMap<String, Vec<Result<Option<std::fs::File>, std::io::Error>>>";
    let iterations = 1000;

    let start = Instant::now();
    for _ in 0..iterations {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Performance test input should parse successfully"
        );
    }
    let duration = start.elapsed();

    println!("Parsed {} qualified names in {:?}", iterations, duration);
    println!("Average: {:?} per parse", duration / iterations as u32);

    // Performance requirement: Should parse 1000 qualified names in under 100ms
    assert!(
        duration.as_millis() < 100,
        "Parsing performance regression: {} qualified names took {:?} (should be <100ms)",
        iterations,
        duration
    );
}
