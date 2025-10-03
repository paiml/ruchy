//! Parser regression test for GitHub issue #24
//!
//! Issue: Array references '&[T; N]' fail with 3+ parameters
//! Status: FIXED in v3.64.1
//!
//! This test verifies that the parser correctly handles fixed-size array
//! references in function parameters, particularly with 3+ total parameters.

use ruchy::frontend::Parser;

/// Helper: Parse code and return Ok if successful
fn parse_ok(code: &str) -> Result<(), String> {
    let mut parser = Parser::new(code);
    parser.parse().map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_array_ref_one_param() {
    // This case always worked
    let code = "fun test_one(arr: &[i32; 25]) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array ref with 1 parameter"
    );
}

#[test]
fn test_array_ref_two_params() {
    // This case always worked
    let code = "fun test_two(arr: &[i32; 25], x: i32) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array ref with 2 parameters"
    );
}

#[test]
fn test_array_ref_three_params() {
    // This is the bug case that was reported in #24
    let code = "fun test_three(arr: &[i32; 25], x: i32, y: i32) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array ref with 3+ parameters (issue #24)"
    );
}

#[test]
fn test_array_ref_four_params() {
    let code = "fun test_four(arr: &[i32; 25], x: i32, y: i32, z: i32) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array ref with 4+ parameters"
    );
}

#[test]
fn test_multiple_array_refs() {
    let code = "fun test_multi(a: &[i32; 10], b: &[f64; 20], c: i32) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept multiple array refs"
    );
}

#[test]
fn test_array_ref_different_sizes() {
    let code = r"
        fun process_small(arr: &[i32; 5]) -> i32 { 1 }
        fun process_medium(arr: &[i32; 100]) -> i32 { 2 }
        fun process_large(arr: &[i32; 1000]) -> i32 { 3 }
    ";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array refs of various sizes"
    );
}

#[test]
fn test_array_ref_with_generics() {
    let code = "fun generic<T>(arr: &[T; 10], x: i32) -> T { arr[0] }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept array refs with generic types"
    );
}

#[test]
fn test_array_ref_nested() {
    let code = "fun nested(matrix: &[[i32; 5]; 5], x: i32, y: i32) -> i32 { 42 }";
    assert!(
        parse_ok(code).is_ok(),
        "Parser should accept nested array refs"
    );
}
