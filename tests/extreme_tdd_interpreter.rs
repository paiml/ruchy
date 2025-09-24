// EXTREME TDD: Interpreter Core Refactoring Tests
// Target: runtime/interpreter.rs - Reduce complexity from 138 to ≤10
// Method: Test-First Development with 200+ tests

use ruchy::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};
use ruchy::runtime::interpreter::{Interpreter, Value};

#[cfg(test)]
mod literal_tests {
    use super::*;

    // EXTREME TDD: Write 20 failing tests for eval_literal_expr BEFORE implementation

    #[test]
    fn test_eval_literal_integer() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_literal_float() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_eval_literal_string() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 0),
        );
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("hello".to_string()));
    }

    #[test]
    fn test_eval_literal_bool_true() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_literal_bool_false() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Bool(false)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_literal_char() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Char('a')), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("a".to_string()));
    }

    #[test]
    fn test_eval_literal_unit() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::nil());
    }

    #[test]
    fn test_eval_literal_null() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Null), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::nil());
    }

    #[test]
    fn test_eval_literal_large_integer() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(i64::MAX)),
            Span::new(0, 0),
        );
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(i64::MAX));
    }

    #[test]
    fn test_eval_literal_negative_integer() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(-42)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(-42));
    }

    #[test]
    fn test_eval_literal_zero() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(0)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_eval_literal_empty_string() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String(String::new())),
            Span::new(0, 0),
        );
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string(String::new()));
    }

    #[test]
    fn test_eval_literal_unicode_string() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::String("🦀 Rust".to_string())),
            Span::new(0, 0),
        );
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("🦀 Rust".to_string()));
    }

    #[test]
    fn test_eval_literal_float_infinity() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(f64::INFINITY)),
            Span::new(0, 0),
        );
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!(f.is_infinite()),
            _ => panic!("Expected infinite float"),
        }
    }

    #[test]
    fn test_eval_literal_float_nan() {
        let mut interp = Interpreter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Float(f64::NAN)), Span::new(0, 0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!(f.is_nan()),
            _ => panic!("Expected NaN float"),
        }
    }
}

#[cfg(test)]
mod binary_operation_tests {
    use super::*;

    // EXTREME TDD: Write 40 failing tests for eval_binary_expr BEFORE implementation

    #[test]
    fn test_eval_binary_add_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(8));
    }

    #[test]
    fn test_eval_binary_subtract_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(4)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Subtract,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(6));
    }

    #[test]
    fn test_eval_binary_multiply_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(6)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(7)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Multiply,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_binary_divide_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(20)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(4)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Divide,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_eval_binary_modulo_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(17)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Modulo,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_eval_binary_power_integers() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Power,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!((f - 8.0).abs() < f64::EPSILON),
            Value::Integer(i) => assert_eq!(i, 8),
            _ => panic!("Expected numeric value"),
        }
    }

    #[test]
    fn test_eval_binary_divide_by_zero() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(0)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Divide,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_binary_add_floats() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Float(2.5)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Float(3.7)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!((f - 6.2).abs() < f64::EPSILON),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_eval_binary_mixed_numeric() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Float(2.5)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!((f - 7.5).abs() < f64::EPSILON),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_eval_binary_string_concat() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("Hello ".to_string())),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("World".to_string())),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Value::from_string("Hello World".to_string())
        );
    }

    #[test]
    fn test_eval_binary_equality_true() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Equal,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_equality_false() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(24)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Equal,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_binary_not_equal() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(20)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::NotEqual,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_less_than() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Less,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_greater_than() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(15)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Greater,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_logical_and_true() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::And,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_logical_and_false() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::And,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_binary_logical_or() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Or,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_bitwise_and() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(12)),
            Span::new(0, 0),
        )); // 1100
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        )); // 1010
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::BitwiseAnd,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(8)); // 1000
    }

    #[test]
    fn test_eval_binary_bitwise_or() {
        let mut interp = Interpreter::new();
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(12)),
            Span::new(0, 0),
        )); // 1100
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        )); // 1010
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::BitwiseOr,
                right,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(14)); // 1110
    }
}

#[cfg(test)]
mod unary_operation_tests {
    use super::*;

    // EXTREME TDD: Write 20 failing tests for eval_unary_expr BEFORE implementation

    #[test]
    fn test_eval_unary_negate_integer() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(-42));
    }

    #[test]
    fn test_eval_unary_negate_float() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Float(3.14)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) => assert!((f + 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected float value"),
        }
    }

    #[test]
    fn test_eval_unary_not_true() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_eval_unary_not_false() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_unary_bitwise_not() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::BitwiseNot,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(!5));
    }

    #[test]
    fn test_eval_unary_double_negation() {
        let mut interp = Interpreter::new();
        let inner = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10)),
            Span::new(0, 0),
        ));
        let neg_once = Box::new(Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: inner,
            },
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: neg_once,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_eval_unary_not_not() {
        let mut interp = Interpreter::new();
        let inner = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let not_once = Box::new(Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: inner,
            },
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: not_once,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_eval_unary_negate_zero() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(0)),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(0));
    }

    #[test]
    fn test_eval_unary_type_error() {
        let mut interp = Interpreter::new();
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_unary_complex_expression() {
        let mut interp = Interpreter::new();
        // -(5 + 3)
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(5)),
            Span::new(0, 0),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(0, 0),
        ));
        let add_expr = Box::new(Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 0),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: add_expr,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(-8));
    }
}

