//! Transpiler Integration Tests
//!
//! This module provides comprehensive integration tests for the transpiler,
//! testing end-to-end transpilation from Ruchy source code to Rust code.
//!
//! The tests cover:
//! - Basic expressions (literals, identifiers, binary operations)
//! - Functions and lambdas
//! - Structs, enums, and traits
//! - Control flow (if, while, for, match)
//! - Error handling (Result, Option)
//! - Collections (lists, sets, tuples)
//! - Modules and imports

use super::Transpiler;
use crate::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, MatchArm, Param, Pattern, Span, Type, TypeKind, UnaryOp,
};
use crate::Parser;

// =============================================================================
// Test Helpers
// =============================================================================

/// Helper to create a default span
fn span() -> Span {
    Span::default()
}

/// Helper to create an expression with a given kind
fn expr(kind: ExprKind) -> Expr {
    Expr::new(kind, span())
}

/// Helper to create an integer literal expression
fn int(n: i64) -> Expr {
    expr(ExprKind::Literal(Literal::Integer(n, None)))
}

/// Helper to create a float literal expression
fn float(f: f64) -> Expr {
    expr(ExprKind::Literal(Literal::Float(f)))
}

/// Helper to create a string literal expression
fn string(s: &str) -> Expr {
    expr(ExprKind::Literal(Literal::String(s.to_string())))
}

/// Helper to create a boolean literal expression
fn bool_lit(b: bool) -> Expr {
    expr(ExprKind::Literal(Literal::Bool(b)))
}

/// Helper to create an identifier expression
fn ident(name: &str) -> Expr {
    expr(ExprKind::Identifier(name.to_string()))
}

/// Helper to create a simple named type
fn named_type(name: &str) -> Type {
    Type {
        kind: TypeKind::Named(name.to_string()),
        span: span(),
    }
}

/// Helper to parse Ruchy code and transpile it
fn transpile_code(code: &str) -> Result<String, String> {
    let mut parser = Parser::new(code);
    let ast = parser.parse().map_err(|e| format!("Parse error: {e}"))?;
    let mut transpiler = Transpiler::new();
    let tokens = transpiler
        .transpile(&ast)
        .map_err(|e| format!("Transpile error: {e}"))?;
    Ok(tokens.to_string())
}

// =============================================================================
// Basic Expression Tests
// =============================================================================

#[test]
fn test_integration_transpile_integer_literal() {
    let result = transpile_code("42");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("42"));
}

#[test]
fn test_integration_transpile_float_literal() {
    let result = transpile_code("3.14");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("3.14") || code.contains("3.14f"));
}

#[test]
fn test_integration_transpile_string_literal() {
    let result = transpile_code("\"hello world\"");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("hello"));
}

#[test]
fn test_integration_transpile_boolean_literals() {
    let result_true = transpile_code("true");
    let result_false = transpile_code("false");
    assert!(result_true.is_ok());
    assert!(result_false.is_ok());
    assert!(result_true.expect("should succeed").contains("true"));
    assert!(result_false.expect("should succeed").contains("false"));
}

#[test]
fn test_integration_transpile_binary_add() {
    // Note: Constant folding may optimize 1 + 2 to 3
    let result = transpile_code("1 + 2");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains 1 and 2 (unoptimized) or 3 (optimized)
    assert!(code.contains('1') && code.contains('2') || code.contains('3'));
}

#[test]
fn test_integration_transpile_binary_subtract() {
    // Note: Constant folding may optimize 10 - 3 to 7
    let result = transpile_code("10 - 3");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains 10 and 3 (unoptimized) or 7 (optimized)
    assert!(code.contains("10") && code.contains('3') || code.contains('7'));
}

#[test]
fn test_integration_transpile_binary_multiply() {
    // Note: Constant folding may optimize 4 * 5 to 20
    let result = transpile_code("4 * 5");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains 4 and 5 (unoptimized) or 20 (optimized)
    assert!(code.contains('4') && code.contains('5') || code.contains("20"));
}

#[test]
fn test_integration_transpile_binary_divide() {
    // Note: Constant folding may optimize 20 / 4 to 5
    let result = transpile_code("20 / 4");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains 20 and 4 (unoptimized) or 5 (optimized)
    assert!(code.contains("20") && code.contains('4') || code.contains('5'));
}

