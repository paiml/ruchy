#![allow(missing_docs)]
//! DEFECT-PARSER-007: Inline Comments in Struct Field Definitions
//!
//! **Problem**: Inline comments after struct field declarations cause "Expected field name" error
//! **Root Cause**: `parse_struct_fields()` doesn't skip comment tokens between fields
//! **Book Examples Affected**: ch19-00-structs-oop.md (example 7)
//!
//! **Reproduction**:
//! ```ruchy
//! struct BankAccount {
//!     pub owner: String,       // Public field
//!     balance: f64            // Private field
//! }
//! ```
//!
//! Run with: cargo test --test `defect_parser_007_struct_inline_comments`

use ruchy::frontend::parser::Parser;

/// Helper: Parse Ruchy code and verify it succeeds
fn parse_ruchy(source: &str) -> Result<(), String> {
    let mut parser = Parser::new(source);
    parser.parse().map_err(|e| format!("Parse error: {e:?}"))?;
    Ok(())
}

// ============================================================================
// RED PHASE: Failing Tests (Reproduce Defect)
// ============================================================================

#[test]
fn test_defect_parser_007_inline_comment_after_field() {
    // SHOULD FAIL: Inline comment after struct field declaration
    let code = r"
        struct BankAccount {
            pub owner: String,  // Public field
            balance: f64
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Inline comments after struct fields should work: {result:?}"
    );
}

#[test]
fn test_defect_parser_007_multiple_inline_comments() {
    // Book example from ch19-00-structs-oop.md
    let code = r"
        struct BankAccount {
            pub owner: String,       // Public field
            balance: f64,           // Private field (default)
            pub(crate) id: i32      // Crate-visible field
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Multiple inline comments in struct should work: {result:?}"
    );
}

#[test]
fn test_defect_parser_007_block_comment_after_field() {
    // Block comments should also work
    let code = r"
        struct Point {
            x: f64, /* X coordinate */
            y: f64  /* Y coordinate */
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Block comments after struct fields should work: {result:?}"
    );
}

#[test]
fn test_defect_parser_007_mixed_comments() {
    // Mix of line and block comments
    let code = r"
        struct Config {
            host: String,     // Server hostname
            port: i32,        /* Port number */
            timeout: i32      // Connection timeout
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Mixed comments in struct should work: {result:?}"
    );
}

#[test]
fn test_defect_parser_007_no_comments_still_works() {
    // Regression test - ensure fix doesn't break existing behavior
    let code = r"
        struct Counter {
            count: i32,
            max: i32
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Struct without comments should still work: {result:?}"
    );
}

#[test]
fn test_defect_parser_007_comment_before_field() {
    // Comments before fields (should already work)
    let code = r"
        struct User {
            // Username field
            name: String,
            // User ID
            id: i32
        }
    ";

    let result = parse_ruchy(code);
    assert!(
        result.is_ok(),
        "Comments before struct fields should work: {result:?}"
    );
}

// ============================================================================
// GREEN PHASE: Implementation (To Be Added)
// ============================================================================
// Implementation changes will be made in:
// - src/frontend/parser/expressions_helpers/structs.rs (parse_struct_fields function)

// ============================================================================
// REFACTOR PHASE: Quality Validation (After Fix)
// ============================================================================
// After fix:
// - Run cargo test --test defect_parser_007_struct_inline_comments
// - Verify all 6 tests pass
// - Run book extraction: deno run --allow-read --allow-write --allow-run ../ruchy-book/scripts/extract-examples.ts
// - Verify ch19-ex7 passes (98% → 99%)
