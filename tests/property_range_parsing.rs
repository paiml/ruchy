//! Property Tests for Range Expression Parsing (PARSER-084)
//!
//! Purpose: Prove that range parsing handles open-ended and closed ranges correctly
//! Ticket: PARSER-084 (GitHub Issue #67)
//! Target: 10,000+ property test iterations pass
//!
//! ## Property Invariants
//!
//! 1. **Never Panics**: All valid range expressions parse without panic
//! 2. **Closed Ranges Work**: Expressions like `2..5` parse successfully
//! 3. **Open-Ended Ranges Work**: Expressions like `2..`, `..5`, `..` parse successfully
//! 4. **Slicing Works**: Array/string slicing like `arr[2..]`, `s[..5]` parse successfully
//! 5. **Context Preservation**: Ranges in different contexts (statements, let bindings, slicing) all work

#![allow(clippy::ignore_without_reason)] // Property tests run with --ignored flag
#![allow(missing_docs)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use proptest::prelude::*;
use ruchy::Parser;

// ============================================================================
// PROPERTY TEST GENERATORS
// ============================================================================

/// Generate arbitrary integer literals for range bounds
fn arb_int() -> impl Strategy<Value = i32> {
    0..100i32
}

/// Generate arbitrary range expressions
fn arb_range_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        // Closed range: 2..5
        (arb_int(), arb_int()).prop_map(|(start, end)| format!("{start}..{end}")),
        // Open-ended range (start only): 2..
        arb_int().prop_map(|start| format!("{start}..")),
        // Open-ended range (end only): ..5
        arb_int().prop_map(|end| format!("..{end}")),
        // Full range: ..
        Just("..".to_string()),
        // Inclusive closed range: 2..=5
        (arb_int(), arb_int()).prop_map(|(start, end)| format!("{start}..={end}")),
    ]
}

/// Generate slice expressions with ranges
#[allow(dead_code)]
fn arb_slice_expr() -> impl Strategy<Value = String> {
    (arb_range_expr(),).prop_map(|(range,)| format!("arr[{range}]"))
}

// ============================================================================
// PROPERTY 1: Range parsing never panics on valid inputs
// ============================================================================