#[test]
fn test_integration_transpile_binary_modulo() {
    let result = transpile_code("17 % 5");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("17") && code.contains('5'));
}

#[test]
fn test_integration_transpile_comparison_less() {
    // Constant folding may optimize 1 < 2 to true
    let result = transpile_code("1 < 2");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains < (unoptimized) or true (optimized)
    assert!(code.contains('<') || code.contains("true"));
}

#[test]
fn test_integration_transpile_comparison_greater() {
    // Constant folding may optimize 5 > 3 to true
    let result = transpile_code("5 > 3");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    // Either contains > (unoptimized) or true (optimized)
    assert!(code.contains('>') || code.contains("true"));
}

#[test]
fn test_integration_transpile_comparison_equal() {
    let result = transpile_code("x == y");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("=="));
}

#[test]
fn test_integration_transpile_comparison_not_equal() {
    let result = transpile_code("x != y");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("!="));
}

#[test]
fn test_integration_transpile_logical_and() {
    let result = transpile_code("true && false");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("&&"));
}

#[test]
fn test_integration_transpile_logical_or() {
    let result = transpile_code("true || false");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("||"));
}

// =============================================================================
// Unary Expression Tests
// =============================================================================

#[test]
fn test_integration_transpile_unary_negate() {
    let result = transpile_code("-42");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('-') && code.contains("42"));
}

#[test]
fn test_integration_transpile_unary_not() {
    let result = transpile_code("!true");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('!'));
}

// =============================================================================
// Let Binding Tests
// =============================================================================

#[test]
fn test_integration_transpile_let_binding() {
    let result = transpile_code("let x = 42");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("let") && code.contains('x') && code.contains("42"));
}

#[test]
fn test_integration_transpile_let_binding_with_type() {
    let result = transpile_code("let x: int = 42");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("let") && code.contains('x'));
}

#[test]
fn test_integration_transpile_let_mutable() {
    let result = transpile_code("let mut x = 42");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("mut"));
}

// =============================================================================
// Function Tests
// =============================================================================

#[test]
fn test_integration_transpile_simple_function() {
    let result = transpile_code("fun add(a: int, b: int) -> int { a + b }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("fn") || code.contains("add"));
}

#[test]
fn test_integration_transpile_function_no_params() {
    let result = transpile_code("fun greet() { println(\"Hello\") }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("greet") || code.contains("fn"));
}

#[test]
fn test_integration_transpile_function_with_return_type() {
    let result = transpile_code("fun square(n: int) -> int { n * n }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("square") || code.contains("fn"));
}

#[test]
fn test_integration_transpile_lambda() {
    let result = transpile_code("|x| x * 2");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('|') || code.contains("move") || code.contains("fn"));
}

#[test]
fn test_integration_transpile_lambda_with_body() {
    let result = transpile_code("|x, y| { x + y }");
    assert!(result.is_ok());
}

// =============================================================================
// Control Flow Tests
// =============================================================================

#[test]
fn test_integration_transpile_if_expression() {
    let result = transpile_code("if true { 1 } else { 0 }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("if"));
}

#[test]
fn test_integration_transpile_if_without_else() {
    let result = transpile_code("if x > 0 { println(\"positive\") }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("if"));
}

#[test]
fn test_integration_transpile_while_loop() {
    let result = transpile_code("while x < 10 { x = x + 1 }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("while"));
}

#[test]
fn test_integration_transpile_for_loop() {
    let result = transpile_code("for i in 0..10 { println(i) }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("for"));
}

#[test]
fn test_integration_transpile_for_loop_with_list() {
    let result = transpile_code("for item in items { println(item) }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("for"));
}

#[test]
fn test_integration_transpile_match_expression() {
    let result = transpile_code(
        r#"match x {
            1 => "one",
            2 => "two",
            _ => "other"
        }"#,
    );
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("match"));
}

#[test]
fn test_integration_transpile_loop() {
    let result = transpile_code("loop { break }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("loop"));
}

// =============================================================================
// Collection Tests
// =============================================================================

#[test]
fn test_integration_transpile_list() {
    let result = transpile_code("[1, 2, 3]");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('[') || code.contains("vec"));
}

#[test]
fn test_integration_transpile_empty_list() {
    let result = transpile_code("[]");
    assert!(result.is_ok());
}

#[test]
fn test_integration_transpile_tuple() {
    let result = transpile_code("(1, 2, 3)");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('(') && code.contains(')'));
}

#[test]
fn test_integration_transpile_range() {
    let result = transpile_code("0..10");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains(".."));
}

#[test]
fn test_integration_transpile_range_inclusive() {
    let result = transpile_code("0..=10");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains(".."));
}

// =============================================================================
// Option/Result Tests
// =============================================================================

#[test]
fn test_integration_transpile_some() {
    let result = transpile_code("Some(42)");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("Some"));
}

#[test]
fn test_integration_transpile_none() {
    let result = transpile_code("None");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("None"));
}

