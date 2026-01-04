//! Parameter usage analysis for type inference
//!
//! This module provides functions for analyzing how parameters are used
//! in function bodies to infer their types.

use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

// Re-export from builtin_type_inference for backwards compatibility
pub use super::builtin_type_inference::{
    infer_param_type_from_builtin_usage, is_string_literal,
};

/// Generic AST traversal for parameter usage checks.
///
/// The `check` closure returns:
/// - `Some(true)` if the check succeeded (stop traversal, return true)
/// - `Some(false)` if the check explicitly failed (stop traversal, return false)
/// - `None` to continue traversal into child nodes
pub fn traverse_expr_for_check<F>(expr: &Expr, check: F) -> bool
where
    F: Fn(&Expr) -> Option<bool> + Copy,
{
    // First try the specific check
    if let Some(result) = check(expr) {
        return result;
    }

    // Generic traversal into child nodes
    match &expr.kind {
        ExprKind::Block(exprs) => exprs.iter().any(|e| traverse_expr_for_check(e, check)),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            traverse_expr_for_check(condition, check)
                || traverse_expr_for_check(then_branch, check)
                || else_branch
                    .as_ref()
                    .is_some_and(|e| traverse_expr_for_check(e, check))
        }
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
            traverse_expr_for_check(value, check) || traverse_expr_for_check(body, check)
        }
        ExprKind::Binary { left, right, .. } => {
            traverse_expr_for_check(left, check) || traverse_expr_for_check(right, check)
        }
        ExprKind::While {
            condition, body, ..
        }
        | ExprKind::For {
            iter: condition,
            body,
            ..
        } => traverse_expr_for_check(condition, check) || traverse_expr_for_check(body, check),
        ExprKind::Assign { target, value } => {
            traverse_expr_for_check(target, check) || traverse_expr_for_check(value, check)
        }
        ExprKind::CompoundAssign { target, value, .. } => {
            traverse_expr_for_check(target, check) || traverse_expr_for_check(value, check)
        }
        ExprKind::Call { args, .. } => args.iter().any(|arg| traverse_expr_for_check(arg, check)),
        ExprKind::IndexAccess { object, index } => {
            traverse_expr_for_check(object, check) || traverse_expr_for_check(index, check)
        }
        ExprKind::Unary { operand, .. } => traverse_expr_for_check(operand, check),
        _ => false,
    }
}

/// Check if parameter is directly in arguments
#[must_use]
pub fn find_param_in_direct_args(param_name: &str, args: &[Expr]) -> bool {
    args.iter()
        .any(|arg| matches!(&arg.kind, ExprKind::Identifier(name) if name == param_name))
}

/// Check if parameter is used as argument in function call
#[must_use]
pub fn check_call_for_param_argument(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    // Check if any argument is the parameter
    if matches!(&func.kind, ExprKind::Identifier(_)) && find_param_in_direct_args(param_name, args)
    {
        return true;
    }
    // Recursively check nested arguments
    args.iter()
        .any(|arg| is_param_used_as_function_argument(param_name, arg))
}

/// Check if parameter is used in expressions list
#[must_use]
pub fn check_expressions_for_param(param_name: &str, exprs: &[Expr]) -> bool {
    exprs
        .iter()
        .any(|e| is_param_used_as_function_argument(param_name, e))
}

