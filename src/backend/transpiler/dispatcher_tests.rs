//! Comprehensive TDD tests for Transpiler Dispatcher module
//! Target: Increase coverage from 33.09% to 80%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod dispatcher_tests {
    use crate::backend::transpiler::Transpiler;
    use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, UnaryOp, Span, Pattern};
    use proc_macro2::TokenStream;
    use quote::quote;
    
    // ========== Basic Expression Tests ==========
    
    #[test]
    fn test_transpile_integer_literal() {
        let transpiler = Transpiler::new();
        let expr = create_literal_expr(Literal::Integer(42));
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "42");
    }
    
    #[test]
    fn test_transpile_float_literal() {
        let transpiler = Transpiler::new();
        let expr = create_literal_expr(Literal::Float(3.14));
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "3.14");
    }
    
    #[test]
    fn test_transpile_bool_literals() {
        let transpiler = Transpiler::new();
        
        let true_expr = create_literal_expr(Literal::Bool(true));
        let result = transpiler.transpile_basic_expr(&true_expr).unwrap();
        assert_eq!(result.to_string(), "true");
        
        let false_expr = create_literal_expr(Literal::Bool(false));
        let result = transpiler.transpile_basic_expr(&false_expr).unwrap();
        assert_eq!(result.to_string(), "false");
    }
    
    #[test]
    fn test_transpile_string_literal() {
        let transpiler = Transpiler::new();
        let expr = create_literal_expr(Literal::String("hello".to_string()));
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        // Quote will include the quotes in the string
        assert!(result.to_string().contains("hello"));
    }
    
    #[test]
    fn test_transpile_char_literal() {
        let transpiler = Transpiler::new();
        let expr = create_literal_expr(Literal::Char('x'));
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "'x'");
    }
    
    // ========== Identifier Tests ==========
    
    #[test]
    fn test_transpile_simple_identifier() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::Identifier("my_var".to_string()),
            span: Span::default(),
        };
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "my_var");
    }
    
    #[test]
    fn test_transpile_qualified_name() {
        let transpiler = Transpiler::new();
        let expr = Expr {
            kind: ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "vec".to_string(),
            },
            span: Span::default(),
        };
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert_eq!(result.to_string(), "std :: vec");
    }
    
    // ========== Type Cast Tests ==========
    
    #[test]
    fn test_transpile_type_cast_to_i32() {
        let transpiler = Transpiler::new();
        let inner_expr = Box::new(create_literal_expr(Literal::Integer(42)));
        let expr = Expr {
            kind: ExprKind::TypeCast {
                expr: inner_expr,
                target_type: "i32".to_string(),
            },
            span: Span::default(),
        };
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert!(result.to_string().contains("as i32"));
    }
    
    #[test]
    fn test_transpile_type_cast_to_f64() {
        let transpiler = Transpiler::new();
        let inner_expr = Box::new(create_literal_expr(Literal::Integer(100)));
        let expr = Expr {
            kind: ExprKind::TypeCast {
                expr: inner_expr,
                target_type: "f64".to_string(),
            },
            span: Span::default(),
        };
        let result = transpiler.transpile_basic_expr(&expr).unwrap();
        assert!(result.to_string().contains("as f64"));
    }
    
    #[test]
    fn test_transpile_type_cast_various_types() {
        let transpiler = Transpiler::new();
        let types = vec!["i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "usize", "f32", "f64"];
        
        for type_name in types {
            let inner_expr = Box::new(create_literal_expr(Literal::Integer(42)));
            let expr = Expr {
                kind: ExprKind::TypeCast {
                    expr: inner_expr,
                    target_type: type_name.to_string(),
                },
                span: Span::default(),
            };
            let result = transpiler.transpile_basic_expr(&expr).unwrap();
            assert!(result.to_string().contains(&format!("as {}", type_name)));
        }
    }
    
    #[test]
    fn test_transpile_unsupported_type_cast() {
        let transpiler = Transpiler::new();
        let inner_expr = Box::new(create_literal_expr(Literal::Integer(42)));
        let expr = Expr {
            kind: ExprKind::TypeCast {
                expr: inner_expr,
                target_type: "MyCustomType".to_string(),
            },
            span: Span::default(),
        };
        let result = transpiler.transpile_basic_expr(&expr);
        assert!(result.is_err());
    }
    
    // ========== Control Flow Tests ==========
    
    #[test]
    fn test_transpile_control_flow_if() {
        let transpiler = Transpiler::new();
        let condition = Box::new(create_literal_expr(Literal::Bool(true)));
        let then_branch = Box::new(create_literal_expr(Literal::Integer(1)));
        let else_branch = Some(Box::new(create_literal_expr(Literal::Integer(2))));
        
        let if_expr = Expr {
            kind: ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_control_flow_expr(&if_expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("if"));
        assert!(result_str.contains("else"));
    }
    
    #[test]
    fn test_transpile_control_flow_match() {
        let transpiler = Transpiler::new();
        let expr = Box::new(create_literal_expr(Literal::Integer(42)));
        let arms = vec![];  // Empty match arms for simplicity
        
        let match_expr = Expr {
            kind: ExprKind::Match { expr, arms },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_control_flow_expr(&match_expr).unwrap();
        assert!(result.to_string().contains("match"));
    }
    
    // ========== Data Structure Tests ==========
    
    #[test]
    fn test_transpile_list_literal() {
        let transpiler = Transpiler::new();
        let elements = vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::Integer(2)),
            create_literal_expr(Literal::Integer(3)),
        ];
        
        let list_expr = Expr {
            kind: ExprKind::List(elements),
            span: Span::default(),
        };
        
        let result = transpiler.transpile_data_structure_expr(&list_expr).unwrap();
        assert!(result.to_string().contains("vec!"));
    }
    
    #[test]
    fn test_transpile_tuple_literal() {
        let transpiler = Transpiler::new();
        let elements = vec![
            create_literal_expr(Literal::Integer(1)),
            create_literal_expr(Literal::String("hello".to_string())),
        ];
        
        let tuple_expr = Expr {
            kind: ExprKind::Tuple(elements),
            span: Span::default(),
        };
        
        let result = transpiler.transpile_data_structure_expr(&tuple_expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("("));
        assert!(result_str.contains(")"));
    }
    
    #[test]
    fn test_transpile_empty_list() {
        let transpiler = Transpiler::new();
        let list_expr = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
        };
        
        let result = transpiler.transpile_data_structure_expr(&list_expr).unwrap();
        assert!(result.to_string().contains("vec!"));
    }
    
    // ========== Function Tests ==========
    
    #[test]
    fn test_transpile_lambda() {
        let transpiler = Transpiler::new();
        let params = vec!["x".to_string(), "y".to_string()];
        let body = Box::new(create_literal_expr(Literal::Integer(42)));
        
        let lambda_expr = Expr {
            kind: ExprKind::Lambda { params, body },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_function_expr(&lambda_expr).unwrap();
        let result_str = result.to_string();
        assert!(result_str.contains("|"));
        assert!(result_str.contains("x"));
        assert!(result_str.contains("y"));
    }
    
    #[test]
    fn test_transpile_function_call() {
        let transpiler = Transpiler::new();
        let func = Box::new(Expr {
            kind: ExprKind::Identifier("println".to_string()),
            span: Span::default(),
        });
        let args = vec![create_literal_expr(Literal::String("test".to_string()))];
        
        let call_expr = Expr {
            kind: ExprKind::Call { func, args },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_function_expr(&call_expr).unwrap();
        assert!(result.to_string().contains("println"));
    }
    
    // ========== Advanced Expression Tests ==========
    
    #[test]
    fn test_transpile_binary_operation() {
        let transpiler = Transpiler::new();
        let left = Box::new(create_literal_expr(Literal::Integer(5)));
        let right = Box::new(create_literal_expr(Literal::Integer(3)));
        
        let binary_expr = Expr {
            kind: ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_advanced_expr(&binary_expr).unwrap();
        assert!(result.to_string().contains("+"));
    }
    
    #[test]
    fn test_transpile_unary_operation() {
        let transpiler = Transpiler::new();
        let operand = Box::new(create_literal_expr(Literal::Integer(42)));
        
        let unary_expr = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Negate,
                operand,
            },
            span: Span::default(),
        };
        
        let result = transpiler.transpile_advanced_expr(&unary_expr).unwrap();
        assert!(result.to_string().contains("-"));
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    /// Helper: Create a literal expression for testing
    fn create_literal_expr(lit: Literal) -> Expr {
        Expr {
            kind: ExprKind::Literal(lit),
            span: Span::default(),
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_transpile_integer_never_panics(n: i64) {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::Integer(n));
            let result = transpiler.transpile_basic_expr(&expr);
            assert!(result.is_ok());
        }
        
        #[test]
        fn test_transpile_float_never_panics(f: f64) {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::Float(f));
            let result = transpiler.transpile_basic_expr(&expr);
            assert!(result.is_ok());
        }
        
        #[test]
        fn test_transpile_string_never_panics(s in "[a-zA-Z0-9 ]{0,50}") {
            let transpiler = Transpiler::new();
            let expr = create_literal_expr(Literal::String(s));
            let result = transpiler.transpile_basic_expr(&expr);
            assert!(result.is_ok());
        }
    }
}