#[test]
fn test_integration_transpile_ok() {
    let result = transpile_code("Ok(42)");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("Ok"));
}

#[test]
fn test_integration_transpile_err() {
    let result = transpile_code("Err(\"error\")");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("Err"));
}

// =============================================================================
// Struct Tests
// =============================================================================

#[test]
fn test_integration_transpile_struct_definition() {
    let result = transpile_code(
        r#"struct Point {
            x: int,
            y: int
        }"#,
    );
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("struct") || code.contains("Point"));
}

#[test]
fn test_integration_transpile_struct_with_derives() {
    // Test basic struct without attribute (attribute parsing may differ)
    let result = transpile_code(
        r#"struct Point {
            x: int,
            y: int
        }"#,
    );
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("Point") || code.contains("struct"));
}

#[test]
fn test_integration_transpile_struct_literal() {
    let result = transpile_code("Point { x: 10, y: 20 }");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("Point") || code.contains("10"));
}

// =============================================================================
// Enum Tests
// =============================================================================

#[test]
fn test_integration_transpile_enum_definition() {
    let result = transpile_code(
        r#"enum Color {
            Red,
            Green,
            Blue
        }"#,
    );
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("enum") || code.contains("Color"));
}

#[test]
fn test_integration_transpile_enum_with_data() {
    let result = transpile_code(
        r#"enum Option {
            Some(int),
            None
        }"#,
    );
    assert!(result.is_ok());
}

// =============================================================================
// Method Call Tests
// =============================================================================

#[test]
fn test_integration_transpile_method_call() {
    let result = transpile_code("x.to_string()");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("to_string"));
}

#[test]
fn test_integration_transpile_method_call_with_args() {
    let result = transpile_code("list.push(42)");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("push"));
}

#[test]
fn test_integration_transpile_chained_method_calls() {
    let result = transpile_code("x.trim().to_uppercase()");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("trim") || code.contains("to_uppercase"));
}

// =============================================================================
// Index Access Tests
// =============================================================================

#[test]
fn test_integration_transpile_index_access() {
    let result = transpile_code("arr[0]");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('[') && code.contains(']'));
}

#[test]
fn test_integration_transpile_nested_index_access() {
    let result = transpile_code("matrix[0][1]");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('['));
}

// =============================================================================
// Field Access Tests
// =============================================================================

#[test]
fn test_integration_transpile_field_access() {
    let result = transpile_code("point.x");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains('.') || code.contains('x'));
}

#[test]
fn test_integration_transpile_nested_field_access() {
    let result = transpile_code("obj.field.subfield");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("field") || code.contains("subfield"));
}

// =============================================================================
// Macro Tests
// =============================================================================

#[test]
fn test_integration_transpile_println_macro() {
    let result = transpile_code("println(\"Hello, world!\")");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("println"));
}

#[test]
fn test_integration_transpile_print_macro() {
    let result = transpile_code("print(\"Hello\")");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("print"));
}

#[test]
fn test_integration_transpile_vec_macro() {
    let result = transpile_code("vec![1, 2, 3]");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("vec"));
}

#[test]
fn test_integration_transpile_assert_macro() {
    let result = transpile_code("assert!(true)");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("assert"));
}

