//! Tests for parameter usage analysis
//!
//! EXTREME TDD Round 85: Comprehensive tests for param_usage_analysis module

#[cfg(test)]
mod tests {
    use super::super::param_usage_analysis::*;
    use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span, UnaryOp};

    /// Helper to create identifier expression
    fn make_ident(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create integer literal
    fn make_int(val: i64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Integer(val, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create string literal
    fn make_string(val: &str) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::String(val.to_string())),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create binary expression
    fn make_binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
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

    /// Helper to create block expression
    fn make_block(exprs: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::Block(exprs),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create if expression
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

    /// Helper to create let expression
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

    /// Helper to create index access expression
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

    /// Helper to create unary expression
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

    /// Helper to create while expression
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

    /// Helper to create for expression
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

    /// Helper to create lambda expression
    fn make_lambda(body: Expr) -> Expr {
        Expr {
            kind: ExprKind::Lambda {
                params: vec![],
                body: Box::new(body),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create assign expression
    fn make_assign(target: Expr, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::Assign {
                target: Box::new(target),
                value: Box::new(value),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create compound assign expression
    fn make_compound_assign(target: Expr, value: Expr) -> Expr {
        Expr {
            kind: ExprKind::CompoundAssign {
                target: Box::new(target),
                op: BinaryOp::Add,
                value: Box::new(value),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    /// Helper to create boolean literal
    fn make_bool(val: bool) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Bool(val)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    // ============== traverse_expr_for_check tests ==============

    #[test]
    fn test_traverse_block() {
        let block = make_block(vec![make_ident("x")]);
        let result = traverse_expr_for_check(&block, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_if_condition() {
        let if_expr = make_if(make_ident("x"), make_int(1), Some(make_int(2)));
        let result = traverse_expr_for_check(&if_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_if_then_branch() {
        let if_expr = make_if(make_bool(true), make_ident("x"), None);
        let result = traverse_expr_for_check(&if_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_if_else_branch() {
        let if_expr = make_if(make_bool(true), make_int(1), Some(make_ident("x")));
        let result = traverse_expr_for_check(&if_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_let_value() {
        let let_expr = make_let("y", make_ident("x"), make_int(1));
        let result = traverse_expr_for_check(&let_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_let_body() {
        let let_expr = make_let("y", make_int(1), make_ident("x"));
        let result = traverse_expr_for_check(&let_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_binary_left() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let result = traverse_expr_for_check(&binary, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_binary_right() {
        let binary = make_binary(make_int(1), BinaryOp::Add, make_ident("x"));
        let result = traverse_expr_for_check(&binary, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_while() {
        let while_expr = make_while(make_ident("x"), make_int(1));
        let result = traverse_expr_for_check(&while_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_while_body() {
        let while_expr = make_while(make_bool(true), make_ident("x"));
        let result = traverse_expr_for_check(&while_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_for() {
        let for_expr = make_for("i", make_ident("x"), make_int(1));
        let result = traverse_expr_for_check(&for_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_for_body() {
        let for_expr = make_for("i", make_int(1), make_ident("x"));
        let result = traverse_expr_for_check(&for_expr, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_assign_target() {
        let assign = make_assign(make_ident("x"), make_int(1));
        let result = traverse_expr_for_check(&assign, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_assign_value() {
        let assign = make_assign(make_ident("y"), make_ident("x"));
        let result = traverse_expr_for_check(&assign, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_compound_assign() {
        let compound = make_compound_assign(make_ident("x"), make_int(1));
        let result = traverse_expr_for_check(&compound, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_call_args() {
        let call = make_call(make_ident("foo"), vec![make_ident("x")]);
        let result = traverse_expr_for_check(&call, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_index_object() {
        let index = make_index(make_ident("x"), make_int(0));
        let result = traverse_expr_for_check(&index, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_index_index() {
        let index = make_index(make_ident("arr"), make_ident("x"));
        let result = traverse_expr_for_check(&index, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_unary() {
        let unary = make_unary(UnaryOp::Not, make_ident("x"));
        let result = traverse_expr_for_check(&unary, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(result);
    }

    #[test]
    fn test_traverse_not_found() {
        let binary = make_binary(make_int(1), BinaryOp::Add, make_int(2));
        let result = traverse_expr_for_check(&binary, |e| {
            if let ExprKind::Identifier(name) = &e.kind {
                if name == "x" {
                    return Some(true);
                }
            }
            None
        });
        assert!(!result);
    }

    // ============== find_param_in_direct_args tests ==============

    #[test]
    fn test_find_param_in_direct_args_found() {
        let args = vec![make_ident("x"), make_int(1)];
        assert!(find_param_in_direct_args("x", &args));
    }

    #[test]
    fn test_find_param_in_direct_args_not_found() {
        let args = vec![make_ident("y"), make_int(1)];
        assert!(!find_param_in_direct_args("x", &args));
    }

    #[test]
    fn test_find_param_in_empty_args() {
        let args: Vec<Expr> = vec![];
        assert!(!find_param_in_direct_args("x", &args));
    }

    // ============== check_call_for_param_argument tests ==============

    #[test]
    fn test_check_call_for_param_direct() {
        let func = make_ident("foo");
        let args = vec![make_ident("x")];
        assert!(check_call_for_param_argument("x", &func, &args));
    }

    #[test]
    fn test_check_call_for_param_nested() {
        let func = make_ident("foo");
        let inner_call = make_call(make_ident("bar"), vec![make_ident("x")]);
        let args = vec![inner_call];
        assert!(check_call_for_param_argument("x", &func, &args));
    }

    #[test]
    fn test_check_call_for_param_not_found() {
        let func = make_ident("foo");
        let args = vec![make_ident("y")];
        assert!(!check_call_for_param_argument("x", &func, &args));
    }

    // ============== check_expressions_for_param tests ==============

    #[test]
    fn test_check_expressions_for_param() {
        let call = make_call(make_ident("foo"), vec![make_ident("x")]);
        let exprs = vec![call];
        assert!(check_expressions_for_param("x", &exprs));
    }

    #[test]
    fn test_check_expressions_for_param_not_found() {
        let exprs = vec![make_int(1)];
        assert!(!check_expressions_for_param("x", &exprs));
    }

    // ============== check_if_for_param tests ==============

    #[test]
    fn test_check_if_for_param_in_condition() {
        let condition = make_call(make_ident("foo"), vec![make_ident("x")]);
        let then_branch = make_int(1);
        assert!(check_if_for_param("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_for_param_in_then() {
        let condition = make_bool(true);
        let then_branch = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(check_if_for_param("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_for_param_in_else() {
        let condition = make_bool(true);
        let then_branch = make_int(1);
        let else_branch = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(check_if_for_param(
            "x",
            &condition,
            &then_branch,
            Some(&else_branch)
        ));
    }

    // ============== check_let_for_param tests ==============

    #[test]
    fn test_check_let_for_param_in_value() {
        let value = make_call(make_ident("foo"), vec![make_ident("x")]);
        let body = make_int(1);
        assert!(check_let_for_param("x", &value, &body));
    }

    #[test]
    fn test_check_let_for_param_in_body() {
        let value = make_int(1);
        let body = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(check_let_for_param("x", &value, &body));
    }

    #[test]
    fn test_check_let_for_param_not_found() {
        let value = make_int(1);
        let body = make_int(2);
        assert!(!check_let_for_param("x", &value, &body));
    }

    // ============== check_binary_for_param tests ==============

    #[test]
    fn test_check_binary_for_param_in_left() {
        let left = make_call(make_ident("foo"), vec![make_ident("x")]);
        let right = make_int(1);
        assert!(check_binary_for_param("x", &left, &right));
    }

    #[test]
    fn test_check_binary_for_param_in_right() {
        let left = make_int(1);
        let right = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(check_binary_for_param("x", &left, &right));
    }

    #[test]
    fn test_check_binary_for_param_not_found() {
        let left = make_int(1);
        let right = make_int(2);
        assert!(!check_binary_for_param("x", &left, &right));
    }

    // ============== is_param_used_as_function_argument tests ==============

    #[test]
    fn test_is_param_used_as_function_argument_call() {
        let call = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(is_param_used_as_function_argument("x", &call));
    }

    #[test]
    fn test_is_param_used_as_function_argument_block() {
        let block = make_block(vec![make_call(
            make_ident("foo"),
            vec![make_ident("x")],
        )]);
        assert!(is_param_used_as_function_argument("x", &block));
    }

    #[test]
    fn test_is_param_used_as_function_argument_if() {
        let if_expr = make_if(
            make_bool(true),
            make_call(make_ident("foo"), vec![make_ident("x")]),
            None,
        );
        assert!(is_param_used_as_function_argument("x", &if_expr));
    }

    #[test]
    fn test_is_param_used_as_function_argument_let() {
        let let_expr = make_let(
            "y",
            make_int(1),
            make_call(make_ident("foo"), vec![make_ident("x")]),
        );
        assert!(is_param_used_as_function_argument("x", &let_expr));
    }

    #[test]
    fn test_is_param_used_as_function_argument_binary() {
        let binary = make_binary(
            make_call(make_ident("foo"), vec![make_ident("x")]),
            BinaryOp::Add,
            make_int(1),
        );
        assert!(is_param_used_as_function_argument("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_function_argument_unary() {
        let unary = make_unary(
            UnaryOp::Not,
            make_call(make_ident("foo"), vec![make_ident("x")]),
        );
        assert!(is_param_used_as_function_argument("x", &unary));
    }

    #[test]
    fn test_is_param_used_as_function_argument_not_found() {
        let binary = make_binary(make_int(1), BinaryOp::Add, make_int(2));
        assert!(!is_param_used_as_function_argument("x", &binary));
    }

    // ============== check_func_call tests ==============

    #[test]
    fn test_check_func_call_direct() {
        let func = make_ident("x");
        let args = vec![make_int(1)];
        assert!(check_func_call("x", &func, &args));
    }

    #[test]
    fn test_check_func_call_nested() {
        let func = make_ident("foo");
        let inner = make_call(make_ident("x"), vec![]);
        let args = vec![inner];
        assert!(check_func_call("x", &func, &args));
    }

    #[test]
    fn test_check_func_call_not_found() {
        let func = make_ident("foo");
        let args = vec![make_int(1)];
        assert!(!check_func_call("x", &func, &args));
    }

    // ============== check_if_for_func tests ==============

    #[test]
    fn test_check_if_for_func_condition() {
        let condition = make_call(make_ident("x"), vec![]);
        let then_branch = make_int(1);
        assert!(check_if_for_func("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_for_func_then() {
        let condition = make_bool(true);
        let then_branch = make_call(make_ident("x"), vec![]);
        assert!(check_if_for_func("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_for_func_else() {
        let condition = make_bool(true);
        let then_branch = make_int(1);
        let else_branch = make_call(make_ident("x"), vec![]);
        assert!(check_if_for_func(
            "x",
            &condition,
            &then_branch,
            Some(&else_branch)
        ));
    }

    // ============== check_let_and_binary_for_func tests ==============

    #[test]
    fn test_check_let_and_binary_for_func_value() {
        let value = make_call(make_ident("x"), vec![]);
        let body = make_int(1);
        assert!(check_let_and_binary_for_func("x", &value, &body));
    }

    #[test]
    fn test_check_let_and_binary_for_func_body() {
        let value = make_int(1);
        let body = make_call(make_ident("x"), vec![]);
        assert!(check_let_and_binary_for_func("x", &value, &body));
    }

    // ============== is_param_used_as_function tests ==============

    #[test]
    fn test_is_param_used_as_function_call() {
        let call = make_call(make_ident("x"), vec![]);
        assert!(is_param_used_as_function("x", &call));
    }

    #[test]
    fn test_is_param_used_as_function_block() {
        let block = make_block(vec![make_call(make_ident("x"), vec![])]);
        assert!(is_param_used_as_function("x", &block));
    }

    #[test]
    fn test_is_param_used_as_function_if() {
        let if_expr = make_if(make_bool(true), make_call(make_ident("x"), vec![]), None);
        assert!(is_param_used_as_function("x", &if_expr));
    }

    #[test]
    fn test_is_param_used_as_function_let() {
        let let_expr = make_let("y", make_int(1), make_call(make_ident("x"), vec![]));
        assert!(is_param_used_as_function("x", &let_expr));
    }

    #[test]
    fn test_is_param_used_as_function_binary() {
        let binary = make_binary(make_call(make_ident("x"), vec![]), BinaryOp::Add, make_int(1));
        assert!(is_param_used_as_function("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_function_lambda() {
        let lambda = make_lambda(make_call(make_ident("x"), vec![]));
        assert!(is_param_used_as_function("x", &lambda));
    }

    #[test]
    fn test_is_param_used_as_function_not_found() {
        let binary = make_binary(make_int(1), BinaryOp::Add, make_int(2));
        assert!(!is_param_used_as_function("x", &binary));
    }

    // ============== is_numeric_operator tests ==============

    #[test]
    fn test_is_numeric_operator_add() {
        assert!(is_numeric_operator(&BinaryOp::Add));
    }

    #[test]
    fn test_is_numeric_operator_subtract() {
        assert!(is_numeric_operator(&BinaryOp::Subtract));
    }

    #[test]
    fn test_is_numeric_operator_multiply() {
        assert!(is_numeric_operator(&BinaryOp::Multiply));
    }

    #[test]
    fn test_is_numeric_operator_divide() {
        assert!(is_numeric_operator(&BinaryOp::Divide));
    }

    #[test]
    fn test_is_numeric_operator_modulo() {
        assert!(is_numeric_operator(&BinaryOp::Modulo));
    }

    #[test]
    fn test_is_numeric_operator_less() {
        assert!(is_numeric_operator(&BinaryOp::Less));
    }

    #[test]
    fn test_is_numeric_operator_greater() {
        assert!(is_numeric_operator(&BinaryOp::Greater));
    }

    #[test]
    fn test_is_numeric_operator_less_equal() {
        assert!(is_numeric_operator(&BinaryOp::LessEqual));
    }

    #[test]
    fn test_is_numeric_operator_greater_equal() {
        assert!(is_numeric_operator(&BinaryOp::GreaterEqual));
    }

    #[test]
    fn test_is_numeric_operator_equal_not_numeric() {
        assert!(!is_numeric_operator(&BinaryOp::Equal));
    }

    #[test]
    fn test_is_numeric_operator_and_not_numeric() {
        assert!(!is_numeric_operator(&BinaryOp::And));
    }

    // ============== has_param_in_operation tests ==============

    #[test]
    fn test_has_param_in_operation_left() {
        let left = make_ident("x");
        let right = make_int(1);
        assert!(has_param_in_operation("x", &left, &right));
    }

    #[test]
    fn test_has_param_in_operation_right() {
        let left = make_int(1);
        let right = make_ident("x");
        assert!(has_param_in_operation("x", &left, &right));
    }

    #[test]
    fn test_has_param_in_operation_none() {
        let left = make_int(1);
        let right = make_int(2);
        assert!(!has_param_in_operation("x", &left, &right));
    }

    // ============== is_string_concatenation tests ==============

    #[test]
    fn test_is_string_concatenation_string_left() {
        let left = make_string("hello");
        let right = make_ident("x");
        assert!(is_string_concatenation(&BinaryOp::Add, &left, &right));
    }

    #[test]
    fn test_is_string_concatenation_string_right() {
        let left = make_ident("x");
        let right = make_string("world");
        assert!(is_string_concatenation(&BinaryOp::Add, &left, &right));
    }

    #[test]
    fn test_is_string_concatenation_not_add() {
        let left = make_string("hello");
        let right = make_string("world");
        assert!(!is_string_concatenation(&BinaryOp::Multiply, &left, &right));
    }

    #[test]
    fn test_is_string_concatenation_no_strings() {
        let left = make_int(1);
        let right = make_int(2);
        assert!(!is_string_concatenation(&BinaryOp::Add, &left, &right));
    }

    // ============== check_binary_numeric_usage tests ==============

    #[test]
    fn test_check_binary_numeric_usage_direct() {
        let left = make_ident("x");
        let right = make_int(1);
        assert!(check_binary_numeric_usage(
            "x",
            &BinaryOp::Add,
            &left,
            &right
        ));
    }

    #[test]
    fn test_check_binary_numeric_usage_string_concat_excluded() {
        let left = make_string("hello");
        let right = make_ident("x");
        assert!(!check_binary_numeric_usage(
            "x",
            &BinaryOp::Add,
            &left,
            &right
        ));
    }

    #[test]
    fn test_check_binary_numeric_usage_nested() {
        let inner = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let left = make_int(1);
        assert!(check_binary_numeric_usage(
            "x",
            &BinaryOp::Add,
            &left,
            &inner
        ));
    }

    // ============== check_block_numeric_usage tests ==============

    #[test]
    fn test_check_block_numeric_usage() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let exprs = vec![binary];
        assert!(check_block_numeric_usage("x", &exprs));
    }

    #[test]
    fn test_check_block_numeric_usage_not_found() {
        let exprs = vec![make_int(1)];
        assert!(!check_block_numeric_usage("x", &exprs));
    }

    // ============== check_if_numeric_usage tests ==============

    #[test]
    fn test_check_if_numeric_usage_condition() {
        let condition = make_binary(make_ident("x"), BinaryOp::Greater, make_int(0));
        let then_branch = make_int(1);
        assert!(check_if_numeric_usage("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_numeric_usage_then() {
        let condition = make_bool(true);
        let then_branch = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(check_if_numeric_usage("x", &condition, &then_branch, None));
    }

    #[test]
    fn test_check_if_numeric_usage_else() {
        let condition = make_bool(true);
        let then_branch = make_int(1);
        let else_branch = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(check_if_numeric_usage(
            "x",
            &condition,
            &then_branch,
            Some(&else_branch)
        ));
    }

    // ============== check_let_numeric_usage tests ==============

    #[test]
    fn test_check_let_numeric_usage_value() {
        let value = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let body = make_int(2);
        assert!(check_let_numeric_usage("x", &value, &body));
    }

    #[test]
    fn test_check_let_numeric_usage_body() {
        let value = make_int(1);
        let body = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(check_let_numeric_usage("x", &value, &body));
    }

    // ============== check_call_numeric_usage tests ==============

    #[test]
    fn test_check_call_numeric_usage() {
        let arg = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let args = vec![arg];
        assert!(check_call_numeric_usage("x", &args));
    }

    #[test]
    fn test_check_call_numeric_usage_not_found() {
        let args = vec![make_int(1)];
        assert!(!check_call_numeric_usage("x", &args));
    }

    // ============== is_param_used_numerically tests ==============

    #[test]
    fn test_is_param_used_numerically_binary() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(is_param_used_numerically("x", &binary));
    }

    #[test]
    fn test_is_param_used_numerically_block() {
        let block = make_block(vec![make_binary(
            make_ident("x"),
            BinaryOp::Add,
            make_int(1),
        )]);
        assert!(is_param_used_numerically("x", &block));
    }

    #[test]
    fn test_is_param_used_numerically_if() {
        let if_expr = make_if(
            make_binary(make_ident("x"), BinaryOp::Greater, make_int(0)),
            make_int(1),
            None,
        );
        assert!(is_param_used_numerically("x", &if_expr));
    }

    #[test]
    fn test_is_param_used_numerically_let() {
        let let_expr = make_let(
            "y",
            make_binary(make_ident("x"), BinaryOp::Add, make_int(1)),
            make_int(2),
        );
        assert!(is_param_used_numerically("x", &let_expr));
    }

    #[test]
    fn test_is_param_used_numerically_call() {
        let call = make_call(
            make_ident("foo"),
            vec![make_binary(make_ident("x"), BinaryOp::Add, make_int(1))],
        );
        assert!(is_param_used_numerically("x", &call));
    }

    #[test]
    fn test_is_param_used_numerically_lambda() {
        let lambda = make_lambda(make_binary(make_ident("x"), BinaryOp::Add, make_int(1)));
        assert!(is_param_used_numerically("x", &lambda));
    }

    #[test]
    fn test_is_param_used_numerically_while() {
        let while_expr = make_while(
            make_binary(make_ident("x"), BinaryOp::Less, make_int(10)),
            make_int(1),
        );
        assert!(is_param_used_numerically("x", &while_expr));
    }

    #[test]
    fn test_is_param_used_numerically_while_body() {
        let while_expr = make_while(
            make_bool(true),
            make_binary(make_ident("x"), BinaryOp::Add, make_int(1)),
        );
        assert!(is_param_used_numerically("x", &while_expr));
    }

    #[test]
    fn test_is_param_used_numerically_for() {
        let for_expr = make_for(
            "i",
            make_binary(make_ident("x"), BinaryOp::Add, make_int(1)),
            make_int(1),
        );
        assert!(is_param_used_numerically("x", &for_expr));
    }

    #[test]
    fn test_is_param_used_numerically_for_body() {
        let for_expr = make_for(
            "i",
            make_int(1),
            make_binary(make_ident("x"), BinaryOp::Add, make_int(1)),
        );
        assert!(is_param_used_numerically("x", &for_expr));
    }

    #[test]
    fn test_is_param_used_numerically_not_found() {
        let call = make_call(make_ident("foo"), vec![make_ident("y")]);
        assert!(!is_param_used_numerically("x", &call));
    }

    // ============== check_call_contains_param tests ==============

    #[test]
    fn test_check_call_contains_param_func() {
        let func = make_ident("x");
        let args = vec![make_int(1)];
        assert!(check_call_contains_param("x", &func, &args));
    }

    #[test]
    fn test_check_call_contains_param_args() {
        let func = make_ident("foo");
        let args = vec![make_ident("x")];
        assert!(check_call_contains_param("x", &func, &args));
    }

    #[test]
    fn test_check_call_contains_param_none() {
        let func = make_ident("foo");
        let args = vec![make_int(1)];
        assert!(!check_call_contains_param("x", &func, &args));
    }

    // ============== contains_param tests ==============

    #[test]
    fn test_contains_param_identifier() {
        let expr = make_ident("x");
        assert!(contains_param("x", &expr));
    }

    #[test]
    fn test_contains_param_binary() {
        let binary = make_binary(make_int(1), BinaryOp::Add, make_ident("x"));
        assert!(contains_param("x", &binary));
    }

    #[test]
    fn test_contains_param_block() {
        let block = make_block(vec![make_ident("x")]);
        assert!(contains_param("x", &block));
    }

    #[test]
    fn test_contains_param_call() {
        let call = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(contains_param("x", &call));
    }

    #[test]
    fn test_contains_param_not_found() {
        let expr = make_int(1);
        assert!(!contains_param("x", &expr));
    }

    // ============== is_param_used_as_array tests ==============

    #[test]
    fn test_is_param_used_as_array() {
        let index = make_index(make_ident("x"), make_int(0));
        assert!(is_param_used_as_array("x", &index));
    }

    #[test]
    fn test_is_param_used_as_array_nested() {
        let index = make_index(make_ident("x"), make_int(0));
        let block = make_block(vec![index]);
        assert!(is_param_used_as_array("x", &block));
    }

    #[test]
    fn test_is_param_used_as_array_not_array() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(!is_param_used_as_array("x", &binary));
    }

    // ============== is_param_used_with_len tests ==============

    #[test]
    fn test_is_param_used_with_len() {
        let call = make_call(make_ident("len"), vec![make_ident("x")]);
        assert!(is_param_used_with_len("x", &call));
    }

    #[test]
    fn test_is_param_used_with_len_nested() {
        let call = make_call(make_ident("len"), vec![make_ident("x")]);
        let block = make_block(vec![call]);
        assert!(is_param_used_with_len("x", &block));
    }

    #[test]
    fn test_is_param_used_with_len_not_len() {
        let call = make_call(make_ident("foo"), vec![make_ident("x")]);
        assert!(!is_param_used_with_len("x", &call));
    }

    // ============== is_param_used_as_index tests ==============

    #[test]
    fn test_is_param_used_as_index() {
        let index = make_index(make_ident("arr"), make_ident("x"));
        assert!(is_param_used_as_index("x", &index));
    }

    #[test]
    fn test_is_param_used_as_index_complex() {
        let idx = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        let index = make_index(make_ident("arr"), idx);
        assert!(is_param_used_as_index("x", &index));
    }

    #[test]
    fn test_is_param_used_as_index_not_index() {
        let index = make_index(make_ident("x"), make_int(0));
        assert!(!is_param_used_as_index("x", &index));
    }

    // ============== is_param_used_as_bool tests ==============

    #[test]
    fn test_is_param_used_as_bool_if_condition() {
        let if_expr = make_if(make_ident("x"), make_int(1), None);
        assert!(is_param_used_as_bool("x", &if_expr));
    }

    #[test]
    fn test_is_param_used_as_bool_while_condition() {
        let while_expr = make_while(make_ident("x"), make_int(1));
        assert!(is_param_used_as_bool("x", &while_expr));
    }

    #[test]
    fn test_is_param_used_as_bool_not() {
        let unary = make_unary(UnaryOp::Not, make_ident("x"));
        assert!(is_param_used_as_bool("x", &unary));
    }

    #[test]
    fn test_is_param_used_as_bool_and_left() {
        let binary = make_binary(make_ident("x"), BinaryOp::And, make_ident("y"));
        assert!(is_param_used_as_bool("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_bool_and_right() {
        let binary = make_binary(make_ident("y"), BinaryOp::And, make_ident("x"));
        assert!(is_param_used_as_bool("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_bool_or_left() {
        let binary = make_binary(make_ident("x"), BinaryOp::Or, make_ident("y"));
        assert!(is_param_used_as_bool("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_bool_or_right() {
        let binary = make_binary(make_ident("y"), BinaryOp::Or, make_ident("x"));
        assert!(is_param_used_as_bool("x", &binary));
    }

    #[test]
    fn test_is_param_used_as_bool_not_bool() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(!is_param_used_as_bool("x", &binary));
    }

    // ============== is_param_used_in_string_concat tests ==============

    #[test]
    fn test_is_param_used_in_string_concat_string_left() {
        let binary = make_binary(make_string("hello"), BinaryOp::Add, make_ident("x"));
        assert!(is_param_used_in_string_concat("x", &binary));
    }

    #[test]
    fn test_is_param_used_in_string_concat_string_right() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_string("world"));
        assert!(is_param_used_in_string_concat("x", &binary));
    }

    #[test]
    fn test_is_param_used_in_string_concat_no_string() {
        let binary = make_binary(make_ident("x"), BinaryOp::Add, make_int(1));
        assert!(!is_param_used_in_string_concat("x", &binary));
    }

    // ============== is_nested_array_access tests ==============

    #[test]
    fn test_is_nested_array_access() {
        let inner = make_index(make_ident("x"), make_int(0));
        let outer = make_index(inner, make_int(1));
        assert!(is_nested_array_access("x", &outer));
    }

    #[test]
    fn test_is_nested_array_access_not_nested() {
        let index = make_index(make_ident("x"), make_int(0));
        assert!(!is_nested_array_access("x", &index));
    }

    #[test]
    fn test_is_nested_array_access_wrong_param() {
        let inner = make_index(make_ident("y"), make_int(0));
        let outer = make_index(inner, make_int(1));
        assert!(!is_nested_array_access("x", &outer));
    }

    // ============== infer_param_type tests ==============

    #[test]
    fn test_infer_param_type_array() {
        let body = make_index(make_ident("x"), make_int(0));
        assert_eq!(infer_param_type("x", &body), Some("Vec<i32>"));
    }

    #[test]
    fn test_infer_param_type_nested_array() {
        let inner = make_index(make_ident("x"), make_int(0));
        let body = make_index(inner, make_int(1));
        assert_eq!(infer_param_type("x", &body), Some("Vec<Vec<i32>>"));
    }

    #[test]
    fn test_infer_param_type_with_len() {
        let body = make_call(make_ident("len"), vec![make_ident("x")]);
        assert_eq!(infer_param_type("x", &body), Some("Vec<i32>"));
    }

    #[test]
    fn test_infer_param_type_as_index() {
        let body = make_index(make_ident("arr"), make_ident("x"));
        assert_eq!(infer_param_type("x", &body), Some("i32"));
    }

    #[test]
    fn test_infer_param_type_no_inference() {
        let body = make_call(make_ident("foo"), vec![make_ident("y")]);
        assert_eq!(infer_param_type("x", &body), None);
    }
}
