// EVALUATOR-002: Method chaining with array indexing
// Test ticket: tests/evaluator_002_method_chaining_with_indexing.rs
// Traceability: docs/execution/roadmap.yaml:4465-4562
//
// Bug: Method chaining with array indexing returns empty string
// Example: html.select('.content')[0].text() returns "" instead of text content
// Works when split: elements = html.select('.content'); elements[0].text()
//
// Root Cause: Interpreter loses value when chaining through array indexing
//
// EXTREME TDD: RED → GREEN → REFACTOR
// RED Phase: All tests FAIL initially (proving bug exists)
// GREEN Phase: Fix makes all tests PASS
// REFACTOR Phase: Property tests + quality gates

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::Interpreter;

// Helper to strip quotes from Value::String display format
fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"')
}

// =============================================================================
// Section 1: Basic Method Chaining (RED)
// =============================================================================

#[test]
fn test_evaluator002_01_method_then_index() {
    let code = r#"
fun get_items() {
    ["apple", "banana", "cherry"]
}

fun main() {
    let item = get_items()[0]
    item
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Execution should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "apple",
        "Method call followed by indexing should return first element"
    );
}

#[test]
fn test_evaluator002_02_method_then_index_then_method() {
    let code = r#"
fun get_items() {
    ["apple", "banana", "cherry"]
}

fun main() {
    let first = get_items()[0].to_uppercase()
    first
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Execution should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "APPLE",
        "Chaining method → index → method should work"
    );
}

#[test]
fn test_evaluator002_03_multiple_index_operations() {
    let code = r#"
fun get_nested() {
    [["a", "b"], ["c", "d"]]
}

fun main() {
    let value = get_nested()[1][0]
    value
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Execution should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "c",
        "Multiple indexing operations should work"
    );
}

// =============================================================================
// Section 2: HTML-Specific Chains (SKIPPED - requires http stdlib)
// =============================================================================

// Note: HTML tests skipped because http module is not available in test environment
// The core functionality (function call → index) is tested in other tests

// =============================================================================
// Section 3: General Array Chains (RED)
// =============================================================================

#[test]
fn test_evaluator002_06_array_method_index_method() {
    let code = r#"
fun create_array() {
    [10, 20, 30]
}

fun main() {
    let doubled = create_array()[1] * 2
    doubled
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Execution should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "40",
        "Array method → index → operation should work"
    );
}

#[test]
fn test_evaluator002_07_nested_arrays_with_chaining() {
    let code = r#"
fun create_nested() {
    [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
}

fun main() {
    let sum = create_nested()[1][0] + create_nested()[1][1]
    sum
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Execution should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "9",
        "Nested array access should work (4 + 5 = 9)"
    );
}

// =============================================================================
// Section 4: Split vs Chained (PROOF)
// =============================================================================

#[test]
fn test_evaluator002_08_split_works_chained_fails() {
    // Test 1: Split version (should work)
    let split_code = r#"
fun get_items() {
    ["first", "second", "third"]
}

fun main() {
    let items = get_items()
    let item = items[1]
    item
}

main()
"#;

    let mut parser1 = Parser::new(split_code);
    let ast1 = parser1.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast1).expect("Split version should work");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "second",
        "Split version: Proves the bug is in chaining, not in basic indexing"
    );

    // Test 2: Chained version (currently fails - this is the bug)
    let chained_code = r#"
fun get_items() {
    ["first", "second", "third"]
}

fun main() {
    let item = get_items()[1]
    item
}

main()
"#;

    let mut parser2 = Parser::new(chained_code);
    let ast2 = parser2.parse().expect("Parse should succeed");
    let mut interpreter2 = Interpreter::new();
    let result2 = interpreter2.eval_expr(&ast2).expect("Chained version should eventually work");

    assert_eq!(
        strip_quotes(&result2.to_string()),
        "second",
        "Chained version: Should work identically to split version"
    );
}

#[test]
fn test_evaluator002_09_debug_intermediate_values() {
    let code = r#"
fun get_array() {
    ["debug1", "debug2", "debug3"]
}

fun main() {
    // Test intermediate values
    let array = get_array()
    println("Array created")

    let indexed = array[0]
    println("Indexed: " + indexed)

    // Now test chained
    let chained = get_array()[0]
    println("Chained: " + chained)

    chained
}

main()
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval_expr(&ast).expect("Debug test should succeed");

    assert_eq!(
        strip_quotes(&result.to_string()),
        "debug1",
        "Chained indexing should return same value as split indexing"
    );
}
// debug_ast_structure test removed - was only for debugging during development
