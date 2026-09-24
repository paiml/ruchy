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
///
/// Every expression kind that holds a sub-expression evaluated as part of the
/// enclosing body is walked. The kinds are split across small category
/// helpers chained through their `_ =>` fallbacks: this function (sequences,
/// control flow, single-child wrappers), [`binding_sub_expressions`] (let,
/// try/catch, effect handlers, pipelines, two-operand forms),
/// [`call_sub_expressions`] (calls, actor asks, slices, data frame
/// operations) and [`literal_sub_expressions`] (struct, object, string,
/// comprehension and data frame literals).
///
/// Deliberately NOT walked:
/// - the item definitions `Module`, `Export`, `ExportDefault`, `Struct`,
///   `TupleStruct`, `Class`, `Enum`, `Trait`, `Impl`, `Actor`, `Effect` and
///   `Extension`: they are top-level items whose bodies are other methods
///   with their own receivers, so a `self` write inside one is not a write to
///   the enclosing method's `self`;
/// - the leaves `Literal`, `Identifier`, `QualifiedName`, `None`,
///   `Continue`, `Command`, `ModuleDeclaration`, `Import`, `ImportAll`,
///   `ImportDefault`, `ExportList`, `ReExport` and `TypeAlias`, which hold no
///   sub-expression.
fn sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Block(exprs)
        | ExprKind::List(exprs)
        | ExprKind::Tuple(exprs)
        | ExprKind::Set(exprs)
        | ExprKind::InfraBlock { body: exprs }
        | ExprKind::Macro { args: exprs, .. }
        | ExprKind::MacroInvocation { args: exprs, .. } => exprs.iter().collect(),
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
        ExprKind::Match { expr, arms } => std::iter::once(&**expr)
            .chain(
                arms.iter()
                    .flat_map(|arm| arm.guard.as_deref().into_iter().chain([&*arm.body])),
            )
            .collect(),
        ExprKind::Return { value } | ExprKind::Break { value, .. } | ExprKind::Yield { value } => {
            value.as_deref().into_iter().collect()
        }
        _ => single_sub_expression(expr)
            .map(|e| vec![e])
            .unwrap_or_else(|| binding_sub_expressions(expr)),
    }
}

/// The one sub-expression of a single-child form (wrappers, closures, `++`/`--`).
fn single_sub_expression(expr: &Expr) -> Option<&Expr> {
    match &expr.kind {
        ExprKind::Loop { body: e, .. }
        | ExprKind::Function { body: e, .. }
        | ExprKind::Lambda { body: e, .. }
        | ExprKind::AsyncLambda { body: e, .. }
        | ExprKind::AsyncBlock { body: e }
        | ExprKind::Lazy { expr: e }
        | ExprKind::Unary { operand: e, .. }
        | ExprKind::Try { expr: e }
        | ExprKind::Await { expr: e }
        | ExprKind::FieldAccess { object: e, .. }
        | ExprKind::OptionalFieldAccess { object: e, .. }
        | ExprKind::Throw { expr: e }
        | ExprKind::Ok { value: e }
        | ExprKind::Err { error: e }
        | ExprKind::Some { value: e }
        | ExprKind::TypeCast { expr: e, .. }
        | ExprKind::Spawn { actor: e }
        | ExprKind::Spread { expr: e }
        | ExprKind::Signal { initial_value: e } => Some(&**e),
        // The target place of `++`/`--` is evaluated too (`v[{ self.n += 1; 0 }]++`).
        ExprKind::PreIncrement { target: e }
        | ExprKind::PostIncrement { target: e }
        | ExprKind::PreDecrement { target: e }
        | ExprKind::PostDecrement { target: e } => Some(&**e),
        _ => None,
    }
}

