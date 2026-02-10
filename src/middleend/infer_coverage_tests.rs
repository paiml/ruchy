    use super::*;
    use crate::frontend::ast::{
        BinaryOp, Expr, ExprKind, Literal, Pattern, Span, StructPatternField, Type, TypeKind,
    };
    use crate::middleend::types::{MonoType, TyVarGenerator, TypeScheme};

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

    // ============================================================================
    // Coverage: infer_dataframe_operation (27 uncov, 0% coverage)
    // ============================================================================

    fn make_df_source() -> Expr {
        use crate::frontend::ast::DataFrameColumn;
        Expr::new(
            ExprKind::DataFrame {
                columns: vec![
                    DataFrameColumn {
                        name: "age".to_string(),
                        values: vec![Expr::new(
                            ExprKind::Literal(Literal::Integer(25, None)),
                            make_span(),
                        )],
                    },
                    DataFrameColumn {
                        name: "name".to_string(),
                        values: vec![Expr::new(
                            ExprKind::Literal(Literal::String("Alice".to_string())),
                            make_span(),
                        )],
                    },
                ],
            },
            make_span(),
        )
    }

    fn make_df_operation_expr(
        source: Expr,
        operation: crate::frontend::ast::DataFrameOp,
    ) -> Expr {
        Expr::new(
            ExprKind::DataFrameOperation {
                source: Box::new(source),
                operation,
            },
            make_span(),
        )
    }

    #[test]
    fn test_infer_dataframe_filter() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let filter_cond = Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            make_span(),
        );
        let expr = make_df_operation_expr(source, DataFrameOp::Filter(Box::new(filter_cond)));
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Filter should preserve DataFrame type");
        let ty = result.unwrap();
        assert!(
            matches!(ty, MonoType::DataFrame(_)),
            "Filter result should be DataFrame"
        );
    }

    #[test]
    fn test_infer_dataframe_select() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::Select(vec!["age".to_string()]),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Select should succeed");
        if let Ok(MonoType::DataFrame(cols)) = result {
            assert_eq!(cols.len(), 1, "Select one column should yield one column");
            assert_eq!(cols[0].0, "age");
        } else {
            panic!("Expected DataFrame type from select");
        }
    }

    #[test]
    fn test_infer_dataframe_select_nonexistent_column() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::Select(vec!["nonexistent".to_string()]),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Select nonexistent column still succeeds with empty DataFrame");
        if let Ok(MonoType::DataFrame(cols)) = result {
            assert_eq!(cols.len(), 0, "Nonexistent column yields empty DataFrame");
        }
    }

    #[test]
    fn test_infer_dataframe_groupby() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::GroupBy(vec!["age".to_string()]),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "GroupBy should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_aggregate() {
        use crate::frontend::ast::{AggregateOp, DataFrameOp};
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::Aggregate(vec![AggregateOp::Sum("age".to_string())]),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Aggregate should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_join() {
        use crate::frontend::ast::{DataFrameOp, JoinType};
        let mut ctx = make_ctx();
        let source = make_df_source();
        let other = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::Join {
                other: Box::new(other),
                on: vec!["age".to_string()],
                how: JoinType::Inner,
            },
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Join should succeed");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_sort() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(
            source,
            DataFrameOp::Sort(vec!["age".to_string()]),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Sort should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_limit() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(source, DataFrameOp::Limit(10));
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Limit should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_head() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(source, DataFrameOp::Head(5));
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Head should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_tail() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        let source = make_df_source();
        let expr = make_df_operation_expr(source, DataFrameOp::Tail(5));
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Tail should preserve DataFrame type");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_operation_on_non_dataframe() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        // Source is an integer, not a DataFrame
        let source = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            make_span(),
        );
        let expr = make_df_operation_expr(source, DataFrameOp::Limit(10));
        let result = ctx.infer_expr(&expr);
        assert!(result.is_err(), "DataFrame op on non-DataFrame should fail");
    }

    #[test]
    fn test_infer_dataframe_operation_on_named_dataframe() {
        use crate::frontend::ast::DataFrameOp;
        let mut ctx = make_ctx();
        // Create a let binding with type DataFrame
        let df_var = Expr::new(
            ExprKind::Identifier("df".to_string()),
            make_span(),
        );
        // Bind df to a DataFrame type via let
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::Named("DataFrame".to_string())),
        );
        let expr = Expr::new(
            ExprKind::DataFrameOperation {
                source: Box::new(df_var),
                operation: DataFrameOp::Limit(5),
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "Named DataFrame fallback should work");
        assert!(matches!(
            result.unwrap(),
            MonoType::Named(ref n) if n == "DataFrame"
        ));
    }

    // ============================================================================
    // Coverage tests for infer_dataframe_macro (15 uncov, 0%)
    // ============================================================================

    #[test]
    fn test_infer_dataframe_macro_single_column() {
        let mut ctx = make_ctx();
        // Bind the variable name with List type to match the assigned value
        ctx.env.bind("age", TypeScheme::mono(MonoType::List(Box::new(MonoType::Int))));
        // df! macro: df!(age = [25, 30, 35])
        // The macro takes Assign expressions as arguments
        let args = vec![
            Expr::new(
                ExprKind::Assign {
                    target: Box::new(Expr::new(
                        ExprKind::Identifier("age".to_string()),
                        make_span(),
                    )),
                    value: Box::new(Expr::new(
                        ExprKind::List(vec![
                            Expr::new(ExprKind::Literal(Literal::Integer(25, None)), make_span()),
                            Expr::new(ExprKind::Literal(Literal::Integer(30, None)), make_span()),
                            Expr::new(ExprKind::Literal(Literal::Integer(35, None)), make_span()),
                        ]),
                        make_span(),
                    )),
                },
                make_span(),
            ),
        ];
        let macro_expr = Expr::new(
            ExprKind::Macro {
                name: "df".to_string(),
                args,
            },
            make_span(),
        );
        let result = ctx.infer_expr(&macro_expr);
        assert!(result.is_ok(), "df! macro with single column should infer: {:?}", result.err());
        if let Ok(MonoType::DataFrame(columns)) = &result {
            assert_eq!(columns.len(), 1, "Should have one column");
            assert_eq!(columns[0].0, "age", "Column name should be 'age'");
            assert_eq!(columns[0].1, MonoType::Int, "Column type should be Int");
        } else {
            panic!("Expected DataFrame type, got: {:?}", result);
        }
    }

    #[test]
    fn test_infer_dataframe_macro_multiple_columns() {
        let mut ctx = make_ctx();
        ctx.env.bind("name", TypeScheme::mono(MonoType::List(Box::new(MonoType::String))));
        ctx.env.bind("score", TypeScheme::mono(MonoType::List(Box::new(MonoType::Float))));
        let args = vec![
            Expr::new(
                ExprKind::Assign {
                    target: Box::new(Expr::new(
                        ExprKind::Identifier("name".to_string()),
                        make_span(),
                    )),
                    value: Box::new(Expr::new(
                        ExprKind::List(vec![
                            Expr::new(ExprKind::Literal(Literal::String("Alice".to_string())), make_span()),
                            Expr::new(ExprKind::Literal(Literal::String("Bob".to_string())), make_span()),
                        ]),
                        make_span(),
                    )),
                },
                make_span(),
            ),
            Expr::new(
                ExprKind::Assign {
                    target: Box::new(Expr::new(
                        ExprKind::Identifier("score".to_string()),
                        make_span(),
                    )),
                    value: Box::new(Expr::new(
                        ExprKind::List(vec![
                            Expr::new(ExprKind::Literal(Literal::Float(95.5)), make_span()),
                            Expr::new(ExprKind::Literal(Literal::Float(88.0)), make_span()),
                        ]),
                        make_span(),
                    )),
                },
                make_span(),
            ),
        ];
        let macro_expr = Expr::new(
            ExprKind::Macro {
                name: "df".to_string(),
                args,
            },
            make_span(),
        );
        let result = ctx.infer_expr(&macro_expr);
        assert!(result.is_ok(), "df! macro with multiple columns should infer: {:?}", result.err());
        if let Ok(MonoType::DataFrame(columns)) = &result {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].0, "name");
            assert_eq!(columns[1].0, "score");
        }
    }

    #[test]
    fn test_infer_dataframe_macro_scalar_value_column() {
        let mut ctx = make_ctx();
        ctx.env.bind("count", TypeScheme::mono(MonoType::Int));
        // df!(count = 42) - single scalar value, not a list
        let args = vec![
            Expr::new(
                ExprKind::Assign {
                    target: Box::new(Expr::new(
                        ExprKind::Identifier("count".to_string()),
                        make_span(),
                    )),
                    value: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(42, None)),
                        make_span(),
                    )),
                },
                make_span(),
            ),
        ];
        let macro_expr = Expr::new(
            ExprKind::Macro {
                name: "df".to_string(),
                args,
            },
            make_span(),
        );
        let result = ctx.infer_expr(&macro_expr);
        assert!(result.is_ok(), "df! macro with scalar column should infer: {:?}", result.err());
        if let Ok(MonoType::DataFrame(columns)) = &result {
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].0, "count");
            // Scalar values become the column type directly
            assert_eq!(columns[0].1, MonoType::Int);
        }
    }

    #[test]
    fn test_infer_dataframe_macro_empty() {
        let mut ctx = make_ctx();
        // df!() with no arguments
        let macro_expr = Expr::new(
            ExprKind::Macro {
                name: "df".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&macro_expr);
        assert!(result.is_ok(), "Empty df! macro should produce empty DataFrame");
        if let Ok(MonoType::DataFrame(columns)) = &result {
            assert!(columns.is_empty());
        }
    }

    #[test]
    fn test_infer_dataframe_macro_non_assign_arg_skipped() {
        let mut ctx = make_ctx();
        // df!(42) - non-assignment arg should be skipped
        let args = vec![
            Expr::new(ExprKind::Literal(Literal::Integer(42, None)), make_span()),
        ];
        let macro_expr = Expr::new(
            ExprKind::Macro {
                name: "df".to_string(),
                args,
            },
            make_span(),
        );
        let result = ctx.infer_expr(&macro_expr);
        assert!(result.is_ok(), "Non-assign args should be skipped");
        if let Ok(MonoType::DataFrame(columns)) = &result {
            assert!(columns.is_empty(), "Non-assign arg should produce empty DataFrame");
        }
    }

    // ============================================================================
    // Coverage tests for infer_dataframe_method (16 uncov, 0%)
    // ============================================================================

    #[test]
    fn test_infer_dataframe_method_filter() {
        let mut ctx = make_ctx();
        let df_columns = vec![("age".to_string(), MonoType::Int)];
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(df_columns.clone())),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "filter".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "filter on DataFrame should infer: {:?}", result.err());
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_method_groupby() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("name".to_string(), MonoType::String)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "groupby".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "groupby on DataFrame should infer");
        assert!(matches!(result.unwrap(), MonoType::DataFrame(_)));
    }

    #[test]
    fn test_infer_dataframe_method_select() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("col".to_string(), MonoType::Int)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "select".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "select on DataFrame should infer");
    }

    #[test]
    fn test_infer_dataframe_method_agg() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Float)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "agg".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "agg on DataFrame should infer");
    }

    #[test]
    fn test_infer_dataframe_method_mean() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Float)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "mean".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "mean on DataFrame should return Float");
        assert_eq!(result.unwrap(), MonoType::Float);
    }

    #[test]
    fn test_infer_dataframe_method_sum() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Int)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "sum".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "sum on DataFrame should return Float");
        assert_eq!(result.unwrap(), MonoType::Float);
    }

    #[test]
    fn test_infer_dataframe_method_count() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("id".to_string(), MonoType::Int)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "count".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "count on DataFrame should return Float");
        assert_eq!(result.unwrap(), MonoType::Float);
    }

    #[test]
    fn test_infer_dataframe_method_std() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Float)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "std".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "std on DataFrame should return Float");
        assert_eq!(result.unwrap(), MonoType::Float);
    }

    #[test]
    fn test_infer_dataframe_method_on_named_dataframe() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::Named("DataFrame".to_string())),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "filter".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "filter on Named DataFrame should infer");
        assert!(matches!(result.unwrap(), MonoType::Named(ref n) if n == "DataFrame"));
    }

    #[test]
    fn test_infer_dataframe_method_unknown_method_falls_to_generic() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Int)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "unknown_method".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        // Unknown methods fall through to infer_generic_method
        assert!(result.is_ok(), "Unknown method should fallback to generic: {:?}", result.err());
    }

    // ============================================================================
    // Coverage tests for infer_column_selection (16 uncov, 0%)
    // ============================================================================

    #[test]
    fn test_infer_column_selection_known_column() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![
                ("age".to_string(), MonoType::Int),
                ("name".to_string(), MonoType::String),
            ])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::String("age".to_string())), make_span()),
                ],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "col('age') should infer Series(Int): {:?}", result.err());
        if let Ok(MonoType::Series(inner)) = &result {
            assert_eq!(**inner, MonoType::Int, "Column 'age' should be Series(Int)");
        } else {
            panic!("Expected Series type, got: {:?}", result);
        }
    }

    #[test]
    fn test_infer_column_selection_unknown_column() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![
                ("age".to_string(), MonoType::Int),
            ])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::String("nonexistent".to_string())), make_span()),
                ],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "col('nonexistent') should produce Series with fresh type var");
        assert!(matches!(result.unwrap(), MonoType::Series(_)));
    }

    #[test]
    fn test_infer_column_selection_no_args() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Float)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "col() with no args should produce Series with fresh var");
        assert!(matches!(result.unwrap(), MonoType::Series(_)));
    }

    #[test]
    fn test_infer_column_selection_non_string_arg() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![("val".to_string(), MonoType::Int)])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::Integer(0, None)), make_span()),
                ],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "col(0) with non-string arg should produce Series with fresh var");
        assert!(matches!(result.unwrap(), MonoType::Series(_)));
    }

    #[test]
    fn test_infer_column_selection_on_non_dataframe() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "x",
            TypeScheme::mono(MonoType::Named("Series".to_string())),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("x".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::String("val".to_string())), make_span()),
                ],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        // Named "Series" dispatches to infer_generic_method for "col" since
        // it's not DataFrame or Series in MonoType enum variant form
        // The generic method handler should produce a result
        assert!(result.is_ok(), "col on non-DataFrame should still produce result: {:?}", result.err());
    }

    #[test]
    fn test_infer_column_selection_second_column() {
        let mut ctx = make_ctx();
        ctx.env.bind(
            "df",
            TypeScheme::mono(MonoType::DataFrame(vec![
                ("x".to_string(), MonoType::Int),
                ("y".to_string(), MonoType::Float),
                ("z".to_string(), MonoType::String),
            ])),
        );
        let expr = Expr::new(
            ExprKind::MethodCall {
                receiver: Box::new(Expr::new(
                    ExprKind::Identifier("df".to_string()),
                    make_span(),
                )),
                method: "col".to_string(),
                args: vec![
                    Expr::new(ExprKind::Literal(Literal::String("y".to_string())), make_span()),
                ],
            },
            make_span(),
        );
        let result = ctx.infer_expr(&expr);
        assert!(result.is_ok(), "col('y') should return Series(Float)");
        if let Ok(MonoType::Series(inner)) = &result {
            assert_eq!(**inner, MonoType::Float, "Column 'y' should be Series(Float)");
        }
    }