proptest! {
    /// Property: Range parser never panics on valid range syntax
    ///
    /// Invariant: For all valid range strings r, parse(r) returns Ok(_) or Err(_), never panics
    #[test]
    #[ignore = "Run with: cargo test property_range -- --ignored --nocapture"]
    fn prop_parse_range_never_panics(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ let r = {range_expr}; }}");
        let result = std::panic::catch_unwind(|| {
            Parser::new(&code).parse()
        });

        // Should return Ok or Err, never panic
        prop_assert!(result.is_ok(), "Parser panicked on range: {}", range_expr);
    }

    /// Property: Closed ranges always parse successfully
    ///
    /// Invariant: For all integers a, b, the expression "a..b" parses successfully
    #[test]
    fn prop_closed_ranges_parse(start in arb_int(), end in arb_int()) {
        let code = format!("fun test() {{ let r = {start}..{end}; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse closed range {}..{}", start, end);
    }

    /// Property: Open-ended ranges (start only) parse successfully
    ///
    /// Invariant: For all integers a, the expression "a.." parses successfully
    #[test]
    fn prop_open_start_ranges_parse(start in arb_int()) {
        let code = format!("fun test() {{ let r = {start}..; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse open-ended range {}..", start);
    }

    /// Property: Open-ended ranges (end only) parse successfully
    ///
    /// Invariant: For all integers b, the expression "..b" parses successfully
    #[test]
    fn prop_open_end_ranges_parse(end in arb_int()) {
        let code = format!("fun test() {{ let r = ..{end}; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse open-ended range ..{}", end);
    }

    /// Property: Slice expressions with ranges parse successfully
    ///
    /// Invariant: For all ranges r, the expression "arr[r]" parses successfully
    #[test]
    fn prop_slice_with_range_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ let arr = [1, 2, 3]; let slice = arr[{range_expr}]; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse slice arr[{}]", range_expr);
    }

    /// Property: String slicing with ranges parses successfully
    ///
    /// Invariant: For all ranges r, the expression "s[r]" parses successfully when s is a string
    #[test]
    fn prop_string_slice_with_range_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ let s = \"hello\"; let slice = &s[{range_expr}]; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse string slice &s[{}]", range_expr);
    }

    /// Property: Ranges in let statements parse successfully
    ///
    /// Invariant: For all ranges r, "let x = r;" parses successfully
    #[test]
    fn prop_range_in_let_statement_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ let x = {range_expr}; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse let x = {};", range_expr);
    }

    /// Property: Ranges as function arguments parse successfully
    ///
    /// Invariant: For all ranges r, "func(r)" parses successfully
    #[test]
    fn prop_range_as_argument_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ func({range_expr}); }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse func({});", range_expr);
    }

    /// Property: Ranges in if conditions parse successfully
    ///
    /// Invariant: For all ranges r, ranges can appear in if block bodies
    #[test]
    fn prop_range_in_if_block_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ if true {{ let r = {range_expr}; }} }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse range in if block: {}", range_expr);
    }

    /// Property: Ranges in while loops parse successfully
    ///
    /// Invariant: For all ranges r, ranges can appear in while loop bodies
    #[test]
    fn prop_range_in_while_loop_parses(range_expr in arb_range_expr()) {
        let code = format!("fun test() {{ while true {{ let r = {range_expr}; break; }} }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse range in while loop: {}", range_expr);
    }

    /// Property: Multiple ranges in same function parse successfully
    ///
    /// Invariant: Parsing multiple range expressions in sequence works
    #[test]
    fn prop_multiple_ranges_parse(r1 in arb_range_expr(), r2 in arb_range_expr()) {
        let code = format!("fun test() {{ let a = {r1}; let b = {r2}; }}");
        let result = Parser::new(&code).parse();

        prop_assert!(result.is_ok(), "Failed to parse multiple ranges: {} and {}", r1, r2);
    }
}

// ============================================================================
// REGRESSION TESTS: PARSER-084 Specific Cases
// ============================================================================

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_parser_084_original_issue() {
        // Original failing case from GitHub Issue #67
        let code = r#"
use std::collections::HashMap;

fun parse_args(args: Vec<String>) -> HashMap<String, String> {
    let mut parsed = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with("--") {
            let key_part = &arg[2..];
            parsed.insert(key_part.to_string(), String::from("value"));
        }

        i += 1;
    }

    parsed
}
"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "PARSER-084 regression: Original issue should parse successfully"
        );
    }

    #[test]
    fn test_minimal_open_ended_range() {
        let code = "fun test() { let r = 2..; }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Minimal open-ended range 2.. should parse");
    }

    #[test]
    fn test_minimal_open_start_range() {
        let code = "fun test() { let r = ..5; }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Minimal open-start range ..5 should parse");
    }

    #[test]
    fn test_array_slice_open_ended() {
        let code = "fun test() { let arr = [1,2,3]; arr[2..]; }";
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Array slice arr[2..] should parse");
    }

    #[test]
    fn test_string_slice_open_ended() {
        let code = r#"fun test() { let s = "hello"; &s[2..]; }"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "String slice &s[2..] should parse");
    }

    #[test]
    fn test_range_in_let_in_if_block() {
        let code = r#"
fun test() {
    let arg = "test";
    if true {
        let key_part = &arg[2..];
    }
}
"#;
        let result = Parser::new(code).parse();
        assert!(
            result.is_ok(),
            "Range in let statement in if block should parse"
        );
    }

    #[test]
    fn test_range_with_method_call_after() {
        let code = r#"
fun test() {
    let arg = "test";
    let key_part = &arg[2..];
    println("done");
}
"#;
        let result = Parser::new(code).parse();
        assert!(result.is_ok(), "Range with statement after should parse");
    }
}
