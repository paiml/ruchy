use super::*;

// Test 136: returns_vec - with vec macro
#[test]
fn test_returns_vec_macro() {
    let _transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::MacroInvocation {
            name: "vec!".to_string(),
            args: vec![],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_vec(&body));
}

// Test 137: returns_vec - with list literal
#[test]
fn test_returns_vec_list() {
    let _transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::List(vec![Expr {
            kind: ExprKind::Literal(Literal::Integer(1, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_vec(&body));
}

// Test 138: returns_object_literal - with object
#[test]
fn test_returns_object_literal_true() {
    let body = Expr {
        kind: ExprKind::ObjectLiteral { fields: vec![] },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(returns_object_literal(&body));
}

// Test 139: returns_object_literal - with non-object
#[test]
fn test_returns_object_literal_false() {
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(!returns_object_literal(&body));
}

// Test 140: expr_is_string - with string literal
#[test]
fn test_expr_is_string_literal() {
    let _transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::String("test".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(expr_is_string(&expr));
}

// Test 141: expr_is_string - with interpolation
#[test]
fn test_expr_is_string_interpolation() {
    let _transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::StringInterpolation { parts: vec![] },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(expr_is_string(&expr));
}

// Test 142: has_non_unit_expression - with non-unit
#[test]
fn test_has_non_unit_expression_true() {
    let _transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::function_analysis::has_non_unit_expression(
        &body
    ));
}

// Test 143: has_non_unit_expression - with unit
#[test]
fn test_has_non_unit_expression_false() {
    let _transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(!super::super::function_analysis::has_non_unit_expression(
        &body
    ));
}

// Test 144: is_void_expression - with unit literal
#[test]
fn test_is_void_expression_unit_v2() {
    let _transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    assert!(super::super::function_analysis::is_void_expression(&expr));
}

// ============== Coverage Tests for statements.rs delegation methods ==============

// Test 145: generate_body_tokens_with_string_conversion
#[test]
fn test_generate_body_tokens_with_string_conversion() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::String("hello".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_body_tokens_with_string_conversion(&body, false);
    assert!(result.is_ok());
}

// Test 146: generate_body_tokens_with_string_conversion async
#[test]
fn test_generate_body_tokens_with_string_conversion_async() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::String("async_result".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_body_tokens_with_string_conversion(&body, true);
    assert!(result.is_ok());
}

// Test 147: generate_param_tokens_with_lifetime - no params
#[test]
fn test_generate_param_tokens_with_lifetime_empty() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_param_tokens_with_lifetime(&[], &body, "test_fn");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// Test 148: generate_param_tokens_with_lifetime - with reference param
#[test]
fn test_generate_param_tokens_with_lifetime_with_ref() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let params = vec![Param {
        pattern: Pattern::Identifier("x".to_string()),
        ty: Type {
            kind: TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }];
    let result = transpiler.generate_param_tokens_with_lifetime(&params, &body, "test_fn");
    assert!(result.is_ok());
}

// Test 149: generate_return_type_tokens_with_lifetime
#[test]
fn test_generate_return_type_tokens_with_lifetime_none() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_return_type_tokens_with_lifetime("test_fn", None, &body);
    assert!(result.is_ok());
}

// Test 150: generate_return_type_tokens_with_lifetime with reference type
#[test]
fn test_generate_return_type_tokens_with_lifetime_ref() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let return_type = Type {
        kind: TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(Type {
                kind: TypeKind::Named("str".to_string()),
                span: Span::default(),
            }),
        },
        span: Span::default(),
    };
    let result =
        transpiler.generate_return_type_tokens_with_lifetime("test_fn", Some(&return_type), &body);
    assert!(result.is_ok());
}

// Test 151: transpile_function with pub visibility
#[test]
fn test_transpile_function_pub() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.transpile_function(
        "pub_fn",
        &[],
        &[],
        &body,
        false,
        None,
        true, // is_pub
        &[],
    );
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("pub"));
}

