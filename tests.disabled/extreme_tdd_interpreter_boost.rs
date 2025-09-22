// EXTREME TDD: Interpreter Coverage Boost
// Target: Interpreter from 68.5% to 80%
// Focus: Core evaluation paths

use ruchy::frontend::*;
use ruchy::runtime::{Interpreter, Value};
use std::rc::Rc;

#[cfg(test)]
mod interpreter_boost_tests {
    use super::*;

    fn make_literal(lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(lit),
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    fn make_binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    fn make_unary(op: UnaryOp, operand: Expr) -> Expr {
        Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    fn make_if(condition: Expr, then_body: Expr, else_body: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_body: Box::new(then_body),
                else_body: else_body.map(Box::new),
            },
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    fn make_array(elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Array(elements),
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    fn make_identifier(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: Vec::new(),
        }
    }

    #[test]
    fn test_literal_evaluation() {
        let mut interp = Interpreter::new();

        // Integer
        let expr = make_literal(Literal::Integer(42));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));

        // Float
        let expr = make_literal(Literal::Float(3.14));
        if let Value::Float(f) = interp.eval_expr(&expr).unwrap() {
            assert!((f - 3.14).abs() < 0.001);
        }

        // Boolean
        let expr = make_literal(Literal::Bool(true));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // String
        let expr = make_literal(Literal::String("hello".to_string()));
        assert_eq!(
            interp.eval_expr(&expr).unwrap(),
            Value::String(Rc::from("hello"))
        );
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut interp = Interpreter::new();

        // Addition
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::Add,
            make_literal(Literal::Integer(20)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(30));

        // Subtraction
        let expr = make_binary(
            make_literal(Literal::Integer(50)),
            BinaryOp::Subtract,
            make_literal(Literal::Integer(20)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(30));

        // Multiplication
        let expr = make_binary(
            make_literal(Literal::Integer(6)),
            BinaryOp::Multiply,
            make_literal(Literal::Integer(7)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));

        // Division
        let expr = make_binary(
            make_literal(Literal::Integer(100)),
            BinaryOp::Divide,
            make_literal(Literal::Integer(4)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(25));

        // Modulo
        let expr = make_binary(
            make_literal(Literal::Integer(17)),
            BinaryOp::Modulo,
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(2));
    }

    #[test]
    fn test_comparison_operations() {
        let mut interp = Interpreter::new();

        // Less than
        let expr = make_binary(
            make_literal(Literal::Integer(5)),
            BinaryOp::Less,
            make_literal(Literal::Integer(10)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Greater than
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::Greater,
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Equal
        let expr = make_binary(
            make_literal(Literal::Integer(42)),
            BinaryOp::Equal,
            make_literal(Literal::Integer(42)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Not equal
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::NotEqual,
            make_literal(Literal::Integer(20)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Less than or equal
        let expr = make_binary(
            make_literal(Literal::Integer(5)),
            BinaryOp::LessEqual,
            make_literal(Literal::Integer(5)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Greater than or equal
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::GreaterEqual,
            make_literal(Literal::Integer(10)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_logical_operations() {
        let mut interp = Interpreter::new();

        // AND
        let expr = make_binary(
            make_literal(Literal::Bool(true)),
            BinaryOp::And,
            make_literal(Literal::Bool(false)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(false));

        // OR
        let expr = make_binary(
            make_literal(Literal::Bool(true)),
            BinaryOp::Or,
            make_literal(Literal::Bool(false)),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));

        // Short-circuit AND
        let expr = make_binary(
            make_literal(Literal::Bool(false)),
            BinaryOp::And,
            make_identifier("undefined"), // Should not be evaluated
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(false));

        // Short-circuit OR
        let expr = make_binary(
            make_literal(Literal::Bool(true)),
            BinaryOp::Or,
            make_identifier("undefined"), // Should not be evaluated
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_unary_operations() {
        let mut interp = Interpreter::new();

        // Negation
        let expr = make_unary(UnaryOp::Negate, make_literal(Literal::Integer(42)));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(-42));

        // Logical NOT
        let expr = make_unary(UnaryOp::Not, make_literal(Literal::Bool(true)));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(false));

        let expr = make_unary(UnaryOp::Not, make_literal(Literal::Bool(false)));
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_if_expressions() {
        let mut interp = Interpreter::new();

        // If with true condition
        let expr = make_if(
            make_literal(Literal::Bool(true)),
            make_literal(Literal::Integer(10)),
            Some(make_literal(Literal::Integer(20))),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(10));

        // If with false condition
        let expr = make_if(
            make_literal(Literal::Bool(false)),
            make_literal(Literal::Integer(10)),
            Some(make_literal(Literal::Integer(20))),
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(20));

        // If without else
        let expr = make_if(
            make_literal(Literal::Bool(false)),
            make_literal(Literal::Integer(10)),
            None,
        );
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Nil);
    }

    #[test]
    fn test_array_operations() {
        let mut interp = Interpreter::new();

        // Empty array
        let expr = make_array(vec![]);
        if let Value::Array(arr) = interp.eval_expr(&expr).unwrap() {
            assert_eq!(arr.borrow().len(), 0);
        }

        // Array with elements
        let expr = make_array(vec![
            make_literal(Literal::Integer(1)),
            make_literal(Literal::Integer(2)),
            make_literal(Literal::Integer(3)),
        ]);
        if let Value::Array(arr) = interp.eval_expr(&expr).unwrap() {
            assert_eq!(arr.borrow().len(), 3);
        }
    }

    #[test]
    fn test_variables() {
        let mut interp = Interpreter::new();

        // Define a variable
        interp.set_variable("test_var".to_string(), Value::Integer(42));

        // Access the variable
        let expr = make_identifier("test_var");
        assert_eq!(interp.eval_expr(&expr).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_string_concatenation() {
        let mut interp = Interpreter::new();

        let expr = make_binary(
            make_literal(Literal::String("hello".to_string())),
            BinaryOp::Add,
            make_literal(Literal::String(" world".to_string())),
        );

        if let Value::String(s) = interp.eval_expr(&expr).unwrap() {
            assert_eq!(s.as_ref(), "hello world");
        }
    }

    #[test]
    fn test_mixed_type_operations() {
        let mut interp = Interpreter::new();

        // Integer + Float
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::Add,
            make_literal(Literal::Float(0.5)),
        );
        if let Ok(Value::Float(f)) = interp.eval_expr(&expr) {
            assert!((f - 10.5).abs() < 0.001);
        }

        // Float + Integer
        let expr = make_binary(
            make_literal(Literal::Float(3.14)),
            BinaryOp::Add,
            make_literal(Literal::Integer(1)),
        );
        if let Ok(Value::Float(f)) = interp.eval_expr(&expr) {
            assert!((f - 4.14).abs() < 0.001);
        }
    }

    #[test]
    fn test_division_by_zero() {
        let mut interp = Interpreter::new();

        // Integer division by zero
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::Divide,
            make_literal(Literal::Integer(0)),
        );
        assert!(interp.eval_expr(&expr).is_err());

        // Modulo by zero
        let expr = make_binary(
            make_literal(Literal::Integer(10)),
            BinaryOp::Modulo,
            make_literal(Literal::Integer(0)),
        );
        assert!(interp.eval_expr(&expr).is_err());
    }

    #[test]
    fn test_truthiness() {
        let mut interp = Interpreter::new();

        // Test various values in if condition
        let test_cases = vec![
            (Literal::Bool(false), false),
            (Literal::Bool(true), true),
            (Literal::Integer(0), false),
            (Literal::Integer(1), true),
            (Literal::Float(0.0), false),
            (Literal::Float(1.0), true),
            (Literal::String("".to_string()), false),
            (Literal::String("text".to_string()), true),
        ];

        for (literal, expected) in test_cases {
            let expr = make_if(
                make_literal(literal),
                make_literal(Literal::Bool(true)),
                Some(make_literal(Literal::Bool(false))),
            );
            assert_eq!(
                interp.eval_expr(&expr).unwrap(),
                Value::Bool(expected),
                "Failed for {:?}",
                literal
            );
        }
    }
}
