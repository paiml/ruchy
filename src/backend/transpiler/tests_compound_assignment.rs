// EXTREME TDD: Compound Assignment Tests for Transpiler Module
// These tests increase coverage of the transpiler module

#[cfg(test)]
mod transpiler_compound_assignment_tests {
    use crate::backend::transpiler::Transpiler;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
    use crate::frontend::parser::Parser;

    fn make_ident(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), Default::default())
    }

    fn make_literal(n: i32) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(i64::from(n))),
            Default::default(),
        )
    }

    #[test]
    fn test_bitwise_and_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::BitwiseAnd,
                value: Box::new(make_literal(3)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("&="));
    }

    #[test]
    fn test_bitwise_or_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::BitwiseOr,
                value: Box::new(make_literal(3)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("|="));
    }

    #[test]
    fn test_bitwise_xor_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::BitwiseXor,
                value: Box::new(make_literal(3)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("^="));
    }

    #[test]
    fn test_left_shift_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::LeftShift,
                value: Box::new(make_literal(2)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains("<<="));
    }

    #[test]
    fn test_right_shift_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::RightShift,
                value: Box::new(make_literal(2)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let tokens = result.unwrap();
        let code = tokens.to_string();
        assert!(code.contains(">>="));
    }

    #[test]
    fn test_add_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("count")),
                op: BinaryOp::Add,
                value: Box::new(make_literal(5)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("+="));
    }

    #[test]
    fn test_subtract_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("count")),
                op: BinaryOp::Subtract,
                value: Box::new(make_literal(3)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("-="));
    }

    #[test]
    fn test_multiply_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("value")),
                op: BinaryOp::Multiply,
                value: Box::new(make_literal(10)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("*="));
    }

    #[test]
    fn test_divide_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("total")),
                op: BinaryOp::Divide,
                value: Box::new(make_literal(2)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("/="));
    }

    #[test]
    fn test_modulo_compound() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("remainder")),
                op: BinaryOp::Modulo,
                value: Box::new(make_literal(7)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("%="));
    }

    #[test]
    fn test_invalid_compound_operator() {
        let transpiler = Transpiler::new();
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(make_ident("x")),
                op: BinaryOp::And, // Logical AND, not valid for compound assignment
                value: Box::new(make_literal(3)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_compound_with_complex_target() {
        let transpiler = Transpiler::new();
        // Test with array index as target: arr[0] += 5
        let target = Expr::new(
            ExprKind::IndexAccess {
                object: Box::new(make_ident("arr")),
                index: Box::new(make_literal(0)),
            },
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(target),
                op: BinaryOp::Add,
                value: Box::new(make_literal(5)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compound_with_field_access() {
        let transpiler = Transpiler::new();
        // Test with field access as target: obj.field += 10
        let target = Expr::new(
            ExprKind::FieldAccess {
                object: Box::new(make_ident("obj")),
                field: "field".to_string(),
            },
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(target),
                op: BinaryOp::Add,
                value: Box::new(make_literal(10)),
            },
            Default::default(),
        );
        let result = transpiler.transpile_expr(&expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_and_transpile_compound() {
        let mut parser = Parser::new("x += 5");
        let ast = parser.parse();
        assert!(ast.is_ok());

        let transpiler = Transpiler::new();
        let result = transpiler.transpile_expr(&ast.unwrap());
        assert!(result.is_ok());
        let code = result.unwrap().to_string();
        assert!(code.contains("+="));
    }

    #[test]
    fn test_multiple_compound_assignments() {
        let code = "x += 1; y -= 2; z *= 3";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert!(ast.is_ok());
    }
}
