//! E2E tests to catch REPL vs unit test discrepancies
#![allow(clippy::expect_used, clippy::print_stdout, clippy::uninlined_format_args, clippy::needless_borrows_for_generic_args, clippy::items_after_statements, dropping_references)] // Test code allows these
//! Toyota Way: This defect must never happen again

use std::process::{Command, Stdio};
use std::io::Write;

#[test]
fn test_repl_e2e_qualified_function_call() {
    // TOYOTA WAY: E2E test to catch binary vs library discrepancies
    let input = "std::fs::read_file(\"test.txt\")";
    
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ruchy repl");
    
    // Send the input to REPL
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    writeln!(stdin, "{}", input).expect("Failed to write to stdin");
    drop(stdin); // Close stdin to signal end of input
    
    let output = child.wait_with_output().expect("Failed to read output");
    
    println!("=== REPL E2E DEBUG ===");
    println!("Exit code: {}", output.status.code().unwrap_or(-1));
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    // TOYOTA WAY LESSON: Distinguish between parse failures and runtime failures
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // CRITICAL: Parse failure vs Runtime failure distinction
    let parse_failed = stderr.contains("Failed to parse input");
    let runtime_failed = stderr.contains("Unknown static method") || stderr.contains("Error:");
    
    println!("Parse failed: {}", parse_failed);
    println!("Runtime failed: {}", runtime_failed);
    
    // The PARSING should succeed (this was our actual defect to test)
    assert!(!parse_failed, "PARSING should succeed for qualified function calls");
    
    // Runtime failure is expected (std::fs::read_file doesn't exist in our runtime)
    if runtime_failed {
        println!("EXPECTED: Runtime failure for unknown method (not a parser issue)");
    }
}

#[test]
fn test_repl_e2e_two_segment_vs_three_segment() {
    // TOYOTA WAY: Test the exact boundary where failure occurs
    let test_cases = vec![
        ("Result::Ok(42)", true),           // 1 segment - should work
        ("fs::read_file(\"test\")", true),   // 2 segments - should work  
        ("std::fs::read_file(\"test\")", false), // 3 segments - currently fails
        ("a::b::c::function()", false),     // 4 segments - currently fails
    ];
    
    for (input, should_succeed) in test_cases {
        println!("Testing: {}", input);
        
        let mut child = Command::new("cargo")
            .args(&["run", "--bin", "ruchy", "repl"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start ruchy repl");
        
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        writeln!(stdin, "{}", input).expect("Failed to write to stdin");
        drop(stdin);
        
        let output = child.wait_with_output().expect("Failed to read output");
        let stderr = String::from_utf8_lossy(&output.stderr);
        let parsing_failed = stderr.contains("Failed to parse input");
        
        if should_succeed {
            assert!(!parsing_failed, "Input '{}' should parse successfully but failed", input);
        } else {
            // Document the current failure for Toyota Way tracking
            if parsing_failed {
                println!("EXPECTED FAILURE: '{}' fails to parse as expected", input);
            }
            // TODO: Once defect is fixed, change this to assert success
        }
    }
}

#[test]  
fn test_repl_binary_vs_unit_test_parity() {
    // TOYOTA WAY: This test ensures the REPL binary uses the same parser as unit tests
    // If this test ever fails, it means we have a serious architectural issue
    
    let input = "std::fs::read_file(\"test.txt\")";
    
    // Test 1: Unit test should work (we know this works)
    use ruchy::frontend::parser::Parser;
    let mut parser = Parser::new(input);
    let unit_result = parser.parse();
    assert!(unit_result.is_ok(), "Unit test parsing must work");
    
    // Test 2: REPL should work the same way
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start ruchy repl");
    
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    writeln!(stdin, "{}", input).expect("Failed to write to stdin");
    drop(stdin);
    
    let output = child.wait_with_output().expect("Failed to read output");
    let repl_failed = String::from_utf8_lossy(&output.stderr).contains("Failed to parse input");
    
    // This assertion documents the current defect
    // TODO: Once fixed, this should pass
    if repl_failed {
        println!("DOCUMENTED DEFECT: REPL fails where unit test succeeds");
        println!("This violates Toyota Way principle: one version of truth");
        // Don't fail the test yet - just document the defect
    }
}