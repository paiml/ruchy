//! TDD Test for BOOK-CH15-002: Multi-statement function body parsing
//!
//! Root Cause: `parse_function` uses `parse_expr_recursive` which only handles single expressions
//! Solution: Use `parse_block` when function body starts with {

use ruchy::Parser;

#[test]
fn test_function_with_multiple_let_statements() {
    // Example 3 from Chapter 15: Data Processor
    let code = r"
        fun calculate_sum(data: Vec<i32>) -> i32 {
            let mut total = 0;
            let mut i = 0;
            while i < data.len() {
                total = total + data[i];
                i = i + 1;
            }
            total
        }
    ";

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse function with multiple let statements: {:?}",
        result.err()
    );
}

#[test]
fn test_function_with_else_if_chain() {
    // Example 2 from Chapter 15: Calculator
    let code = r#"
        fun evaluate_expression(expr: String, a: f64, b: f64) -> f64 {
            if expr == "+" {
                a + b
            } else if expr == "-" {
                a - b
            } else if expr == "*" {
                a * b
            } else if expr == "/" {
                a / b
            } else {
                0.0
            }
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse function with else-if chain: {:?}",
        result.err()
    );
}

#[test]
fn test_function_with_multiple_statements() {
    // General multi-statement function
    let code = r#"
        fun example() {
            let x = 1;
            let y = 2;
            let z = x + y;
            println("Result: {}", z);
            z
        }
    "#;

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Failed to parse function with multiple statements: {:?}",
        result.err()
    );
}

#[test]
fn test_single_expression_function_still_works() {
    // Ensure backward compatibility with single expression functions
    let code = r"
        fun add(x: i32, y: i32) -> i32 { x + y }
    ";

    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Single expression function should still work: {:?}",
        result.err()
    );
}

#[test]
fn test_function_without_braces() {
    // Some languages allow: fun f() = expr
    // But Ruchy requires braces - this should fail gracefully
    let code = "fun add(x: i32, y: i32) -> i32 x + y";

    let mut parser = Parser::new(code);
    let result = parser.parse();

    // This should fail - we require braces
    assert!(
        result.is_err(),
        "Function without braces should fail to parse"
    );
}
