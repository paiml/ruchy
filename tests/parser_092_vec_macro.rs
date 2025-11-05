//! PARSER-092: vec![] Macro Syntax Support
//!
//! EXTREME TDD - RED Phase Tests
//!
//! Purpose: Add vec![] macro syntax to parser (Issue #137)
//! Target: Enable idiomatic Rust vector initialization in Ruchy
//!
//! Test Strategy:
//! 1. Basic syntax - vec![0u8; 1024] (repeat pattern)
//! 2. Element list - vec![1, 2, 3] (comma-separated elements)
//! 3. Type annotations - vec![0i32; 10]
//! 4. Expression elements - vec![x * 2, y + 1, z]
//! 5. Empty vectors - vec![]
//! 6. Nested vectors - vec![vec![0; 5]; 10]
//! 7. Complex expressions - vec![compute(i); n]
//! 8. Integration - vec![] in functions, assignments, returns

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]

use ruchy::Parser;

// ============================================================================
// RED-001: Basic vec![] Syntax (Repeat Pattern)
// ============================================================================

#[test]
fn test_parser_092_vec_repeat_u8() {
    // Test: vec![0u8; 1024] - byte buffer pattern from Issue #137
    let code = r"
        let buffer = vec![0u8; 1024];
        buffer
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![0u8; 1024]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_repeat_i32() {
    // Test: vec![0i32; 10] - typed integer vector
    let code = r"
        let numbers = vec![0i32; 10];
        numbers
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![0i32; 10]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_repeat_literal() {
    // Test: vec![5; 100] - repeat literal value
    let code = r"
        let fives = vec![5; 100];
        fives
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![5; 100]: {:?}", result.err());
}

// ============================================================================
// RED-002: Element List Syntax
// ============================================================================

#[test]
fn test_parser_092_vec_element_list_simple() {
    // Test: vec![1, 2, 3] - basic element list
    let code = r"
        let numbers = vec![1, 2, 3];
        numbers
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![1, 2, 3]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_element_list_single() {
    // Test: vec![42] - single element
    let code = r"
        let singleton = vec![42];
        singleton
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![42]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_element_list_many() {
    // Test: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10] - many elements
    let code = r"
        let range = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        range
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![1..10]: {:?}", result.err());
}

// ============================================================================
// RED-003: Expression Elements
// ============================================================================

#[test]
fn test_parser_092_vec_expression_elements() {
    // Test: vec![x, y, z] - variable elements
    let code = r"
        let x = 1;
        let y = 2;
        let z = 3;
        let coords = vec![x, y, z];
        coords
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![x, y, z]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_computed_elements() {
    // Test: vec![x * 2, y + 1, z / 3] - computed expressions
    let code = r"
        let x = 1;
        let y = 2;
        let z = 3;
        let computed = vec![x * 2, y + 1, z / 3];
        computed
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![x*2, y+1, z/3]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_repeat_expression() {
    // Test: vec![x * 2; 5] - repeat computed expression
    let code = r"
        let x = 10;
        let repeated = vec![x * 2; 5];
        repeated
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![x*2; 5]: {:?}", result.err());
}

// ============================================================================
// RED-004: Empty Vectors
// ============================================================================

#[test]
fn test_parser_092_vec_empty() {
    // Test: vec![] - empty vector
    let code = r"
        let empty = vec![];
        empty
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![]: {:?}", result.err());
}

// ============================================================================
// RED-005: Nested Vectors
// ============================================================================

#[test]
fn test_parser_092_vec_nested_simple() {
    // Test: vec![vec![0; 5]; 10] - nested initialization
    let code = r"
        let matrix = vec![vec![0; 5]; 10];
        matrix
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![vec![0; 5]; 10]: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_nested_element_list() {
    // Test: vec![vec![1, 2], vec![3, 4]] - nested element lists
    let code = r"
        let pairs = vec![vec![1, 2], vec![3, 4]];
        pairs
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![vec![1,2], vec![3,4]]: {:?}", result.err());
}

// ============================================================================
// RED-006: Integration Tests (Functions, Returns)
// ============================================================================

#[test]
fn test_parser_092_vec_in_function() {
    // Test: vec![] used inside function
    let code = r"
        fun create_buffer(size: usize) -> Vec<u8> {
            vec![0u8; size]
        }
        create_buffer(1024)
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![] in function: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_as_argument() {
    // Test: vec![] passed as function argument
    let code = r"
        fun process(data: Vec<i32>) -> i32 {
            42
        }
        process(vec![1, 2, 3])
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![] as argument: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_issue_137_reproduction() {
    // Test: Exact reproduction from Issue #137
    // This is the pattern that currently fails in ruchy-lambda
    let code = r#"
        fun http_post(path: &str, body: &str) -> bool {
            let mut buffer = vec![0u8; 1024];
            true
        }
        http_post("/api/invoke", "data")
    "#;
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse Issue #137 example: {:?}", result.err());
}

// ============================================================================
// RED-007: Edge Cases
// ============================================================================

#[test]
fn test_parser_092_vec_trailing_comma() {
    // Test: vec![1, 2, 3,] - trailing comma
    let code = r"
        let numbers = vec![1, 2, 3,];
        numbers
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse vec![1,2,3,] with trailing comma: {:?}", result.err());
}

#[test]
fn test_parser_092_vec_multiline() {
    // Test: vec![] with multiline formatting
    let code = r"
        let numbers = vec![
            1,
            2,
            3
        ];
        numbers
    ";
    let result = Parser::new(code).parse();

    assert!(result.is_ok(), "Should parse multiline vec![]: {:?}", result.err());
}
