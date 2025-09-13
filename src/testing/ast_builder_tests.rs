//! Comprehensive tests for AST Builder
//! Target: Increase coverage for testing/ast_builder
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod ast_builder_tests {
    use crate::testing::ast_builder::AstBuilder;
    use crate::frontend::ast::{ExprKind, Literal, BinaryOp, UnaryOp, Pattern};
    
    // ========== Basic Builder Tests ==========
    
    #[test]
    fn test_builder_creation() {
        let builder = AstBuilder::new();
        // Just verify it creates successfully
        let _b = builder;
    }
    
    // ========== Literal Tests ==========
    
    #[test]
    fn test_int_literal() {
        let builder = AstBuilder::new();
        let expr = builder.int(42);
        
        match expr.kind {
            ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected Integer literal"),
        }
    }
    
    #[test]
    fn test_float_literal() {
        let builder = AstBuilder::new();
        let expr = builder.float(3.14);
        
        match expr.kind {
            ExprKind::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float literal"),
        }
    }
    
    #[test]
    fn test_string_literal() {
        let builder = AstBuilder::new();
        let expr = builder.string("hello");
        
        match expr.kind {
            ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected String literal"),
        }
    }
    
    #[test]
    fn test_bool_literal() {
        let builder = AstBuilder::new();
        
        let true_expr = builder.bool(true);
        match true_expr.kind {
            ExprKind::Literal(Literal::Bool(b)) => assert!(b),
            _ => panic!("Expected Bool literal"),
        }
        
        let false_expr = builder.bool(false);
        match false_expr.kind {
            ExprKind::Literal(Literal::Bool(b)) => assert!(!b),
            _ => panic!("Expected Bool literal"),
        }
    }
    
    // ========== Identifier Tests ==========
    
    #[test]
    fn test_identifier() {
        let builder = AstBuilder::new();
        let expr = builder.ident("variable");
        
        match expr.kind {
            ExprKind::Identifier(name) => assert_eq!(name, "variable"),
            _ => panic!("Expected Identifier"),
        }
    }
    
    #[test]
    fn test_various_identifiers() {
        let builder = AstBuilder::new();
        
        let names = vec!["x", "myVar", "snake_case", "CONSTANT", "_private"];
        
        for name in names {
            let expr = builder.ident(name);
            match expr.kind {
                ExprKind::Identifier(n) => assert_eq!(n, name),
                _ => panic!("Expected Identifier"),
            }
        }
    }
    
    // ========== Binary Operation Tests ==========
    
    #[test]
    fn test_binary_add() {
        let builder = AstBuilder::new();
        let left = builder.int(5);
        let right = builder.int(3);
        let expr = builder.binary(left, BinaryOp::Add, right);
        
        match expr.kind {
            ExprKind::Binary { left, op, right } => {
                assert_eq!(op, BinaryOp::Add);
                match left.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 5),
                    _ => panic!("Expected Integer literal on left"),
                }
                match right.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 3),
                    _ => panic!("Expected Integer literal on right"),
                }
            }
            _ => panic!("Expected Binary operation"),
        }
    }
    
    #[test]
    fn test_binary_operations() {
        let builder = AstBuilder::new();
        
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::Greater,
        ];
        
        for op in ops {
            let left = builder.int(10);
            let right = builder.int(5);
            let expr = builder.binary(left, op.clone(), right);
            
            match expr.kind {
                ExprKind::Binary { op: result_op, .. } => {
                    assert_eq!(result_op, op);
                }
                _ => panic!("Expected Binary operation"),
            }
        }
    }
    
    // ========== Unary Operation Tests ==========
    
    #[test]
    fn test_unary_negate() {
        let builder = AstBuilder::new();
        let operand = builder.int(42);
        let expr = builder.unary(UnaryOp::Negate, operand);
        
        match expr.kind {
            ExprKind::Unary { op, operand } => {
                assert_eq!(op, UnaryOp::Negate);
                match operand.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Expected Integer literal"),
                }
            }
            _ => panic!("Expected Unary operation"),
        }
    }
    
    #[test]
    fn test_unary_not() {
        let builder = AstBuilder::new();
        let operand = builder.bool(true);
        let expr = builder.unary(UnaryOp::Not, operand);
        
        match expr.kind {
            ExprKind::Unary { op, operand } => {
                assert_eq!(op, UnaryOp::Not);
                match operand.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Expected Bool literal"),
                }
            }
            _ => panic!("Expected Unary operation"),
        }
    }
    
    // ========== Control Flow Tests ==========
    
    #[test]
    fn test_if_expression() {
        let builder = AstBuilder::new();
        let condition = builder.bool(true);
        let then_branch = builder.int(1);
        let else_branch = Some(builder.int(2));
        
        let expr = builder.if_expr(condition, then_branch, else_branch);
        
        match expr.kind {
            ExprKind::If { condition, then_branch, else_branch } => {
                match condition.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Expected Bool condition"),
                }
                match then_branch.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Expected Integer in then branch"),
                }
                assert!(else_branch.is_some());
                match else_branch.unwrap().kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 2),
                    _ => panic!("Expected Integer in else branch"),
                }
            }
            _ => panic!("Expected If expression"),
        }
    }
    
    #[test]
    fn test_if_without_else() {
        let builder = AstBuilder::new();
        let condition = builder.bool(false);
        let then_branch = builder.string("then");
        
        let expr = builder.if_expr(condition, then_branch, None);
        
        match expr.kind {
            ExprKind::If { else_branch, .. } => {
                assert!(else_branch.is_none());
            }
            _ => panic!("Expected If expression"),
        }
    }
    
    #[test]
    fn test_match_expression() {
        let builder = AstBuilder::new();
        let expr = builder.int(1);
        
        let pattern1 = Pattern::Literal(Literal::Integer(1));
        let arm1 = builder.match_arm(pattern1, None, builder.string("one"));
        
        let pattern2 = Pattern::Wildcard;
        let arm2 = builder.match_arm(pattern2, None, builder.string("other"));
        
        let match_expr = builder.match_expr(expr, vec![arm1, arm2]);
        
        match match_expr.kind {
            ExprKind::Match { expr, arms } => {
                match expr.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 1),
                    _ => panic!("Expected Integer literal"),
                }
                assert_eq!(arms.len(), 2);
            }
            _ => panic!("Expected Match expression"),
        }
    }
    
    #[test]
    fn test_match_arm_with_guard() {
        let builder = AstBuilder::new();
        
        let pattern = Pattern::Identifier("x".to_string());
        let guard = Some(builder.binary(
            builder.ident("x"),
            BinaryOp::Greater,
            builder.int(0)
        ));
        let body = builder.string("positive");
        
        let arm = builder.match_arm(pattern, guard, body);
        
        assert!(arm.guard.is_some());
        match arm.body.kind {
            ExprKind::Literal(Literal::String(s)) => assert_eq!(s, "positive"),
            _ => panic!("Expected String literal in body"),
        }
    }
    
    // ========== Complex Expression Tests ==========
    
    #[test]
    fn test_nested_binary_expression() {
        let builder = AstBuilder::new();
        
        // (2 + 3) * 4
        let add = builder.binary(
            builder.int(2),
            BinaryOp::Add,
            builder.int(3)
        );
        let multiply = builder.binary(
            add,
            BinaryOp::Multiply,
            builder.int(4)
        );
        
        match multiply.kind {
            ExprKind::Binary { left, op, right } => {
                assert_eq!(op, BinaryOp::Multiply);
                match left.kind {
                    ExprKind::Binary { op, .. } => assert_eq!(op, BinaryOp::Add),
                    _ => panic!("Expected Binary on left"),
                }
                match right.kind {
                    ExprKind::Literal(Literal::Integer(n)) => assert_eq!(n, 4),
                    _ => panic!("Expected Integer on right"),
                }
            }
            _ => panic!("Expected Binary operation"),
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_int_literal_properties(n: i64) {
            let builder = AstBuilder::new();
            let expr = builder.int(n);
            
            match expr.kind {
                ExprKind::Literal(Literal::Integer(val)) => assert_eq!(val, n),
                _ => panic!("Expected Integer literal"),
            }
        }
        
        #[test]
        fn test_string_literal_properties(s: String) {
            let builder = AstBuilder::new();
            let expr = builder.string(&s);
            
            match expr.kind {
                ExprKind::Literal(Literal::String(val)) => assert_eq!(val, s),
                _ => panic!("Expected String literal"),
            }
        }
        
        #[test]
        fn test_identifier_properties(name in "[a-zA-Z_][a-zA-Z0-9_]*") {
            let builder = AstBuilder::new();
            let expr = builder.ident(&name);
            
            match expr.kind {
                ExprKind::Identifier(n) => assert_eq!(n, name),
                _ => panic!("Expected Identifier"),
            }
        }
    }
}