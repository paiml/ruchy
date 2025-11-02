// OPT-CODEGEN-004: Inline Expansion Optimization
// GREEN Phase: Minimal implementation to make tests pass
// Complexity target: ≤10 per function

use crate::frontend::ast::{Expr, ExprKind, Param};
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
/// Expression with eligible functions inlined
///
/// # Complexity
/// Cyclomatic: 4 (≤10 target)
pub fn inline_small_functions(expr: Expr) -> Expr {
    // First pass: collect inline candidates (small, non-recursive functions)
    let mut functions = HashMap::new();
    collect_inline_candidates(&expr, &mut functions);

    // Second pass: replace function calls with inlined bodies
    inline_function_calls(expr, &functions)
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

            if body_size <= 10 && !is_recursive {
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
/// Cyclomatic: 7 (≤10 target)
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
        _ => expr, // Other expressions: return as-is
    }
}

/// Estimate body size (LOC heuristic)
///
/// # Complexity
/// Cyclomatic: 5 (≤10 target)
fn estimate_body_size(body: &Expr) -> usize {
    match &body.kind {
        ExprKind::Block(exprs) => exprs.len(),
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
/// Cyclomatic: 6 (≤10 target)
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
        ExprKind::Block(exprs) => exprs.iter().any(|e| check_recursion(func_name, e)),
        ExprKind::If { condition, then_branch, else_branch } => {
            check_recursion(func_name, condition)
                || check_recursion(func_name, then_branch)
                || else_branch.as_ref().map_or(false, |e| check_recursion(func_name, e))
        }
        ExprKind::Let { value, body, .. } => {
            check_recursion(func_name, value) || check_recursion(func_name, body)
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

        let result = inline_small_functions(block);

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

        let result = inline_small_functions(block);

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
