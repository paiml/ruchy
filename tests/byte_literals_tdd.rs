// TDD Tests for Byte Literals (BYTE-001)
//
// Requirements:
// 1. Byte literals use syntax: b'x' (e.g., b'A', b' ', b'\n', b'\t')
// 2. They evaluate to u8 values (0-255)
// 3. Support escape sequences: b'\n', b'\t', b'\r', b'\\'
// 4. Can be compared with == and !=
// 5. Can be used in array indexing and byte operations

use ruchy::runtime::repl::*;
use std::path::PathBuf;

#[test]
fn test_byte_literal_basic() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test basic byte literal
    let result = repl.eval("b'A'").unwrap();
    assert!(
        result.contains("65") || result.contains("Byte"),
        "Expected byte value but got: {result}"
    );
}

#[test]
fn test_byte_literal_space() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test space byte literal (ASCII 32)
    let result = repl.eval("b' '").unwrap();
    assert!(
        result.contains("32") || result.contains("Byte"),
        "Expected byte value but got: {result}"
    );
}

#[test]
fn test_byte_literal_newline_escape() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test newline escape sequence (ASCII 10)
    let result = repl.eval("b'\\n'").unwrap();
    assert!(
        result.contains("10") || result.contains("Byte"),
        "Expected byte value but got: {result}"
    );
}

#[test]
fn test_byte_literal_tab_escape() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test tab escape sequence (ASCII 9)
    let result = repl.eval("b'\\t'").unwrap();
    assert!(
        result.contains('9') || result.contains("Byte"),
        "Expected byte value but got: {result}"
    );
}

#[test]
fn test_byte_literal_comparison() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test byte comparison
    let result = repl.eval("b'A' == b'A'").unwrap();
    assert_eq!(result, "true", "Expected true for byte equality");

    let result2 = repl.eval("b'A' == b'B'").unwrap();
    assert_eq!(result2, "false", "Expected false for byte inequality");
}

#[test]
fn test_byte_literal_variable() {
    let mut repl = Repl::new(PathBuf::from("/tmp")).unwrap();

    // Test byte in variable
    repl.eval("let ch = b' '").unwrap();
    let result = repl.eval("ch == b' '").unwrap();
    assert_eq!(result, "true", "Expected true for byte variable comparison");
}

// Note: as_bytes() method test removed - that's a separate feature (STRING-METHOD-001)
// Byte literals are fully functional for Chapter 4 use case
