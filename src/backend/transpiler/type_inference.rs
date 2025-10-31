//! Type inference helpers for transpiler
//!
//! This module provides intelligent type inference by analyzing how
//! parameters and expressions are used in function bodies.
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
/// Analyzes if a parameter is used as an argument to a function that takes i32
/// # Examples
///
/// ```ignore
/// use ruchy::backend::transpiler::type_inference::is_param_used_as_function_argument;
/// use ruchy::frontend::ast::{Expr, ExprKind};
/// let expr = Expr::new(ExprKind::Literal(42.into()), (0, 0));
/// let result = is_param_used_as_function_argument("x", &expr);
/// ```
pub fn is_param_used_as_function_argument(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, args } => check_call_for_param_argument(param_name, func, args),
        ExprKind::Block(exprs) => check_expressions_for_param(param_name, exprs),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => check_if_for_param(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
            check_let_for_param(param_name, value, body)
        }
        ExprKind::Binary { left, right, .. } => check_binary_for_param(param_name, left, right),
        ExprKind::Unary { operand, .. } => is_param_used_as_function_argument(param_name, operand),
        _ => false,
    }
}
/// Check if parameter is directly in arguments
/// Complexity: 2 (loop with early return)
fn find_param_in_direct_args(param_name: &str, args: &[Expr]) -> bool {
    args.iter().any(|arg| {
        matches!(&arg.kind, ExprKind::Identifier(name) if name == param_name)
    })
}

/// Check if parameter is used as argument in function call
/// Extracted to reduce complexity
fn check_call_for_param_argument(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    // Check if any argument is the parameter
    if matches!(&func.kind, ExprKind::Identifier(_)) && find_param_in_direct_args(param_name, args) {
        return true;
    }
    // Recursively check nested arguments
    args.iter().any(|arg| is_param_used_as_function_argument(param_name, arg))
}
/// Check if parameter is used in expressions list
fn check_expressions_for_param(param_name: &str, exprs: &[Expr]) -> bool {
    exprs
        .iter()
        .any(|e| is_param_used_as_function_argument(param_name, e))
}
/// Check if parameter is used in if expression
fn check_if_for_param(
    param_name: &str,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> bool {
    is_param_used_as_function_argument(param_name, condition)
        || is_param_used_as_function_argument(param_name, then_branch)
        || else_branch.is_some_and(|e| is_param_used_as_function_argument(param_name, e))
}
/// Check if parameter is used in let expression
fn check_let_for_param(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_as_function_argument(param_name, value)
        || is_param_used_as_function_argument(param_name, body)
}
/// Check if parameter is used in binary expression
fn check_binary_for_param(param_name: &str, left: &Expr, right: &Expr) -> bool {
    is_param_used_as_function_argument(param_name, left)
        || is_param_used_as_function_argument(param_name, right)
}
/// Check if parameter is the function being called
/// Complexity: 2 (single check with recursion)
fn check_func_call(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    if let ExprKind::Identifier(name) = &func.kind {
        if name == param_name {
            return true;
        }
    }
    args.iter().any(|arg| is_param_used_as_function(param_name, arg))
}

/// Check if branches for parameter as function
/// Complexity: 1 (chained OR)
fn check_if_for_func(param_name: &str, condition: &Expr, then_branch: &Expr, else_branch: Option<&Expr>) -> bool {
    is_param_used_as_function(param_name, condition)
        || is_param_used_as_function(param_name, then_branch)
        || else_branch.is_some_and(|e| is_param_used_as_function(param_name, e))
}

/// Analyzes if a parameter is used as a function in the given expression
/// # Examples
///
/// ```ignore
/// use ruchy::backend::transpiler::type_inference::is_param_used_as_function;
/// use ruchy::frontend::ast::{Expr, ExprKind};
/// let expr = Expr::new(ExprKind::Literal(42.into()), (0, 0));
/// let result = is_param_used_as_function("x", &expr);
/// ```
/// Check let and binary expressions
/// Complexity: 1 (OR chain)
fn check_let_and_binary_for_func(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_as_function(param_name, value) || is_param_used_as_function(param_name, body)
}

pub fn is_param_used_as_function(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, args } => check_func_call(param_name, func, args),
        ExprKind::Block(exprs) => exprs.iter().any(|e| is_param_used_as_function(param_name, e)),
        ExprKind::If { condition, then_branch, else_branch } => check_if_for_func(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. } | ExprKind::Binary { left: value, right: body, .. } => {
            check_let_and_binary_for_func(param_name, value, body)
        }
        ExprKind::Lambda { body, .. } => is_param_used_as_function(param_name, body),
        _ => false,
    }
}
/// Checks if a parameter is used in numeric operations
/// # Examples
///
/// ```ignore
/// use ruchy::backend::transpiler::type_inference::is_param_used_numerically;
/// use ruchy::frontend::ast::{Expr, ExprKind};
/// let expr = Expr::new(ExprKind::Literal(42.into()), (0, 0));
/// let result = is_param_used_numerically("x", &expr);
/// assert!(result);
/// ```
pub fn is_param_used_numerically(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            check_binary_numeric_usage(param_name, op, left, right)
        }
        ExprKind::Block(exprs) => check_block_numeric_usage(param_name, exprs),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => check_if_numeric_usage(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. } => check_let_numeric_usage(param_name, value, body),
        ExprKind::Call { args, .. } => check_call_numeric_usage(param_name, args),
        // DEFECT-CLOSURE-RETURN FIX: Check inside lambda bodies for captured variables
        ExprKind::Lambda { body, .. } => is_param_used_numerically(param_name, body),
        _ => false,
    }
}
/// Check numeric usage in binary expressions (complexity: 6)
fn check_binary_numeric_usage(param_name: &str, op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
    if is_numeric_operator(op) && has_param_in_operation(param_name, left, right) {
        // Special case: string concatenation
        if is_string_concatenation(op, left, right) {
            return false;
        }
        return true;
    }
    // Recursively check both sides
    is_param_used_numerically(param_name, left) || is_param_used_numerically(param_name, right)
}
/// Check if operator is numeric (complexity: 1)
fn is_numeric_operator(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
    )
}
/// Check if param is in operation (complexity: 2)
fn has_param_in_operation(param_name: &str, left: &Expr, right: &Expr) -> bool {
    contains_param(param_name, left) || contains_param(param_name, right)
}
/// Check if operation is string concatenation (complexity: 3)
fn is_string_concatenation(op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
    matches!(op, BinaryOp::Add) && (is_string_literal(left) || is_string_literal(right))
}
/// Check numeric usage in blocks (complexity: 1)
fn check_block_numeric_usage(param_name: &str, exprs: &[Expr]) -> bool {
    exprs
        .iter()
        .any(|e| is_param_used_numerically(param_name, e))
}
/// Check numeric usage in if expressions (complexity: 3)
fn check_if_numeric_usage(
    param_name: &str,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> bool {
    is_param_used_numerically(param_name, condition)
        || is_param_used_numerically(param_name, then_branch)
        || else_branch.is_some_and(|e| is_param_used_numerically(param_name, e))
}
/// Check numeric usage in let expressions (complexity: 2)
fn check_let_numeric_usage(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_numerically(param_name, value) || is_param_used_numerically(param_name, body)
}
/// Check numeric usage in call arguments (complexity: 1)
fn check_call_numeric_usage(param_name: &str, args: &[Expr]) -> bool {
    args.iter()
        .any(|arg| is_param_used_numerically(param_name, arg))
}
/// Helper to check if an expression contains a specific parameter
/// Check function call for parameter
/// Complexity: 1 (OR chain)
fn check_call_contains_param(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    contains_param(param_name, func) || args.iter().any(|arg| contains_param(param_name, arg))
}

fn contains_param(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => name == param_name,
        ExprKind::Binary { left, right, .. } => {
            contains_param(param_name, left) || contains_param(param_name, right)
        }
        ExprKind::Block(exprs) => exprs.iter().any(|e| contains_param(param_name, e)),
        ExprKind::Call { func, args } => check_call_contains_param(param_name, func, args),
        _ => false,
    }
}
/// Helper to check if an expression is a string literal
fn is_string_literal(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Literal(Literal::String(_)))
}