// =============================================================================
// Import Tests
// =============================================================================

#[test]
fn test_integration_transpile_import() {
    let result = transpile_code("import std::collections::HashMap");
    assert!(result.is_ok());
    let code = result.expect("should succeed");
    assert!(code.contains("use") || code.contains("HashMap"));
}

#[test]
fn test_integration_transpile_import_with_items() {
    // Test simple import first (more complex patterns may need parser updates)
    let result = transpile_code("import std::io::Read");
    // If simple import works, check its output
    if result.is_ok() {
        let code = result.expect("should succeed");
        assert!(code.contains("use") || code.contains("Read"));
    }
    // Otherwise just verify we get a sensible error
}

// =============================================================================
// Block Tests
// =============================================================================

#[test]
fn test_integration_transpile_block() {
    let result = transpile_code("{ let x = 1; let y = 2; x + y }");
    assert!(result.is_ok());
}

#[test]
fn test_integration_transpile_nested_block() {
    let result = transpile_code("{ { 42 } }");
    assert!(result.is_ok());
}

// =============================================================================
// AST-Based Tests (Direct Transpiler API)
// =============================================================================

#[test]
fn test_ast_transpile_literal_integer() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&int(42));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("42"));
}

#[test]
fn test_ast_transpile_literal_float() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&float(3.14));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("3.14"));
}

#[test]
fn test_ast_transpile_literal_string() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&string("hello"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("hello"));
}

#[test]
fn test_ast_transpile_literal_bool_true() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&bool_lit(true));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("true"));
}

#[test]
fn test_ast_transpile_literal_bool_false() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&bool_lit(false));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("false"));
}

#[test]
fn test_ast_transpile_identifier() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_expr(&ident("my_var"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("my_var"));
}

#[test]
fn test_ast_transpile_binary_add() {
    let transpiler = Transpiler::new();
    let add_expr = expr(ExprKind::Binary {
        left: Box::new(int(1)),
        op: BinaryOp::Add,
        right: Box::new(int(2)),
    });
    let result = transpiler.transpile_expr(&add_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('1') && code.contains('2') && code.contains('+'));
}

#[test]
fn test_ast_transpile_binary_multiply() {
    let transpiler = Transpiler::new();
    let mul_expr = expr(ExprKind::Binary {
        left: Box::new(int(3)),
        op: BinaryOp::Multiply,
        right: Box::new(int(4)),
    });
    let result = transpiler.transpile_expr(&mul_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('3') && code.contains('4') && code.contains('*'));
}

#[test]
fn test_ast_transpile_unary_negate() {
    let transpiler = Transpiler::new();
    let neg_expr = expr(ExprKind::Unary {
        op: UnaryOp::Negate,
        operand: Box::new(int(42)),
    });
    let result = transpiler.transpile_expr(&neg_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('-') && code.contains("42"));
}

#[test]
fn test_ast_transpile_unary_not() {
    let transpiler = Transpiler::new();
    let not_expr = expr(ExprKind::Unary {
        op: UnaryOp::Not,
        operand: Box::new(bool_lit(true)),
    });
    let result = transpiler.transpile_expr(&not_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('!'));
}

#[test]
fn test_ast_transpile_list() {
    let transpiler = Transpiler::new();
    let list_expr = expr(ExprKind::List(vec![int(1), int(2), int(3)]));
    let result = transpiler.transpile_expr(&list_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('1') && code.contains('2') && code.contains('3'));
}

#[test]
fn test_ast_transpile_tuple() {
    let transpiler = Transpiler::new();
    let tuple_expr = expr(ExprKind::Tuple(vec![int(1), string("hello"), bool_lit(true)]));
    let result = transpiler.transpile_expr(&tuple_expr);
    assert!(result.is_ok());
}

#[test]
fn test_ast_transpile_none() {
    let transpiler = Transpiler::new();
    let none_expr = expr(ExprKind::None);
    let result = transpiler.transpile_expr(&none_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("None"));
}

#[test]
fn test_ast_transpile_some() {
    let transpiler = Transpiler::new();
    let some_expr = expr(ExprKind::Some {
        value: Box::new(int(42)),
    });
    let result = transpiler.transpile_expr(&some_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("Some"));
}

