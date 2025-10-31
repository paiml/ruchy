// CRITICAL-FMT-DATA-LOSS Regression Test (GitHub Issue #64)
// RED Phase: Prove formatter deletes code (bug reproduced)
// GREEN Phase: Fix with minimal changes
// REFACTOR Phase: Redesign architecture to prevent recurrence

#![allow(clippy::similar_names)] // formatter/formatted are acceptable in test code
#![allow(missing_docs)]
#![allow(clippy::ignore_without_reason)] // RED phase tests intentionally ignored
#![allow(missing_docs)]

use ruchy::frontend::parser::Parser;
use ruchy::quality::formatter::Formatter;

// Helper to strip quotes from Value::String display format
fn normalize_whitespace(s: &str) -> String {
    s.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

// =============================================================================
// RED PHASE: Failing Tests (Reproduce Bug)
// =============================================================================

#[test]
fn test_formatter_data_loss_01_nested_let_statements() {
    // Bug: Formatter deletes nested let statements
    let code = r#"
fun test() {
    let x = 1
    let y = 2
    let z = 3
    println("hello")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Verify all 3 let statements are preserved
    assert!(formatted.contains("let x = 1"), "Formatter deleted 'let x = 1': {formatted}");
    assert!(formatted.contains("let y = 2"), "Formatter deleted 'let y = 2': {formatted}");
    assert!(formatted.contains("let z = 3"), "Formatter deleted 'let z = 3': {formatted}");
    assert!(formatted.contains("println"), "Formatter deleted println call: {formatted}");
}

#[test]
fn test_formatter_data_loss_02_if_else_blocks() {
    // Bug: Formatter deletes if/else branches
    let code = r#"
fun test() {
    let x = 10
    if x > 5 {
        println("greater")
    } else {
        println("lesser")
    }
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    assert!(formatted.contains("let x = 10"), "Formatter deleted let statement: {formatted}");
    assert!(formatted.contains("if"), "Formatter deleted if keyword: {formatted}");
    assert!(formatted.contains("greater"), "Formatter deleted then branch: {formatted}");
    assert!(formatted.contains("else"), "Formatter deleted else keyword: {formatted}");
    assert!(formatted.contains("lesser"), "Formatter deleted else branch: {formatted}");
}

#[test]
fn test_formatter_data_loss_03_multiple_statements_after_let() {
    // Bug: Formatter deletes statements after let
    let code = r#"
fun test() {
    let unused = detect_unused("path")
    let count = unused.len()
    println("Found: " + count.to_string())
    if count > 0 {
        println("Example: " + unused[0])
    }
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Verify NO code deletion
    assert!(formatted.contains("let unused"), "Deleted: let unused");
    assert!(formatted.contains("let count"), "Deleted: let count");
    assert!(formatted.contains("Found:"), "Deleted: println statement");
    assert!(formatted.contains("if count > 0"), "Deleted: if statement");
    assert!(formatted.contains("Example:"), "Deleted: if body");
}

#[test]
#[ignore = "TODO: Parser doesn't attach top-level comments to AST - separate issue"]
fn test_formatter_data_loss_04_comments_preserved() {
    // Bug: Formatter deletes comments
    let code = r"
// Header comment
fun test() {
    // Inline comment
    let x = 1
    // Another comment
    let y = 2
}
";

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Comments should be preserved
    assert!(formatted.contains("Header comment"), "Deleted: header comment");
    assert!(formatted.contains("Inline comment"), "Deleted: inline comment");
    assert!(formatted.contains("Another comment"), "Deleted: another comment");
}

// =============================================================================
// PROPERTY TESTS: Invariant Enforcement
// =============================================================================

#[test]
fn test_formatter_property_01_idempotence() {
    // Property: format(format(x)) == format(x)
    let code = r#"
fun test() {
    let x = 1
    let y = 2
    println("test")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted1 = formatter.format(&ast).expect("First format should succeed");

    // Format the formatted code again
    let mut parser2 = Parser::new(&formatted1);
    let ast2 = parser2.parse().expect("Formatted code should parse");
    let formatted2 = formatter.format(&ast2).expect("Second format should succeed");

    // Idempotence: second format should be identical to first
    assert_eq!(
        normalize_whitespace(&formatted1),
        normalize_whitespace(&formatted2),
        "Formatter is not idempotent! First:\n{formatted1}\n\nSecond:\n{formatted2}"
    );
}

#[test]
fn test_formatter_property_02_ast_node_count() {
    // Property: Formatter should never decrease AST node count
    let code = r#"
fun test() {
    let x = 1
    let y = 2
    let z = 3
    println("hello")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let original_node_count = count_ast_nodes(&ast);

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Parse formatted output
    let mut parser2 = Parser::new(&formatted);
    let ast2 = parser2.parse().expect("Formatted code should parse");
    let formatted_node_count = count_ast_nodes(&ast2);

    // Formatter should NEVER decrease node count (that means code deletion)
    assert!(
        formatted_node_count >= original_node_count,
        "Formatter DELETED AST nodes! Original: {original_node_count}, Formatted: {formatted_node_count}. Code:\n{formatted}"
    );
}

#[test]
fn test_formatter_property_03_semantic_equivalence() {
    // Property: format(code) should evaluate to same result as code
    let code = r"
fun main() {
    let x = 10
    let y = 20
    let z = x + y
    z
}

main()
";

    // Evaluate original code
    let mut parser1 = Parser::new(code);
    let ast1 = parser1.parse().expect("Parse should succeed");
    let mut interp1 = ruchy::runtime::interpreter::Interpreter::new();
    let result1 = interp1.eval_expr(&ast1).expect("Original should execute");

    // Format code
    let formatter = Formatter::new();
    let formatted = formatter.format(&ast1).expect("Format should succeed");

    // Evaluate formatted code
    let mut parser2 = Parser::new(&formatted);
    let ast2 = parser2.parse().expect("Formatted code should parse");
    let mut interp2 = ruchy::runtime::interpreter::Interpreter::new();
    let result2 = interp2.eval_expr(&ast2).expect("Formatted should execute");

    // Results must be identical
    assert_eq!(
        result1.to_string(),
        result2.to_string(),
        "Formatter changed semantics! Original result: {result1}, Formatted result: {result2}. Formatted code:\n{formatted}"
    );
}

#[test]
fn test_formatter_property_04_roundtrip_parse() {
    // Property: parse(format(parse(code))) should succeed
    let code = r#"
fun test() {
    let x = 1
    let y = 2
    println("test")
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Formatted code MUST be valid Ruchy code
    let mut parser2 = Parser::new(&formatted);
    let result = parser2.parse();

    assert!(
        result.is_ok(),
        "Formatter produced INVALID Ruchy code! Error: {:?}. Code:\n{}",
        result.err(),
        formatted
    );
}

// =============================================================================
// REGRESSION TEST: RuchyRuchy dead_code_simple_test.ruchy
// =============================================================================

#[test]
fn test_formatter_regression_ruchyruchy_dead_code_test() {
    // This is the EXACT file from RuchyRuchy that triggered the bug report
    let code = r#"
// QUALITY-002: Dead Code Detection - REFACTOR Phase
fun main() {
    println("QUALITY-002: Dead Code Detection - Simple Test")
    test_unused_functions()
}

fun test_unused_functions() {
    println("Test 1: Unused function detection")
    let unused = detect_unused_functions("bootstrap/stage0/")
    let count = unused.len()
    println("Found " + count.to_string() + " unused functions")
    if count > 0 {
        println("Example: " + unused[0])
        println("PASS - Detected unused functions")
    } else {
        println("FAIL - Expected to find unused functions")
    }
}

fun detect_unused_functions(path: String) -> Vec<String> {
    let unused = Vec.new()
    unused.push("unused_helper_function")
    return unused
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse should succeed");
    let original_node_count = count_ast_nodes(&ast);

    let formatter = Formatter::new();
    let formatted = formatter.format(&ast).expect("Format should succeed");

    // Parse formatted output
    let mut parser2 = Parser::new(&formatted);
    let ast2 = parser2.parse().expect("Formatted code should parse");
    let formatted_node_count = count_ast_nodes(&ast2);

    // Verify NO code deletion
    assert!(
        formatted_node_count >= original_node_count,
        "Formatter DELETED {} AST nodes! Original: {}, Formatted: {}",
        original_node_count - formatted_node_count,
        original_node_count,
        formatted_node_count
    );

    // Verify specific statements preserved
    assert!(formatted.contains("let unused"), "Deleted: let unused");
    assert!(formatted.contains("let count"), "Deleted: let count");
    assert!(formatted.contains("Found"), "Deleted: println");
    assert!(formatted.contains("if count > 0"), "Deleted: if statement");
    assert!(formatted.contains("Example:"), "Deleted: if body");
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Count total AST nodes (expressions) in a tree
fn count_ast_nodes(expr: &ruchy::frontend::ast::Expr) -> usize {
    use ruchy::frontend::ast::ExprKind;

    1 + match &expr.kind {
        ExprKind::Let { value, body, .. } => {
            count_ast_nodes(value) + count_ast_nodes(body)
        }
        ExprKind::Binary { left, right, .. } => {
            count_ast_nodes(left) + count_ast_nodes(right)
        }
        ExprKind::Block(exprs) => {
            exprs.iter().map(count_ast_nodes).sum()
        }
        ExprKind::Function { body, .. } => {
            count_ast_nodes(body)
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            count_ast_nodes(condition)
                + count_ast_nodes(then_branch)
                + else_branch.as_ref().map_or(0, |e| count_ast_nodes(e))
        }
        ExprKind::Call { func, args } => {
            count_ast_nodes(func) + args.iter().map(count_ast_nodes).sum::<usize>()
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            count_ast_nodes(receiver) + args.iter().map(count_ast_nodes).sum::<usize>()
        }
        ExprKind::IndexAccess { object, index } => {
            count_ast_nodes(object) + count_ast_nodes(index)
        }
        ExprKind::While { condition, body, .. } => {
            count_ast_nodes(condition) + count_ast_nodes(body)
        }
        ExprKind::Match { expr, arms } => {
            count_ast_nodes(expr) + arms.iter().map(|arm| count_ast_nodes(&arm.body)).sum::<usize>()
        }
        _ => 0, // Literals, identifiers, etc. have no children
    }
}
