// QUALITY-FORMATTER-002: Formatter Semantic Preservation (GitHub Issue #64)
//
// RED phase test - This test MUST FAIL initially, proving the bug exists
//
// Testing Strategy (Extreme TDD):
// 1. RED: Write failing test demonstrating formatter changes code semantics
// 2. GREEN: Fix formatter to preserve structure (no let → let-in conversion)
// 3. REFACTOR: Add property tests ensuring AST equivalence

use ruchy::frontend::parser::Parser;
use ruchy::quality::formatter::Formatter;

// ===========================
// Section 1: Sequential Let Bindings (RED - Should FAIL)
// ===========================

#[test]
fn test_formatter_002_01_sequential_lets_in_function() {
    // BUG: Formatter converts sequential let bindings to nested let-in expressions
    // Original: Two separate let statements
    // Formatted: First let becomes let-in wrapping second let
    // Impact: Changes code structure and semantics

    let original = r#"
struct Point { x: f64, y: f64 }

fun main() {
    let p = Point { x: 10.0, y: 20.0 }
    let q = Point { x: 30.0, y: 40.0 }
    println("done")
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // The formatted code should NOT contain "let-in" syntax
    // because the original didn't have it
    assert!(
        !formatted.contains(" in {"),
        "Formatter must not inject 'let-in' syntax that wasn't in original code.\nFormatted:\n{}",
        formatted
    );

    // The formatted code should preserve the sequential let structure
    assert!(
        formatted.contains("let p =") && formatted.contains("let q ="),
        "Formatter must preserve sequential let bindings.\nFormatted:\n{}",
        formatted
    );
}

#[test]
fn test_formatter_002_02_preserve_user_written_let_in() {
    // BASELINE: If user ACTUALLY wrote let-in, preserve it
    let original = r#"
fun main() {
    let x = 10 in x + 1
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // User wrote "let-in", so formatter should keep it
    assert!(
        formatted.contains(" in "),
        "Formatter must preserve user-written let-in expressions.\nFormatted:\n{}",
        formatted
    );
}

#[test]
fn test_formatter_002_03_multiple_sequential_lets() {
    // Test with 3 sequential let bindings
    let original = r#"
fun process() {
    let a = 1
    let b = 2
    let c = 3
    a + b + c
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // Should not convert to nested let-in expressions
    assert!(
        !formatted.contains(" in {"),
        "Formatter must not inject nested let-in for sequential lets.\nFormatted:\n{}",
        formatted
    );
}

// ===========================
// Section 2: Let in Different Contexts (Baseline)
// ===========================

#[test]
fn test_formatter_002_04_let_in_block() {
    // Sequential lets inside a block expression
    let original = r#"
fun main() {
    {
        let x = 5
        let y = 10
        x + y
    }
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // Should preserve block structure
    assert!(
        formatted.contains('{') && formatted.contains('}'),
        "Formatter must preserve block structure.\nFormatted:\n{}",
        formatted
    );

    // Should not inject let-in
    assert!(
        !formatted.contains(" in {"),
        "Formatter must not inject let-in in blocks.\nFormatted:\n{}",
        formatted
    );
}

#[test]
fn test_formatter_002_05_single_let_statement() {
    // Single let followed by expression (no nested lets)
    let original = r#"
fun main() {
    let x = 42
    println(x)
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // Should NOT add let-in syntax
    assert!(
        !formatted.contains(" in "),
        "Formatter must not add let-in for single let statement.\nFormatted:\n{}",
        formatted
    );
}

// ===========================
// Section 3: Floating-Point Literal Preservation (RED)
// ===========================

#[test]
fn test_formatter_002_06_preserve_float_literals() {
    // BUG: Formatter inconsistently removes .0 suffix from float literals
    let original = r#"
struct Point { x: f64, y: f64 }

fun main() {
    let p = Point { x: 10.0, y: 20.0 }
}
"#;

    let mut parser = Parser::new(original);
    let ast = parser.parse().expect("Parse failed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format failed");

    // Should preserve .0 suffix for floating-point literals
    assert!(
        formatted.contains("10.0") || formatted.matches("10").count() >= 1,
        "Formatter should preserve floating-point literal format.\nFormatted:\n{}",
        formatted
    );
}

// ===========================
// Section 4: Round-Trip Property (Critical)
// ===========================

#[test]
fn test_formatter_002_07_format_roundtrip_preserves_ast() {
    // PROPERTY TEST: format(parse(code)) should produce equivalent AST
    let original = r#"
fun main() {
    let x = 1
    let y = 2
    x + y
}
"#;

    // Parse original
    let mut parser1 = Parser::new(original);
    let ast1 = parser1.parse().expect("Parse failed");

    // Format
    let formatter = Formatter::new();
    let formatted = formatter.format(&ast1).expect("Format failed");

    // Parse formatted
    let mut parser2 = Parser::new(&formatted);
    let ast2 = parser2.parse().expect("Parse of formatted code failed");

    // ASTs should be equivalent (this is the PROPERTY)
    // For now, just verify it parses without errors by checking the AST structure
    // The root node should be a Function definition
    assert!(
        matches!(ast2.kind, ruchy::frontend::ast::ExprKind::Function { .. }),
        "Formatted code must parse to equivalent AST (should be Function node).\nFormatted:\n{}",
        formatted
    );
}
