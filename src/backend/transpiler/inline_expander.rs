// OPT-CODEGEN-004: Inline Expansion Optimization
// GREEN Phase: Minimal implementation to make tests pass
// Complexity target: ≤10 per function

use crate::frontend::ast::{Expr, ExprKind, Param, Pattern};
use std::collections::HashMap;

/// Inline small, non-recursive functions into their call sites
///
/// Examples:
/// - `fun add(x) { x + 1 }; add(5)` → `5 + 1`
/// - Skips large functions (>10 LOC heuristic)
/// - Skips recursive functions (correctness risk)
///
/// # Arguments
/// * `expr` - Expression tree to optimize
///
/// # Returns
/// Tuple of (optimized expression, set of inlined function names)
///
/// # Complexity
/// Cyclomatic: 4 (≤10 target)
pub fn inline_small_functions(expr: Expr) -> (Expr, std::collections::HashSet<String>) {
    use std::collections::HashSet;

    // First pass: collect inline candidates (small, non-recursive functions)
    let mut functions = HashMap::new();
    collect_inline_candidates(&expr, &mut functions);

    // Track which functions are candidates for inlining
    let inline_candidates: HashSet<String> = functions.keys().cloned().collect();

    // Second pass: replace function calls with inlined bodies
    let optimized = inline_function_calls(expr, &functions);

    (optimized, inline_candidates)
}

/// Collect functions that are candidates for inlining
///
/// # Complexity
/// Cyclomatic: 6 (≤10 target)
fn collect_inline_candidates(expr: &Expr, functions: &mut HashMap<String, FunctionDef>) {
    match &expr.kind {
        ExprKind::Function {
            name,
            params,
            body,
            ..
        } => {
            // Check if function is small enough to inline (≤10 LOC heuristic)
            let body_size = estimate_body_size(body);
            let is_recursive = check_recursion(name, body);
            let accesses_globals = accesses_global_variables(params, body);

            // TRANSPILER-001: Don't inline functions that access global variables
            // Inlining them breaks scope - global vars are not accessible where inlined
            if body_size <= 10 && !is_recursive && !accesses_globals {
                functions.insert(
                    name.clone(),
                    FunctionDef {
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
        }
        ExprKind::Block(exprs) => {
            for e in exprs {
                collect_inline_candidates(e, functions);
            }
        }
        _ => {}
    }
}

/// Inline function calls by substituting function bodies
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn inline_function_calls(expr: Expr, functions: &HashMap<String, FunctionDef>) -> Expr {
    match expr.kind {
        ExprKind::Call { func, args } => {
            // Check if this is a simple function call (not method call)
            if let ExprKind::Identifier(func_name) = &func.kind {
                if let Some(func_def) = functions.get(func_name) {
                    // Inline: substitute parameters with arguments
                    return substitute_params(&func_def.body, &func_def.params, &args);
                }
            }
            // Not inlineable - recursively process children
            Expr::new(
                ExprKind::Call {
                    func: Box::new(inline_function_calls(*func, functions)),
                    args: args.into_iter().map(|a| inline_function_calls(a, functions)).collect(),
                },
                expr.span,
            )
        }
        ExprKind::Block(exprs) => {
            let inlined_exprs = exprs.into_iter().map(|e| inline_function_calls(e, functions)).collect();
            Expr::new(ExprKind::Block(inlined_exprs), expr.span)
        }
        ExprKind::Let { name, type_annotation, value, body, is_mutable, else_block } => {
            Expr::new(
                ExprKind::Let {
                    name,
                    type_annotation,
                    value: Box::new(inline_function_calls(*value, functions)),
                    body: Box::new(inline_function_calls(*body, functions)),
                    is_mutable,
                    else_block: else_block.map(|e| Box::new(inline_function_calls(*e, functions))),
                },
                expr.span,
            )
        }
        ExprKind::Function { name, type_params, params, return_type, body, is_async, is_pub } => {
            // Recursively inline calls inside function body, but keep the function definition itself
            Expr::new(
                ExprKind::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body: Box::new(inline_function_calls(*body, functions)),
                    is_async,
                    is_pub,
                },
                expr.span,
            )
        }
        ExprKind::Binary { left, op, right } => {
            // Recursively inline calls in binary expressions (critical for nested inlining)
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(inline_function_calls(*left, functions)),
                    op,
                    right: Box::new(inline_function_calls(*right, functions)),
                },
                expr.span,
            )
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            // Recursively inline calls in if expressions
            Expr::new(
                ExprKind::If {
                    condition: Box::new(inline_function_calls(*condition, functions)),
                    then_branch: Box::new(inline_function_calls(*then_branch, functions)),
                    else_branch: else_branch.map(|e| Box::new(inline_function_calls(*e, functions))),
                },
                expr.span,
            )
        }
        _ => expr,
    }
}