/// Check if parameter is used in if expression
#[must_use]
pub fn check_if_for_param(
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
#[must_use]
pub fn check_let_for_param(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_as_function_argument(param_name, value)
        || is_param_used_as_function_argument(param_name, body)
}

/// Check if parameter is used in binary expression
#[must_use]
pub fn check_binary_for_param(param_name: &str, left: &Expr, right: &Expr) -> bool {
    is_param_used_as_function_argument(param_name, left)
        || is_param_used_as_function_argument(param_name, right)
}

/// Analyzes if a parameter is used as an argument to a function
#[must_use]
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

/// Check if parameter is the function being called
#[must_use]
pub fn check_func_call(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    if let ExprKind::Identifier(name) = &func.kind {
        if name == param_name {
            return true;
        }
    }
    args.iter()
        .any(|arg| is_param_used_as_function(param_name, arg))
}

/// Check if branches for parameter as function
#[must_use]
pub fn check_if_for_func(
    param_name: &str,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> bool {
    is_param_used_as_function(param_name, condition)
        || is_param_used_as_function(param_name, then_branch)
        || else_branch.is_some_and(|e| is_param_used_as_function(param_name, e))
}

/// Check let and binary expressions for function usage
#[must_use]
pub fn check_let_and_binary_for_func(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_as_function(param_name, value) || is_param_used_as_function(param_name, body)
}

/// Analyzes if a parameter is used as a function in the given expression
#[must_use]
pub fn is_param_used_as_function(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, args } => check_func_call(param_name, func, args),
        ExprKind::Block(exprs) => exprs
            .iter()
            .any(|e| is_param_used_as_function(param_name, e)),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => check_if_for_func(param_name, condition, then_branch, else_branch.as_deref()),
        ExprKind::Let { value, body, .. }
        | ExprKind::Binary {
            left: value,
            right: body,
            ..
        } => check_let_and_binary_for_func(param_name, value, body),
        ExprKind::Lambda { body, .. } => is_param_used_as_function(param_name, body),
        _ => false,
    }
}

/// Check if operator is numeric
#[must_use]
pub fn is_numeric_operator(op: &BinaryOp) -> bool {
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
    )
}

/// Check if param is in operation
#[must_use]
pub fn has_param_in_operation(param_name: &str, left: &Expr, right: &Expr) -> bool {
    contains_param(param_name, left) || contains_param(param_name, right)
}

/// Check if operation is string concatenation
#[must_use]
pub fn is_string_concatenation(op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
    matches!(op, BinaryOp::Add) && (is_string_literal(left) || is_string_literal(right))
}

/// Check numeric usage in binary expressions
#[must_use]
pub fn check_binary_numeric_usage(param_name: &str, op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
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

/// Check numeric usage in blocks
#[must_use]
pub fn check_block_numeric_usage(param_name: &str, exprs: &[Expr]) -> bool {
    exprs
        .iter()
        .any(|e| is_param_used_numerically(param_name, e))
}

/// Check numeric usage in if expressions
#[must_use]
pub fn check_if_numeric_usage(
    param_name: &str,
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
) -> bool {
    is_param_used_numerically(param_name, condition)
        || is_param_used_numerically(param_name, then_branch)
        || else_branch.is_some_and(|e| is_param_used_numerically(param_name, e))
}

/// Check numeric usage in let expressions
#[must_use]
pub fn check_let_numeric_usage(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_numerically(param_name, value) || is_param_used_numerically(param_name, body)
}

/// Check numeric usage in call arguments
#[must_use]
pub fn check_call_numeric_usage(param_name: &str, args: &[Expr]) -> bool {
    args.iter()
        .any(|arg| is_param_used_numerically(param_name, arg))
}

/// Checks if a parameter is used in numeric operations
#[must_use]
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
        ExprKind::Lambda { body, .. } => is_param_used_numerically(param_name, body),
        ExprKind::While {
            condition, body, ..
        } => {
            is_param_used_numerically(param_name, condition)
                || is_param_used_numerically(param_name, body)
        }
        ExprKind::For { iter, body, .. } => {
            is_param_used_numerically(param_name, iter)
                || is_param_used_numerically(param_name, body)
        }
        _ => false,
    }
}

/// Check function call for parameter
#[must_use]
pub fn check_call_contains_param(param_name: &str, func: &Expr, args: &[Expr]) -> bool {
    contains_param(param_name, func) || args.iter().any(|arg| contains_param(param_name, arg))
}

/// Helper to check if an expression contains a specific parameter
#[must_use]
pub fn contains_param(param_name: &str, expr: &Expr) -> bool {
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

/// Check if parameter is used as an array (indexed like param[...])
#[must_use]
pub fn is_param_used_as_array(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        // Direct indexing: param[index]
        if let ExprKind::IndexAccess { object, .. } = &e.kind {
            if let ExprKind::Identifier(name) = &object.kind {
                if name == param_name {
                    return Some(true);
                }
            }
        }
        None // Continue traversal
    })
}