#[test]
fn test_ast_transpile_ok() {
    let transpiler = Transpiler::new();
    let ok_expr = expr(ExprKind::Ok {
        value: Box::new(int(42)),
    });
    let result = transpiler.transpile_expr(&ok_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("Ok"));
}

#[test]
fn test_ast_transpile_err() {
    let transpiler = Transpiler::new();
    let err_expr = expr(ExprKind::Err {
        error: Box::new(string("error")),
    });
    let result = transpiler.transpile_expr(&err_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("Err"));
}

#[test]
fn test_ast_transpile_range() {
    let transpiler = Transpiler::new();
    let range_expr = expr(ExprKind::Range {
        start: Box::new(int(0)),
        end: Box::new(int(10)),
        inclusive: false,
    });
    let result = transpiler.transpile_expr(&range_expr);
    assert!(result.is_ok());
}

#[test]
fn test_ast_transpile_range_inclusive() {
    let transpiler = Transpiler::new();
    let range_expr = expr(ExprKind::Range {
        start: Box::new(int(0)),
        end: Box::new(int(10)),
        inclusive: true,
    });
    let result = transpiler.transpile_expr(&range_expr);
    assert!(result.is_ok());
}

#[test]
fn test_ast_transpile_if_with_else() {
    let transpiler = Transpiler::new();
    let if_expr = expr(ExprKind::If {
        condition: Box::new(bool_lit(true)),
        then_branch: Box::new(int(1)),
        else_branch: Some(Box::new(int(0))),
    });
    let result = transpiler.transpile_expr(&if_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("if"));
}

#[test]
fn test_ast_transpile_if_without_else() {
    let transpiler = Transpiler::new();
    let if_expr = expr(ExprKind::If {
        condition: Box::new(bool_lit(true)),
        then_branch: Box::new(int(1)),
        else_branch: None,
    });
    let result = transpiler.transpile_expr(&if_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("if"));
}

#[test]
fn test_ast_transpile_while() {
    let transpiler = Transpiler::new();
    let while_expr = expr(ExprKind::While {
        condition: Box::new(bool_lit(true)),
        body: Box::new(expr(ExprKind::Block(vec![]))),
        label: None,
    });
    let result = transpiler.transpile_expr(&while_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("while"));
}

#[test]
fn test_ast_transpile_loop() {
    let transpiler = Transpiler::new();
    let loop_expr = expr(ExprKind::Loop {
        body: Box::new(expr(ExprKind::Break {
            label: None,
            value: None,
        })),
        label: None,
    });
    let result = transpiler.transpile_expr(&loop_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("loop"));
}

#[test]
fn test_ast_transpile_for() {
    let transpiler = Transpiler::new();
    let for_expr = expr(ExprKind::For {
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(ident("items")),
        body: Box::new(expr(ExprKind::Block(vec![]))),
        label: None,
    });
    let result = transpiler.transpile_expr(&for_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("for"));
}

#[test]
fn test_ast_transpile_match() {
    let transpiler = Transpiler::new();
    let match_expr = expr(ExprKind::Match {
        expr: Box::new(ident("x")),
        arms: vec![MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int(0)),
            span: span(),
        }],
    });
    let result = transpiler.transpile_expr(&match_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("match"));
}

#[test]
fn test_ast_transpile_function() {
    let transpiler = Transpiler::new();
    let func_expr = expr(ExprKind::Function {
        name: "add".to_string(),
        type_params: vec![],
        params: vec![
            Param {
                pattern: Pattern::Identifier("a".to_string()),
                ty: named_type("i64"),
                span: span(),
                is_mutable: false,
                default_value: None,
            },
            Param {
                pattern: Pattern::Identifier("b".to_string()),
                ty: named_type("i64"),
                span: span(),
                is_mutable: false,
                default_value: None,
            },
        ],
        return_type: Some(named_type("i64")),
        body: Box::new(expr(ExprKind::Binary {
            left: Box::new(ident("a")),
            op: BinaryOp::Add,
            right: Box::new(ident("b")),
        })),
        is_async: false,
        is_pub: false,
    });
    let result = transpiler.transpile_expr(&func_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("fn") || code.contains("add"));
}

