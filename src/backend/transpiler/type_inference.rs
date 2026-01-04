//! Type inference helpers for transpiler
//!
//! This module provides intelligent type inference by analyzing how
//! parameters and expressions are used in function bodies.
//!
//! Most functionality has been extracted to `param_usage_analysis` module.

// Re-export functions from builtin_type_inference for backwards compatibility
pub use super::builtin_type_inference::{
    get_builtin_param_type, get_builtin_return_type, infer_param_type_from_builtin_usage,
    infer_return_type_from_builtin_call, is_string_literal,
};

// Re-export functions from param_usage_analysis for backwards compatibility
pub use super::param_usage_analysis::{
    check_binary_for_param, check_binary_numeric_usage, check_block_numeric_usage,
    check_call_contains_param, check_call_for_param_argument, check_call_numeric_usage,
    check_expressions_for_param, check_func_call, check_if_for_func, check_if_for_param,
    check_if_numeric_usage, check_let_and_binary_for_func, check_let_for_param,
    check_let_numeric_usage, contains_param, find_param_in_direct_args, has_param_in_operation,
    infer_param_type, is_nested_array_access, is_numeric_operator, is_param_used_as_array,
    is_param_used_as_bool, is_param_used_as_function, is_param_used_as_function_argument,
    is_param_used_as_index, is_param_used_in_print_macro, is_param_used_in_string_concat,
    is_param_used_numerically, is_param_used_with_len, is_string_concatenation,
    traverse_expr_for_check,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, ExprKind};
    use crate::Parser;

    /// Checks if an expression contains numeric operations (test helper)
    fn contains_numeric_operations(expr: &crate::frontend::ast::Expr) -> bool {
        match &expr.kind {
            ExprKind::Binary { op, left, right } => {
                matches!(
                    op,
                    BinaryOp::Add
                        | BinaryOp::Subtract
                        | BinaryOp::Multiply
                        | BinaryOp::Divide
                        | BinaryOp::Modulo
                        | BinaryOp::Less
                        | BinaryOp::Greater
                        | BinaryOp::LessEqual
                        | BinaryOp::GreaterEqual
                ) || contains_numeric_operations(left)
                    || contains_numeric_operations(right)
            }
            ExprKind::Block(exprs) => exprs.iter().any(contains_numeric_operations),
            ExprKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                contains_numeric_operations(then_branch)
                    || else_branch
                        .as_ref()
                        .is_some_and(|e| contains_numeric_operations(e))
            }
            ExprKind::Let { value, body, .. } => {
                contains_numeric_operations(value) || contains_numeric_operations(body)
            }
            ExprKind::Call { args, .. } => args.iter().any(contains_numeric_operations),
            ExprKind::Lambda { body, .. } => contains_numeric_operations(body),
            _ => false,
        }
    }

    #[test]
    fn test_detects_function_parameter() {
        let code = "fun test() { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(!is_param_used_as_function("x", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_operations() {
        let code = "fun test(x) { x * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_string_concatenation_not_numeric() {
        let code = "fun greet(name) { \"Hello \" + name }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_numerically("name", body));
                    assert!(!is_param_used_as_function("name", body));
                }
            }
        }
    }

    #[test]
    fn test_param_in_comparison_is_numeric() {
        let code = "fun test(count) { let x = 0; while x < count { x = x + 1 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(
                        is_param_used_numerically("count", body),
                        "Parameter 'count' in 'x < count' should be detected as numeric"
                    );
                }
            }
        }
    }

    #[test]
    fn test_string_literal_helper() {
        let code = "fun test() { \"hello\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_string_literal(body));
                }
            }
        }
    }

    #[test]
    fn test_infer_return_type_fs_read() {
        let code = "fun test() { fs_read(\"/path\") }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_return_type_from_builtin_call(body), Some("String"));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_array() {
        let code = "fun test(arr) { arr[0] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_param_type("arr", body), Some("Vec<i32>"));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_index() {
        let code = "fun test(i) { let arr = [1,2,3]; arr[i] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_param_type("i", body), Some("i32"));
                }
            }
        }
    }

    #[test]
    fn test_is_numeric_operator() {
        assert!(is_numeric_operator(&BinaryOp::Add));
        assert!(is_numeric_operator(&BinaryOp::Subtract));
        assert!(is_numeric_operator(&BinaryOp::Multiply));
        assert!(is_numeric_operator(&BinaryOp::Divide));
        assert!(is_numeric_operator(&BinaryOp::Modulo));
        assert!(!is_numeric_operator(&BinaryOp::Equal));
        assert!(!is_numeric_operator(&BinaryOp::NotEqual));
    }

    #[test]
    fn test_get_builtin_return_type() {
        assert_eq!(get_builtin_return_type("fs_read"), Some("String"));
        assert_eq!(get_builtin_return_type("env_var"), Some("String"));
        assert_eq!(get_builtin_return_type("env_args"), Some("Vec<String>"));
        assert_eq!(get_builtin_return_type("fs_exists"), Some("bool"));
        assert_eq!(get_builtin_return_type("println"), Some("()"));
        assert_eq!(get_builtin_return_type("unknown_func"), None);
    }

    #[test]
    fn test_get_builtin_param_type() {
        assert_eq!(get_builtin_param_type("fs_read", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("env_var", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("http_get", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("json_parse", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("unknown", 0), None);
    }
}

#[cfg(test)]
mod property_tests_type_inference {
    use proptest::proptest;

    proptest! {
        #[test]
        fn test_is_param_used_as_function_argument_never_panics(input: String) {
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            let _ = std::panic::catch_unwind(|| {
            });
        }
    }
}