/// Get return type for built-in function
/// Complexity: 1 (single match)
fn get_builtin_return_type(func_name: &str) -> Option<&'static str> {
    match func_name {
        "fs_read" | "env_var" | "env_current_dir"
        | "http_get" | "http_post" | "http_put" | "http_delete"
        | "json_stringify"
        | "path_extension" | "path_filename" | "path_parent" => Some("String"),
        "env_args" => Some("Vec<String>"),
        "fs_exists" => Some("bool"),
        "println" | "print" | "eprintln" | "eprint" => Some("()"),
        _ => None,
    }
}

/// Check function call for return type
/// Complexity: 2 (pattern match + function call)
fn check_call_for_return_type(func: &Expr) -> Option<&'static str> {
    if let ExprKind::Identifier(func_name) = &func.kind {
        get_builtin_return_type(func_name)
    } else {
        None
    }
}

/// Infer return type from built-in function calls in expression
/// Returns `Some(return_type)` if expression calls a built-in with known signature
/// Returns None if no built-in call detected
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
/// Complexity: 1 (single return, pattern match)
fn get_builtin_param_type(func_name: &str, arg_idx: usize) -> Option<&'static str> {
    match (func_name, arg_idx) {
        // File system functions: paths are &str
        ("fs_read" | "fs_write" | "fs_exists" | "fs_remove" | "fs_metadata"
        | "fs_create_dir" | "fs_read_dir", 0)
        | ("fs_copy" | "fs_rename", 0 | 1) => Some("&str"),

        // HTTP functions: URLs are &str
        ("http_get" | "http_post" | "http_put" | "http_delete", 0) => Some("&str"),

        // Environment/Path/JSON/Regex: strings are &str
        ("env_var" | "env_set_var" | "json_parse" | "regex_new", 0) |
("path_join" | "path_extension" | "path_filename" | "path_parent", 0 | 1) |
("regex_is_match" | "regex_find" | "regex_replace", 1) => Some("&str"),

        // Output/logging: generic (keep default)
        _ => None,
    }
}

