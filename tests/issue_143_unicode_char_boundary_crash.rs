//! Issue #143: Parser crashes on Unicode characters with "byte index not a char boundary"
//!
//! **Root Cause**: `is_on_same_line()` function slices source string without checking char boundaries
//! **Location**: `src/frontend/parser/mod.rs:218`
//! **Impact**: Crashes on any Ruchy code containing Unicode symbols (✓, ✗, →, etc.)
//!
//! **Test Strategy**:
//! - RED: Test currently fails with panic
//! - GREEN: Fix `is_on_same_line()` to respect char boundaries
//! - REFACTOR: Add comprehensive Unicode test coverage

use ruchy::frontend::parser::Parser;

#[test]
fn test_issue_143_unicode_checkmark_in_string() {
    // Reproduces crash from ruchy-book experiment_001_test_functions.ruchy
    let code = r#"
fun test() {
    println("✓ PASSED")
    println("✗ FAILED")
    println("→ INFO")
}
"#;

    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Parser should handle Unicode symbols in strings"
    );
}

#[test]
fn test_issue_143_unicode_in_comments() {
    let code = r"
// Test with checkmarks: ✓ ✗ → ← ↑ ↓
fun main() {
    let x = 42
}
";

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Parser should handle Unicode in comments");
}

#[test]
fn test_issue_143_mixed_unicode() {
    // Mix of ASCII and multi-byte UTF-8
    let code = r#"
fun test_unicode() {
    println("Hello 世界")  // Chinese characters
    println("Привет мир")   // Cyrillic
    println("مرحبا")        // Arabic
    println("✓✗→←↑↓")       // Symbols
}
"#;

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Parser should handle mixed Unicode");
}

#[test]
fn test_issue_143_emoji() {
    let code = r#"
fun main() {
    println("Test result: 🎉✅")
    println("Error: ❌")
}
"#;

    let result = Parser::new(code).parse();
    assert!(result.is_ok(), "Parser should handle emoji");
}

#[test]
fn test_issue_143_long_file_with_unicode() {
    // Simulates the exact scenario from experiment_001_test_functions.ruchy
    // Unicode at specific byte position (byte 557 in original)
    let code = r#"#!/usr/bin/env ruchy
// Experiment: 001_test_functions
// Version: ruchy 3.62.9
// Date: 2025-01-10
// Author: Ruchy Book Test Suite
// Status: FAILING

// PURPOSE:
// Test whether functions can be defined and called within test assertions

// HYPOTHESIS:
// Functions defined in the same file should be accessible in test functions

fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun test_addition() {
    println("Test 1: Basic addition")
    let result = add(2, 3)
    if result == 5 {
        println("✓ PASSED: 2 + 3 = 5")
    } else {
        println("✗ FAILED: Expected 5, got " + result)
    }
}
"#;

    let result = Parser::new(code).parse();
    assert!(
        result.is_ok(),
        "Parser should handle Unicode at any position"
    );
}
