//! Parameter usage analysis for type inference
//!
//! This module provides functions for analyzing how parameters are used
//! in function bodies to infer their types.

use crate::frontend::ast::{BinaryOp, Expr, ExprKind};

// Re-export from builtin_type_inference for backwards compatibility
pub use super::builtin_type_inference::{infer_param_type_from_builtin_usage, is_string_literal};

/// Generic AST traversal for parameter usage checks.
///
/// The `check` closure returns:
/// - `Some(true)` if the check succeeded (stop traversal, return true)
/// - `Some(false)` if the check explicitly failed (stop traversal, return false)
/// - `None` to continue traversal into child nodes
/// Collect child expressions for generic AST traversal
fn collect_child_exprs(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Block(exprs) => exprs.iter().collect(),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let mut children = vec![condition.as_ref(), then_branch.as_ref()];
            if let Some(e) = else_branch {
                children.push(e);
            }
            children
        }
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
            vec![value.as_ref(), body.as_ref()]
        }
        ExprKind::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
        ExprKind::While {
            condition, body, ..
        }
        | ExprKind::For {
            iter: condition,
            body,
            ..
        } => vec![condition.as_ref(), body.as_ref()],
        ExprKind::Assign { target, value } | ExprKind::CompoundAssign { target, value, .. } => {
            vec![target.as_ref(), value.as_ref()]
        }
        ExprKind::Call { args, .. } => args.iter().collect(),
        ExprKind::IndexAccess { object, index } => vec![object.as_ref(), index.as_ref()],
        ExprKind::Unary { operand, .. } => vec![operand.as_ref()],
        _ => vec![],
    }
}

pub fn traverse_expr_for_check<F>(expr: &Expr, check: F) -> bool
where
    F: Fn(&Expr) -> Option<bool> + Copy,
{
    if let Some(result) = check(expr) {
        return result;
    }
    collect_child_exprs(expr)
        .into_iter()
        .any(|child| traverse_expr_for_check(child, check))
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
pub fn check_binary_numeric_usage(
    param_name: &str,
    op: &BinaryOp,
    left: &Expr,
    right: &Expr,
) -> bool {
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
fn is_call_to_func_with_param(e: &Expr, func_name_target: &str, param_name: &str) -> bool {
    if let ExprKind::Call { func, args } = &e.kind {
        if let ExprKind::Identifier(func_name) = &func.kind {
            if func_name == func_name_target {
                return args
                    .iter()
                    .any(|arg| is_identifier_matching(arg, param_name));
            }
        }
    }
    false
}

pub fn is_param_used_with_len(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        if is_call_to_func_with_param(e, "len", param_name) {
            Some(true)
        } else {
            None
        }
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
fn is_identifier_matching(expr: &Expr, param_name: &str) -> bool {
    matches!(&expr.kind, ExprKind::Identifier(name) if name == param_name)
}

pub fn is_param_used_as_bool(param_name: &str, expr: &Expr) -> bool {
    traverse_expr_for_check(expr, |e| {
        match &e.kind {
            ExprKind::If { condition, .. } | ExprKind::While { condition, .. } => {
                if is_identifier_matching(condition, param_name) {
                    return Some(true);
                }
            }
            ExprKind::Unary {
                op: crate::frontend::ast::UnaryOp::Not,
                operand,
            } => {
                if is_identifier_matching(operand, param_name) {
                    return Some(true);
                }
            }
            ExprKind::Binary {
                op: crate::frontend::ast::BinaryOp::And | crate::frontend::ast::BinaryOp::Or,
                left,
                right,
            } => {
                if is_identifier_matching(left, param_name)
                    || is_identifier_matching(right, param_name)
                {
                    return Some(true);
                }
            }
            _ => {}
        }
        None
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
#[path = "param_usage_analysis_inline_tests.rs"]
mod tests;
