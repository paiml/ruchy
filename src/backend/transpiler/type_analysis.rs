//! Type analysis utilities for transpiler
//!
//! This module provides functions to analyze types and expressions
//! for type-related decisions during transpilation.

use crate::frontend::ast::{Expr, ExprKind, Literal, Param, Type, TypeKind};

/// Check if a type is a reference type (&T, &str, &[T], etc.)
#[must_use]
pub fn is_reference_type(ty: &Type) -> bool {
    matches!(ty.kind, TypeKind::Reference { .. })
}

/// Check if a type is String
#[must_use]
pub fn is_string_type(ty: &Type) -> bool {
    matches!(&ty.kind, TypeKind::Named(name) if name == "String")
}

/// Check if function needs a lifetime parameter
///
/// Returns true if:
/// - Function has 2+ reference parameters AND
/// - Function returns a reference type
#[must_use]
pub fn needs_lifetime_parameter(params: &[Param], return_type: Option<&Type>) -> bool {
    // Count parameters with reference types
    let ref_param_count = params.iter().filter(|p| is_reference_type(&p.ty)).count();

    // Check if return type is a reference
    let returns_reference = return_type.is_some_and(|rt| is_reference_type(rt));

    // Need lifetime if 2+ ref params and ref return
    ref_param_count >= 2 && returns_reference
}