/// Substitute function parameters with call arguments
///
/// # Complexity
/// Cyclomatic: 4 (≤10 target)
fn substitute_params(body: &Expr, params: &[Param], args: &[Expr]) -> Expr {
    let mut substitutions = HashMap::new();

    // Build parameter → argument mapping
    for (param, arg) in params.iter().zip(args.iter()) {
        if let crate::frontend::ast::Pattern::Identifier(param_name) = &param.pattern {
            substitutions.insert(param_name.clone(), arg.clone());
        }
    }

    // Recursively substitute in body
    substitute_identifiers(body.clone(), &substitutions)
}

/// Replace identifiers with their substituted values
///
/// # Complexity
/// Cyclomatic: 8 (≤10 target)
fn substitute_identifiers(expr: Expr, subs: &HashMap<String, Expr>) -> Expr {
    match &expr.kind {
        ExprKind::Identifier(name) => {
            // Substitute if this identifier is a parameter
            subs.get(name).cloned().unwrap_or(expr)
        }
        ExprKind::Binary { left, op, right } => {
            Expr::new(
                ExprKind::Binary {
                    left: Box::new(substitute_identifiers((**left).clone(), subs)),
                    op: *op,
                    right: Box::new(substitute_identifiers((**right).clone(), subs)),
                },
                expr.span,
            )
        }
        ExprKind::Call { func, args } => {
            Expr::new(
                ExprKind::Call {
                    func: Box::new(substitute_identifiers((**func).clone(), subs)),
                    args: args.iter().map(|a| substitute_identifiers(a.clone(), subs)).collect(),
                },
                expr.span,
            )
        }
        ExprKind::Block(exprs) => {
            Expr::new(
                ExprKind::Block(
                    exprs.iter().map(|e| substitute_identifiers(e.clone(), subs)).collect()
                ),
                expr.span,
            )
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            // ISSUE-128 FIX: Substitute in if-else expressions
            Expr::new(
                ExprKind::If {
                    condition: Box::new(substitute_identifiers((**condition).clone(), subs)),
                    then_branch: Box::new(substitute_identifiers((**then_branch).clone(), subs)),
                    else_branch: else_branch.as_ref().map(|e| Box::new(substitute_identifiers((**e).clone(), subs))),
                },
                expr.span,
            )
        }
        ExprKind::Return { value } => {
            // ISSUE-128 FIX: Substitute identifiers inside return expressions
            Expr::new(
                ExprKind::Return {
                    value: value.as_ref().map(|v| Box::new(substitute_identifiers((**v).clone(), subs)))
                },
                expr.span,
            )
        }
        _ => expr, // Other expressions: return as-is
    }
}

/// Estimate body size (LOC heuristic)
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn estimate_body_size(body: &Expr) -> usize {
    match &body.kind {
        ExprKind::Block(exprs) => {
            // Recursively sum sizes of all expressions in block
            exprs.iter().map(estimate_body_size).sum()
        }
        ExprKind::Let { body, .. } => 1 + estimate_body_size(body),
        ExprKind::If { then_branch, else_branch, .. } => {
            1 + estimate_body_size(then_branch)
                + else_branch.as_ref().map_or(0, |e| estimate_body_size(e))
        }
        _ => 1, // Single expression = 1 LOC
    }
}

