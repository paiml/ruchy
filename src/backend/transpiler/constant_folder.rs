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
        // ASYNC-AWAIT: Handle await expressions to prevent DCE from removing async functions
        ExprKind::Await { expr } => {
            collect_used_functions_rec(expr, used);
        }
        ExprKind::AsyncBlock { body } => {
            collect_used_functions_rec(body, used);
        }
        ExprKind::Spawn { actor } => {
            collect_used_functions_rec(actor, used);
        }
        _ => {
            // Other expressions: no function calls to track
        }
    }
}

/// Collect all variable names that are used (read) in the expression tree
///
/// PERF-002-C: Liveness analysis for unused variable elimination
///
/// # Complexity
/// Cyclomatic: 1 (≤10 target)
fn collect_used_variables(expr: &Expr) -> HashSet<String> {
    let mut used = HashSet::new();
    collect_used_variables_rec(expr, &mut used, &HashSet::new());
    used
}

/// Recursive helper to collect used variable names
///
/// # Arguments
/// * `expr` - Expression to scan
/// * `used` - Set to accumulate used variable names
/// * `bound` - Set of variables currently in scope (to distinguish from function calls)
///
/// # Complexity
/// Cyclomatic: 9 (≤10 target)
fn collect_used_variables_rec(expr: &Expr, used: &mut HashSet<String>, bound: &HashSet<String>) {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            // Only count as variable use if it's bound (not a function name)
            if bound.contains(name) {
                used.insert(name.clone());
            }
        }
        ExprKind::Let { name, value, body, else_block, .. } => {
            // First scan the value (before name is bound)
            collect_used_variables_rec(value, used, bound);

            // Then scan body with name added to bound set
            let mut new_bound = bound.clone();
            new_bound.insert(name.clone());
            collect_used_variables_rec(body, used, &new_bound);

            // Scan else block if present
            if let Some(else_expr) = else_block {
                collect_used_variables_rec(else_expr, used, bound);
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                collect_used_variables_rec(e, used, bound);
            }
        }
        ExprKind::Function { body, .. } => {
            // Functions create new scope - don't propagate outer bindings
            collect_used_variables_rec(body, used, &HashSet::new());
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            collect_used_variables_rec(condition, used, bound);
            collect_used_variables_rec(then_branch, used, bound);
            if let Some(else_expr) = else_branch {
                collect_used_variables_rec(else_expr, used, bound);
            }
        }
        ExprKind::While { condition, body, .. } => {
            collect_used_variables_rec(condition, used, bound);
            collect_used_variables_rec(body, used, bound);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_used_variables_rec(left, used, bound);
            collect_used_variables_rec(right, used, bound);
        }
        ExprKind::Call { func, args } => {
            collect_used_variables_rec(func, used, bound);
            for arg in args {
                collect_used_variables_rec(arg, used, bound);
            }
        }
        ExprKind::Return { value } => {
            if let Some(val) = value {
                collect_used_variables_rec(val, used, bound);
            }
        }
        _ => {
            // Other expressions: no variable uses to track
        }
    }
}

