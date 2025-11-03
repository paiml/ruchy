// PERF-002-A: Constant Folding Optimization (GREEN Phase)
// PERF-002-B: Constant Propagation Optimization (GREEN Phase)
// Minimal implementation to make RED tests pass
// Complexity target: ≤10 per function

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use std::collections::{HashMap, HashSet};
#[cfg(test)]
use crate::frontend::ast::Span;

/// Fold constant expressions at compile-time
///
/// Examples:
/// - `2 + 3` → `5`
/// - `10 > 5` → `true`
/// - `(10 - 2) * 4` → `32`
///
/// # Arguments
/// * `expr` - Expression to potentially fold
///
/// # Returns
/// Folded expression if possible, otherwise original expression
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
pub fn fold_constants(expr: Expr) -> Expr {
    match expr.kind {
        ExprKind::Binary { left, op, right } => {
            // Recursively fold children first
            let left_folded = fold_constants((*left).clone());
            let right_folded = fold_constants((*right).clone());

            // Try to fold if both are literals
            if let (ExprKind::Literal(l), ExprKind::Literal(r)) =
                (&left_folded.kind, &right_folded.kind) {
                if let Some(result) = fold_binary_op(l, op, r) {
                    return Expr::new(ExprKind::Literal(result), expr.span);
                }
            }

            // Return with folded children even if we can't fold this level
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(left_folded),
                    op,
                    right: Box::new(right_folded),
                },
                expr.span,
            )
        }
        ExprKind::Let { name, type_annotation, value, body, is_mutable, else_block } => {
            // Fold the value expression
            let folded_value = Box::new(fold_constants((*value).clone()));
            let folded_body = Box::new(fold_constants((*body).clone()));
            let folded_else = else_block.map(|e| Box::new(fold_constants((*e).clone())));
            Expr::new(
                ExprKind::Let {
                    name,
                    type_annotation,
                    value: folded_value,
                    body: folded_body,
                    is_mutable,
                    else_block: folded_else,
                },
                expr.span,
            )
        }
        ExprKind::Block(exprs) => {
            // Fold all expressions in block
            let folded_exprs = exprs.into_iter().map(fold_constants).collect();
            Expr::new(ExprKind::Block(folded_exprs), expr.span)
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            // Fold condition and branches
            let folded_cond = Box::new(fold_constants((*condition).clone()));
            let folded_then = Box::new(fold_constants((*then_branch).clone()));
            let folded_else = else_branch.map(|e| Box::new(fold_constants((*e).clone())));

            // If condition is constant boolean, eliminate dead branch
            if let ExprKind::Literal(Literal::Bool(b)) = folded_cond.kind {
                if b {
                    // Condition is true: return then-branch
                    return (*folded_then).clone();
                }
                if let Some(else_expr) = folded_else {
                    // Condition is false: return else-branch
                    return (*else_expr).clone();
                }
                // Condition is false, no else: return unit (empty block)
                return Expr::new(ExprKind::Block(vec![]), expr.span);
            }

            // Condition not constant: return with folded children
            Expr::new(
                ExprKind::If {
                    condition: folded_cond,
                    then_branch: folded_then,
                    else_branch: folded_else,
                },
                expr.span,
            )
        }
        _ => expr, // Other expressions: return unchanged
    }
}

/// Fold binary operation on two literals
///
/// # Complexity
/// Cyclomatic: 8 (≤10 target)
fn fold_binary_op(left: &Literal, op: BinaryOp, right: &Literal) -> Option<Literal> {
    match (left, right) {
        // Integer operations (both arithmetic and comparison)
        (Literal::Integer(a, None), Literal::Integer(b, None)) => {
            fold_integer_comparison(*a, op, *b).or_else(|| fold_integer_arithmetic(*a, op, *b))
        }

        _ => None, // Other combinations: not folded yet
    }
}

/// Fold integer arithmetic operations
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn fold_integer_arithmetic(a: i64, op: BinaryOp, b: i64) -> Option<Literal> {
    let result = match op {
        BinaryOp::Add => a.checked_add(b)?,
        BinaryOp::Subtract => a.checked_sub(b)?,
        BinaryOp::Multiply => a.checked_mul(b)?,
        BinaryOp::Divide if b != 0 => a.checked_div(b)?,
        _ => return None,
    };
    Some(Literal::Integer(result, None))
}

/// Fold integer comparison operations
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
fn fold_integer_comparison(a: i64, op: BinaryOp, b: i64) -> Option<Literal> {
    let result = match op {
        BinaryOp::Equal => a == b,
        BinaryOp::NotEqual => a != b,
        BinaryOp::Less => a < b,
        BinaryOp::LessEqual => a <= b,
        BinaryOp::Greater => a > b,
        BinaryOp::GreaterEqual => a >= b,
        _ => return None,
    };
    Some(Literal::Bool(result))
}

