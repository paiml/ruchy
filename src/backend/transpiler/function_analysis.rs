//! Function and expression analysis for transpiler
//!
//! This module provides functions to analyze properties of expressions and functions,
//! such as whether a function returns void, is numeric, or returns a closure.

use crate::frontend::ast::{Expr, ExprKind, Literal};

/// Check if function name suggests numeric operations
///
/// Returns true for function names that typically perform mathematical operations.
#[must_use]
pub fn looks_like_numeric_function(name: &str) -> bool {
    matches!(
        name,
        // Basic arithmetic
        "add" | "subtract" | "multiply" | "divide" | "sum" | "product" |
        "min" | "max" | "abs" | "sqrt" | "pow" | "mod" | "gcd" | "lcm" |
        "factorial" | "fibonacci" | "prime" | "even" | "odd" | "square" | "cube" |
        "double" | "triple" | "quadruple" |

        // Trigonometric functions
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "atan2" |
        "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh" |

        // Exponential and logarithmic functions
        "exp" | "exp2" | "ln" | "log" | "log2" | "log10" |

        // Power and root functions
        "cbrt" | "powf" | "powi" |

        // Sign and comparison functions
        "signum" | "copysign" |

        // Rounding and truncation functions
        "floor" | "ceil" | "round" | "trunc" | "fract" |

        // Range functions
        "clamp" |

        // Conversion functions
        "to_degrees" | "to_radians"
    )
}

