//! Comprehensive tests for Code Formatter module
//! Target: Increase coverage from 1.13% to >80%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod formatter_tests {
    use crate::quality::formatter::Formatter;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, BinaryOp, UnaryOp, Attribute};
    
    // ========== Helper Functions ==========
    
    fn create_literal_expr(literal: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(literal),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        }
    }
    
    fn create_identifier_expr(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        }
    }
    
    // ========== Formatter Tests ==========
    
    #[test]
    fn test_formatter_creation() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::Integer(42));
        assert!(formatter.format(&expr).is_ok());
    }
    
    #[test]
    fn test_format_integer_literal() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::Integer(12345));
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "12345");
    }
    
    #[test]
    fn test_format_float_literal() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::Float(3.14159));
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("3.14"));
    }
    
    #[test]
    fn test_format_string_literal() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::String("Hello, World!".to_string()));
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#""Hello, World!""#);
    }
    
    #[test]
    fn test_format_bool_literals() {
        let formatter = Formatter::new();
        
        let true_expr = create_literal_expr(Literal::Bool(true));
        assert_eq!(formatter.format(&true_expr).unwrap(), "true");
        
        let false_expr = create_literal_expr(Literal::Bool(false));
        assert_eq!(formatter.format(&false_expr).unwrap(), "false");
    }
    
    #[test]
    fn test_format_char_literal() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::Char('a'));
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "'a'");
    }
    
    #[test]
    fn test_format_unit_literal() {
        let formatter = Formatter::new();
        let expr = create_literal_expr(Literal::Unit);
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "()");
    }
    
    #[test]
    fn test_format_identifier() {
        let formatter = Formatter::new();
        let expr = create_identifier_expr("my_variable");
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_variable");
    }
    
    #[test]
    fn test_format_binary_expression() {
        let formatter = Formatter::new();
        
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Integer(5))),
                op: BinaryOp::Add,
                right: Box::new(create_literal_expr(Literal::Integer(3))),
            },
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("5"));
        assert!(formatted.contains("3"));
        assert!(formatted.contains("+"));
    }
    
    #[test]
    fn test_format_unary_expression() {
        let formatter = Formatter::new();
        
        let expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(create_literal_expr(Literal::Integer(42))),
            },
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("42"));
    }
    
    #[test]
    fn test_format_tuple() {
        let formatter = Formatter::new();
        
        let expr = Expr {
            kind: ExprKind::Tuple(vec![
                create_literal_expr(Literal::Integer(1)),
                create_literal_expr(Literal::Integer(2)),
                create_literal_expr(Literal::Integer(3)),
            ]),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("1"));
        assert!(formatted.contains("2"));
        assert!(formatted.contains("3"));
    }
    
    #[test]
    fn test_format_list() {
        let formatter = Formatter::new();
        
        let expr = Expr {
            kind: ExprKind::List(vec![
                create_literal_expr(Literal::String("a".to_string())),
                create_literal_expr(Literal::String("b".to_string())),
            ]),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("a"));
        assert!(formatted.contains("b"));
    }
    
    #[test]
    fn test_format_if_expression() {
        let formatter = Formatter::new();
        
        let expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(create_literal_expr(Literal::Bool(true))),
                then_branch: Box::new(create_literal_expr(Literal::Integer(1))),
                else_branch: Some(Box::new(create_literal_expr(Literal::Integer(2)))),
            },
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
        let formatted = result.unwrap();
        assert!(formatted.contains("if"));
        assert!(formatted.contains("true"));
    }
    
    #[test]
    fn test_format_complex_nested() {
        let formatter = Formatter::new();
        
        // Create nested expression: (1 + 2) * 3
        let add_expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(create_literal_expr(Literal::Integer(1))),
                op: BinaryOp::Add,
                right: Box::new(create_literal_expr(Literal::Integer(2))),
            },
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(add_expr),
                op: BinaryOp::Multiply,
                right: Box::new(create_literal_expr(Literal::Integer(3))),
            },
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let result = formatter.format(&expr);
        assert!(result.is_ok());
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_format_never_panics(n in any::<i64>()) {
            let formatter = Formatter::new();
            let expr = create_literal_expr(Literal::Integer(n));
            let _ = formatter.format(&expr); // Should not panic
        }
        
        #[test]
        fn test_format_preserves_integers(
            ints in prop::collection::vec(-1000i64..1000, 1..10)
        ) {
            let formatter = Formatter::new();
            
            for n in ints {
                let expr = create_literal_expr(Literal::Integer(n));
                let result = formatter.format(&expr);
                
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), n.to_string());
            }
        }
        
        #[test]
        fn test_format_string_safety(s in "\\PC*") {
            let formatter = Formatter::new();
            let expr = create_literal_expr(Literal::String(s.clone()));
            
            let result = formatter.format(&expr);
            assert!(result.is_ok());
            
            let formatted = result.unwrap();
            assert!(formatted.starts_with('"'));
            assert!(formatted.ends_with('"'));
        }
    }
}