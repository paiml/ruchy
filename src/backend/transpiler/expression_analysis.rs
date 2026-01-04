//! Expression analysis utilities for transpiler optimization
//!
//! This module provides pure functions for analyzing expressions,
//! including side effect detection, early exit detection, and
//! liveness analysis for dead code elimination.

use crate::frontend::ast::{Expr, ExprKind};
use std::collections::HashSet;

/// Check if an expression has side effects
///
/// Side effects include:
/// - Function calls (may modify external state)
/// - Assignments (modify variables)
#[must_use]
pub fn has_side_effects(expr: &Expr) -> bool {
    matches!(expr.kind, ExprKind::Call { .. } | ExprKind::Assign { .. })
}

/// Check if expression causes early exit
///
/// Early exits include:
/// - Return statements
/// - Break statements
/// - Continue statements
#[must_use]
pub fn has_early_exit(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Return { .. } | ExprKind::Break { .. } | ExprKind::Continue { .. }
    )
}

/// Check if a function should be removed during dead code elimination
///
/// A function should be removed if:
/// - It was inlined AND
/// - It's not used anywhere else AND
/// - It's not the main function
#[must_use]
pub fn should_remove_function(
    name: &str,
    used_functions: &HashSet<String>,
    inlined_functions: &HashSet<String>,
) -> bool {
    inlined_functions.contains(name) && !used_functions.contains(name) && name != "main"
}

/// Collect all function names that are called in the expression tree
#[must_use]
pub fn collect_used_functions(expr: &Expr) -> HashSet<String> {
    let mut used = HashSet::new();
    collect_used_functions_rec(expr, &mut used);
    used
}

/// Recursive helper to collect used function names
fn collect_used_functions_rec(expr: &Expr, used: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Call { func, args } => {
            if let ExprKind::Identifier(func_name) = &func.kind {
                used.insert(func_name.clone());
            }
            collect_used_functions_rec(func, used);
            for arg in args {
                collect_used_functions_rec(arg, used);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                collect_used_functions_rec(e, used);
            }
        }
        ExprKind::Function { body, .. } => {
            collect_used_functions_rec(body, used);
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_used_functions_rec(condition, used);
            collect_used_functions_rec(then_branch, used);
            if let Some(else_expr) = else_branch {
                collect_used_functions_rec(else_expr, used);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            collect_used_functions_rec(left, used);
            collect_used_functions_rec(right, used);
        }
        ExprKind::Let { value, body, .. } => {
            collect_used_functions_rec(value, used);
            collect_used_functions_rec(body, used);
        }
        ExprKind::Await { expr } => {
            collect_used_functions_rec(expr, used);
        }
        ExprKind::AsyncBlock { body } => {
            collect_used_functions_rec(body, used);
        }
        ExprKind::Spawn { actor } => {
            collect_used_functions_rec(actor, used);
        }
        _ => {}
    }
}

/// Collect all variable names that are used (read) in the expression tree
#[must_use]
pub fn collect_used_variables(expr: &Expr) -> HashSet<String> {
    let mut used = HashSet::new();
    collect_used_variables_rec(expr, &mut used, &HashSet::new());
    used
}

/// Recursive helper to collect used variable names
fn collect_used_variables_rec(expr: &Expr, used: &mut HashSet<String>, bound: &HashSet<String>) {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            if bound.contains(name) {
                used.insert(name.clone());
            }
        }
        ExprKind::Let { name, value, body, .. } => {
            collect_used_variables_rec(value, used, bound);
            let mut new_bound = bound.clone();
            new_bound.insert(name.clone());
            collect_used_variables_rec(body, used, &new_bound);
        }
        ExprKind::Block(exprs) => {
            let mut current_bound = bound.clone();
            for e in exprs {
                if let ExprKind::Let { name, .. } = &e.kind {
                    current_bound.insert(name.clone());
                }
                collect_used_variables_rec(e, used, &current_bound);
            }
        }
        ExprKind::Function { params, body, .. } => {
            let mut func_bound = bound.clone();
            for param in params {
                if let crate::frontend::ast::Pattern::Identifier(name) = &param.pattern {
                    func_bound.insert(name.clone());
                }
            }
            collect_used_variables_rec(body, used, &func_bound);
        }
        ExprKind::Call { func, args } => {
            collect_used_variables_rec(func, used, bound);
            for arg in args {
                collect_used_variables_rec(arg, used, bound);
            }
        }
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_used_variables_rec(condition, used, bound);
            collect_used_variables_rec(then_branch, used, bound);
            if let Some(else_expr) = else_branch {
                collect_used_variables_rec(else_expr, used, bound);
            }
        }
        ExprKind::Binary { left, right, .. } => {
            collect_used_variables_rec(left, used, bound);
            collect_used_variables_rec(right, used, bound);
        }
        ExprKind::Unary { operand, .. } => {
            collect_used_variables_rec(operand, used, bound);
        }
        ExprKind::Assign { target, value } => {
            collect_used_variables_rec(target, used, bound);
            collect_used_variables_rec(value, used, bound);
        }
        ExprKind::Return { value } => {
            if let Some(val) = value {
                collect_used_variables_rec(val, used, bound);
            }
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            collect_used_variables_rec(receiver, used, bound);
            for arg in args {
                collect_used_variables_rec(arg, used, bound);
            }
        }
        ExprKind::IndexAccess { object, index } => {
            collect_used_variables_rec(object, used, bound);
            collect_used_variables_rec(index, used, bound);
        }
        ExprKind::FieldAccess { object, .. } => {
            collect_used_variables_rec(object, used, bound);
        }
        _ => {}
    }
}

