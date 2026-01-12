//! Return type analysis helpers for the transpiler
//!
//! This module provides functions to analyze function bodies and determine
//! appropriate return types. It handles:
//! - String literal detection
//! - Boolean return type inference
//! - Vec/array return type inference
//! - Object literal return type inference
//! - Parameter-based return type inference
//!
//! These helpers enable automatic return type annotation when not explicitly provided.

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Param, Type, UnaryOp};
use anyhow::Result;
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};

/// Check if function body returns a string literal (for &'static str return type)
pub fn returns_string_literal(body: &Expr) -> bool {
    returns_string_literal_with_vars(body, &HashSet::new())
}

/// Internal helper that tracks string variables through nested let bindings
/// ISSUE-114 FIX: Handle patterns like `let s = "hello"; let n = 42; s`
pub fn returns_string_literal_with_vars(body: &Expr, string_vars: &HashSet<String>) -> bool {
    match &body.kind {
        ExprKind::Literal(Literal::String(_)) => true,
        // ISSUE-114: Check if identifier is a known string variable
        ExprKind::Identifier(name) => string_vars.contains(name),
        ExprKind::Block(exprs) if !exprs.is_empty() => {
            // Track string vars through block expressions
            let mut vars = string_vars.clone();
            for expr in &exprs[..exprs.len().saturating_sub(1)] {
                collect_string_vars(expr, &mut vars);
            }
            if let Some(last_expr) = exprs.last() {
                returns_string_literal_with_vars(last_expr, &vars)
            } else {
                false
            }
        }
        // Let expression - track string bindings and check body
        ExprKind::Let {
            name,
            value,
            body: let_body,
            is_mutable,
            ..
        } => {
            let mut vars = string_vars.clone();
            // Track immutable string literal bindings
            if !is_mutable && matches!(&value.kind, ExprKind::Literal(Literal::String(_))) {
                vars.insert(name.clone());
            }
            returns_string_literal_with_vars(let_body, &vars)
        }
        // If expression - check if both branches return string literals
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_is_literal = returns_string_literal_with_vars(then_branch, string_vars);
            let else_is_literal = else_branch
                .as_ref()
                .is_some_and(|e| returns_string_literal_with_vars(e, string_vars));
            then_is_literal && else_is_literal
        }
        // Return statement with string literal
        ExprKind::Return { value: Some(val) } => returns_string_literal_with_vars(val, string_vars),
        _ => false,
    }
}

/// Helper: Collect string variable bindings from an expression
pub fn collect_string_vars(expr: &Expr, string_vars: &mut HashSet<String>) {
    if let ExprKind::Let {
        name,
        value,
        is_mutable,
        ..
    } = &expr.kind
    {
        if !is_mutable && matches!(&value.kind, ExprKind::Literal(Literal::String(_))) {
            string_vars.insert(name.clone());
        }
    }
}

/// Check if function body returns a boolean value (ISSUE-113)
/// Detects: true, false, comparison expressions, return statements with booleans
pub fn returns_boolean(body: &Expr) -> bool {
    match &body.kind {
        // Direct boolean literals
        ExprKind::Literal(Literal::Bool(_)) => true,

        // Comparison operators return bool
        ExprKind::Binary { op, .. } => matches!(
            op,
            BinaryOp::Less
                | BinaryOp::Greater
                | BinaryOp::LessEqual
                | BinaryOp::GreaterEqual
                | BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::And
                | BinaryOp::Or
        ),

        // Return statement with boolean value
        ExprKind::Return { value: Some(val) } => returns_boolean(val),

        // Block - check last expression AND all return statements
        ExprKind::Block(exprs) => {
            // Check if any expression is a return with boolean
            let has_boolean_return = exprs.iter().any(|e| {
                matches!(&e.kind, ExprKind::Return { value: Some(val) }
                    if matches!(&val.kind, ExprKind::Literal(Literal::Bool(_))))
            });

            // Check last expression
            let last_is_boolean = exprs.last().is_some_and(returns_boolean);

            has_boolean_return || last_is_boolean
        }

        // If expression - check both branches
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            returns_boolean(then_branch) || else_branch.as_ref().is_some_and(|e| returns_boolean(e))
        }

        // Unary not operator on boolean
        ExprKind::Unary {
            op: UnaryOp::Not, ..
        } => true,

        _ => false,
    }
}

