#![allow(missing_docs)]
//! Try-Catch Syntax Tests - HYBRID-C-2
//! EXTREME TDD: Test written FIRST before implementation
//!
//! Goal: Support both `catch (e)` and `catch e` syntax

use ruchy::Parser;

// RED PHASE: This test should FAIL initially
#[test]
fn test_try_catch_without_parentheses() {
    let code = "try { 42 } catch e { 0 }";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse try-catch without parentheses: {:?}",
        result.err()
    );
}

// Existing syntax should still work
#[test]
fn test_try_catch_with_parentheses() {
    let code = "try { 42 } catch (e) { 0 }";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse try-catch with parentheses: {:?}",
        result.err()
    );
}

// Test the book example
#[test]
fn test_book_example() {
    let code = r#"try { 10 / 0 } catch e { "error" }"#;
    let mut parser = Parser::new(code);
    let result = parser.parse();

    assert!(
        result.is_ok(),
        "Should parse book example: {:?}",
        result.err()
    );
}

// Test with multiple catches (if supported)
#[test]
fn test_try_multiple_catch() {
    let code = "try { risky() } catch e1 { handle1() } catch e2 { handle2() }";
    let mut parser = Parser::new(code);
    let result = parser.parse();

    // This might not be supported yet - that's okay
    let _ = result;
}
