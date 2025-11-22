//! EXTREME TDD: Frontend Parser Comprehensive Coverage
//!
//! Target: frontend/parser/* modules (large codebase, high impact)
//! Strategy: Parse valid Ruchy code via CLI, exercise parser paths
//! Coverage: Expressions, statements, literals, operators, keywords

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Literals Parsing
// ============================================================================

#[test]
fn test_parse_integer_literal() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_negative_integer() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(-99)")
        .assert()
        .success()
        .stdout(predicate::str::contains("-99"));
}

#[test]
fn test_parse_float_literal() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(3.14159)")
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_parse_string_literal_double_quotes() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(\"hello world\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_parse_string_literal_single_quotes() {
    ruchy_cmd()
        .arg("-e")
        .arg("println('single')")
        .assert()
        .success();
}

#[test]
fn test_parse_boolean_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_boolean_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(false)")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_parse_nil_literal() {
    ruchy_cmd().arg("-e").arg("println(nil)").assert().success();
}

#[test]
fn test_parse_array_literal_empty() {
    ruchy_cmd().arg("-e").arg("println([])").assert().success();
}

#[test]
fn test_parse_array_literal_integers() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, 2, 3, 4, 5]; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_parse_array_literal_mixed_types() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, \"two\", 3.0, true]; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_parse_array_literal_nested() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [[1, 2], [3, 4]]; println(len(arr))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

// ============================================================================
// Binary Operators Parsing
// ============================================================================

#[test]
fn test_parse_addition() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(10 + 32)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_subtraction() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(50 - 8)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_multiplication() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(6 * 7)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_division() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(84 / 2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_modulo() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(47 % 5)")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_parse_comparison_eq() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 == 5)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_comparison_ne() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 != 3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_comparison_lt() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(3 < 5)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_comparison_gt() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 > 3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_comparison_le() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(3 <= 5)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_comparison_ge() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 >= 3)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_logical_and() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(true && true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_logical_or() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(false || true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// Unary Operators Parsing
// ============================================================================

#[test]
fn test_parse_unary_negation_numeric() {
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
// Variable Declarations
// ============================================================================

#[test]
fn test_parse_let_simple() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_let_with_type_annotation() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x: i32 = 42; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_let_mut() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut x = 10; x = 20; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_parse_multiple_lets() {
    ruchy_cmd()
        .arg("-e")
        .arg("let a = 1; let b = 2; let c = 3; println(a + b + c)")
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

// ============================================================================
// Control Flow: If Expressions
// ============================================================================

#[test]
fn test_parse_if_then() {
    ruchy_cmd()
        .arg("-e")
        .arg("if true { println(\"yes\") }")
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn test_parse_if_else() {
    ruchy_cmd()
        .arg("-e")
        .arg("if false { println(\"no\") } else { println(\"yes\") }")
        .assert()
        .success()
        .stdout(predicate::str::contains("yes"));
}

#[test]
fn test_parse_if_elif_else() {
    ruchy_cmd()
        .arg("-e")
        .arg("if false { println(\"a\") } else if true { println(\"b\") } else { println(\"c\") }")
        .assert()
        .success()
        .stdout(predicate::str::contains("b"));
}

#[test]
fn test_parse_if_expression_value() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = if true { 42 } else { 99 }; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Control Flow: Loops
// ============================================================================

#[test]
fn test_parse_for_loop() {
    ruchy_cmd()
        .arg("-e")
        .arg("for i in range(3) { println(i) }")
        .assert()
        .success();
}

#[test]
fn test_parse_while_loop() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut i = 0; while i < 3 { i = i + 1 }; println(i)")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_parse_loop_break() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut i = 0; loop { i = i + 1; if i > 5 { break } }; println(i)")
        .assert()
        .success()
        .stdout(predicate::str::contains("6"));
}

#[test]
fn test_parse_loop_continue() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut sum = 0; for i in range(10) { if i % 2 == 0 { continue }; sum = sum + i }; println(sum)")
        .assert()
        .success();
}

// ============================================================================
// Functions
// ============================================================================

#[test]
fn test_parse_function_definition_no_params() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn greet() { println(\"hello\") }; greet()")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

#[test]
fn test_parse_function_with_params() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn add(a, b) { a + b }; println(add(2, 3))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_parse_function_with_return_type() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn get_answer() -> i32 { 42 }; println(get_answer())")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_function_explicit_return() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn early() { return 42; 99 }; println(early())")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_recursive_function() {
    ruchy_cmd()
        .arg("-e")
        .arg("fn fib(n) { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }; println(fib(6))")
        .assert()
        .success()
        .stdout(predicate::str::contains("8"));
}

// ============================================================================
// Pattern Matching
// ============================================================================

#[test]
fn test_parse_match_literal() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = match 2 { 1 => \"one\", 2 => \"two\", _ => \"other\" }; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("two"));
}

#[test]
fn test_parse_match_range() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = match 5 { 1..=3 => \"low\", 4..=6 => \"mid\", _ => \"high\" }; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("mid"));
}

#[test]
fn test_parse_match_with_guard() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = match 5 { n if n > 10 => \"big\", _ => \"small\" }; println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("small"));
}

// ============================================================================
// Array Operations
// ============================================================================

#[test]
fn test_parse_array_index_access() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [10, 20, 30]; println(arr[1])")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_parse_array_index_assignment() {
    ruchy_cmd()
        .arg("-e")
        .arg("let mut arr = [1, 2, 3]; arr[1] = 99; println(arr[1])")
        .assert()
        .success()
        .stdout(predicate::str::contains("99"));
}

