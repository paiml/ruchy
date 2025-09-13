//! Comprehensive TDD tests for MinimalCodeGen transpiler module
//! Target: Increase coverage from 33.82% to 80%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod codegen_minimal_tests {
    use crate::backend::transpiler::codegen_minimal::MinimalCodeGen;
    use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Pattern, Span};
    
    // ========== Literal Generation Tests ==========
    
    #[test]
    fn test_gen_integer_literal() {
        let expr = create_literal_expr(Literal::Integer(42));
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, "42");
    }
    
    #[test]
    fn test_gen_float_literal() {
        let expr = create_literal_expr(Literal::Float(3.14159));
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, "3.14159");
    }
    
    #[test]
    fn test_gen_bool_literals() {
        let true_expr = create_literal_expr(Literal::Bool(true));
        let result = MinimalCodeGen::gen_expr(&true_expr).unwrap();
        assert_eq!(result, "true");
        
        let false_expr = create_literal_expr(Literal::Bool(false));
        let result = MinimalCodeGen::gen_expr(&false_expr).unwrap();
        assert_eq!(result, "false");
    }
    
    #[test]
    fn test_gen_string_literal() {
        let expr = create_literal_expr(Literal::String("hello world".to_string()));
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, r#""hello world""#);
    }
    
    #[test]
    fn test_gen_char_literal() {
        let expr = create_literal_expr(Literal::Char('a'));
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, "'a'");
    }
    
    // ========== Identifier and Variable Tests ==========
    
    #[test]
    fn test_gen_identifier() {
        let expr = Expr {
            kind: ExprKind::Identifier("my_variable".to_string()),
            span: Span::default(),
        };
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, "my_variable");
    }
    
    #[test]
    fn test_gen_qualified_name() {
        let expr = Expr {
            kind: ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "vec".to_string(),
            },
            span: Span::default(),
        };
        let result = MinimalCodeGen::gen_expr(&expr).unwrap();
        assert_eq!(result, "std::vec");
    }
    
    // ========== Binary Expression Tests ==========
    
    #[test]
    fn test_gen_arithmetic_binary_ops() {
        test_binary_op(BinaryOp::Add, "42", "8", "(42 + 8)");
        test_binary_op(BinaryOp::Subtract, "100", "25", "(100 - 25)");
        test_binary_op(BinaryOp::Multiply, "6", "7", "(6 * 7)");
        test_binary_op(BinaryOp::Divide, "100", "4", "(100 / 4)");
        test_binary_op(BinaryOp::Modulo, "10", "3", "(10 % 3)");
    }
    
    #[test]
    fn test_gen_comparison_binary_ops() {
        test_binary_op(BinaryOp::Equal, "5", "5", "(5 == 5)");
        test_binary_op(BinaryOp::NotEqual, "5", "3", "(5 != 3)");
        test_binary_op(BinaryOp::Less, "3", "5", "(3 < 5)");
        test_binary_op(BinaryOp::Greater, "5", "3", "(5 > 3)");
        test_binary_op(BinaryOp::LessEqual, "3", "5", "(3 <= 5)");
        test_binary_op(BinaryOp::GreaterEqual, "5", "3", "(5 >= 3)");
    }
    
    #[test]
    fn test_gen_logical_binary_ops() {
        test_binary_op(BinaryOp::And, "true", "false", "(true && false)");
        test_binary_op(BinaryOp::Or, "true", "false", "(true || false)");
    }
    
    // ========== Unary Expression Tests ==========
    
    #[test]
    fn test_gen_unary_ops() {
        // Test negation
        let neg_expr = create_unary_expr(UnaryOp::Negate, Literal::Integer(42));
        let result = MinimalCodeGen::gen_expr(&neg_expr).unwrap();
        assert_eq!(result, "(-42)");
        
        // Test logical not
        let not_expr = create_unary_expr(UnaryOp::Not, Literal::Bool(true));
        let result = MinimalCodeGen::gen_expr(&not_expr).unwrap();
        assert_eq!(result, "(!true)");
    }
    
    // ========== Control Flow Tests ==========
    
    #[test]
    fn test_gen_if_expression() {
        let condition = create_literal_expr(Literal::Bool(true));
        let then_branch = create_literal_expr(Literal::Integer(1));
        let else_branch = Some(Box::new(create_literal_expr(Literal::Integer(2))));
        
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&if_expr).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("true"));
        assert!(result.contains("1"));
        assert!(result.contains("else"));
        assert!(result.contains("2"));
    }
    
    #[test]
    fn test_gen_if_without_else() {
        let condition = create_literal_expr(Literal::Bool(false));
        let then_branch = create_literal_expr(Literal::String("yes".to_string()));
        
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: None,
            },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&if_expr).unwrap();
        assert!(result.contains("if"));
        assert!(result.contains("false"));
        assert!(result.contains(r#""yes""#));
    }
    
    // ========== Block Expression Tests ==========
    
    #[test]
    fn test_gen_block_expression() {
        let exprs = vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::Integer(2)),
            create_literal_expr(Literal::Integer(3)),
        ];
        
        let block_expr = Expr {
            kind: ExprKind::Block(exprs),
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&block_expr).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }
    
    #[test]
    fn test_gen_empty_block() {
        let block_expr = Expr {
            kind: ExprKind::Block(vec![]),
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&block_expr).unwrap();
        assert_eq!(result, "{ () }");
    }
    
    // ========== List Expression Tests ==========
    
    #[test]
    fn test_gen_list_expression() {
        let elements = vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::Integer(2)),
            create_literal_expr(Literal::Integer(3)),
        ];
        
        let list_expr = Expr {
            kind: ExprKind::List(elements),
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&list_expr).unwrap();
        assert_eq!(result, "vec![1, 2, 3]");
    }
    
    #[test]
    fn test_gen_empty_list() {
        let list_expr = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&list_expr).unwrap();
        assert_eq!(result, "vec![]");
    }
    
    // ========== Function and Lambda Tests ==========
    
    #[test]
    fn test_gen_lambda_expression() {
        let params = vec!["x".to_string(), "y".to_string()];
        let body = Box::new(create_literal_expr(Literal::Integer(42)));
        
        let lambda_expr = Expr {
            kind: ExprKind::Lambda { params, body },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&lambda_expr).unwrap();
        assert!(result.contains("|x, y|"));
        assert!(result.contains("42"));
    }
    
    #[test]
    fn test_gen_lambda_no_params() {
        let params = vec![];
        let body = Box::new(create_literal_expr(Literal::String("hello".to_string())));
        
        let lambda_expr = Expr {
            kind: ExprKind::Lambda { params, body },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&lambda_expr).unwrap();
        assert!(result.contains("||"));
        assert!(result.contains(r#""hello""#));
    }
    
    // ========== Call Expression Tests ==========
    
    #[test]
    fn test_gen_function_call() {
        let func = Box::new(Expr {
            kind: ExprKind::Identifier("println".to_string()),
            span: Span::default(),
        });
        let args = vec![create_literal_expr(Literal::String("Hello!".to_string()))];
        
        let call_expr = Expr {
            kind: ExprKind::Call { func, args },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&call_expr).unwrap();
        assert_eq!(result, r#"println("Hello!")"#);
    }
    
    #[test]
    fn test_gen_method_call() {
        let receiver = Box::new(create_literal_expr(Literal::String("hello".to_string())));
        let method = "len".to_string();
        let args = vec![];
        
        let method_call = Expr {
            kind: ExprKind::MethodCall { receiver, method, args },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&method_call).unwrap();
        assert_eq!(result, r#""hello".len()"#);
    }
    
    // ========== Macro Call Tests ==========
    
    #[test]
    fn test_gen_macro_call() {
        let name = "vec".to_string();
        let args = vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::Integer(2)),
        ];
        
        let macro_expr = Expr {
            kind: ExprKind::Macro { name, args },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&macro_expr).unwrap();
        assert_eq!(result, "vec![1, 2]");
    }
    
    // ========== Error Handling Tests ==========
    
    #[test]
    fn test_gen_unsupported_expression() {
        // Test that unsupported expressions return errors
        let expr = Expr {
            kind: ExprKind::Async(Box::new(create_literal_expr(Literal::Integer(42)))),
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&expr);
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(error.contains("Minimal codegen does not support"));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    /// Helper: Create a literal expression
    fn create_literal_expr(lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(lit),
            span: Span::default(),
        }
    }
    
    /// Helper: Create a unary expression
    fn create_unary_expr(op: UnaryOp, lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(create_literal_expr(lit)),
            },
            span: Span::default(),
        }
    }
    
    /// Helper: Test binary operation code generation
    fn test_binary_op(op: BinaryOp, left_str: &str, right_str: &str, expected: &str) {
        let left = Expr {
            kind: ExprKind::Identifier(left_str.to_string()),
            span: Span::default(),
        };
        let right = Expr {
            kind: ExprKind::Identifier(right_str.to_string()),
            span: Span::default(),
        };
        
        let binary_expr = Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Span::default(),
        };
        
        let result = MinimalCodeGen::gen_expr(&binary_expr).unwrap();
        assert_eq!(result, expected);
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_gen_integer_never_panics(n: i64) {
            let expr = create_literal_expr(Literal::Integer(n));
            let result = MinimalCodeGen::gen_expr(&expr);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), n.to_string());
        }
        
        #[test]
        fn test_gen_string_never_panics(s in "[a-zA-Z0-9 ]{0,50}") {
            let expr = create_literal_expr(Literal::String(s.clone()));
            let result = MinimalCodeGen::gen_expr(&expr);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), format!(r#""{}"#, s));
        }
        
        #[test]
        fn test_gen_identifier_never_panics(name in "[a-z][a-z0-9_]{0,20}") {
            let expr = Expr {
                kind: ExprKind::Identifier(name.clone()),
                span: Span::default(),
            };
            let result = MinimalCodeGen::gen_expr(&expr);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), name);
        }
    }
}