//! Issue #99: BUG: provability score only counts assertions, ignores actual formal verification
//!
//! Tests the provability score calculation to ensure it integrates ALL verification analyses,
//! not just assertion density.
//!
//! Reference: <https://github.com/paiml/ruchy/issues/99>
//! EXTREME TDD: These tests demonstrate the expected behavior (RED phase)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

/// Helper: Create ruchy command
fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

/// Helper: Create temp file with content
fn create_temp_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

// ============================================================================
// CORE BUG: Score Should Include ALL Verification Analyses
// ============================================================================

#[test]
fn test_issue_099_pure_safe_terminating_code_scores_above_zero() {
    // CRITICAL: Pure, safe, terminating code WITHOUT assertions should score > 0.0
    // Expected: ~60-80/100 (purity + safety + termination, no assertions)
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "pure.ruchy",
        r#"
fun greet() {
    println("Hello from function!");
}

fun main() {
    greet();
}
"#,
    );

    let output = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Extract score from "Provability Score: XX.X/100"
    let score_line = stdout
        .lines()
        .find(|line| line.contains("Provability Score:"))
        .expect("Should have provability score line");

    let score_str = score_line
        .split("Provability Score:")
        .nth(1)
        .expect("Should have score after colon")
        .split('/')
        .next()
        .expect("Should have score before slash")
        .trim();

    let score: f64 = score_str.parse().expect("Should parse as float");

    // CRITICAL: Pure, safe, terminating code should NOT score 0.0
    assert!(
        score > 0.0,
        "Pure, safe, terminating code should score > 0.0, got {score}"
    );

    // Expected: At least 60/100 (purity + safety + termination = 20 + 20 + 20)
    assert!(
        score >= 60.0,
        "Pure, safe, terminating code should score >= 60.0, got {score}"
    );
}

#[test]
fn test_issue_099_code_with_assertions_scores_higher() {
    // Code with assertions should score HIGHER than code without
    let temp = TempDir::new().unwrap();
    let file_with_assertions = create_temp_file(
        &temp,
        "with_asserts.ruchy",
        r"
fun test_add() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

fun main() {
    test_add();
}
",
    );

    let output_with = ruchy_cmd()
        .arg("provability")
        .arg(&file_with_assertions)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout_with = String::from_utf8(output_with).unwrap();
    let score_with = extract_score(&stdout_with);

    // Expected: ~80-100/100 (purity + safety + termination + assertions)
    assert!(
        score_with >= 80.0,
        "Code with assertions should score >= 80.0, got {score_with}"
    );
}

#[test]
fn test_issue_099_verify_flag_contributes_to_score() {
    // When --verify flag is used, verification analysis should contribute to score
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "verified.ruchy",
        r"
fun pure_function(x) {
    x * 2
}

fun main() {
    let result = pure_function(5);
    println(result);
}
",
    );

    let output = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should include verification section
    assert!(
        stdout.contains("=== Formal Verification ==="),
        "Should have Formal Verification section"
    );

    // Score should reflect verification results
    let score = extract_score(&stdout);
    assert!(
        score > 0.0,
        "Verified code should score > 0.0, got {score}"
    );
}

#[test]
fn test_issue_099_termination_flag_contributes_to_score() {
    // When --termination flag is used, termination analysis should contribute to score
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "terminates.ruchy",
        r"
fun factorial(n) {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fun main() {
    println(factorial(5));
}
",
    );

    let output = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--termination")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should include termination section
    assert!(
        stdout.contains("=== Termination Analysis ==="),
        "Should have Termination Analysis section"
    );

    // Score should be positive
    let score = extract_score(&stdout);
    assert!(
        score > 0.0,
        "Code with termination analysis should score > 0.0, got {score}"
    );
}

#[test]
fn test_issue_099_bounds_flag_contributes_to_score() {
    // When --bounds flag is used, bounds checking should contribute to score
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "bounds.ruchy",
        r"
fun safe_access(arr, i) {
    if i < arr.len() {
        arr[i]
    } else {
        0
    }
}

fun main() {
    let arr = [1, 2, 3];
    println(safe_access(arr, 1));
}
",
    );

    let output = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--bounds")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should include bounds section
    assert!(
        stdout.contains("=== Bounds Checking ==="),
        "Should have Bounds Checking section"
    );

    // Score should be positive
    let score = extract_score(&stdout);
    assert!(
        score > 0.0,
        "Code with bounds checking should score > 0.0, got {score}"
    );
}

