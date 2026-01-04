//! Built-in function type inference for transpiler
//!
//! This module provides type inference for built-in functions,
//! determining parameter types and return types based on function signatures.

use crate::frontend::ast::{Expr, ExprKind, Literal};

/// Helper to check if an expression is a string literal
pub fn is_string_literal(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Literal(Literal::String(_)))
}

/// Get return type for built-in function
///
/// Maps built-in function names to their return types.
///
/// # Examples
/// ```ignore
/// assert_eq!(get_builtin_return_type("fs_read"), Some("String"));
/// assert_eq!(get_builtin_return_type("fs_exists"), Some("bool"));
/// assert_eq!(get_builtin_return_type("unknown"), None);
/// ```
pub fn get_builtin_return_type(func_name: &str) -> Option<&'static str> {
    match func_name {
        "fs_read" | "env_var" | "env_current_dir" | "http_get" | "http_post" | "http_put"
        | "http_delete" | "json_stringify" | "path_extension" | "path_filename" | "path_parent" => {
            Some("String")
        }
        "env_args" => Some("Vec<String>"),
        "fs_exists" => Some("bool"),
        "println" | "print" | "eprintln" | "eprint" => Some("()"),
        _ => None,
    }
}

/// Check function call for return type
fn check_call_for_return_type(func: &Expr) -> Option<&'static str> {
    if let ExprKind::Identifier(func_name) = &func.kind {
        get_builtin_return_type(func_name)
    } else {
        None
    }
}

/// Infer return type from built-in function calls in expression
///
/// Returns `Some(return_type)` if expression calls a built-in with known signature
/// Returns None if no built-in call detected
///
/// # Examples
/// ```ignore
/// // Expression: fs_read("path")
/// let result = infer_return_type_from_builtin_call(&call_expr);
/// assert_eq!(result, Some("String"));
/// ```
pub fn infer_return_type_from_builtin_call(expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Call { func, .. } => check_call_for_return_type(func),
        // ISSUE-103 FIX: Handle macro invocations (println!, format!, etc.)
        ExprKind::MacroInvocation { name, .. } => get_builtin_return_type(name),
        ExprKind::Block(exprs) => exprs.last().and_then(infer_return_type_from_builtin_call),
        ExprKind::If { then_branch, .. } => infer_return_type_from_builtin_call(then_branch),
        ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
            infer_return_type_from_builtin_call(body)
        }
        _ => None,
    }
}

/// Get parameter type hint from built-in function signature
///
/// Returns the expected type for a parameter at a given position in a built-in function call.
///
/// # Examples
/// ```ignore
/// assert_eq!(get_builtin_param_type("fs_read", 0), Some("&str"));
/// assert_eq!(get_builtin_param_type("http_get", 0), Some("&str"));
/// assert_eq!(get_builtin_param_type("println", 0), None);
/// ```
pub fn get_builtin_param_type(func_name: &str, arg_idx: usize) -> Option<&'static str> {
    match (func_name, arg_idx) {
        // File system functions: paths are &str
        (
            "fs_read" | "fs_write" | "fs_exists" | "fs_remove" | "fs_metadata" | "fs_create_dir"
            | "fs_read_dir",
            0,
        )
        | ("fs_copy" | "fs_rename", 0 | 1) => Some("&str"),

        // HTTP functions: URLs are &str
        ("http_get" | "http_post" | "http_put" | "http_delete", 0) => Some("&str"),

        // Environment/Path/JSON/Regex: strings are &str
        ("env_var" | "env_set_var" | "json_parse" | "regex_new", 0)
        | ("path_join" | "path_extension" | "path_filename" | "path_parent", 0 | 1)
        | ("regex_is_match" | "regex_find" | "regex_replace", 1) => Some("&str"),

        // Output/logging: generic (keep default)
        _ => None,
    }
}

/// Find parameter in argument list and return its type hint
fn find_param_in_args(param_name: &str, func_name: &str, args: &[Expr]) -> Option<&'static str> {
    for (arg_idx, arg) in args.iter().enumerate() {
        if let ExprKind::Identifier(arg_name) = &arg.kind {
            if arg_name == param_name {
                return get_builtin_param_type(func_name, arg_idx);
            }
        }
    }
    None
}

/// Check recursively in nested arguments
fn check_nested_args(param_name: &str, args: &[Expr]) -> Option<&'static str> {
    args.iter()
        .find_map(|arg| infer_param_type_from_builtin_usage(param_name, arg))
}

