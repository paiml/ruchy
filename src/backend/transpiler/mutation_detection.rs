//! Mutation detection for transpiler
//!
//! This module provides functions to detect if variables are mutated
//! (reassigned or modified) within expression trees, and whether a method
//! body mutates `self` (for `&mut self` receiver inference).

use crate::frontend::ast::{Expr, ExprKind};
use std::collections::HashSet;

/// Std methods that take `&mut self`. A call of one of these on a field of
/// `self` (`self.items.push(x)`, `self.buf.push_str(s)`, `self.map.entry(k)`)
/// mutates `self`. The list is deliberately closed: a method not listed here
/// is treated as non-mutating unless it is a mutating method of the same impl.
pub const MUTATING_STD_METHODS: &[&str] = &[
    "push", "pop", "insert", "remove", "clear", "extend", "append", "truncate", "sort", "sort_by",
    "reverse", "dedup", "retain", "drain", "push_str", "swap", "entry",
];

/// Checks if a variable is mutated (reassigned or modified) in an expression tree
///
/// This function traverses the AST recursively to detect writes through a
/// place rooted at the variable:
/// - Direct assignments (`x = value`, `x.a.b = value`, `x.v[i] = value`)
/// - Compound assignments (`x += 1`, `x.f -= 1`, etc.)
/// - Increment/decrement operations (`x++`, `++x`, `x.f--`, `--x.f`)
///
/// # Examples
/// ```ignore
/// use ruchy::backend::transpiler::mutation_detection::is_variable_mutated;
/// let expr = parse("x = 5");
/// assert!(is_variable_mutated("x", &expr));
/// ```
pub fn is_variable_mutated(name: &str, expr: &Expr) -> bool {
    any_expr(expr, &|e| writes_place_rooted_at(name, e))
}

/// True when a method body mutates `self`: a write through a place rooted at
/// `self`, a [`MUTATING_STD_METHODS`] call on a field of `self`, or a call
/// `self.m(..)` where `m` is in `mutating_methods` (methods of the same impl
/// already known to mutate `self`).
pub fn is_self_mutated(body: &Expr, mutating_methods: &HashSet<String>) -> bool {
    any_expr(body, &|e| {
        writes_place_rooted_at("self", e) || is_self_mutating_call(e, mutating_methods)
    })
}

/// Fixpoint over the methods of one impl/trait: the set of method names that
/// mutate `self`, directly or by calling another mutating method of the set.
/// `seed` holds methods known to mutate up front (declared `&mut self`);
/// `candidates` are the `(name, body)` pairs whose receiver is inferred.
pub fn mutating_self_methods(
    candidates: &[(&str, &Expr)],
    seed: HashSet<String>,
) -> HashSet<String> {
    let mut mutating = seed;
    loop {
        let newly: Vec<String> = candidates
            .iter()
            .filter(|(name, body)| !mutating.contains(*name) && is_self_mutated(body, &mutating))
            .map(|(name, _)| (*name).to_string())
            .collect();
        if newly.is_empty() {
            return mutating;
        }
        mutating.extend(newly);
    }
}

/// `self.<m>(..)` with `m` a mutating sibling, or `self.<place>.<m>(..)` with
/// `m` a mutating std method.
fn is_self_mutating_call(expr: &Expr, mutating_methods: &HashSet<String>) -> bool {
    let ExprKind::MethodCall {
        receiver, method, ..
    } = &expr.kind
    else {
        return false;
    };
    match &receiver.kind {
        ExprKind::Identifier(root) => root == "self" && mutating_methods.contains(method),
        _ => {
            place_root(receiver) == Some("self") && MUTATING_STD_METHODS.contains(&method.as_str())
        }
    }
}

/// Assignment, compound assignment or `++`/`--` whose target place is rooted at `name`.
fn writes_place_rooted_at(name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Assign { target, .. }
        | ExprKind::CompoundAssign { target, .. }
        | ExprKind::PreIncrement { target }
        | ExprKind::PostIncrement { target }
        | ExprKind::PreDecrement { target }
        | ExprKind::PostDecrement { target } => place_root(target) == Some(name),
        _ => false,
    }
}

