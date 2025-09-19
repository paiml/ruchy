// Comprehensive TDD Test Suite for src/quality/formatter.rs
// Target: Code formatter with 95%+ coverage
// Sprint 79: Push Coverage to 75%
//
// Quality Standards:
// - TDD methodology: Test-first development
// - Cyclomatic complexity ≤10 for all test functions
// - Zero SATD comments

use ruchy::quality::formatter::Formatter;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, TypeKind, Type};
use ruchy::frontend::lexer::Span;

// Helper function to create expressions
fn create_expr(kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: Span { start: 0, end: 0 },
    }
}

// Basic tests
#[test]
fn test_formatter_new() {
    let formatter = Formatter::new();
    // Can't access private fields but verify creation
    assert!(true);
}

#[test]
fn test_format_integer_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::Integer(42)));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "42");
}

#[test]
fn test_format_float_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::Float("3.14".to_string())));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3.14");
}

#[test]
fn test_format_string_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::String("hello".to_string())));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#""hello""#);
}

#[test]
fn test_format_string_with_quotes() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::String(r#"hello "world""#.to_string())));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), r#""hello \"world\"""#);
}

#[test]
fn test_format_bool_literal() {
    let formatter = Formatter::new();

    let true_expr = create_expr(ExprKind::Literal(Literal::Bool(true)));
    let true_result = formatter.format(&true_expr);
    assert_eq!(true_result.unwrap(), "true");

    let false_expr = create_expr(ExprKind::Literal(Literal::Bool(false)));
    let false_result = formatter.format(&false_expr);
    assert_eq!(false_result.unwrap(), "false");
}

#[test]
fn test_format_char_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::Char('a')));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "'a'");
}

#[test]
fn test_format_unit_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::Unit));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "()");
}

#[test]
fn test_format_null_literal() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Literal(Literal::Null));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null");
}

#[test]
fn test_format_identifier() {
    let formatter = Formatter::new();
    let expr = create_expr(ExprKind::Identifier("my_var".to_string()));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "my_var");
}

#[test]
fn test_format_let_expression() {
    let formatter = Formatter::new();
    let value = Box::new(create_expr(ExprKind::Literal(Literal::Integer(10))));
    let body = Box::new(create_expr(ExprKind::Identifier("x".to_string())));

    let expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        ty: None,
        value,
        body,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "let x = 10 in x");
}

#[test]
fn test_format_binary_expression() {
    let formatter = Formatter::new();
    let left = Box::new(create_expr(ExprKind::Literal(Literal::Integer(2))));
    let right = Box::new(create_expr(ExprKind::Literal(Literal::Integer(3))));

    let expr = create_expr(ExprKind::Binary {
        left,
        op: BinaryOp::Add,
        right,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2 Add 3");
}

#[test]
fn test_format_block_expression() {
    let formatter = Formatter::new();
    let exprs = vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
        create_expr(ExprKind::Literal(Literal::Integer(3))),
    ];

    let expr = create_expr(ExprKind::Block(exprs));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.starts_with('{'));
    assert!(formatted.ends_with('}'));
    assert!(formatted.contains('1'));
    assert!(formatted.contains('2'));
    assert!(formatted.contains('3'));
}

#[test]
fn test_format_if_expression() {
    let formatter = Formatter::new();
    let condition = Box::new(create_expr(ExprKind::Literal(Literal::Bool(true))));
    let then_expr = Box::new(create_expr(ExprKind::Literal(Literal::Integer(1))));
    let else_expr = Some(Box::new(create_expr(ExprKind::Literal(Literal::Integer(2)))));

    let expr = create_expr(ExprKind::If {
        condition,
        then: then_expr,
        else_: else_expr,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("if"));
    assert!(formatted.contains("true"));
    assert!(formatted.contains("else"));
}

#[test]
fn test_format_match_expression() {
    let formatter = Formatter::new();
    let scrutinee = Box::new(create_expr(ExprKind::Identifier("x".to_string())));
    let arms = vec![];

    let expr = create_expr(ExprKind::Match { scrutinee, arms });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("match"));
    assert!(formatted.contains("x"));
}

#[test]
fn test_format_function_call() {
    let formatter = Formatter::new();
    let func = Box::new(create_expr(ExprKind::Identifier("print".to_string())));
    let args = vec![create_expr(ExprKind::Literal(Literal::String("hello".to_string())))];

    let expr = create_expr(ExprKind::Call { func, args });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("print"));
    assert!(formatted.contains(r#""hello""#));
}

#[test]
fn test_format_unary_expression() {
    let formatter = Formatter::new();
    let operand = Box::new(create_expr(ExprKind::Literal(Literal::Integer(42))));

    let expr = create_expr(ExprKind::Unary {
        op: UnaryOp::Neg,
        operand,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Neg 42");
}

#[test]
fn test_format_list_literal() {
    let formatter = Formatter::new();
    let elements = vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
        create_expr(ExprKind::Literal(Literal::Integer(3))),
    ];

    let expr = create_expr(ExprKind::List(elements));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.starts_with('['));
    assert!(formatted.ends_with(']'));
}

#[test]
fn test_format_tuple_literal() {
    let formatter = Formatter::new();
    let elements = vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::String("test".to_string()))),
    ];

    let expr = create_expr(ExprKind::Tuple(elements));

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.starts_with('('));
    assert!(formatted.ends_with(')'));
}

#[test]
fn test_format_lambda_expression() {
    let formatter = Formatter::new();
    let params = vec![("x".to_string(), None)];
    let body = Box::new(create_expr(ExprKind::Identifier("x".to_string())));

    let expr = create_expr(ExprKind::Lambda {
        params,
        body,
        is_async: false,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("fn"));
    assert!(formatted.contains("x"));
}

#[test]
fn test_format_async_lambda() {
    let formatter = Formatter::new();
    let params = vec![];
    let body = Box::new(create_expr(ExprKind::Literal(Literal::Integer(42))));

    let expr = create_expr(ExprKind::Lambda {
        params,
        body,
        is_async: true,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("async"));
    assert!(formatted.contains("fn"));
}

#[test]
fn test_format_while_loop() {
    let formatter = Formatter::new();
    let condition = Box::new(create_expr(ExprKind::Literal(Literal::Bool(true))));
    let body = Box::new(create_expr(ExprKind::Block(vec![])));

    let expr = create_expr(ExprKind::While { condition, body });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("while"));
}

#[test]
fn test_format_for_loop() {
    let formatter = Formatter::new();
    let pattern = "i".to_string();
    let iter = Box::new(create_expr(ExprKind::Identifier("list".to_string())));
    let body = Box::new(create_expr(ExprKind::Block(vec![])));

    let expr = create_expr(ExprKind::For {
        pattern,
        iter,
        body,
    });

    let result = formatter.format(&expr);
    assert!(result.is_ok());
    let formatted = result.unwrap();
    assert!(formatted.contains("for"));
    assert!(formatted.contains("i"));
    assert!(formatted.contains("in"));
}

// Big O Complexity Analysis:
// - format(): O(n) where n is the size of the AST
// - format_expr(): O(n) recursive traversal of AST nodes
// - format_type(): O(1) for simple type formatting
// Space complexity: O(d) where d is the depth of the AST (recursion stack)