/// Check if expression body needs `.to_string()` conversion
///
/// Used for DEFECT-012/013 to ensure proper String ownership.
#[must_use]
pub fn body_needs_string_conversion(body: &Expr) -> bool {
    match &body.kind {
        ExprKind::Literal(Literal::String(_)) => true,
        ExprKind::Identifier(_) => true, // Could be &str variable
        ExprKind::IndexAccess { .. } => true, // DEFECT-013: Vec/array indexing may return &str
        // DEFECT-016-C: Match expressions may have string literal arms
        ExprKind::Match { arms, .. } => {
            // Check if any arm has a string literal body
            arms.iter()
                .any(|arm| matches!(&arm.body.kind, ExprKind::Literal(Literal::String(_))))
        }
        ExprKind::Block(exprs) if !exprs.is_empty() => body_needs_string_conversion(
            exprs
                .last()
                .expect("exprs is non-empty due to guard condition"),
        ),
        ExprKind::Let { body, .. } => {
            // Let expressions have the return value in their body field
            body_needs_string_conversion(body)
        }
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

    fn int_lit(n: i64) -> Expr {
        make_expr(ExprKind::Literal(Literal::Integer(n, None)))
    }

    fn string_lit(s: &str) -> Expr {
        make_expr(ExprKind::Literal(Literal::String(s.to_string())))
    }

    fn block(exprs: Vec<Expr>) -> Expr {
        make_expr(ExprKind::Block(exprs))
    }

    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: Span::default(),
        }
    }

    fn make_param(name: &str, ty: Type) -> Param {
        Param {
            pattern: Pattern::Identifier(name.to_string()),
            ty,
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        }
    }

    // ==================== is_reference_type Tests ====================

    #[test]
    fn test_is_reference_type_immutable_ref() {
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        assert!(is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_mutable_ref() {
        let ty = make_type(TypeKind::Reference {
            is_mut: true,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("Vec".to_string()))),
        });
        assert!(is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_named_not_ref() {
        let ty = make_type(TypeKind::Named("String".to_string()));
        assert!(!is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_i32() {
        let ty = make_type(TypeKind::Named("i32".to_string()));
        assert!(!is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_array() {
        let ty = make_type(TypeKind::Array {
            elem_type: Box::new(make_type(TypeKind::Named("i32".to_string()))),
            size: 10,
        });
        assert!(!is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_tuple() {
        let ty = make_type(TypeKind::Tuple(vec![
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("String".to_string())),
        ]));
        assert!(!is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_optional() {
        let ty = make_type(TypeKind::Optional(Box::new(make_type(TypeKind::Named(
            "String".to_string(),
        )))));
        assert!(!is_reference_type(&ty));
    }

    #[test]
    fn test_is_reference_type_list() {
        let ty = make_type(TypeKind::List(Box::new(make_type(TypeKind::Named(
            "String".to_string(),
        )))));
        assert!(!is_reference_type(&ty));
    }

    // ==================== is_string_type Tests ====================

    #[test]
    fn test_is_string_type_string() {
        let ty = make_type(TypeKind::Named("String".to_string()));
        assert!(is_string_type(&ty));
    }

    #[test]
    fn test_is_string_type_not_string() {
        let ty = make_type(TypeKind::Named("str".to_string()));
        assert!(!is_string_type(&ty));
    }

    #[test]
    fn test_is_string_type_i32() {
        let ty = make_type(TypeKind::Named("i32".to_string()));
        assert!(!is_string_type(&ty));
    }

    #[test]
    fn test_is_string_type_reference_to_string() {
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("String".to_string()))),
        });
        assert!(!is_string_type(&ty)); // Reference is not the same as String
    }

    #[test]
    fn test_is_string_type_list_string() {
        let ty = make_type(TypeKind::List(Box::new(make_type(TypeKind::Named(
            "String".to_string(),
        )))));
        assert!(!is_string_type(&ty)); // Vec<String> is not String
    }

    #[test]
    fn test_is_string_type_option_string() {
        let ty = make_type(TypeKind::Optional(Box::new(make_type(TypeKind::Named(
            "String".to_string(),
        )))));
        assert!(!is_string_type(&ty)); // Option<String> is not String
    }

    // ==================== needs_lifetime_parameter Tests ====================

    #[test]
    fn test_needs_lifetime_no_refs() {
        let params = vec![
            make_param("a", make_type(TypeKind::Named("i32".to_string()))),
            make_param("b", make_type(TypeKind::Named("i32".to_string()))),
        ];
        let return_type = make_type(TypeKind::Named("i32".to_string()));
        assert!(!needs_lifetime_parameter(&params, Some(&return_type)));
    }

    #[test]
    fn test_needs_lifetime_one_ref_param() {
        let params = vec![
            make_param(
                "a",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param("b", make_type(TypeKind::Named("i32".to_string()))),
        ];
        let return_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        assert!(!needs_lifetime_parameter(&params, Some(&return_type)));
    }

    #[test]
    fn test_needs_lifetime_two_ref_params_ref_return() {
        let params = vec![
            make_param(
                "a",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param(
                "b",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
        ];
        let return_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        assert!(needs_lifetime_parameter(&params, Some(&return_type)));
    }

    #[test]
    fn test_needs_lifetime_two_ref_params_non_ref_return() {
        let params = vec![
            make_param(
                "a",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param(
                "b",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
        ];
        let return_type = make_type(TypeKind::Named("String".to_string()));
        assert!(!needs_lifetime_parameter(&params, Some(&return_type)));
    }

    #[test]
    fn test_needs_lifetime_two_ref_params_no_return() {
        let params = vec![
            make_param(
                "a",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param(
                "b",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
        ];
        assert!(!needs_lifetime_parameter(&params, None));
    }

    #[test]
    fn test_needs_lifetime_three_ref_params_ref_return() {
        let params = vec![
            make_param(
                "a",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param(
                "b",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
            make_param(
                "c",
                make_type(TypeKind::Reference {
                    is_mut: false,
                    lifetime: None,
                    inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
                }),
            ),
        ];
        let return_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        assert!(needs_lifetime_parameter(&params, Some(&return_type)));
    }

    #[test]
    fn test_needs_lifetime_empty_params() {
        let params: Vec<Param> = vec![];
        let return_type = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(make_type(TypeKind::Named("str".to_string()))),
        });
        assert!(!needs_lifetime_parameter(&params, Some(&return_type)));
    }

    // ==================== body_needs_string_conversion Tests ====================

    #[test]
    fn test_body_needs_string_conversion_string_literal() {
        let body = string_lit("hello");
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_identifier() {
        let body = ident("my_var");
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_index_access() {
        let body = make_expr(ExprKind::IndexAccess {
            object: Box::new(ident("arr")),
            index: Box::new(int_lit(0)),
        });
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_int_literal() {
        let body = int_lit(42);
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_bool_literal() {
        let body = make_expr(ExprKind::Literal(Literal::Bool(true)));
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_block_with_string() {
        let body = block(vec![int_lit(1), string_lit("result")]);
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_block_with_int() {
        let body = block(vec![string_lit("ignored"), int_lit(42)]);
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_empty_block() {
        let body = block(vec![]);
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_let_with_string_body() {
        let body = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(int_lit(1)),
            body: Box::new(string_lit("result")),
            is_mutable: false,
            else_block: None,
        });
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_let_with_int_body() {
        let body = make_expr(ExprKind::Let {
            name: "x".to_string(),
            type_annotation: None,
            value: Box::new(string_lit("value")),
            body: Box::new(int_lit(42)),
            is_mutable: false,
            else_block: None,
        });
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_match_with_string_arm() {
        let body = make_expr(ExprKind::Match {
            expr: Box::new(ident("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(string_lit("yes")),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(string_lit("no")),
                    span: Span::default(),
                },
            ],
        });
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_match_no_string_arm() {
        let body = make_expr(ExprKind::Match {
            expr: Box::new(ident("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(int_lit(1)),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(int_lit(2)),
                    span: Span::default(),
                },
            ],
        });
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_match_mixed_arms() {
        let body = make_expr(ExprKind::Match {
            expr: Box::new(ident("x")),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(int_lit(1)),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(string_lit("hello")),
                    span: Span::default(),
                },
            ],
        });
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_nested_block() {
        let inner_block = block(vec![string_lit("inner")]);
        let body = block(vec![int_lit(1), inner_block]);
        // The last expression in the outer block is the inner block,
        // and the last expression of inner block is a string
        assert!(body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_function_call() {
        let body = make_expr(ExprKind::Call {
            func: Box::new(ident("get_string")),
            args: vec![],
        });
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_binary_op() {
        let body = make_expr(ExprKind::Binary {
            left: Box::new(int_lit(1)),
            op: crate::frontend::ast::BinaryOp::Add,
            right: Box::new(int_lit(2)),
        });
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_if_expr() {
        let body = make_expr(ExprKind::If {
            condition: Box::new(ident("cond")),
            then_branch: Box::new(string_lit("yes")),
            else_branch: Some(Box::new(string_lit("no"))),
        });
        assert!(!body_needs_string_conversion(&body)); // If is not handled
    }

    #[test]
    fn test_body_needs_string_conversion_float_literal() {
        let body = make_expr(ExprKind::Literal(Literal::Float(3.14)));
        assert!(!body_needs_string_conversion(&body));
    }

    #[test]
    fn test_body_needs_string_conversion_unit_literal() {
        let body = make_expr(ExprKind::Literal(Literal::Unit));
        assert!(!body_needs_string_conversion(&body));
    }
}