/// Check if function body returns a Vec/array (ISSUE-113)
/// Detects: [], `array.push()`, array literals, vec! macro
pub fn returns_vec(body: &Expr) -> bool {
    match &body.kind {
        // Array literal []
        ExprKind::List(_) => true,

        // vec! macro invocation
        ExprKind::MacroInvocation { name, .. } if name == "vec!" => true,

        // Return statement with vec
        ExprKind::Return { value: Some(val) } => returns_vec(val),

        // Block - check last expression
        ExprKind::Block(exprs) => exprs.last().is_some_and(|e| {
            // Last expression is array literal
            matches!(&e.kind, ExprKind::List(_))
            // OR recursively check
            || returns_vec(e)
        }),

        // Let expression - check body
        ExprKind::Let { body: let_body, .. } | ExprKind::LetPattern { body: let_body, .. } => {
            returns_vec(let_body)
        }

        _ => false,
    }
}

/// TRANSPILER-013: Check if expression returns an object literal (transpiled to `BTreeMap`)
/// Used to infer return type annotation for functions
pub fn returns_object_literal(body: &Expr) -> bool {
    match &body.kind {
        // Direct object literal { key: value, ... }
        ExprKind::ObjectLiteral { .. } => true,

        // Return statement with object literal
        ExprKind::Return { value: Some(val) } => returns_object_literal(val),

        // Block - check last expression
        ExprKind::Block(exprs) => exprs.last().is_some_and(returns_object_literal),

        // If expression - both branches return object literal
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            let then_is_object = returns_object_literal(then_branch);
            let else_is_object = else_branch
                .as_ref()
                .is_some_and(|e| returns_object_literal(e));
            then_is_object && else_is_object
        }

        // Let expression - body returns object literal
        ExprKind::Let { body: let_body, .. } | ExprKind::LetPattern { body: let_body, .. } => {
            returns_object_literal(let_body)
        }

        _ => false,
    }
}

/// Check if function body returns an owned String (ISSUE-114)
/// Detects: string concatenation, string variables, string mutations
/// Note: This is for owned String, not &'static str (use `returns_string_literal` for that)
pub fn returns_string(body: &Expr) -> bool {
    match &body.kind {
        // String concatenation with + operator returns owned String
        ExprKind::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => {
            // If either side is a string, result is String
            expr_is_string(left) || expr_is_string(right)
        }

        // Return statement with string
        ExprKind::Return { value: Some(val) } => returns_string(val),

        // Block - check last expression
        ExprKind::Block(exprs) => {
            if let Some(last) = exprs.last() {
                // If last expression is an identifier, check if it was bound to a mutable string
                if let ExprKind::Identifier(name) = &last.kind {
                    // Search for mutable Let binding with string value
                    for expr in exprs {
                        if let ExprKind::Let {
                            name: let_name,
                            value,
                            is_mutable,
                            ..
                        } = &expr.kind
                        {
                            if let_name == name && *is_mutable {
                                // Check if initial value is string
                                if matches!(&value.kind, ExprKind::Literal(Literal::String(_))) {
                                    return true; // Mutable string variable returned
                                }
                            }
                        }
                    }
                }
                // Otherwise recursively check
                returns_string(last)
            } else {
                false
            }
        }

        // Let expression - check if the variable is used in string operations
        ExprKind::Let {
            name,
            value,
            body: let_body,
            is_mutable,
            ..
        } => {
            let value_is_string = matches!(&value.kind, ExprKind::Literal(Literal::String(_)));

            // If mutable string variable that's returned, it's likely being mutated (String)
            if let ExprKind::Identifier(ident) = &let_body.kind {
                if ident == name && value_is_string && *is_mutable {
                    return true; // mut string variables return String
                }
            }

            // Check if the body contains string operations
            returns_string(let_body)
        }

        // If expression - check both branches
        ExprKind::If {
            then_branch,
            else_branch,
            ..
        } => returns_string(then_branch) || else_branch.as_ref().is_some_and(|e| returns_string(e)),

        _ => false,
    }
}

