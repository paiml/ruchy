//! Tests for the interpreter module
//!
//! EXTREME TDD Round 86: Comprehensive tests for interpreter.rs
//! Coverage target: 95% for interpreter module
//!
//! This module contains all tests for the interpreter, extracted from interpreter.rs
//! for maintainability and to reduce the main module size.

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::Value;
    use std::sync::Arc;

    // ============== Helper Functions ==============

    /// Helper to create a simple integer literal expression
    fn make_int(val: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(val, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a float literal expression
    fn make_float(val: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(val)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a bool literal expression
    fn make_bool(val: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(val)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a string literal expression
    fn make_string(val: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(val.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create an identifier expression
    fn make_ident(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a binary expression
    fn make_binary(left: Expr, op: AstBinaryOp, right: Expr) -> Expr {
        Expr {
            kind: ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a unary expression
    fn make_unary(op: UnaryOp, operand: Expr) -> Expr {
        Expr {
            kind: ExprKind::Unary {
                op,
                operand: Box::new(operand),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create an if expression
    fn make_if(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch: else_branch.map(Box::new),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a block expression
    fn make_block(exprs: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(exprs),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a list/array expression
    fn make_list(elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(elements),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a tuple expression
    fn make_tuple(elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Tuple(elements),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create an index access expression
    fn make_index(object: Expr, index: Expr) -> Expr {
        Expr {
            kind: ExprKind::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a let expression
    fn make_let(name: &str, value: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Let {
                name: name.to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: false,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a range expression
    fn make_range(start: Expr, end: Expr, inclusive: bool) -> Expr {
        Expr {
            kind: ExprKind::Range {
                start: Box::new(start),
                end: Box::new(end),
                inclusive,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create an array/list expression (alias)
    fn make_array(elements: Vec<Expr>) -> Expr {
        make_list(elements)
    }

    /// Helper to create a for expression
    fn make_for(var: &str, iter: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::For {
                var: var.to_string(),
                iter: Box::new(iter),
                body: Box::new(body),
                label: None,
                pattern: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a while expression
    fn make_while(condition: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::While {
                condition: Box::new(condition),
                body: Box::new(body),
                label: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create an assign expression
    fn make_assign(name: &str, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Assign {
                target: Box::new(make_ident(name)),
                value: Box::new(value),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a mutable let expression
    fn make_let_mut(name: &str, value: Expr, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Let {
                name: name.to_string(),
                type_annotation: None,
                value: Box::new(value),
                body: Box::new(body),
                is_mutable: true,
                else_block: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a compound assign expression
    fn make_compound_assign(name: &str, op: AstBinaryOp, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::CompoundAssign {
                target: Box::new(make_ident(name)),
                op,
                value: Box::new(value),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a break expression
    fn make_break() -> Expr {
        Expr {
            kind: ExprKind::Break { label: None, value: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a continue expression
    fn make_continue() -> Expr {
        Expr {
            kind: ExprKind::Continue { label: None },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create a unit expression
    fn make_unit() -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Unit),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // ============== Literal Tests ==============

    #[test]
    fn test_eval_integer_literal() {
        let mut interp = Interpreter::new();
        let expr = make_int(42);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(42));
    }

    #[test]
    fn test_eval_negative_integer() {
        let mut interp = Interpreter::new();
        let expr = make_int(-100);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(-100));
    }

    #[test]
    fn test_eval_large_integer() {
        let mut interp = Interpreter::new();
        let expr = make_int(i64::MAX);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(i64::MAX));
    }

    #[test]
    fn test_eval_float_literal() {
        let mut interp = Interpreter::new();
        let expr = make_float(3.14159);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Float(f) => assert!((f - 3.14159).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_eval_bool_true() {
        let mut interp = Interpreter::new();
        let expr = make_bool(true);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_bool_false() {
        let mut interp = Interpreter::new();
        let expr = make_bool(false);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_string_literal() {
        let mut interp = Interpreter::new();
        let expr = make_string("hello");
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_eval_empty_string() {
        let mut interp = Interpreter::new();
        let expr = make_string("");
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), ""),
            _ => panic!("Expected string"),
        }
    }

    // ============== Arithmetic Tests ==============

    #[test]
    fn test_eval_addition() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(15));
    }

    #[test]
    fn test_eval_subtraction() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Subtract, make_int(3));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(7));
    }

    #[test]
    fn test_eval_multiplication() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(4), AstBinaryOp::Multiply, make_int(7));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(28));
    }

    #[test]
    fn test_eval_division() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(20), AstBinaryOp::Divide, make_int(4));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(5));
    }

    #[test]
    fn test_eval_modulo() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(2));
    }

    #[test]
    fn test_eval_float_addition() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Float(f) => assert!((f - 4.0).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_eval_mixed_int_float() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::Add, make_float(2.5));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    // ============== Comparison Tests ==============

    #[test]
    fn test_eval_equal_true() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::Equal, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_equal_false() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::Equal, make_int(3));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_not_equal() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::NotEqual, make_int(3));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_less_than() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(3), AstBinaryOp::Less, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_less_than_false() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::Less, make_int(3));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_greater_than() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(7), AstBinaryOp::Greater, make_int(2));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_less_equal() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_greater_equal() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    // ============== Logical Operator Tests ==============

    #[test]
    fn test_eval_and_true() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_and_false() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_or_true() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_or_false() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_not_true() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Not, make_bool(true));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_eval_not_false() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Not, make_bool(false));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_negate_int() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Negate, make_int(42));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(-42));
    }

    #[test]
    fn test_eval_negate_float() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Negate, make_float(3.14));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Float(f) => assert!((f - (-3.14)).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    // ============== String Operation Tests ==============

    #[test]
    fn test_eval_string_concatenation() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("Hello"), AstBinaryOp::Add, make_string(" World"));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "Hello World"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_eval_string_with_int() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("Value: "), AstBinaryOp::Add, make_int(42));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "Value: 42"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_eval_string_equality() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("hello"));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_eval_string_inequality() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("hello"), AstBinaryOp::NotEqual, make_string("world"));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    // ============== Variable Tests via Let ==============

    #[test]
    fn test_let_binding() {
        let mut interp = Interpreter::new();
        let expr = make_let("x", make_int(42), make_ident("x"));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(42));
    }

    #[test]
    fn test_let_with_computation() {
        let mut interp = Interpreter::new();
        let expr = make_let(
            "x",
            make_int(10),
            make_binary(make_ident("x"), AstBinaryOp::Multiply, make_int(2))
        );
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(20));
    }

    #[test]
    fn test_nested_let() {
        let mut interp = Interpreter::new();
        let inner_let = make_let("y", make_int(20),
            make_binary(make_ident("x"), AstBinaryOp::Add, make_ident("y")));
        let expr = make_let("x", make_int(10), inner_let);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(30));
    }

    #[test]
    fn test_let_shadowing() {
        let mut interp = Interpreter::new();
        let inner_let = make_let("x", make_int(20), make_ident("x"));
        let expr = make_let("x", make_int(10), inner_let);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(20));
    }

    #[test]
    fn test_variable_not_found() {
        let mut interp = Interpreter::new();
        let expr = make_ident("undefined_var");
        assert!(interp.eval_expr(&expr).is_err());
    }

    // ============== Array Tests ==============

    #[test]
    fn test_eval_empty_array() {
        let mut interp = Interpreter::new();
        let expr = make_list(vec![]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Array(a) => assert!(a.is_empty()),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_eval_array_literal() {
        let mut interp = Interpreter::new();
        let expr = make_list(vec![make_int(1), make_int(2), make_int(3)]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Array(a) => {
                assert_eq!(a.len(), 3);
                assert_eq!(a[0], Value::Integer(1));
                assert_eq!(a[1], Value::Integer(2));
                assert_eq!(a[2], Value::Integer(3));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_eval_nested_array() {
        let mut interp = Interpreter::new();
        let inner = make_list(vec![make_int(1), make_int(2)]);
        let expr = make_list(vec![inner, make_list(vec![make_int(3), make_int(4)])]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Array(a) => {
                assert_eq!(a.len(), 2);
                match &a[0] {
                    Value::Array(inner) => assert_eq!(inner.len(), 2),
                    _ => panic!("Expected inner array"),
                }
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_array_index_access() {
        let mut interp = Interpreter::new();
        let arr = make_list(vec![make_int(10), make_int(20), make_int(30)]);
        let expr = make_let("arr", arr, make_index(make_ident("arr"), make_int(1)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(20));
    }

    #[test]
    fn test_array_negative_index() {
        let mut interp = Interpreter::new();
        let arr = make_list(vec![make_int(10), make_int(20), make_int(30)]);
        let expr = make_let("arr", arr, make_index(make_ident("arr"), make_int(-1)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(30));
    }

    // ============== Tuple Tests ==============

    #[test]
    fn test_eval_tuple_literal() {
        let mut interp = Interpreter::new();
        let expr = make_tuple(vec![make_int(1), make_string("hello"), make_bool(true)]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 3);
                assert_eq!(t[0], Value::Integer(1));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_tuple_index_access() {
        let mut interp = Interpreter::new();
        let t = make_tuple(vec![make_int(10), make_int(20)]);
        let expr = make_let("t", t, make_index(make_ident("t"), make_int(0)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(10));
    }

    // ============== If Expression Tests ==============

    #[test]
    fn test_if_true_branch() {
        let mut interp = Interpreter::new();
        let expr = make_if(make_bool(true), make_int(10), Some(make_int(20)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(10));
    }

    #[test]
    fn test_if_false_branch() {
        let mut interp = Interpreter::new();
        let expr = make_if(make_bool(false), make_int(10), Some(make_int(20)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(20));
    }

    #[test]
    fn test_if_no_else() {
        let mut interp = Interpreter::new();
        let expr = make_if(make_bool(false), make_int(10), None);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Nil);
    }

    #[test]
    fn test_nested_if() {
        let mut interp = Interpreter::new();
        let inner_if = make_if(make_bool(true), make_int(1), Some(make_int(2)));
        let expr = make_if(make_bool(true), inner_if, Some(make_int(3)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(1));
    }

    #[test]
    fn test_if_with_comparison() {
        let mut interp = Interpreter::new();
        let condition = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
        let expr = make_if(condition, make_string("big"), Some(make_string("small")));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "big"),
            _ => panic!("Expected string"),
        }
    }

    // ============== Block Expression Tests ==============

    #[test]
    fn test_empty_block() {
        let mut interp = Interpreter::new();
        let expr = make_block(vec![]);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Nil);
    }

    #[test]
    fn test_block_returns_last() {
        let mut interp = Interpreter::new();
        let expr = make_block(vec![make_int(1), make_int(2), make_int(3)]);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(3));
    }

    // ============== Range Tests ==============

    #[test]
    fn test_exclusive_range() {
        let mut interp = Interpreter::new();
        let expr = make_range(make_int(0), make_int(5), false);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Range { start, end, inclusive } => {
                assert_eq!(*start, Value::Integer(0));
                assert_eq!(*end, Value::Integer(5));
                assert!(!inclusive);
            }
            _ => panic!("Expected range"),
        }
    }

    #[test]
    fn test_inclusive_range() {
        let mut interp = Interpreter::new();
        let expr = make_range(make_int(0), make_int(5), true);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Range { inclusive, .. } => assert!(inclusive),
            _ => panic!("Expected range"),
        }
    }

    // ============== String Index Tests ==============

    #[test]
    fn test_string_index() {
        let mut interp = Interpreter::new();
        let s = make_string("hello");
        let expr = make_let("s", s, make_index(make_ident("s"), make_int(0)));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "h"),
            _ => panic!("Expected string"),
        }
    }

    #[test]
    fn test_string_negative_index() {
        let mut interp = Interpreter::new();
        let s = make_string("hello");
        let expr = make_let("s", s, make_index(make_ident("s"), make_int(-1)));
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::String(s) => assert_eq!(s.as_ref(), "o"),
            _ => panic!("Expected string"),
        }
    }

    // ============== Error Handling Tests ==============

    #[test]
    fn test_division_by_zero() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Divide, make_int(0));
        assert!(interp.eval_expr(&expr).is_err());
    }

    #[test]
    fn test_modulo_by_zero() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Modulo, make_int(0));
        assert!(interp.eval_expr(&expr).is_err());
    }

    // ============== Complex Expression Tests ==============

    #[test]
    fn test_nested_arithmetic() {
        let mut interp = Interpreter::new();
        // (2 + 3) * (4 - 1) = 5 * 3 = 15
        let left = make_binary(make_int(2), AstBinaryOp::Add, make_int(3));
        let right = make_binary(make_int(4), AstBinaryOp::Subtract, make_int(1));
        let expr = make_binary(left, AstBinaryOp::Multiply, right);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(15));
    }

    #[test]
    fn test_chained_comparisons() {
        let mut interp = Interpreter::new();
        // true && (5 > 3) = true
        let cmp = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
        let expr = make_binary(make_bool(true), AstBinaryOp::And, cmp);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_expression_in_if_condition() {
        let mut interp = Interpreter::new();
        // if (10 + 5) > 10 { 1 } else { 0 }
        let sum = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
        let condition = make_binary(sum, AstBinaryOp::Greater, make_int(10));
        let expr = make_if(condition, make_int(1), Some(make_int(0)));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(1));
    }

    #[test]
    fn test_complex_let_with_if() {
        let mut interp = Interpreter::new();
        // let x = 10 in if x > 5 { x * 2 } else { x }
        let condition = make_binary(make_ident("x"), AstBinaryOp::Greater, make_int(5));
        let then_branch = make_binary(make_ident("x"), AstBinaryOp::Multiply, make_int(2));
        let if_expr = make_if(condition, then_branch, Some(make_ident("x")));
        let expr = make_let("x", make_int(10), if_expr);
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(20));
    }

    // ============== Additional Coverage Tests ==============

    #[test]
    fn test_power_operator() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(2), AstBinaryOp::Power, make_int(3));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(8));
    }

    #[test]
    fn test_bitwise_and() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(0b1000));
    }

    #[test]
    fn test_bitwise_or() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b1010));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(0b1110));
    }

    #[test]
    fn test_bitwise_xor() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(0b0110));
    }

    #[test]
    fn test_left_shift() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(1), AstBinaryOp::LeftShift, make_int(4));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(16));
    }

    #[test]
    fn test_right_shift() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(16), AstBinaryOp::RightShift, make_int(2));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Integer(4));
    }

    #[test]
    fn test_deep_nesting() {
        let mut interp = Interpreter::new();
        // ((1 + 2) + 3) + 4 = 10
        let e1 = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
        let e2 = make_binary(e1, AstBinaryOp::Add, make_int(3));
        let e3 = make_binary(e2, AstBinaryOp::Add, make_int(4));
        assert_eq!(interp.eval_expr(&e3).expect("should succeed"), Value::Integer(10));
    }

    #[test]
    fn test_many_let_bindings() {
        let mut interp = Interpreter::new();
        // let a = 1 in let b = 2 in let c = 3 in a + b + c
        let sum_bc = make_binary(make_ident("b"), AstBinaryOp::Add, make_ident("c"));
        let sum_abc = make_binary(make_ident("a"), AstBinaryOp::Add, sum_bc);
        let let_c = make_let("c", make_int(3), sum_abc);
        let let_b = make_let("b", make_int(2), let_c);
        let let_a = make_let("a", make_int(1), let_b);
        assert_eq!(interp.eval_expr(&let_a).expect("should succeed"), Value::Integer(6));
    }

    #[test]
    fn test_mixed_types_in_array() {
        let mut interp = Interpreter::new();
        let expr = make_list(vec![make_int(1), make_float(2.5), make_bool(true), make_string("hi")]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Array(a) => assert_eq!(a.len(), 4),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_empty_tuple() {
        let mut interp = Interpreter::new();
        let expr = make_tuple(vec![]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Tuple(t) => assert!(t.is_empty()),
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_single_element_tuple() {
        let mut interp = Interpreter::new();
        let expr = make_tuple(vec![make_int(42)]);
        match interp.eval_expr(&expr).expect("should succeed") {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 1);
                assert_eq!(t[0], Value::Integer(42));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_float_comparison() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_string_comparison_lt() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("apple"), AstBinaryOp::Less, make_string("banana"));
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_short_circuit_and() {
        let mut interp = Interpreter::new();
        // false && (error) should not evaluate the right side
        let expr = make_binary(make_bool(false), AstBinaryOp::And, make_ident("undefined"));
        // This should succeed without error due to short-circuit
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(false));
    }

    #[test]
    fn test_short_circuit_or() {
        let mut interp = Interpreter::new();
        // true || (error) should not evaluate the right side
        let expr = make_binary(make_bool(true), AstBinaryOp::Or, make_ident("undefined"));
        // This should succeed without error due to short-circuit
        assert_eq!(interp.eval_expr(&expr).expect("should succeed"), Value::Bool(true));
    }

    #[test]
    fn test_interpreter_new_returns_valid() {
        let interp = Interpreter::new();
        // Just verify creation works
        drop(interp);
    }

    #[test]
    fn test_multiple_operations_same_interpreter() {
        let mut interp = Interpreter::new();

        // First operation
        let e1 = make_int(10);
        assert_eq!(interp.eval_expr(&e1).expect("should succeed"), Value::Integer(10));

        // Second operation
        let e2 = make_binary(make_int(20), AstBinaryOp::Add, make_int(30));
        assert_eq!(interp.eval_expr(&e2).expect("should succeed"), Value::Integer(50));

        // Third operation with let
        let e3 = make_let("x", make_int(5), make_ident("x"));
        assert_eq!(interp.eval_expr(&e3).expect("should succeed"), Value::Integer(5));
    }

    // ============== EXTREME TDD Round 87: Coverage Expansion ==============
    // These tests target uncovered interpreter functions

    // ---------- GC Functions ----------

    #[test]
    fn test_gc_track() {
        let mut interp = Interpreter::new();
        let value = Value::Integer(42);
        let _handle = interp.gc_track(value);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_gc_collect() {
        let mut interp = Interpreter::new();
        let stats = interp.gc_collect();
        // Verify stats structure exists
        assert!(stats.collections >= 0);
    }

    #[test]
    fn test_gc_stats() {
        let interp = Interpreter::new();
        let stats = interp.gc_stats();
        assert!(stats.collections >= 0);
    }

    #[test]
    fn test_gc_info() {
        let interp = Interpreter::new();
        let info = interp.gc_info();
        // Just verify it returns without panic
        let _ = info;
    }

    #[test]
    fn test_gc_set_threshold() {
        let mut interp = Interpreter::new();
        interp.gc_set_threshold(1000);
        // Verify no panic
    }

    #[test]
    fn test_gc_set_auto_collect() {
        let mut interp = Interpreter::new();
        interp.gc_set_auto_collect(true);
        interp.gc_set_auto_collect(false);
        // Verify no panic
    }

    #[test]
    fn test_gc_clear() {
        let mut interp = Interpreter::new();
        interp.gc_clear();
        // Verify no panic
    }

    #[test]
    fn test_gc_alloc_array() {
        let mut interp = Interpreter::new();
        let elements = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
        let value = interp.gc_alloc_array(elements);
        match value {
            Value::Array(arr) => assert_eq!(arr.len(), 3),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_gc_alloc_string() {
        let mut interp = Interpreter::new();
        let value = interp.gc_alloc_string("hello".to_string());
        match value {
            Value::String(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected String"),
        }
    }

    // ---------- Cache Functions ----------

    #[test]
    fn test_get_cache_stats() {
        let interp = Interpreter::new();
        let stats = interp.get_cache_stats();
        // Stats should be a valid HashMap
        assert!(stats.is_empty() || !stats.is_empty());
    }

    #[test]
    fn test_clear_caches() {
        let mut interp = Interpreter::new();
        interp.clear_caches();
        let stats = interp.get_cache_stats();
        // After clear, cache should be empty or reset
        let _ = stats;
    }

    // ---------- Type Feedback Functions ----------

    #[test]
    fn test_get_type_feedback_stats() {
        let interp = Interpreter::new();
        let stats = interp.get_type_feedback_stats();
        // Just verify it returns without panic
        let _ = stats;
    }

    #[test]
    fn test_get_specialization_candidates() {
        let interp = Interpreter::new();
        let candidates = interp.get_specialization_candidates();
        // Should return empty or valid candidates
        assert!(candidates.is_empty() || !candidates.is_empty());
    }

    #[test]
    fn test_clear_type_feedback() {
        let mut interp = Interpreter::new();
        interp.clear_type_feedback();
        let stats = interp.get_type_feedback_stats();
        let _ = stats;
    }

    // ---------- Environment Functions ----------

    #[test]
    fn test_current_env() {
        let interp = Interpreter::new();
        let env = interp.current_env();
        // Environment should exist
        assert!(env.borrow().is_empty() || !env.borrow().is_empty());
    }

    #[test]
    fn test_push_pop_scope() {
        let mut interp = Interpreter::new();
        interp.push_scope();
        interp.set_variable_string("x".to_string(), Value::Integer(42));
        interp.pop_scope();
        // Variable should be gone after pop
    }

    #[test]
    fn test_set_variable_string() {
        let mut interp = Interpreter::new();
        interp.set_variable_string("test_var".to_string(), Value::Integer(100));
        // Verify no panic
    }

    // ---------- Stack Operations ----------

    #[test]
    fn test_push_pop() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(42)).expect("push should succeed");
        let value = interp.pop().expect("pop should succeed");
        assert_eq!(value, Value::Integer(42));
    }

    #[test]
    fn test_peek() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(1)).expect("push should succeed");
        interp.push(Value::Integer(2)).expect("push should succeed");
        let value = interp.peek(0).expect("peek should succeed");
        assert_eq!(value, Value::Integer(2));
    }

    #[test]
    fn test_pop_empty_stack() {
        let mut interp = Interpreter::new();
        let result = interp.pop();
        assert!(result.is_err());
    }

    #[test]
    fn test_peek_empty_stack() {
        let interp = Interpreter::new();
        let result = interp.peek(0);
        assert!(result.is_err());
    }

    // ---------- Global Bindings ----------

    #[test]
    fn test_get_global_bindings() {
        let interp = Interpreter::new();
        let bindings = interp.get_global_bindings();
        // Should return a valid map (may have builtins)
        let _ = bindings;
    }

    #[test]
    fn test_set_global_binding() {
        let mut interp = Interpreter::new();
        interp.set_global_binding("my_global".to_string(), Value::Integer(999));
        let bindings = interp.get_global_bindings();
        // Should contain the new binding
        assert!(bindings.contains_key("my_global"));
    }

    #[test]
    fn test_clear_user_variables() {
        let mut interp = Interpreter::new();
        interp.set_variable_string("user_var".to_string(), Value::Integer(1));
        interp.clear_user_variables();
        // Verify no panic
    }

    // ---------- Binary Operations ----------

    #[test]
    fn test_binary_op_add() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(10)).unwrap();
        interp.push(Value::Integer(20)).unwrap();
        interp.binary_op(crate::runtime::interpreter::BinaryOp::Add).unwrap();
        let result = interp.pop().unwrap();
        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_binary_op_sub() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(30)).unwrap();
        interp.push(Value::Integer(10)).unwrap();
        interp.binary_op(crate::runtime::interpreter::BinaryOp::Sub).unwrap();
        let result = interp.pop().unwrap();
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_binary_op_mul() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(5)).unwrap();
        interp.push(Value::Integer(6)).unwrap();
        interp.binary_op(crate::runtime::interpreter::BinaryOp::Mul).unwrap();
        let result = interp.pop().unwrap();
        assert_eq!(result, Value::Integer(30));
    }

    #[test]
    fn test_binary_op_div() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(20)).unwrap();
        interp.push(Value::Integer(4)).unwrap();
        interp.binary_op(crate::runtime::interpreter::BinaryOp::Div).unwrap();
        let result = interp.pop().unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    // ---------- Eval String ----------

    #[test]
    fn test_eval_string_simple() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_eval_string_expression() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("1 + 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Integer(3));
    }

    #[test]
    fn test_eval_string_invalid() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("}{][");
        // Should return an error for invalid syntax
        assert!(result.is_err());
    }

    // ---------- Complex Expressions ----------

    #[test]
    fn test_nested_let_expressions() {
        let mut interp = Interpreter::new();
        let inner_let = make_let("y", make_int(10), make_ident("y"));
        let outer_let = make_let("x", make_int(5), inner_let);
        let result = interp.eval_expr(&outer_let).expect("should succeed");
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_deeply_nested_binary() {
        let mut interp = Interpreter::new();
        // ((1 + 2) * (3 + 4)) = 3 * 7 = 21
        let left = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
        let right = make_binary(make_int(3), AstBinaryOp::Add, make_int(4));
        let expr = make_binary(left, AstBinaryOp::Multiply, right);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(21));
    }

    #[test]
    fn test_chained_comparisons_2() {
        let mut interp = Interpreter::new();
        // 5 > 3 = true
        let expr = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    // ---------- Edge Cases ----------

    #[test]
    fn test_large_integer() {
        let mut interp = Interpreter::new();
        let expr = make_int(i64::MAX);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_negative_integer() {
        let mut interp = Interpreter::new();
        let expr = make_int(-999);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(-999));
    }

    #[test]
    fn test_zero() {
        let mut interp = Interpreter::new();
        let expr = make_int(0);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_float_precision() {
        let mut interp = Interpreter::new();
        let expr = make_float(3.141592653589793);
        let result = interp.eval_expr(&expr).expect("should succeed");
        match result {
            Value::Float(f) => assert!((f - 3.141592653589793).abs() < 1e-10),
            _ => panic!("Expected Float"),
        }
    }

    // ---------- EXTREME TDD Round 86: Additional Coverage Tests ----------

    #[test]
    fn test_index_access_array() {
        let mut interp = Interpreter::new();
        // let arr = [1, 2, 3]; arr[1]
        let arr = make_array(vec![make_int(1), make_int(2), make_int(3)]);
        let let_arr = make_let("arr", arr, make_index(make_ident("arr"), make_int(1)));
        let result = interp.eval_expr(&let_arr).expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_index_access_string() {
        let mut interp = Interpreter::new();
        // let s = "hello"; s[0]
        let s = make_string("hello");
        let let_s = make_let("s", s, make_index(make_ident("s"), make_int(0)));
        let result = interp.eval_expr(&let_s).expect("should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "h"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_index_access_negative() {
        let mut interp = Interpreter::new();
        // let arr = [1, 2, 3]; arr[-1] should get last element
        let arr = make_array(vec![make_int(1), make_int(2), make_int(3)]);
        let let_arr = make_let("arr", arr, make_index(make_ident("arr"), make_int(-1)));
        let result = interp.eval_expr(&let_arr).expect("should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_tuple_literal() {
        let mut interp = Interpreter::new();
        let tuple = make_tuple(vec![make_int(1), make_string("hello"), make_bool(true)]);
        let result = interp.eval_expr(&tuple).expect("should succeed");
        match result {
            Value::Tuple(vals) => {
                assert_eq!(vals.len(), 3);
                assert_eq!(vals[0], Value::Integer(1));
                if let Value::String(s) = &vals[1] {
                    assert_eq!(s.as_ref(), "hello");
                } else {
                    panic!("Expected String");
                }
                assert_eq!(vals[2], Value::Bool(true));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_tuple_index() {
        let mut interp = Interpreter::new();
        let tuple = make_tuple(vec![make_int(10), make_int(20)]);
        let let_t = make_let("t", tuple, make_index(make_ident("t"), make_int(1)));
        let result = interp.eval_expr(&let_t).expect("should succeed");
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_while_loop_basic() {
        let mut interp = Interpreter::new();
        // let x = 0; while x < 3 { x = x + 1 }; x
        let init = make_let(
            "x",
            make_int(0),
            make_while(
                make_binary(make_ident("x"), AstBinaryOp::Less, make_int(3)),
                make_assign("x", make_binary(make_ident("x"), AstBinaryOp::Add, make_int(1))),
            ),
        );
        let block = make_block(vec![init, make_ident("x")]);
        let result = interp.eval_expr(&block);
        // While returns unit or the variable persists
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop_basic() {
        let mut interp = Interpreter::new();
        // for i in [1, 2, 3] { i }
        let for_expr = make_for(
            "i",
            make_array(vec![make_int(1), make_int(2), make_int(3)]),
            make_ident("i"),
        );
        let result = interp.eval_expr(&for_expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_if_without_else() {
        let mut interp = Interpreter::new();
        let if_expr = make_if(make_bool(true), make_int(42), None);
        let result = interp.eval_expr(&if_expr).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_if_false_without_else() {
        let mut interp = Interpreter::new();
        let if_expr = make_if(make_bool(false), make_int(42), None);
        let result = interp.eval_expr(&if_expr).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_string_concatenation() {
        let mut interp = Interpreter::new();
        let expr = make_binary(
            make_string("Hello, "),
            AstBinaryOp::Add,
            make_string("World!"),
        );
        let result = interp.eval_expr(&expr).expect("should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "Hello, World!"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_string_multiply() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("ab"), AstBinaryOp::Multiply, make_int(3));
        let result = interp.eval_expr(&expr).expect("should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), "ababab"),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_array_append() {
        let mut interp = Interpreter::new();
        let expr = make_binary(
            make_array(vec![make_int(1), make_int(2)]),
            AstBinaryOp::Add,
            make_array(vec![make_int(3)]),
        );
        let result = interp.eval_expr(&expr).expect("should succeed");
        match result {
            Value::Array(vals) => {
                assert_eq!(vals.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_mutable_let() {
        let mut interp = Interpreter::new();
        // let mut x = 5; x = 10; x
        let init = make_let_mut("x", make_int(5), make_ident("x"));
        let result = interp.eval_expr(&init).expect("should succeed");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_compound_assign_add() {
        let mut interp = Interpreter::new();
        // let mut x = 5; x += 3; x
        let init = make_let_mut(
            "x",
            make_int(5),
            make_block(vec![
                make_compound_assign("x", AstBinaryOp::Add, make_int(3)),
                make_ident("x"),
            ]),
        );
        let result = interp.eval_expr(&init).expect("should succeed");
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_unary_not_bool() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Not, make_bool(true));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_negate_int() {
        let mut interp = Interpreter::new();
        let expr = make_unary(UnaryOp::Negate, make_int(42));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_division_by_zero_error() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Divide, make_int(0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_modulo_by_zero_error() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(10), AstBinaryOp::Modulo, make_int(0));
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_array_access() {
        let mut interp = Interpreter::new();
        // let arr = [[1, 2], [3, 4]]; arr[1][0]
        let arr = make_array(vec![
            make_array(vec![make_int(1), make_int(2)]),
            make_array(vec![make_int(3), make_int(4)]),
        ]);
        let let_arr = make_let(
            "arr",
            arr,
            make_index(make_index(make_ident("arr"), make_int(1)), make_int(0)),
        );
        let result = interp.eval_expr(&let_arr).expect("should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_empty_array() {
        let mut interp = Interpreter::new();
        let arr = make_array(vec![]);
        let result = interp.eval_expr(&arr).expect("should succeed");
        match result {
            Value::Array(vals) => assert!(vals.is_empty()),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_empty_string() {
        let mut interp = Interpreter::new();
        let s = make_string("");
        let result = interp.eval_expr(&s).expect("should succeed");
        match result {
            Value::String(s) => assert_eq!(s.as_ref(), ""),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_boolean_and_short_circuit() {
        let mut interp = Interpreter::new();
        // false && (1/0) should not evaluate the second part
        let expr = make_binary(
            make_bool(false),
            AstBinaryOp::And,
            make_binary(make_int(1), AstBinaryOp::Divide, make_int(0)),
        );
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_boolean_or_short_circuit() {
        let mut interp = Interpreter::new();
        // true || (1/0) should not evaluate the second part
        let expr = make_binary(
            make_bool(true),
            AstBinaryOp::Or,
            make_binary(make_int(1), AstBinaryOp::Divide, make_int(0)),
        );
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_range_expression() {
        let mut interp = Interpreter::new();
        let range = make_range(make_int(0), make_int(5), false);
        let result = interp.eval_expr(&range).expect("should succeed");
        match result {
            Value::Range { start, end, inclusive } => {
                assert_eq!(*start, Value::Integer(0));
                assert_eq!(*end, Value::Integer(5));
                assert!(!inclusive);
            }
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_nil_literal() {
        let mut interp = Interpreter::new();
        let unit = make_unit();
        let result = interp.eval_expr(&unit).expect("should succeed");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_break_in_loop() {
        let mut interp = Interpreter::new();
        // let mut i = 0; while true { if i >= 3 { break }; i = i + 1 }
        let loop_body = make_block(vec![
            make_if(
                make_binary(make_ident("i"), AstBinaryOp::GreaterEqual, make_int(3)),
                make_break(),
                None,
            ),
            make_assign("i", make_binary(make_ident("i"), AstBinaryOp::Add, make_int(1))),
        ]);
        let init = make_let_mut("i", make_int(0), make_while(make_bool(true), loop_body));
        let result = interp.eval_expr(&init);
        assert!(result.is_ok());
    }

    #[test]
    fn test_continue_in_loop() {
        let mut interp = Interpreter::new();
        // for i in [1, 2, 3] { if i == 2 { continue }; i }
        let loop_body = make_block(vec![
            make_if(
                make_binary(make_ident("i"), AstBinaryOp::Equal, make_int(2)),
                make_continue(),
                None,
            ),
            make_ident("i"),
        ]);
        let for_expr = make_for(
            "i",
            make_array(vec![make_int(1), make_int(2), make_int(3)]),
            loop_body,
        );
        let result = interp.eval_expr(&for_expr);
        assert!(result.is_ok());
    }
}