/// Sub-expressions of the binding, handler and two-operand forms.
fn binding_sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Let {
            value,
            body,
            else_block,
            ..
        }
        | ExprKind::LetPattern {
            value,
            body,
            else_block,
            ..
        } => with_optional(vec![value, body], else_block.as_deref()),
        ExprKind::TryCatch {
            try_block,
            catch_clauses,
            finally_block,
        } => std::iter::once(&**try_block)
            .chain(catch_clauses.iter().map(|c| &*c.body))
            .chain(finally_block.as_deref())
            .collect(),
        ExprKind::Handle { expr, handlers } => std::iter::once(&**expr)
            .chain(handlers.iter().map(|h| &*h.body))
            .collect(),
        ExprKind::Pipeline { expr, stages } => std::iter::once(&**expr)
            .chain(stages.iter().map(|s| &*s.op))
            .collect(),
        ExprKind::Ternary {
            condition,
            true_expr,
            false_expr,
        } => vec![&**condition, &**true_expr, &**false_expr],
        // An assignment evaluates its target place as well as its value, so a
        // mutation nested in either is found (e.g. `v[{ self.n += 1; 0 }] = 2`).
        ExprKind::Assign {
            target: a,
            value: b,
        }
        | ExprKind::CompoundAssign {
            target: a,
            value: b,
            ..
        }
        | ExprKind::Binary {
            left: a, right: b, ..
        }
        | ExprKind::IndexAccess {
            object: a,
            index: b,
        }
        | ExprKind::While {
            condition: a,
            body: b,
            ..
        }
        | ExprKind::WhileLet {
            expr: a, body: b, ..
        }
        | ExprKind::For {
            iter: a, body: b, ..
        }
        | ExprKind::Range {
            start: a, end: b, ..
        }
        | ExprKind::ArrayInit { value: a, size: b }
        | ExprKind::VecRepeat { value: a, count: b }
        | ExprKind::Send {
            actor: a,
            message: b,
        }
        | ExprKind::ActorSend {
            actor: a,
            message: b,
        }
        | ExprKind::ActorQuery {
            actor: a,
            message: b,
        } => vec![&**a, &**b],
        _ => call_sub_expressions(expr),
    }
}

/// Sub-expressions of calls, actor asks, slices and data frame operations.
fn call_sub_expressions(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Call { func: r, args }
        | ExprKind::MethodCall {
            receiver: r, args, ..
        }
        | ExprKind::OptionalMethodCall {
            receiver: r, args, ..
        } => std::iter::once(&**r).chain(args).collect(),
        ExprKind::Ask {
            actor,
            message,
            timeout,
        } => with_optional(vec![actor, message], timeout.as_deref()),
        ExprKind::Slice { object, start, end } => std::iter::once(&**object)
            .chain(start.as_deref())
            .chain(end.as_deref())
            .collect(),
        ExprKind::DataFrameOperation { source, operation } => std::iter::once(&**source)
            .chain(data_frame_op_expr(operation))
            .collect(),
        _ => literal_sub_expressions(expr),
    }
}

/// The expression argument of a data frame operation (`filter(pred)`, `join(other)`).
fn data_frame_op_expr(op: &crate::frontend::ast::DataFrameOp) -> Option<&Expr> {
    use crate::frontend::ast::DataFrameOp;
    match op {
        DataFrameOp::Filter(e) | DataFrameOp::Join { other: e, .. } => Some(&**e),
        _ => None,
    }
}

/// Sub-expressions of struct, object, interpolated-string, comprehension and
/// data frame literals.
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
        ExprKind::ListComprehension { element, clauses }
        | ExprKind::SetComprehension { element, clauses } => std::iter::once(&**element)
            .chain(clause_exprs(clauses))
            .collect(),
        ExprKind::DictComprehension {
            key,
            value,
            clauses,
        } => [&**key, &**value]
            .into_iter()
            .chain(clause_exprs(clauses))
            .collect(),
        ExprKind::DataFrame { columns } => columns.iter().flat_map(|c| &c.values).collect(),
        _ => Vec::new(),
    }
}

