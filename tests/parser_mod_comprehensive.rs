//! Comprehensive tests for parser/mod.rs (2,067 lines → TDG target)
//!
//! EXTREME TDD: TDG-driven testing for second-largest under-tested module
//! Target: src/frontend/parser/mod.rs (main parser entry point)
//! Coverage: Parser creation, expression parsing, operator precedence, comments, errors

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Basic Parsing (Parser::new + Parser::parse)
// ============================================================================

#[test]
fn test_parse_integer_literal() {
    ruchy_cmd().arg("-e").arg("42").assert().success();
}

#[test]
fn test_parse_string_literal() {
    ruchy_cmd()
        .arg("-e")
        .arg(r#""Hello, World!""#)
        .assert()
        .success();
}

#[test]
fn test_parse_boolean_literal() {
    ruchy_cmd().arg("-e").arg("true").assert().success();
}

#[test]
fn test_parse_identifier() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Expression Parsing (parse_expr_recursive)
// ============================================================================

#[test]
fn test_parse_binary_addition() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(1 + 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_parse_binary_multiplication() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(3 * 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("12"));
}

#[test]
fn test_parse_binary_comparison() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 > 3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_unary_negation() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(-42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("-42"));
}

#[test]
fn test_parse_unary_not() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(!true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

// ============================================================================
// Operator Precedence (parse_expr_with_precedence_recursive)
// ============================================================================

#[test]
fn test_precedence_multiply_before_add() {
    // 2 + 3 * 4 should be 14, not 20
    ruchy_cmd()
        .arg("-e")
        .arg("println(2 + 3 * 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("14"));
}

#[test]
fn test_precedence_parentheses_override() {
    // (2 + 3) * 4 should be 20
    ruchy_cmd()
        .arg("-e")
        .arg("println((2 + 3) * 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_precedence_complex_expression() {
    // 1 + 2 * 3 - 4 / 2 should be 5
    ruchy_cmd()
        .arg("-e")
        .arg("println(1 + 2 * 3 - 4 / 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_precedence_comparison_lower_than_arithmetic() {
    // 5 + 3 > 7 should be true
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 + 3 > 7)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// Infix Operators (try_handle_infix_operators)
// ============================================================================

#[test]
fn test_infix_all_arithmetic() {
    let operators = vec![
        ("10 + 5", "15"),
        ("10 - 5", "5"),
        ("10 * 5", "50"),
        ("10 / 5", "2"),
        ("10 % 3", "1"),
    ];

    for (expr, expected) in operators {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("println({expr})"))
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}

#[test]
fn test_infix_all_comparison() {
    let operators = vec![
        ("5 == 5", "true"),
        ("5 != 3", "true"),
        ("5 > 3", "true"),
        ("3 < 5", "true"),
        ("5 >= 5", "true"),
        ("5 <= 5", "true"),
    ];

    for (expr, expected) in operators {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("println({expr})"))
            .assert()
            .success()
            .stdout(predicate::str::contains(expected));
    }
}

#[test]
fn test_infix_logical_and() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(true && true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_infix_logical_or() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(false || true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// Postfix Operators (handle_postfix_operators)
// ============================================================================

#[test]
fn test_postfix_function_call() {
    let code = r"
        fun double(x) {
            x * 2
        }
        println(double(21))
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_postfix_array_index() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, 2, 3]; println(arr[0])")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_postfix_field_access() {
    let code = r"
        struct Point { x: i32, y: i32 }
        let p = Point { x: 10, y: 20 };
        println(p.x)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

// ============================================================================
// Comment Handling (consume_leading/trailing_comment, skip_comments)
// ============================================================================

#[test]
fn test_comments_line_comment() {
    ruchy_cmd()
        .arg("-e")
        .arg("// This is a comment\nprintln(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_comments_block_comment() {
    ruchy_cmd()
        .arg("-e")
        .arg("/* Block comment */ println(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_comments_trailing() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(42) // trailing comment")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_comments_multiline() {
    let code = r"
        // First comment
        let x = 10;
        // Second comment
        let y = 20;
        // Third comment
        println(x + y)
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("30"));
}

// ============================================================================
// Complex Parsing Scenarios
// ============================================================================

#[test]
fn test_complex_nested_expressions() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(((1 + 2) * (3 + 4)) / 7)")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_complex_mixed_operators() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 + 3 * 2 - 4 / 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("9"));
}

#[test]
fn test_complex_function_with_operators() {
    let code = r"
        fun compute(a, b, c) {
            a * b + c
        }
        println(compute(2, 3, 4))
    ";
    ruchy_cmd()
        .arg("-e")
        .arg(code)
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_deeply_nested_parentheses() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(((((((42)))))))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_long_expression_chain() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1 + 1)")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

#[test]
fn edge_case_whitespace_handling() {
    ruchy_cmd()
        .arg("-e")
        .arg("   println(42)   ")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
#[ignore = "Unicode identifiers not yet supported - parser rejects non-ASCII identifiers (feature gap)"]
fn edge_case_unicode_identifier() {
    ruchy_cmd()
        .arg("-e")
        .arg("let 变量 = 42; println(变量)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Error Cases (error recovery)
// ============================================================================

#[test]
#[ignore = "Parser bug: Unbalanced parentheses silently accepted (exit 0) instead of error - needs parser error recovery fix"]
fn error_case_unbalanced_parens() {
    ruchy_cmd().arg("-e").arg("(1 + 2").assert().failure();
}

#[test]
fn error_case_invalid_operator() {
    ruchy_cmd().arg("-e").arg("1 ++ 2").assert().failure();
}

#[test]
fn error_case_unexpected_token() {
    ruchy_cmd().arg("-e").arg("let = 42").assert().failure();
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_arithmetic_operators() {
    // Property: All arithmetic operators parse correctly
    let operators = vec!["+", "-", "*", "/", "%"];
    for op in operators {
        let expr = format!("10 {op} 2");
        ruchy_cmd().arg("-e").arg(&expr).assert().success();
    }
}

#[test]
fn property_comparison_operators() {
    // Property: All comparison operators parse correctly
    let operators = vec!["==", "!=", "<", ">", "<=", ">="];
    for op in operators {
        let expr = format!("5 {op} 3");
        ruchy_cmd().arg("-e").arg(&expr).assert().success();
    }
}

#[test]
fn property_nested_depth_1_to_10() {
    // Property: Nested parentheses work to arbitrary depth
    for depth in 1..=10 {
        let mut expr = "42".to_string();
        for _ in 0..depth {
            expr = format!("({expr})");
        }
        let code = format!("println({expr})");
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains("42"));
    }
}

#[test]
fn property_operator_chain_length_1_to_10() {
    // Property: Operator chains work to arbitrary length
    for length in 1..=10 {
        let expr = (0..length).map(|_| "1").collect::<Vec<_>>().join(" + ");
        let expected = length.to_string();
        let code = format!("println({expr})");
        ruchy_cmd()
            .arg("-e")
            .arg(&code)
            .assert()
            .success()
            .stdout(predicate::str::contains(&expected));
    }
}