/// Check if function body calls itself (recursion detection)
///
/// # Complexity
/// Cyclomatic: 8 (≤10 target)
fn check_recursion(func_name: &str, body: &Expr) -> bool {
    match &body.kind {
        ExprKind::Call { func, .. } => {
            if let ExprKind::Identifier(called_name) = &func.kind {
                if called_name == func_name {
                    return true; // Direct recursion
                }
            }
            false
        }
        ExprKind::Binary { left, right, .. } => {
            // ISSUE-128 FIX: Check for recursion inside binary expressions (e.g., fib(n-1) + fib(n-2))
            check_recursion(func_name, left) || check_recursion(func_name, right)
        }
        ExprKind::Block(exprs) => exprs.iter().any(|e| check_recursion(func_name, e)),
        ExprKind::If { condition, then_branch, else_branch } => {
            check_recursion(func_name, condition)
                || check_recursion(func_name, then_branch)
                || else_branch.as_ref().is_some_and(|e| check_recursion(func_name, e))
        }
        ExprKind::Let { value, body, .. } => {
            check_recursion(func_name, value) || check_recursion(func_name, body)
        }
        ExprKind::Return { value } => {
            // ISSUE-128 FIX: Check for recursion inside return expressions
            value.as_ref().map_or(false, |v| check_recursion(func_name, v))
        }
        _ => false,
    }
}

/// Check if function body accesses variables outside its parameters (global variables)
///
/// TRANSPILER-001: Functions that access globals should not be inlined
/// because the global variables won't be in scope at the inline site.
///
/// # Complexity
/// Cyclomatic: 7 (≤10 target)
fn accesses_global_variables(params: &[Param], body: &Expr) -> bool {
    use std::collections::HashSet;

    // Build set of parameter names
    let param_names: HashSet<String> = params.iter()
        .map(|p| match &p.pattern {
            Pattern::Identifier(name) => name.clone(),
            _ => String::new(),
        })
        .collect();

    // Check if body references any identifiers not in params
    check_for_external_refs(body, &param_names)
}

/// Recursively check if expression references identifiers outside the allowed set
///
/// # Complexity
/// Cyclomatic: 9 (≤10 target)
fn check_for_external_refs(expr: &Expr, allowed: &std::collections::HashSet<String>) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => !allowed.contains(name),
        ExprKind::Assign { target, value } => {
            check_for_external_refs(target, allowed) || check_for_external_refs(value, allowed)
        }
        ExprKind::Binary { left, right, .. } => {
            check_for_external_refs(left, allowed) || check_for_external_refs(right, allowed)
        }
        ExprKind::Block(exprs) => exprs.iter().any(|e| check_for_external_refs(e, allowed)),
        ExprKind::If { condition, then_branch, else_branch } => {
            check_for_external_refs(condition, allowed)
                || check_for_external_refs(then_branch, allowed)
                || else_branch.as_ref().map_or(false, |e| check_for_external_refs(e, allowed))
        }
        ExprKind::Let { name, value, body, .. } => {
            // Check value first (binding not yet available)
            if check_for_external_refs(value, allowed) {
                return true;
            }

            // Add bound variable to allowed set for checking body
            // OPT-CODEGEN-004 FIX: Track Let bindings to avoid false "global access" detection
            let mut allowed_with_binding = allowed.clone();
            allowed_with_binding.insert(name.clone());
            check_for_external_refs(body, &allowed_with_binding)
        }
        ExprKind::Return { value } => {
            value.as_ref().map_or(false, |v| check_for_external_refs(v, allowed))
        }
        ExprKind::Call { func, args } => {
            check_for_external_refs(func, allowed)
                || args.iter().any(|a| check_for_external_refs(a, allowed))
        }
        _ => false,
    }
}

