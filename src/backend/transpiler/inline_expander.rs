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
            is_pub,
            ..
        } => {
            // Check if function is small enough to inline (≤10 LOC heuristic)
            let body_size = estimate_body_size(body);
            let is_recursive = check_recursion(name, body);
            let accesses_globals = accesses_global_variables(params, body);

            // TRANSPILER-001: Don't inline functions that access global variables
            // Inlining them breaks scope - global vars are not accessible where inlined
            // TRANSPILER-136: Don't inline pub fun - they must be preserved for library exports
            if body_size <= 10 && !is_recursive && !accesses_globals && !is_pub {
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
            value.as_ref().is_some_and(|v| check_recursion(func_name, v))
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
        ExprKind::CompoundAssign { target, value, .. } => {
            // FIX: Detect globals in compound assignments (total += x)
            check_for_external_refs(target, allowed) || check_for_external_refs(value, allowed)
        }
        ExprKind::Binary { left, right, .. } => {
            check_for_external_refs(left, allowed) || check_for_external_refs(right, allowed)
        }
        ExprKind::Block(exprs) => exprs.iter().any(|e| check_for_external_refs(e, allowed)),
        ExprKind::If { condition, then_branch, else_branch } => {
            check_for_external_refs(condition, allowed)
                || check_for_external_refs(then_branch, allowed)
                || else_branch.as_ref().is_some_and(|e| check_for_external_refs(e, allowed))
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
            value.as_ref().is_some_and(|v| check_for_external_refs(v, allowed))
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

    // Helper: Create integer literal
    fn int_lit(n: i64) -> Expr {
        Expr::new(
            ExprKind::Literal(Literal::Integer(n, None)),
            Span::default(),
        )
    }

    // Helper: Create identifier
    fn ident(name: &str) -> Expr {
        Expr::new(
            ExprKind::Identifier(name.to_string()),
            Span::default(),
        )
    }

    // Helper: Create binary expression
    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr::new(
            ExprKind::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
            Span::default(),
        )
    }

    // Test 4: estimate_body_size - single expression
    #[test]
    fn test_estimate_body_size_single() {
        let expr = int_lit(42);
        assert_eq!(estimate_body_size(&expr), 1);
    }

    // Test 5: estimate_body_size - block with multiple expressions
    #[test]
    fn test_estimate_body_size_block() {
        let block = Expr::new(
            ExprKind::Block(vec![int_lit(1), int_lit(2), int_lit(3)]),
            Span::default(),
        );
        assert_eq!(estimate_body_size(&block), 3);
    }

    // Test 6: estimate_body_size - let expression
    #[test]
    fn test_estimate_body_size_let() {
        let let_expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(int_lit(10)),
                is_mutable: false,
                else_block: None,
            },
            Span::default(),
        );
        assert_eq!(estimate_body_size(&let_expr), 2); // 1 + body size
    }

    // Test 7: estimate_body_size - if expression with else
    #[test]
    fn test_estimate_body_size_if_else() {
        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(ident("x")),
                then_branch: Box::new(int_lit(1)),
                else_branch: Some(Box::new(int_lit(2))),
            },
            Span::default(),
        );
        assert_eq!(estimate_body_size(&if_expr), 3); // 1 + then + else
    }

    // Test 8: check_recursion - direct recursion
    #[test]
    fn test_check_recursion_direct() {
        let call = Expr::new(
            ExprKind::Call {
                func: Box::new(ident("factorial")),
                args: vec![],
            },
            Span::default(),
        );
        assert!(check_recursion("factorial", &call));
    }

    // Test 9: check_recursion - recursion in binary expression
    #[test]
    fn test_check_recursion_binary() {
        let expr = binary(
            Expr::new(
                ExprKind::Call {
                    func: Box::new(ident("fib")),
                    args: vec![],
                },
                Span::default(),
            ),
            BinaryOp::Add,
            int_lit(1),
        );
        assert!(check_recursion("fib", &expr));
    }

    // Test 10: check_recursion - recursion in if expression
    #[test]
    fn test_check_recursion_if() {
        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(ident("x")),
                then_branch: Box::new(Expr::new(
                    ExprKind::Call {
                        func: Box::new(ident("recurse")),
                        args: vec![],
                    },
                    Span::default(),
                )),
                else_branch: None,
            },
            Span::default(),
        );
        assert!(check_recursion("recurse", &if_expr));
    }

    // Test 11: check_recursion - no recursion
    #[test]
    fn test_check_recursion_none() {
        let expr = binary(int_lit(1), BinaryOp::Add, int_lit(2));
        assert!(!check_recursion("foo", &expr));
    }

    // Test 12: accesses_global_variables - no globals (all params)
    #[test]
    fn test_accesses_global_variables_none() {
        let params = vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }];
        let body = ident("x");
        assert!(!accesses_global_variables(&params, &body));
    }

    // Test 13: accesses_global_variables - accesses global
    #[test]
    fn test_accesses_global_variables_global() {
        let params = vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }];
        let body = binary(ident("x"), BinaryOp::Add, ident("global_var"));
        assert!(accesses_global_variables(&params, &body));
    }

    // Test 14: accesses_global_variables - let binding not global
    #[test]
    fn test_accesses_global_variables_let_binding() {
        let params = vec![];
        let body = Expr::new(
            ExprKind::Let {
                name: "y".to_string(),
                type_annotation: None,
                value: Box::new(int_lit(5)),
                body: Box::new(ident("y")), // References let binding, not global
                is_mutable: false,
                else_block: None,
            },
            Span::default(),
        );
        assert!(!accesses_global_variables(&params, &body));
    }

    // Test 15: check_for_external_refs - identifier in allowed set
    #[test]
    fn test_check_for_external_refs_allowed() {
        use std::collections::HashSet;
        let mut allowed = HashSet::new();
        allowed.insert("x".to_string());
        assert!(!check_for_external_refs(&ident("x"), &allowed));
    }

    // Test 16: check_for_external_refs - identifier not allowed
    #[test]
    fn test_check_for_external_refs_external() {
        use std::collections::HashSet;
        let allowed = HashSet::new();
        assert!(check_for_external_refs(&ident("external"), &allowed));
    }

    // Test 17: check_for_external_refs - compound assign
    #[test]
    fn test_check_for_external_refs_compound_assign() {
        use std::collections::HashSet;
        let mut allowed = HashSet::new();
        allowed.insert("total".to_string());
        let expr = Expr::new(
            ExprKind::CompoundAssign {
                target: Box::new(ident("total")),
                op: BinaryOp::Add,
                value: Box::new(ident("x")), // x is external
            },
            Span::default(),
        );
        assert!(check_for_external_refs(&expr, &allowed));
    }

    // Test 18: substitute_identifiers - identifier substitution
    #[test]
    fn test_substitute_identifiers_identifier() {
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), int_lit(42));
        let result = substitute_identifiers(ident("x"), &subs);
        assert!(matches!(result.kind, ExprKind::Literal(Literal::Integer(42, None))));
    }

    // Test 19: substitute_identifiers - binary expression
    #[test]
    fn test_substitute_identifiers_binary() {
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), int_lit(5));
        let expr = binary(ident("x"), BinaryOp::Add, int_lit(1));
        let result = substitute_identifiers(expr, &subs);
        if let ExprKind::Binary { left, .. } = result.kind {
            assert!(matches!(left.kind, ExprKind::Literal(Literal::Integer(5, None))));
        } else {
            panic!("Expected binary expression");
        }
    }

    // Test 20: substitute_identifiers - call expression
    #[test]
    fn test_substitute_identifiers_call() {
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), int_lit(10));
        let expr = Expr::new(
            ExprKind::Call {
                func: Box::new(ident("foo")),
                args: vec![ident("x")],
            },
            Span::default(),
        );
        let result = substitute_identifiers(expr, &subs);
        if let ExprKind::Call { args, .. } = result.kind {
            assert!(matches!(args[0].kind, ExprKind::Literal(Literal::Integer(10, None))));
        } else {
            panic!("Expected call expression");
        }
    }

    // Test 21: substitute_identifiers - block
    #[test]
    fn test_substitute_identifiers_block() {
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), int_lit(7));
        let block = Expr::new(
            ExprKind::Block(vec![ident("x"), ident("x")]),
            Span::default(),
        );
        let result = substitute_identifiers(block, &subs);
        if let ExprKind::Block(exprs) = result.kind {
            assert_eq!(exprs.len(), 2);
            assert!(matches!(exprs[0].kind, ExprKind::Literal(Literal::Integer(7, None))));
            assert!(matches!(exprs[1].kind, ExprKind::Literal(Literal::Integer(7, None))));
        } else {
            panic!("Expected block");
        }
    }

    // Test 22: substitute_identifiers - if expression
    #[test]
    fn test_substitute_identifiers_if() {
        let mut subs = HashMap::new();
        subs.insert("x".to_string(), int_lit(1));
        let if_expr = Expr::new(
            ExprKind::If {
                condition: Box::new(ident("x")),
                then_branch: Box::new(ident("x")),
                else_branch: Some(Box::new(int_lit(0))),
            },
            Span::default(),
        );
        let result = substitute_identifiers(if_expr, &subs);
        if let ExprKind::If { condition, then_branch, .. } = result.kind {
            assert!(matches!(condition.kind, ExprKind::Literal(Literal::Integer(1, None))));
            assert!(matches!(then_branch.kind, ExprKind::Literal(Literal::Integer(1, None))));
        } else {
            panic!("Expected if expression");
        }
    }

    // Test 23: substitute_identifiers - return expression
    #[test]
    fn test_substitute_identifiers_return() {
        let mut subs = HashMap::new();
        subs.insert("result".to_string(), int_lit(99));
        let ret = Expr::new(
            ExprKind::Return {
                value: Some(Box::new(ident("result"))),
            },
            Span::default(),
        );
        let result = substitute_identifiers(ret, &subs);
        if let ExprKind::Return { value } = result.kind {
            let val = value.unwrap();
            assert!(matches!(val.kind, ExprKind::Literal(Literal::Integer(99, None))));
        } else {
            panic!("Expected return expression");
        }
    }
}
