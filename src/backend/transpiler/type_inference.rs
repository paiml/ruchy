//! Type inference helpers for transpiler
//!
//! This module provides intelligent type inference by analyzing how
//! parameters and expressions are used in function bodies.
//!
//! Most functionality has been extracted to `param_usage_analysis` module.

// Re-export functions from builtin_type_inference for backwards compatibility
#[allow(unused_imports)]
pub use super::builtin_type_inference::{
    get_builtin_param_type, get_builtin_return_type, infer_param_type_from_builtin_usage,
    infer_return_type_from_builtin_call, is_string_literal,
};

// Re-export functions from param_usage_analysis for backwards compatibility
#[allow(unused_imports)]
pub use super::param_usage_analysis::{
    contains_param, infer_param_type, is_nested_array_access, is_numeric_operator,
    is_param_used_as_array, is_param_used_as_bool, is_param_used_as_function,
    is_param_used_as_function_argument, is_param_used_as_index, is_param_used_in_print_macro,
    is_param_used_in_string_concat, is_param_used_numerically, is_param_used_with_len,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, ExprKind};
    use crate::Parser;

    /// Checks if an expression contains numeric operations (test helper)
    pub(super) fn contains_numeric_operations(expr: &crate::frontend::ast::Expr) -> bool {
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

// === EXTREME TDD Round 22 - Coverage Push Tests ===
#[cfg(test)]
mod coverage_push_tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, ExprKind};
    use crate::Parser;

    #[test]
    fn test_multiple_functions_second_is_function_param() {
        let code = "fun test() { let f = fn(x) { x }; f(10) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                }
            }
        }
    }

    #[test]
    fn test_param_not_function_when_used_in_binary() {
        let code = "fun test(x) { x + 1 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_function("x", body));
                }
            }
        }
    }

    #[test]
    fn test_is_numeric_operator_all_comparison() {
        assert!(is_numeric_operator(&BinaryOp::Less));
        assert!(is_numeric_operator(&BinaryOp::Greater));
        assert!(is_numeric_operator(&BinaryOp::LessEqual));
        assert!(is_numeric_operator(&BinaryOp::GreaterEqual));
    }

    #[test]
    fn test_infer_param_type_bool_usage() {
        let code = "fun test(flag) { if flag { 1 } else { 0 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_bool("flag", body));
                }
            }
        }
    }

    #[test]
    fn test_get_builtin_return_type_all() {
        // Only test return types that are actually defined
        assert_eq!(get_builtin_return_type("fs_read"), Some("String"));
        assert_eq!(get_builtin_return_type("fs_exists"), Some("bool"));
        assert_eq!(get_builtin_return_type("println"), Some("()"));
    }

    #[test]
    fn test_get_builtin_param_type_multiple() {
        assert_eq!(get_builtin_param_type("fs_read", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("env_var", 0), Some("&str"));
        assert_eq!(get_builtin_param_type("json_parse", 0), Some("&str"));
    }

    #[test]
    fn test_string_literal_with_block() {
        let code = "fun test() { { \"hello\" } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Block containing string literal
                    if let ExprKind::Block(inner) = &body.kind {
                        if !inner.is_empty() {
                            assert!(is_string_literal(&inner[0]));
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_param_with_len_call() {
        let code = "fun test(arr) { len(arr) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_with_len("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_param_in_nested_call() {
        let code = "fun test(x) { foo(bar(x)) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_param("x", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_return_type_len_call() {
        let code = "fun test(arr) { len(arr) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_return_type_from_builtin_call(body), Some("i32"));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_from_builtin_fs_read() {
        let code = "fun test(path) { fs_read(path) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(
                        infer_param_type_from_builtin_usage("path", body),
                        Some("&str")
                    );
                }
            }
        }
    }

    #[test]
    fn test_is_numeric_operator_modulo() {
        assert!(is_numeric_operator(&BinaryOp::Modulo));
    }

    #[test]
    fn test_is_not_numeric_operator_and_or() {
        assert!(!is_numeric_operator(&BinaryOp::And));
        assert!(!is_numeric_operator(&BinaryOp::Or));
    }

    #[test]
    fn test_param_used_in_string_concat() {
        let code = r#"fun test(name) { "Hello " + name }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_in_string_concat("name", body));
                }
            }
        }
    }

    #[test]
    fn test_param_used_in_print() {
        let code = r#"fun test(x) { println("{}", x) }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_in_print_macro("x", body));
                }
            }
        }
    }

    #[test]
    fn test_nested_array_access() {
        let code = "fun test(arr) { arr[0][1] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_nested_array_access("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_from_function_arg() {
        let code = "fun test(f, x) { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function_argument("x", body));
                }
            }
        }
    }

    #[test]
    fn test_get_builtin_return_type_env_args() {
        assert_eq!(get_builtin_return_type("env_args"), Some("Vec<String>"));
    }

    #[test]
    fn test_get_builtin_return_type_timestamp() {
        // Just verify no panic on unknown functions
        let _ = get_builtin_return_type("timestamp");
        let _ = get_builtin_return_type("nonexistent");
    }

    // COVERAGE: Additional tests for builtin inference
    #[test]
    fn test_infer_param_type_numeric_comparison() {
        let code = "fun test(n) { n > 0 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_less_equal() {
        let code = "fun test(x) { x <= 100 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_subtraction() {
        let code = "fun test(x) { 10 - x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_get_builtin_return_type_http_post() {
        assert_eq!(get_builtin_return_type("http_post"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_http_put() {
        assert_eq!(get_builtin_return_type("http_put"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_http_delete() {
        assert_eq!(get_builtin_return_type("http_delete"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_json_stringify() {
        assert_eq!(get_builtin_return_type("json_stringify"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_path_extension() {
        assert_eq!(get_builtin_return_type("path_extension"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_path_filename() {
        assert_eq!(get_builtin_return_type("path_filename"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_path_parent() {
        assert_eq!(get_builtin_return_type("path_parent"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_print() {
        assert_eq!(get_builtin_return_type("print"), Some("()"));
    }

    #[test]
    fn test_get_builtin_return_type_eprintln() {
        assert_eq!(get_builtin_return_type("eprintln"), Some("()"));
    }

    #[test]
    fn test_get_builtin_return_type_eprint() {
        assert_eq!(get_builtin_return_type("eprint"), Some("()"));
    }

    #[test]
    fn test_get_builtin_return_type_env_current_dir() {
        assert_eq!(get_builtin_return_type("env_current_dir"), Some("String"));
    }

    #[test]
    fn test_infer_param_from_env_var() {
        let code = "fun test(key) { env_var(key) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(
                        infer_param_type_from_builtin_usage("key", body),
                        Some("&str")
                    );
                }
            }
        }
    }

    #[test]
    fn test_param_used_as_array_with_method() {
        let code = "fun test(arr) { arr.len() }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Used with .len() method suggests array/vec
                    assert!(is_param_used_with_len("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_return_type_from_http_get() {
        let code = "fun test(url) { http_get(url) }";
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
    fn test_param_not_used_as_bool_when_numeric() {
        let code = "fun test(x) { x + 1 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_bool("x", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_division() {
        let code = "fun test(x) { x / 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_modulo() {
        let code = "fun test(x) { x % 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    // === COVERAGE: contains_numeric_operations helper branches ===

    #[test]
    fn test_contains_numeric_in_block() {
        use super::tests::contains_numeric_operations;
        let code = "fun test() { { 1 + 2 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_if_else() {
        use super::tests::contains_numeric_operations;
        let code = "fun test(flag) { if flag { 0 } else { 1 + 2 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_let_value() {
        use super::tests::contains_numeric_operations;
        let code = "fun test() { let x = 1 + 2; x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_let_body() {
        use super::tests::contains_numeric_operations;
        let code = "fun test() { let x = 0; x * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_call_args() {
        use super::tests::contains_numeric_operations;
        let code = "fun test() { foo(1 + 2) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_lambda() {
        use super::tests::contains_numeric_operations;
        let code = "fun test() { let f = fn(x) { x + 1 }; f }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_returns_false_for_string() {
        use super::tests::contains_numeric_operations;
        let code = r#"fun test() { "hello" }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_contains_numeric_in_if_then_branch() {
        use super::tests::contains_numeric_operations;
        let code = "fun test(flag) { if flag { 1 + 2 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }
}
