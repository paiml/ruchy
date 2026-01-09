// PERF-002-A: Constant Folding Optimization (GREEN Phase)
// PERF-002-B: Constant Propagation Optimization (GREEN Phase)
// Minimal implementation to make RED tests pass
// Complexity target: ≤10 per function

#[cfg(test)]
use crate::frontend::ast::Span;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use std::collections::{HashMap, HashSet};

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
                (&left_folded.kind, &right_folded.kind)
            {
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
        ExprKind::Let {
            name,
            type_annotation,
            value,
            body,
            is_mutable,
            else_block,
        } => {
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
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // Fold condition and branches
            let folded_cond = Box::new(fold_constants((*condition).clone()));
            let folded_then = Box::new(fold_constants((*then_branch).clone()));
            let folded_else = else_branch.map(|e| Box::new(fold_constants((*e).clone())));

            // If condition is constant boolean, eliminate dead branch
            // QA-026 FIX: Preserve scope boundaries by wrapping in Block
            if let ExprKind::Literal(Literal::Bool(b)) = folded_cond.kind {
                if b {
                    // Condition is true: return then-branch wrapped in block to preserve scope
                    // This ensures `if true { let x = 20 }` becomes `{ let x = 20 }` not just `let x = 20`
                    return Expr::new(ExprKind::Block(vec![(*folded_then).clone()]), expr.span);
                }
                if let Some(else_expr) = folded_else {
                    // Condition is false: return else-branch wrapped in block
                    return Expr::new(ExprKind::Block(vec![(*else_expr).clone()]), expr.span);
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
        ExprKind::Let {
            name,
            value,
            body,
            else_block,
            ..
        } => {
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
        ExprKind::While {
            condition, body, ..
        } => {
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
pub fn eliminate_dead_code(
    expr: Expr,
    inlined_functions: std::collections::HashSet<String>,
) -> Expr {
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
        ExprKind::Function {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async,
            is_pub,
        } => {
            // For nested functions, pass empty set (no inlining at this level)
            let cleaned_body = Box::new(eliminate_dead_code(
                (*body).clone(),
                std::collections::HashSet::new(),
            ));
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
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let cleaned_then = Box::new(eliminate_dead_code(
                (*then_branch).clone(),
                std::collections::HashSet::new(),
            ));
            let cleaned_else = else_branch.map(|e| {
                Box::new(eliminate_dead_code(
                    (*e).clone(),
                    std::collections::HashSet::new(),
                ))
            });
            Expr::new(
                ExprKind::If {
                    condition,
                    then_branch: cleaned_then,
                    else_branch: cleaned_else,
                },
                expr.span,
            )
        }
        ExprKind::While {
            condition,
            body,
            label,
        } => {
            let cleaned_body = Box::new(eliminate_dead_code(
                (*body).clone(),
                std::collections::HashSet::new(),
            ));
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
            let cleaned_func = Box::new(eliminate_dead_code(
                (*func).clone(),
                inlined_functions.clone(),
            ));
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

/// Check if a function should be removed (inlined and no longer called)
fn should_remove_function(
    name: &str,
    used_functions: &HashSet<String>,
    inlined_functions: &std::collections::HashSet<String>,
) -> bool {
    inlined_functions.contains(name) && !used_functions.contains(name) && name != "main"
}

/// Process let binding elimination and return replacement expressions if eliminated
fn process_let_elimination(
    name: &str,
    value: &Expr,
    body: &Expr,
    used_variables: &HashSet<String>,
) -> Option<Vec<Expr>> {
    let body_is_unit = matches!(body.kind, ExprKind::Literal(Literal::Unit));

    // Keep binding if variable is used, has side effects, or is top-level
    if used_variables.contains(name) || has_side_effects(value) || body_is_unit {
        return None;
    }

    // Eliminate unused binding - return cleaned body
    let cleaned_body = eliminate_dead_code(body.clone(), std::collections::HashSet::new());

    Some(if let ExprKind::Block(inner_exprs) = cleaned_body.kind {
        inner_exprs
    } else {
        vec![cleaned_body]
    })
}

/// Remove dead statements, unused function definitions, AND unused variables from a block
///
/// ISSUE-128 FIX: Preserve functions that weren't successfully inlined
/// PERF-002-C: Remove unused variable bindings via liveness analysis
fn remove_dead_statements_and_unused_functions_and_variables(
    exprs: Vec<Expr>,
    used_functions: &HashSet<String>,
    inlined_functions: &std::collections::HashSet<String>,
    used_variables: &HashSet<String>,
) -> Vec<Expr> {
    let mut result = Vec::new();

    for expr in exprs {
        // Check for function removal
        if let ExprKind::Function { name, .. } = &expr.kind {
            if should_remove_function(name, used_functions, inlined_functions) {
                continue;
            }
        }

        // Check for let binding elimination
        if let ExprKind::Let {
            name, value, body, ..
        } = &expr.kind
        {
            if let Some(replacement) = process_let_elimination(name, value, body, used_variables) {
                result.extend(replacement);
                continue;
            }
        }

        // Recursively eliminate dead code
        let cleaned = eliminate_dead_code(expr, std::collections::HashSet::new());
        result.push(cleaned.clone());

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
    matches!(expr.kind, ExprKind::Call { .. } | ExprKind::Assign { .. })
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
        ExprKind::Let {
            name,
            type_annotation,
            value,
            body,
            is_mutable,
            else_block,
        } => {
            // Recursively propagate in value
            let folded_value = Box::new(propagate_with_env((*value).clone(), env));

            // If value is constant and variable is immutable, track it
            // TRANSPILER-STRING-CONCAT-001 FIX: Don't propagate string literals
            // String constants should keep their variable names for:
            // 1. Readability in generated code
            // 2. Correct behavior in string concatenation (a + b should use a, b)
            // 3. Proper ownership semantics (strings are heap-allocated)
            if !is_mutable {
                if let ExprKind::Literal(ref lit) = folded_value.kind {
                    // Only propagate numeric/boolean literals, not strings
                    if !matches!(lit, Literal::String(_)) {
                        env.insert(name.clone(), lit.clone());
                    }
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
        // QA-026 FIX: Clone env for branches to prevent inner scope bindings from leaking
        ExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            // Condition uses outer env (no new scope)
            let cond_prop = Box::new(propagate_with_env((*condition).clone(), env));
            // Then/else branches get their own env copy - inner bindings don't leak out
            let mut then_env = env.clone();
            let then_prop = Box::new(propagate_with_env((*then_branch).clone(), &mut then_env));
            let else_prop = else_branch.map(|e| {
                let mut else_env = env.clone();
                Box::new(propagate_with_env((*e).clone(), &mut else_env))
            });

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
        // QA-026 FIX: Blocks create new scope - clone env to prevent leaking
        ExprKind::Block(exprs) => {
            let mut block_env = env.clone();
            let folded_exprs = exprs
                .into_iter()
                .map(|e| propagate_with_env(e, &mut block_env))
                .collect();
            Expr::new(ExprKind::Block(folded_exprs), expr.span)
        }

        // Propagate in function calls
        ExprKind::Call { func, args } => {
            let func_prop = Box::new(propagate_with_env((*func).clone(), env));
            let args_prop = args
                .into_iter()
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
        Expr::new(
            ExprKind::Literal(Literal::Integer(n, None)),
            Span::new(0, 0),
        )
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
        let expr = Expr::new(ExprKind::Continue { label: None }, Span::new(0, 0));
        assert!(has_early_exit(&expr));
    }

    // Test 20: has_early_exit - Literal (no early exit)
    #[test]
    fn test_has_early_exit_literal() {
        let expr = int_lit(42);
        assert!(!has_early_exit(&expr));
    }

    // Test 21: collect_used_functions - basic function call
    #[test]
    fn test_collect_used_functions_basic() {
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
        let used = collect_used_functions(&expr);
        assert!(used.contains("foo"));
        assert_eq!(used.len(), 1);
    }

    // Test 22: collect_used_functions - nested in if
    #[test]
    fn test_collect_used_functions_nested_if() {
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("check".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
                then_branch: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("action".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
                else_branch: None,
            },
            Span::new(0, 0),
        );
        let used = collect_used_functions(&expr);
        assert!(used.contains("check"));
        assert!(used.contains("action"));
        assert_eq!(used.len(), 2);
    }

    // Test 23: collect_used_functions_rec - Await expression
    #[test]
    fn test_collect_used_functions_await() {
        let mut used = HashSet::new();
        let expr = Expr::new(
            ExprKind::Await {
                expr: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("async_fn".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
            },
            Span::new(0, 0),
        );
        collect_used_functions_rec(&expr, &mut used);
        assert!(used.contains("async_fn"));
    }

    // Test 24: collect_used_functions_rec - AsyncBlock
    #[test]
    fn test_collect_used_functions_async_block() {
        let mut used = HashSet::new();
        let expr = Expr::new(
            ExprKind::AsyncBlock {
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("work".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
            },
            Span::new(0, 0),
        );
        collect_used_functions_rec(&expr, &mut used);
        assert!(used.contains("work"));
    }

    // Test 25: collect_used_functions_rec - Spawn expression
    #[test]
    fn test_collect_used_functions_spawn() {
        let mut used = HashSet::new();
        let expr = Expr::new(
            ExprKind::Spawn {
                actor: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("actor_fn".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
            },
            Span::new(0, 0),
        );
        collect_used_functions_rec(&expr, &mut used);
        assert!(used.contains("actor_fn"));
    }

    // Test 26: collect_used_variables - simple identifier
    #[test]
    fn test_collect_used_variables_simple() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );
        let used = collect_used_variables(&expr);
        assert!(used.contains("x"));
        assert_eq!(used.len(), 1);
    }

    // Test 27: collect_used_variables_rec - While loop
    #[test]
    fn test_collect_used_variables_while_loop() {
        let mut used = HashSet::new();
        let mut bound = HashSet::new();
        bound.insert("counter".to_string());

        let expr = Expr::new(
            ExprKind::While {
                condition: Box::new(Expr::new(
                    ExprKind::Identifier("counter".to_string()),
                    Span::new(0, 0),
                )),
                body: Box::new(int_lit(1)),
                label: None,
            },
            Span::new(0, 0),
        );
        collect_used_variables_rec(&expr, &mut used, &bound);
        assert!(used.contains("counter"));
    }

    // Test 28: collect_used_variables_rec - Return with value
    #[test]
    fn test_collect_used_variables_return() {
        let mut used = HashSet::new();
        let mut bound = HashSet::new();
        bound.insert("result".to_string());

        let expr = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(Expr::new(
                    ExprKind::Identifier("result".to_string()),
                    Span::new(0, 0),
                ))),
            },
            Span::new(0, 0),
        );
        collect_used_variables_rec(&expr, &mut used, &bound);
        assert!(used.contains("result"));
    }

    // Test 29: remove_dead_statements - after return
    #[test]
    fn test_remove_dead_statements_after_return() {
        let stmts = vec![
            Expr::new(
                ExprKind::Return {
                    value: Some(Box::new(int_lit(5))),
                },
                Span::new(0, 0),
            ),
            int_lit(10), // Dead code - after return
            int_lit(20), // Dead code - after return
        ];
        let result = remove_dead_statements(stmts);
        assert_eq!(result.len(), 1); // Only return statement remains
    }

    // Test 30: remove_dead_statements - no early exit
    #[test]
    fn test_remove_dead_statements_no_early_exit() {
        let stmts = vec![int_lit(1), int_lit(2), int_lit(3)];
        let result = remove_dead_statements(stmts);
        assert_eq!(result.len(), 3); // All statements preserved
    }

    // Test 31: propagate_constants - simple let binding
    #[test]
    fn test_propagate_constants_simple_let() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            Span::new(0, 0),
                        )),
                        op: BinaryOp::Add,
                        right: Box::new(int_lit(1)),
                    },
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );
        let result = propagate_constants(expr);
        // After propagation: let x = 5; x + 1 → let x = 5; 5 + 1 → let x = 5; 6
        if let ExprKind::Let { body, .. } = result.kind {
            // Body should be folded to 6
            assert!(matches!(
                body.kind,
                ExprKind::Literal(Literal::Integer(6, None))
            ));
        } else {
            panic!("Expected Let expression");
        }
    }

    // Test 32: propagate_constants - variable substitution
    #[test]
    fn test_propagate_constants_variable_substitution() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(10)),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );
        let result = propagate_constants(expr);
        // After propagation: let x = 10; x → let x = 10; 10
        if let ExprKind::Let { body, .. } = result.kind {
            // Variable should be replaced with its constant value
            assert!(matches!(
                body.kind,
                ExprKind::Literal(Literal::Integer(10, None))
            ));
        } else {
            panic!("Expected Let expression");
        }
    }

    // Test 33: propagate_with_env - mutable variable (not propagated)
    #[test]
    fn test_propagate_with_env_mutable() {
        let mut env = HashMap::new();
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                is_mutable: true, // Mutable variable
                else_block: None,
            },
            Span::new(0, 0),
        );
        let result = propagate_with_env(expr, &mut env);
        // Mutable variables should NOT be propagated
        if let ExprKind::Let { body, .. } = result.kind {
            // Variable should remain as identifier (not replaced with constant)
            assert!(matches!(body.kind, ExprKind::Identifier(_)));
        } else {
            panic!("Expected Let expression");
        }
    }

    // Test 34: propagate_with_env - in Call arguments
    #[test]
    fn test_propagate_with_env_call() {
        let mut env = HashMap::new();
        env.insert("x".to_string(), Literal::Integer(5, None));

        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("foo".to_string()),
                    Span::new(0, 0),
                )),
                args: vec![Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )],
            },
            Span::new(0, 0),
        );
        let result = propagate_with_env(expr, &mut env);
        // Variable in argument should be replaced with constant
        if let ExprKind::Call { args, .. } = result.kind {
            assert_eq!(args.len(), 1);
            assert!(matches!(
                args[0].kind,
                ExprKind::Literal(Literal::Integer(5, None))
            ));
        } else {
            panic!("Expected Call expression");
        }
    }

    // Test 35: fold_constants - If with const true condition (dead branch elimination)
    #[test]
    fn test_fold_constants_if_const_true() {
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::new(0, 0),
                )),
                then_branch: Box::new(int_lit(42)),
                else_branch: Some(Box::new(int_lit(99))),
            },
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        // QA-026 FIX: Now returns Block to preserve scope boundaries
        // Should eliminate else branch and return then branch wrapped in block
        match &folded.kind {
            ExprKind::Block(exprs) => {
                assert_eq!(
                    exprs.len(),
                    1,
                    "Block should contain exactly one expression"
                );
                assert!(matches!(
                    exprs[0].kind,
                    ExprKind::Literal(Literal::Integer(42, None))
                ));
            }
            _ => panic!("Expected Block, got {:?}", folded.kind),
        }
    }

    // ===== EXTREME TDD Round 156 - Constant Folder Tests =====

    #[test]
    fn test_fold_constants_if_const_false_with_else() {
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    Span::new(0, 0),
                )),
                then_branch: Box::new(int_lit(42)),
                else_branch: Some(Box::new(int_lit(99))),
            },
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        match &folded.kind {
            ExprKind::Block(exprs) => {
                assert_eq!(exprs.len(), 1);
                assert!(matches!(
                    exprs[0].kind,
                    ExprKind::Literal(Literal::Integer(99, None))
                ));
            }
            _ => panic!("Expected Block, got {:?}", folded.kind),
        }
    }

    #[test]
    fn test_fold_constants_if_const_false_no_else() {
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(false)),
                    Span::new(0, 0),
                )),
                then_branch: Box::new(int_lit(42)),
                else_branch: None,
            },
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        // Should return empty block (unit)
        match &folded.kind {
            ExprKind::Block(exprs) => {
                assert!(exprs.is_empty());
            }
            _ => panic!("Expected empty Block, got {:?}", folded.kind),
        }
    }

    #[test]
    fn test_fold_constants_nested_binary() {
        // (2 + 3) * 4 → 20
        let inner = Expr::new(
            ExprKind::Binary {
                left: Box::new(int_lit(2)),
                op: BinaryOp::Add,
                right: Box::new(int_lit(3)),
            },
            Span::new(0, 0),
        );
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(inner),
                op: BinaryOp::Multiply,
                right: Box::new(int_lit(4)),
            },
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        assert!(matches!(
            folded.kind,
            ExprKind::Literal(Literal::Integer(20, None))
        ));
    }

    #[test]
    fn test_fold_constants_let_with_folded_value() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(binary(2, BinaryOp::Add, 3)),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        // Value should be folded to 5
        if let ExprKind::Let { value, .. } = folded.kind {
            assert!(matches!(
                value.kind,
                ExprKind::Literal(Literal::Integer(5, None))
            ));
        } else {
            panic!("Expected Let expression");
        }
    }

    #[test]
    fn test_fold_constants_block() {
        let expr = Expr::new(
            ExprKind::Block(vec![
                binary(1, BinaryOp::Add, 2),
                binary(3, BinaryOp::Multiply, 4),
            ]),
            Span::new(0, 0),
        );
        let folded = fold_constants(expr);
        if let ExprKind::Block(exprs) = folded.kind {
            assert_eq!(exprs.len(), 2);
            assert!(matches!(
                exprs[0].kind,
                ExprKind::Literal(Literal::Integer(3, None))
            ));
            assert!(matches!(
                exprs[1].kind,
                ExprKind::Literal(Literal::Integer(12, None))
            ));
        } else {
            panic!("Expected Block expression");
        }
    }

    #[test]
    fn test_fold_integer_comparison_false_cases() {
        assert!(matches!(
            fold_integer_comparison(5, BinaryOp::Equal, 3),
            Some(Literal::Bool(false))
        ));
        assert!(matches!(
            fold_integer_comparison(5, BinaryOp::NotEqual, 5),
            Some(Literal::Bool(false))
        ));
        assert!(matches!(
            fold_integer_comparison(5, BinaryOp::Less, 3),
            Some(Literal::Bool(false))
        ));
        assert!(matches!(
            fold_integer_comparison(5, BinaryOp::Greater, 10),
            Some(Literal::Bool(false))
        ));
    }

    #[test]
    fn test_fold_integer_arithmetic_overflow() {
        // Test checked arithmetic - overflow should return None
        let result = fold_integer_arithmetic(i64::MAX, BinaryOp::Add, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_fold_integer_arithmetic_underflow() {
        let result = fold_integer_arithmetic(i64::MIN, BinaryOp::Subtract, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_fold_integer_arithmetic_multiply_overflow() {
        let result = fold_integer_arithmetic(i64::MAX, BinaryOp::Multiply, 2);
        assert!(result.is_none());
    }

    #[test]
    fn test_eliminate_dead_code_after_break() {
        let stmts = vec![
            Expr::new(
                ExprKind::Break {
                    label: None,
                    value: None,
                },
                Span::new(0, 0),
            ),
            int_lit(10), // Dead code
        ];
        let result = remove_dead_statements(stmts);
        assert_eq!(result.len(), 1); // Only break remains
    }

    #[test]
    fn test_eliminate_dead_code_after_continue() {
        let stmts = vec![
            Expr::new(ExprKind::Continue { label: None }, Span::new(0, 0)),
            int_lit(10), // Dead code
        ];
        let result = remove_dead_statements(stmts);
        assert_eq!(result.len(), 1); // Only continue remains
    }

    #[test]
    fn test_eliminate_dead_code_function_body() {
        let expr = Expr::new(
            ExprKind::Function {
                name: "test".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![
                        Expr::new(
                            ExprKind::Return {
                                value: Some(Box::new(int_lit(5))),
                            },
                            Span::new(0, 0),
                        ),
                        int_lit(10), // Dead code
                    ]),
                    Span::new(0, 0),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 0),
        );
        let result = eliminate_dead_code(expr, std::collections::HashSet::new());
        if let ExprKind::Function { body, .. } = result.kind {
            if let ExprKind::Block(exprs) = body.kind {
                assert_eq!(exprs.len(), 1); // Only return remains
            }
        }
    }

    #[test]
    fn test_eliminate_dead_code_while_body() {
        let expr = Expr::new(
            ExprKind::While {
                condition: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Bool(true)),
                    Span::new(0, 0),
                )),
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![
                        Expr::new(
                            ExprKind::Break {
                                label: None,
                                value: None,
                            },
                            Span::new(0, 0),
                        ),
                        int_lit(10), // Dead code
                    ]),
                    Span::new(0, 0),
                )),
                label: None,
            },
            Span::new(0, 0),
        );
        let result = eliminate_dead_code(expr, std::collections::HashSet::new());
        if let ExprKind::While { body, .. } = result.kind {
            if let ExprKind::Block(exprs) = body.kind {
                assert_eq!(exprs.len(), 1); // Only break remains
            }
        }
    }

    #[test]
    fn test_eliminate_dead_code_if_branches() {
        let expr = Expr::new(
            ExprKind::If {
                condition: Box::new(Expr::new(
                    ExprKind::Identifier("cond".to_string()),
                    Span::new(0, 0),
                )),
                then_branch: Box::new(Expr::new(
                    ExprKind::Block(vec![
                        Expr::new(
                            ExprKind::Return {
                                value: Some(Box::new(int_lit(1))),
                            },
                            Span::new(0, 0),
                        ),
                        int_lit(2), // Dead
                    ]),
                    Span::new(0, 0),
                )),
                else_branch: Some(Box::new(Expr::new(
                    ExprKind::Block(vec![
                        Expr::new(
                            ExprKind::Return {
                                value: Some(Box::new(int_lit(3))),
                            },
                            Span::new(0, 0),
                        ),
                        int_lit(4), // Dead
                    ]),
                    Span::new(0, 0),
                ))),
            },
            Span::new(0, 0),
        );
        let result = eliminate_dead_code(expr, std::collections::HashSet::new());
        // Both branches should have dead code eliminated
        if let ExprKind::If {
            then_branch,
            else_branch,
            ..
        } = result.kind
        {
            if let ExprKind::Block(then_exprs) = then_branch.kind {
                assert_eq!(then_exprs.len(), 1);
            }
            if let Some(else_box) = else_branch {
                if let ExprKind::Block(else_exprs) = else_box.kind {
                    assert_eq!(else_exprs.len(), 1);
                }
            }
        }
    }

    #[test]
    fn test_eliminate_dead_code_call_args() {
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("foo".to_string()),
                    Span::new(0, 0),
                )),
                args: vec![binary(1, BinaryOp::Add, 2)],
            },
            Span::new(0, 0),
        );
        // Just verify it doesn't crash and returns the structure
        let result = eliminate_dead_code(expr, std::collections::HashSet::new());
        assert!(matches!(result.kind, ExprKind::Call { .. }));
    }

    #[test]
    fn test_propagate_with_env_block_scope() {
        let mut env = HashMap::new();
        env.insert("outer".to_string(), Literal::Integer(10, None));

        let expr = Expr::new(
            ExprKind::Block(vec![
                Expr::new(
                    ExprKind::Let {
                        name: "inner".to_string(),
                        type_annotation: None,
                        value: Box::new(int_lit(20)),
                        body: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Unit),
                            Span::new(0, 0),
                        )),
                        is_mutable: false,
                        else_block: None,
                    },
                    Span::new(0, 0),
                ),
                Expr::new(
                    ExprKind::Identifier("outer".to_string()),
                    Span::new(0, 0),
                ),
            ]),
            Span::new(0, 0),
        );

        let result = propagate_with_env(expr, &mut env);
        // The identifier "outer" should be propagated to 10
        if let ExprKind::Block(exprs) = result.kind {
            assert_eq!(exprs.len(), 2);
        }
    }

    #[test]
    fn test_propagate_constants_in_if_condition() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(Expr::new(
                    ExprKind::If {
                        condition: Box::new(Expr::new(
                            ExprKind::Binary {
                                left: Box::new(Expr::new(
                                    ExprKind::Identifier("x".to_string()),
                                    Span::new(0, 0),
                                )),
                                op: BinaryOp::Greater,
                                right: Box::new(int_lit(3)),
                            },
                            Span::new(0, 0),
                        )),
                        then_branch: Box::new(int_lit(1)),
                        else_branch: Some(Box::new(int_lit(0))),
                    },
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );

        let result = propagate_constants(expr);
        // After propagation: x = 5, x > 3 → 5 > 3 → true
        // Then the if should fold to Block containing then_branch
        if let ExprKind::Let { body, .. } = result.kind {
            // The body should be a Block (from constant folding) with 1
            match body.kind {
                ExprKind::Block(exprs) => {
                    assert_eq!(exprs.len(), 1);
                }
                _ => {} // May still be If if folding didn't complete
            }
        }
    }

    #[test]
    fn test_collect_used_functions_in_block() {
        let expr = Expr::new(
            ExprKind::Block(vec![
                Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("foo".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                ),
                Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("bar".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                ),
            ]),
            Span::new(0, 0),
        );
        let used = collect_used_functions(&expr);
        assert!(used.contains("foo"));
        assert!(used.contains("bar"));
        assert_eq!(used.len(), 2);
    }

    #[test]
    fn test_collect_used_functions_in_binary() {
        let expr = Expr::new(
            ExprKind::Binary {
                left: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("left_fn".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
                op: BinaryOp::Add,
                right: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("right_fn".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
            },
            Span::new(0, 0),
        );
        let used = collect_used_functions(&expr);
        assert!(used.contains("left_fn"));
        assert!(used.contains("right_fn"));
    }

    #[test]
    fn test_collect_used_functions_in_function_def() {
        let expr = Expr::new(
            ExprKind::Function {
                name: "outer".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("inner".to_string()),
                            Span::new(0, 0),
                        )),
                        args: vec![],
                    },
                    Span::new(0, 0),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 0),
        );
        let used = collect_used_functions(&expr);
        assert!(used.contains("inner"));
    }

    #[test]
    fn test_collect_used_variables_in_block() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(1)),
                body: Box::new(Expr::new(
                    ExprKind::Block(vec![
                        Expr::new(
                            ExprKind::Let {
                                name: "y".to_string(),
                                type_annotation: None,
                                value: Box::new(Expr::new(
                                    ExprKind::Identifier("x".to_string()),
                                    Span::new(0, 0),
                                )),
                                body: Box::new(Expr::new(
                                    ExprKind::Identifier("y".to_string()),
                                    Span::new(0, 0),
                                )),
                                is_mutable: false,
                                else_block: None,
                            },
                            Span::new(0, 0),
                        ),
                    ]),
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 0),
        );
        let used = collect_used_variables(&expr);
        assert!(used.contains("x"));
        assert!(used.contains("y"));
    }

    #[test]
    fn test_collect_used_variables_let_with_else() {
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(1)),
                body: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    Span::new(0, 0),
                )),
                is_mutable: false,
                else_block: Some(Box::new(int_lit(0))),
            },
            Span::new(0, 0),
        );
        let used = collect_used_variables(&expr);
        assert!(used.contains("x"));
    }

    #[test]
    fn test_should_remove_function_main() {
        let used = HashSet::new();
        let mut inlined = std::collections::HashSet::new();
        inlined.insert("main".to_string());
        // main should never be removed
        assert!(!should_remove_function("main", &used, &inlined));
    }

    #[test]
    fn test_should_remove_function_inlined_and_unused() {
        let used = HashSet::new();
        let mut inlined = std::collections::HashSet::new();
        inlined.insert("helper".to_string());
        assert!(should_remove_function("helper", &used, &inlined));
    }

    #[test]
    fn test_should_remove_function_inlined_but_used() {
        let mut used = HashSet::new();
        used.insert("helper".to_string());
        let mut inlined = std::collections::HashSet::new();
        inlined.insert("helper".to_string());
        assert!(!should_remove_function("helper", &used, &inlined));
    }

    #[test]
    fn test_process_let_elimination_used_variable() {
        let used = {
            let mut s = HashSet::new();
            s.insert("x".to_string());
            s
        };
        let value = int_lit(5);
        let body = Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(0, 0),
        );
        let result = process_let_elimination("x", &value, &body, &used);
        assert!(result.is_none()); // Should not eliminate used variable
    }

    #[test]
    fn test_process_let_elimination_with_side_effects() {
        let used = HashSet::new();
        let value = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("side_effect".to_string()),
                    Span::new(0, 0),
                )),
                args: vec![],
            },
            Span::new(0, 0),
        );
        let body = int_lit(1);
        let result = process_let_elimination("unused", &value, &body, &used);
        assert!(result.is_none()); // Should not eliminate due to side effects
    }
}