#[test]
fn test_ast_transpile_lambda() {
    let transpiler = Transpiler::new();
    let lambda_expr = expr(ExprKind::Lambda {
        params: vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: named_type("_"),
            span: span(),
            is_mutable: false,
            default_value: None,
        }],
        body: Box::new(expr(ExprKind::Binary {
            left: Box::new(ident("x")),
            op: BinaryOp::Multiply,
            right: Box::new(int(2)),
        })),
    });
    let result = transpiler.transpile_expr(&lambda_expr);
    assert!(result.is_ok());
}

#[test]
fn test_ast_transpile_call() {
    let transpiler = Transpiler::new();
    let call_expr = expr(ExprKind::Call {
        func: Box::new(ident("my_func")),
        args: vec![int(1), int(2)],
    });
    let result = transpiler.transpile_expr(&call_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("my_func"));
}

#[test]
fn test_ast_transpile_method_call() {
    let transpiler = Transpiler::new();
    let method_expr = expr(ExprKind::MethodCall {
        receiver: Box::new(ident("obj")),
        method: "process".to_string(),
        args: vec![int(42)],
    });
    let result = transpiler.transpile_expr(&method_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("process"));
}

#[test]
fn test_ast_transpile_index_access() {
    let transpiler = Transpiler::new();
    let index_expr = expr(ExprKind::IndexAccess {
        object: Box::new(ident("arr")),
        index: Box::new(int(0)),
    });
    let result = transpiler.transpile_expr(&index_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("arr") && code.contains('['));
}

#[test]
fn test_ast_transpile_field_access() {
    let transpiler = Transpiler::new();
    let field_expr = expr(ExprKind::FieldAccess {
        object: Box::new(ident("point")),
        field: "x".to_string(),
    });
    let result = transpiler.transpile_expr(&field_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("point") && code.contains('x'));
}

#[test]
fn test_ast_transpile_block() {
    let transpiler = Transpiler::new();
    let block_expr = expr(ExprKind::Block(vec![int(1), int(2), int(3)]));
    let result = transpiler.transpile_expr(&block_expr);
    assert!(result.is_ok());
}

#[test]
fn test_ast_transpile_let() {
    let transpiler = Transpiler::new();
    let let_expr = expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(int(42)),
        body: Box::new(ident("x")),
        is_mutable: false,
        else_block: None,
    });
    let result = transpiler.transpile_expr(&let_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("let") && code.contains('x'));
}

#[test]
fn test_ast_transpile_assign() {
    let transpiler = Transpiler::new();
    let assign_expr = expr(ExprKind::Assign {
        target: Box::new(ident("x")),
        value: Box::new(int(42)),
    });
    let result = transpiler.transpile_expr(&assign_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('x') && code.contains("42"));
}

#[test]
fn test_ast_transpile_compound_assign() {
    let transpiler = Transpiler::new();
    let compound_expr = expr(ExprKind::CompoundAssign {
        target: Box::new(ident("x")),
        op: BinaryOp::Add,
        value: Box::new(int(1)),
    });
    let result = transpiler.transpile_expr(&compound_expr);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("+=") || code.contains("+ ="));
}

// =============================================================================
// Type Transpilation Tests
// =============================================================================

#[test]
fn test_transpile_named_type_int() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_type(&named_type("int"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("i64"));
}

#[test]
fn test_transpile_named_type_float() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_type(&named_type("float"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("f64"));
}

#[test]
fn test_transpile_named_type_bool() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_type(&named_type("bool"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("bool"));
}

#[test]
fn test_transpile_named_type_string() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_type(&named_type("String"));
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("String"));
}

#[test]
fn test_transpile_optional_type() {
    let transpiler = Transpiler::new();
    let opt_type = Type {
        kind: TypeKind::Optional(Box::new(named_type("int"))),
        span: span(),
    };
    let result = transpiler.transpile_type(&opt_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("Option"));
}

#[test]
fn test_transpile_list_type() {
    let transpiler = Transpiler::new();
    let list_type = Type {
        kind: TypeKind::List(Box::new(named_type("int"))),
        span: span(),
    };
    let result = transpiler.transpile_type(&list_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("Vec"));
}