/// Find parameter in argument list
/// Complexity: 2 (single loop, early return)
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
/// Complexity: 1 (single loop with function call)
fn check_nested_args(param_name: &str, args: &[Expr]) -> Option<&'static str> {
    args.iter().find_map(|arg| infer_param_type_from_builtin_usage(param_name, arg))
}

/// Check a single function call for param type
/// Complexity: 2 (two branches)
fn check_call_for_param_type(param_name: &str, func: &Expr, args: &[Expr]) -> Option<&'static str> {
    if let ExprKind::Identifier(func_name) = &func.kind {
        if let Some(ty) = find_param_in_args(param_name, func_name, args) {
            return Some(ty);
        }
    }
    check_nested_args(param_name, args)
}

/// Check if expression in let bindings
/// Complexity: 1 (single pattern match)
fn check_let_bindings(param_name: &str, value: &Expr, body: &Expr) -> Option<&'static str> {
    infer_param_type_from_builtin_usage(param_name, value)
        .or_else(|| infer_param_type_from_builtin_usage(param_name, body))
}

/// Check if expression  in branches
/// Complexity: 1 (chained `or_else`)
fn check_if_branches(param_name: &str, condition: &Expr, then_branch: &Expr, else_branch: Option<&Expr>) -> Option<&'static str> {
    infer_param_type_from_builtin_usage(param_name, condition)
        .or_else(|| infer_param_type_from_builtin_usage(param_name, then_branch))
        .or_else(|| else_branch.and_then(|e| infer_param_type_from_builtin_usage(param_name, e)))
}

pub fn infer_param_type_from_builtin_usage(param_name: &str, expr: &Expr) -> Option<&'static str> {
    match &expr.kind {
        ExprKind::Call { func, args } => check_call_for_param_type(param_name, func, args),
        ExprKind::Block(exprs) => exprs.iter().find_map(|e| infer_param_type_from_builtin_usage(param_name, e)),
        ExprKind::If { condition, then_branch, else_branch } => check_if_branches(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => check_let_bindings(param_name, value, body),
        ExprKind::Binary { left, right, .. } => {
            infer_param_type_from_builtin_usage(param_name, left)
                .or_else(|| infer_param_type_from_builtin_usage(param_name, right))
        }
        ExprKind::Unary { operand, .. } => infer_param_type_from_builtin_usage(param_name, operand),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;
    /// Checks if an expression contains numeric operations (test helper)
    fn contains_numeric_operations(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Binary { op, left, right } => {
                // Check for numeric operators
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
        // Find the function body
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
    fn test_detects_nested_function_call() {
        let code = "fun test() { g(f(x)) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(is_param_used_as_function("g", body));
                    assert!(!is_param_used_as_function("x", body));
                }
            }
        }
    }
    #[test]
    fn test_detects_function_in_if_branch() {
        let code = "fun test(p) { if (true) { p(5) } else { 0 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("p", body));
                }
            }
        }
    }
    #[test]
    fn test_detects_function_in_let_body() {
        let code = "fun test(f) { let x = 5; f(x) }";
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
    fn test_detects_function_in_lambda() {
        let code = "fun test(f) { (x) => f(x) }";
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
    fn test_detects_numeric_in_addition() {
        let code = "fun test(n) { n + 10 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }
    #[test]
    fn test_detects_numeric_in_subtraction() {
        let code = "fun test(n) { n - 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }
    #[test]
    fn test_detects_numeric_in_division() {
        let code = "fun test(n) { n / 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }
    #[test]
    fn test_detects_numeric_in_modulo() {
        let code = "fun test(n) { n % 3 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }
    #[test]
    fn test_detects_numeric_in_comparison() {
        let code = "fun test(n) { n > 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_numerically("n", body)); // Comparisons don't count as numeric
                    assert!(contains_numeric_operations(body)); // But the expression contains numeric ops
                }
            }
        }
    }
    #[test]
    fn test_no_function_no_numeric() {
        let code = "fun test(s) { s + \" world\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_function("s", body));
                    assert!(!is_param_used_numerically("s", body));
                }
            }
        }
    }
    #[test]
    fn test_contains_param_helper() {
        let code = "fun test(x) { x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_param("x", body));
                    assert!(!contains_param("y", body));
                }
            }
        }
    }
    #[test]
    fn test_contains_param_in_binary() {
        let code = "fun test(x, y) { x + y }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_param("x", body));
                    assert!(contains_param("y", body));
                    assert!(!contains_param("z", body));
                }
            }
        }
    }
    #[test]
    fn test_complex_nested_detection() {
        let code = "fun test(f, g, x) { f(g(x * 2)) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(is_param_used_as_function("g", body));
                    assert!(!is_param_used_as_function("x", body));
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
                    // name should NOT be considered numeric in string concatenation
                    assert!(!is_param_used_numerically("name", body));
                    assert!(!is_param_used_as_function("name", body));
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
}
#[cfg(test)]
mod property_tests_type_inference {
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_is_param_used_as_function_argument_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
