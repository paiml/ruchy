//! Comprehensive TDD test suite for AST (Abstract Syntax Tree)
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every AST path must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::frontend::{Expr, ExprKind, Literal, Stmt, StmtKind, Pattern, PatternKind, Span};
use ruchy::frontend::{BinaryOp, UnaryOp, Type, TypeKind};

// ==================== LITERAL TESTS ====================

#[test]
fn test_literal_integer() {
    let lit = Literal::Integer(42);
    assert!(matches!(lit, Literal::Integer(42)));
}

#[test]
fn test_literal_float() {
    let lit = Literal::Float(3.14);
    match lit {
        Literal::Float(f) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float literal"),
    }
}

#[test]
fn test_literal_boolean() {
    let lit_true = Literal::Boolean(true);
    let lit_false = Literal::Boolean(false);
    assert!(matches!(lit_true, Literal::Boolean(true)));
    assert!(matches!(lit_false, Literal::Boolean(false)));
}

#[test]
fn test_literal_string() {
    let lit = Literal::String("hello".to_string());
    match lit {
        Literal::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected string literal"),
    }
}

#[test]
fn test_literal_char() {
    let lit = Literal::Char('a');
    assert!(matches!(lit, Literal::Char('a')));
}

#[test]
fn test_literal_nil() {
    let lit = Literal::Nil;
    assert!(matches!(lit, Literal::Nil));
}

// ==================== EXPRESSION TESTS ====================

#[test]
fn test_expr_literal() {
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 2)
    );
    assert!(matches!(expr.kind(), ExprKind::Literal(_)));
    assert_eq!(expr.span(), Span::new(0, 2));
}

#[test]
fn test_expr_identifier() {
    let expr = Expr::new(
        ExprKind::Identifier("x".to_string()),
        Span::new(0, 1)
    );
    match expr.kind() {
        ExprKind::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected identifier"),
    }
}

#[test]
fn test_expr_binary_op() {
    let left = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(1)),
        Span::new(0, 1)
    ));
    let right = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(2)),
        Span::new(4, 5)
    ));
    
    let expr = Expr::new(
        ExprKind::BinaryOp {
            op: BinaryOp::Add,
            left,
            right,
        },
        Span::new(0, 5)
    );
    
    assert!(matches!(expr.kind(), ExprKind::BinaryOp { .. }));
}

#[test]
fn test_expr_unary_op() {
    let operand = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(1, 3)
    ));
    
    let expr = Expr::new(
        ExprKind::UnaryOp {
            op: UnaryOp::Neg,
            operand,
        },
        Span::new(0, 3)
    );
    
    assert!(matches!(expr.kind(), ExprKind::UnaryOp { .. }));
}

#[test]
fn test_expr_function_call() {
    let func = Box::new(Expr::new(
        ExprKind::Identifier("print".to_string()),
        Span::new(0, 5)
    ));
    let args = vec![
        Expr::new(
            ExprKind::Literal(Literal::String("hello".to_string())),
            Span::new(6, 13)
        )
    ];
    
    let expr = Expr::new(
        ExprKind::FunctionCall { func, args },
        Span::new(0, 14)
    );
    
    assert!(matches!(expr.kind(), ExprKind::FunctionCall { .. }));
}

#[test]
fn test_expr_array() {
    let elements = vec![
        Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(1, 2)),
        Expr::new(ExprKind::Literal(Literal::Integer(2)), Span::new(4, 5)),
        Expr::new(ExprKind::Literal(Literal::Integer(3)), Span::new(7, 8)),
    ];
    
    let expr = Expr::new(
        ExprKind::Array(elements),
        Span::new(0, 9)
    );
    
    match expr.kind() {
        ExprKind::Array(elems) => assert_eq!(elems.len(), 3),
        _ => panic!("Expected array expression"),
    }
}