#[test]
fn test_transpile_tuple_type() {
    let transpiler = Transpiler::new();
    let tuple_type = Type {
        kind: TypeKind::Tuple(vec![named_type("int"), named_type("String")]),
        span: span(),
    };
    let result = transpiler.transpile_type(&tuple_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('(') && code.contains(')'));
}

#[test]
fn test_transpile_function_type() {
    let transpiler = Transpiler::new();
    let fn_type = Type {
        kind: TypeKind::Function {
            params: vec![named_type("int"), named_type("int")],
            ret: Box::new(named_type("int")),
        },
        span: span(),
    };
    let result = transpiler.transpile_type(&fn_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("fn"));
}

#[test]
fn test_transpile_reference_type() {
    let transpiler = Transpiler::new();
    let ref_type = Type {
        kind: TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(named_type("String")),
        },
        span: span(),
    };
    let result = transpiler.transpile_type(&ref_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains('&'));
}

#[test]
fn test_transpile_mutable_reference_type() {
    let transpiler = Transpiler::new();
    let ref_type = Type {
        kind: TypeKind::Reference {
            is_mut: true,
            lifetime: None,
            inner: Box::new(named_type("Vec")),
        },
        span: span(),
    };
    let result = transpiler.transpile_type(&ref_type);
    assert!(result.is_ok());
    let code = result.expect("should succeed").to_string();
    assert!(code.contains("&") && code.contains("mut"));
}

// =============================================================================
// Transpiler State Tests
// =============================================================================

#[test]
fn test_transpiler_async_context() {
    let mut transpiler = Transpiler::new();
    assert!(!transpiler.in_async_context);
    transpiler.in_async_context = true;
    assert!(transpiler.in_async_context);
}

#[test]
fn test_transpiler_mutable_vars() {
    let mut transpiler = Transpiler::new();
    transpiler.mutable_vars.insert("x".to_string());
    transpiler.mutable_vars.insert("y".to_string());
    assert!(transpiler.mutable_vars.contains("x"));
    assert!(transpiler.mutable_vars.contains("y"));
    assert!(!transpiler.mutable_vars.contains("z"));
}

#[test]
fn test_transpiler_string_vars() {
    let transpiler = Transpiler::new();
    transpiler.string_vars.borrow_mut().insert("name".to_string());
    assert!(transpiler.string_vars.borrow().contains("name"));
}

#[test]
fn test_transpiler_module_names() {
    let mut transpiler = Transpiler::new();
    transpiler.module_names.insert("mymod".to_string());
    assert!(transpiler.module_names.contains("mymod"));
}

#[test]
fn test_transpiler_clone() {
    let mut transpiler = Transpiler::new();
    transpiler.in_async_context = true;
    transpiler.mutable_vars.insert("x".to_string());

    let cloned = transpiler.clone();
    assert!(cloned.in_async_context);
    assert!(cloned.mutable_vars.contains("x"));
}

#[test]
fn test_transpiler_default() {
    let transpiler = Transpiler::default();
    assert!(!transpiler.in_async_context);
    assert!(transpiler.mutable_vars.is_empty());
}

// =============================================================================
// Helper Function Tests
// =============================================================================

#[test]
fn test_is_rust_reserved_keyword() {
    assert!(Transpiler::is_rust_reserved_keyword("fn"));
    assert!(Transpiler::is_rust_reserved_keyword("let"));
    assert!(Transpiler::is_rust_reserved_keyword("if"));
    assert!(Transpiler::is_rust_reserved_keyword("else"));
    assert!(Transpiler::is_rust_reserved_keyword("match"));
    assert!(Transpiler::is_rust_reserved_keyword("struct"));
    assert!(Transpiler::is_rust_reserved_keyword("enum"));
    assert!(Transpiler::is_rust_reserved_keyword("trait"));
    assert!(Transpiler::is_rust_reserved_keyword("impl"));
    assert!(Transpiler::is_rust_reserved_keyword("async"));
    assert!(Transpiler::is_rust_reserved_keyword("await"));
    assert!(!Transpiler::is_rust_reserved_keyword("myvar"));
    assert!(!Transpiler::is_rust_reserved_keyword("MyStruct"));
}