/// Check a single function call for param type
fn check_call_for_param_type(param_name: &str, func: &Expr, args: &[Expr]) -> Option<&'static str> {
    if let ExprKind::Identifier(func_name) = &func.kind {
        if let Some(ty) = find_param_in_args(param_name, func_name, args) {
            return Some(ty);
        }
    }
    check_nested_args(param_name, args)
}

/// Check if expression in let bindings
fn check_let_bindings(param_name: &str, value: &Expr, body: &Expr) -> Option<&'static str> {
    infer_param_type_from_builtin_usage(param_name, value)
        .or_else(|| infer_param_type_from_builtin_usage(param_name, body))
}

/// Check if expression in branches
fn check_if_branches(
    param_name: &str,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> Option<&'static str> {
    infer_param_type_from_builtin_usage(param_name, condition)
        .or_else(|| infer_param_type_from_builtin_usage(param_name, then_branch))
        .or_else(|| else_branch.and_then(|e| infer_param_type_from_builtin_usage(param_name, e)))
}

/// Infer parameter type from how it's used in built-in function calls
///
/// Analyzes the expression tree to find where a parameter is used as an argument
/// to a built-in function, and returns the expected type for that position.
///
/// # Examples
/// ```ignore
/// // In: fs_read(path)
/// // param "path" at position 0 -> "&str"
/// let ty = infer_param_type_from_builtin_usage("path", &body);
/// assert_eq!(ty, Some("&str"));
/// ```
pub fn infer_param_type_from_builtin_usage(param_name: &str, expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Call { func, args } => check_call_for_param_type(param_name, func, args),
        ExprKind::Block(exprs) => exprs
            .iter()
            .find_map(|e| infer_param_type_from_builtin_usage(param_name, e)),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => check_if_branches(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
            check_let_bindings(param_name, value, body)
        }
        ExprKind::Binary { left, right, .. } => infer_param_type_from_builtin_usage(param_name, left)
            .or_else(|| infer_param_type_from_builtin_usage(param_name, right)),
        ExprKind::Unary { operand, .. } => infer_param_type_from_builtin_usage(param_name, operand),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

    // ==================== Test Helpers ====================

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_lit(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn call(func_name: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Call {
            func: Box::new(ident(func_name)),
            args,
        })
    }

    fn block(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn if_expr(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        make_expr(ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        })
    }

    fn let_expr(name: &str, value: Expr, body: Expr) -> Expr {
        make_expr(ExprKind::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: false,
            type_annotation: None,
            else_block: None,
        })
    }

    fn bool_lit(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn macro_invocation(name: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::MacroInvocation {
            name: name.to_string(),
            args,
        })
    }

    fn binary(left: Expr, right: Expr) -> Expr {
        make_expr(ExprKind::Binary {
            left: Box::new(left),
            op: crate::frontend::ast::BinaryOp::Add,
            right: Box::new(right),
        })
    }

    fn unary(operand: Expr) -> Expr {
        make_expr(ExprKind::Unary {
            op: crate::frontend::ast::UnaryOp::Negate,
            operand: Box::new(operand),
        })
    }

    // ==================== is_string_literal tests ====================

    #[test]
    fn test_is_string_literal_true() {
        assert!(is_string_literal(&string_lit("hello")));
    }

    #[test]
    fn test_is_string_literal_empty_string() {
        assert!(is_string_literal(&string_lit("")));
    }

    #[test]
    fn test_is_string_literal_not_int() {
        assert!(!is_string_literal(&int_lit(42)));
    }

    #[test]
    fn test_is_string_literal_not_bool() {
        assert!(!is_string_literal(&bool_lit(true)));
    }

    #[test]
    fn test_is_string_literal_not_identifier() {
        assert!(!is_string_literal(&ident("x")));
    }

    // ==================== get_builtin_return_type tests ====================

    #[test]
    fn test_get_builtin_return_type_fs_read() {
        assert_eq!(get_builtin_return_type("fs_read"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_env_var() {
        assert_eq!(get_builtin_return_type("env_var"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_env_current_dir() {
        assert_eq!(get_builtin_return_type("env_current_dir"), Some("String"));
    }

    #[test]
    fn test_get_builtin_return_type_http_get() {
        assert_eq!(get_builtin_return_type("http_get"), Some("String"));
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
    fn test_get_builtin_return_type_env_args() {
        assert_eq!(get_builtin_return_type("env_args"), Some("Vec<String>"));
    }

    #[test]
    fn test_get_builtin_return_type_fs_exists() {
        assert_eq!(get_builtin_return_type("fs_exists"), Some("bool"));
    }

    #[test]
    fn test_get_builtin_return_type_println() {
        assert_eq!(get_builtin_return_type("println"), Some("()"));
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
    fn test_get_builtin_return_type_unknown() {
        assert_eq!(get_builtin_return_type("unknown_function"), None);
    }

    #[test]
    fn test_get_builtin_return_type_custom() {
        assert_eq!(get_builtin_return_type("my_custom_func"), None);
    }

    // ==================== get_builtin_param_type tests ====================

    #[test]
    fn test_get_builtin_param_type_fs_read() {
        assert_eq!(get_builtin_param_type("fs_read", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_write() {
        assert_eq!(get_builtin_param_type("fs_write", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_exists() {
        assert_eq!(get_builtin_param_type("fs_exists", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_remove() {
        assert_eq!(get_builtin_param_type("fs_remove", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_metadata() {
        assert_eq!(get_builtin_param_type("fs_metadata", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_create_dir() {
        assert_eq!(get_builtin_param_type("fs_create_dir", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_read_dir() {
        assert_eq!(get_builtin_param_type("fs_read_dir", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_copy_first() {
        assert_eq!(get_builtin_param_type("fs_copy", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_copy_second() {
        assert_eq!(get_builtin_param_type("fs_copy", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_rename_first() {
        assert_eq!(get_builtin_param_type("fs_rename", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_fs_rename_second() {
        assert_eq!(get_builtin_param_type("fs_rename", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_http_get() {
        assert_eq!(get_builtin_param_type("http_get", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_http_post() {
        assert_eq!(get_builtin_param_type("http_post", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_http_put() {
        assert_eq!(get_builtin_param_type("http_put", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_http_delete() {
        assert_eq!(get_builtin_param_type("http_delete", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_env_var() {
        assert_eq!(get_builtin_param_type("env_var", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_env_set_var() {
        assert_eq!(get_builtin_param_type("env_set_var", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_json_parse() {
        assert_eq!(get_builtin_param_type("json_parse", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_regex_new() {
        assert_eq!(get_builtin_param_type("regex_new", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_path_join_first() {
        assert_eq!(get_builtin_param_type("path_join", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_path_join_second() {
        assert_eq!(get_builtin_param_type("path_join", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_path_extension() {
        assert_eq!(get_builtin_param_type("path_extension", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_path_filename() {
        assert_eq!(get_builtin_param_type("path_filename", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_path_parent() {
        assert_eq!(get_builtin_param_type("path_parent", 0), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_regex_is_match() {
        assert_eq!(get_builtin_param_type("regex_is_match", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_regex_find() {
        assert_eq!(get_builtin_param_type("regex_find", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_regex_replace() {
        assert_eq!(get_builtin_param_type("regex_replace", 1), Some("&str"));
    }

    #[test]
    fn test_get_builtin_param_type_println_no_type() {
        assert_eq!(get_builtin_param_type("println", 0), None);
    }

    #[test]
    fn test_get_builtin_param_type_unknown() {
        assert_eq!(get_builtin_param_type("unknown", 0), None);
    }

    #[test]
    fn test_get_builtin_param_type_wrong_arg_idx() {
        assert_eq!(get_builtin_param_type("fs_read", 1), None);
    }

    // ==================== infer_return_type_from_builtin_call tests ====================

    #[test]
    fn test_infer_return_type_call_fs_read() {
        let expr = call("fs_read", vec![string_lit("path")]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("String"));
    }

    #[test]
    fn test_infer_return_type_call_fs_exists() {
        let expr = call("fs_exists", vec![string_lit("path")]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("bool"));
    }

    #[test]
    fn test_infer_return_type_call_println() {
        let expr = call("println", vec![string_lit("hello")]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("()"));
    }

    #[test]
    fn test_infer_return_type_call_env_args() {
        let expr = call("env_args", vec![]);
        assert_eq!(
            infer_return_type_from_builtin_call(&expr),
            Some("Vec<String>")
        );
    }

    #[test]
    fn test_infer_return_type_call_unknown() {
        let expr = call("my_function", vec![int_lit(42)]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), None);
    }

    #[test]
    fn test_infer_return_type_macro_println() {
        let expr = macro_invocation("println", vec![string_lit("hello")]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("()"));
    }

    #[test]
    fn test_infer_return_type_macro_unknown() {
        let expr = macro_invocation("custom_macro", vec![]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), None);
    }

    #[test]
    fn test_infer_return_type_block_last_expr() {
        let expr = block(vec![int_lit(1), call("fs_read", vec![string_lit("path")])]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("String"));
    }

    #[test]
    fn test_infer_return_type_block_empty() {
        let expr = block(vec![]);
        assert_eq!(infer_return_type_from_builtin_call(&expr), None);
    }

    #[test]
    fn test_infer_return_type_if_then_branch() {
        let expr = if_expr(
            bool_lit(true),
            call("fs_read", vec![string_lit("path")]),
            None,
        );
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("String"));
    }

    #[test]
    fn test_infer_return_type_let_body() {
        let expr = let_expr(
            "x",
            int_lit(5),
            call("fs_exists", vec![string_lit("path")]),
        );
        assert_eq!(infer_return_type_from_builtin_call(&expr), Some("bool"));
    }

    #[test]
    fn test_infer_return_type_other_expr() {
        let expr = int_lit(42);
        assert_eq!(infer_return_type_from_builtin_call(&expr), None);
    }

    #[test]
    fn test_infer_return_type_identifier() {
        let expr = ident("x");
        assert_eq!(infer_return_type_from_builtin_call(&expr), None);
    }

    // ==================== infer_param_type_from_builtin_usage tests ====================

    #[test]
    fn test_infer_param_type_fs_read_param() {
        let expr = call("fs_read", vec![ident("path")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_http_get_param() {
        let expr = call("http_get", vec![ident("url")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("url", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_env_var_param() {
        let expr = call("env_var", vec![ident("key")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("key", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_fs_copy_both_params() {
        let expr = call("fs_copy", vec![ident("src"), ident("dst")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("src", &expr),
            Some("&str")
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("dst", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_not_found() {
        let expr = call("fs_read", vec![string_lit("literal")]);
        assert_eq!(infer_param_type_from_builtin_usage("x", &expr), None);
    }

    #[test]
    fn test_infer_param_type_unknown_function() {
        let expr = call("custom_func", vec![ident("param")]);
        assert_eq!(infer_param_type_from_builtin_usage("param", &expr), None);
    }

    #[test]
    fn test_infer_param_type_in_block() {
        let expr = block(vec![
            int_lit(1),
            call("fs_read", vec![ident("path")]),
        ]);
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_if_condition() {
        let expr = if_expr(
            call("fs_exists", vec![ident("path")]),
            int_lit(1),
            None,
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_if_then() {
        let expr = if_expr(
            bool_lit(true),
            call("fs_read", vec![ident("path")]),
            None,
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_if_else() {
        let expr = if_expr(
            bool_lit(false),
            int_lit(0),
            Some(call("fs_read", vec![ident("path")])),
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_let_value() {
        let expr = let_expr(
            "result",
            call("fs_read", vec![ident("path")]),
            ident("result"),
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_let_body() {
        let expr = let_expr(
            "x",
            int_lit(5),
            call("fs_read", vec![ident("path")]),
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_binary_left() {
        let expr = binary(
            call("fs_read", vec![ident("path")]),
            string_lit("suffix"),
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_binary_right() {
        let expr = binary(
            string_lit("prefix"),
            call("fs_read", vec![ident("path")]),
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_in_unary() {
        // Unusual but test unary path
        let expr = unary(call("fs_read", vec![ident("path")]));
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_nested_call() {
        // outer(fs_read(path))
        let inner = call("fs_read", vec![ident("path")]);
        let outer = call("outer", vec![inner]);
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &outer),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_deeply_nested() {
        // block { let x = 5; if true { fs_read(path) } else { "default" } }
        let inner_if = if_expr(
            bool_lit(true),
            call("fs_read", vec![ident("path")]),
            Some(string_lit("default")),
        );
        let expr = let_expr("x", int_lit(5), inner_if);
        assert_eq!(
            infer_param_type_from_builtin_usage("path", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_regex_pattern() {
        // regex_is_match(regex, text) - text at idx 1 is &str
        let expr = call("regex_is_match", vec![ident("regex"), ident("text")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("text", &expr),
            Some("&str")
        );
    }

    #[test]
    fn test_infer_param_type_path_join_both() {
        let expr = call("path_join", vec![ident("base"), ident("child")]);
        assert_eq!(
            infer_param_type_from_builtin_usage("base", &expr),
            Some("&str")
        );
        assert_eq!(
            infer_param_type_from_builtin_usage("child", &expr),
            Some("&str")
        );
    }
}
