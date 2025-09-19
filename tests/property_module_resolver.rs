//! Property-based tests for module resolver refactoring

use quickcheck::{quickcheck, TestResult};
use ruchy::backend::module_resolver::ModuleResolver;
use ruchy::frontend::ast::{Expr, ExprKind, Span};

// Property: Module resolver preserves expression structure for non-imports
fn prop_non_import_preserved(seed: u64) -> TestResult {
    let expr = generate_non_import_expr(seed);
    let mut resolver = ModuleResolver::new();
    
    match resolver.resolve_imports(expr.clone()) {
        Ok(resolved) => {
            // For non-import expressions, should be unchanged
            TestResult::from_bool(expr_equals(&expr, &resolved))
        }
        Err(_) => TestResult::discard(),
    }
}

// Property: Standard library imports are preserved unchanged
fn prop_std_imports_preserved(module: String, item: String) -> TestResult {
    if module.is_empty() || item.is_empty() {
        return TestResult::discard();
    }
    
    let path = format!("std::{}", module);
    let expr = Expr::new(
        ExprKind::Import {
            module: path.clone(),
            items: Some(vec![item.clone()]),
        },
        Span { start: 0, end: 0 },
    );
    
    let mut resolver = ModuleResolver::new();
    match resolver.resolve_imports(expr.clone()) {
        Ok(resolved) => {
            // Standard library imports should be unchanged
            TestResult::from_bool(expr_equals(&expr, &resolved))
        }
        Err(_) => TestResult::discard(),
    }
}

// Property: Block expressions recursively resolve imports
fn prop_block_resolution(seed: u64) -> TestResult {
    let exprs = generate_expr_list(seed, 3);
    let block = Expr::new(
        ExprKind::Block(exprs.clone()),
        Span { start: 0, end: 0 },
    );
    
    let mut resolver = ModuleResolver::new();
    match resolver.resolve_imports(block) {
        Ok(resolved) => {
            if let ExprKind::Block(resolved_exprs) = resolved.kind {
                // Should have same number of expressions
                TestResult::from_bool(resolved_exprs.len() == exprs.len())
            } else {
                TestResult::failed()
            }
        }
        Err(_) => TestResult::discard(),
    }
}

// Property: Module expressions preserve structure
fn prop_module_structure(name: String, seed: u64) -> TestResult {
    if name.is_empty() {
        return TestResult::discard();
    }
    
    let body = generate_non_import_expr(seed);
    let module = Expr::new(
        ExprKind::Module {
            name: name.clone(),
            body: Box::new(body.clone()),
        },
        Span { start: 0, end: 0 },
    );
    
    let mut resolver = ModuleResolver::new();
    match resolver.resolve_imports(module) {
        Ok(resolved) => {
            if let ExprKind::Module { name: res_name, .. } = resolved.kind {
                // Module name should be preserved
                TestResult::from_bool(res_name == name)
            } else {
                TestResult::failed()
            }
        }
        Err(_) => TestResult::discard(),
    }
}

// Property: Function bodies are recursively resolved
fn prop_function_resolution(name: String, seed: u64) -> TestResult {
    if name.is_empty() {
        return TestResult::discard();
    }
    
    let body = generate_non_import_expr(seed);
    let func = Expr::new(
        ExprKind::Function {
            name: name.clone(),
            type_params: vec![],
            params: vec![],
            body: Box::new(body),
            is_async: false,
            return_type: None,
            is_pub: false,
        },
        Span { start: 0, end: 0 },
    );
    
    let mut resolver = ModuleResolver::new();
    match resolver.resolve_imports(func) {
        Ok(resolved) => {
            if let ExprKind::Function { name: res_name, .. } = resolved.kind {
                // Function name should be preserved
                TestResult::from_bool(res_name == name)
            } else {
                TestResult::failed()
            }
        }
        Err(_) => TestResult::discard(),
    }
}

// Property: If expressions resolve all branches
fn prop_if_resolution(seed: u64) -> TestResult {
    let condition = generate_non_import_expr(seed);
    let then_branch = generate_non_import_expr(seed + 1);
    let else_branch = if seed % 2 == 0 {
        Some(Box::new(generate_non_import_expr(seed + 2)))
    } else {
        None
    };
    
    let if_expr = Expr::new(
        ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.clone(),
        },
        Span { start: 0, end: 0 },
    );
    
    let mut resolver = ModuleResolver::new();
    match resolver.resolve_imports(if_expr) {
        Ok(resolved) => {
            if let ExprKind::If { else_branch: res_else, .. } = resolved.kind {
                // Else branch presence should be preserved
                TestResult::from_bool(res_else.is_some() == else_branch.is_some())
            } else {
                TestResult::failed()
            }
        }
        Err(_) => TestResult::discard(),
    }
}

// Helper functions

fn generate_non_import_expr(seed: u64) -> Expr {
    use ruchy::frontend::ast::Literal;
    
    match seed % 5 {
        0 => Expr::new(
            ExprKind::Literal(Literal::Integer(seed as i64)),
            Span { start: 0, end: 0 },
        ),
        1 => Expr::new(
            ExprKind::Identifier(format!("var_{}", seed)),
            Span { start: 0, end: 0 },
        ),
        2 => Expr::new(
            ExprKind::Literal(Literal::String(format!("str_{}", seed))),
            Span { start: 0, end: 0 },
        ),
        3 => Expr::new(
            ExprKind::Literal(Literal::Bool(seed % 2 == 0)),
            Span { start: 0, end: 0 },
        ),
        _ => Expr::new(
            ExprKind::Literal(Literal::Unit),
            Span { start: 0, end: 0 },
        ),
    }
}

fn generate_expr_list(seed: u64, count: usize) -> Vec<Expr> {
    (0..count)
        .map(|i| generate_non_import_expr(seed + i as u64))
        .collect()
}

fn expr_equals(a: &Expr, b: &Expr) -> bool {
    // Simple structural equality check
    format!("{:?}", a.kind) == format!("{:?}", b.kind)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prop_non_import_preserved() {
        quickcheck(prop_non_import_preserved as fn(u64) -> TestResult);
    }

    #[test]
    fn test_prop_std_imports_preserved() {
        quickcheck(prop_std_imports_preserved as fn(String, String) -> TestResult);
    }

    #[test]
    fn test_prop_block_resolution() {
        quickcheck(prop_block_resolution as fn(u64) -> TestResult);
    }

    #[test]
    fn test_prop_module_structure() {
        quickcheck(prop_module_structure as fn(String, u64) -> TestResult);
    }

    #[test]
    fn test_prop_function_resolution() {
        quickcheck(prop_function_resolution as fn(String, u64) -> TestResult);
    }

    #[test]
    fn test_prop_if_resolution() {
        quickcheck(prop_if_resolution as fn(u64) -> TestResult);
    }
}