/// Check if expression evaluates to a string
pub fn expr_is_string(expr: &Expr) -> bool {
    matches!(
        &expr.kind,
        ExprKind::Literal(Literal::String(_))
            | ExprKind::Binary {
                op: BinaryOp::Add,
                ..
            }
            | ExprKind::StringInterpolation { .. }
    )
}

/// Check if identifier in block was assigned a string value
pub fn identifier_is_string(name: &str, block_exprs: &[Expr]) -> bool {
    // Search for let binding that creates string
    for e in block_exprs {
        if let ExprKind::Let {
            name: let_name,
            value,
            ..
        } = &e.kind
        {
            if let_name == name {
                // Check if initial value is string literal
                if matches!(&value.kind, ExprKind::Literal(Literal::String(_))) {
                    return true;
                }
            }
        }
        // Check for reassignment with string concatenation
        if let ExprKind::Assign { target, value, .. } = &e.kind {
            if let ExprKind::Identifier(ident) = &target.kind {
                if ident == name && expr_is_string(value) {
                    return true;
                }
            }
        }
    }
    false
}

/// Helper: Check if expression in a block creates or returns a Vec
pub fn expr_creates_or_returns_vec(expr: &Expr, block_exprs: &[Expr]) -> bool {
    if let ExprKind::Identifier(name) = &expr.kind {
        // Search backwards for let binding that creates array
        for e in block_exprs.iter().rev() {
            if let ExprKind::Let {
                name: let_name,
                value,
                ..
            } = &e.kind
            {
                if let_name == name && matches!(&value.kind, ExprKind::List(_)) {
                    return true;
                }
            }
        }
    }
    false
}

/// Helper: Get the actual final expression, drilling through Let/Block wrappers
/// Complexity: 3 (simple recursive pattern matching)
pub fn get_final_expression(expr: &Expr) -> Option<&Expr> {
    match &expr.kind {
        ExprKind::Block(exprs) => exprs.last().and_then(get_final_expression),
        ExprKind::Let { body, .. } | ExprKind::LetPattern { body, .. } => {
            get_final_expression(body)
        }
        _ => Some(expr),
    }
}

/// Helper: Trace variable assignments to find which vars hold parameter values
/// Complexity: 6 (recursive traversal with simple matching)
pub fn trace_param_assignments<'a>(
    expr: &Expr,
    var_to_param: &mut HashMap<String, &'a Type>,
    params: &'a [Param],
) {
    match &expr.kind {
        ExprKind::Block(exprs) => {
            for e in exprs {
                trace_param_assignments(e, var_to_param, params);
            }
        }
        ExprKind::Let {
            name, value, body, ..
        } => {
            // Check if value is a parameter (direct assignment)
            if let ExprKind::Identifier(value_name) = &value.kind {
                if let Some(param) = params.iter().find(|p| &p.name() == value_name) {
                    var_to_param.insert(name.clone(), &param.ty);
                }
            }
            // TRANSPILER-TYPE-INFER-EXPR: Check if value is an expression involving parameters
            else if let Some(inferred_type) = infer_expr_type_from_params(value, params) {
                var_to_param.insert(name.clone(), inferred_type);
            }
            trace_param_assignments(body, var_to_param, params);
        }
        _ => {}
    }
}