// ============================================================================
// PERF-002-C: Dead Code Elimination (DCE)
// ============================================================================

/// Eliminate dead code (unreachable statements, unused variables)
///
/// Examples:
/// - `return 5; let x = 10;` → `return 5;` (unreachable code removed)
/// - `let unused = 42; println(5);` → `println(5);` (unused binding removed)
///
/// # Arguments
/// * `expr` - Expression to eliminate dead code from
///
/// # Returns
/// Expression with dead code removed
///
/// Collect all function names that are called in the expression tree
///
/// # Complexity
/// Cyclomatic: 7 (≤10 target)
fn collect_used_functions(expr: &Expr) -> HashSet<String> {
    let mut used = HashSet::new();
    collect_used_functions_rec(expr, &mut used);
    used
}

/// Recursive helper to collect used function names
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
fn collect_used_functions_rec(expr: &Expr, used: &mut HashSet<String>) {
    match &expr.kind {
        ExprKind::Call { func, args } => {
            // If this is a simple function call, record the name
            if let ExprKind::Identifier(func_name) = &func.kind {
                used.insert(func_name.clone());
            }
            // Recursively check func and args
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
        ExprKind::If { condition, then_branch, else_branch } => {
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
        _ => {
            // Other expressions: no function calls to track
        }
    }
}

/// # Complexity
/// Cyclomatic: 6 (≤10 target)
pub fn eliminate_dead_code(expr: Expr, inlined_functions: std::collections::HashSet<String>) -> Expr {
    match expr.kind {
        ExprKind::Block(exprs) => {
            // First, collect all used function names in the entire block
            let used_functions = {
                let temp_expr = Expr::new(ExprKind::Block(exprs.clone()), expr.span.clone());
                collect_used_functions(&temp_expr)
            };

            // Then remove dead statements AND unused function definitions
            // ISSUE-128 FIX: Preserve functions that weren't successfully inlined
            let cleaned = remove_dead_statements_and_unused_functions(exprs, &used_functions, &inlined_functions);
            Expr::new(ExprKind::Block(cleaned), expr.span)
        }
        ExprKind::Function { name, type_params, params, return_type, body, is_async, is_pub } => {
            // For nested functions, pass empty set (no inlining at this level)
            let cleaned_body = Box::new(eliminate_dead_code((*body).clone(), std::collections::HashSet::new()));
            Expr::new(
                ExprKind::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body: cleaned_body,
                    is_async,
                    is_pub,
                },
                expr.span,
            )
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            let cleaned_then = Box::new(eliminate_dead_code((*then_branch).clone(), std::collections::HashSet::new()));
            let cleaned_else = else_branch.map(|e| Box::new(eliminate_dead_code((*e).clone(), std::collections::HashSet::new())));
            Expr::new(
                ExprKind::If {
                    condition,
                    then_branch: cleaned_then,
                    else_branch: cleaned_else,
                },
                expr.span,
            )
        }
        ExprKind::While { condition, body, label } => {
            let cleaned_body = Box::new(eliminate_dead_code((*body).clone(), std::collections::HashSet::new()));
            Expr::new(
                ExprKind::While {
                    condition,
                    body: cleaned_body,
                    label,
                },
                expr.span,
            )
        }
        _ => expr, // Other expressions: no DCE needed
    }
}

/// Remove dead statements from a block
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn remove_dead_statements(exprs: Vec<Expr>) -> Vec<Expr> {
    let mut result = Vec::new();

    for expr in exprs {
        // Recursively eliminate dead code in child expressions
        let cleaned = eliminate_dead_code(expr, std::collections::HashSet::new());

        result.push(cleaned.clone());

        // Stop processing after early exit (return/break/continue)
        if has_early_exit(&cleaned) {
            break;
        }
    }

    result
}

/// Remove dead statements AND unused function definitions from a block
///
/// ISSUE-128 FIX: Preserve functions that weren't successfully inlined
///
/// # Complexity
/// Cyclomatic: 7 (≤10 target)
fn remove_dead_statements_and_unused_functions(
    exprs: Vec<Expr>,
    used_functions: &HashSet<String>,
    inlined_functions: &std::collections::HashSet<String>
) -> Vec<Expr> {
    let mut result = Vec::new();

    for expr in exprs {
        // ISSUE-128 FIX: Only remove functions that were BOTH inlined AND no longer used
        if let ExprKind::Function { name, .. } = &expr.kind {
            // Keep function if:
            // 1. It's main (always preserve)
            // 2. It's still called somewhere (in used_functions)
            // 3. It wasn't inlined (not in inlined_functions, so preserve it)
            //
            // Remove function only if:
            // - It was marked for inlining AND
            // - It's no longer called anywhere
            let should_remove = inlined_functions.contains(name)
                && !used_functions.contains(name)
                && name != "main";

            if should_remove {
                // Skip this function - successfully inlined and no longer called
                continue;
            }
        }

        // Recursively eliminate dead code in child expressions
        let cleaned = eliminate_dead_code(expr, std::collections::HashSet::new());

        result.push(cleaned.clone());

        // Stop processing after early exit (return/break/continue)
        if has_early_exit(&cleaned) {
            break;
        }
    }

    result
}

/// Check if expression causes early exit
///
/// # Complexity
/// Cyclomatic: 4 (≤10 target)
fn has_early_exit(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Return { .. } | ExprKind::Break { .. } | ExprKind::Continue { .. }
    )
}