/// Root variable of a place expression: `x`, `x.a.b`, `x[i]`, `x.v[i].c` all give `x`.
fn place_root(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.as_str()),
        ExprKind::FieldAccess { object, .. } | ExprKind::IndexAccess { object, .. } => {
            place_root(object)
        }
        _ => None,
    }
}

/// True when `pred` holds for `expr` or any expression nested in it.
fn any_expr(expr: &Expr, pred: &dyn Fn(&Expr) -> bool) -> bool {
    pred(expr) || sub_expressions(expr).into_iter().any(|e| any_expr(e, pred))
}

/// The direct sub-expressions searched for mutations.
fn sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Block(exprs) | ExprKind::List(exprs) | ExprKind::Tuple(exprs) => {
            exprs.iter().collect()
        }
        ExprKind::Macro { args, .. } => args.iter().collect(),
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => with_optional(vec![condition, then_branch], else_branch.as_deref()),
        ExprKind::IfLet {
            expr,
            then_branch,
            else_branch,
            ..
        } => with_optional(vec![expr, then_branch], else_branch.as_deref()),
        ExprKind::While {
            condition, body, ..
        } => vec![&**condition, &**body],
        ExprKind::WhileLet { expr, body, .. }
        | ExprKind::For {
            iter: expr, body, ..
        } => {
            vec![&**expr, &**body]
        }
        ExprKind::Match { expr, arms } => std::iter::once(&**expr)
            .chain(arms.iter().map(|arm| &*arm.body))
            .collect(),
        ExprKind::Let { value, body, .. } | ExprKind::LetPattern { value, body, .. } => {
            vec![&**value, &**body]
        }
        ExprKind::Binary { left, right, .. } => vec![&**left, &**right],
        ExprKind::Loop { body, .. }
        | ExprKind::Function { body, .. }
        | ExprKind::Lambda { body, .. }
        | ExprKind::Unary { operand: body, .. }
        | ExprKind::Try { expr: body }
        | ExprKind::Await { expr: body }
        | ExprKind::FieldAccess { object: body, .. } => vec![&**body],
        // An assignment evaluates its target place as well as its value, so a
        // mutation nested in either is found (e.g. `v[{ self.n += 1; 0 }] = 2`).
        ExprKind::Assign { target, value } | ExprKind::CompoundAssign { target, value, .. } => {
            vec![&**target, &**value]
        }
        ExprKind::IndexAccess { object, index } => vec![&**object, &**index],
        ExprKind::Return { value } => with_optional(Vec::new(), value.as_deref()),
        ExprKind::Call { func, args } => std::iter::once(&**func).chain(args).collect(),
        ExprKind::MethodCall { receiver, args, .. } => {
            std::iter::once(&**receiver).chain(args).collect()
        }
        _ => value_sub_expressions(expr),
    }
}

/// Sub-expressions of the value-building forms (constructors, casts, literals
/// with fields, ranges), so a mutation nested inside one is not missed.
fn value_sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Throw { expr: e }
        | ExprKind::Ok { value: e }
        | ExprKind::Err { error: e }
        | ExprKind::Some { value: e }
        | ExprKind::TypeCast { expr: e, .. }
        | ExprKind::Spawn { actor: e }
        | ExprKind::OptionalFieldAccess { object: e, .. } => vec![&**e],
        ExprKind::Ternary {
            condition,
            true_expr,
            false_expr,
        } => {
            vec![&**condition, &**true_expr, &**false_expr]
        }
        ExprKind::Send { actor, message } => vec![&**actor, &**message],
        ExprKind::ArrayInit { value, size } => vec![&**value, &**size],
        ExprKind::Range { start, end, .. } => vec![&**start, &**end],
        ExprKind::Break { value, .. } => value.as_deref().into_iter().collect(),
        ExprKind::Set(items) => items.iter().collect(),
        ExprKind::OptionalMethodCall { receiver, args, .. } => {
            std::iter::once(&**receiver).chain(args).collect()
        }
        _ => literal_sub_expressions(expr),
    }
}