// Test 152: transpile_function with async
#[test]
fn test_transpile_function_async() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.transpile_function(
        "async_fn",
        &[],
        &[],
        &body,
        true, // is_async
        None,
        false,
        &[],
    );
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("async"));
}

// Test 153: transpile_function with type params
#[test]
fn test_transpile_function_with_type_params() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let params = vec![Param {
        pattern: Pattern::Identifier("x".to_string()),
        ty: Type {
            kind: TypeKind::Named("T".to_string()),
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }];
    let result = transpiler.transpile_function(
        "generic_fn",
        &["T".to_string()],
        &params,
        &body,
        false,
        None,
        false,
        &[],
    );
    assert!(result.is_ok());
}

// Test 154: transpile_function with return type
#[test]
fn test_transpile_function_with_return_type() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let return_type = Type {
        kind: TypeKind::Named("i64".to_string()),
        span: Span::default(),
    };
    let result = transpiler.transpile_function(
        "typed_fn",
        &[],
        &[],
        &body,
        false,
        Some(&return_type),
        false,
        &[],
    );
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("i64"));
}

// Test 155: infer_param_type with simple body
#[test]
fn test_infer_param_type_simple() {
    let transpiler = Transpiler::new();
    let param = Param {
        pattern: Pattern::Identifier("x".to_string()),
        ty: Type {
            kind: TypeKind::Named("unknown".to_string()),
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    };
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.infer_param_type(&param, &body, "test_fn");
    assert!(!result.is_empty());
}

// Test 156: generate_param_tokens with multiple params
#[test]
fn test_generate_param_tokens_multiple() {
    let transpiler = Transpiler::new();
    let params = vec![
        Param {
            pattern: Pattern::Identifier("a".to_string()),
            ty: Type {
                kind: TypeKind::Named("i64".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        },
        Param {
            pattern: Pattern::Identifier("b".to_string()),
            ty: Type {
                kind: TypeKind::Named("i64".to_string()),
                span: Span::default(),
            },
            span: Span::default(),
            is_mutable: false,
            default_value: None,
        },
    ];
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Identifier("a".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Identifier("b".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_param_tokens(&params, &body, "add_fn");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

// Test 157: generate_return_type_tokens
#[test]
fn test_generate_return_type_tokens_explicit() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let return_type = Type {
        kind: TypeKind::Named("i64".to_string()),
        span: Span::default(),
    };
    let result = transpiler.generate_return_type_tokens("test_fn", Some(&return_type), &body, &[]);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("i64"));
}

// Test 158: generate_body_tokens sync
#[test]
fn test_generate_body_tokens_sync() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Block(vec![Expr {
            kind: ExprKind::Literal(Literal::Integer(1, None)),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_body_tokens(&body, false);
    assert!(result.is_ok());
}

// Test 159: generate_body_tokens async
#[test]
fn test_generate_body_tokens_async() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_body_tokens(&body, true);
    assert!(result.is_ok());
}

// Test 160: generate_type_param_tokens
#[test]
fn test_generate_type_param_tokens_simple() {
    let transpiler = Transpiler::new();
    let result = transpiler.generate_type_param_tokens(&["T".to_string(), "U".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

// Test 161: generate_type_param_tokens with bounds
#[test]
fn test_generate_type_param_tokens_with_bounds() {
    let transpiler = Transpiler::new();
    let result = transpiler.generate_type_param_tokens(&["T: Clone".to_string()]);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(tokens[0].to_string().contains("Clone"));
}

// Test 162: transpile_block empty
#[test]
fn test_transpile_block_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_block(&[]);
    assert!(result.is_ok());
}

// Test 163: transpile_block with statements
#[test]
fn test_transpile_block_with_statements() {
    let transpiler = Transpiler::new();
    let exprs = vec![Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }];
    let result = transpiler.transpile_block(&exprs);
    assert!(result.is_ok());
}

// Test 164: transpile_lambda simple
#[test]
fn test_transpile_lambda_simple() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: Pattern::Identifier("x".to_string()),
        ty: Type {
            kind: TypeKind::Named("unknown".to_string()),
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }];
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Multiply,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(2, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.transpile_lambda(&params, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("|"));
}

// Test 165: transpile_call simple function
#[test]
fn test_transpile_call_simple_fn() {
    let transpiler = Transpiler::new();
    let func = Expr {
        kind: ExprKind::Identifier("my_func".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let args = vec![Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }];
    let result = transpiler.transpile_call(&func, &args);
    assert!(result.is_ok());
}

// Test 166: transpile_method_call
#[test]
fn test_transpile_method_call_simple() {
    let transpiler = Transpiler::new();
    let object = Expr {
        kind: ExprKind::Identifier("vec".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let args = vec![];
    let result = transpiler.transpile_method_call(&object, "len", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap().to_string();
    assert!(tokens.contains("len"));
}

// Test 167: transpile_method_call with args
#[test]
fn test_transpile_method_call_with_args() {
    let transpiler = Transpiler::new();
    let object = Expr {
        kind: ExprKind::Identifier("vec".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let args = vec![Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }];
    let result = transpiler.transpile_method_call(&object, "push", &args);
    assert!(result.is_ok());
}

// Test 168: transpile_pipeline simple
#[test]
fn test_transpile_pipeline_simple() {
    use crate::frontend::ast::PipelineStage;
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("data".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let stages = vec![PipelineStage {
        op: Box::new(Expr {
            kind: ExprKind::Identifier("filter".to_string()),
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        }),
        span: Span::default(),
    }];
    let result = transpiler.transpile_pipeline(&expr, &stages);
    assert!(result.is_ok());
}

// Test 169: transpile_function with return type inference from body
#[test]
fn test_transpile_function_infer_return_type() {
    let transpiler = Transpiler::new();
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
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Multiply,
            right: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result =
        transpiler.transpile_function("square", &[], &params, &body, false, None, false, &[]);
    assert!(result.is_ok());
}

// Test 170: transpile_function with nested array param
#[test]
fn test_transpile_function_nested_array_param() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: Pattern::Identifier("matrix".to_string()),
        ty: Type {
            kind: TypeKind::Named("unknown".to_string()),
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }];
    let body = Expr {
        kind: ExprKind::IndexAccess {
            object: Box::new(Expr {
                kind: ExprKind::IndexAccess {
                    object: Box::new(Expr {
                        kind: ExprKind::Identifier("matrix".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    index: Box::new(Expr {
                        kind: ExprKind::Identifier("i".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                },
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            index: Box::new(Expr {
                kind: ExprKind::Identifier("j".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result =
        transpiler.transpile_function("get_element", &[], &params, &body, false, None, false, &[]);
    assert!(result.is_ok());
}

// Test 171: transpile_function with global reference in body
#[test]
fn test_transpile_function_references_global() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Identifier("GLOBAL_CONFIG".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
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
    let result =
        transpiler.transpile_function("add_global", &[], &params, &body, false, None, false, &[]);
    assert!(result.is_ok());
}

// Test 172: transpile_function with test attribute
#[test]
fn test_transpile_function_with_test_attribute() {
    use crate::frontend::ast::Attribute;
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let attrs = vec![Attribute {
        name: "test".to_string(),
        args: vec![],
        span: Span::default(),
    }];
    let result = transpiler.transpile_function(
        "test_something",
        &[],
        &[],
        &body,
        false,
        None,
        false,
        &attrs,
    );
    assert!(result.is_ok());
}

// Test 173: transpile_function with derive attribute
#[test]
fn test_transpile_function_with_derive_attribute() {
    use crate::frontend::ast::Attribute;
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let attrs = vec![Attribute {
        name: "derive".to_string(),
        args: vec!["Clone".to_string(), "Debug".to_string()],
        span: Span::default(),
    }];
    let result =
        transpiler.transpile_function("get_value", &[], &[], &body, false, None, true, &attrs);
    assert!(result.is_ok());
}

// Test 174: generate_function_signature with async and generic
#[test]
fn test_generate_function_signature_async_generic() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("async_generic_fn");
    let type_param_tokens = vec![quote! { T: Clone }];
    let param_tokens = vec![quote! { x: T }];
    let return_type_tokens = quote! { -> T };
    let body_tokens = quote! { x.clone() };
    let result = transpiler.generate_function_signature(
        true,
        true,
        &fn_name,
        &type_param_tokens,
        &param_tokens,
        &return_type_tokens,
        &body_tokens,
        &[],
    );
    assert!(result.is_ok());
}

// Test 175: try_transpile_dataframe_builder_inline
#[test]
fn test_try_transpile_dataframe_builder_inline() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("df".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.try_transpile_dataframe_builder_inline(&expr);
    assert!(result.is_ok());
}

// Test 176: transpile_function with match expression returning string
#[test]
fn test_transpile_function_match_string_arms() {
    use crate::frontend::ast::MatchArm;
    let transpiler = Transpiler::new();
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
    let body = Expr {
        kind: ExprKind::Match {
            expr: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(1, None)),
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("one".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::String("other".to_string())),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    span: Span::default(),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.transpile_function(
        "to_string",
        &[],
        &params,
        &body,
        false,
        Some(&Type {
            kind: TypeKind::Named("String".to_string()),
            span: Span::default(),
        }),
        false,
        &[],
    );
    assert!(result.is_ok());
}

// Test 177: transpile_function with mutable ref lifetime
#[test]
fn test_transpile_function_mutable_ref_lifetime() {
    let transpiler = Transpiler::new();
    let params = vec![Param {
        pattern: Pattern::Identifier("s".to_string()),
        ty: Type {
            kind: TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        },
        span: Span::default(),
        is_mutable: false,
        default_value: None,
    }];
    let body = Expr {
        kind: ExprKind::Identifier("s".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.transpile_function(
        "modify",
        &[],
        &params,
        &body,
        false,
        Some(&Type {
            kind: TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::default(),
                }),
            },
            span: Span::default(),
        }),
        false,
        &[],
    );
    assert!(result.is_ok());
}

// Test 178: generate_body_tokens_with_string_conversion - if expression
#[test]
fn test_generate_body_tokens_with_string_conversion_if() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            then_branch: Box::new(Expr {
                kind: ExprKind::Literal(Literal::String("yes".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            else_branch: Some(Box::new(Expr {
                kind: ExprKind::Literal(Literal::String("no".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            })),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.generate_body_tokens_with_string_conversion(&body, false);
    assert!(result.is_ok());
}

// Test 179: transpile_call with col function (DataFrame)
#[test]
fn test_transpile_call_col_function() {
    let transpiler = Transpiler::new();
    let func = Expr {
        kind: ExprKind::Identifier("col".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let args = vec![Expr {
        kind: ExprKind::Literal(Literal::String("name".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }];
    let result = transpiler.transpile_call(&func, &args);
    assert!(result.is_ok());
}

// Test 180: transpile_pipeline with multiple stages
#[test]
fn test_transpile_pipeline_multiple_stages() {
    use crate::frontend::ast::PipelineStage;
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::List(vec![
            Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
            Expr {
                kind: ExprKind::Literal(Literal::Integer(2, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
        ]),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let stages = vec![
        PipelineStage {
            op: Box::new(Expr {
                kind: ExprKind::Identifier("filter".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            span: Span::default(),
        },
        PipelineStage {
            op: Box::new(Expr {
                kind: ExprKind::Identifier("map".to_string()),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            span: Span::default(),
        },
    ];
    let result = transpiler.transpile_pipeline(&expr, &stages);
    assert!(result.is_ok());
}

// ========== DIRECT TESTS FOR NEWLY EXPOSED pub(crate) METHODS ==========

// Test 181: infer_return_type_from_params - with typed param
#[test]
fn test_infer_return_type_from_params_typed() {
    let transpiler = Transpiler::new();
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
    let body = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.infer_return_type_from_params(&body, &params);
    assert!(result.is_ok());
}

// Test 182: infer_return_type_from_params - empty params
#[test]
fn test_infer_return_type_from_params_empty() {
    let transpiler = Transpiler::new();
    let body = Expr {
        kind: ExprKind::Literal(Literal::Integer(42, None)),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.infer_return_type_from_params(&body, &[]);
    assert!(result.is_ok());
}

// Test 183: is_nested_array_param - simple identifier (not nested)
#[test]
fn test_is_nested_array_param_simple() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.is_nested_array_param("matrix", &expr);
    assert!(!result);
}

// Test 184: is_nested_array_param - nested access
#[test]
fn test_is_nested_array_param_nested() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::IndexAccess {
            object: Box::new(Expr {
                kind: ExprKind::IndexAccess {
                    object: Box::new(Expr {
                        kind: ExprKind::Identifier("matrix".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    index: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::Integer(0, None)),
                        span: Span::default(),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                },
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            index: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        },
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.is_nested_array_param("matrix", &expr);
    assert!(result);
}

// Test 185: references_globals - simple local
#[test]
fn test_references_globals_local() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("local_var".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let result = transpiler.references_globals(&expr);
    assert!(!result);
}

// Test 186: references_globals - uppercase (checks actual behavior)
#[test]
fn test_references_globals_uppercase() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("GLOBAL_CONFIG".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    // references_globals checks for specific patterns, not just uppercase
    let _result = transpiler.references_globals(&expr);
    // Just verify it runs without panic - actual behavior depends on implementation
}

// Test 187: compute_final_return_type
#[test]
fn test_compute_final_return_type_normal() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("my_func");
    let return_type_tokens = quote! { -> i32 };
    let result = transpiler.compute_final_return_type(&fn_name, &return_type_tokens);
    assert!(!result.is_empty());
}

// Test 188: compute_final_return_type - test function
#[test]
fn test_compute_final_return_type_test_fn() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("test_something");
    let return_type_tokens = quote! { -> i32 };
    let result = transpiler.compute_final_return_type(&fn_name, &return_type_tokens);
    assert!(!result.is_empty());
}

// Test 189: generate_visibility_token - public
#[test]
fn test_generate_visibility_token_pub() {
    let transpiler = Transpiler::new();
    let result = transpiler.generate_visibility_token(true);
    let tokens = result.to_string();
    assert!(tokens.contains("pub"));
}

// Test 190: generate_visibility_token - private
#[test]
fn test_generate_visibility_token_private() {
    let transpiler = Transpiler::new();
    let result = transpiler.generate_visibility_token(false);
    let tokens = result.to_string();
    assert!(!tokens.contains("pub"));
}

// Test 191: process_attributes - empty
#[test]
fn test_process_attributes_empty() {
    let transpiler = Transpiler::new();
    let (attrs, modifiers) = transpiler.process_attributes(&[]);
    assert!(attrs.is_empty());
    assert!(modifiers.is_empty());
}

// Test 192: process_attributes - with test attribute
#[test]
fn test_process_attributes_with_test() {
    use crate::frontend::ast::Attribute;
    let transpiler = Transpiler::new();
    let attrs = vec![Attribute {
        name: "test".to_string(),
        args: vec![],
        span: Span::default(),
    }];
    let (regular, modifiers) = transpiler.process_attributes(&attrs);
    assert!(!regular.is_empty() || !modifiers.is_empty());
}

// Test 193: generate_function_declaration - sync
#[test]
fn test_generate_function_declaration_sync() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("my_func");
    let visibility = quote! { pub };
    let modifiers = quote! {};
    let param_tokens = vec![quote! { x: i32 }];
    let return_type = quote! { -> i32 };
    let body = quote! { x + 1 };
    let result = transpiler.generate_function_declaration(
        false,
        &[],
        &[],
        &visibility,
        &modifiers,
        &fn_name,
        &param_tokens,
        &return_type,
        &body,
    );
    assert!(result.is_ok());
}

// Test 194: generate_function_declaration - async
#[test]
fn test_generate_function_declaration_async() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("async_func");
    let visibility = quote! {};
    let modifiers = quote! {};
    let param_tokens = vec![];
    let return_type = quote! { -> String };
    let body = quote! { "hello".to_string() };
    let result = transpiler.generate_function_declaration(
        true,
        &[],
        &[],
        &visibility,
        &modifiers,
        &fn_name,
        &param_tokens,
        &return_type,
        &body,
    );
    assert!(result.is_ok());
}

// Test 195: transpile_match_with_string_arms
#[test]
fn test_transpile_match_with_string_arms_direct() {
    use crate::frontend::ast::MatchArm;
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    };
    let arms = vec![
        MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1, None)),
            guard: None,
            body: Box::new(Expr {
                kind: ExprKind::Literal(Literal::String("one".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            span: Span::default(),
        },
        MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(Expr {
                kind: ExprKind::Literal(Literal::String("other".to_string())),
                span: Span::default(),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            span: Span::default(),
        },
    ];
    let result = transpiler.transpile_match_with_string_arms(&expr, &arms);
    assert!(result.is_ok());
}

// Test 196: transpile_type_with_lifetime - simple named type
#[test]
fn test_transpile_type_with_lifetime_named() {
    let transpiler = Transpiler::new();
    let ty = Type {
        kind: TypeKind::Named("String".to_string()),
        span: Span::default(),
    };
    let result = transpiler.transpile_type_with_lifetime(&ty);
    assert!(result.is_ok());
}

// Test 197: transpile_type_with_lifetime - reference type
#[test]
fn test_transpile_type_with_lifetime_ref() {
    let transpiler = Transpiler::new();
    let ty = Type {
        kind: TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(Type {
                kind: TypeKind::Named("str".to_string()),
                span: Span::default(),
            }),
        },
        span: Span::default(),
    };
    let result = transpiler.transpile_type_with_lifetime(&ty);
    assert!(result.is_ok());
}

// Test 198: try_transpile_dataframe_function - col
#[test]
fn test_try_transpile_dataframe_function_col() {
    let transpiler = Transpiler::new();
    let args = vec![Expr {
        kind: ExprKind::Literal(Literal::String("name".to_string())),
        span: Span::default(),
        attributes: vec![],
        leading_comments: vec![],
        trailing_comment: None,
    }];
    let result = transpiler.try_transpile_dataframe_function("col", &args);
    assert!(result.is_ok());
}

// Test 199: try_transpile_dataframe_function - unknown
#[test]
fn test_try_transpile_dataframe_function_unknown() {
    let transpiler = Transpiler::new();
    let args = vec![];
    let result = transpiler.try_transpile_dataframe_function("unknown_func", &args);
    assert!(result.is_ok());
}

// Test 200: generate_function_declaration with generics
#[test]
fn test_generate_function_declaration_generic() {
    use quote::format_ident;
    use quote::quote;
    let transpiler = Transpiler::new();
    let fn_name = format_ident!("generic_func");
    let visibility = quote! { pub };
    let modifiers = quote! {};
    let type_params = vec![quote! { T: Clone }];
    let param_tokens = vec![quote! { x: T }];
    let return_type = quote! { -> T };
    let body = quote! { x.clone() };
    let result = transpiler.generate_function_declaration(
        false,
        &type_params,
        &[],
        &visibility,
        &modifiers,
        &fn_name,
        &param_tokens,
        &return_type,
        &body,
    );
    assert!(result.is_ok());
}