// ============================================================================
// String Operations
// ============================================================================

#[test]
fn test_parse_string_concatenation() {
    ruchy_cmd()
        .arg("-e")
        .arg("let s = \"hello\" + \" \" + \"world\"; println(s)")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world"));
}

#[test]
fn test_parse_f_string_basic() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; println(f\"answer: {x}\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("answer:"))
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_f_string_multiple_vars() {
    ruchy_cmd()
        .arg("-e")
        .arg("let a = 1; let b = 2; println(f\"{a} + {b} = {a + b}\")")
        .assert()
        .success();
}

// ============================================================================
// Comments
// ============================================================================

#[test]
fn test_parse_line_comment() {
    ruchy_cmd()
        .arg("-e")
        .arg("// comment\nprintln(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_parse_block_comment() {
    ruchy_cmd()
        .arg("-e")
        .arg("/* block */ println(42)")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Operator Precedence
// ============================================================================

#[test]
fn test_parse_precedence_mul_before_add() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(2 + 3 * 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("14"));
}

#[test]
fn test_parse_precedence_parentheses() {
    ruchy_cmd()
        .arg("-e")
        .arg("println((2 + 3) * 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("20"));
}

#[test]
fn test_parse_precedence_comparison_and_logical() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(5 > 3 && 2 < 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

// ============================================================================
// Complex Expressions
// ============================================================================

#[test]
fn test_parse_nested_function_calls() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(min(max(1.0, 2.0), 3.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_parse_chained_comparisons() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(1 < 2 && 2 < 3 && 3 < 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_parse_complex_arithmetic() {
    ruchy_cmd()
        .arg("-e")
        .arg("println((10 + 5) * 2 - 8 / 4)")
        .assert()
        .success()
        .stdout(predicate::str::contains("28"));
}

// ============================================================================
// Property-Based Parser Tests
// ============================================================================

#[test]
fn property_parse_all_integers() {
    // Property: Parser handles all integer ranges
    for n in [-1000, -1, 0, 1, 42, 999, 1_000_000] {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("println({n})"))
            .assert()
            .success();
    }
}

#[test]
fn property_parse_all_operators() {
    // Property: All binary operators parse correctly
    let ops = vec![
        "+", "-", "*", "/", "%", "==", "!=", "<", ">", "<=", ">=", "&&", "||",
    ];
    for op in ops {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("let _ = 5 {op} 3"))
            .assert()
            .success();
    }
}

#[test]
fn property_parse_nested_arrays() {
    // Property: Arrays can be nested to arbitrary depth
    for depth in 1..=5 {
        let mut arr = "42".to_string();
        for _ in 0..depth {
            arr = format!("[{arr}]");
        }
        ruchy_cmd()
            .arg("-e")
            .arg(format!("println({arr})"))
            .assert()
            .success();
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_empty_program() {
    // Empty programs error (expected behavior)
    ruchy_cmd()
        .arg("-e")
        .arg("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn edge_case_only_whitespace() {
    // Whitespace-only programs error (expected behavior)
    ruchy_cmd()
        .arg("-e")
        .arg("   \n\t  ")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn edge_case_only_comments() {
    // Comment-only programs error (expected behavior)
    ruchy_cmd()
        .arg("-e")
        .arg("// just a comment")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty program"));
}

#[test]
fn edge_case_unicode_in_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(\"hello ✓ world\")")
        .assert()
        .success();
}

#[test]
fn edge_case_very_long_identifier() {
    let long_name = "a".repeat(100);
    ruchy_cmd()
        .arg("-e")
        .arg(format!("let {long_name} = 42; println({long_name})"))
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_deeply_nested_expressions() {
    // Nested parentheses
    ruchy_cmd()
        .arg("-e")
        .arg("println(((((1 + 1)))))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}
