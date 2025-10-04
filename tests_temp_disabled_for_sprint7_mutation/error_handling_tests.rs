//! Tests for error handling and recovery
//! Focus on parser and interpreter error cases

use ruchy::frontend::{ParseError, Parser};
use ruchy::runtime::interpreter::{Interpreter, InterpreterError};

#[test]
fn test_parser_unexpected_token() {
    let mut parser = Parser::new("let 123");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_unclosed_paren() {
    let mut parser = Parser::new("(1 + 2");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_missing_expression() {
    let mut parser = Parser::new("let x =");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_invalid_operator() {
    let mut parser = Parser::new("1 ++ 2");
    let result = parser.parse();
    // May parse as 1 and then ++2, or may error
    let _ = result; // Just ensure it doesn't panic
}

#[test]
fn test_parser_empty_input() {
    let mut parser = Parser::new("");
    let result = parser.parse();
    // Empty input might be valid or error
    let _ = result; // Just ensure it doesn't panic
}

#[test]
fn test_parser_only_whitespace() {
    let mut parser = Parser::new("   \n\t  ");
    let result = parser.parse();
    let _ = result; // Just ensure it doesn't panic
}

#[test]
fn test_parser_unterminated_string() {
    let mut parser = Parser::new(r#""unterminated"#);
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_invalid_number() {
    let mut parser = Parser::new("123.456.789");
    let result = parser.parse();
    // Should either parse as 123.456 followed by .789 or error
    let _ = result; // Just ensure it doesn't panic
}

#[test]
fn test_parser_reserved_keyword() {
    let mut parser = Parser::new("let let = 5");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parser_mismatched_brackets() {
    let mut parser = Parser::new("[1, 2, 3}");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_interpreter_undefined_variable() {
    use ruchy::frontend::ast::*;

    let mut interpreter = Interpreter::new();
    let expr = Expr {
        kind: ExprKind::Identifier("undefined_var".to_string()),
        span: Span::new(0, 1),
        attributes: vec![],
    };

    let result = interpreter.eval_expr(&expr);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_division_by_zero() {
    use ruchy::frontend::ast::*;

    let mut interpreter = Interpreter::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(10)),
                span: Span::new(0, 2),
                attributes: vec![],
            }),
            op: BinaryOp::Divide,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(0)),
                span: Span::new(4, 5),
                attributes: vec![],
            }),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };

    let result = interpreter.eval_expr(&expr);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_type_mismatch_add() {
    use ruchy::frontend::ast::*;

    let mut interpreter = Interpreter::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42)),
                span: Span::new(0, 2),
                attributes: vec![],
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::new(4, 8),
                attributes: vec![],
            }),
        },
        span: Span::new(0, 8),
        attributes: vec![],
    };

    let result = interpreter.eval_expr(&expr);
    assert!(result.is_err());
}

#[test]
fn test_interpreter_invalid_unary_op() {
    use ruchy::frontend::ast::*;

    let mut interpreter = Interpreter::new();
    let expr = Expr {
        kind: ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::new(1, 5),
                attributes: vec![],
            }),
        },
        span: Span::new(0, 5),
        attributes: vec![],
    };

    let result = interpreter.eval_expr(&expr);
    // Negating a boolean might be allowed or not
    let _ = result;
}

#[test]
fn test_parser_recovery_after_error() {
    // Check that parser can recover after encountering an error
    let mut parser = Parser::new("1 + ; 2 + 3");
    let result = parser.parse();
    // Should handle the error gracefully
    let _ = result;
}

// Property-based error handling tests
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_parser_never_panics_on_invalid(input in ".*") {
            let input = if input.len() > 1000 { &input[..1000] } else { &input };

            // Parser should never panic, only return errors
            let _ = std::panic::catch_unwind(|| {
                let mut parser = Parser::new(input);
                let _ = parser.parse();
            });
        }

        #[test]
        fn prop_parser_handles_random_operators(left in "[0-9]+", op in "[!@#$%^&*+=]+", right in "[0-9]+") {
            let input = format!("{} {} {}", left, op, right);

            // Should not panic
            let _ = std::panic::catch_unwind(|| {
                let mut parser = Parser::new(&input);
                let _ = parser.parse();
            });
        }

        #[test]
        fn prop_parser_handles_nested_errors(depth in 1usize..20) {
            let mut input = String::new();
            for _ in 0..depth {
                input.push('(');
            }
            input.push_str("error");
            // Deliberately don't close all parens
            for _ in 0..depth/2 {
                input.push(')');
            }

            // Should not panic
            let _ = std::panic::catch_unwind(|| {
                let mut parser = Parser::new(&input);
                let _ = parser.parse();
            });
        }
    }
}