/// Check if parameter is used with `len()` function
#[must_use]
pub fn is_param_used_with_len(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        // Check if param is argument to len() function
        if let ExprKind::Call { func, args } = &e.kind {
            if let ExprKind::Identifier(func_name) = &func.kind {
                if func_name == "len" {
                    for arg in args {
                        if let ExprKind::Identifier(arg_name) = &arg.kind {
                            if arg_name == param_name {
                                return Some(true);
                            }
                        }
                    }
                }
            }
        }
        None // Continue traversal
    })
}

/// Check if parameter is used as an index (like array[param])
#[must_use]
pub fn is_param_used_as_index(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        // Check if param is the index in array[param]
        if let ExprKind::IndexAccess { index, .. } = &e.kind {
            if contains_param(param_name, index) {
                return Some(true);
            }
        }
        None // Continue traversal
    })
}

/// Check if parameter is used as a boolean condition
#[must_use]
pub fn is_param_used_as_bool(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        match &e.kind {
            ExprKind::If { condition, .. } | ExprKind::While { condition, .. } => {
                if let ExprKind::Identifier(name) = &condition.kind {
                    if name == param_name {
                        return Some(true);
                    }
                }
            }
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand,
            } => {
                if let ExprKind::Identifier(name) = &operand.kind {
                    if name == param_name {
                        return Some(true);
                    }
                }
            }
            ExprKind::Binary {
                op: crate::frontend::ast::BinaryOp::And | crate::frontend::ast::BinaryOp::Or,
                left,
                right,
            } => {
                if let ExprKind::Identifier(name) = &left.kind {
                    if name == param_name {
                        return Some(true);
                    }
                }
                if let ExprKind::Identifier(name) = &right.kind {
                    if name == param_name {
                        return Some(true);
                    }
                }
            }
            _ => {}
        }
        None // Continue traversal
    })
}

/// Check if parameter is used in string concatenation
#[must_use]
pub fn is_param_used_in_string_concat(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        if let ExprKind::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } = &e.kind
        {
            let left_is_string = is_string_literal(left);
            let right_is_string = is_string_literal(right);

            if left_is_string && contains_param(param_name, right) {
                return Some(true);
            }
            if right_is_string && contains_param(param_name, left) {
                return Some(true);
            }
        }
        None // Continue traversal
    })
}

/// Check if parameter is used in a print/format macro
#[must_use]
pub fn is_param_used_in_print_macro(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        if let ExprKind::MacroInvocation { name, args, .. } = &e.kind {
            if matches!(
                name.as_str(),
                "println" | "print" | "eprintln" | "eprint" | "format" | "write" | "writeln"
            ) {
                for arg in args.iter().skip(1) {
                    if contains_param(param_name, arg) {
                        return Some(true);
                    }
                }
            }
        }
        None // Continue traversal
    })
}

/// Detect nested array access like param[i][j]
#[must_use]
pub fn is_nested_array_access(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        if let ExprKind::IndexAccess { object, .. } = &e.kind {
            if let ExprKind::IndexAccess { object: inner, .. } = &object.kind {
                if let ExprKind::Identifier(name) = &inner.kind {
                    if name == param_name {
                        return Some(true);
                    }
                }
            }
        }
        None
    })
}

