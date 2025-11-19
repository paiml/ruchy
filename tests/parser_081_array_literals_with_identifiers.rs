#![allow(missing_docs)]
// PARSER-081: Array Literals with Identifiers (GitHub Issue #62)
//
// RED phase test - These tests MUST FAIL initially, proving the bug exists
//
// Testing Strategy (Extreme TDD):
// 1. RED: Write 10 failing tests demonstrating parser limitation
// 2. GREEN: Fix parser to support identifiers in arrays
// 3. REFACTOR: Add property tests ensuring array parsing completeness
//
// BUG: Parser only supports numeric literals in arrays [1, 2, 3]
//      but fails on identifiers [title, count] or mixed [1, x, 3]

use ruchy::frontend::parser::Parser;

// ===========================
// Section 1: Array Literals with Variables (RED - Should FAIL)
// ===========================

#[test]
fn test_parser081_01_array_with_single_variable() {
    // BUG: Parser should support single variable in array
    let code = r"
fun test() {
    let x = 42
    [x]
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with variable in array");

    // Verify we got a Function with array containing identifier
    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array containing variable"
    );
}

#[test]
fn test_parser081_02_array_with_two_variables() {
    // BUG: Parser should support multiple variables in array
    let code = r#"
fun test() {
    let title = "Hello"
    let count = 42
    [title, count]
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with two variables in array");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array containing two variables"
    );
}

#[test]
fn test_parser081_03_array_with_three_variables() {
    // BUG: Parser should support three or more variables in array
    let code = r"
fun test() {
    let a = 1
    let b = 2
    let c = 3
    [a, b, c]
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with three variables in array");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array containing three variables"
    );
}

#[test]
fn test_parser081_04_array_mixed_literals_and_variables() {
    // BUG: Parser should support mixing literals and variables
    let code = r"
fun test() {
    let x = 42
    [1, x, 3]
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with mixed array elements");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array mixing literals and variables"
    );
}

#[test]
fn test_parser081_05_nested_array_with_variables() {
    // BUG: Parser should support nested arrays with variables
    let code = r"
fun test() {
    let x = 1
    let y = 2
    [[x], [y]]
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with nested arrays containing variables");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with nested arrays containing variables"
    );
}

// ===========================
// Section 2: Array Literals in Context (RED - Should FAIL)
// ===========================

#[test]
fn test_parser081_06_array_in_function_return() {
    // BUG: Parser should support returning array with variables
    let code = r"
fun get_pair(a, b) {
    [a, b]
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with array of parameters as return value");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function returning array with parameters"
    );
}

#[test]
fn test_parser081_07_array_in_let_binding() {
    // BUG: Parser should support let binding with array of variables
    let code = r"
fun test() {
    let x = 10
    let y = 20
    let pair = [x, y]
    pair
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with array in let binding");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array assigned to variable"
    );
}

#[test]
fn test_parser081_08_array_passed_to_function() {
    // BUG: Parser should support passing array with variables to function
    let code = r"
fun test() {
    let x = 1
    let y = 2
    process([x, y])
}
";

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with array passed to function");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array as function argument"
    );
}

// ===========================
// Section 3: Edge Cases (RED - Should FAIL)
// ===========================

#[test]
fn test_parser081_09_array_with_method_calls() {
    // BUG: Parser should support method calls in arrays
    let code = r#"
fun test() {
    let x = "hello"
    [x.len(), x.to_uppercase()]
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser
        .parse()
        .expect("Parse should succeed with method calls in array");

    assert!(
        matches!(ast.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Should parse function with array containing method calls"
    );
}

#[test]
fn test_parser081_10_array_with_field_access() {
    // BUG: Parser should support field access in arrays
    let code = r"
struct Point { x: f64, y: f64 }

fun test() {
    let p = Point { x: 10.0, y: 20.0 }
    [p.x, p.y]
}
";

    let mut parser = Parser::new(code);
    let _ast = parser
        .parse()
        .expect("Parse should succeed with field access in array");
    // Test passes if parsing succeeds - the fix allows [p.x, p.y] to be parsed as array literal
}