/// # Complexity
/// Cyclomatic: 6 (≤10 target)
pub fn eliminate_dead_code(expr: Expr, inlined_functions: std::collections::HashSet<String>) -> Expr {
    match expr.kind {
        ExprKind::Block(exprs) => {
            // PERF-002-C: Collect all used function names in the entire block
            let used_functions = {
                let temp_expr = Expr::new(ExprKind::Block(exprs.clone()), expr.span);
                collect_used_functions(&temp_expr)
            };

            // PERF-002-C: Collect all used variable names for liveness analysis
            let used_variables = {
                let temp_expr = Expr::new(ExprKind::Block(exprs.clone()), expr.span);
                collect_used_variables(&temp_expr)
            };

            // Remove dead statements, unused functions, AND unused variables
            // ISSUE-128 FIX: Preserve functions that weren't successfully inlined
            let cleaned = remove_dead_statements_and_unused_functions_and_variables(
                exprs,
                &used_functions,
                &inlined_functions,
                &used_variables,
            );
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
        ExprKind::Call { func, args } => {
            // PERF-002-C: Recursively eliminate dead code in function arguments
            let cleaned_func = Box::new(eliminate_dead_code((*func).clone(), inlined_functions.clone()));
            let cleaned_args: Vec<Expr> = args
                .into_iter()
                .map(|arg| eliminate_dead_code(arg, inlined_functions.clone()))
                .collect();
            Expr::new(
                ExprKind::Call {
                    func: cleaned_func,
                    args: cleaned_args,
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

/// Remove dead statements, unused function definitions, AND unused variables from a block
///
/// ISSUE-128 FIX: Preserve functions that weren't successfully inlined
/// PERF-002-C: Remove unused variable bindings via liveness analysis
///
/// # Complexity
/// Cyclomatic: 9 (≤10 target)
fn remove_dead_statements_and_unused_functions_and_variables(
    exprs: Vec<Expr>,
    used_functions: &HashSet<String>,
    inlined_functions: &std::collections::HashSet<String>,
    used_variables: &HashSet<String>,
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

        // PERF-002-C: Remove unused variable bindings
        if let ExprKind::Let { name, value, body, .. } = &expr.kind {
            // TRANSPILER-BUG FIX: Don't eliminate Let bindings with Unit body
            // These are top-level statements (let x = 0;) not scoped bindings (let x = 0 in expr)
            // The variable may be used in subsequent statements outside this Let expression
            let body_is_unit = matches!(body.kind, ExprKind::Literal(Literal::Unit));

            // Skip this let binding if the variable is never used
            // BUT keep it if:
            // 1. The value has side effects (like function calls)
            // 2. The body is Unit (top-level statement, may be used in subsequent statements)
            if !used_variables.contains(name) && !has_side_effects(value) && !body_is_unit {
                // Variable is unused and value has no side effects - eliminate it
                // Continue with the body instead
                let cleaned_body = eliminate_dead_code((**body).clone(), std::collections::HashSet::new());

                // If body is a block, unwrap it
                if let ExprKind::Block(inner_exprs) = cleaned_body.kind {
                    result.extend(inner_exprs);
                } else {
                    result.push(cleaned_body);
                }
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

/// Check if expression has side effects (prevents DCE)
///
/// Side effects include:
/// - Function calls (may have I/O, mutations, etc.)
/// - Assignments
///
/// # Complexity
/// Cyclomatic: 3 (≤10 target)
fn has_side_effects(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Call { .. } | ExprKind::Assign { .. }
    )
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

    // Helper: Create integer literal
    fn int_lit(n: i64) -> Expr {
        Expr::new(ExprKind::Literal(Literal::Integer(n, None)), Span::new(0, 0))
    }

    // Helper: Create binary expression
    fn binary(left: i64, op: BinaryOp, right: i64) -> Expr {
        Expr::new(
            ExprKind::Binary {
                left: Box::new(int_lit(left)),
                op,
                right: Box::new(int_lit(right)),
            },
            Span::new(0, 0),
        )
    }

    // Test 1: fold_constants - simple addition
    #[test]
    fn test_fold_simple_add() {
        // 2 + 3 → 5
        let expr = binary(2, BinaryOp::Add, 3);
        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Integer(5, None))
        ));
    }

    // Test 2: fold_constants - comparison (10 > 5)
    #[test]
    fn test_fold_comparison() {
        // 10 > 5 → true
        let expr = binary(10, BinaryOp::Greater, 5);
        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Bool(true))
        ));
    }

    // Test 3: fold_integer_arithmetic - subtract
    #[test]
    fn test_fold_integer_arithmetic_subtract() {
        let result = fold_integer_arithmetic(10, BinaryOp::Subtract, 3);
        assert!(matches!(result, Some(Literal::Integer(7, None))));
    }

    // Test 4: fold_integer_arithmetic - multiply
    #[test]
    fn test_fold_integer_arithmetic_multiply() {
        let result = fold_integer_arithmetic(4, BinaryOp::Multiply, 5);
        assert!(matches!(result, Some(Literal::Integer(20, None))));
    }

    // Test 5: fold_integer_arithmetic - divide
    #[test]
    fn test_fold_integer_arithmetic_divide() {
        let result = fold_integer_arithmetic(20, BinaryOp::Divide, 4);
        assert!(matches!(result, Some(Literal::Integer(5, None))));
    }

    // Test 6: fold_integer_arithmetic - divide by zero
    #[test]
    fn test_fold_integer_arithmetic_divide_by_zero() {
        let result = fold_integer_arithmetic(20, BinaryOp::Divide, 0);
        assert!(result.is_none());
    }

    // Test 7: fold_integer_arithmetic - unsupported operation
    #[test]
    fn test_fold_integer_arithmetic_unsupported() {
        let result = fold_integer_arithmetic(10, BinaryOp::Equal, 5);
        assert!(result.is_none());
    }

    // Test 8: fold_integer_comparison - equal
    #[test]
    fn test_fold_integer_comparison_equal() {
        let result = fold_integer_comparison(5, BinaryOp::Equal, 5);
        assert!(matches!(result, Some(Literal::Bool(true))));
    }

    // Test 9: fold_integer_comparison - not equal
    #[test]
    fn test_fold_integer_comparison_not_equal() {
        let result = fold_integer_comparison(5, BinaryOp::NotEqual, 3);
        assert!(matches!(result, Some(Literal::Bool(true))));
    }

    // Test 10: fold_integer_comparison - less
    #[test]
    fn test_fold_integer_comparison_less() {
        let result = fold_integer_comparison(3, BinaryOp::Less, 5);
        assert!(matches!(result, Some(Literal::Bool(true))));
    }

    // Test 11: fold_integer_comparison - less equal
    #[test]
    fn test_fold_integer_comparison_less_equal() {
        let result = fold_integer_comparison(5, BinaryOp::LessEqual, 5);
        assert!(matches!(result, Some(Literal::Bool(true))));
    }

    // Test 12: fold_integer_comparison - greater equal
    #[test]
    fn test_fold_integer_comparison_greater_equal() {
        let result = fold_integer_comparison(5, BinaryOp::GreaterEqual, 5);
        assert!(matches!(result, Some(Literal::Bool(true))));
    }

    // Test 13: fold_integer_comparison - unsupported operation
    #[test]
    fn test_fold_integer_comparison_unsupported() {
        let result = fold_integer_comparison(5, BinaryOp::Add, 3);
        assert!(result.is_none());
    }

    // Test 14: has_side_effects - Call expression
    #[test]
    fn test_has_side_effects_call() {
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("foo".to_string()),
                    Span::new(0, 0),
                )),
                args: vec![],
            },
            Span::new(0, 0),
        );
        assert!(has_side_effects(&expr));
    }

    // Test 15: has_side_effects - Assign expression
    #[test]
    fn test_has_side_effects_assign() {
        let expr = Expr::new(
            ExprKind::Assign {
                target: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                value: Box::new(int_lit(5)),
            },
            Span::new(0, 0),
        );
        assert!(has_side_effects(&expr));
    }

    // Test 16: has_side_effects - Literal (no side effects)
    #[test]
    fn test_has_side_effects_literal() {
        let expr = int_lit(42);
        assert!(!has_side_effects(&expr));
    }

    // Test 17: has_early_exit - Return
    #[test]
    fn test_has_early_exit_return() {
        let expr = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(int_lit(5))),
            },
            Span::new(0, 0),
        );
        assert!(has_early_exit(&expr));
    }

    // Test 18: has_early_exit - Break
    #[test]
    fn test_has_early_exit_break() {
        let expr = Expr::new(
            ExprKind::Break {
                label: None,
                value: None,
            },
            Span::new(0, 0),
        );
        assert!(has_early_exit(&expr));
    }

    // Test 19: has_early_exit - Continue
    #[test]
    fn test_has_early_exit_continue() {
        let expr = Expr::new(
            ExprKind::Continue { label: None },
            Span::new(0, 0),
        );
        assert!(has_early_exit(&expr));
    }

    // Test 20: has_early_exit - Literal (no early exit)
    #[test]
    fn test_has_early_exit_literal() {
        let expr = int_lit(42);
        assert!(!has_early_exit(&expr));
    }
}