// ============================================================================
// MULTI-FACTOR SCORING TESTS
// ============================================================================

#[test]
fn test_issue_099_multi_factor_score_calculation() {
    // Test that all factors contribute to final score:
    // - Purity (20 points)
    // - Safety (20 points)
    // - Termination (20 points)
    // - Bounds checking (20 points)
    // - Assertions (20 points)

    let temp = TempDir::new().unwrap();

    // 1. Pure, safe, terminating code with bounds checking (no assertions)
    let file1 = create_temp_file(
        &temp,
        "no_asserts.ruchy",
        r"
fun pure_add(x, y) { x + y }
fun main() { println(pure_add(1, 2)); }
",
    );

    let score1 = get_provability_score(&file1);

    // Expected: 80/100 (purity + safety + termination + bounds, NO assertions)
    assert!(
        (60.0..=85.0).contains(&score1),
        "Code without assertions should score 60-85, got {score1}"
    );

    // 2. Same code WITH assertions
    let file2 = create_temp_file(
        &temp,
        "with_asserts.ruchy",
        r"
fun pure_add(x, y) {
    let result = x + y;
    assert!(result > 0);
    result
}
fun main() {
    let r = pure_add(1, 2);
    assert_eq!(r, 3);
    println(r);
}
",
    );

    let score2 = get_provability_score(&file2);

    // Expected: 100/100 (all factors present)
    assert!(
        score2 >= 90.0,
        "Code with all factors should score >= 90, got {score2}"
    );

    // Score with assertions should be > score without
    assert!(
        score2 > score1,
        "Code with assertions ({score2}) should score higher than without ({score1})"
    );
}

#[test]
fn test_issue_099_empty_code_default_score() {
    // Minimal code (just a comment) should have default score (50.0 per current implementation)
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(&temp, "minimal.ruchy", "// Minimal program");

    // Note: Parser rejects truly empty files, so we use a comment-only file
    // This should result in empty AST after parsing (no executable statements)
    let result = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .assert()
        .get_output()
        .clone();

    // Either parse error OR success with default score
    if result.status.success() {
        let stdout = String::from_utf8(result.stdout).unwrap();
        let score = extract_score(&stdout);
        assert_eq!(
            score, 50.0,
            "Minimal code should score 50.0 (neutral), got {score}"
        );
    } else {
        // Parse error is acceptable for comment-only file
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(
            stderr.contains("Parse error") || stderr.contains("Empty program"),
            "Should have parse error for minimal code"
        );
    }
}

// ============================================================================
// REGRESSION TESTS: Verify All Flags Work
// ============================================================================

#[test]
fn test_issue_099_all_flags_together() {
    // Test that all flags can be used together
    let temp = TempDir::new().unwrap();
    let file = create_temp_file(
        &temp,
        "comprehensive.ruchy",
        r"
fun factorial(n) {
    assert!(n >= 0);
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fun main() {
    let result = factorial(5);
    assert_eq!(result, 120);
    println(result);
}
",
    );

    let output = ruchy_cmd()
        .arg("provability")
        .arg(&file)
        .arg("--verify")
        .arg("--contracts")
        .arg("--invariants")
        .arg("--termination")
        .arg("--bounds")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Should include ALL sections
    assert!(stdout.contains("=== Formal Verification ==="));
    assert!(stdout.contains("=== Contract Verification ==="));
    assert!(stdout.contains("=== Loop Invariants ==="));
    assert!(stdout.contains("=== Termination Analysis ==="));
    assert!(stdout.contains("=== Bounds Checking ==="));

    // Score should be high (all factors + assertions)
    let score = extract_score(&stdout);
    assert!(
        score >= 90.0,
        "Comprehensive analysis with assertions should score >= 90.0, got {score}"
    );
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Extract provability score from command output
fn extract_score(stdout: &str) -> f64 {
    let score_line = stdout
        .lines()
        .find(|line| line.contains("Provability Score:"))
        .expect("Should have provability score line");

    let score_str = score_line
        .split("Provability Score:")
        .nth(1)
        .expect("Should have score after colon")
        .split('/')
        .next()
        .expect("Should have score before slash")
        .trim();

    score_str.parse().expect("Should parse as float")
}

/// Get provability score for a file
fn get_provability_score(file: &std::path::Path) -> f64 {
    let output = ruchy_cmd()
        .arg("provability")
        .arg(file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();
    extract_score(&stdout)
}
