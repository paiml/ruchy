//! Mutation detection for transpiler
//!
//! This module provides functions to detect if variables are mutated
//! (reassigned or modified) within expression trees.

use crate::frontend::ast::{Expr, ExprKind};

/// Checks if a variable is mutated (reassigned or modified) in an expression tree
///
/// This function traverses the AST recursively to detect:
/// - Direct assignments (`x = value`)
/// - Compound assignments (`x += 1`, `x -= 1`, etc.)
/// - Increment/decrement operations (`x++`, `++x`, `x--`, `--x`)
///
/// # Examples
/// ```ignore
/// use ruchy::backend::transpiler::mutation_detection::is_variable_mutated;
/// let expr = parse("x = 5");
/// assert!(is_variable_mutated("x", &expr));
/// ```
pub fn is_variable_mutated(name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        // Direct assignment to the variable
        ExprKind::Assign { target, value: _ } => {
            if let ExprKind::Identifier(var_name) = &target.kind {
                if var_name == name {
                    return true;
                }
            }
            false
        }
        // Compound assignment (+=, -=, etc.)
        ExprKind::CompoundAssign {
            target, value: _, ..
        } => {
            if let ExprKind::Identifier(var_name) = &target.kind {
                if var_name == name {
                    return true;
                }
            }
            false
        }
        // Pre/Post increment/decrement
        ExprKind::PreIncrement { target }
        | ExprKind::PostIncrement { target }
        | ExprKind::PreDecrement { target }
        | ExprKind::PostDecrement { target } => {
            if let ExprKind::Identifier(var_name) = &target.kind {
                if var_name == name {
                    return true;
                }
            }
            false
        }
        // Check in blocks
        ExprKind::Block(exprs) => exprs.iter().any(|e| is_variable_mutated(name, e)),
        // Check in if branches
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            is_variable_mutated(name, condition)
                || is_variable_mutated(name, then_branch)
                || else_branch
                    .as_ref()
                    .is_some_and(|e| is_variable_mutated(name, e))
        }
        // Check in while loops
        ExprKind::While {
            condition, body, ..
        } => is_variable_mutated(name, condition) || is_variable_mutated(name, body),
        // Check in for loops
        ExprKind::For { body, .. } => is_variable_mutated(name, body),
        // Check in match expressions
        ExprKind::Match { expr, arms } => {
            is_variable_mutated(name, expr)
                || arms.iter().any(|arm| is_variable_mutated(name, &arm.body))
        }
        // Check in nested let expressions
        ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
            is_variable_mutated(name, body)
        }
        // Check in function bodies
        ExprKind::Function { body, .. } => is_variable_mutated(name, body),
        // Check in lambda bodies
        ExprKind::Lambda { body, .. } => is_variable_mutated(name, body),
        // Check binary operations
        ExprKind::Binary { left, right, .. } => {
            is_variable_mutated(name, left) || is_variable_mutated(name, right)
        }
        // Check unary operations
        ExprKind::Unary { operand, .. } => is_variable_mutated(name, operand),
        // Check function/method calls
        ExprKind::Call { func, args } => {
            is_variable_mutated(name, func) || args.iter().any(|a| is_variable_mutated(name, a))
        }
        ExprKind::MethodCall { receiver, args, .. } => {
            is_variable_mutated(name, receiver)
                || args.iter().any(|a| is_variable_mutated(name, a))
        }
        // Other expressions don't contain mutations
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Literal, MatchArm, Pattern, Span, UnaryOp};

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

    fn ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn assign(target: Expr, value: Expr) -> Expr {
        make_expr(ExprKind::Assign {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn compound_assign(target: Expr, value: Expr) -> Expr {
        make_expr(ExprKind::CompoundAssign {
            target: Box::new(target),
            value: Box::new(value),
            op: BinaryOp::Add,
        })
    }

    fn pre_increment(target: Expr) -> Expr {
        make_expr(ExprKind::PreIncrement {
            target: Box::new(target),
        })
    }

    fn post_increment(target: Expr) -> Expr {
        make_expr(ExprKind::PostIncrement {
            target: Box::new(target),
        })
    }

    fn pre_decrement(target: Expr) -> Expr {
        make_expr(ExprKind::PreDecrement {
            target: Box::new(target),
        })
    }

    fn post_decrement(target: Expr) -> Expr {
        make_expr(ExprKind::PostDecrement {
            target: Box::new(target),
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

    fn while_expr(condition: Expr, body: Expr) -> Expr {
        make_expr(ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
            label: None,
        })
    }

    fn for_expr(var: &str, iter: Expr, body: Expr) -> Expr {
        make_expr(ExprKind::For {
            var: var.to_string(),
            pattern: None,
            iter: Box::new(iter),
            body: Box::new(body),
            label: None,
        })
    }

    fn match_expr(expr: Expr, arms: Vec<MatchArm>) -> Expr {
        make_expr(ExprKind::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn match_arm(pattern: Pattern, body: Expr) -> MatchArm {
        MatchArm {
            pattern,
            guard: None,
            body: Box::new(body),
            span: Span::default(),
        }
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

    fn lambda(params: Vec<&str>, body: Expr) -> Expr {
        use crate::frontend::ast::{Param, Type, TypeKind};
        make_expr(ExprKind::Lambda {
            params: params
                .into_iter()
                .map(|name| Param {
                    pattern: Pattern::Identifier(name.to_string()),
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
        })
    }

    fn binary(left: Expr, right: Expr) -> Expr {
        make_expr(ExprKind::Binary {
            left: Box::new(left),
            op: BinaryOp::Add,
            right: Box::new(right),
        })
    }

    fn unary(operand: Expr) -> Expr {
        make_expr(ExprKind::Unary {
            op: UnaryOp::Negate,
            operand: Box::new(operand),
        })
    }

    fn call(func: Expr, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Call {
            func: Box::new(func),
            args,
        })
    }

    fn method_call(receiver: Expr, method: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method: method.to_string(),
            args,
        })
    }

    fn bool_lit(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    // ==================== Direct Assignment Tests ====================

    #[test]
    fn test_direct_assignment_mutates() {
        let expr = assign(ident("x"), int_lit(5));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_direct_assignment_other_var() {
        let expr = assign(ident("y"), int_lit(5));
        assert!(!is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_no_assignment() {
        let expr = int_lit(42);
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Compound Assignment Tests ====================

    #[test]
    fn test_compound_assignment_mutates() {
        let expr = compound_assign(ident("x"), int_lit(1));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_compound_assignment_other_var() {
        let expr = compound_assign(ident("y"), int_lit(1));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Increment/Decrement Tests ====================

    #[test]
    fn test_pre_increment_mutates() {
        let expr = pre_increment(ident("x"));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_post_increment_mutates() {
        let expr = post_increment(ident("x"));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_pre_decrement_mutates() {
        let expr = pre_decrement(ident("x"));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_post_decrement_mutates() {
        let expr = post_decrement(ident("x"));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_increment_other_var() {
        let expr = pre_increment(ident("y"));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Block Tests ====================

    #[test]
    fn test_block_with_mutation() {
        let expr = block(vec![int_lit(1), assign(ident("x"), int_lit(5))]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_block_without_mutation() {
        let expr = block(vec![int_lit(1), int_lit(2)]);
        assert!(!is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_empty_block() {
        let expr = block(vec![]);
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== If Expression Tests ====================

    #[test]
    fn test_if_condition_mutation() {
        let expr = if_expr(assign(ident("x"), int_lit(1)), int_lit(2), None);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_if_then_mutation() {
        let expr = if_expr(bool_lit(true), assign(ident("x"), int_lit(1)), None);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_if_else_mutation() {
        let expr = if_expr(
            bool_lit(false),
            int_lit(0),
            Some(assign(ident("x"), int_lit(1))),
        );
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_if_no_mutation() {
        let expr = if_expr(bool_lit(true), int_lit(1), Some(int_lit(2)));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== While Loop Tests ====================

    #[test]
    fn test_while_condition_mutation() {
        let expr = while_expr(assign(ident("x"), int_lit(1)), int_lit(0));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_while_body_mutation() {
        let expr = while_expr(bool_lit(true), assign(ident("x"), int_lit(1)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_while_no_mutation() {
        let expr = while_expr(bool_lit(false), int_lit(0));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== For Loop Tests ====================

    #[test]
    fn test_for_body_mutation() {
        let expr = for_expr("i", ident("items"), assign(ident("x"), int_lit(1)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_for_no_mutation() {
        let expr = for_expr("i", ident("items"), int_lit(0));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Match Expression Tests ====================

    #[test]
    fn test_match_expr_mutation() {
        let expr = match_expr(
            assign(ident("x"), int_lit(1)),
            vec![match_arm(Pattern::Wildcard, int_lit(0))],
        );
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_match_arm_mutation() {
        let expr = match_expr(
            int_lit(1),
            vec![match_arm(Pattern::Wildcard, assign(ident("x"), int_lit(5)))],
        );
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_match_no_mutation() {
        let expr = match_expr(int_lit(1), vec![match_arm(Pattern::Wildcard, int_lit(0))]);
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Let Expression Tests ====================

    #[test]
    fn test_let_body_mutation() {
        let expr = let_expr("y", int_lit(5), assign(ident("x"), int_lit(1)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_let_no_mutation() {
        let expr = let_expr("y", int_lit(5), int_lit(0));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Lambda Tests ====================

    #[test]
    fn test_lambda_body_mutation() {
        let expr = lambda(vec!["a"], assign(ident("x"), int_lit(1)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_lambda_no_mutation() {
        let expr = lambda(vec!["a"], int_lit(0));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Binary Expression Tests ====================

    #[test]
    fn test_binary_left_mutation() {
        let expr = binary(assign(ident("x"), int_lit(1)), int_lit(2));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_binary_right_mutation() {
        let expr = binary(int_lit(1), assign(ident("x"), int_lit(2)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_binary_no_mutation() {
        let expr = binary(int_lit(1), int_lit(2));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Unary Expression Tests ====================

    #[test]
    fn test_unary_mutation() {
        let expr = unary(assign(ident("x"), int_lit(1)));
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_unary_no_mutation() {
        let expr = unary(int_lit(1));
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Call Expression Tests ====================

    #[test]
    fn test_call_func_mutation() {
        let expr = call(assign(ident("x"), int_lit(1)), vec![]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_call_arg_mutation() {
        let expr = call(ident("f"), vec![assign(ident("x"), int_lit(1))]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_call_no_mutation() {
        let expr = call(ident("f"), vec![int_lit(1)]);
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Method Call Tests ====================

    #[test]
    fn test_method_receiver_mutation() {
        let expr = method_call(assign(ident("x"), int_lit(1)), "foo", vec![]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_method_arg_mutation() {
        let expr = method_call(ident("obj"), "foo", vec![assign(ident("x"), int_lit(1))]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_method_no_mutation() {
        let expr = method_call(ident("obj"), "foo", vec![int_lit(1)]);
        assert!(!is_variable_mutated("x", &expr));
    }

    // ==================== Nested/Complex Tests ====================

    #[test]
    fn test_deeply_nested_mutation() {
        // block { if true { while true { x = 1 } } }
        let inner = while_expr(bool_lit(true), assign(ident("x"), int_lit(1)));
        let mid = if_expr(bool_lit(true), inner, None);
        let expr = block(vec![mid]);
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_multiple_vars() {
        let expr = block(vec![
            assign(ident("a"), int_lit(1)),
            assign(ident("b"), int_lit(2)),
            assign(ident("c"), int_lit(3)),
        ]);
        assert!(is_variable_mutated("a", &expr));
        assert!(is_variable_mutated("b", &expr));
        assert!(is_variable_mutated("c", &expr));
        assert!(!is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_identifier_only() {
        let expr = ident("x");
        assert!(!is_variable_mutated("x", &expr));
    }
}