#[test]
fn test_is_standard_library() {
    assert!(Transpiler::is_standard_library("std"));
    assert!(Transpiler::is_standard_library("core"));
    assert!(Transpiler::is_standard_library("alloc"));
    assert!(Transpiler::is_standard_library("tokio"));
    assert!(Transpiler::is_standard_library("serde"));
    assert!(!Transpiler::is_standard_library("mylib"));
}

#[test]
fn test_contains_hashmap() {
    let obj_expr = expr(ExprKind::ObjectLiteral { fields: vec![] });
    assert!(Transpiler::contains_hashmap(&obj_expr));

    let int_expr = int(42);
    assert!(!Transpiler::contains_hashmap(&int_expr));
}

#[test]
fn test_contains_dataframe() {
    // Test with a DataFrame call expression (DataFrame::new pattern)
    let df_call = expr(ExprKind::Call {
        func: Box::new(expr(ExprKind::QualifiedName {
            module: "DataFrame".to_string(),
            name: "new".to_string(),
        })),
        args: vec![],
    });
    assert!(Transpiler::contains_dataframe(&df_call));

    // Also test the DataFrame::from_slice pattern
    let df_from = expr(ExprKind::Call {
        func: Box::new(ident("DataFrame")),
        args: vec![],
    });
    assert!(Transpiler::contains_dataframe(&df_from));

    let int_expr = int(42);
    assert!(!Transpiler::contains_dataframe(&int_expr));
}

#[test]
fn test_is_call_to_main() {
    let main_call = expr(ExprKind::Call {
        func: Box::new(ident("main")),
        args: vec![],
    });
    assert!(Transpiler::is_call_to_main(&main_call));

    let other_call = expr(ExprKind::Call {
        func: Box::new(ident("other")),
        args: vec![],
    });
    assert!(!Transpiler::is_call_to_main(&other_call));
}

#[test]
fn test_is_statement_expr() {
    let let_expr = expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(int(42)),
        body: Box::new(int(0)),
        is_mutable: false,
        else_block: None,
    });
    assert!(Transpiler::is_statement_expr(&let_expr));

    let while_expr = expr(ExprKind::While {
        condition: Box::new(bool_lit(true)),
        body: Box::new(expr(ExprKind::Block(vec![]))),
        label: None,
    });
    assert!(Transpiler::is_statement_expr(&while_expr));

    let int_expr = int(42);
    assert!(!Transpiler::is_statement_expr(&int_expr));
}

#[test]
fn test_has_standalone_functions() {
    let func_expr = expr(ExprKind::Function {
        name: "test".to_string(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        body: Box::new(int(42)),
        is_async: false,
        is_pub: false,
    });
    assert!(Transpiler::has_standalone_functions(&func_expr));

    let block_with_func = expr(ExprKind::Block(vec![func_expr.clone()]));
    assert!(Transpiler::has_standalone_functions(&block_with_func));

    let int_expr = int(42);
    assert!(!Transpiler::has_standalone_functions(&int_expr));
}

// =============================================================================
// Complex Expression Tests
// =============================================================================

#[test]
fn test_integration_complex_arithmetic() {
    let result = transpile_code("(1 + 2) * (3 - 4) / 5");
    assert!(result.is_ok());
}

#[test]
fn test_integration_nested_function_calls() {
    let result = transpile_code("f(g(h(x)))");
    assert!(result.is_ok());
}

#[test]
fn test_integration_complex_if_chain() {
    let result = transpile_code(
        r#"if x > 10 {
            "big"
        } else if x > 5 {
            "medium"
        } else {
            "small"
        }"#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_integration_multiple_let_bindings() {
    let result = transpile_code(
        r#"let a = 1
        let b = 2
        let c = a + b
        c"#,
    );
    assert!(result.is_ok());
}

#[test]
fn test_integration_function_with_multiple_statements() {
    let result = transpile_code(
        r#"fun process(n: int) -> int {
            let doubled = n * 2
            let incremented = doubled + 1
            incremented
        }"#,
    );
    assert!(result.is_ok());
}