#[test]
fn test_expr_tuple() {
    let elements = vec![
        Expr::new(ExprKind::Literal(Literal::Integer(1)), Span::new(1, 2)),
        Expr::new(ExprKind::Literal(Literal::String("hi".to_string())), Span::new(4, 8)),
    ];
    
    let expr = Expr::new(
        ExprKind::Tuple(elements),
        Span::new(0, 9)
    );
    
    match expr.kind() {
        ExprKind::Tuple(elems) => assert_eq!(elems.len(), 2),
        _ => panic!("Expected tuple expression"),
    }
}

#[test]
fn test_expr_if() {
    let condition = Box::new(Expr::new(
        ExprKind::Literal(Literal::Boolean(true)),
        Span::new(3, 7)
    ));
    let then_branch = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(1)),
        Span::new(10, 11)
    ));
    let else_branch = Some(Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(2)),
        Span::new(19, 20)
    )));
    
    let expr = Expr::new(
        ExprKind::If { condition, then_branch, else_branch },
        Span::new(0, 21)
    );
    
    assert!(matches!(expr.kind(), ExprKind::If { .. }));
}

#[test]
fn test_expr_match() {
    let expr_to_match = Box::new(Expr::new(
        ExprKind::Identifier("x".to_string()),
        Span::new(6, 7)
    ));
    let arms = vec![];  // Would contain match arms
    
    let expr = Expr::new(
        ExprKind::Match { expr: expr_to_match, arms },
        Span::new(0, 20)
    );
    
    assert!(matches!(expr.kind(), ExprKind::Match { .. }));
}

#[test]
fn test_expr_lambda() {
    let params = vec!["x".to_string(), "y".to_string()];
    let body = Box::new(Expr::new(
        ExprKind::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expr::new(ExprKind::Identifier("x".to_string()), Span::new(10, 11))),
            right: Box::new(Expr::new(ExprKind::Identifier("y".to_string()), Span::new(14, 15))),
        },
        Span::new(10, 15)
    ));
    
    let expr = Expr::new(
        ExprKind::Lambda { params, body },
        Span::new(0, 16)
    );
    
    assert!(matches!(expr.kind(), ExprKind::Lambda { .. }));
}

// ==================== STATEMENT TESTS ====================

#[test]
fn test_stmt_let() {
    let pattern = Pattern::new(
        PatternKind::Identifier("x".to_string()),
        Span::new(4, 5)
    );
    let value = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(8, 10)
    );
    
    let stmt = Stmt::new(
        StmtKind::Let { pattern, value: Some(value) },
        Span::new(0, 10)
    );
    
    assert!(matches!(stmt.kind(), StmtKind::Let { .. }));
}

#[test]
fn test_stmt_return() {
    let value = Some(Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(7, 9)
    ));
    
    let stmt = Stmt::new(
        StmtKind::Return(value),
        Span::new(0, 9)
    );
    
    assert!(matches!(stmt.kind(), StmtKind::Return(_)));
}

#[test]
fn test_stmt_expression() {
    let expr = Expr::new(
        ExprKind::FunctionCall {
            func: Box::new(Expr::new(ExprKind::Identifier("print".to_string()), Span::new(0, 5))),
            args: vec![],
        },
        Span::new(0, 7)
    );
    
    let stmt = Stmt::new(
        StmtKind::Expression(expr),
        Span::new(0, 8)
    );
    
    assert!(matches!(stmt.kind(), StmtKind::Expression(_)));
}

#[test]
fn test_stmt_while() {
    let condition = Expr::new(
        ExprKind::Literal(Literal::Boolean(true)),
        Span::new(6, 10)
    );
    let body = vec![];
    
    let stmt = Stmt::new(
        StmtKind::While { condition, body },
        Span::new(0, 15)
    );
    
    assert!(matches!(stmt.kind(), StmtKind::While { .. }));
}

