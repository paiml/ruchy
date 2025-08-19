//! Tests to boost code coverage for various components
#![allow(clippy::unwrap_used)]

use ruchy::{compile, is_valid_syntax};

#[test]
fn test_dataframe_edge_cases() {
    // Empty DataFrame
    assert!(is_valid_syntax("df![]"));

    // DataFrame with multiple columns
    assert!(is_valid_syntax("df![a => [1, 2], b => [3, 4]]"));

    // DataFrame chain operations
    assert!(is_valid_syntax("df![x => [1]].head(5).tail(3)"));

    // DataFrame with sort_by
    assert!(is_valid_syntax("df![x => [1]].sort_by(x)"));

    // DataFrame with groupby
    assert!(is_valid_syntax("df![x => [1], y => [2]].groupby(x)"));

    // DataFrame median operation
    assert!(is_valid_syntax("df![x => [1, 2, 3]].median()"));

    // DataFrame var operation
    assert!(is_valid_syntax("df![x => [1, 2, 3]].var()"));

    // DataFrame std operation
    assert!(is_valid_syntax("df![x => [1, 2, 3]].std()"));
}

#[test]
fn test_pattern_matching_edge_cases() {
    // Pattern with underscore
    assert!(is_valid_syntax("match x { _ => 1 }"));

    // Pattern with literals
    assert!(is_valid_syntax("match x { 1 => a, 2 => b, _ => c }"));

    // Pattern with guards
    assert!(is_valid_syntax("match x { n if n > 0 => 1, _ => 0 }"));
}

#[test]
fn test_type_annotations() {
    // Simple types in struct definitions
    assert!(is_valid_syntax("struct Foo { x: i32 }"));

    // Nested types
    assert!(is_valid_syntax("struct Bar { items: Vec<String> }"));
}

#[test]
fn test_string_operations() {
    // String interpolation
    assert!(is_valid_syntax("\"Hello {name}\""));

    // Raw string
    assert!(is_valid_syntax("r\"Hello\\nWorld\""));

    // Multiline string
    assert!(is_valid_syntax("\"Line 1\nLine 2\""));

    // String with escaped quotes
    assert!(is_valid_syntax("\"Say \\\"Hello\\\"\""));
}

#[test]
fn test_control_flow_edge_cases() {
    // If without else
    assert!(is_valid_syntax("if x > 0 { 1 }"));

    // If-else if chain
    assert!(is_valid_syntax(
        "if x > 0 { 1 } else if x < 0 { -1 } else { 0 }"
    ));

    // While loop
    assert!(is_valid_syntax("while x > 0 { x - 1 }"));

    // For with range
    assert!(is_valid_syntax("for i in 0..10 { i }"));

    // For with inclusive range
    assert!(is_valid_syntax("for i in 0..=10 { i }"));
}

#[test]
fn test_operator_precedence() {
    // Arithmetic precedence
    assert!(is_valid_syntax("1 + 2 * 3"));
    assert!(is_valid_syntax("(1 + 2) * 3"));

    // Power operator
    assert!(is_valid_syntax("2 ** 3"));

    // Logical operators
    assert!(is_valid_syntax("true && false || true"));
    assert!(is_valid_syntax("!true && false"));

    // Comparison operators
    assert!(is_valid_syntax("1 < 2 && 3 > 2"));
    assert!(is_valid_syntax("x >= 5 || y <= 10"));

    // Bitwise operators
    assert!(is_valid_syntax("1 & 2 | 3"));
    assert!(is_valid_syntax("1 << 2 >> 1"));
    assert!(is_valid_syntax("~5 ^ 3"));
}

#[test]
fn test_lambda_expressions() {
    // Simple lambda
    assert!(is_valid_syntax("fn(x) { x + 1 }"));

    // Lambda with multiple params
    assert!(is_valid_syntax("fn(x, y) { x + y }"));
}

#[test]
fn test_collection_operations() {
    // List comprehension
    assert!(is_valid_syntax("[x * 2 for x in [1, 2, 3]]"));

    // List comprehension with filter
    assert!(is_valid_syntax("[x for x in [1, 2, 3] if x > 1]"));
}

#[test]
fn test_pipeline_operations() {
    // Basic pipeline
    assert!(is_valid_syntax("x |> f |> g"));
}

#[test]
fn test_struct_operations() {
    // Struct definition
    assert!(is_valid_syntax("struct Point { x: f64, y: f64 }"));

    // Struct literal
    assert!(is_valid_syntax("Point { x: 1.0, y: 2.0 }"));
}

#[test]
fn test_trait_operations() {
    // Trait definition with body
    assert!(is_valid_syntax("trait Drawable { }"));
}

#[test]
fn test_module_operations() {
    // Module definition
    assert!(is_valid_syntax("module Math { }"));

    // Module with content
    assert!(is_valid_syntax("module Math { 42 }"));

    // Export statement
    assert!(is_valid_syntax("export sqrt"));

    // Export multiple
    assert!(is_valid_syntax("export { sqrt, pow, log }"));
}

#[test]
fn test_import_operations() {
    // Simple import
    assert!(is_valid_syntax("import std::io"));

    // Import with items
    assert!(is_valid_syntax("import std::io::{Read, Write}"));

    // Import with alias
    assert!(is_valid_syntax("import std::collections::HashMap as Map"));

    // Import wildcard
    assert!(is_valid_syntax("import std::prelude::*"));
}

#[test]
fn test_error_handling() {
    // Try operator
    assert!(is_valid_syntax("f()?"));
}

#[test]
fn test_assignment_operations() {
    // Simple assignment
    assert!(is_valid_syntax("x = 5"));

    // Compound assignments
    assert!(is_valid_syntax("x += 5"));
    assert!(is_valid_syntax("x -= 5"));
    assert!(is_valid_syntax("x *= 5"));
    assert!(is_valid_syntax("x /= 5"));
    assert!(is_valid_syntax("x %= 5"));
    assert!(is_valid_syntax("x **= 2"));
    assert!(is_valid_syntax("x &= 5"));
    assert!(is_valid_syntax("x |= 5"));
    assert!(is_valid_syntax("x ^= 5"));
    assert!(is_valid_syntax("x <<= 2"));
    assert!(is_valid_syntax("x >>= 2"));
}

#[test]
fn test_increment_decrement() {
    // Post-increment/decrement
    assert!(is_valid_syntax("x++"));
    assert!(is_valid_syntax("x--"));

    // Pre-increment/decrement
    assert!(is_valid_syntax("++x"));
    assert!(is_valid_syntax("--x"));
}

#[test]
fn test_complex_expressions() {
    // Nested function calls
    assert!(is_valid_syntax("f(g(h(x)))"));

    // Method chaining
    assert!(is_valid_syntax("x.foo().bar().baz()"));

    // Array indexing
    assert!(is_valid_syntax("arr[0][1][2]"));

    // Mixed operations
    assert!(is_valid_syntax("(a + b) * c[0].method() |> f"));
}

#[test]
fn test_compile_operations() {
    // Test that compile generates valid Rust code
    assert!(compile("let x = 5").is_ok());
    assert!(compile("fn add(a, b) { a + b }").is_ok());
    assert!(compile("[1, 2, 3]").is_ok());
    assert!(compile("x + y * z").is_ok());
}
