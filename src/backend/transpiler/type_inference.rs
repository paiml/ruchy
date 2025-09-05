//! Type inference helpers for transpiler
//! 
//! This module provides intelligent type inference by analyzing how
//! parameters and expressions are used in function bodies.

use crate::frontend::ast::{Expr, ExprKind, BinaryOp, Literal};

/// Analyzes if a parameter is used as an argument to a function that takes i32
pub fn is_param_used_as_function_argument(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, args } => {
            // Check if the function being called has the parameter as an argument
            if let ExprKind::Identifier(_func_name) = &func.kind {
                // If this is calling another function parameter (higher-order function)
                for arg in args {
                    if let ExprKind::Identifier(arg_name) = &arg.kind {
                        if arg_name == param_name {
                            return true; // Parameter is used as argument to function call
                        }
                    }
                }
            }
            // Recursively check arguments
            args.iter().any(|arg| is_param_used_as_function_argument(param_name, arg))
        }
        ExprKind::Block(exprs) => {
            exprs.iter().any(|e| is_param_used_as_function_argument(param_name, e))
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            is_param_used_as_function_argument(param_name, condition) ||
            is_param_used_as_function_argument(param_name, then_branch) ||
            else_branch.as_ref().is_some_and(|e| is_param_used_as_function_argument(param_name, e))
        }
        ExprKind::Let { value, body, .. } => {
            is_param_used_as_function_argument(param_name, value) ||
            is_param_used_as_function_argument(param_name, body)
        }
        ExprKind::LetPattern { value, body, .. } => {
            is_param_used_as_function_argument(param_name, value) ||
            is_param_used_as_function_argument(param_name, body)
        }
        ExprKind::Binary { left, right, .. } => {
            is_param_used_as_function_argument(param_name, left) ||
            is_param_used_as_function_argument(param_name, right)
        }
        ExprKind::Unary { operand, .. } => {
            is_param_used_as_function_argument(param_name, operand)
        }
        _ => false
    }
}

/// Analyzes if a parameter is used as a function in the given expression
pub fn is_param_used_as_function(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, args } => {
            // Check if the function being called is the parameter
            if let ExprKind::Identifier(name) = &func.kind {
                if name == param_name {
                    return true;
                }
            }
            // Recursively check arguments
            args.iter().any(|arg| is_param_used_as_function(param_name, arg))
        }
        ExprKind::Block(exprs) => {
            exprs.iter().any(|e| is_param_used_as_function(param_name, e))
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            is_param_used_as_function(param_name, condition) ||
            is_param_used_as_function(param_name, then_branch) ||
            else_branch.as_ref().is_some_and(|e| is_param_used_as_function(param_name, e))
        }
        ExprKind::Let { value, body, .. } => {
            is_param_used_as_function(param_name, value) ||
            is_param_used_as_function(param_name, body)
        }
        ExprKind::Binary { left, right, .. } => {
            is_param_used_as_function(param_name, left) ||
            is_param_used_as_function(param_name, right)
        }
        ExprKind::Lambda { body, .. } => {
            is_param_used_as_function(param_name, body)
        }
        _ => false
    }
}


/// Checks if a parameter is used in numeric operations
pub fn is_param_used_numerically(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Binary { op, left, right } => {
            check_binary_numeric_usage(param_name, op, left, right)
        }
        ExprKind::Block(exprs) => {
            check_block_numeric_usage(param_name, exprs)
        }
        ExprKind::If { condition, then_branch, else_branch } => {
            check_if_numeric_usage(param_name, condition, then_branch, else_branch.as_deref())
        }
        ExprKind::Let { value, body, .. } => {
            check_let_numeric_usage(param_name, value, body)
        }
        ExprKind::Call { args, .. } => {
            check_call_numeric_usage(param_name, args)
        }
        _ => false
    }
}

/// Check numeric usage in binary expressions (complexity: 6)
fn check_binary_numeric_usage(param_name: &str, op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
    if is_numeric_operator(op) && has_param_in_operation(param_name, left, right) {
        // Special case: string concatenation
        if is_string_concatenation(op, left, right) {
            return false;
        }
        return true;
    }
    
    // Recursively check both sides
    is_param_used_numerically(param_name, left) ||
    is_param_used_numerically(param_name, right)
}

/// Check if operator is numeric (complexity: 1)
fn is_numeric_operator(op: &BinaryOp) -> bool {
    matches!(op, 
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | 
        BinaryOp::Divide | BinaryOp::Modulo
    )
}

/// Check if param is in operation (complexity: 2)
fn has_param_in_operation(param_name: &str, left: &Expr, right: &Expr) -> bool {
    contains_param(param_name, left) || contains_param(param_name, right)
}

/// Check if operation is string concatenation (complexity: 3)
fn is_string_concatenation(op: &BinaryOp, left: &Expr, right: &Expr) -> bool {
    matches!(op, BinaryOp::Add) && 
    (is_string_literal(left) || is_string_literal(right))
}

/// Check numeric usage in blocks (complexity: 1)
fn check_block_numeric_usage(param_name: &str, exprs: &[Expr]) -> bool {
    exprs.iter().any(|e| is_param_used_numerically(param_name, e))
}

/// Check numeric usage in if expressions (complexity: 3)
fn check_if_numeric_usage(param_name: &str, condition: &Expr, then_branch: &Expr, else_branch: Option<&Expr>) -> bool {
    is_param_used_numerically(param_name, condition) ||
    is_param_used_numerically(param_name, then_branch) ||
    else_branch.is_some_and(|e| is_param_used_numerically(param_name, e))
}