/// Function definition for inlining
#[derive(Debug, Clone)]
struct FunctionDef {
    params: Vec<Param>,
    body: Box<Expr>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{BinaryOp, Literal, Pattern, Span, Type, TypeKind};

    #[test]
    fn test_inline_simple_function() {
        // fun add_one(x) { x + 1 }; add_one(5) → 5 + 1
        let func_def = Expr::new(
            ExprKind::Function {
                name: "add_one".to_string(),
                type_params: vec![],
                params: vec![Param {
                    pattern: Pattern::Identifier("x".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                }],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            Span::default(),
                        )),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(1, None)),
                            Span::default(),
                        )),
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(Expr::new(
                    ExprKind::Identifier("add_one".to_string()),
                    Span::default(),
                )),
                args: vec![Expr::new(
                    ExprKind::Literal(Literal::Integer(5, None)),
                    Span::default(),
                )],
            },
            Span::default(),
        );

        let block = Expr::new(
            ExprKind::Block(vec![func_def, call]),
            Span::default(),
        );

        let (result, _inlined) = inline_small_functions(block);

        // Should inline: call replaced with body
        if let ExprKind::Block(exprs) = result.kind {
            assert_eq!(exprs.len(), 2);
            // Second expression should be the inlined body (5 + 1)
            if let ExprKind::Binary { .. } = exprs[1].kind {
                // Success: inlined
            } else {
                panic!("Expected inlined binary expression");
            }
        }
    }

    #[test]
    fn test_inline_inside_function_body() {
        // fun add_one(x) { x + 1 }; fun main() { add_one(5) } → main should have inlined body
        let add_one_func = Expr::new(
            ExprKind::Function {
                name: "add_one".to_string(),
                type_params: vec![],
                params: vec![Param {
                    pattern: Pattern::Identifier("x".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::default(),
                    },
                    span: Span::default(),
                    is_mutable: false,
                    default_value: None,
                }],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Binary {
                        left: Box::new(Expr::new(
                            ExprKind::Identifier("x".to_string()),
                            Span::default(),
                        )),
                        op: BinaryOp::Add,
                        right: Box::new(Expr::new(
                            ExprKind::Literal(Literal::Integer(1, None)),
                            Span::default(),
                        )),
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let main_func = Expr::new(
            ExprKind::Function {
                name: "main".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("add_one".to_string()),
                            Span::default(),
                        )),
                        args: vec![Expr::new(
                            ExprKind::Literal(Literal::Integer(5, None)),
                            Span::default(),
                        )],
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let block = Expr::new(
            ExprKind::Block(vec![add_one_func, main_func]),
            Span::default(),
        );

        let (result, _inlined) = inline_small_functions(block);

        // Verify result
        if let ExprKind::Block(exprs) = &result.kind {
            assert_eq!(exprs.len(), 2);
            // Second expression should be main function with inlined body
            if let ExprKind::Function { name, body, .. } = &exprs[1].kind {
                assert_eq!(name, "main");
                // Body should be the inlined expression (5 + 1)
                if let ExprKind::Binary { .. } = body.kind {
                    // Success: call was inlined inside main's body
                } else {
                    panic!("Expected main's body to have inlined binary expression, got: {:?}", body.kind);
                }
            } else {
                panic!("Expected second expression to be main function");
            }
        } else {
            panic!("Expected block result");
        }
    }

    #[test]
    fn test_no_inline_recursive() {
        // Recursive function should NOT be inlined
        let recursive_func = Expr::new(
            ExprKind::Function {
                name: "factorial".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(Expr::new(
                            ExprKind::Identifier("factorial".to_string()),
                            Span::default(),
                        )),
                        args: vec![],
                    },
                    Span::default(),
                )),
                is_async: false,
                is_pub: false,
            },
            Span::default(),
        );

        let mut functions = HashMap::new();
        collect_inline_candidates(&recursive_func, &mut functions);

        // Should NOT collect recursive function
        assert!(!functions.contains_key("factorial"));
    }
}
