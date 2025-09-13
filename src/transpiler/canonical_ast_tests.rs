//! Comprehensive tests for Canonical AST Normalization
//! Target: Increase coverage for transpiler/canonical_ast
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod canonical_ast_tests {
    use crate::transpiler::canonical_ast::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, BinaryOp, Span};
    
    // ========== Helper Functions ==========
    
    fn create_test_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
        }
    }
    
    // ========== DeBruijnIndex Tests ==========
    
    #[test]
    fn test_debruijn_index_creation() {
        let idx = DeBruijnIndex(0);
        assert_eq!(idx.0, 0);
        
        let idx2 = DeBruijnIndex(42);
        assert_eq!(idx2.0, 42);
    }
    
    #[test]
    fn test_debruijn_index_equality() {
        let idx1 = DeBruijnIndex(5);
        let idx2 = DeBruijnIndex(5);
        let idx3 = DeBruijnIndex(6);
        
        assert_eq!(idx1, idx2);
        assert_ne!(idx1, idx3);
    }
    
    #[test]
    fn test_debruijn_index_clone() {
        let idx = DeBruijnIndex(10);
        let cloned = idx.clone();
        
        assert_eq!(idx, cloned);
    }
    
    // ========== CoreLiteral Tests ==========
    
    #[test]
    fn test_core_literal_integer() {
        let lit = CoreLiteral::Integer(42);
        
        match lit {
            CoreLiteral::Integer(n) => assert_eq!(n, 42),
            _ => panic!("Expected Integer"),
        }
    }
    
    #[test]
    fn test_core_literal_float() {
        let lit = CoreLiteral::Float(3.14);
        
        match lit {
            CoreLiteral::Float(f) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_core_literal_string() {
        let lit = CoreLiteral::String("hello".to_string());
        
        match lit {
            CoreLiteral::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String"),
        }
    }
    
    #[test]
    fn test_core_literal_bool() {
        let lit_true = CoreLiteral::Bool(true);
        let lit_false = CoreLiteral::Bool(false);
        
        match lit_true {
            CoreLiteral::Bool(b) => assert!(b),
            _ => panic!("Expected Bool"),
        }
        
        match lit_false {
            CoreLiteral::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool"),
        }
    }
    
    #[test]
    fn test_core_literal_char() {
        let lit = CoreLiteral::Char('a');
        
        match lit {
            CoreLiteral::Char(c) => assert_eq!(c, 'a'),
            _ => panic!("Expected Char"),
        }
    }
    
    #[test]
    fn test_core_literal_unit() {
        let lit = CoreLiteral::Unit;
        
        match lit {
            CoreLiteral::Unit => {},
            _ => panic!("Expected Unit"),
        }
    }
    
    // ========== PrimOp Tests ==========
    
    #[test]
    fn test_prim_op_arithmetic() {
        let ops = vec![
            PrimOp::Add,
            PrimOp::Sub,
            PrimOp::Mul,
            PrimOp::Div,
            PrimOp::Mod,
            PrimOp::Pow,
        ];
        
        for op in ops {
            match op {
                PrimOp::Add => assert_eq!(op, PrimOp::Add),
                PrimOp::Sub => assert_eq!(op, PrimOp::Sub),
                PrimOp::Mul => assert_eq!(op, PrimOp::Mul),
                PrimOp::Div => assert_eq!(op, PrimOp::Div),
                PrimOp::Mod => assert_eq!(op, PrimOp::Mod),
                PrimOp::Pow => assert_eq!(op, PrimOp::Pow),
                _ => panic!("Unexpected arithmetic op"),
            }
        }
    }
    
    #[test]
    fn test_prim_op_comparison() {
        let ops = vec![
            PrimOp::Eq,
            PrimOp::Ne,
            PrimOp::Lt,
            PrimOp::Le,
            PrimOp::Gt,
            PrimOp::Ge,
        ];
        
        for op in ops {
            match op {
                PrimOp::Eq => assert_eq!(op, PrimOp::Eq),
                PrimOp::Ne => assert_eq!(op, PrimOp::Ne),
                PrimOp::Lt => assert_eq!(op, PrimOp::Lt),
                PrimOp::Le => assert_eq!(op, PrimOp::Le),
                PrimOp::Gt => assert_eq!(op, PrimOp::Gt),
                PrimOp::Ge => assert_eq!(op, PrimOp::Ge),
                _ => panic!("Unexpected comparison op"),
            }
        }
    }
    
    #[test]
    fn test_prim_op_logical() {
        let ops = vec![
            PrimOp::And,
            PrimOp::Or,
            PrimOp::Not,
            PrimOp::NullCoalesce,
        ];
        
        for op in ops {
            match op {
                PrimOp::And => assert_eq!(op, PrimOp::And),
                PrimOp::Or => assert_eq!(op, PrimOp::Or),
                PrimOp::Not => assert_eq!(op, PrimOp::Not),
                PrimOp::NullCoalesce => assert_eq!(op, PrimOp::NullCoalesce),
                _ => panic!("Unexpected logical op"),
            }
        }
    }
    
    #[test]
    fn test_prim_op_other() {
        let ops = vec![
            PrimOp::Concat,
            PrimOp::ArrayNew,
            PrimOp::ArrayIndex,
            PrimOp::ArrayLen,
            PrimOp::If,
        ];
        
        for op in ops {
            match op {
                PrimOp::Concat => assert_eq!(op, PrimOp::Concat),
                PrimOp::ArrayNew => assert_eq!(op, PrimOp::ArrayNew),
                PrimOp::ArrayIndex => assert_eq!(op, PrimOp::ArrayIndex),
                PrimOp::ArrayLen => assert_eq!(op, PrimOp::ArrayLen),
                PrimOp::If => assert_eq!(op, PrimOp::If),
                _ => panic!("Unexpected other op"),
            }
        }
    }
    
    // ========== CoreExpr Tests ==========
    
    #[test]
    fn test_core_expr_var() {
        let expr = CoreExpr::Var(DeBruijnIndex(3));
        
        match expr {
            CoreExpr::Var(idx) => assert_eq!(idx.0, 3),
            _ => panic!("Expected Var"),
        }
    }
    
    #[test]
    fn test_core_expr_lambda() {
        let body = Box::new(CoreExpr::Var(DeBruijnIndex(0)));
        let expr = CoreExpr::Lambda {
            param_name: Some("x".to_string()),
            body,
        };
        
        match expr {
            CoreExpr::Lambda { param_name, body } => {
                assert_eq!(param_name, Some("x".to_string()));
                match *body {
                    CoreExpr::Var(idx) => assert_eq!(idx.0, 0),
                    _ => panic!("Expected Var in body"),
                }
            }
            _ => panic!("Expected Lambda"),
        }
    }
    
    #[test]
    fn test_core_expr_app() {
        let func = Box::new(CoreExpr::Var(DeBruijnIndex(1)));
        let arg = Box::new(CoreExpr::Literal(CoreLiteral::Integer(42)));
        let expr = CoreExpr::App(func, arg);
        
        match expr {
            CoreExpr::App(f, a) => {
                match *f {
                    CoreExpr::Var(idx) => assert_eq!(idx.0, 1),
                    _ => panic!("Expected Var as function"),
                }
                match *a {
                    CoreExpr::Literal(CoreLiteral::Integer(n)) => assert_eq!(n, 42),
                    _ => panic!("Expected Integer literal as argument"),
                }
            }
            _ => panic!("Expected App"),
        }
    }
    
    #[test]
    fn test_core_expr_let() {
        let value = Box::new(CoreExpr::Literal(CoreLiteral::Integer(10)));
        let body = Box::new(CoreExpr::Var(DeBruijnIndex(0)));
        let expr = CoreExpr::Let {
            name: Some("x".to_string()),
            value,
            body,
        };
        
        match expr {
            CoreExpr::Let { name, value, body } => {
                assert_eq!(name, Some("x".to_string()));
                match *value {
                    CoreExpr::Literal(CoreLiteral::Integer(n)) => assert_eq!(n, 10),
                    _ => panic!("Expected Integer literal"),
                }
                match *body {
                    CoreExpr::Var(idx) => assert_eq!(idx.0, 0),
                    _ => panic!("Expected Var"),
                }
            }
            _ => panic!("Expected Let"),
        }
    }
    
    #[test]
    fn test_core_expr_prim() {
        let args = vec![
            CoreExpr::Literal(CoreLiteral::Integer(5)),
            CoreExpr::Literal(CoreLiteral::Integer(3)),
        ];
        let expr = CoreExpr::Prim(PrimOp::Add, args);
        
        match expr {
            CoreExpr::Prim(op, args) => {
                assert_eq!(op, PrimOp::Add);
                assert_eq!(args.len(), 2);
                match &args[0] {
                    CoreExpr::Literal(CoreLiteral::Integer(n)) => assert_eq!(*n, 5),
                    _ => panic!("Expected Integer literal"),
                }
                match &args[1] {
                    CoreExpr::Literal(CoreLiteral::Integer(n)) => assert_eq!(*n, 3),
                    _ => panic!("Expected Integer literal"),
                }
            }
            _ => panic!("Expected Prim"),
        }
    }
    
    // ========== AstNormalizer Tests ==========
    
    #[test]
    fn test_normalizer_creation() {
        let normalizer = AstNormalizer::new();
        // Just verify it creates successfully
        let _n = normalizer;
    }
    
    #[test]
    fn test_normalizer_default() {
        let normalizer = AstNormalizer::default();
        // Just verify it creates successfully
        let _n = normalizer;
    }
    
    #[test]
    fn test_normalize_literal() {
        let mut normalizer = AstNormalizer::new();
        
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
        let core = normalizer.normalize(&expr);
        
        match core {
            CoreExpr::Literal(CoreLiteral::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected Integer literal"),
        }
    }
    
    #[test]
    fn test_normalize_binary_add() {
        let mut normalizer = AstNormalizer::new();
        
        let left = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(5))));
        let right = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(3))));
        let expr = create_test_expr(ExprKind::Binary {
            left,
            op: BinaryOp::Add,
            right,
        });
        
        let core = normalizer.normalize(&expr);
        
        match core {
            CoreExpr::Prim(op, args) => {
                assert_eq!(op, PrimOp::Add);
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected Prim operation"),
        }
    }
    
    #[test]
    fn test_normalize_literals() {
        let mut normalizer = AstNormalizer::new();
        
        // Test various literal types
        let literals = vec![
            (Literal::Integer(100), CoreLiteral::Integer(100)),
            (Literal::Float(3.14), CoreLiteral::Float(3.14)),
            (Literal::String("test".to_string()), CoreLiteral::String("test".to_string())),
            (Literal::Bool(true), CoreLiteral::Bool(true)),
            (Literal::Char('x'), CoreLiteral::Char('x')),
        ];
        
        for (ast_lit, expected_core_lit) in literals {
            let expr = create_test_expr(ExprKind::Literal(ast_lit));
            let core = normalizer.normalize(&expr);
            
            match core {
                CoreExpr::Literal(lit) => assert_eq!(lit, expected_core_lit),
                _ => panic!("Expected Literal"),
            }
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_debruijn_index_properties(idx: usize) {
            let index = DeBruijnIndex(idx);
            assert_eq!(index.0, idx);
            
            // Test clone
            let cloned = index.clone();
            assert_eq!(index, cloned);
        }
        
        #[test]
        fn test_core_literal_integer_properties(n: i64) {
            let lit = CoreLiteral::Integer(n);
            
            match lit {
                CoreLiteral::Integer(val) => assert_eq!(val, n),
                _ => panic!("Expected Integer"),
            }
        }
        
        #[test]
        fn test_core_literal_string_properties(s: String) {
            let lit = CoreLiteral::String(s.clone());
            
            match lit {
                CoreLiteral::String(val) => assert_eq!(val, s),
                _ => panic!("Expected String"),
            }
        }
    }
}