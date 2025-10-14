//! BUG-031: ruchy fmt corrupts files - writes AST instead of formatted code
//!
//! Issue: https://github.com/paiml/ruchy/issues/31
//! Severity: CRITICAL - Data loss
//!
//! Root Cause: Formatter fallback uses Debug format for unimplemented ExprKind variants
//! Location: src/quality/formatter.rs:146
//!
//! Test Strategy: EXTREME TDD (RED → GREEN → REFACTOR)

use ruchy::quality::formatter::Formatter;
use ruchy::Parser;

/// RED TEST: Formatter should format function calls correctly, not output AST
#[test]
fn test_bug_031_function_call_corrupts_file() {
    let code = r#"
fun main() {
    println("Hello, World!")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let formatter = Formatter::new();
    let result = formatter.format(&ast).expect("Failed to format");

    // Should contain valid Ruchy code
    assert!(result.contains("println"), "Formatted output should contain 'println'");
    assert!(result.contains("Hello, World!"), "Formatted output should contain string literal");

    // Should NOT contain AST debug output
    assert!(!result.contains("Call {"), "Formatted output should NOT contain 'Call {{' (AST debug)");
    assert!(!result.contains("Expr {"), "Formatted output should NOT contain 'Expr {{' (AST debug)");
    assert!(!result.contains("kind:"), "Formatted output should NOT contain 'kind:' (AST debug)");
}

/// RED TEST: Formatter should handle method calls
#[test]
fn test_bug_031_method_call_formatting() {
    let code = r#"
fun example() {
    let x = "test".to_uppercase()
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let formatter = Formatter::new();
    let result = formatter.format(&ast).expect("Failed to format");

    // Should contain valid Ruchy code
    assert!(result.contains("to_uppercase"), "Should contain method name");

    // Should NOT contain AST debug output
    assert!(!result.contains("MethodCall {"), "Should NOT contain 'MethodCall {{' (AST debug)");
}

/// RED TEST: Formatter should handle for loops with range
#[test]
fn test_bug_031_for_loop_formatting() {
    let code = r#"
fun example() {
    for i in range(0, 10) {
        println(i)
    }
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let formatter = Formatter::new();
    let result = formatter.format(&ast).expect("Failed to format");

    // Should contain valid Ruchy code
    assert!(result.contains("for"), "Should contain 'for' keyword");
    assert!(result.contains("range"), "Should contain 'range' function");

    // Should NOT contain AST debug output
    assert!(!result.contains("For {"), "Should NOT contain 'For {{' (AST debug)");
}

/// RED TEST: Formatter should handle variable assignments
#[test]
fn test_bug_031_assignment_formatting() {
    let code = r#"
fun example() {
    let x = 42
    let y = "hello"
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let formatter = Formatter::new();
    let result = formatter.format(&ast).expect("Failed to format");

    // Should contain valid Ruchy code
    assert!(result.contains("let x"), "Should contain 'let x'");
    assert!(result.contains("42"), "Should contain integer literal");
    assert!(result.contains("hello"), "Should contain string literal");

    // Should NOT contain AST debug output
    assert!(!result.contains("Assign {"), "Should NOT contain 'Assign {{' (AST debug)");
}

/// Property test: Formatter should NEVER output Debug representation
#[test]
fn property_formatter_never_outputs_debug_format() {
    let test_cases = vec![
        "fun f() { println(\"test\") }",
        "fun f() { let x = 1 }",
        "fun f() { for i in range(0, 5) { println(i) } }",
        "fun f() { if x { y } else { z } }",
        "fun f() { x.method() }",
        "fun f() { array[0] }",
    ];

    for code in test_cases {
        let mut parser = Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let formatter = Formatter::new();
            if let Ok(result) = formatter.format(&ast) {
                // Should never contain AST debug markers
                assert!(!result.contains(" {"),
                    "Code: {}\nFormatted output should not contain AST debug format", code);
                assert!(!result.contains("kind:"),
                    "Code: {}\nFormatted output should not contain 'kind:' field", code);
            }
        }
    }
}