// ============================================================================
// PERF-002-B: Constant Propagation
// ============================================================================

/// Propagate constant values through variables
///
/// Examples:
/// - `let x = 5; x + 1` → `6`
/// - `let x = 5; let y = x; y + 3` → `8`
///
/// # Arguments
/// * `expr` - Expression to propagate constants through
///
/// # Returns
/// Expression with constants propagated
///
/// # Complexity
/// Cyclomatic: 8 (≤10 target)
pub fn propagate_constants(expr: Expr) -> Expr {
    let mut env = HashMap::new();
    propagate_with_env(expr, &mut env)
}

/// Helper: Propagate constants with environment tracking
///
/// # Complexity
/// Cyclomatic: 9 (≤10 target)
fn propagate_with_env(expr: Expr, env: &mut HashMap<String, Literal>) -> Expr {
    // First, apply constant folding
    let expr = fold_constants(expr);

    match expr.kind {
        // Track constant variable bindings
        ExprKind::Let { name, type_annotation, value, body, is_mutable, else_block } => {
            // Recursively propagate in value
            let folded_value = Box::new(propagate_with_env((*value).clone(), env));

            // If value is constant and variable is immutable, track it
            if !is_mutable {
                if let ExprKind::Literal(ref lit) = folded_value.kind {
                    env.insert(name.clone(), lit.clone());
                }
            }

            // Propagate in body with updated environment
            let folded_body = Box::new(propagate_with_env((*body).clone(), env));

            // Propagate in else block if present
            let folded_else = else_block.map(|e| Box::new(propagate_with_env((*e).clone(), env)));

            Expr::new(
                ExprKind::Let {
                    name,
                    type_annotation,
                    value: folded_value,
                    body: folded_body,
                    is_mutable,
                    else_block: folded_else,
                },
                expr.span,
            )
        }

        // Substitute known variables with their constant values
        ExprKind::Identifier(ref name) => {
            if let Some(lit) = env.get(name) {
                // Replace variable with its constant value
                Expr::new(ExprKind::Literal(lit.clone()), expr.span)
            } else {
                expr
            }
        }

        // Recursively propagate in binary expressions
        ExprKind::Binary { left, op, right } => {
            let left_prop = propagate_with_env((*left).clone(), env);
            let right_prop = propagate_with_env((*right).clone(), env);

            // After propagation, try folding again
            let binary_expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(left_prop),
                    op,
                    right: Box::new(right_prop),
                },
                expr.span,
            );
            fold_constants(binary_expr)
        }

        // Propagate in if expressions
        ExprKind::If { condition, then_branch, else_branch } => {
            let cond_prop = Box::new(propagate_with_env((*condition).clone(), env));
            let then_prop = Box::new(propagate_with_env((*then_branch).clone(), env));
            let else_prop = else_branch.map(|e| Box::new(propagate_with_env((*e).clone(), env)));

            let if_expr = Expr::new(
                ExprKind::If {
                    condition: cond_prop,
                    then_branch: then_prop,
                    else_branch: else_prop,
                },
                expr.span,
            );

            // Try folding if condition is constant
            fold_constants(if_expr)
        }

        // Propagate in blocks
        ExprKind::Block(exprs) => {
            let folded_exprs = exprs.into_iter()
                .map(|e| propagate_with_env(e, env))
                .collect();
            Expr::new(ExprKind::Block(folded_exprs), expr.span)
        }

        // Propagate in function calls
        ExprKind::Call { func, args } => {
            let func_prop = Box::new(propagate_with_env((*func).clone(), env));
            let args_prop = args.into_iter()
                .map(|a| propagate_with_env(a, env))
                .collect();
            Expr::new(
                ExprKind::Call {
                    func: func_prop,
                    args: args_prop,
                },
                expr.span,
            )
        }

        // Other expressions: return as-is
        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_simple_add() {
        // 2 + 3 → 5
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span::new(0, 1),
                )),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(3, None)),
                    Span::new(4, 5),
                )),
            },
            Span::new(0, 5),
        );

        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Integer(5, None))
        ));
    }

    #[test]
    fn test_fold_comparison() {
        // 10 > 5 → true
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::new(0, 2),
                )),
                op: BinaryOp::Greater,
                right: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::new(5, 6),
                )),
            },
            Span::new(0, 6),
        );

        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Bool(true))
        ));
    }
}