/// Check if an expression is a pure function call
/// (no side effects besides returning a value)
#[must_use]
pub fn is_pure_expression(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Literal(_) => true,
        ExprKind::Identifier(_) => true,
        ExprKind::Binary { left, right, .. } => {
            is_pure_expression(left) && is_pure_expression(right)
        }
        ExprKind::Unary { operand, .. } => is_pure_expression(operand),
        ExprKind::Tuple(exprs) | ExprKind::List(exprs) => {
            exprs.iter().all(is_pure_expression)
        }
        _ => false,
    }
}

/// Check if an expression is a constant (can be evaluated at compile time)
#[must_use]
pub fn is_constant_expression(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::Literal(_)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn call(func_name: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Call {
            func: Box::new(ident(func_name)),
            args,
        })
    }

    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        make_expr(ExprKind::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    }

    fn block(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn ret(value: Option<Expr>) -> Expr {
        make_expr(ExprKind::Return {
            value: value.map(Box::new),
        })
    }

    fn brk() -> Expr {
        make_expr(ExprKind::Break { label: None, value: None })
    }

    fn cont() -> Expr {
        make_expr(ExprKind::Continue { label: None })
    }

    fn assign(target: Expr, value: Expr) -> Expr {
        make_expr(ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    // ==================== has_side_effects Tests ====================

    #[test]
    fn test_has_side_effects_call() {
        let expr = call("foo", vec![]);
        assert!(has_side_effects(&expr));
    }

    #[test]
    fn test_has_side_effects_call_with_args() {
        let expr = call("bar", vec![int_lit(1), int_lit(2)]);
        assert!(has_side_effects(&expr));
    }

    #[test]
    fn test_has_side_effects_assign() {
        let expr = assign(ident("x"), int_lit(5));
        assert!(has_side_effects(&expr));
    }

    #[test]
    fn test_has_side_effects_literal() {
        let expr = int_lit(42);
        assert!(!has_side_effects(&expr));
    }

    #[test]
    fn test_has_side_effects_identifier() {
        let expr = ident("x");
        assert!(!has_side_effects(&expr));
    }

    #[test]
    fn test_has_side_effects_binary() {
        let expr = binary(int_lit(1), BinaryOp::Add, int_lit(2));
        assert!(!has_side_effects(&expr));
    }

    // ==================== has_early_exit Tests ====================

    #[test]
    fn test_has_early_exit_return() {
        let expr = ret(Some(int_lit(0)));
        assert!(has_early_exit(&expr));
    }

    #[test]
    fn test_has_early_exit_return_void() {
        let expr = ret(None);
        assert!(has_early_exit(&expr));
    }

    #[test]
    fn test_has_early_exit_break() {
        let expr = brk();
        assert!(has_early_exit(&expr));
    }

    #[test]
    fn test_has_early_exit_continue() {
        let expr = cont();
        assert!(has_early_exit(&expr));
    }

    #[test]
    fn test_has_early_exit_call() {
        let expr = call("foo", vec![]);
        assert!(!has_early_exit(&expr));
    }

    #[test]
    fn test_has_early_exit_literal() {
        let expr = int_lit(42);
        assert!(!has_early_exit(&expr));
    }

    // ==================== should_remove_function Tests ====================

    #[test]
    fn test_should_remove_function_inlined_and_unused() {
        let mut inlined = HashSet::new();
        inlined.insert("helper".to_string());
        let used = HashSet::new();
        assert!(should_remove_function("helper", &used, &inlined));
    }

    #[test]
    fn test_should_remove_function_inlined_but_used() {
        let mut inlined = HashSet::new();
        inlined.insert("helper".to_string());
        let mut used = HashSet::new();
        used.insert("helper".to_string());
        assert!(!should_remove_function("helper", &used, &inlined));
    }

    #[test]
    fn test_should_remove_function_not_inlined() {
        let inlined = HashSet::new();
        let used = HashSet::new();
        assert!(!should_remove_function("helper", &used, &inlined));
    }

    #[test]
    fn test_should_remove_function_main() {
        let mut inlined = HashSet::new();
        inlined.insert("main".to_string());
        let used = HashSet::new();
        // main should never be removed
        assert!(!should_remove_function("main", &used, &inlined));
    }

    // ==================== collect_used_functions Tests ====================

    #[test]
    fn test_collect_used_functions_single_call() {
        let expr = call("foo", vec![]);
        let used = collect_used_functions(&expr);
        assert!(used.contains("foo"));
        assert_eq!(used.len(), 1);
    }

    #[test]
    fn test_collect_used_functions_multiple_calls() {
        let expr = block(vec![
            call("foo", vec![]),
            call("bar", vec![]),
        ]);
        let used = collect_used_functions(&expr);
        assert!(used.contains("foo"));
        assert!(used.contains("bar"));
        assert_eq!(used.len(), 2);
    }

    #[test]
    fn test_collect_used_functions_nested_calls() {
        let expr = call("outer", vec![call("inner", vec![])]);
        let used = collect_used_functions(&expr);
        assert!(used.contains("outer"));
        assert!(used.contains("inner"));
    }

    #[test]
    fn test_collect_used_functions_no_calls() {
        let expr = binary(int_lit(1), BinaryOp::Add, int_lit(2));
        let used = collect_used_functions(&expr);
        assert!(used.is_empty());
    }

    #[test]
    fn test_collect_used_functions_in_binary() {
        let expr = binary(call("f", vec![]), BinaryOp::Add, call("g", vec![]));
        let used = collect_used_functions(&expr);
        assert!(used.contains("f"));
        assert!(used.contains("g"));
    }

    // ==================== collect_used_variables Tests ====================

    #[test]
    fn test_collect_used_variables_identifier() {
        // Variable not in scope won't be collected
        let expr = ident("x");
        let used = collect_used_variables(&expr);
        assert!(used.is_empty());
    }

    #[test]
    fn test_collect_used_variables_in_binary() {
        let expr = binary(ident("a"), BinaryOp::Add, ident("b"));
        let used = collect_used_variables(&expr);
        // Variables not in scope
        assert!(used.is_empty());
    }

    #[test]
    fn test_collect_used_variables_literal() {
        let expr = int_lit(42);
        let used = collect_used_variables(&expr);
        assert!(used.is_empty());
    }

    // ==================== is_pure_expression Tests ====================

    #[test]
    fn test_is_pure_expression_literal() {
        let expr = int_lit(42);
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_string_literal() {
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_identifier() {
        let expr = ident("x");
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_binary() {
        let expr = binary(int_lit(1), BinaryOp::Add, int_lit(2));
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_nested_binary() {
        let expr = binary(
            binary(int_lit(1), BinaryOp::Add, int_lit(2)),
            BinaryOp::Multiply,
            int_lit(3),
        );
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_call() {
        let expr = call("foo", vec![]);
        assert!(!is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_tuple() {
        let expr = make_expr(ExprKind::Tuple(vec![int_lit(1), int_lit(2)]));
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_list() {
        let expr = make_expr(ExprKind::List(vec![int_lit(1), int_lit(2)]));
        assert!(is_pure_expression(&expr));
    }

    #[test]
    fn test_is_pure_expression_list_with_call() {
        let expr = make_expr(ExprKind::List(vec![call("foo", vec![])]));
        assert!(!is_pure_expression(&expr));
    }

    // ==================== is_constant_expression Tests ====================

    #[test]
    fn test_is_constant_expression_int() {
        let expr = int_lit(42);
        assert!(is_constant_expression(&expr));
    }

    #[test]
    fn test_is_constant_expression_string() {
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        assert!(is_constant_expression(&expr));
    }

    #[test]
    fn test_is_constant_expression_bool() {
        let expr = make_expr(ExprKind::Literal(Literal::Bool(true)));
        assert!(is_constant_expression(&expr));
    }

    #[test]
    fn test_is_constant_expression_identifier() {
        let expr = ident("x");
        assert!(!is_constant_expression(&expr));
    }

    #[test]
    fn test_is_constant_expression_binary() {
        let expr = binary(int_lit(1), BinaryOp::Add, int_lit(2));
        assert!(!is_constant_expression(&expr));
    }
}