/// Check numeric usage in let expressions (complexity: 2)
fn check_let_numeric_usage(param_name: &str, value: &Expr, body: &Expr) -> bool {
    is_param_used_numerically(param_name, value) ||
    is_param_used_numerically(param_name, body)
}

/// Check numeric usage in call arguments (complexity: 1)
fn check_call_numeric_usage(param_name: &str, args: &[Expr]) -> bool {
    args.iter().any(|arg| is_param_used_numerically(param_name, arg))
}

/// Helper to check if an expression contains a specific parameter
fn contains_param(param_name: &str, expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => name == param_name,
        ExprKind::Binary { left, right, .. } => {
            contains_param(param_name, left) || contains_param(param_name, right)
        }
        ExprKind::Block(exprs) => {
            exprs.iter().any(|e| contains_param(param_name, e))
        }
        ExprKind::Call { func, args } => {
            contains_param(param_name, func) ||
            args.iter().any(|arg| contains_param(param_name, arg))
        }
        _ => false
    }
}

/// Helper to check if an expression is a string literal
fn is_string_literal(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::Literal(Literal::String(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;

    /// Checks if an expression contains numeric operations (test helper)
    fn contains_numeric_operations(expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Binary { op, left, right } => {
                // Check for numeric operators
                matches!(op, 
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | 
                    BinaryOp::Divide | BinaryOp::Modulo | BinaryOp::Less | 
                    BinaryOp::Greater | BinaryOp::LessEqual | BinaryOp::GreaterEqual
                ) || contains_numeric_operations(left) || contains_numeric_operations(right)
            }
            ExprKind::Block(exprs) => {
                exprs.iter().any(contains_numeric_operations)
            }
            ExprKind::If { then_branch, else_branch, .. } => {
                contains_numeric_operations(then_branch) ||
                else_branch.as_ref().is_some_and(|e| contains_numeric_operations(e))
            }
            ExprKind::Let { value, body, .. } => {
                contains_numeric_operations(value) || contains_numeric_operations(body)
            }
            ExprKind::Call { args, .. } => {
                args.iter().any(contains_numeric_operations)
            }
            ExprKind::Lambda { body, .. } => {
                contains_numeric_operations(body)
            }
            _ => false
        }
    }

    #[test]
    fn test_detects_function_parameter() {
        let code = "fun test() { f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        // Find the function body
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(!is_param_used_as_function("x", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_operations() {
        let code = "fun test(x) { x * 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_numeric_operations(body));
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_nested_function_call() {
        let code = "fun test() { g(f(x)) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(is_param_used_as_function("g", body));
                    assert!(!is_param_used_as_function("x", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_function_in_if_branch() {
        let code = "fun test(p) { if (true) { p(5) } else { 0 } }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("p", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_function_in_let_body() {
        let code = "fun test(f) { let x = 5; f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_function_in_lambda() {
        let code = "fun test(f) { (x) => f(x) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_in_addition() {
        let code = "fun test(n) { n + 10 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_in_subtraction() {
        let code = "fun test(n) { n - 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_in_division() {
        let code = "fun test(n) { n / 2 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_in_modulo() {
        let code = "fun test(n) { n % 3 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_numerically("n", body));
                    assert!(contains_numeric_operations(body));
                }
            }
        }
    }

    #[test]
    fn test_detects_numeric_in_comparison() {
        let code = "fun test(n) { n > 5 }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_numerically("n", body)); // Comparisons don't count as numeric
                    assert!(contains_numeric_operations(body)); // But the expression contains numeric ops
                }
            }
        }
    }

    #[test]
    fn test_no_function_no_numeric() {
        let code = "fun test(s) { s + \" world\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(!is_param_used_as_function("s", body));
                    assert!(!is_param_used_numerically("s", body));
                }
            }
        }
    }

    #[test]
    fn test_contains_param_helper() {
        let code = "fun test(x) { x }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_param("x", body));
                    assert!(!contains_param("y", body));
                }
            }
        }
    }

    #[test]
    fn test_contains_param_in_binary() {
        let code = "fun test(x, y) { x + y }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(contains_param("x", body));
                    assert!(contains_param("y", body));
                    assert!(!contains_param("z", body));
                }
            }
        }
    }

    #[test]
    fn test_complex_nested_detection() {
        let code = "fun test(f, g, x) { f(g(x * 2)) }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_param_used_as_function("f", body));
                    assert!(is_param_used_as_function("g", body));
                    assert!(!is_param_used_as_function("x", body));
                    assert!(is_param_used_numerically("x", body));
                }
            }
        }
    }

    #[test]
    fn test_string_concatenation_not_numeric() {
        let code = "fun greet(name) { \"Hello \" + name }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    // name should NOT be considered numeric in string concatenation
                    assert!(!is_param_used_numerically("name", body));
                    assert!(!is_param_used_as_function("name", body));
                }
            }
        }
    }

    #[test]
    fn test_string_literal_helper() {
        let code = "fun test() { \"hello\" }";
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        
        if let ExprKind::Block(exprs) = &ast.kind {
            for expr in exprs {
                if let ExprKind::Function { body, .. } = &expr.kind {
                    assert!(is_string_literal(body));
                }
            }
        }
    }
}