/// TRANSPILER-TYPE-INFER-EXPR: Infer type of expressions involving parameters
/// Recursively analyzes expressions to find parameter types
/// For Binary expressions, returns the type of the parameter operand
/// Complexity: 6 (recursive with 3 match arms)
pub fn infer_expr_type_from_params<'a>(expr: &Expr, params: &'a [Param]) -> Option<&'a Type> {
    match &expr.kind {
        // If expression is an identifier, check if it's a parameter
        ExprKind::Identifier(name) => params.iter().find(|p| &p.name() == name).map(|p| &p.ty),
        // For binary operations, recursively check operands
        // If either operand is a parameter, assume result has that type
        // (Works for numeric operations: f64 * f64 = f64, i32 + i32 = i32, etc.)
        ExprKind::Binary { left, right, .. } => infer_expr_type_from_params(left, params)
            .or_else(|| infer_expr_type_from_params(right, params)),
        _ => None,
    }
}

/// Infer return type from parameter types when body returns a parameter value
/// Returns `TokenStream` for return type if body returns a parameter value
/// Complexity: 9 (handles blocks, lets, identifiers, variable tracing)
pub fn infer_return_type_from_params<F>(
    body: &Expr,
    params: &[Param],
    transpile_type: F,
) -> Result<Option<TokenStream>>
where
    F: Fn(&Type) -> Result<TokenStream>,
{
    // Build a map of variable -> parameter type by tracing let bindings
    let mut var_to_param: HashMap<String, &Type> = HashMap::new();

    // Helper: check if identifier is a parameter
    let is_param =
        |name: &str| -> Option<&Type> { params.iter().find(|p| p.name() == name).map(|p| &p.ty) };

    // Trace variable assignments in body
    trace_param_assignments(body, &mut var_to_param, params);

    // Extract final expression from body (handle nested Let/Block structures)
    let final_expr = get_final_expression(body);

    if let Some(expr) = final_expr {
        if let ExprKind::Identifier(name) = &expr.kind {
            // Check if it's directly a parameter
            if let Some(param_type) = is_param(name) {
                let type_tokens = transpile_type(param_type)?;
                return Ok(Some(quote! { -> #type_tokens }));
            }

            // Check if it's a variable that was assigned from a parameter
            if let Some(&param_type) = var_to_param.get(name) {
                let type_tokens = transpile_type(param_type)?;
                return Ok(Some(quote! { -> #type_tokens }));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span};

    fn make_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }
    }

    fn string_lit(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn bool_lit(b: bool) -> Expr {
        make_expr(ExprKind::Literal(Literal::Bool(b)))
    }

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn ident(name: &str) -> Expr {
        make_expr(ExprKind::Identifier(name.to_string()))
    }

    fn block(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn let_expr(name: &str, value: Expr, body: Expr, is_mutable: bool) -> Expr {
        make_expr(ExprKind::Let {
            name: name.to_string(),
            value: Box::new(value),
            body: Box::new(body),
            is_mutable,
            type_annotation: None,
            else_block: None,
        })
    }

    fn return_expr(value: Expr) -> Expr {
        make_expr(ExprKind::Return {
            value: Some(Box::new(value)),
        })
    }

    fn if_expr(cond: Expr, then_b: Expr, else_b: Option<Expr>) -> Expr {
        make_expr(ExprKind::If {
            condition: Box::new(cond),
            then_branch: Box::new(then_b),
            else_branch: else_b.map(Box::new),
        })
    }

    fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        make_expr(ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn unary(op: UnaryOp, operand: Expr) -> Expr {
        make_expr(ExprKind::Unary {
            op,
            operand: Box::new(operand),
        })
    }

    fn list(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::List(exprs))
    }

    fn object_literal() -> Expr {
        make_expr(ExprKind::ObjectLiteral { fields: vec![] })
    }

    fn vec_macro() -> Expr {
        make_expr(ExprKind::MacroInvocation {
            name: "vec!".to_string(),
            args: vec![],
        })
    }

    // ==================== returns_string_literal tests ====================

    #[test]
    fn test_returns_string_literal_direct() {
        assert!(returns_string_literal(&string_lit("hello")));
    }

    #[test]
    fn test_returns_string_literal_not_int() {
        assert!(!returns_string_literal(&int_lit(42)));
    }

    #[test]
    fn test_returns_string_literal_in_block() {
        let expr = block(vec![int_lit(1), string_lit("result")]);
        assert!(returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_let_binding() {
        let expr = let_expr("s", string_lit("hello"), ident("s"), false);
        assert!(returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_mutable_not_tracked() {
        // Mutable bindings are not tracked as string literals
        let expr = let_expr("s", string_lit("hello"), ident("s"), true);
        assert!(!returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_if_both_branches() {
        let expr = if_expr(bool_lit(true), string_lit("yes"), Some(string_lit("no")));
        assert!(returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_if_one_branch_only() {
        let expr = if_expr(bool_lit(true), string_lit("yes"), Some(int_lit(42)));
        assert!(!returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_return_statement() {
        let expr = return_expr(string_lit("returned"));
        assert!(returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_nested_let() {
        // let s = "hello"; let n = 42; s
        let inner = let_expr("n", int_lit(42), ident("s"), false);
        let expr = let_expr("s", string_lit("hello"), inner, false);
        assert!(returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_string_literal_block_with_var_tracking() {
        let expr = block(vec![let_expr("s", string_lit("hello"), ident("s"), false)]);
        assert!(returns_string_literal(&expr));
    }

    // ==================== returns_boolean tests ====================

    #[test]
    fn test_returns_boolean_true() {
        assert!(returns_boolean(&bool_lit(true)));
    }

    #[test]
    fn test_returns_boolean_false() {
        assert!(returns_boolean(&bool_lit(false)));
    }

    #[test]
    fn test_returns_boolean_comparison_less() {
        let expr = binary(BinaryOp::Less, int_lit(1), int_lit(2));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_comparison_greater() {
        let expr = binary(BinaryOp::Greater, int_lit(5), int_lit(3));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_comparison_equal() {
        let expr = binary(BinaryOp::Equal, int_lit(1), int_lit(1));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_comparison_not_equal() {
        let expr = binary(BinaryOp::NotEqual, int_lit(1), int_lit(2));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_and_operator() {
        let expr = binary(BinaryOp::And, bool_lit(true), bool_lit(false));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_or_operator() {
        let expr = binary(BinaryOp::Or, bool_lit(true), bool_lit(false));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_not_add() {
        let expr = binary(BinaryOp::Add, int_lit(1), int_lit(2));
        assert!(!returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_return_statement() {
        let expr = return_expr(bool_lit(true));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_in_block() {
        let expr = block(vec![int_lit(1), bool_lit(true)]);
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_if_expression() {
        let expr = if_expr(bool_lit(true), bool_lit(true), None);
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_unary_not() {
        let expr = unary(UnaryOp::Not, bool_lit(true));
        assert!(returns_boolean(&expr));
    }

    #[test]
    fn test_returns_boolean_block_with_return() {
        let expr = block(vec![return_expr(bool_lit(false)), int_lit(42)]);
        assert!(returns_boolean(&expr));
    }

    // ==================== returns_vec tests ====================

    #[test]
    fn test_returns_vec_array_literal() {
        let expr = list(vec![int_lit(1), int_lit(2)]);
        assert!(returns_vec(&expr));
    }

    #[test]
    fn test_returns_vec_empty_array() {
        let expr = list(vec![]);
        assert!(returns_vec(&expr));
    }

    #[test]
    fn test_returns_vec_macro() {
        assert!(returns_vec(&vec_macro()));
    }

    #[test]
    fn test_returns_vec_return_statement() {
        let expr = return_expr(list(vec![int_lit(1)]));
        assert!(returns_vec(&expr));
    }

    #[test]
    fn test_returns_vec_in_block() {
        let expr = block(vec![int_lit(1), list(vec![int_lit(2)])]);
        assert!(returns_vec(&expr));
    }

    #[test]
    fn test_returns_vec_not_int() {
        assert!(!returns_vec(&int_lit(42)));
    }

    #[test]
    fn test_returns_vec_let_body() {
        let expr = let_expr("x", int_lit(1), list(vec![int_lit(2)]), false);
        assert!(returns_vec(&expr));
    }

    // ==================== returns_object_literal tests ====================

    #[test]
    fn test_returns_object_literal_direct() {
        assert!(returns_object_literal(&object_literal()));
    }

    #[test]
    fn test_returns_object_literal_return() {
        let expr = return_expr(object_literal());
        assert!(returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_object_literal_in_block() {
        let expr = block(vec![int_lit(1), object_literal()]);
        assert!(returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_object_literal_if_both_branches() {
        let expr = if_expr(bool_lit(true), object_literal(), Some(object_literal()));
        assert!(returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_object_literal_if_one_branch() {
        let expr = if_expr(bool_lit(true), object_literal(), Some(int_lit(42)));
        assert!(!returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_object_literal_let_body() {
        let expr = let_expr("x", int_lit(1), object_literal(), false);
        assert!(returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_object_literal_not_int() {
        assert!(!returns_object_literal(&int_lit(42)));
    }

    // ==================== returns_string tests ====================

    #[test]
    fn test_returns_string_concatenation() {
        let expr = binary(BinaryOp::Add, string_lit("a"), string_lit("b"));
        assert!(returns_string(&expr));
    }

    #[test]
    fn test_returns_string_return_concat() {
        let expr = return_expr(binary(BinaryOp::Add, string_lit("a"), string_lit("b")));
        assert!(returns_string(&expr));
    }

    #[test]
    fn test_returns_string_mutable_var() {
        // let mut s = "hello"; s
        let expr = let_expr("s", string_lit("hello"), ident("s"), true);
        assert!(returns_string(&expr));
    }

    #[test]
    fn test_returns_string_not_immutable() {
        // Immutable string binding returns &str not String
        let expr = let_expr("s", string_lit("hello"), ident("s"), false);
        assert!(!returns_string(&expr));
    }

    #[test]
    fn test_returns_string_in_block_with_mutable() {
        let expr = block(vec![let_expr("s", string_lit("hello"), ident("s"), true)]);
        assert!(returns_string(&expr));
    }

    #[test]
    fn test_returns_string_if_branch() {
        let concat = binary(BinaryOp::Add, string_lit("a"), string_lit("b"));
        let expr = if_expr(bool_lit(true), concat, None);
        assert!(returns_string(&expr));
    }

    // ==================== expr_is_string tests ====================

    #[test]
    fn test_expr_is_string_literal() {
        assert!(expr_is_string(&string_lit("hello")));
    }

    #[test]
    fn test_expr_is_string_concatenation() {
        let expr = binary(BinaryOp::Add, string_lit("a"), string_lit("b"));
        assert!(expr_is_string(&expr));
    }

    #[test]
    fn test_expr_is_string_not_int() {
        assert!(!expr_is_string(&int_lit(42)));
    }

    #[test]
    fn test_expr_is_string_interpolation() {
        let expr = make_expr(ExprKind::StringInterpolation { parts: vec![] });
        assert!(expr_is_string(&expr));
    }

    // ==================== identifier_is_string tests ====================

    #[test]
    fn test_identifier_is_string_from_let() {
        let exprs = vec![let_expr("s", string_lit("hello"), ident("s"), false)];
        assert!(identifier_is_string("s", &exprs));
    }

    #[test]
    fn test_identifier_is_string_not_found() {
        let exprs = vec![let_expr("x", int_lit(42), ident("x"), false)];
        assert!(!identifier_is_string("s", &exprs));
    }

    #[test]
    fn test_identifier_is_string_from_assignment() {
        let assign = make_expr(ExprKind::Assign {
            target: Box::new(ident("s")),
            value: Box::new(string_lit("hello")),
        });
        let exprs = vec![assign];
        assert!(identifier_is_string("s", &exprs));
    }

    // ==================== expr_creates_or_returns_vec tests ====================

    #[test]
    fn test_expr_creates_vec_from_let() {
        let exprs = vec![let_expr("arr", list(vec![int_lit(1)]), ident("arr"), false)];
        assert!(expr_creates_or_returns_vec(&ident("arr"), &exprs));
    }

    #[test]
    fn test_expr_creates_vec_not_found() {
        let exprs = vec![let_expr("x", int_lit(42), ident("x"), false)];
        assert!(!expr_creates_or_returns_vec(&ident("arr"), &exprs));
    }

    // ==================== get_final_expression tests ====================

    #[test]
    fn test_get_final_expression_simple() {
        let expr = int_lit(42);
        assert!(get_final_expression(&expr).is_some());
    }

    #[test]
    fn test_get_final_expression_block() {
        let expr = block(vec![int_lit(1), int_lit(42)]);
        let final_expr = get_final_expression(&expr).unwrap();
        assert!(matches!(
            &final_expr.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));
    }

    #[test]
    fn test_get_final_expression_let() {
        let expr = let_expr("x", int_lit(1), int_lit(42), false);
        let final_expr = get_final_expression(&expr).unwrap();
        assert!(matches!(
            &final_expr.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));
    }

    #[test]
    fn test_get_final_expression_nested() {
        let inner = let_expr("y", int_lit(2), int_lit(42), false);
        let expr = let_expr("x", int_lit(1), inner, false);
        let final_expr = get_final_expression(&expr).unwrap();
        assert!(matches!(
            &final_expr.kind,
            ExprKind::Literal(Literal::Integer(42, None))
        ));
    }

    #[test]
    fn test_get_final_expression_empty_block() {
        let expr = block(vec![]);
        assert!(get_final_expression(&expr).is_none());
    }

    // ==================== collect_string_vars tests ====================

    #[test]
    fn test_collect_string_vars_immutable() {
        let mut vars = HashSet::new();
        let expr = let_expr("s", string_lit("hello"), ident("s"), false);
        collect_string_vars(&expr, &mut vars);
        assert!(vars.contains("s"));
    }

    #[test]
    fn test_collect_string_vars_mutable_not_collected() {
        let mut vars = HashSet::new();
        let expr = let_expr("s", string_lit("hello"), ident("s"), true);
        collect_string_vars(&expr, &mut vars);
        assert!(!vars.contains("s"));
    }

    #[test]
    fn test_collect_string_vars_int_not_collected() {
        let mut vars = HashSet::new();
        let expr = let_expr("n", int_lit(42), ident("n"), false);
        collect_string_vars(&expr, &mut vars);
        assert!(!vars.contains("n"));
    }

    // ==================== infer_expr_type_from_params tests ====================

    #[test]
    fn test_infer_expr_type_no_params() {
        let expr = int_lit(42);
        let params: Vec<Param> = vec![];
        assert!(infer_expr_type_from_params(&expr, &params).is_none());
    }

    // ==================== edge case tests ====================

    #[test]
    fn test_returns_string_literal_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_string_literal(&expr));
    }

    #[test]
    fn test_returns_boolean_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_boolean(&expr));
    }

    #[test]
    fn test_returns_vec_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_vec(&expr));
    }

    #[test]
    fn test_returns_object_literal_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_object_literal(&expr));
    }

    #[test]
    fn test_returns_string_empty_block() {
        let expr = block(vec![]);
        assert!(!returns_string(&expr));
    }
}