/// Check if expression is a void/unit function call
///
/// Returns true for calls to functions that return nothing (println, panic, etc.)
#[must_use]
pub fn is_void_function_call(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Call { func, .. } => {
            if let ExprKind::Identifier(name) = &func.kind {
                matches!(
                    name.as_str(),
                    // Output functions
                    "println" | "print" | "eprintln" | "eprint" |
                    // Debug functions
                    "dbg" | "debug" | "trace" | "info" | "warn" | "error" |
                    // Control flow functions
                    "panic" | "assert" | "assert_eq" | "assert_ne" |
                    "todo" | "unimplemented" | "unreachable"
                )
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Check if an expression is void (returns unit/nothing)
///
/// Recursively analyzes expressions to determine if they produce no value.
#[must_use]
pub fn is_void_expression(expr: &Expr) -> bool {
    match &expr.kind {
        // Unit literal is void
        ExprKind::Literal(Literal::Unit) => true,
        // Void function calls
        ExprKind::Call { .. } if is_void_function_call(expr) => true,
        // Macro invocations (println!, etc.)
        ExprKind::MacroInvocation { name, .. }
            if matches!(name.as_str(), "println" | "print" | "eprintln" | "eprint") =>
        {
            true
        }
        // Assignments are void
        ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. } => true,
        // Loops are void
        ExprKind::While { .. } | ExprKind::For { .. } => true,
        // Let bindings - check the body expression
        ExprKind::Let { body, .. } => is_void_expression(body),
        // Block - check last expression
        ExprKind::Block(exprs) => exprs.last().is_none_or(is_void_expression),
        // If expression - both branches must be void
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            is_void_expression(then_branch)
                && else_branch.as_ref().is_none_or(|e| is_void_expression(e))
        }
        // Match expression - all arms must be void
        ExprKind::Match { arms, .. } => arms.iter().all(|arm| is_void_expression(&arm.body)),
        // Return without value is void
        ExprKind::Return { value } if value.is_none() => true,
        // Everything else produces a value
        _ => false,
    }
}

/// Check if expression has a non-unit value (i.e., returns something meaningful)
#[must_use]
pub fn has_non_unit_expression(body: &Expr) -> bool {
    !is_void_expression(body)
}

/// Check if function body returns a closure (Lambda expression)
#[must_use]
pub fn returns_closure(body: &Expr) -> bool {
    match &body.kind {
        ExprKind::Lambda { .. } => true,
        ExprKind::Block(exprs) if !exprs.is_empty() => exprs
            .last()
            .is_some_and(|last_expr| matches!(last_expr.kind, ExprKind::Lambda { .. })),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{MatchArm, Pattern, Span};

    // ==================== Test Helpers ====================

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn call(func_name: &str, args: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Call {
            func: Box::new(ident(func_name)),
            args,
        })
    }

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn unit_lit() -> Expr {
        make_expr(ExprKind::Literal(Literal::Unit))
    }

    fn block(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    // ==================== looks_like_numeric_function Tests ====================

    #[test]
    fn test_numeric_basic_arithmetic() {
        assert!(looks_like_numeric_function("add"));
        assert!(looks_like_numeric_function("subtract"));
        assert!(looks_like_numeric_function("multiply"));
        assert!(looks_like_numeric_function("divide"));
        assert!(looks_like_numeric_function("sum"));
        assert!(looks_like_numeric_function("product"));
    }

    #[test]
    fn test_numeric_min_max() {
        assert!(looks_like_numeric_function("min"));
        assert!(looks_like_numeric_function("max"));
        assert!(looks_like_numeric_function("abs"));
        assert!(looks_like_numeric_function("clamp"));
    }

    #[test]
    fn test_numeric_math_functions() {
        assert!(looks_like_numeric_function("sqrt"));
        assert!(looks_like_numeric_function("pow"));
        assert!(looks_like_numeric_function("mod"));
        assert!(looks_like_numeric_function("gcd"));
        assert!(looks_like_numeric_function("lcm"));
    }

    #[test]
    fn test_numeric_special_sequences() {
        assert!(looks_like_numeric_function("factorial"));
        assert!(looks_like_numeric_function("fibonacci"));
        assert!(looks_like_numeric_function("prime"));
    }

    #[test]
    fn test_numeric_parity() {
        assert!(looks_like_numeric_function("even"));
        assert!(looks_like_numeric_function("odd"));
    }

    #[test]
    fn test_numeric_powers() {
        assert!(looks_like_numeric_function("square"));
        assert!(looks_like_numeric_function("cube"));
        assert!(looks_like_numeric_function("double"));
        assert!(looks_like_numeric_function("triple"));
        assert!(looks_like_numeric_function("quadruple"));
    }

    #[test]
    fn test_numeric_trigonometric() {
        assert!(looks_like_numeric_function("sin"));
        assert!(looks_like_numeric_function("cos"));
        assert!(looks_like_numeric_function("tan"));
        assert!(looks_like_numeric_function("asin"));
        assert!(looks_like_numeric_function("acos"));
        assert!(looks_like_numeric_function("atan"));
        assert!(looks_like_numeric_function("atan2"));
    }

    #[test]
    fn test_numeric_hyperbolic() {
        assert!(looks_like_numeric_function("sinh"));
        assert!(looks_like_numeric_function("cosh"));
        assert!(looks_like_numeric_function("tanh"));
        assert!(looks_like_numeric_function("asinh"));
        assert!(looks_like_numeric_function("acosh"));
        assert!(looks_like_numeric_function("atanh"));
    }

    #[test]
    fn test_numeric_exponential() {
        assert!(looks_like_numeric_function("exp"));
        assert!(looks_like_numeric_function("exp2"));
        assert!(looks_like_numeric_function("ln"));
        assert!(looks_like_numeric_function("log"));
        assert!(looks_like_numeric_function("log2"));
        assert!(looks_like_numeric_function("log10"));
    }

    #[test]
    fn test_numeric_power_root() {
        assert!(looks_like_numeric_function("cbrt"));
        assert!(looks_like_numeric_function("powf"));
        assert!(looks_like_numeric_function("powi"));
    }

    #[test]
    fn test_numeric_sign() {
        assert!(looks_like_numeric_function("signum"));
        assert!(looks_like_numeric_function("copysign"));
    }

    #[test]
    fn test_numeric_rounding() {
        assert!(looks_like_numeric_function("floor"));
        assert!(looks_like_numeric_function("ceil"));
        assert!(looks_like_numeric_function("round"));
        assert!(looks_like_numeric_function("trunc"));
        assert!(looks_like_numeric_function("fract"));
    }

    #[test]
    fn test_numeric_conversion() {
        assert!(looks_like_numeric_function("to_degrees"));
        assert!(looks_like_numeric_function("to_radians"));
    }

    #[test]
    fn test_non_numeric_functions() {
        assert!(!looks_like_numeric_function("println"));
        assert!(!looks_like_numeric_function("concat"));
        assert!(!looks_like_numeric_function("format"));
        assert!(!looks_like_numeric_function("parse"));
        assert!(!looks_like_numeric_function("read"));
        assert!(!looks_like_numeric_function("write"));
        assert!(!looks_like_numeric_function("foo"));
        assert!(!looks_like_numeric_function("bar"));
        assert!(!looks_like_numeric_function(""));
    }

    // ==================== is_void_function_call Tests ====================

    #[test]
    fn test_void_output_functions() {
        assert!(is_void_function_call(&call("println", vec![])));
        assert!(is_void_function_call(&call("print", vec![])));
        assert!(is_void_function_call(&call("eprintln", vec![])));
        assert!(is_void_function_call(&call("eprint", vec![])));
    }

    #[test]
    fn test_void_debug_functions() {
        assert!(is_void_function_call(&call("dbg", vec![])));
        assert!(is_void_function_call(&call("debug", vec![])));
        assert!(is_void_function_call(&call("trace", vec![])));
        assert!(is_void_function_call(&call("info", vec![])));
        assert!(is_void_function_call(&call("warn", vec![])));
        assert!(is_void_function_call(&call("error", vec![])));
    }

    #[test]
    fn test_void_control_flow() {
        assert!(is_void_function_call(&call("panic", vec![])));
        assert!(is_void_function_call(&call("assert", vec![])));
        assert!(is_void_function_call(&call("assert_eq", vec![])));
        assert!(is_void_function_call(&call("assert_ne", vec![])));
        assert!(is_void_function_call(&call("todo", vec![])));
        assert!(is_void_function_call(&call("unimplemented", vec![])));
        assert!(is_void_function_call(&call("unreachable", vec![])));
    }

    #[test]
    fn test_non_void_function_calls() {
        assert!(!is_void_function_call(&call("add", vec![])));
        assert!(!is_void_function_call(&call("len", vec![])));
        assert!(!is_void_function_call(&call("read", vec![])));
        assert!(!is_void_function_call(&call("foo", vec![])));
    }

    #[test]
    fn test_void_function_call_not_call() {
        assert!(!is_void_function_call(&int_lit(42)));
        assert!(!is_void_function_call(&ident("x")));
        assert!(!is_void_function_call(&unit_lit()));
    }

    #[test]
    fn test_void_function_call_with_args() {
        let expr = call("println", vec![int_lit(42)]);
        assert!(is_void_function_call(&expr));
    }

    // ==================== is_void_expression Tests ====================

    #[test]
    fn test_void_unit_literal() {
        assert!(is_void_expression(&unit_lit()));
    }

    #[test]
    fn test_void_function_call_expr() {
        assert!(is_void_expression(&call("println", vec![])));
        assert!(is_void_expression(&call("panic", vec![])));
    }

    #[test]
    fn test_void_macro_invocation() {
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "println".to_string(),
            args: vec![],
        });
        assert!(is_void_expression(&expr));

        let expr2 = make_expr(ExprKind::MacroInvocation {
            name: "print".to_string(),
            args: vec![],
        });
        assert!(is_void_expression(&expr2));

        let expr3 = make_expr(ExprKind::MacroInvocation {
            name: "eprintln".to_string(),
            args: vec![],
        });
        assert!(is_void_expression(&expr3));

        let expr4 = make_expr(ExprKind::MacroInvocation {
            name: "eprint".to_string(),
            args: vec![],
        });
        assert!(is_void_expression(&expr4));
    }

    #[test]
    fn test_void_non_void_macro() {
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![],
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_assignment() {
        let expr = make_expr(ExprKind::Assign {
            target: Box::new(ident("x")),
            value: Box::new(int_lit(42)),
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_compound_assignment() {
        let expr = make_expr(ExprKind::CompoundAssign {
            target: Box::new(ident("x")),
            op: crate::frontend::ast::BinaryOp::Add,
            value: Box::new(int_lit(1)),
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_while_loop() {
        let expr = make_expr(ExprKind::While {
            label: None,
            condition: Box::new(ident("true")),
            body: Box::new(block(vec![])),
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_for_loop() {
        let expr = make_expr(ExprKind::For {
            label: None,
            var: "i".to_string(),
            pattern: None,
            iter: Box::new(ident("items")),
            body: Box::new(block(vec![])),
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_let_with_void_body() {
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_lit(42)),
            body: Box::new(unit_lit()),
            is_mutable: false,
            else_block: None,
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_let_with_non_void_body() {
        let expr = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_lit(42)),
            body: Box::new(int_lit(100)),
            is_mutable: false,
            else_block: None,
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_empty_block() {
        assert!(is_void_expression(&block(vec![])));
    }

    #[test]
    fn test_void_block_ending_with_void() {
        let expr = block(vec![int_lit(1), call("println", vec![])]);
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_block_ending_with_value() {
        let expr = block(vec![call("println", vec![]), int_lit(42)]);
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_if_both_branches_void() {
        let expr = make_expr(ExprKind::If {
            condition: Box::new(ident("cond")),
            then_branch: Box::new(unit_lit()),
            else_branch: Some(Box::new(unit_lit())),
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_if_then_only_void() {
        let expr = make_expr(ExprKind::If {
            condition: Box::new(ident("cond")),
            then_branch: Box::new(unit_lit()),
            else_branch: None,
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_if_then_not_void() {
        let expr = make_expr(ExprKind::If {
            condition: Box::new(ident("cond")),
            then_branch: Box::new(int_lit(42)),
            else_branch: None,
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_if_else_not_void() {
        let expr = make_expr(ExprKind::If {
            condition: Box::new(ident("cond")),
            then_branch: Box::new(unit_lit()),
            else_branch: Some(Box::new(int_lit(42))),
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_match_all_arms_void() {
        let expr = make_expr(ExprKind::Match {
            expr: Box::new(ident("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(unit_lit()),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(call("println", vec![])),
                    span: Span::default(),
                },
            ],
        });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_match_one_arm_not_void() {
        let expr = make_expr(ExprKind::Match {
            expr: Box::new(ident("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(unit_lit()),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(int_lit(42)),
                    span: Span::default(),
                },
            ],
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_void_return_no_value() {
        let expr = make_expr(ExprKind::Return { value: None });
        assert!(is_void_expression(&expr));
    }

    #[test]
    fn test_void_return_with_value() {
        let expr = make_expr(ExprKind::Return {
            value: Some(Box::new(int_lit(42))),
        });
        assert!(!is_void_expression(&expr));
    }

    #[test]
    fn test_non_void_literals() {
        assert!(!is_void_expression(&int_lit(42)));
        assert!(!is_void_expression(&make_expr(ExprKind::Literal(
            Literal::Bool(true)
        ))));
        assert!(!is_void_expression(&make_expr(ExprKind::Literal(
            Literal::String("hello".to_string())
        ))));
    }

    #[test]
    fn test_non_void_identifier() {
        assert!(!is_void_expression(&ident("x")));
    }

    #[test]
    fn test_non_void_function_call() {
        assert!(!is_void_expression(&call(
            "add",
            vec![int_lit(1), int_lit(2)]
        )));
    }

    // ==================== has_non_unit_expression Tests ====================

    #[test]
    fn test_has_non_unit_true() {
        assert!(has_non_unit_expression(&int_lit(42)));
        assert!(has_non_unit_expression(&ident("x")));
        assert!(has_non_unit_expression(&call("add", vec![])));
    }

    #[test]
    fn test_has_non_unit_false() {
        assert!(!has_non_unit_expression(&unit_lit()));
        assert!(!has_non_unit_expression(&call("println", vec![])));
        assert!(!has_non_unit_expression(&block(vec![])));
    }

    // ==================== returns_closure Tests ====================

    #[test]
    fn test_returns_closure_direct_lambda() {
        let expr = make_expr(ExprKind::Lambda {
            params: vec![],
            body: Box::new(int_lit(42)),
        });
        assert!(returns_closure(&expr));
    }

    #[test]
    fn test_returns_closure_block_with_lambda() {
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![],
            body: Box::new(int_lit(42)),
        });
        let expr = block(vec![int_lit(1), lambda]);
        assert!(returns_closure(&expr));
    }

    #[test]
    fn test_returns_closure_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_closure(&expr));
    }

    #[test]
    fn test_returns_closure_block_no_lambda() {
        let expr = block(vec![int_lit(1), int_lit(2)]);
        assert!(!returns_closure(&expr));
    }

    #[test]
    fn test_returns_closure_non_lambda() {
        assert!(!returns_closure(&int_lit(42)));
        assert!(!returns_closure(&ident("x")));
        assert!(!returns_closure(&call("foo", vec![])));
    }

    #[test]
    fn test_returns_closure_single_lambda_in_block() {
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![],
            body: Box::new(int_lit(42)),
        });
        let expr = block(vec![lambda]);
        assert!(returns_closure(&expr));
    }

    #[test]
    fn test_returns_closure_lambda_not_last() {
        let lambda = make_expr(ExprKind::Lambda {
            params: vec![],
            body: Box::new(int_lit(42)),
        });
        let expr = block(vec![lambda, int_lit(99)]);
        assert!(!returns_closure(&expr));
    }
}