/// The iterables and filter conditions of comprehension clauses.
fn clause_exprs(
    clauses: &[crate::frontend::ast::ComprehensionClause],
) -> impl Iterator<Item = &Expr> {
    clauses
        .iter()
        .flat_map(|c| std::iter::once(&*c.iterable).chain(c.condition.as_deref()))
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

    // ==================== Walk-every-kind table (G2BFA2) ====================

    /// `self.n += 1`: the mutation every table case nests.
    fn bump() -> Expr {
        let self_n = make_expr(ExprKind::FieldAccess {
            object: Box::new(ident("self")),
            field: "n".to_string(),
        });
        compound_assign(self_n, int_lit(1))
    }

    fn bx(e: Expr) -> Box<Expr> {
        Box::new(e)
    }

    fn clause(
        iterable: Expr,
        condition: Option<Expr>,
    ) -> crate::frontend::ast::ComprehensionClause {
        crate::frontend::ast::ComprehensionClause {
            variable: "i".to_string(),
            iterable: bx(iterable),
            condition: condition.map(bx),
        }
    }

    /// One case per walked expression kind (and per walked field where a kind
    /// holds several), each nesting `self.n += 1`. Deleting any walk arm fails
    /// exactly the cases naming that kind.
    #[allow(clippy::too_many_lines)]
    fn walk_cases() -> Vec<(&'static str, Expr)> {
        use crate::frontend::ast::{
            CatchClause, DataFrameColumn, DataFrameOp, EffectHandler, JoinType, ObjectField,
            PipelineStage, StringPart,
        };
        let e = make_expr;
        let wild_arm = |guard: Option<Expr>, body: Expr| MatchArm {
            pattern: Pattern::Wildcard,
            guard: guard.map(bx),
            body: bx(body),
            span: Span::default(),
        };
        let let_else = |else_block: Expr| ExprKind::Let {
            name: "y".to_string(),
            type_annotation: None,
            value: bx(int_lit(1)),
            body: bx(int_lit(0)),
            is_mutable: false,
            else_block: Some(bx(else_block)),
        };
        let try_catch = |t: Expr, c: Expr, f: Option<Expr>| ExprKind::TryCatch {
            try_block: bx(t),
            catch_clauses: vec![CatchClause {
                pattern: Pattern::Wildcard,
                body: bx(c),
            }],
            finally_block: f.map(bx),
        };
        vec![
            // ---- walked before commit 0383bc72
            ("Block", block(vec![bump()])),
            ("List", e(ExprKind::List(vec![bump()]))),
            ("Tuple", e(ExprKind::Tuple(vec![bump()]))),
            (
                "Macro",
                e(ExprKind::Macro {
                    name: "sql".into(),
                    args: vec![bump()],
                }),
            ),
            ("If", if_expr(bool_lit(true), bump(), None)),
            (
                "IfLet",
                e(ExprKind::IfLet {
                    pattern: Pattern::Wildcard,
                    expr: bx(int_lit(1)),
                    then_branch: bx(bump()),
                    else_branch: None,
                }),
            ),
            ("While", while_expr(bool_lit(true), bump())),
            (
                "WhileLet",
                e(ExprKind::WhileLet {
                    label: None,
                    pattern: Pattern::Wildcard,
                    expr: bx(int_lit(1)),
                    body: bx(bump()),
                }),
            ),
            ("For", for_expr("i", ident("v"), bump())),
            (
                "Match arm body",
                match_expr(int_lit(1), vec![wild_arm(None, bump())]),
            ),
            ("Let value", let_expr("y", bump(), int_lit(0))),
            ("Binary", binary(int_lit(1), bump())),
            (
                "Loop",
                e(ExprKind::Loop {
                    label: None,
                    body: bx(bump()),
                }),
            ),
            ("Lambda", lambda(vec![], bump())),
            ("Unary", unary(bump())),
            ("Try", e(ExprKind::Try { expr: bx(bump()) })),
            ("Await", e(ExprKind::Await { expr: bx(bump()) })),
            ("CompoundAssign value", compound_assign(ident("z"), bump())),
            (
                "Return",
                e(ExprKind::Return {
                    value: Some(bx(bump())),
                }),
            ),
            ("Call", call(ident("f"), vec![bump()])),
            ("MethodCall", method_call(ident("o"), "m", vec![bump()])),
            // ---- arms added in commit 0383bc72
            ("Some", e(ExprKind::Some { value: bx(bump()) })),
            ("Ok", e(ExprKind::Ok { value: bx(bump()) })),
            ("Err", e(ExprKind::Err { error: bx(bump()) })),
            ("Throw", e(ExprKind::Throw { expr: bx(bump()) })),
            (
                "TypeCast",
                e(ExprKind::TypeCast {
                    expr: bx(bump()),
                    target_type: "i64".into(),
                }),
            ),
            (
                "Ternary",
                e(ExprKind::Ternary {
                    condition: bx(bool_lit(true)),
                    true_expr: bx(int_lit(0)),
                    false_expr: bx(bump()),
                }),
            ),
            ("Spawn", e(ExprKind::Spawn { actor: bx(bump()) })),
            (
                "OptionalFieldAccess",
                e(ExprKind::OptionalFieldAccess {
                    object: bx(bump()),
                    field: "f".into(),
                }),
            ),
            (
                "Send",
                e(ExprKind::Send {
                    actor: bx(ident("a")),
                    message: bx(bump()),
                }),
            ),
            (
                "ArrayInit",
                e(ExprKind::ArrayInit {
                    value: bx(int_lit(0)),
                    size: bx(bump()),
                }),
            ),
            (
                "Range",
                e(ExprKind::Range {
                    start: bx(int_lit(0)),
                    end: bx(bump()),
                    inclusive: false,
                }),
            ),
            (
                "Break",
                e(ExprKind::Break {
                    label: None,
                    value: Some(bx(bump())),
                }),
            ),
            ("Set", e(ExprKind::Set(vec![bump()]))),
            (
                "OptionalMethodCall",
                e(ExprKind::OptionalMethodCall {
                    receiver: bx(ident("o")),
                    method: "m".into(),
                    args: vec![bump()],
                }),
            ),
            (
                "StructLiteral",
                e(ExprKind::StructLiteral {
                    name: "P".into(),
                    fields: vec![("x".into(), bump())],
                    base: None,
                }),
            ),
            (
                "ObjectLiteral",
                e(ExprKind::ObjectLiteral {
                    fields: vec![ObjectField::KeyValue {
                        key: "k".into(),
                        value: bump(),
                    }],
                }),
            ),
            (
                "StringInterpolation",
                e(ExprKind::StringInterpolation {
                    parts: vec![StringPart::Expr(bx(bump()))],
                }),
            ),
            (
                "FieldAccess",
                e(ExprKind::FieldAccess {
                    object: bx(bump()),
                    field: "f".into(),
                }),
            ),
            (
                "IndexAccess",
                e(ExprKind::IndexAccess {
                    object: bx(ident("v")),
                    index: bx(bump()),
                }),
            ),
            (
                "Assign target",
                assign(
                    e(ExprKind::IndexAccess {
                        object: bx(ident("v")),
                        index: bx(bump()),
                    }),
                    int_lit(2),
                ),
            ),
            // ---- kinds the round-3 review found unwalked
            (
                "MacroInvocation",
                e(ExprKind::MacroInvocation {
                    name: "println".into(),
                    args: vec![bump()],
                }),
            ),
            (
                "VecRepeat",
                e(ExprKind::VecRepeat {
                    value: bx(int_lit(0)),
                    count: bx(bump()),
                }),
            ),
            ("TryCatch try_block", e(try_catch(bump(), int_lit(0), None))),
            (
                "TryCatch catch body",
                e(try_catch(int_lit(0), bump(), None)),
            ),
            (
                "TryCatch finally_block",
                e(try_catch(int_lit(0), int_lit(0), Some(bump()))),
            ),
            ("Let else_block", e(let_else(bump()))),
            (
                "LetPattern else_block",
                e(ExprKind::LetPattern {
                    pattern: Pattern::Wildcard,
                    type_annotation: None,
                    value: bx(int_lit(1)),
                    body: bx(int_lit(0)),
                    is_mutable: false,
                    else_block: Some(bx(bump())),
                }),
            ),
            (
                "MatchArm guard",
                match_expr(int_lit(1), vec![wild_arm(Some(bump()), int_lit(0))]),
            ),
            ("AsyncBlock", e(ExprKind::AsyncBlock { body: bx(bump()) })),
            (
                "AsyncLambda",
                e(ExprKind::AsyncLambda {
                    params: vec![],
                    body: bx(bump()),
                }),
            ),
            ("Lazy", e(ExprKind::Lazy { expr: bx(bump()) })),
            (
                "Slice object",
                e(ExprKind::Slice {
                    object: bx(bump()),
                    start: None,
                    end: None,
                }),
            ),
            (
                "Slice start",
                e(ExprKind::Slice {
                    object: bx(ident("v")),
                    start: Some(bx(bump())),
                    end: None,
                }),
            ),
            (
                "Slice end",
                e(ExprKind::Slice {
                    object: bx(ident("v")),
                    start: None,
                    end: Some(bx(bump())),
                }),
            ),
            ("Spread", e(ExprKind::Spread { expr: bx(bump()) })),
            (
                "Pipeline expr",
                e(ExprKind::Pipeline {
                    expr: bx(bump()),
                    stages: vec![],
                }),
            ),
            (
                "Pipeline stage",
                e(ExprKind::Pipeline {
                    expr: bx(ident("v")),
                    stages: vec![PipelineStage {
                        op: bx(bump()),
                        span: Span::default(),
                    }],
                }),
            ),
            (
                "ListComprehension element",
                e(ExprKind::ListComprehension {
                    element: bx(bump()),
                    clauses: vec![clause(ident("v"), None)],
                }),
            ),
            (
                "ListComprehension iterable",
                e(ExprKind::ListComprehension {
                    element: bx(ident("i")),
                    clauses: vec![clause(bump(), None)],
                }),
            ),
            (
                "ListComprehension condition",
                e(ExprKind::ListComprehension {
                    element: bx(ident("i")),
                    clauses: vec![clause(ident("v"), Some(bump()))],
                }),
            ),
            (
                "SetComprehension",
                e(ExprKind::SetComprehension {
                    element: bx(bump()),
                    clauses: vec![clause(ident("v"), None)],
                }),
            ),
            (
                "DictComprehension key",
                e(ExprKind::DictComprehension {
                    key: bx(bump()),
                    value: bx(ident("i")),
                    clauses: vec![clause(ident("v"), None)],
                }),
            ),
            (
                "DictComprehension value",
                e(ExprKind::DictComprehension {
                    key: bx(ident("i")),
                    value: bx(bump()),
                    clauses: vec![clause(ident("v"), None)],
                }),
            ),
            (
                "DictComprehension clause",
                e(ExprKind::DictComprehension {
                    key: bx(ident("i")),
                    value: bx(ident("i")),
                    clauses: vec![clause(bump(), None)],
                }),
            ),
            (
                "Ask",
                e(ExprKind::Ask {
                    actor: bx(ident("a")),
                    message: bx(bump()),
                    timeout: None,
                }),
            ),
            (
                "Ask timeout",
                e(ExprKind::Ask {
                    actor: bx(ident("a")),
                    message: bx(ident("m")),
                    timeout: Some(bx(bump())),
                }),
            ),
            (
                "ActorSend",
                e(ExprKind::ActorSend {
                    actor: bx(ident("a")),
                    message: bx(bump()),
                }),
            ),
            (
                "ActorQuery",
                e(ExprKind::ActorQuery {
                    actor: bx(ident("a")),
                    message: bx(bump()),
                }),
            ),
            (
                "Handle expr",
                e(ExprKind::Handle {
                    expr: bx(bump()),
                    handlers: vec![],
                }),
            ),
            (
                "Handle handler body",
                e(ExprKind::Handle {
                    expr: bx(ident("x")),
                    handlers: vec![EffectHandler {
                        operation: "op".into(),
                        params: vec![],
                        body: bx(bump()),
                    }],
                }),
            ),
            (
                "Yield",
                e(ExprKind::Yield {
                    value: Some(bx(bump())),
                }),
            ),
            (
                "Signal",
                e(ExprKind::Signal {
                    initial_value: bx(bump()),
                }),
            ),
            (
                "DataFrameOperation source",
                e(ExprKind::DataFrameOperation {
                    source: bx(bump()),
                    operation: DataFrameOp::Head(1),
                }),
            ),
            (
                "DataFrameOperation filter",
                e(ExprKind::DataFrameOperation {
                    source: bx(ident("df")),
                    operation: DataFrameOp::Filter(bx(bump())),
                }),
            ),
            (
                "DataFrameOperation join",
                e(ExprKind::DataFrameOperation {
                    source: bx(ident("df")),
                    operation: DataFrameOp::Join {
                        other: bx(bump()),
                        on: vec![],
                        how: JoinType::Inner,
                    },
                }),
            ),
            (
                "DataFrame column",
                e(ExprKind::DataFrame {
                    columns: vec![DataFrameColumn {
                        name: "c".into(),
                        values: vec![bump()],
                    }],
                }),
            ),
            ("InfraBlock", e(ExprKind::InfraBlock { body: vec![bump()] })),
            (
                "PreIncrement target",
                pre_increment(e(ExprKind::IndexAccess {
                    object: bx(ident("v")),
                    index: bx(bump()),
                })),
            ),
            (
                "PostIncrement target",
                post_increment(e(ExprKind::IndexAccess {
                    object: bx(ident("v")),
                    index: bx(bump()),
                })),
            ),
            (
                "PreDecrement target",
                pre_decrement(e(ExprKind::IndexAccess {
                    object: bx(ident("v")),
                    index: bx(bump()),
                })),
            ),
            (
                "PostDecrement target",
                post_decrement(e(ExprKind::IndexAccess {
                    object: bx(ident("v")),
                    index: bx(bump()),
                })),
            ),
        ]
    }

    #[test]
    fn test_g2bfa2_self_mutation_found_in_every_walked_kind() {
        let missed: Vec<&str> = walk_cases()
            .iter()
            .filter(|(_, expr)| !is_variable_mutated("self", expr))
            .map(|(kind, _)| *kind)
            .collect();
        assert!(missed.is_empty(), "self mutation missed inside: {missed:?}");
    }

    #[test]
    fn test_g2bfa2_self_mutation_found_by_is_self_mutated_in_every_walked_kind() {
        let none = HashSet::new();
        for (kind, expr) in walk_cases() {
            assert!(
                is_self_mutated(&expr, &none),
                "is_self_mutated missed {kind}"
            );
        }
    }

    #[test]
    fn test_g2bfa2_no_false_positive_without_a_mutation() {
        let quiet = make_expr(ExprKind::MacroInvocation {
            name: "println".into(),
            args: vec![make_expr(ExprKind::FieldAccess {
                object: Box::new(ident("self")),
                field: "n".to_string(),
            })],
        });
        assert!(!is_variable_mutated("self", &quiet));
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