/// Infer parameter type based on usage patterns in function body
#[must_use]
pub fn infer_param_type(param_name: &str, body: &Expr) -> Option<&'static str> {
    // Check for array indexing patterns
    if is_param_used_as_array(param_name, body) {
        // Detect dimensionality
        if is_nested_array_access(param_name, body) {
            return Some("Vec<Vec<i32>>");
        }
        return Some("Vec<i32>");
    }

    // Check for len() usage
    if is_param_used_with_len(param_name, body) {
        return Some("Vec<i32>");
    }

    // Check if used as index
    if is_param_used_as_index(param_name, body) {
        return Some("i32");
    }

    // Check for builtin function usage
    if let Some(typ) = infer_param_type_from_builtin_usage(param_name, body) {
        return Some(typ);
    }

    // No inference - keep original type
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    // ==================== traverse_expr_for_check Tests ====================

    #[test]
    fn test_traverse_expr_for_check_returns_early_on_true() {
        let code = "fun test() { 42 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        let result = traverse_expr_for_check(&ast, |_| Some(true));
        assert!(result);
    }

    #[test]
    fn test_traverse_expr_for_check_returns_early_on_false() {
        let code = "fun test() { 42 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        let result = traverse_expr_for_check(&ast, |_| Some(false));
        assert!(!result);
    }

    #[test]
    fn test_traverse_expr_for_check_continues_on_none() {
        let code = "fun test() { 42 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        let result = traverse_expr_for_check(&ast, |_| None);
        assert!(!result);
    }

    #[test]
    fn test_traverse_expr_for_check_traverses_block() {
        // Use is_param_used_as_array which uses traverse_expr_for_check
        let code = "fun test(arr) { arr[0] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Verify that traverse works by checking array usage
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_traverse_expr_for_check_traverses_if() {
        let code = "fun test(arr) { if true { arr[0] } else { 0 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Verify traversal into if branches
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_traverse_expr_for_check_traverses_while() {
        let code = "fun test(arr) { while true { arr[0] } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Verify traversal into while body
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_traverse_expr_for_check_traverses_for() {
        let code = "fun test(arr) { for x in [1,2,3] { arr[0] } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // Verify traversal into for body
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    // ==================== find_param_in_direct_args Tests ====================

    #[test]
    fn test_find_param_in_direct_args_found() {
        let code = "fun test(x) { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Call { args, .. } = &body.kind {
                        assert!(find_param_in_direct_args("x", args));
                    }
                }
            }
        }
    }

    #[test]
    fn test_find_param_in_direct_args_not_found() {
        let code = "fun test(x) { f(y) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Call { args, .. } = &body.kind {
                        assert!(!find_param_in_direct_args("x", args));
                    }
                }
            }
        }
    }

    #[test]
    fn test_find_param_in_direct_args_empty() {
        let args: Vec<Expr> = vec![];
        assert!(!find_param_in_direct_args("x", &args));
    }

    // ==================== is_param_used_as_function_argument Tests ====================

    #[test]
    fn test_is_param_used_as_function_argument_direct() {
        let code = "fun test(x) { some_func(x) }";
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
    fn test_is_param_used_as_function_argument_nested() {
        let code = "fun test(x) { outer(inner(x)) }";
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
    fn test_is_param_used_as_function_argument_in_if() {
        let code = "fun test(x) { if true { f(x) } else { 0 } }";
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
    fn test_is_param_used_as_function_argument_in_else() {
        let code = "fun test(x) { if true { 0 } else { f(x) } }";
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
    fn test_is_param_used_as_function_argument_in_let() {
        let code = "fun test(x) { let y = f(x); y }";
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
    fn test_is_param_used_as_function_argument_in_binary() {
        let code = "fun test(x) { f(x) + g(x) }";
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
    fn test_is_param_used_as_function_argument_in_unary() {
        let code = "fun test(x) { -f(x) }";
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
    fn test_is_param_used_as_function_argument_false() {
        let code = "fun test(x) { x + 1 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_function_argument("x", body));
                }
            }
        }
    }

    // ==================== is_param_used_as_function Tests ====================

    #[test]
    fn test_is_param_used_as_function_direct() {
        let code = "fun test(f) { f() }";
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
    fn test_is_param_used_as_function_with_args() {
        let code = "fun test(callback) { callback(1, 2, 3) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("callback", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_function_in_block() {
        let code = "fun test(f) { let x = 1; f(x) }";
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
    fn test_is_param_used_as_function_in_if() {
        let code = "fun test(f) { if true { f(5) } else { 0 } }";
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
    fn test_is_param_used_as_function_in_lambda() {
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
    fn test_is_param_used_as_function_false() {
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

    // ==================== is_numeric_operator Tests ====================

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
    fn test_is_numeric_operator_equal_false() {
        assert!(!is_numeric_operator(&BinaryOp::Equal));
    }

    #[test]
    fn test_is_numeric_operator_not_equal_false() {
        assert!(!is_numeric_operator(&BinaryOp::NotEqual));
    }

    #[test]
    fn test_is_numeric_operator_and_false() {
        assert!(!is_numeric_operator(&BinaryOp::And));
    }

    #[test]
    fn test_is_numeric_operator_or_false() {
        assert!(!is_numeric_operator(&BinaryOp::Or));
    }

    // ==================== is_param_used_numerically Tests ====================

    #[test]
    fn test_is_param_used_numerically_add() {
        let code = "fun test(x) { x + 5 }";
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
    fn test_is_param_used_numerically_subtract() {
        let code = "fun test(x) { x - 5 }";
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
    fn test_is_param_used_numerically_multiply() {
        let code = "fun test(x) { x * 2 }";
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
    fn test_is_param_used_numerically_divide() {
        let code = "fun test(x) { x / 3 }";
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
    fn test_is_param_used_numerically_modulo() {
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

    #[test]
    fn test_is_param_used_numerically_comparison() {
        let code = "fun test(count) { let x = 0; while x < count { x = x + 1 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("count", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_numerically_in_block() {
        let code = "fun test(x) { let y = 1; x - y }";
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
    fn test_is_param_used_numerically_in_if() {
        let code = "fun test(x) { if true { x * 2 } else { 0 } }";
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
    fn test_is_param_used_numerically_in_lambda() {
        let code = "fun test(x) { (y) => x + y }";
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
    fn test_is_param_used_numerically_in_while() {
        let code = "fun test(x) { while x > 0 { x = x - 1 } }";
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
    fn test_is_param_used_numerically_in_for() {
        let code = "fun test(x) { for i in [1,2,3] { x + i } }";
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
    fn test_is_param_used_numerically_string_concat_false() {
        let code = r#"fun test(name) { "Hello " + name }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_numerically("name", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_numerically_false() {
        let code = r#"fun test(x) { "hello" }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_numerically("x", body));
                }
            }
        }
    }

    // ==================== contains_param Tests ====================

    #[test]
    fn test_contains_param_identifier() {
        let code = "fun test(x) { x }";
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
    fn test_contains_param_not_found() {
        let code = "fun test(x) { y }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!contains_param("x", body));
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
                }
            }
        }
    }

    #[test]
    fn test_contains_param_in_block() {
        let code = "fun test(x) { let y = 1; x }";
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
    fn test_contains_param_in_call() {
        let code = "fun test(x) { f(x) }";
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

    // ==================== is_param_used_as_array Tests ====================

    #[test]
    fn test_is_param_used_as_array_direct() {
        let code = "fun test(arr) { arr[0] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_array_in_if() {
        let code = "fun test(arr) { if true { arr[1] } else { 0 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_array_in_while() {
        let code = "fun test(arr) { let i = 0; while i < 10 { arr[i] } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_array("arr", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_array_false() {
        let code = "fun test(x) { x + 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_array("x", body));
                }
            }
        }
    }

    // ==================== is_param_used_with_len Tests ====================

    #[test]
    fn test_is_param_used_with_len_direct() {
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
    fn test_is_param_used_with_len_in_while() {
        let code = "fun test(arr) { let i = 0; while i < len(arr) { i = i + 1 } }";
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
    fn test_is_param_used_with_len_false() {
        let code = "fun test(x) { x * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_with_len("x", body));
                }
            }
        }
    }

    // ==================== is_param_used_as_index Tests ====================

    #[test]
    fn test_is_param_used_as_index_direct() {
        let code = "fun test(i) { let arr = [1,2,3]; arr[i] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_index("i", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_index_in_for() {
        let code = "fun test(i) { for x in [1,2,3] { arr[i] } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_index("i", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_index_false() {
        let code = "fun test(x) { x + 10 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_index("x", body));
                }
            }
        }
    }

    // ==================== is_param_used_as_bool Tests ====================

    #[test]
    fn test_is_param_used_as_bool_in_if() {
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
    fn test_is_param_used_as_bool_in_while() {
        let code = "fun test(flag) { while flag { } }";
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
    fn test_is_param_used_as_bool_with_not() {
        let code = "fun test(flag) { !flag }";
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
    fn test_is_param_used_as_bool_with_and() {
        let code = "fun test(a) { a && true }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_bool("a", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_bool_with_or() {
        let code = "fun test(a) { a || false }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_bool("a", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_bool_right_side_and() {
        let code = "fun test(b) { true && b }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_bool("b", body));
                }
            }
        }
    }

    #[test]
    fn test_is_param_used_as_bool_false() {
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

    // ==================== is_param_used_in_string_concat Tests ====================

    #[test]
    fn test_is_param_used_in_string_concat_left() {
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
    fn test_is_param_used_in_string_concat_right() {
        let code = r#"fun test(name) { name + "!" }"#;
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
    fn test_is_param_used_in_string_concat_false() {
        let code = "fun test(x) { x + 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_in_string_concat("x", body));
                }
            }
        }
    }

    // ==================== is_nested_array_access Tests ====================

    #[test]
    fn test_is_nested_array_access_true() {
        let code = "fun test(matrix) { matrix[0][1] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_nested_array_access("matrix", body));
                }
            }
        }
    }

    #[test]
    fn test_is_nested_array_access_false() {
        let code = "fun test(arr) { arr[0] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_nested_array_access("arr", body));
                }
            }
        }
    }

    // ==================== infer_param_type Tests ====================

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
    fn test_infer_param_type_nested_array() {
        let code = "fun test(matrix) { matrix[0][1] }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_param_type("matrix", body), Some("Vec<Vec<i32>>"));
                }
            }
        }
    }

    #[test]
    fn test_infer_param_type_with_len() {
        let code = "fun test(arr) { len(arr) }";
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
    fn test_infer_param_type_none() {
        let code = r#"fun test(x) { "hello" }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert_eq!(infer_param_type("x", body), None);
                }
            }
        }
    }

    // ==================== is_string_concatenation Tests ====================

    #[test]
    fn test_is_string_concatenation_left_string() {
        let code = r#"fun test() { "hello" + x }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { op, left, right } = &body.kind {
                        assert!(is_string_concatenation(op, left, right));
                    }
                }
            }
        }
    }

    #[test]
    fn test_is_string_concatenation_right_string() {
        let code = r#"fun test() { x + "world" }"#;
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { op, left, right } = &body.kind {
                        assert!(is_string_concatenation(op, left, right));
                    }
                }
            }
        }
    }

    #[test]
    fn test_is_string_concatenation_numeric() {
        let code = "fun test() { 1 + 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { op, left, right } = &body.kind {
                        assert!(!is_string_concatenation(op, left, right));
                    }
                }
            }
        }
    }

    // ==================== has_param_in_operation Tests ====================

    #[test]
    fn test_has_param_in_operation_left() {
        let code = "fun test(x) { x + 1 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { left, right, .. } = &body.kind {
                        assert!(has_param_in_operation("x", left, right));
                    }
                }
            }
        }
    }

    #[test]
    fn test_has_param_in_operation_right() {
        let code = "fun test(x) { 1 + x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { left, right, .. } = &body.kind {
                        assert!(has_param_in_operation("x", left, right));
                    }
                }
            }
        }
    }

    #[test]
    fn test_has_param_in_operation_false() {
        let code = "fun test(x) { 1 + 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Binary { left, right, .. } = &body.kind {
                        assert!(!has_param_in_operation("x", left, right));
                    }
                }
            }
        }
    }

    // ==================== Additional Edge Case Tests ====================

    #[test]
    fn test_check_call_contains_param_in_func() {
        let code = "fun test(f) { f(1) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Call { func, args } = &body.kind {
                        assert!(check_call_contains_param("f", func, args));
                    }
                }
            }
        }
    }

    #[test]
    fn test_check_call_contains_param_in_args() {
        let code = "fun test(x) { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().expect("Failed to parse");
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    if let ExprKind::Call { func, args } = &body.kind {
                        assert!(check_call_contains_param("x", func, args));
                    }
                }
            }
        }
    }
}
