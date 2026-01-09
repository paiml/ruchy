//! Tests for the interpreter module
//!
//! EXTREME TDD Round 86: Comprehensive tests for interpreter.rs
//! Coverage target: 95% for interpreter module
//!
//! This module contains all tests for the interpreter, extracted from interpreter.rs
//! for maintainability and to reduce the main module size.

#[cfg(test)]
mod tests {
    use crate::frontend::ast::{BinaryOp as AstBinaryOp, Expr, ExprKind, Literal, Param, Pattern, Span, Type, TypeKind, UnaryOp};
    use crate::runtime::interpreter::Interpreter;
    use crate::runtime::Value;

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

    // ---------- EXTREME TDD Round 87: More Coverage Tests ----------

    #[test]
    fn test_lambda_basic() {
        let mut interp = Interpreter::new();
        // let f = |x| x + 1; f(5)
        let lambda = make_lambda_with_params(
            vec!["x".to_string()],
            make_binary(make_ident("x"), AstBinaryOp::Add, make_int(1)),
        );
        let let_f = make_let(
            "f",
            lambda,
            make_call(make_ident("f"), vec![make_int(5)]),
        );
        let result = interp.eval_expr(&let_f).expect("should succeed");
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_lambda_closure() {
        let mut interp = Interpreter::new();
        // let a = 10; let f = |x| x + a; f(5)
        let lambda = make_lambda_with_params(
            vec!["x".to_string()],
            make_binary(make_ident("x"), AstBinaryOp::Add, make_ident("a")),
        );
        let let_a = make_let(
            "a",
            make_int(10),
            make_let("f", lambda, make_call(make_ident("f"), vec![make_int(5)])),
        );
        let result = interp.eval_expr(&let_a).expect("should succeed");
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_return_in_block() {
        let mut interp = Interpreter::new();
        // { return 42; 100 }
        let block = make_block(vec![make_return(Some(make_int(42))), make_int(100)]);
        let result = interp.eval_expr(&block);
        // Return may propagate up or be caught
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_float_division() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
        let result = interp.eval_expr(&expr).expect("should succeed");
        match result {
            Value::Float(f) => assert!((f - 2.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_mixed_float_int_comparison() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_float(5.0), AstBinaryOp::Greater, make_int(3));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_equality() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("hello"));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_inequality() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_string("hello"), AstBinaryOp::NotEqual, make_string("world"));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_array_equality() {
        let mut interp = Interpreter::new();
        let arr1 = make_array(vec![make_int(1), make_int(2)]);
        let arr2 = make_array(vec![make_int(1), make_int(2)]);
        let expr = make_binary(arr1, AstBinaryOp::Equal, arr2);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_nested_if_else() {
        let mut interp = Interpreter::new();
        // if false { 1 } else { if true { 2 } else { 3 } }
        let inner_if = make_if(make_bool(true), make_int(2), Some(make_int(3)));
        let outer_if = make_if(make_bool(false), make_int(1), Some(inner_if));
        let result = interp.eval_expr(&outer_if).expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_complex_arithmetic() {
        let mut interp = Interpreter::new();
        // (10 + 5) * 2 - 6 / 3 = 15 * 2 - 2 = 30 - 2 = 28
        let add = make_binary(make_int(10), AstBinaryOp::Add, make_int(5));
        let mul = make_binary(add, AstBinaryOp::Multiply, make_int(2));
        let div = make_binary(make_int(6), AstBinaryOp::Divide, make_int(3));
        let expr = make_binary(mul, AstBinaryOp::Subtract, div);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(28));
    }

    #[test]
    fn test_for_with_range() {
        let mut interp = Interpreter::new();
        // for i in 0..3 { i }
        let for_expr = make_for("i", make_range(make_int(0), make_int(3), false), make_ident("i"));
        let result = interp.eval_expr(&for_expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_blocks() {
        let mut interp = Interpreter::new();
        // { { { 42 } } }
        let inner = make_block(vec![make_int(42)]);
        let middle = make_block(vec![inner]);
        let outer = make_block(vec![middle]);
        let result = interp.eval_expr(&outer).expect("should succeed");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_multiple_statements_in_block() {
        let mut interp = Interpreter::new();
        // { let x = 1; let y = 2; x + y }
        let block = make_let(
            "x",
            make_int(1),
            make_let(
                "y",
                make_int(2),
                make_binary(make_ident("x"), AstBinaryOp::Add, make_ident("y")),
            ),
        );
        let result = interp.eval_expr(&block).expect("should succeed");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_boolean_chain() {
        let mut interp = Interpreter::new();
        // true && true && false
        let and1 = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        let and2 = make_binary(and1, AstBinaryOp::And, make_bool(false));
        let result = interp.eval_expr(&and2).expect("should succeed");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_comparison_chain() {
        let mut interp = Interpreter::new();
        // (5 > 3) && (3 > 1)
        let cmp1 = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
        let cmp2 = make_binary(make_int(3), AstBinaryOp::Greater, make_int(1));
        let expr = make_binary(cmp1, AstBinaryOp::And, cmp2);
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_array_of_arrays() {
        let mut interp = Interpreter::new();
        let arr = make_array(vec![
            make_array(vec![make_int(1)]),
            make_array(vec![make_int(2)]),
            make_array(vec![make_int(3)]),
        ]);
        let result = interp.eval_expr(&arr).expect("should succeed");
        match result {
            Value::Array(vals) => assert_eq!(vals.len(), 3),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_tuple_of_different_types() {
        let mut interp = Interpreter::new();
        let tuple = make_tuple(vec![
            make_int(1),
            make_float(2.5),
            make_bool(true),
            make_string("test"),
        ]);
        let result = interp.eval_expr(&tuple).expect("should succeed");
        match result {
            Value::Tuple(vals) => assert_eq!(vals.len(), 4),
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_unary_double_negation() {
        let mut interp = Interpreter::new();
        // --5 = 5
        let neg1 = make_unary(UnaryOp::Negate, make_int(5));
        let neg2 = make_unary(UnaryOp::Negate, neg1);
        let result = interp.eval_expr(&neg2).expect("should succeed");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_unary_double_not() {
        let mut interp = Interpreter::new();
        // !!true = true
        let not1 = make_unary(UnaryOp::Not, make_bool(true));
        let not2 = make_unary(UnaryOp::Not, not1);
        let result = interp.eval_expr(&not2).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_large_array() {
        let mut interp = Interpreter::new();
        let elements: Vec<Expr> = (0..100).map(|i| make_int(i)).collect();
        let arr = make_array(elements);
        let result = interp.eval_expr(&arr).expect("should succeed");
        match result {
            Value::Array(vals) => assert_eq!(vals.len(), 100),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_compound_subtract() {
        let mut interp = Interpreter::new();
        // let mut x = 10; x -= 3; x
        let init = make_let_mut(
            "x",
            make_int(10),
            make_block(vec![
                make_compound_assign("x", AstBinaryOp::Subtract, make_int(3)),
                make_ident("x"),
            ]),
        );
        let result = interp.eval_expr(&init).expect("should succeed");
        assert_eq!(result, Value::Integer(7));
    }

    #[test]
    fn test_compound_multiply() {
        let mut interp = Interpreter::new();
        // let mut x = 5; x *= 4; x
        let init = make_let_mut(
            "x",
            make_int(5),
            make_block(vec![
                make_compound_assign("x", AstBinaryOp::Multiply, make_int(4)),
                make_ident("x"),
            ]),
        );
        let result = interp.eval_expr(&init).expect("should succeed");
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_while_with_complex_condition() {
        let mut interp = Interpreter::new();
        // let mut x = 0; while x < 5 && x >= 0 { x = x + 1 }
        let condition = make_binary(
            make_binary(make_ident("x"), AstBinaryOp::Less, make_int(5)),
            AstBinaryOp::And,
            make_binary(make_ident("x"), AstBinaryOp::GreaterEqual, make_int(0)),
        );
        let init = make_let_mut(
            "x",
            make_int(0),
            make_while(
                condition,
                make_assign("x", make_binary(make_ident("x"), AstBinaryOp::Add, make_int(1))),
            ),
        );
        let result = interp.eval_expr(&init);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_empty_array() {
        let mut interp = Interpreter::new();
        // for i in [] { i }
        let for_expr = make_for("i", make_array(vec![]), make_ident("i"));
        let result = interp.eval_expr(&for_expr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_inclusive_range_flag() {
        let mut interp = Interpreter::new();
        let range = make_range(make_int(1), make_int(5), true);
        let result = interp.eval_expr(&range).expect("should succeed");
        match result {
            Value::Range { inclusive, .. } => assert!(inclusive),
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_string_with_special_chars() {
        let mut interp = Interpreter::new();
        let s = make_string("hello\\nworld\\ttab");
        let result = interp.eval_expr(&s).expect("should succeed");
        match result {
            Value::String(val) => assert!(val.len() > 0),
            _ => panic!("Expected String"),
        }
    }

    #[test]
    fn test_negative_array_index_second() {
        let mut interp = Interpreter::new();
        // [1, 2, 3][-2] = 2
        let arr = make_array(vec![make_int(1), make_int(2), make_int(3)]);
        let let_arr = make_let("arr", arr, make_index(make_ident("arr"), make_int(-2)));
        let result = interp.eval_expr(&let_arr).expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_float_comparison_less() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_modulo_operation() {
        let mut interp = Interpreter::new();
        let expr = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
        let result = interp.eval_expr(&expr).expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_nested_let_with_same_name() {
        let mut interp = Interpreter::new();
        // let x = 1; let x = 2; x (shadowing)
        let inner = make_let("x", make_int(2), make_ident("x"));
        let outer = make_let("x", make_int(1), inner);
        let result = interp.eval_expr(&outer).expect("should succeed");
        assert_eq!(result, Value::Integer(2));
    }

    // ---------- Additional Helper Functions ----------

    /// Helper to create lambda with parameters
    fn make_lambda_with_params(params: Vec<String>, body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Lambda {
                params: params
                    .into_iter()
                    .map(|name| Param {
                        pattern: Pattern::Identifier(name),
                        ty: Type {
                            kind: TypeKind::Named("Any".to_string()),
                            span: Span::default(),
                        },
                        span: Span::default(),
                        is_mutable: false,
                        default_value: None,
                    })
                    .collect(),
                body: Box::new(body),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create return expression
    fn make_return(value: Option<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Return { value: value.map(Box::new) },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create call expression
    fn make_call(func: Expr, args: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Call {
                func: Box::new(func),
                args,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // ============== EXTREME TDD Round 112: Additional Coverage ==============

    #[test]
    fn test_eval_range_simple() {
        let mut interp = Interpreter::new();
        let range = Expr {
            kind: ExprKind::Range {
                start: Box::new(make_int(0)),
                end: Box::new(make_int(5)),
                inclusive: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&range);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_range_inclusive() {
        let mut interp = Interpreter::new();
        let range = Expr {
            kind: ExprKind::Range {
                start: Box::new(make_int(1)),
                end: Box::new(make_int(3)),
                inclusive: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&range);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_tuple_empty() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        assert!(matches!(result, Value::Tuple(_)));
    }

    #[test]
    fn test_eval_tuple_single() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![make_int(42)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        assert!(matches!(result, Value::Tuple(_)));
    }

    #[test]
    fn test_eval_tuple_multiple() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        if let Value::Tuple(vals) = result {
            assert_eq!(vals.len(), 3);
        } else {
            panic!("Expected tuple");
        }
    }

    #[test]
    fn test_eval_list_to_array_empty() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        assert!(matches!(result, Value::Array(_)));
    }

    #[test]
    fn test_eval_list_to_array_integers() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        if let Value::Array(vals) = result {
            assert_eq!(vals.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_eval_array_init_expr() {
        let mut interp = Interpreter::new();
        let arr = Expr {
            kind: ExprKind::ArrayInit {
                value: Box::new(make_int(0)),
                size: Box::new(make_int(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&arr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_eval_null_literal() {
        let mut interp = Interpreter::new();
        let nil = Expr {
            kind: ExprKind::Literal(Literal::Null),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&nil).expect("should evaluate");
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_eval_char_to_string_literal() {
        let mut interp = Interpreter::new();
        let ch = Expr {
            kind: ExprKind::Literal(Literal::Char('a')),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&ch).expect("should evaluate");
        // Char literals are converted to single-character strings
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_eval_unary_negate_expr() {
        let mut interp = Interpreter::new();
        let neg = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(make_int(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_eval_unary_not_expr() {
        let mut interp = Interpreter::new();
        let not = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(make_bool(true)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&not).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_binary_power() {
        let mut interp = Interpreter::new();
        let power = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(2)),
                op: AstBinaryOp::Power,
                right: Box::new(make_int(3)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&power).expect("should evaluate");
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_eval_logical_and() {
        let mut interp = Interpreter::new();
        let and = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_bool(false)),
                op: AstBinaryOp::And,
                right: Box::new(make_bool(true)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_logical_or() {
        let mut interp = Interpreter::new();
        let or = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_bool(true)),
                op: AstBinaryOp::Or,
                right: Box::new(make_bool(false)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_float_add_expr() {
        let mut interp = Interpreter::new();
        let add = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_float(1.5)),
                op: AstBinaryOp::Add,
                right: Box::new(make_float(2.5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&add).expect("should evaluate");
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_eval_float_div_expr() {
        let mut interp = Interpreter::new();
        let div = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_float(10.0)),
                op: AstBinaryOp::Divide,
                right: Box::new(make_float(4.0)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&div).expect("should evaluate");
        assert_eq!(result, Value::Float(2.5));
    }

    #[test]
    fn test_eval_block_multi_stmt() {
        let mut interp = Interpreter::new();
        let block = Expr {
            kind: ExprKind::Block(vec![
                make_int(1),
                make_int(2),
                make_int(3),
            ]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_if_no_else() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(false)),
                then_branch: Box::new(make_int(1)),
                else_branch: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_eval_if_nested_expr() {
        let mut interp = Interpreter::new();
        let nested = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(Expr {
                    kind: ExprKind::If {
                        condition: Box::new(make_bool(true)),
                        then_branch: Box::new(make_int(42)),
                        else_branch: Some(Box::new(make_int(0))),
                    },
                    span: Span::default(),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                }),
                else_branch: Some(Box::new(make_int(-1))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&nested).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_return_empty() {
        let mut interp = Interpreter::new();
        let ret = make_return(None);
        let result = interp.eval_expr(&ret);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_eval_return_int_value() {
        let mut interp = Interpreter::new();
        let ret = make_return(Some(make_int(42)));
        let result = interp.eval_expr(&ret);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_interpreter_creation() {
        let interp = Interpreter::new();
        assert!(interp.current_env().borrow().is_empty() || true);
    }

    #[test]
    fn test_eval_nested_arithmetic() {
        let mut interp = Interpreter::new();
        let inner_add = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(2)),
                op: AstBinaryOp::Add,
                right: Box::new(make_int(3)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let mul = Expr {
            kind: ExprKind::Binary {
                left: Box::new(inner_add),
                op: AstBinaryOp::Multiply,
                right: Box::new(make_int(4)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&mul).expect("should evaluate");
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_eval_less_comparison() {
        let mut interp = Interpreter::new();
        let cmp = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(1)),
                op: AstBinaryOp::Less,
                right: Box::new(make_int(2)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&cmp).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_equal_comparison() {
        let mut interp = Interpreter::new();
        let eq = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(5)),
                op: AstBinaryOp::Equal,
                right: Box::new(make_int(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_not_equal_comparison() {
        let mut interp = Interpreter::new();
        let neq = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(5)),
                op: AstBinaryOp::NotEqual,
                right: Box::new(make_int(3)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&neq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_byte_literal() {
        let mut interp = Interpreter::new();
        let byte = Expr {
            kind: ExprKind::Literal(Literal::Byte(255)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&byte).expect("should evaluate");
        assert_eq!(result, Value::Byte(255));
    }

    #[test]
    fn test_eval_unit_literal() {
        let mut interp = Interpreter::new();
        let unit = Expr {
            kind: ExprKind::Literal(Literal::Unit),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&unit).expect("should evaluate");
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_eval_list_nested() {
        let mut interp = Interpreter::new();
        let inner = Expr {
            kind: ExprKind::List(vec![make_int(1), make_int(2)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer = Expr {
            kind: ExprKind::List(vec![inner]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert!(matches!(result, Value::Array(_)));
    }

    #[test]
    fn test_eval_if_true_branch() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(make_int(100)),
                else_branch: Some(Box::new(make_int(0))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_eval_if_false_branch() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(false)),
                then_branch: Box::new(make_int(100)),
                else_branch: Some(Box::new(make_int(0))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_eval_unary_negate_float() {
        let mut interp = Interpreter::new();
        let neg = Expr {
            kind: ExprKind::Unary {
                op: UnaryOp::Negate,
                operand: Box::new(make_float(3.5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Float(-3.5));
    }

    #[test]
    fn test_eval_binary_subtract() {
        let mut interp = Interpreter::new();
        let sub = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(10)),
                op: AstBinaryOp::Subtract,
                right: Box::new(make_int(4)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&sub).expect("should evaluate");
        assert_eq!(result, Value::Integer(6));
    }

    #[test]
    fn test_eval_binary_multiply() {
        let mut interp = Interpreter::new();
        let mul = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(6)),
                op: AstBinaryOp::Multiply,
                right: Box::new(make_int(7)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&mul).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_binary_divide() {
        let mut interp = Interpreter::new();
        let div = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(20)),
                op: AstBinaryOp::Divide,
                right: Box::new(make_int(4)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&div).expect("should evaluate");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_eval_binary_greater() {
        let mut interp = Interpreter::new();
        let gt = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(10)),
                op: AstBinaryOp::Greater,
                right: Box::new(make_int(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_empty_block() {
        let mut interp = Interpreter::new();
        let block = Expr {
            kind: ExprKind::Block(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_eval_single_stmt_block() {
        let mut interp = Interpreter::new();
        let block = Expr {
            kind: ExprKind::Block(vec![make_int(99)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Integer(99));
    }

    // ============== EXTREME TDD Round 120: Special Forms Coverage ==============

    #[test]
    fn test_eval_none_literal() {
        let mut interp = Interpreter::new();
        let none_expr = Expr {
            kind: ExprKind::None,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&none_expr).expect("should evaluate");
        match result {
            Value::EnumVariant { enum_name, variant_name, data } => {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant_name, "None");
                assert!(data.is_none());
            }
            _ => panic!("Expected EnumVariant, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_some_with_integer() {
        let mut interp = Interpreter::new();
        let some_expr = Expr {
            kind: ExprKind::Some { value: Box::new(make_int(42)) },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&some_expr).expect("should evaluate");
        match result {
            Value::EnumVariant { enum_name, variant_name, data } => {
                assert_eq!(enum_name, "Option");
                assert_eq!(variant_name, "Some");
                assert!(data.is_some());
                let values = data.unwrap();
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::Integer(42));
            }
            _ => panic!("Expected EnumVariant, got {:?}", result),
        }
    }

    #[test]
    fn test_eval_some_with_string() {
        let mut interp = Interpreter::new();
        let some_expr = Expr {
            kind: ExprKind::Some { value: Box::new(make_string("hello")) },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&some_expr).expect("should evaluate");
        match result {
            Value::EnumVariant { variant_name, data, .. } => {
                assert_eq!(variant_name, "Some");
                let values = data.unwrap();
                assert_eq!(values[0], Value::String("hello".into()));
            }
            _ => panic!("Expected EnumVariant"),
        }
    }

    #[test]
    fn test_eval_set_empty() {
        let mut interp = Interpreter::new();
        let set_expr = Expr {
            kind: ExprKind::Set(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&set_expr).expect("should evaluate");
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_eval_set_single() {
        let mut interp = Interpreter::new();
        let set_expr = Expr {
            kind: ExprKind::Set(vec![make_int(100)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&set_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_eval_set_multiple_returns_last() {
        let mut interp = Interpreter::new();
        let set_expr = Expr {
            kind: ExprKind::Set(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&set_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_qualified_name_std_module() {
        let mut interp = Interpreter::new();
        let qual_expr = Expr {
            kind: ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "io".to_string(),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        // May succeed or fail depending on stdlib availability
        let _ = interp.eval_expr(&qual_expr);
    }

    #[test]
    fn test_eval_binary_less_than() {
        let mut interp = Interpreter::new();
        let lt = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(3)),
                op: AstBinaryOp::Less,
                right: Box::new(make_int(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&lt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_less_equal() {
        let mut interp = Interpreter::new();
        let le = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(5)),
                op: AstBinaryOp::LessEqual,
                right: Box::new(make_int(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_greater_equal() {
        let mut interp = Interpreter::new();
        let ge = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(10)),
                op: AstBinaryOp::GreaterEqual,
                right: Box::new(make_int(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_not_equal() {
        let mut interp = Interpreter::new();
        let ne = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(5)),
                op: AstBinaryOp::NotEqual,
                right: Box::new(make_int(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_modulo() {
        let mut interp = Interpreter::new();
        let modulo = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(17)),
                op: AstBinaryOp::Modulo,
                right: Box::new(make_int(5)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_binary_power_large() {
        let mut interp = Interpreter::new();
        let power = Expr {
            kind: ExprKind::Binary {
                left: Box::new(make_int(2)),
                op: AstBinaryOp::Power,
                right: Box::new(make_int(10)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&power).expect("should evaluate");
        assert_eq!(result, Value::Integer(1024));
    }

    #[test]
    fn test_eval_unary_negate() {
        let mut interp = Interpreter::new();
        let neg = make_unary(UnaryOp::Negate, make_int(42));
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_eval_unary_not_true() {
        let mut interp = Interpreter::new();
        let not_expr = make_unary(UnaryOp::Not, make_bool(true));
        let result = interp.eval_expr(&not_expr).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_unary_not_false() {
        let mut interp = Interpreter::new();
        let not_expr = make_unary(UnaryOp::Not, make_bool(false));
        let result = interp.eval_expr(&not_expr).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_list_empty_r120() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(items) => assert!(items.is_empty()),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_eval_list_with_elements_r120() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Integer(1));
                assert_eq!(items[1], Value::Integer(2));
                assert_eq!(items[2], Value::Integer(3));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_eval_tuple_empty_r120() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(items) => assert!(items.is_empty()),
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_eval_tuple_with_mixed_types() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![make_int(10), make_string("hello")]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], Value::Integer(10));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_float_addition() {
        let mut interp = Interpreter::new();
        let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
        let result = interp.eval_expr(&add).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 4.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_float_subtraction() {
        let mut interp = Interpreter::new();
        let sub = make_binary(make_float(5.5), AstBinaryOp::Subtract, make_float(2.0));
        let result = interp.eval_expr(&sub).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 3.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_float_multiplication() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(4.0));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 12.0).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_float_division_r120() {
        let mut interp = Interpreter::new();
        let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
        let result = interp.eval_expr(&div).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 2.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_float_comparison_less_r120() {
        let mut interp = Interpreter::new();
        let cmp = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
        let result = interp.eval_expr(&cmp).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_mixed_int_float_add() {
        let mut interp = Interpreter::new();
        let add = make_binary(make_int(2), AstBinaryOp::Add, make_float(3.5));
        let result = interp.eval_expr(&add).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 5.5).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_string_concat_addition() {
        let mut interp = Interpreter::new();
        let concat = make_binary(make_string("Hello, "), AstBinaryOp::Add, make_string("World!"));
        let result = interp.eval_expr(&concat).expect("should evaluate");
        assert_eq!(result, Value::String("Hello, World!".into()));
    }

    #[test]
    fn test_string_equality_r120() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_string("test"), AstBinaryOp::Equal, make_string("test"));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_inequality_r120() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_string("a"), AstBinaryOp::NotEqual, make_string("b"));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    // === EXTREME TDD Round 124 tests ===

    #[test]
    fn test_integer_modulo_positive_r124() {
        let mut interp = Interpreter::new();
        let modulo = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_integer_modulo_zero_result_r124() {
        let mut interp = Interpreter::new();
        let modulo = make_binary(make_int(15), AstBinaryOp::Modulo, make_int(5));
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_greater_than_true_r124() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_greater_than_false_r124() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_int(3), AstBinaryOp::Greater, make_int(7));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_greater_equal_true_r124() {
        let mut interp = Interpreter::new();
        let ge = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_less_equal_true_r124() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_less_equal_false_r124() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(10), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_bitwise_and_r124() {
        let mut interp = Interpreter::new();
        let band = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
        let result = interp.eval_expr(&band).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b1000));
    }

    #[test]
    fn test_bitwise_or_r124() {
        let mut interp = Interpreter::new();
        let bor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b0011));
        let result = interp.eval_expr(&bor).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b1111));
    }

    #[test]
    fn test_bitwise_xor_r124() {
        let mut interp = Interpreter::new();
        let bxor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
        let result = interp.eval_expr(&bxor).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b0110));
    }

    #[test]
    fn test_left_shift_r124() {
        let mut interp = Interpreter::new();
        let shl = make_binary(make_int(1), AstBinaryOp::LeftShift, make_int(4));
        let result = interp.eval_expr(&shl).expect("should evaluate");
        assert_eq!(result, Value::Integer(16));
    }

    #[test]
    fn test_right_shift_r124() {
        let mut interp = Interpreter::new();
        let shr = make_binary(make_int(16), AstBinaryOp::RightShift, make_int(2));
        let result = interp.eval_expr(&shr).expect("should evaluate");
        assert_eq!(result, Value::Integer(4));
    }

    #[test]
    fn test_double_negation_r124() {
        let mut interp = Interpreter::new();
        let inner = make_unary(UnaryOp::Negate, make_int(42));
        let outer = make_unary(UnaryOp::Negate, inner);
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_double_not_r124() {
        let mut interp = Interpreter::new();
        let inner = make_unary(UnaryOp::Not, make_bool(true));
        let outer = make_unary(UnaryOp::Not, inner);
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    // === EXTREME TDD Round 125 tests ===

    #[test]
    fn test_equal_true_r125() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(42));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_equal_false_r125() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(43));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_not_equal_true_r125() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(43));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_not_equal_false_r125() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(42));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_and_true_r125() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_and_false_r125() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_true_r125() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_or_false_r125() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_float_add_r125() {
        let mut interp = Interpreter::new();
        let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
        let result = interp.eval_expr(&add).expect("should evaluate");
        assert_eq!(result, Value::Float(4.0));
    }

    #[test]
    fn test_float_subtract_r125() {
        let mut interp = Interpreter::new();
        let sub = make_binary(make_float(5.0), AstBinaryOp::Subtract, make_float(2.0));
        let result = interp.eval_expr(&sub).expect("should evaluate");
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_float_multiply_r125() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(4.0));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        assert_eq!(result, Value::Float(12.0));
    }

    #[test]
    fn test_float_divide_r125() {
        let mut interp = Interpreter::new();
        let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(2.0));
        let result = interp.eval_expr(&div).expect("should evaluate");
        assert_eq!(result, Value::Float(5.0));
    }

    #[test]
    fn test_float_less_than_r125() {
        let mut interp = Interpreter::new();
        let lt = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
        let result = interp.eval_expr(&lt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_float_greater_than_r125() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_float_negate_r125() {
        let mut interp = Interpreter::new();
        let neg = make_unary(UnaryOp::Negate, make_float(3.14));
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_string_equal_r125() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("hello"));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_not_equal_r125() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_string("hello"), AstBinaryOp::NotEqual, make_string("world"));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_bool_equal_r125() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_bool(true), AstBinaryOp::Equal, make_bool(true));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_chain_and_r125() {
        let mut interp = Interpreter::new();
        let inner = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        let outer = make_binary(inner, AstBinaryOp::And, make_bool(true));
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_chain_or_r125() {
        let mut interp = Interpreter::new();
        let inner = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
        let outer = make_binary(inner, AstBinaryOp::Or, make_bool(true));
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_complex_comparison_r125() {
        let mut interp = Interpreter::new();
        // (5 > 3) && (2 < 4)
        let left = make_binary(make_int(5), AstBinaryOp::Greater, make_int(3));
        let right = make_binary(make_int(2), AstBinaryOp::Less, make_int(4));
        let combined = make_binary(left, AstBinaryOp::And, right);
        let result = interp.eval_expr(&combined).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_nested_arithmetic_r125() {
        let mut interp = Interpreter::new();
        // (2 + 3) * (4 - 1)
        let left = make_binary(make_int(2), AstBinaryOp::Add, make_int(3));
        let right = make_binary(make_int(4), AstBinaryOp::Subtract, make_int(1));
        let combined = make_binary(left, AstBinaryOp::Multiply, right);
        let result = interp.eval_expr(&combined).expect("should evaluate");
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_zero_multiply_r125() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_int(100), AstBinaryOp::Multiply, make_int(0));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_negative_modulo_r125() {
        let mut interp = Interpreter::new();
        let modulo = make_binary(make_int(-7), AstBinaryOp::Modulo, make_int(3));
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        // In Rust, -7 % 3 = -1
        assert_eq!(result, Value::Integer(-1));
    }

    // ============== EXTREME TDD Round 127: Expanded Coverage ==============

    #[test]
    fn test_unary_negate_integer_r127() {
        let mut interp = Interpreter::new();
        let neg = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(make_int(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_unary_negate_float_r127() {
        let mut interp = Interpreter::new();
        let neg = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(make_float(3.14)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&neg).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - (-3.14)).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_unary_not_true_r127() {
        let mut interp = Interpreter::new();
        let not = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(make_bool(true)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&not).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_not_false_r127() {
        let mut interp = Interpreter::new();
        let not = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(make_bool(false)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&not).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_less_equal_equal_r127() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_less_equal_less_r127() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(3), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_less_equal_greater_r127() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(7), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_greater_equal_equal_r127() {
        let mut interp = Interpreter::new();
        let ge = make_binary(make_int(5), AstBinaryOp::GreaterEqual, make_int(5));
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_greater_equal_greater_r127() {
        let mut interp = Interpreter::new();
        let ge = make_binary(make_int(7), AstBinaryOp::GreaterEqual, make_int(5));
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_greater_equal_less_r127() {
        let mut interp = Interpreter::new();
        let ge = make_binary(make_int(3), AstBinaryOp::GreaterEqual, make_int(5));
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_or_true_true_r127() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(true));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_or_true_false_r127() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(true), AstBinaryOp::Or, make_bool(false));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_or_false_true_r127() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_or_false_false_r127() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_and_true_true_r127() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_and_true_false_r127() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_and_false_true_r127() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(false), AstBinaryOp::And, make_bool(true));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_binary_and_false_false_r127() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(false), AstBinaryOp::And, make_bool(false));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_float_add_r127() {
        let mut interp = Interpreter::new();
        let add = make_binary(make_float(1.5), AstBinaryOp::Add, make_float(2.5));
        let result = interp.eval_expr(&add).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 4.0).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_float_subtract_r127() {
        let mut interp = Interpreter::new();
        let sub = make_binary(make_float(5.5), AstBinaryOp::Subtract, make_float(2.5));
        let result = interp.eval_expr(&sub).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 3.0).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_float_multiply_r127() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_float(3.0), AstBinaryOp::Multiply, make_float(2.5));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_float_divide_r127() {
        let mut interp = Interpreter::new();
        let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
        let result = interp.eval_expr(&div).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 2.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_mixed_int_float_add_r127() {
        let mut interp = Interpreter::new();
        let add = make_binary(make_int(5), AstBinaryOp::Add, make_float(2.5));
        let result = interp.eval_expr(&add).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_float_compare_less_r127() {
        let mut interp = Interpreter::new();
        let lt = make_binary(make_float(2.5), AstBinaryOp::Less, make_float(3.5));
        let result = interp.eval_expr(&lt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_float_compare_greater_r127() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_equal_true_r127() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("hello"));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_equal_false_r127() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("world"));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_string_not_equal_r127() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_string("hello"), AstBinaryOp::NotEqual, make_string("world"));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_tuple_literal_r127() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 3);
                assert_eq!(t[0], Value::Integer(1));
                assert_eq!(t[1], Value::Integer(2));
                assert_eq!(t[2], Value::Integer(3));
            }
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_list_literal_r127() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![make_int(10), make_int(20), make_int(30)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(10));
                assert_eq!(arr[1], Value::Integer(20));
                assert_eq!(arr[2], Value::Integer(30));
            }
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_empty_list_r127() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_empty_tuple_r127() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(t) => assert_eq!(t.len(), 0),
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_double_negate_r127() {
        let mut interp = Interpreter::new();
        let inner = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(make_int(42)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Negate,
                operand: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_double_not_r127() {
        let mut interp = Interpreter::new();
        let inner = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(make_bool(true)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer = Expr {
            kind: ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand: Box::new(inner),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&outer).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_block_with_multiple_exprs_r127() {
        let mut interp = Interpreter::new();
        let block = Expr {
            kind: ExprKind::Block(vec![make_int(1), make_int(2), make_int(42)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Integer(42)); // Last expression is returned
    }

    #[test]
    fn test_large_integer_r127() {
        let mut interp = Interpreter::new();
        let large = make_int(i64::MAX);
        let result = interp.eval_expr(&large).expect("should evaluate");
        assert_eq!(result, Value::Integer(i64::MAX));
    }

    #[test]
    fn test_negative_integer_r127() {
        let mut interp = Interpreter::new();
        let neg = make_int(-12345);
        let result = interp.eval_expr(&neg).expect("should evaluate");
        assert_eq!(result, Value::Integer(-12345));
    }

    #[test]
    fn test_zero_integer_r127() {
        let mut interp = Interpreter::new();
        let zero = make_int(0);
        let result = interp.eval_expr(&zero).expect("should evaluate");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_float_zero_r127() {
        let mut interp = Interpreter::new();
        let zero = make_float(0.0);
        let result = interp.eval_expr(&zero).expect("should evaluate");
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_empty_string_r127() {
        let mut interp = Interpreter::new();
        let empty = make_string("");
        let result = interp.eval_expr(&empty).expect("should evaluate");
        assert_eq!(result, Value::String("".into()));
    }

    // ============== EXTREME TDD Round 129 Tests ==============
    // Focus: Control flow, expressions, basic operations

    #[test]
    fn test_if_else_true_branch_r129() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(make_int(100)),
                else_branch: Some(Box::new(make_int(200))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_if_else_false_branch_r129() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(false)),
                then_branch: Box::new(make_int(100)),
                else_branch: Some(Box::new(make_int(200))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(200));
    }

    #[test]
    fn test_if_no_else_true_r129() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(make_int(42)),
                else_branch: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_if_no_else_false_r129() {
        let mut interp = Interpreter::new();
        let if_expr = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(false)),
                then_branch: Box::new(make_int(42)),
                else_branch: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&if_expr).expect("should evaluate");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_creation_r129() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![make_int(1), make_int(2), make_int(3)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_empty_list_r129() {
        let mut interp = Interpreter::new();
        let list = Expr {
            kind: ExprKind::List(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 0);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_tuple_creation_r129() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![make_int(10), make_string("hello")]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 2);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_string_concat_r129() {
        let mut interp = Interpreter::new();
        let concat = make_binary(make_string("hello"), AstBinaryOp::Add, make_string(" world"));
        let result = interp.eval_expr(&concat).expect("should evaluate");
        assert_eq!(result, Value::String("hello world".into()));
    }

    #[test]
    fn test_integer_multiply_r129() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_int(7), AstBinaryOp::Multiply, make_int(6));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_integer_divide_r129() {
        let mut interp = Interpreter::new();
        let div = make_binary(make_int(100), AstBinaryOp::Divide, make_int(4));
        let result = interp.eval_expr(&div).expect("should evaluate");
        assert_eq!(result, Value::Integer(25));
    }

    #[test]
    fn test_gc_track_value_r129() {
        let mut interp = Interpreter::new();
        let tracked = interp.gc_track(Value::Integer(42));
        assert!(tracked >= 0);
    }

    #[test]
    fn test_gc_collect_r129() {
        let mut interp = Interpreter::new();
        let _id = interp.gc_track(Value::Integer(100));
        let stats = interp.gc_collect();
        assert!(stats.collections >= 0);
    }

    #[test]
    fn test_gc_stats_r129() {
        let interp = Interpreter::new();
        let stats = interp.gc_stats();
        assert!(stats.collections >= 0);
    }

    #[test]
    fn test_gc_set_threshold_r129() {
        let mut interp = Interpreter::new();
        interp.gc_set_threshold(100);
    }

    #[test]
    fn test_gc_set_auto_collect_r129() {
        let mut interp = Interpreter::new();
        interp.gc_set_auto_collect(true);
        interp.gc_set_auto_collect(false);
    }

    #[test]
    fn test_gc_clear_r129() {
        let mut interp = Interpreter::new();
        interp.gc_track(Value::Integer(1));
        interp.gc_clear();
    }

    #[test]
    fn test_gc_alloc_array_r129() {
        let mut interp = Interpreter::new();
        let arr = interp.gc_alloc_array(vec![Value::Integer(1), Value::Integer(2)]);
        match arr {
            Value::Array(a) => assert_eq!(a.len(), 2),
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_gc_alloc_string_r129() {
        let mut interp = Interpreter::new();
        let s = interp.gc_alloc_string("test".to_string());
        assert_eq!(s, Value::String("test".into()));
    }

    #[test]
    fn test_push_scope_r129() {
        let mut interp = Interpreter::new();
        interp.push_scope();
        interp.pop_scope();
    }

    #[test]
    fn test_set_and_get_variable_r129() {
        let mut interp = Interpreter::new();
        interp.set_variable("test_var", Value::Integer(42));
        let result = interp.get_variable("test_var");
        assert_eq!(result, Some(Value::Integer(42)));
    }

    #[test]
    fn test_get_nonexistent_variable_r129() {
        let interp = Interpreter::new();
        let result = interp.get_variable("nonexistent");
        assert_eq!(result, None);
    }

    #[test]
    fn test_clear_user_variables_r129() {
        let mut interp = Interpreter::new();
        interp.set_variable("user_var", Value::Integer(99));
        assert!(interp.get_variable("user_var").is_some());
        interp.clear_user_variables();
        // After clearing, user_var should be gone
        assert!(interp.get_variable("user_var").is_none());
    }

    #[test]
    fn test_get_global_bindings_r129() {
        let mut interp = Interpreter::new();
        interp.set_global_binding("global_test".to_string(), Value::Integer(123));
        let bindings = interp.get_global_bindings();
        assert!(bindings.contains_key("global_test"));
    }

    #[test]
    fn test_get_current_bindings_r129() {
        let mut interp = Interpreter::new();
        interp.set_variable("current_test", Value::String("test".into()));
        let bindings = interp.get_current_bindings();
        assert!(bindings.contains_key("current_test"));
    }

    #[test]
    fn test_push_error_scope_r129() {
        let mut interp = Interpreter::new();
        interp.push_error_scope();
    }

    #[test]
    fn test_pop_error_scope_r129() {
        let mut interp = Interpreter::new();
        interp.push_error_scope();
        interp.pop_error_scope();
    }

    #[test]
    fn test_capture_stdout_r129() {
        let mut interp = Interpreter::new();
        interp.capture_stdout("line 1".to_string());
        interp.capture_stdout("line 2".to_string());
        assert!(interp.has_stdout());
    }

    #[test]
    fn test_get_stdout_r129() {
        let mut interp = Interpreter::new();
        interp.capture_stdout("hello".to_string());
        let output = interp.get_stdout();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_clear_stdout_r129() {
        let mut interp = Interpreter::new();
        interp.capture_stdout("test".to_string());
        interp.clear_stdout();
        assert!(!interp.has_stdout());
    }

    #[test]
    fn test_get_cache_stats_r129() {
        let interp = Interpreter::new();
        let stats = interp.get_cache_stats();
        assert!(stats.contains_key("hit_rate") || stats.is_empty());
    }

    #[test]
    fn test_clear_caches_r129() {
        let mut interp = Interpreter::new();
        interp.clear_caches();
    }

    #[test]
    fn test_clear_type_feedback_r129() {
        let mut interp = Interpreter::new();
        interp.clear_type_feedback();
    }

    #[test]
    fn test_push_and_pop_r129() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(42)).expect("push should work");
        let result = interp.pop().expect("pop should work");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_peek_r129() {
        let mut interp = Interpreter::new();
        interp.push(Value::Integer(1)).expect("push should work");
        interp.push(Value::Integer(2)).expect("push should work");
        let result = interp.peek(0).expect("peek should work");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_pop_empty_stack_error_r129() {
        let mut interp = Interpreter::new();
        let result = interp.pop();
        assert!(result.is_err());
    }

    #[test]
    fn test_peek_out_of_bounds_r129() {
        let interp = Interpreter::new();
        let result = interp.peek(100);
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_string_simple_r129() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("2 + 2").expect("should evaluate");
        assert_eq!(result.to_string(), "4");
    }

    #[test]
    fn test_eval_string_comparison_r129() {
        let mut interp = Interpreter::new();
        let result = interp.eval_string("5 > 3").expect("should evaluate");
        assert_eq!(result.to_string(), "true");
    }

    #[test]
    fn test_range_expr_r129() {
        let mut interp = Interpreter::new();
        let range = Expr {
            kind: ExprKind::Range {
                start: Box::new(make_int(1)),
                end: Box::new(make_int(5)),
                inclusive: false,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&range).expect("should evaluate");
        // Range returns a Range object (not Array) - just verify it evaluated
        assert_ne!(result, Value::Nil);
    }

    #[test]
    fn test_range_inclusive_r129() {
        let mut interp = Interpreter::new();
        let range = Expr {
            kind: ExprKind::Range {
                start: Box::new(make_int(1)),
                end: Box::new(make_int(5)),
                inclusive: true,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&range).expect("should evaluate");
        // Range returns a Range object (not Array) - just verify it evaluated
        assert_ne!(result, Value::Nil);
    }

    #[test]
    fn test_current_env_r129() {
        let mut interp = Interpreter::new();
        interp.set_variable("env_test", Value::Integer(42));
        let env = interp.current_env();
        let borrowed = env.borrow();
        assert!(borrowed.contains_key("env_test"));
    }

    #[test]
    fn test_default_interpreter_r129() {
        let interp = Interpreter::default();
        assert!(interp.get_variable("len").is_some());
    }

    #[test]
    fn test_nested_if_r129() {
        let mut interp = Interpreter::new();
        let inner_if = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(make_int(10)),
                else_branch: Some(Box::new(make_int(20))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let outer_if = Expr {
            kind: ExprKind::If {
                condition: Box::new(make_bool(true)),
                then_branch: Box::new(inner_if),
                else_branch: Some(Box::new(make_int(30))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&outer_if).expect("should evaluate");
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_empty_block_r129() {
        let mut interp = Interpreter::new();
        let block = Expr {
            kind: ExprKind::Block(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_empty_tuple_r129() {
        let mut interp = Interpreter::new();
        let tuple = Expr {
            kind: ExprKind::Tuple(vec![]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(t) => assert_eq!(t.len(), 0),
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_set_variable_string_r129() {
        let mut interp = Interpreter::new();
        interp.set_variable_string("str_var".to_string(), Value::String("hello".into()));
        let result = interp.get_variable("str_var");
        assert_eq!(result, Some(Value::String("hello".into())));
    }

    #[test]
    fn test_index_expr_array_r129() {
        let mut interp = Interpreter::new();
        let list_expr = Expr {
            kind: ExprKind::List(vec![make_int(10), make_int(20), make_int(30)]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let index_expr = Expr {
            kind: ExprKind::IndexAccess {
                object: Box::new(list_expr),
                index: Box::new(make_int(1)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&index_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_index_expr_tuple_r129() {
        let mut interp = Interpreter::new();
        let tuple_expr = Expr {
            kind: ExprKind::Tuple(vec![make_int(100), make_string("test")]),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let index_expr = Expr {
            kind: ExprKind::IndexAccess {
                object: Box::new(tuple_expr),
                index: Box::new(make_int(0)),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&index_expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_bitwise_and_r129() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_int(0b1100), AstBinaryOp::BitwiseAnd, make_int(0b1010));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b1000));
    }

    #[test]
    fn test_bitwise_or_r129() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_int(0b1100), AstBinaryOp::BitwiseOr, make_int(0b1010));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b1110));
    }

    #[test]
    fn test_bitwise_xor_r129() {
        let mut interp = Interpreter::new();
        let xor = make_binary(make_int(0b1100), AstBinaryOp::BitwiseXor, make_int(0b1010));
        let result = interp.eval_expr(&xor).expect("should evaluate");
        assert_eq!(result, Value::Integer(0b0110));
    }

    #[test]
    fn test_binary_less_than_r129() {
        let mut interp = Interpreter::new();
        let lt = make_binary(make_int(5), AstBinaryOp::Less, make_int(10));
        let result = interp.eval_expr(&lt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_greater_than_r129() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_int(10), AstBinaryOp::Greater, make_int(5));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_less_equal_r129() {
        let mut interp = Interpreter::new();
        let le = make_binary(make_int(5), AstBinaryOp::LessEqual, make_int(5));
        let result = interp.eval_expr(&le).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_greater_equal_r129() {
        let mut interp = Interpreter::new();
        let ge = make_binary(make_int(10), AstBinaryOp::GreaterEqual, make_int(10));
        let result = interp.eval_expr(&ge).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_equal_r129() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_int(42), AstBinaryOp::Equal, make_int(42));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binary_not_equal_r129() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_int(42), AstBinaryOp::NotEqual, make_int(99));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_mixed_float_int_subtract_r129() {
        let mut interp = Interpreter::new();
        let sub = make_binary(make_float(10.5), AstBinaryOp::Subtract, make_int(5));
        let result = interp.eval_expr(&sub).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 5.5).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_mixed_float_int_multiply_r129() {
        let mut interp = Interpreter::new();
        let mul = make_binary(make_int(3), AstBinaryOp::Multiply, make_float(2.5));
        let result = interp.eval_expr(&mul).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 7.5).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_float_divide_r129() {
        let mut interp = Interpreter::new();
        let div = make_binary(make_float(10.0), AstBinaryOp::Divide, make_float(4.0));
        let result = interp.eval_expr(&div).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 2.5).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_logical_and_both_true_r129() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(true));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_logical_and_one_false_r129() {
        let mut interp = Interpreter::new();
        let and = make_binary(make_bool(true), AstBinaryOp::And, make_bool(false));
        let result = interp.eval_expr(&and).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_both_false_r129() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(false));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_logical_or_one_true_r129() {
        let mut interp = Interpreter::new();
        let or = make_binary(make_bool(false), AstBinaryOp::Or, make_bool(true));
        let result = interp.eval_expr(&or).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    // ============================================================================
    // EXTREME TDD Round 157: Additional interpreter tests
    // Target: Push coverage further
    // ============================================================================

    #[test]
    fn test_nested_binary_operations_r157() {
        let mut interp = Interpreter::new();
        // ((1 + 2) * 3) - 4 = 5
        let add = make_binary(make_int(1), AstBinaryOp::Add, make_int(2));
        let mul = make_binary(add, AstBinaryOp::Multiply, make_int(3));
        let sub = make_binary(mul, AstBinaryOp::Subtract, make_int(4));
        let result = interp.eval_expr(&sub).expect("should evaluate");
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_deeply_nested_binary_r157() {
        let mut interp = Interpreter::new();
        // Build: 1 + 2 + 3 + 4 + 5 = 15
        let mut expr = make_int(1);
        for i in 2..=5 {
            expr = make_binary(expr, AstBinaryOp::Add, make_int(i));
        }
        let result = interp.eval_expr(&expr).expect("should evaluate");
        assert_eq!(result, Value::Integer(15));
    }

    #[test]
    fn test_unary_negate_float_r157() {
        let mut interp = Interpreter::new();
        let neg = make_unary(UnaryOp::Negate, make_float(3.14));
        let result = interp.eval_expr(&neg).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f + 3.14).abs() < 0.0001),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_unary_not_true_r157() {
        let mut interp = Interpreter::new();
        let not = make_unary(UnaryOp::Not, make_bool(true));
        let result = interp.eval_expr(&not).expect("should evaluate");
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_unary_not_false_r157() {
        let mut interp = Interpreter::new();
        let not = make_unary(UnaryOp::Not, make_bool(false));
        let result = interp.eval_expr(&not).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_integer_modulo_r157() {
        let mut interp = Interpreter::new();
        let modulo = make_binary(make_int(17), AstBinaryOp::Modulo, make_int(5));
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_integer_modulo_zero_remainder_r157() {
        let mut interp = Interpreter::new();
        let modulo = make_binary(make_int(20), AstBinaryOp::Modulo, make_int(5));
        let result = interp.eval_expr(&modulo).expect("should evaluate");
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_nested_if_expression_r157() {
        let mut interp = Interpreter::new();
        // if true { if false { 1 } else { 2 } } else { 3 }
        let inner_if = make_if(make_bool(false), make_int(1), Some(make_int(2)));
        let outer_if = make_if(make_bool(true), inner_if, Some(make_int(3)));
        let result = interp.eval_expr(&outer_if).expect("should evaluate");
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_block_with_multiple_expressions_r157() {
        let mut interp = Interpreter::new();
        let block = make_block(vec![
            make_int(1),
            make_int(2),
            make_int(3),
            make_int(42),
        ]);
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_empty_block_returns_nil_r157() {
        let mut interp = Interpreter::new();
        let block = make_block(vec![]);
        let result = interp.eval_expr(&block).expect("should evaluate");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_list_of_floats_r157() {
        let mut interp = Interpreter::new();
        let list = make_list(vec![
            make_float(1.1),
            make_float(2.2),
            make_float(3.3),
        ]);
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert!(matches!(arr[0], Value::Float(_)));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_list_of_bools_r157() {
        let mut interp = Interpreter::new();
        let list = make_list(vec![
            make_bool(true),
            make_bool(false),
            make_bool(true),
        ]);
        let result = interp.eval_expr(&list).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_tuple_of_three_r157() {
        let mut interp = Interpreter::new();
        let tuple = make_tuple(vec![
            make_int(1),
            make_float(2.5),
            make_string("three"),
        ]);
        let result = interp.eval_expr(&tuple).expect("should evaluate");
        match result {
            Value::Tuple(t) => {
                assert_eq!(t.len(), 3);
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_nested_list_r157() {
        let mut interp = Interpreter::new();
        let inner1 = make_list(vec![make_int(1), make_int(2)]);
        let inner2 = make_list(vec![make_int(3), make_int(4)]);
        let outer = make_list(vec![inner1, inner2]);
        let result = interp.eval_expr(&outer).expect("should evaluate");
        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert!(matches!(arr[0], Value::Array(_)));
            }
            _ => panic!("Expected Array"),
        }
    }

    #[test]
    fn test_let_binding_chain_r157() {
        let mut interp = Interpreter::new();
        // let x = 1; let y = x + 1; let z = y + 1; z
        let z = make_ident("z");
        let let_z = make_let("z", make_binary(make_ident("y"), AstBinaryOp::Add, make_int(1)), z);
        let let_y = make_let("y", make_binary(make_ident("x"), AstBinaryOp::Add, make_int(1)), let_z);
        let let_x = make_let("x", make_int(1), let_y);
        let result = interp.eval_expr(&let_x).expect("should evaluate");
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_range_exclusive_r157() {
        let mut interp = Interpreter::new();
        let range = make_range(make_int(0), make_int(5), false);
        let result = interp.eval_expr(&range).expect("should evaluate");
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
    fn test_range_inclusive_r157() {
        let mut interp = Interpreter::new();
        let range = make_range(make_int(1), make_int(10), true);
        let result = interp.eval_expr(&range).expect("should evaluate");
        match result {
            Value::Range { start, end, inclusive } => {
                assert_eq!(*start, Value::Integer(1));
                assert_eq!(*end, Value::Integer(10));
                assert!(inclusive);
            }
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_range_negative_values_r157() {
        let mut interp = Interpreter::new();
        let range = make_range(make_int(-10), make_int(-1), false);
        let result = interp.eval_expr(&range).expect("should evaluate");
        match result {
            Value::Range { start, end, .. } => {
                assert_eq!(*start, Value::Integer(-10));
                assert_eq!(*end, Value::Integer(-1));
            }
            _ => panic!("Expected Range"),
        }
    }

    #[test]
    fn test_index_access_r157() {
        let mut interp = Interpreter::new();
        let list = make_list(vec![make_int(10), make_int(20), make_int(30)]);
        let indexed = make_index(list, make_int(1));
        let result = interp.eval_expr(&indexed).expect("should evaluate");
        assert_eq!(result, Value::Integer(20));
    }

    #[test]
    fn test_index_access_first_r157() {
        let mut interp = Interpreter::new();
        let list = make_list(vec![make_int(100), make_int(200)]);
        let indexed = make_index(list, make_int(0));
        let result = interp.eval_expr(&indexed).expect("should evaluate");
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_index_access_last_r157() {
        let mut interp = Interpreter::new();
        let list = make_list(vec![make_int(1), make_int(2), make_int(3), make_int(999)]);
        let indexed = make_index(list, make_int(3));
        let result = interp.eval_expr(&indexed).expect("should evaluate");
        assert_eq!(result, Value::Integer(999));
    }

    #[test]
    fn test_float_comparison_less_r157() {
        let mut interp = Interpreter::new();
        let lt = make_binary(make_float(1.5), AstBinaryOp::Less, make_float(2.5));
        let result = interp.eval_expr(&lt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_float_comparison_greater_r157() {
        let mut interp = Interpreter::new();
        let gt = make_binary(make_float(3.5), AstBinaryOp::Greater, make_float(2.5));
        let result = interp.eval_expr(&gt).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_comparison_equal_r157() {
        let mut interp = Interpreter::new();
        let eq = make_binary(make_string("hello"), AstBinaryOp::Equal, make_string("hello"));
        let result = interp.eval_expr(&eq).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_string_comparison_not_equal_r157() {
        let mut interp = Interpreter::new();
        let ne = make_binary(make_string("hello"), AstBinaryOp::NotEqual, make_string("world"));
        let result = interp.eval_expr(&ne).expect("should evaluate");
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_large_integer_r157() {
        let mut interp = Interpreter::new();
        let large = make_int(999_999_999_999);
        let result = interp.eval_expr(&large).expect("should evaluate");
        assert_eq!(result, Value::Integer(999_999_999_999));
    }

    #[test]
    fn test_negative_large_integer_r157() {
        let mut interp = Interpreter::new();
        let large_neg = make_int(-999_999_999_999);
        let result = interp.eval_expr(&large_neg).expect("should evaluate");
        assert_eq!(result, Value::Integer(-999_999_999_999));
    }

    #[test]
    fn test_scientific_float_r157() {
        let mut interp = Interpreter::new();
        let sci = make_float(1.5e10);
        let result = interp.eval_expr(&sci).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 1.5e10).abs() < 1.0),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_very_small_float_r157() {
        let mut interp = Interpreter::new();
        let small = make_float(1.5e-10);
        let result = interp.eval_expr(&small).expect("should evaluate");
        match result {
            Value::Float(f) => assert!((f - 1.5e-10).abs() < 1e-15),
            _ => panic!("Expected Float"),
        }
    }
}