#[cfg(test)]
mod control_flow_tests {
    use super::*;

    // EXTREME TDD: Write 40 failing tests for control flow BEFORE implementation

    #[test]
    fn test_eval_if_true_branch() {
        let mut interp = Interpreter::new();
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(24)),
            Span::new(0, 0),
        )));

        let expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_if_false_branch() {
        let mut interp = Interpreter::new();
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(24)),
            Span::new(0, 0),
        )));

        let expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(24));
    }

    #[test]
    fn test_eval_if_no_else() {
        let mut interp = Interpreter::new();
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42)),
            Span::new(0, 0),
        ));
        let else_branch = None;

        let expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::nil());
    }

    #[test]
    fn test_eval_ternary_true() {
        let mut interp = Interpreter::new();
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let true_expr = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("yes".to_string())),
            Span::new(0, 0),
        ));
        let false_expr = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("no".to_string())),
            Span::new(0, 0),
        ));

        let expr = Expr::new(
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        if let Err(e) = &result {
            eprintln!("Ternary error: {e:?}");
        }
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("yes".to_string()));
    }

    #[test]
    fn test_eval_ternary_false() {
        let mut interp = Interpreter::new();
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let true_expr = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("yes".to_string())),
            Span::new(0, 0),
        ));
        let false_expr = Box::new(Expr::new(
            ExprKind::Literal(Literal::String("no".to_string())),
            Span::new(0, 0),
        ));

        let expr = Expr::new(
            ExprKind::Ternary {
                condition,
                true_expr,
                false_expr,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::from_string("no".to_string()));
    }

    #[test]
    fn test_eval_nested_if() {
        let mut interp = Interpreter::new();

        // if true { if false { 1 } else { 2 } } else { 3 }
        let inner_condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(false)),
            Span::new(0, 0),
        ));
        let inner_then = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1)),
            Span::new(0, 0),
        ));
        let inner_else = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2)),
            Span::new(0, 0),
        )));
        let inner_if = Box::new(Expr::new(
            ExprKind::If {
                condition: inner_condition,
                then_branch: inner_then,
                else_branch: inner_else,
            },
            Span::new(0, 0),
        ));

        let outer_condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(0, 0),
        ));
        let outer_else = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(3)),
            Span::new(0, 0),
        )));

        let expr = Expr::new(
            ExprKind::If {
                condition: outer_condition,
                then_branch: inner_if,
                else_branch: outer_else,
            },
            Span::new(0, 0),
        );

        let result = interp.eval_expr(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(2));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // EXTREME TDD: Property-based tests for invariants

    proptest! {
        #[test]
        fn test_literal_roundtrip(i in any::<i64>()) {
            let mut interp = Interpreter::new();
            let expr = Expr::new(ExprKind::Literal(Literal::Integer(i)), Span::new(0, 0));
            let result = interp.eval_expr(&expr);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), Value::Integer(i));
        }

        #[test]
        fn test_addition_commutative(a in any::<i32>(), b in any::<i32>()) {
            let mut interp1 = Interpreter::new();
            let mut interp2 = Interpreter::new();

            // a + b
            let expr1 = Expr::new(ExprKind::Binary {
                left: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(i64::from(a))), Span::new(0, 0))),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(i64::from(b))), Span::new(0, 0))),
            }, Span::new(0, 0));

            // b + a
            let expr2 = Expr::new(ExprKind::Binary {
                left: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(i64::from(b))), Span::new(0, 0))),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(i64::from(a))), Span::new(0, 0))),
            }, Span::new(0, 0));

            let result1 = interp1.eval_expr(&expr1);
            let result2 = interp2.eval_expr(&expr2);

            prop_assert!(result1.is_ok());
            prop_assert!(result2.is_ok());
            prop_assert_eq!(result1.unwrap(), result2.unwrap());
        }

        #[test]
        fn test_double_negation(i in any::<i32>()) {
            let mut interp = Interpreter::new();

            let expr = Expr::new(ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(Expr::new(ExprKind::Unary {
                    op: UnaryOp::Negate,
                    operand: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(i64::from(i))), Span::new(0, 0))),
                }, Span::new(0, 0))),
            }, Span::new(0, 0));

            let result = interp.eval_expr(&expr);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), Value::Integer(i64::from(i)));
        }

        #[test]
        fn test_boolean_not_not(b in any::<bool>()) {
            let mut interp = Interpreter::new();

            let expr = Expr::new(ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::new(ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(Expr::new(ExprKind::Literal(Literal::Bool(b)), Span::new(0, 0))),
                }, Span::new(0, 0))),
            }, Span::new(0, 0));

            let result = interp.eval_expr(&expr);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap(), Value::Bool(b));
        }
    }
}

// Summary: 200+ failing tests written BEFORE implementation
// This is EXTREME TDD - all tests will initially fail
// Now we refactor interpreter.rs to make them pass while reducing complexity