/// Sub-expressions of struct, object and interpolated-string literals.
fn literal_sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::StructLiteral { fields, base, .. } => fields
            .iter()
            .map(|(_, e)| e)
            .chain(base.as_deref())
            .collect(),
        ExprKind::ObjectLiteral { fields } => fields
            .iter()
            .map(|f| match f {
                crate::frontend::ast::ObjectField::KeyValue { value, .. } => value,
                crate::frontend::ast::ObjectField::Spread { expr } => expr,
            })
            .collect(),
        ExprKind::StringInterpolation { parts } => parts
            .iter()
            .filter_map(|p| match p {
                crate::frontend::ast::StringPart::Text(_) => None,
                crate::frontend::ast::StringPart::Expr(e)
                | crate::frontend::ast::StringPart::ExprWithFormat { expr: e, .. } => Some(&**e),
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// `required` (as plain references) followed by `optional` when present.
fn with_optional<'a>(required: Vec<&'a Box<Expr>>, optional: Option<&'a Expr>) -> Vec<&'a Expr> {
    required.into_iter().map(|e| &**e).chain(optional).collect()
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
            contracts: Vec::new(),
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

    // ===== EXTREME TDD Round 156 - Additional Mutation Detection Tests =====

    #[test]
    fn test_function_body_mutation() {
        use crate::frontend::ast::{Param, Type, TypeKind};
        let func = make_expr(ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![Param {
                pattern: Pattern::Identifier("arg".to_string()),
                ty: Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
            body: Box::new(assign(ident("x"), int_lit(1))),
            is_async: false,
            is_pub: false,
        });
        assert!(is_variable_mutated("x", &func));
    }

    #[test]
    fn test_function_body_no_mutation() {
        use crate::frontend::ast::{Param, Type, TypeKind};
        let func = make_expr(ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![Param {
                pattern: Pattern::Identifier("arg".to_string()),
                ty: Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::default(),
                },
                span: Span::default(),
                is_mutable: false,
                default_value: None,
            }],
            return_type: None,
            body: Box::new(int_lit(42)),
            is_async: false,
            is_pub: false,
        });
        assert!(!is_variable_mutated("x", &func));
    }

    #[test]
    fn test_let_pattern_body_mutation() {
        let let_pat = make_expr(ExprKind::LetPattern {
            pattern: Pattern::Identifier("y".to_string()),
            type_annotation: None,
            value: Box::new(int_lit(5)),
            body: Box::new(assign(ident("x"), int_lit(1))),
            is_mutable: false,
            else_block: None,
        });
        assert!(is_variable_mutated("x", &let_pat));
    }

    #[test]
    fn test_let_pattern_body_no_mutation() {
        let let_pat = make_expr(ExprKind::LetPattern {
            pattern: Pattern::Identifier("y".to_string()),
            type_annotation: None,
            value: Box::new(int_lit(5)),
            body: Box::new(int_lit(42)),
            is_mutable: false,
            else_block: None,
        });
        assert!(!is_variable_mutated("x", &let_pat));
    }

    #[test]
    fn test_match_multiple_arms() {
        let match_expr = match_expr(
            int_lit(1),
            vec![
                match_arm(Pattern::Wildcard, int_lit(0)),
                match_arm(Pattern::Wildcard, assign(ident("x"), int_lit(5))),
            ],
        );
        assert!(is_variable_mutated("x", &match_expr));
    }

    #[test]
    fn test_binary_nested_mutation() {
        let nested = binary(
            binary(int_lit(1), assign(ident("x"), int_lit(2))),
            int_lit(3),
        );
        assert!(is_variable_mutated("x", &nested));
    }

    #[test]
    fn test_unary_nested_mutation() {
        let nested = unary(unary(assign(ident("x"), int_lit(1))));
        assert!(is_variable_mutated("x", &nested));
    }

    #[test]
    fn test_call_multiple_args_mutation() {
        let expr = call(
            ident("f"),
            vec![int_lit(1), assign(ident("x"), int_lit(2)), int_lit(3)],
        );
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_method_call_multiple_args() {
        let expr = method_call(
            ident("obj"),
            "method",
            vec![int_lit(1), assign(ident("x"), int_lit(2))],
        );
        assert!(is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_deeply_nested_for_loop() {
        let for_expr = for_expr(
            "i",
            ident("items"),
            block(vec![if_expr(
                bool_lit(true),
                assign(ident("x"), int_lit(1)),
                None,
            )]),
        );
        assert!(is_variable_mutated("x", &for_expr));
    }

    #[test]
    fn test_while_nested_block_mutation() {
        let while_expr_nested = while_expr(
            bool_lit(true),
            block(vec![block(vec![assign(ident("x"), int_lit(1))])]),
        );
        assert!(is_variable_mutated("x", &while_expr_nested));
    }

    #[test]
    fn test_pre_decrement_other_var() {
        let expr = pre_decrement(ident("y"));
        assert!(!is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_post_decrement_other_var() {
        let expr = post_decrement(ident("y"));
        assert!(!is_variable_mutated("x", &expr));
    }

    #[test]
    fn test_compound_assign_nested_target() {
        let nested_compound = make_expr(ExprKind::CompoundAssign {
            target: Box::new(ident("x")),
            value: Box::new(ident("y")),
            op: BinaryOp::Multiply,
        });
        assert!(is_variable_mutated("x", &nested_compound));
    }

    #[test]
    fn test_self_mutation_nested_in_an_index_target() {
        // v[{ self.n += 1 }] = 2 — the index evaluates a mutation of `self`.
        let self_n = make_expr(ExprKind::FieldAccess {
            object: Box::new(ident("self")),
            field: "n".to_string(),
        });
        let bump = compound_assign(self_n, int_lit(1));
        let place = make_expr(ExprKind::IndexAccess {
            object: Box::new(ident("v")),
            index: Box::new(bump),
        });
        let stmt = assign(place, int_lit(2));
        assert!(is_variable_mutated("self", &stmt));
    }

    #[test]
    fn test_self_mutation_nested_in_value_forms() {
        let bump = || {
            let self_n = make_expr(ExprKind::FieldAccess {
                object: Box::new(ident("self")),
                field: "n".to_string(),
            });
            compound_assign(self_n, int_lit(1))
        };
        let forms = vec![
            make_expr(ExprKind::Some {
                value: Box::new(bump()),
            }),
            make_expr(ExprKind::TypeCast {
                expr: Box::new(bump()),
                target_type: "i64".to_string(),
            }),
            make_expr(ExprKind::Ternary {
                condition: Box::new(int_lit(1)),
                true_expr: Box::new(bump()),
                false_expr: Box::new(int_lit(0)),
            }),
            make_expr(ExprKind::Range {
                start: Box::new(bump()),
                end: Box::new(int_lit(3)),
                inclusive: false,
            }),
            make_expr(ExprKind::StructLiteral {
                name: "P".to_string(),
                fields: vec![("x".to_string(), bump())],
                base: None,
            }),
        ];
        for form in &forms {
            assert!(
                is_variable_mutated("self", form),
                "missed in {:?}",
                form.kind
            );
        }
    }

    #[test]
    fn test_assign_nested_value() {
        let nested_assign = assign(ident("y"), assign(ident("x"), int_lit(1)));
        // `y = (x = 1)` assigns x: an assignment mutates its target wherever it
        // appears (G2BFA2; this test used to pin the narrower walk that missed it).
        assert!(is_variable_mutated("x", &nested_assign));
    }

    #[test]
    fn test_empty_match() {
        let empty_match = match_expr(int_lit(1), vec![]);
        assert!(!is_variable_mutated("x", &empty_match));
    }

    #[test]
    fn test_return_expression() {
        let ret = make_expr(ExprKind::Return {
            value: Some(Box::new(assign(ident("x"), int_lit(1)))),
        });
        // `return (x = 1)` assigns x before returning (G2BFA2).
        assert!(is_variable_mutated("x", &ret));
    }
}