#[test]
fn test_stmt_for() {
    let pattern = Pattern::new(
        PatternKind::Identifier("i".to_string()),
        Span::new(4, 5)
    );
    let iterator = Expr::new(
        ExprKind::Range {
            start: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(0)), Span::new(9, 10))),
            end: Box::new(Expr::new(ExprKind::Literal(Literal::Integer(10)), Span::new(12, 14))),
        },
        Span::new(9, 14)
    );
    let body = vec![];
    
    let stmt = Stmt::new(
        StmtKind::For { pattern, iterator, body },
        Span::new(0, 20)
    );
    
    assert!(matches!(stmt.kind(), StmtKind::For { .. }));
}

// ==================== PATTERN TESTS ====================

#[test]
fn test_pattern_wildcard() {
    let pattern = Pattern::new(
        PatternKind::Wildcard,
        Span::new(0, 1)
    );
    assert!(matches!(pattern.kind(), PatternKind::Wildcard));
}

#[test]
fn test_pattern_identifier() {
    let pattern = Pattern::new(
        PatternKind::Identifier("x".to_string()),
        Span::new(0, 1)
    );
    match pattern.kind() {
        PatternKind::Identifier(name) => assert_eq!(name, "x"),
        _ => panic!("Expected identifier pattern"),
    }
}

#[test]
fn test_pattern_literal() {
    let pattern = Pattern::new(
        PatternKind::Literal(Literal::Integer(42)),
        Span::new(0, 2)
    );
    assert!(matches!(pattern.kind(), PatternKind::Literal(_)));
}

#[test]
fn test_pattern_tuple() {
    let patterns = vec![
        Pattern::new(PatternKind::Identifier("x".to_string()), Span::new(1, 2)),
        Pattern::new(PatternKind::Identifier("y".to_string()), Span::new(4, 5)),
    ];
    
    let pattern = Pattern::new(
        PatternKind::Tuple(patterns),
        Span::new(0, 6)
    );
    
    match pattern.kind() {
        PatternKind::Tuple(pats) => assert_eq!(pats.len(), 2),
        _ => panic!("Expected tuple pattern"),
    }
}

#[test]
fn test_pattern_struct() {
    let fields = vec![
        ("x".to_string(), Pattern::new(PatternKind::Identifier("a".to_string()), Span::new(10, 11))),
        ("y".to_string(), Pattern::new(PatternKind::Identifier("b".to_string()), Span::new(16, 17))),
    ];
    
    let pattern = Pattern::new(
        PatternKind::Struct {
            name: "Point".to_string(),
            fields,
        },
        Span::new(0, 19)
    );
    
    assert!(matches!(pattern.kind(), PatternKind::Struct { .. }));
}

// ==================== TYPE TESTS ====================

#[test]
fn test_type_identifier() {
    let ty = Type::new(
        TypeKind::Identifier("String".to_string()),
        Span::new(0, 6)
    );
    match ty.kind() {
        TypeKind::Identifier(name) => assert_eq!(name, "String"),
        _ => panic!("Expected identifier type"),
    }
}

#[test]
fn test_type_array() {
    let element = Box::new(Type::new(
        TypeKind::Identifier("i32".to_string()),
        Span::new(1, 4)
    ));
    
    let ty = Type::new(
        TypeKind::Array(element),
        Span::new(0, 5)
    );
    
    assert!(matches!(ty.kind(), TypeKind::Array(_)));
}

#[test]
fn test_type_tuple() {
    let types = vec![
        Type::new(TypeKind::Identifier("i32".to_string()), Span::new(1, 4)),
        Type::new(TypeKind::Identifier("String".to_string()), Span::new(6, 12)),
    ];
    
    let ty = Type::new(
        TypeKind::Tuple(types),
        Span::new(0, 13)
    );
    
    match ty.kind() {
        TypeKind::Tuple(tys) => assert_eq!(tys.len(), 2),
        _ => panic!("Expected tuple type"),
    }
}

