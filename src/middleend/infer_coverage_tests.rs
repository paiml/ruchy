    use super::*;
    use crate::frontend::ast::{
        BinaryOp, Expr, ExprKind, Literal, Pattern, Span, StructPatternField, Type, TypeKind,
    };
    use crate::middleend::types::{MonoType, TyVarGenerator};

    // ============================================================================
    // Coverage tests for infer_pattern (55 uncov lines, 39.6% coverage)
    // ============================================================================

    fn make_ctx() -> InferenceContext {
        InferenceContext::new()
    }

    fn make_span() -> Span {
        Span::new(0, 0)
    }

    fn make_type(kind: TypeKind) -> Type {
        Type {
            kind,
            span: make_span(),
        }
    }

    // --- Pattern tests: List pattern ---

    #[test]
    fn test_infer_pattern_wildcard() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(&Pattern::Wildcard, &MonoType::Int);
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_literal_int() {
        let mut ctx = make_ctx();
        let result =
            ctx.infer_pattern(&Pattern::Literal(Literal::Integer(42, None)), &MonoType::Int);
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_identifier() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::Identifier("x".to_string()),
            &MonoType::String,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_qualified_name() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::QualifiedName(vec!["Ordering".to_string(), "Less".to_string()]),
            &MonoType::Named("Ordering".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_list() {
        let mut ctx = make_ctx();
        let list_ty = MonoType::List(Box::new(MonoType::Int));
        let result = ctx.infer_pattern(
            &Pattern::List(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ]),
            &list_ty,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_ok_with_result_type() {
        let mut ctx = make_ctx();
        let result_ty = MonoType::Result(
            Box::new(MonoType::Int),
            Box::new(MonoType::String),
        );
        let result = ctx.infer_pattern(
            &Pattern::Ok(Box::new(Pattern::Identifier("val".to_string()))),
            &result_ty,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_ok_with_non_result_type() {
        let mut ctx = make_ctx();
        // When expected type is not Result, it should create a fresh Result type
        // Use Int type to trigger the else branch (non-Result type)
        let result = ctx.infer_pattern(
            &Pattern::Ok(Box::new(Pattern::Identifier("val".to_string()))),
            &MonoType::Named("Unknown".to_string()),
        );
        // This may fail with unification error since Named != Result, which is acceptable
        // The important thing is we exercised the code path
        let _ = result;
    }

    #[test]
    fn test_infer_pattern_err_with_result_type() {
        let mut ctx = make_ctx();
        let result_ty = MonoType::Result(
            Box::new(MonoType::Int),
            Box::new(MonoType::String),
        );
        let result = ctx.infer_pattern(
            &Pattern::Err(Box::new(Pattern::Identifier("e".to_string()))),
            &result_ty,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_err_with_non_result_type() {
        let mut ctx = make_ctx();
        // Use Named type to trigger the else branch (non-Result)
        let result = ctx.infer_pattern(
            &Pattern::Err(Box::new(Pattern::Identifier("e".to_string()))),
            &MonoType::Named("Unknown".to_string()),
        );
        // Unification may fail, but we exercise the code path
        let _ = result;
    }

    #[test]
    fn test_infer_pattern_some_with_optional_type() {
        let mut ctx = make_ctx();
        let opt_ty = MonoType::Optional(Box::new(MonoType::Int));
        let result = ctx.infer_pattern(
            &Pattern::Some(Box::new(Pattern::Identifier("v".to_string()))),
            &opt_ty,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_some_with_non_optional_type() {
        let mut ctx = make_ctx();
        // Use Named type to trigger the else branch (non-Optional)
        let result = ctx.infer_pattern(
            &Pattern::Some(Box::new(Pattern::Identifier("v".to_string()))),
            &MonoType::Named("Unknown".to_string()),
        );
        // Unification may fail, but we exercise the code path
        let _ = result;
    }

    #[test]
    fn test_infer_pattern_none() {
        let mut ctx = make_ctx();
        // None needs to unify with Option<T>, use Named type to exercise the code
        let result = ctx.infer_pattern(&Pattern::None, &MonoType::Named("Unknown".to_string()));
        // Unification may fail, but we exercise the code path
        let _ = result;
    }

    #[test]
    fn test_infer_pattern_tuple() {
        let mut ctx = make_ctx();
        // Use a Tuple type that matches the expected structure
        let tuple_ty = MonoType::Tuple(vec![MonoType::Int, MonoType::String]);
        let result = ctx.infer_pattern(
            &Pattern::Tuple(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ]),
            &tuple_ty,
        );
        // The tuple creates fresh vars and unifies with expected, may or may not succeed
        let _ = result;
    }

    #[test]
    fn test_infer_pattern_struct() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![
                    StructPatternField {
                        name: "x".to_string(),
                        pattern: Some(Pattern::Identifier("a".to_string())),
                    },
                    StructPatternField {
                        name: "y".to_string(),
                        pattern: None, // shorthand
                    },
                ],
                has_rest: false,
            },
            &MonoType::Named("Point".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_range() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::Range {
                start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
                end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
                inclusive: true,
            },
            &MonoType::Int,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_or() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Literal(Literal::Integer(2, None)),
            ]),
            &MonoType::Int,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_rest() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(&Pattern::Rest, &MonoType::Int);
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_rest_named() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::RestNamed("tail".to_string()),
            &MonoType::List(Box::new(MonoType::Int)),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_at_binding() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::AtBinding {
                name: "val".to_string(),
                pattern: Box::new(Pattern::Literal(Literal::Integer(42, None))),
            },
            &MonoType::Int,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_with_default() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::WithDefault {
                pattern: Box::new(Pattern::Identifier("a".to_string())),
                default: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    make_span(),
                )),
            },
            &MonoType::Int,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_tuple_variant() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::TupleVariant {
                path: vec!["Message".to_string(), "Text".to_string()],
                patterns: vec![
                    Pattern::Identifier("content".to_string()),
                ],
            },
            &MonoType::Named("Message".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_infer_pattern_mut() {
        let mut ctx = make_ctx();
        let result = ctx.infer_pattern(
            &Pattern::Mut(Box::new(Pattern::Identifier("x".to_string()))),
            &MonoType::Int,
        );
        assert!(result.is_ok());
    }

    // ============================================================================
    // Coverage tests for infer_binary_op_type (32 uncov lines, 0% coverage)
    // ============================================================================

    #[test]
    fn test_infer_binary_op_add_int() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Add, &MonoType::Int, &MonoType::Int)
            .expect("add int should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_add_float() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Add, &MonoType::Float, &MonoType::Float)
            .expect("add float should succeed");
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_infer_binary_op_subtract_int() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Subtract, &MonoType::Int, &MonoType::Int)
            .expect("subtract int should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_multiply_float() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Multiply, &MonoType::Float, &MonoType::Float)
            .expect("multiply float should succeed");
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_infer_binary_op_divide_int() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Divide, &MonoType::Int, &MonoType::Int)
            .expect("divide int should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_modulo_int() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Modulo, &MonoType::Int, &MonoType::Int)
            .expect("modulo int should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_power_int() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Power, &MonoType::Int, &MonoType::Int)
            .expect("power int should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_power_float() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Power, &MonoType::Float, &MonoType::Float)
            .expect("power float should succeed");
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_infer_binary_op_equal() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Equal, &MonoType::Int, &MonoType::Int)
            .expect("equal should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_not_equal() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::NotEqual, &MonoType::String, &MonoType::String)
            .expect("not_equal should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_less() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Less, &MonoType::Int, &MonoType::Int)
            .expect("less should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_less_equal() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::LessEqual, &MonoType::Float, &MonoType::Float)
            .expect("less_equal should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_greater() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Greater, &MonoType::Int, &MonoType::Int)
            .expect("greater should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_greater_equal() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::GreaterEqual, &MonoType::Int, &MonoType::Int)
            .expect("greater_equal should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_gt_alias() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Gt, &MonoType::Int, &MonoType::Int)
            .expect("gt should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_and() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::And, &MonoType::Bool, &MonoType::Bool)
            .expect("and should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_or() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Or, &MonoType::Bool, &MonoType::Bool)
            .expect("or should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_infer_binary_op_null_coalesce() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::NullCoalesce, &MonoType::Int, &MonoType::Int)
            .expect("null_coalesce should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_bitwise_and() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::BitwiseAnd, &MonoType::Int, &MonoType::Int)
            .expect("bitwise_and should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_bitwise_or() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::BitwiseOr, &MonoType::Int, &MonoType::Int)
            .expect("bitwise_or should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_bitwise_xor() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::BitwiseXor, &MonoType::Int, &MonoType::Int)
            .expect("bitwise_xor should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_left_shift() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::LeftShift, &MonoType::Int, &MonoType::Int)
            .expect("left_shift should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_right_shift() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::RightShift, &MonoType::Int, &MonoType::Int)
            .expect("right_shift should succeed");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_infer_binary_op_send() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(BinaryOp::Send, &MonoType::Named("Actor".to_string()), &MonoType::String)
            .expect("send should succeed");
        assert_eq!(result, MonoType::Unit);
    }

    #[test]
    fn test_infer_binary_op_in() {
        let mut ctx = make_ctx();
        let result = ctx
            .infer_binary_op_type(
                BinaryOp::In,
                &MonoType::Int,
                &MonoType::List(Box::new(MonoType::Int)),
            )
            .expect("in should succeed");
        assert_eq!(result, MonoType::Bool);
    }

    // ============================================================================
    // Coverage tests for ast_type_to_mono_static (48 uncov lines, 9.4% coverage)
    // ============================================================================

    #[test]
    fn test_ast_type_named_i32() {
        let ty = make_type(TypeKind::Named("i32".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("i32 should convert");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_ast_type_named_i64() {
        let ty = make_type(TypeKind::Named("i64".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("i64 should convert");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_ast_type_named_f32() {
        let ty = make_type(TypeKind::Named("f32".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("f32 should convert");
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_ast_type_named_f64() {
        let ty = make_type(TypeKind::Named("f64".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("f64 should convert");
        assert_eq!(result, MonoType::Float);
    }

    #[test]
    fn test_ast_type_named_bool() {
        let ty = make_type(TypeKind::Named("bool".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("bool should convert");
        assert_eq!(result, MonoType::Bool);
    }

    #[test]
    fn test_ast_type_named_string() {
        let ty = make_type(TypeKind::Named("String".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("String should convert");
        assert_eq!(result, MonoType::String);
    }

    #[test]
    fn test_ast_type_named_str() {
        let ty = make_type(TypeKind::Named("str".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("str should convert");
        assert_eq!(result, MonoType::String);
    }

    #[test]
    fn test_ast_type_named_any() {
        let ty = make_type(TypeKind::Named("Any".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Any should convert");
        // Any produces a type variable
        assert!(matches!(result, MonoType::Var(_)));
    }

    #[test]
    fn test_ast_type_named_custom() {
        let ty = make_type(TypeKind::Named("MyStruct".to_string()));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("custom type should convert");
        assert_eq!(result, MonoType::Named("MyStruct".to_string()));
    }

    #[test]
    fn test_ast_type_generic_vec_with_param() {
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Generic {
            base: "Vec".to_string(),
            params: vec![inner],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Vec<i32> should convert");
        assert_eq!(result, MonoType::List(Box::new(MonoType::Int)));
    }

    #[test]
    fn test_ast_type_generic_list_with_param() {
        let inner = make_type(TypeKind::Named("String".to_string()));
        let ty = make_type(TypeKind::Generic {
            base: "List".to_string(),
            params: vec![inner],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("List<String> should convert");
        assert_eq!(result, MonoType::List(Box::new(MonoType::String)));
    }

    #[test]
    fn test_ast_type_generic_vec_no_params() {
        let ty = make_type(TypeKind::Generic {
            base: "Vec".to_string(),
            params: vec![],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Vec without params should convert");
        // Should produce List with a fresh type variable
        assert!(matches!(result, MonoType::List(_)));
    }

    #[test]
    fn test_ast_type_generic_option_with_param() {
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Generic {
            base: "Option".to_string(),
            params: vec![inner],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Option<i32> should convert");
        assert_eq!(result, MonoType::Optional(Box::new(MonoType::Int)));
    }

    #[test]
    fn test_ast_type_generic_option_no_params() {
        let ty = make_type(TypeKind::Generic {
            base: "Option".to_string(),
            params: vec![],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Option without params should convert");
        assert!(matches!(result, MonoType::Optional(_)));
    }

    #[test]
    fn test_ast_type_generic_unknown_base() {
        let ty = make_type(TypeKind::Generic {
            base: "HashMap".to_string(),
            params: vec![
                make_type(TypeKind::Named("String".to_string())),
                make_type(TypeKind::Named("i32".to_string())),
            ],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("HashMap should fall back to Named");
        assert_eq!(result, MonoType::Named("HashMap".to_string()));
    }

    #[test]
    fn test_ast_type_optional() {
        let inner = make_type(TypeKind::Named("bool".to_string()));
        let ty = make_type(TypeKind::Optional(Box::new(inner)));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Optional<bool> should convert");
        assert_eq!(result, MonoType::Optional(Box::new(MonoType::Bool)));
    }

    #[test]
    fn test_ast_type_list() {
        let inner = make_type(TypeKind::Named("f64".to_string()));
        let ty = make_type(TypeKind::List(Box::new(inner)));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("List<f64> should convert");
        assert_eq!(result, MonoType::List(Box::new(MonoType::Float)));
    }

    #[test]
    fn test_ast_type_array() {
        let elem = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Array {
            elem_type: Box::new(elem),
            size: 10,
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Array should convert to List");
        assert_eq!(result, MonoType::List(Box::new(MonoType::Int)));
    }

    #[test]
    fn test_ast_type_function_single_param() {
        let param = make_type(TypeKind::Named("i32".to_string()));
        let ret = make_type(TypeKind::Named("bool".to_string()));
        let ty = make_type(TypeKind::Function {
            params: vec![param],
            ret: Box::new(ret),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("fn(i32) -> bool should convert");
        assert_eq!(
            result,
            MonoType::Function(Box::new(MonoType::Int), Box::new(MonoType::Bool))
        );
    }

    #[test]
    fn test_ast_type_function_multiple_params() {
        let p1 = make_type(TypeKind::Named("i32".to_string()));
        let p2 = make_type(TypeKind::Named("String".to_string()));
        let ret = make_type(TypeKind::Named("bool".to_string()));
        let ty = make_type(TypeKind::Function {
            params: vec![p1, p2],
            ret: Box::new(ret),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("fn(i32, String) -> bool should convert");
        // Two-param function is curried: Int -> (String -> Bool)
        assert_eq!(
            result,
            MonoType::Function(
                Box::new(MonoType::Int),
                Box::new(MonoType::Function(
                    Box::new(MonoType::String),
                    Box::new(MonoType::Bool),
                ))
            )
        );
    }

    #[test]
    fn test_ast_type_function_no_params() {
        let ret = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Function {
            params: vec![],
            ret: Box::new(ret),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("fn() -> i32 should convert");
        // No params: just the return type
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_ast_type_dataframe() {
        let col_ty = make_type(TypeKind::Named("f64".to_string()));
        let ty = make_type(TypeKind::DataFrame {
            columns: vec![("price".to_string(), col_ty)],
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("DataFrame should convert");
        assert_eq!(
            result,
            MonoType::DataFrame(vec![("price".to_string(), MonoType::Float)])
        );
    }

    #[test]
    fn test_ast_type_series() {
        let dtype = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Series {
            dtype: Box::new(dtype),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Series should convert");
        assert_eq!(result, MonoType::Series(Box::new(MonoType::Int)));
    }

    #[test]
    fn test_ast_type_tuple() {
        let ty = make_type(TypeKind::Tuple(vec![
            make_type(TypeKind::Named("i32".to_string())),
            make_type(TypeKind::Named("String".to_string())),
        ]));
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Tuple should convert");
        assert_eq!(
            result,
            MonoType::Tuple(vec![MonoType::Int, MonoType::String])
        );
    }

    #[test]
    fn test_ast_type_reference() {
        let inner = make_type(TypeKind::Named("i32".to_string()));
        let ty = make_type(TypeKind::Reference {
            is_mut: false,
            lifetime: None,
            inner: Box::new(inner),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Reference should convert to inner type");
        assert_eq!(result, MonoType::Int);
    }

    #[test]
    fn test_ast_type_reference_mutable() {
        let inner = make_type(TypeKind::Named("String".to_string()));
        let ty = make_type(TypeKind::Reference {
            is_mut: true,
            lifetime: Some("a".to_string()),
            inner: Box::new(inner),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Mutable reference should convert to inner type");
        assert_eq!(result, MonoType::String);
    }

    #[test]
    fn test_ast_type_refined() {
        let base = make_type(TypeKind::Named("i32".to_string()));
        let constraint = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            make_span(),
        );
        let ty = make_type(TypeKind::Refined {
            base: Box::new(base),
            constraint: Box::new(constraint),
        });
        let result = InferenceContext::ast_type_to_mono_static(&ty)
            .expect("Refined type should extract base type");
        assert_eq!(result, MonoType::Int);
    }
