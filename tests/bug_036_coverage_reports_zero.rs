//! BUG-036: Coverage Reports 0/0 Lines (100%)
//!
//! **Problem**: Coverage always reports 0/0 (100%) - no actual measurement
//! **Discovered**: GitHub Issue #36
//! **Severity**: LOW - Misleading output but doesn't block development
//!
//! **Expected**: Actual line/function counts and coverage percentages
//! **Actual**: Always shows 0/0 = 100%
//!
//! **Root Cause**: `execute_with_coverage()` never calls `analyze_file()`
//! Line 378: checks `if let Some(coverage) = self.coverage_data.get_mut(file_str)`
//! But `coverage_data` is empty because `analyze_file()` was never called!
//!
//! This test follows EXTREME TDD (RED → GREEN → REFACTOR)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to get ruchy binary
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper to create temp directory
fn temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

// ==================== RED PHASE: Failing Tests ====================

/// Test 1: Coverage should report actual line counts
#[test]
fn test_bug_036_red_reports_line_counts() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun add(a, b) {
    a + b
}

fun main() {
    let result = add(1, 2)
    println(result)
}
";

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("coverage")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // RED: Currently shows "Total Lines: 0/0"
    // Should show actual counts like "Total Lines: 5/7"
    assert!(
        !output_str.contains("Total Lines: 0/0"),
        "Should report actual line counts, not 0/0. Found: {output_str}"
    );
}

/// Test 2: Coverage should report actual function counts
#[test]
fn test_bug_036_red_reports_function_counts() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    let code = r"
fun multiply(x, y) {
    x * y
}

fun divide(x, y) {
    x / y
}
";

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("coverage")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should show "Total Functions: X/2" not "0/0"
    assert!(
        !output_str.contains("Total Functions: 0/0"),
        "Should report actual function counts, not 0/0. Found: {output_str}"
    );
}

/// Test 3: Coverage reports actual numbers (not always 100%)
#[test]
fn test_bug_036_red_reports_actual_coverage() {
    let temp = temp_dir();
    let source = temp.path().join("test.ruchy");

    // Simple code
    let code = r"
fun add(a, b) {
    a + b
}
";

    fs::write(&source, code).expect("Failed to write test file");

    let output = ruchy_cmd()
        .arg("coverage")
        .arg(&source)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Should report actual numbers, not 0/0
    // Can be 100% if all code executes, that's fine
    // Just verify it's NOT showing 0/0
    assert!(
        output_str.contains("Total Lines:") && !output_str.contains("0/0"),
        "Should report actual line numbers, not 0/0. Found: {output_str}"
    );
}

// ==================== PROPERTY TESTS ====================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_coverage_never_reports_zero_over_zero(line_count in 1usize..100) {
            let temp = temp_dir();
            let source = temp.path().join("test.ruchy");

            // Generate code with N lines
            let mut code = String::new();
            for i in 0..line_count {
                code.push_str(&format!("let x{i} = {i}\n"));
            }

            fs::write(&source, code).expect("Failed to write test file");

            let output = ruchy_cmd()
                .arg("coverage")
                .arg(&source)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            let output_str = String::from_utf8_lossy(&output);

            // Property: Should NEVER show 0/0 for files with code
            prop_assert!(
                !output_str.contains("Total Lines: 0/0"),
                "Coverage should not report 0/0 for file with {} lines",
                line_count
            );
        }

        #[test]
        fn test_coverage_totals_are_non_negative(func_count in 0usize..10) {
            let temp = temp_dir();
            let source = temp.path().join("test.ruchy");

            // Generate code with N functions
            let mut code = String::new();
            for i in 0..func_count {
                code.push_str(&format!("fun func{i}() {{ {i} }}\n"));
            }

            fs::write(&source, code).expect("Failed to write test file");

            let output = ruchy_cmd()
                .arg("coverage")
                .arg(&source)
                .assert()
                .success()
                .get_output()
                .stdout
                .clone();

            let output_str = String::from_utf8_lossy(&output);

            // Property: Coverage percentages should be valid (0-100)
            // Check that output doesn't contain invalid patterns like "Total Lines: -1"
            prop_assert!(
                !output_str.contains("Lines: -") && !output_str.contains("Functions: -"),
                "Coverage should not have negative line/function counts"
            );
        }
    }
}

// ==================== RED PHASE SUMMARY ====================

/// Summary test to document the RED phase
#[test]
fn test_bug_036_red_phase_summary() {
    println!("BUG-036 RED Phase: Coverage Reports 0/0");
    println!();
    println!("Problem: Coverage tool never calls analyze_file()");
    println!("Impact: Always shows 0/0 = 100% (meaningless)");
    println!();
    println!("Root Cause:");
    println!("- execute_with_coverage() line 378: if let Some(coverage) = self.coverage_data.get_mut(file_str)");
    println!("- coverage_data is empty HashMap");
    println!("- analyze_file() exists but is never called");
    println!();
    println!("Test Suite Created:");
    println!("1. Reports actual line counts (not 0/0)");
    println!("2. Reports actual function counts (not 0/0)");
    println!("3. Coverage percentage not always 100%");
    println!();
    println!("Expected Results:");
    println!("- RED Phase: All 3 tests FAIL (shows 0/0)");
    println!("- GREEN Phase: ALL tests PASS after fix");
    println!();
    println!("Fix Strategy:");
    println!("- Call analyze_file() before execute_with_coverage() marks coverage");
    println!("- This populates total_lines and total_functions");
    println!("- Then marking covered lines/functions will give real percentages");
}
