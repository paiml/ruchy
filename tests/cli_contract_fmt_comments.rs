#![allow(missing_docs)]
// CLI Contract Tests for Comment Preservation in `ruchy fmt`
//
// Purpose: Validate that formatter preserves ALL comments (Layer 4: Black Box)
// Context: DEFECT-FMT-002 - v3.88.0 strips all comments (P1 HIGH)
//
// Sprint 1 Goal: 100% comment preservation
// Tickets: [FMT-PERFECT-001] through [FMT-PERFECT-007]
//
// Test Strategy (Extreme TDD):
// - RED: Write failing test FIRST
// - GREEN: Implement minimal code to pass
// - REFACTOR: Apply quality standards (≤10 complexity)

use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// RED PHASE: These tests will FAIL until lexer tracks comments
// ============================================================================

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_single_line_comment() {
    // [FMT-PERFECT-001] RED: Failing test for line comment preservation
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("line_comment.ruchy");

    let original = "// This is a comment\nlet x = 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // CRITICAL: Must preserve comment
    assert!(
        formatted.contains("// This is a comment"),
        "Line comment was stripped! Got:\n{formatted}"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_block_comment() {
    // [FMT-PERFECT-001] RED: Failing test for block comment preservation
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("block_comment.ruchy");

    let original = "/* This is a block comment */\nlet x = 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("/* This is a block comment */"),
        "Block comment was stripped! Got:\n{formatted}"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_doc_comment() {
    // [FMT-PERFECT-001] RED: Failing test for doc comment preservation
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("doc_comment.ruchy");

    let original = "/// Returns the sum of two numbers\nfun add(a, b) { a + b }";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("/// Returns the sum"),
        "Doc comment was stripped! Got:\n{formatted}"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_trailing_comment() {
    // [FMT-PERFECT-001] RED: Failing test for trailing comment preservation
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("trailing_comment.ruchy");

    let original = "let x = 42  // Important value";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("// Important value"),
        "Trailing comment was stripped! Got:\n{formatted}"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_multiple_line_comments() {
    // [FMT-PERFECT-001] RED: Multiple comments
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiple_comments.ruchy");

    let original = r"// Comment 1
// Comment 2
// Comment 3
let x = 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("// Comment 1"),
        "Comment 1 was stripped!"
    );
    assert!(
        formatted.contains("// Comment 2"),
        "Comment 2 was stripped!"
    );
    assert!(
        formatted.contains("// Comment 3"),
        "Comment 3 was stripped!"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_mixed_comment_types() {
    // [FMT-PERFECT-001] RED: Mix of line, block, and doc comments
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("mixed_comments.ruchy");

    let original = r"// Line comment
/* Block comment */
/// Doc comment
let x = 42  // Trailing comment";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(formatted.contains("// Line comment"));
    assert!(formatted.contains("/* Block comment */"));
    assert!(formatted.contains("/// Doc comment"));
    assert!(formatted.contains("// Trailing comment"));
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_comment_inside_function() {
    // [FMT-PERFECT-001] RED: Comments inside function bodies
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("comment_in_function.ruchy");

    let original = r"fun add(a, b) {
    // Calculate sum
    a + b
}";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    assert!(
        formatted.contains("// Calculate sum"),
        "Comment inside function was stripped!"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_comment_order() {
    // [FMT-PERFECT-001] RED: Comment order must be preserved
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("comment_order.ruchy");

    let original = r"// Before function
fun add(a, b) {
    // Inside function
    a + b  // End of line
}
// After function";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();
    let lines: Vec<&str> = formatted.lines().collect();

    // Verify all comments present
    assert!(formatted.contains("// Before function"));
    assert!(formatted.contains("// Inside function"));
    assert!(formatted.contains("// End of line"));
    assert!(formatted.contains("// After function"));

    // Verify order preserved (before < inside < after)
    let before_idx = lines
        .iter()
        .position(|l| l.contains("Before function"))
        .expect("Before comment missing");
    let inside_idx = lines
        .iter()
        .position(|l| l.contains("Inside function"))
        .expect("Inside comment missing");
    let after_idx = lines
        .iter()
        .position(|l| l.contains("After function"))
        .expect("After comment missing");

    assert!(
        before_idx < inside_idx && inside_idx < after_idx,
        "Comment order not preserved! before={before_idx} inside={inside_idx} after={after_idx}"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_multiline_block_comment() {
    // [FMT-PERFECT-001] RED: Multi-line block comments
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiline_block.ruchy");

    let original = r"/*
 * This is a multi-line
 * block comment with
 * proper formatting
 */
let x = 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Must preserve the multi-line structure
    assert!(
        formatted.contains("This is a multi-line"),
        "Multi-line block comment was stripped!"
    );
}

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_empty_line_comments() {
    // [FMT-PERFECT-001] RED: Empty comments should be preserved
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty_comment.ruchy");

    let original = "//\nlet x = 42";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Empty comment should still be present
    assert!(
        formatted.lines().any(|l| l.trim() == "//"),
        "Empty comment was stripped!"
    );
}

// ============================================================================
// Real-World Test: head.ruchy with extensive documentation
// ============================================================================

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_head_ruchy_comments() {
    // [FMT-PERFECT-006] Integration test with real documented code
    // This is the actual file from ruchy-cli-tools-book that triggered DEFECT-FMT-002
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("head.ruchy");

    // Simplified head.ruchy with key documentation comments
    let original = r#"// ruchy-head: Output the first n lines of a file
// Sprint 4 - Chapter 4 example from Ruchy CLI Tools Book

// Returns the first n lines from a file.
// If n is greater than the number of lines, returns all lines.
// If n is 0, returns empty string.
// Algorithm: O(n) single pass through file content.
fun head_lines(file_path, n) {
    let content = fs_read(file_path)
    let result = ""
    let line_count = 0

    for i in range(0, content.len()) {
        let ch = content[i]

        if line_count < n {
            result = result + ch
        }

        if ch == "\n" {
            line_count = line_count + 1
            if line_count >= n {
                return result
            }
        }
    }

    result
}"#;
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // CRITICAL: All 6 documentation comments MUST be preserved
    assert!(
        formatted.contains("ruchy-head:"),
        "Header comment was stripped!"
    );
    assert!(
        formatted.contains("Sprint 4"),
        "Sprint reference was stripped!"
    );
    assert!(
        formatted.contains("Returns the first n lines"),
        "Function description was stripped!"
    );
    assert!(
        formatted.contains("If n is greater"),
        "Edge case documentation was stripped!"
    );
    assert!(
        formatted.contains("If n is 0"),
        "Edge case documentation was stripped!"
    );
    assert!(
        formatted.contains("Algorithm: O(n)"),
        "Complexity documentation was stripped!"
    );

    // Verify code is still functional (doesn't corrupt)
    assert!(
        !formatted.contains("IndexAccess {"),
        "REGRESSION: Code was corrupted with AST debug output!"
    );
    assert!(
        !formatted.contains("Assign {"),
        "REGRESSION: Code was corrupted with AST debug output!"
    );
}

// ============================================================================
// Comment Count Validation
// ============================================================================

#[test]
#[ignore = "RED phase TDD - lexer doesn't track comments yet. Implement comment preservation in Sprint FORMATTER-003"]
fn test_fmt_preserves_exact_comment_count() {
    // [FMT-PERFECT-001] RED: Exact comment count must match
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("comment_count.ruchy");

    let original = r"// Comment 1
let x = 42  // Comment 2
/* Comment 3 */
let y = x * 2
// Comment 4
// Comment 5";
    fs::write(&test_file, original).unwrap();

    let output = ruchy_cmd()
        .arg("fmt")
        .arg(&test_file)
        .arg("--stdout")
        .output()
        .expect("Failed to run fmt");

    let formatted = String::from_utf8(output.stdout).unwrap();

    // Count comments in original
    let original_line_comments = original.matches("//").count();
    let original_block_comments = original.matches("/*").count();

    // Count comments in formatted
    let formatted_line_comments = formatted.matches("//").count();
    let formatted_block_comments = formatted.matches("/*").count();

    assert_eq!(
        original_line_comments, formatted_line_comments,
        "Line comment count mismatch! Original: {original_line_comments}, Formatted: {formatted_line_comments}"
    );

    assert_eq!(
        original_block_comments, formatted_block_comments,
        "Block comment count mismatch! Original: {original_block_comments}, Formatted: {formatted_block_comments}"
    );
}