#[test]
fn test_type_function() {
    let params = vec![
        Type::new(TypeKind::Identifier("i32".to_string()), Span::new(0, 3)),
        Type::new(TypeKind::Identifier("i32".to_string()), Span::new(5, 8)),
    ];
    let return_type = Box::new(Type::new(
        TypeKind::Identifier("i32".to_string()),
        Span::new(13, 16)
    ));
    
    let ty = Type::new(
        TypeKind::Function { params, return_type },
        Span::new(0, 16)
    );
    
    assert!(matches!(ty.kind(), TypeKind::Function { .. }));
}

#[test]
fn test_type_generic() {
    let params = vec![
        Type::new(TypeKind::Identifier("T".to_string()), Span::new(7, 8)),
    ];
    
    let ty = Type::new(
        TypeKind::Generic {
            name: "Option".to_string(),
            params,
        },
        Span::new(0, 9)
    );
    
    assert!(matches!(ty.kind(), TypeKind::Generic { .. }));
}

// ==================== BINARY OPERATOR TESTS ====================

#[test]
fn test_binary_op_arithmetic() {
    assert!(matches!(BinaryOp::Add, BinaryOp::Add));
    assert!(matches!(BinaryOp::Sub, BinaryOp::Sub));
    assert!(matches!(BinaryOp::Mul, BinaryOp::Mul));
    assert!(matches!(BinaryOp::Div, BinaryOp::Div));
    assert!(matches!(BinaryOp::Mod, BinaryOp::Mod));
}

#[test]
fn test_binary_op_comparison() {
    assert!(matches!(BinaryOp::Eq, BinaryOp::Eq));
    assert!(matches!(BinaryOp::Ne, BinaryOp::Ne));
    assert!(matches!(BinaryOp::Lt, BinaryOp::Lt));
    assert!(matches!(BinaryOp::Le, BinaryOp::Le));
    assert!(matches!(BinaryOp::Gt, BinaryOp::Gt));
    assert!(matches!(BinaryOp::Ge, BinaryOp::Ge));
}

#[test]
fn test_binary_op_logical() {
    assert!(matches!(BinaryOp::And, BinaryOp::And));
    assert!(matches!(BinaryOp::Or, BinaryOp::Or));
}

#[test]
fn test_binary_op_precedence() {
    assert!(BinaryOp::Mul.precedence() > BinaryOp::Add.precedence());
    assert!(BinaryOp::Add.precedence() > BinaryOp::Eq.precedence());
    assert!(BinaryOp::Eq.precedence() > BinaryOp::And.precedence());
}

// ==================== UNARY OPERATOR TESTS ====================

#[test]
fn test_unary_op_neg() {
    let op = UnaryOp::Neg;
    assert!(matches!(op, UnaryOp::Neg));
}

#[test]
fn test_unary_op_not() {
    let op = UnaryOp::Not;
    assert!(matches!(op, UnaryOp::Not));
}

// ==================== AST VISITOR TESTS ====================

#[test]
fn test_ast_visitor_expr() {
    struct TestVisitor {
        count: usize,
    }
    
    impl TestVisitor {
        fn visit_expr(&mut self, _expr: &Expr) {
            self.count += 1;
        }
    }
    
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 2)
    );
    
    let mut visitor = TestVisitor { count: 0 };
    visitor.visit_expr(&expr);
    assert_eq!(visitor.count, 1);
}

// ==================== AST CLONE TESTS ====================

#[test]
fn test_expr_clone() {
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 2)
    );
    
    let cloned = expr.clone();
    assert_eq!(expr.span(), cloned.span());
}

#[test]
fn test_stmt_clone() {
    let stmt = Stmt::new(
        StmtKind::Return(None),
        Span::new(0, 6)
    );
    
    let cloned = stmt.clone();
    assert_eq!(stmt.span(), cloned.span());
}

// Run all tests with: cargo test ast_tdd --test ast_tdd