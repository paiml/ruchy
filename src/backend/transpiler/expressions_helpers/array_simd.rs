//! Array SIMD Lowering (Ruchy 5.0 Alpha.2)
//!
//! Per trueno-first-class-integration.md Section 2.1: when both operands of a
//! binary expression resolve to f32[]/f64[], emit trueno SIMD vector calls
//! instead of scalar loops.
//!
//! # Type-Directed Lowering Table
//!
//! | Ruchy Source | Inferred Types | Transpiled Rust |
//! |-------------|----------------|-----------------|
//! | `a + b` | `f32[], f32[]` | `trueno_bridge::add_f32(&a, &b)?` |
//! | `a - b` | `f32[], f32[]` | `trueno_bridge::sub_f32(&a, &b)?` |
//! | `a * b` | `f32[], f32[]` | `trueno_bridge::mul_f32(&a, &b)?` |
//! | `a / b` | `f32[], f32[]` | `trueno_bridge::div_f32(&a, &b)?` |

use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal};
use proc_macro2::TokenStream;
use quote::quote;

/// Check if an expression is a list/array literal.
pub fn is_list_expr(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::List(_))
}

/// Check if an expression is a known f32 array (all elements are float literals).
pub fn is_f32_array_expr(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::List(elements) => {
            !elements.is_empty()
                && elements
                    .iter()
                    .all(|e| matches!(&e.kind, ExprKind::Literal(Literal::Float(..))))
        }
        _ => false,
    }
}

/// Attempt to lower a binary operation on arrays to trueno SIMD calls.
///
/// Returns `Some(TokenStream)` if lowering was possible, `None` if the
/// expression should fall through to the default scalar transpilation.
pub fn try_lower_array_binary(
    left: &TokenStream,
    op: BinaryOp,
    right: &TokenStream,
    left_expr: &Expr,
    right_expr: &Expr,
) -> Option<TokenStream> {
    if !is_arithmetic_op(op) {
        return None;
    }
    // Only lower when both operands are list/array literals
    if !is_list_expr(left_expr) || !is_list_expr(right_expr) {
        return None;
    }

    let bridge_fn = match op {
        BinaryOp::Add => quote! { ruchy::stdlib::trueno_bridge::add_f32 },
        BinaryOp::Subtract => quote! { ruchy::stdlib::trueno_bridge::sub_f32 },
        BinaryOp::Multiply => quote! { ruchy::stdlib::trueno_bridge::mul_f32 },
        BinaryOp::Divide => quote! { ruchy::stdlib::trueno_bridge::div_f32 },
        _ => return None,
    };

    Some(quote! {
        #bridge_fn(&#left, &#right).expect("SIMD vector operation failed")
    })
}

fn is_arithmetic_op(op: BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

    fn float_lit(v: f64) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Float(v)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    fn make_list(elements: Vec<Expr>) -> Expr {
        Expr {
            kind: ExprKind::List(elements),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    fn make_ident(name: &str) -> Expr {
        Expr {
            kind: ExprKind::Identifier(name.to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            contracts: Vec::new(),
        }
    }

    #[test]
    fn test_is_f32_array_expr_floats() {
        let arr = make_list(vec![float_lit(1.0), float_lit(2.0)]);
        assert!(is_f32_array_expr(&arr));
    }

    #[test]
    fn test_is_f32_array_expr_empty() {
        let arr = make_list(vec![]);
        assert!(!is_f32_array_expr(&arr));
    }

    #[test]
    fn test_is_f32_array_expr_non_array() {
        let ident = make_ident("x");
        assert!(!is_f32_array_expr(&ident));
    }

    #[test]
    fn test_is_list_expr() {
        let arr = make_list(vec![float_lit(1.0)]);
        assert!(is_list_expr(&arr));
        let ident = make_ident("x");
        assert!(!is_list_expr(&ident));
    }

    #[test]
    fn test_try_lower_non_arithmetic_returns_none() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_ident("a");
        let re = make_ident("b");
        assert!(try_lower_array_binary(&left, BinaryOp::Equal, &right, &le, &re).is_none());
    }

    #[test]
    fn test_try_lower_non_array_returns_none() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_ident("a");
        let re = make_ident("b");
        assert!(try_lower_array_binary(&left, BinaryOp::Add, &right, &le, &re).is_none());
    }

    #[test]
    fn test_try_lower_array_add() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_list(vec![float_lit(1.0)]);
        let re = make_list(vec![float_lit(2.0)]);
        let result = try_lower_array_binary(&left, BinaryOp::Add, &right, &le, &re);
        assert!(result.is_some());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("add_f32"), "Expected add_f32, got: {tokens}");
    }

    #[test]
    fn test_try_lower_array_sub() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_list(vec![float_lit(1.0)]);
        let re = make_list(vec![float_lit(2.0)]);
        let result = try_lower_array_binary(&left, BinaryOp::Subtract, &right, &le, &re);
        assert!(result.is_some());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("sub_f32"), "Expected sub_f32, got: {tokens}");
    }

    #[test]
    fn test_try_lower_array_mul() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_list(vec![float_lit(1.0)]);
        let re = make_list(vec![float_lit(2.0)]);
        let result = try_lower_array_binary(&left, BinaryOp::Multiply, &right, &le, &re);
        assert!(result.is_some());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("mul_f32"), "Expected mul_f32, got: {tokens}");
    }

    #[test]
    fn test_try_lower_array_div() {
        let left = quote! { a };
        let right = quote! { b };
        let le = make_list(vec![float_lit(1.0)]);
        let re = make_list(vec![float_lit(2.0)]);
        let result = try_lower_array_binary(&left, BinaryOp::Divide, &right, &le, &re);
        assert!(result.is_some());
        let tokens = result.unwrap().to_string();
        assert!(tokens.contains("div_f32"), "Expected div_f32, got: {tokens}");
    }
}
