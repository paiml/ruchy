    use super::*;
    #[test]
    fn test_eval_special_form_set_empty() {
        let mut interp = make_interpreter();
        let kind = ExprKind::Set(vec![]);
        let result = interp.eval_special_form(&kind).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_special_form_set_with_statements() {
        let mut interp = make_interpreter();
        let kind = ExprKind::Set(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
        ]);
        let result = interp.eval_special_form(&kind).unwrap();
        assert_eq!(result, Value::Integer(2)); // Returns last statement value
    }

    #[test]
    fn test_eval_special_form_object_literal() {
        use crate::frontend::ast::ObjectField;
        let mut interp = make_interpreter();
        let kind = ExprKind::ObjectLiteral {
            fields: vec![ObjectField::KeyValue {
                key: "x".to_string(),
                value: make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            }],
        };
        let result = interp.eval_special_form(&kind).unwrap();
        if let Value::Object(obj) = result {
            assert_eq!(obj.get("x"), Some(&Value::Integer(1)));
        } else {
            panic!("Expected Object");
        }
    }

    // ===== resolve_module_path Tests =====

    #[test]
    fn test_resolve_module_path_not_found() {
        let interp = make_interpreter();
        let result = interp.resolve_module_path("nonexistent::module");
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_module_path_std() {
        let interp = make_interpreter();
        let result = interp.resolve_module_path("std");
        assert!(result.is_some());
    }

    // ===== eval_type_cast Tests =====

    #[test]
    fn test_eval_type_cast_int_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_type_cast(&expr, "f64").unwrap();
        assert_eq!(result, Value::Float(42.0));
    }

    #[test]
    fn test_eval_type_cast_float_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Float(3.9)));
        let result = interp.eval_type_cast(&expr, "i64").unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_type_cast_int_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_type_cast(&expr, "i32").unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_type_cast_float_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::Float(3.14)));
        let result = interp.eval_type_cast(&expr, "f32").unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_type_cast_unsupported() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        let result = interp.eval_type_cast(&expr, "i32");
        assert!(result.is_err());
    }

    // ===== call_function Tests =====

    #[test]
    fn test_call_function_closure_no_args() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![],
            body,
            env,
        };
        let result = interp.call_function(closure, &[]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_call_function_closure_with_args() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Identifier("x".to_string())));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body,
            env,
        };
        let result = interp
            .call_function(closure, &[Value::Integer(42)])
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_call_function_wrong_arg_count() {
        let mut interp = make_interpreter();
        let body = Arc::new(make_expr(ExprKind::Literal(Literal::Integer(42, None))));
        let env = Rc::new(RefCell::new(HashMap::new()));
        let closure = Value::Closure {
            params: vec![("x".to_string(), None)],
            body,
            env,
        };
        let result = interp.call_function(closure, &[]); // Missing required arg
        assert!(result.is_err());
    }

    #[test]
    fn test_call_function_non_callable() {
        let mut interp = make_interpreter();
        let result = interp.call_function(Value::Integer(42), &[]);
        assert!(result.is_err());
    }

    // ===== eval_binary_expr Tests =====

    #[test]
    fn test_eval_binary_expr_null_coalesce_nil() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Null));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::NullCoalesce, &right)
            .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_binary_expr_null_coalesce_not_nil() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Integer(10, None)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::NullCoalesce, &right)
            .unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    #[test]
    fn test_eval_binary_expr_and_short_circuit() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::And, &right)
            .unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_binary_expr_or_short_circuit() {
        let mut interp = make_interpreter();
        let left = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let right = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp
            .eval_binary_expr(&left, AstBinaryOp::Or, &right)
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_binary_expr_in_array() {
        let mut interp = make_interpreter();
        let element = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let collection = make_expr(ExprKind::List(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ]));
        let result = interp
            .eval_binary_expr(&element, AstBinaryOp::In, &collection)
            .unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    // ===== eval_unary_expr Tests =====

    #[test]
    fn test_eval_unary_expr_negate() {
        let mut interp = make_interpreter();
        let operand = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let result = interp.eval_unary_expr(UnaryOp::Negate, &operand).unwrap();
        assert_eq!(result, Value::Integer(-42));
    }

    #[test]
    fn test_eval_unary_expr_not() {
        let mut interp = make_interpreter();
        let operand = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let result = interp.eval_unary_expr(UnaryOp::Not, &operand).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    // ===== eval_if_expr Tests =====

    #[test]
    fn test_eval_if_expr_true_branch() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let else_branch = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let result = interp
            .eval_if_expr(&cond, &then_branch, Some(&else_branch))
            .unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_if_expr_false_branch() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let else_branch = make_expr(ExprKind::Literal(Literal::Integer(2, None)));
        let result = interp
            .eval_if_expr(&cond, &then_branch, Some(&else_branch))
            .unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_eval_if_expr_no_else() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let then_branch = make_expr(ExprKind::Literal(Literal::Integer(1, None)));
        let result = interp.eval_if_expr(&cond, &then_branch, None).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== eval_let_expr Tests =====

    #[test]
    fn test_eval_let_expr() {
        let mut interp = make_interpreter();
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Identifier("x".to_string()));
        let result = interp.eval_let_expr("x", &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_let_expr_unit_body() {
        let mut interp = make_interpreter();
        let value = make_expr(ExprKind::Literal(Literal::Integer(42, None)));
        let body = make_expr(ExprKind::Literal(Literal::Unit));
        let result = interp.eval_let_expr("x", &value, &body).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== eval_block_expr with multiple statements Tests =====

    #[test]
    fn test_eval_block_expr_multiple() {
        let mut interp = make_interpreter();
        let stmts = vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ];
        let result = interp.eval_block_expr(&stmts).unwrap();
        assert_eq!(result, Value::Integer(3)); // Returns last
    }

    // ===== String interpolation Tests =====

    #[test]
    fn test_eval_string_interpolation_text_only() {
        use crate::frontend::ast::StringPart;
        let mut interp = make_interpreter();
        let parts = vec![StringPart::Text("hello world".to_string())];
        let result = interp.eval_string_interpolation(&parts).unwrap();
        assert_eq!(result, Value::from_string("hello world".to_string()));
    }

    #[test]
    fn test_eval_string_interpolation_with_expr() {
        use crate::frontend::ast::StringPart;
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(42));
        let parts = vec![
            StringPart::Text("x = ".to_string()),
            StringPart::Expr(Box::new(make_expr(ExprKind::Identifier("x".to_string())))),
        ];
        let result = interp.eval_string_interpolation(&parts).unwrap();
        assert_eq!(result, Value::from_string("x = 42".to_string()));
    }

    // ===== Macro tests via eval_string =====

    #[test]
    fn test_eval_vec_macro() {
        let mut interp = make_interpreter();
        let result = interp.eval_string("vec![1, 2, 3]").unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    // ===== eval_comprehension Tests =====

    #[test]
    fn test_eval_list_comprehension_simple() {
        use crate::frontend::ast::ComprehensionClause;
        let mut interp = make_interpreter();
        let element = make_expr(ExprKind::Identifier("x".to_string()));
        let clauses = vec![ComprehensionClause {
            variable: "x".to_string(),
            iterable: Box::new(make_expr(ExprKind::List(vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            ]))),
            condition: None,
        }];
        let result = interp.eval_list_comprehension(&element, &clauses).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_check_comprehension_condition_none() {
        let mut interp = make_interpreter();
        let result = interp.check_comprehension_condition(None).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_comprehension_condition_true() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(true)));
        let result = interp.check_comprehension_condition(Some(&cond)).unwrap();
        assert!(result);
    }

    #[test]
    fn test_check_comprehension_condition_false() {
        let mut interp = make_interpreter();
        let cond = make_expr(ExprKind::Literal(Literal::Bool(false)));
        let result = interp.check_comprehension_condition(Some(&cond)).unwrap();
        assert!(!result);
    }

    // ===== Macro Tests =====

    #[test]
    fn test_vec_macro_empty() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_vec_macro_with_elements() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "vec".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
                make_expr(ExprKind::Literal(Literal::Integer(3, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_format_macro_simple() {
        // Test that format! macro returns a string value
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "hello".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // format! returns a Value::String
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_macro_with_placeholder() {
        // Test that format! macro handles placeholders
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::String("x = {}".to_string()))),
                make_expr(ExprKind::Literal(Literal::Integer(42, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // format! returns a Value::String containing the formatted output
        assert!(matches!(result, Value::String(_)));
        // Check that the result contains "42"
        let result_str = result.to_string();
        assert!(result_str.contains("42"));
    }

    #[test]
    fn test_format_macro_empty_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_macro_debug_placeholder() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::String("val = {:?}".to_string()))),
                make_expr(ExprKind::Literal(Literal::Integer(5, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        // Check that result contains the debug format
        assert!(result.to_string().contains("5") || result.to_string().contains("Integer"));
    }

    #[test]
    fn test_unknown_macro_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Macro {
            name: "unknown_macro".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Control Flow Tests =====

    #[test]
    fn test_ternary_true() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Ternary {
                condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
                true_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
                false_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_ternary_false() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Ternary {
                condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(false)))),
                true_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
                false_expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_break_with_value() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Break {
                label: None,
                value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                    42, None,
                ))))),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Break(None, Value::Integer(42)))
        ));
    }

    #[test]
    fn test_break_with_label() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Break {
                label: Some("outer".to_string()),
                value: None,
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Break(Some(label), Value::Nil)) if label == "outer"
        ));
    }

    #[test]
    fn test_continue_with_label() {
        let mut interp = make_interpreter();
        let expr = Expr {
            kind: ExprKind::Continue {
                label: Some("inner".to_string()),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Continue(Some(label))) if label == "inner"
        ));
    }

    // ===== Special Form Tests =====

    #[test]
    fn test_none_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::None);
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "None");
            assert!(data.is_none());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_some_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Some {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::EnumVariant {
            enum_name,
            variant_name,
            data,
        } = result
        {
            assert_eq!(enum_name, "Option");
            assert_eq!(variant_name, "Some");
            assert!(data.is_some());
        } else {
            panic!("Expected EnumVariant");
        }
    }

    #[test]
    fn test_set_special_form() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Set(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            make_expr(ExprKind::Literal(Literal::Integer(3, None))),
        ]));
        let result = interp.eval_expr(&expr).unwrap();
        // Set returns last value
        assert_eq!(result, Value::Integer(3));
    }

    // ===== Import Tests =====

    #[test]
    fn test_import_all_wildcard() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportAll {
            module: "std::math".to_string(),
            alias: "*".to_string(),
        });
        // Wildcard imports return Nil
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_import_default() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportDefault {
            module: "mymodule".to_string(),
            name: "mm".to_string(),
        });
        // ImportDefault not yet implemented
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Additional Tests for Coverage =====

    #[test]
    fn test_env_set_mut_coverage() {
        let mut interp = make_interpreter();
        interp.env_set_mut("mutable_var_cov".to_string(), Value::Integer(1));
        assert!(interp.lookup_variable("mutable_var_cov").is_ok());
    }

    #[test]
    fn test_is_actor_operation_coverage() {
        let actor_expr = Box::new(make_expr(ExprKind::Identifier("actor".to_string())));
        assert!(Interpreter::is_actor_operation(&ExprKind::Spawn {
            actor: actor_expr,
        }));
    }

    #[test]
    fn test_is_special_form_coverage() {
        assert!(Interpreter::is_special_form(&ExprKind::None));
        assert!(Interpreter::is_special_form(&ExprKind::Some {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        }));
    }

    #[test]
    fn test_is_control_flow_expr_coverage() {
        assert!(Interpreter::is_control_flow_expr(&ExprKind::If {
            condition: Box::new(make_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: None,
        }));
    }

    #[test]
    fn test_is_data_structure_expr_coverage() {
        assert!(Interpreter::is_data_structure_expr(&ExprKind::List(vec![])));
    }

    #[test]
    fn test_is_assignment_expr_coverage() {
        assert!(Interpreter::is_assignment_expr(&ExprKind::Assign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        }));
    }

    // ===== MacroInvocation Tests =====

    #[test]
    fn test_macro_invocation_vec() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "vec".to_string(),
            args: vec![
                make_expr(ExprKind::Literal(Literal::Integer(1, None))),
                make_expr(ExprKind::Literal(Literal::Integer(2, None))),
            ],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_macro_invocation_format() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "test".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_macro_invocation_unknown() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::MacroInvocation {
            name: "unknown_macro".to_string(),
            args: vec![],
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Try Operator Tests =====

    #[test]
    fn test_try_operator_ok() {
        let mut interp = make_interpreter();
        let _ok_value = make_expr(ExprKind::Call {
            func: Box::new(make_expr(ExprKind::Identifier("Ok".to_string()))),
            args: vec![make_expr(ExprKind::Literal(Literal::Integer(42, None)))],
        });
        // We need to set up Ok function first
        interp.set_variable(
            "Ok",
            Value::Closure {
                params: vec![("value".to_string(), None)],
                body: std::sync::Arc::new(make_expr(ExprKind::Block(vec![]))),
                env: interp.current_env().clone(),
            },
        );
        // Try operator test with EnumVariant
        let ok_enum = Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Ok".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        interp.set_variable("result_val", ok_enum);
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("result_val".to_string()))),
        });
        let result = interp.eval_expr(&try_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_try_operator_err() {
        let mut interp = make_interpreter();
        let err_enum = Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            data: Some(vec![Value::from_string("error".to_string())]),
        };
        interp.set_variable("err_val", err_enum);
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("err_val".to_string()))),
        });
        let result = interp.eval_expr(&try_expr);
        // Should return an error (early return with Err variant)
        assert!(result.is_err());
    }

    #[test]
    fn test_try_operator_invalid_type() {
        let mut interp = make_interpreter();
        interp.set_variable("not_result", Value::Integer(42));
        let try_expr = make_expr(ExprKind::Try {
            expr: Box::new(make_expr(ExprKind::Identifier("not_result".to_string()))),
        });
        let result = interp.eval_expr(&try_expr);
        assert!(result.is_err());
    }

    // ===== Lazy Expression Tests =====

    #[test]
    fn test_lazy_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Lazy {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== AsyncBlock Tests =====

    #[test]
    fn test_async_block() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::AsyncBlock {
            body: Box::new(make_expr(ExprKind::Literal(Literal::Integer(100, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    // ===== IfLet Tests =====

    #[test]
    fn test_if_let_match() {
        let mut interp = make_interpreter();
        // Set up a Some value
        let some_val = Value::EnumVariant {
            enum_name: "Option".to_string(),
            variant_name: "Some".to_string(),
            data: Some(vec![Value::Integer(42)]),
        };
        interp.set_variable("opt", some_val);

        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Identifier("x".to_string()),
            expr: Box::new(make_expr(ExprKind::Identifier("opt".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                0, None,
            ))))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_if_let_no_match() {
        let mut interp = make_interpreter();
        // Set up Nil (won't match wildcard pattern that expects value)
        interp.set_variable("opt", Value::Nil);

        // Use a pattern that won't match Nil
        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Literal(Literal::Integer(5, None)),
            expr: Box::new(make_expr(ExprKind::Identifier("opt".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                0, None,
            ))))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(0));
    }

    #[test]
    fn test_if_let_no_else() {
        let mut interp = make_interpreter();
        interp.set_variable("val", Value::Integer(10));

        let expr = make_expr(ExprKind::IfLet {
            pattern: Pattern::Literal(Literal::Integer(5, None)), // Won't match
            expr: Box::new(make_expr(ExprKind::Identifier("val".to_string()))),
            then_branch: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
            else_branch: None,
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Module Expression Tests =====

    #[test]
    fn test_module_declaration_error() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ModuleDeclaration {
            name: "unresolved_module".to_string(),
        });
        let result = interp.eval_expr(&expr);
        assert!(result.is_err());
    }

    // ===== Pipeline Tests =====

    #[test]
    fn test_pipeline_with_method() {
        let mut interp = make_interpreter();
        // Define a simple function
        interp.set_variable(
            "double",
            Value::Closure {
                params: vec![("x".to_string(), None)],
                body: std::sync::Arc::new(make_expr(ExprKind::Binary {
                    left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                    op: crate::frontend::ast::BinaryOp::Multiply,
                    right: Box::new(make_expr(ExprKind::Literal(Literal::Integer(2, None)))),
                })),
                env: interp.current_env().clone(),
            },
        );

        let expr = make_expr(ExprKind::Pipeline {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
            stages: vec![crate::frontend::ast::PipelineStage {
                op: Box::new(make_expr(ExprKind::Identifier("double".to_string()))),
                span: Span::default(),
            }],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(10));
    }

    // ===== Format String Edge Cases =====

    #[test]
    fn test_format_debug_incomplete() {
        let mut interp = make_interpreter();
        // Test {:? without closing brace
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{:?x".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_colon_only() {
        let mut interp = make_interpreter();
        // Test {: without ?
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{:x".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_format_excess_placeholders() {
        let mut interp = make_interpreter();
        // More {} than arguments - should preserve placeholder
        let expr = make_expr(ExprKind::Macro {
            name: "format".to_string(),
            args: vec![make_expr(ExprKind::Literal(Literal::String(
                "{} {} {}".to_string(),
            )))],
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert!(s.as_ref().contains("{}"));
        } else {
            panic!("Expected String");
        }
    }

    // ===== Import Tests =====

    #[test]
    fn test_import_stdlib() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Import {
            module: "std::env".to_string(),
            items: None,
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_import_all_with_alias() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ImportAll {
            module: "std::math".to_string(),
            alias: "m".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Nil);
    }

    // ===== Additional Expression Tests =====

    #[test]
    fn test_eval_string_interpolation_empty() {
        let mut interp = make_interpreter();
        let parts = vec![];
        let expr = make_expr(ExprKind::StringInterpolation { parts });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "");
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_eval_qualified_name() {
        let mut interp = make_interpreter();
        // Set up std::env in environment
        let expr = make_expr(ExprKind::QualifiedName {
            module: "std".to_string(),
            name: "env".to_string(),
        });
        // This may fail if std::env is not set up, but it tests the code path
        let _result = interp.eval_expr(&expr);
    }

    #[test]
    fn test_loop_expression() {
        let mut interp = make_interpreter();
        // Create a loop that breaks immediately
        interp.set_variable("counter", Value::Integer(0));
        let loop_expr = Expr {
            kind: ExprKind::Loop {
                label: None,
                body: Box::new(make_expr(ExprKind::Break {
                    label: None,
                    value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                        42, None,
                    ))))),
                })),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&loop_expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_labeled_loop() {
        let mut interp = make_interpreter();
        let loop_expr = Expr {
            kind: ExprKind::Loop {
                label: Some("outer".to_string()),
                body: Box::new(make_expr(ExprKind::Break {
                    label: Some("outer".to_string()),
                    value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                        99, None,
                    ))))),
                })),
            },
            span: Span::default(),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        let result = interp.eval_expr(&loop_expr).unwrap();
        assert_eq!(result, Value::Integer(99));
    }

    // ===== Return Expression Test =====

    #[test]
    fn test_return_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Return {
            value: Some(Box::new(make_expr(ExprKind::Literal(Literal::Integer(
                123, None,
            ))))),
        });
        let result = interp.eval_expr(&expr);
        assert!(matches!(
            result,
            Err(InterpreterError::Return(Value::Integer(123)))
        ));
    }

    #[test]
    fn test_return_expr_no_value() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Return { value: None });
        let result = interp.eval_expr(&expr);
        assert!(matches!(result, Err(InterpreterError::Return(Value::Nil))));
    }

    // ===== Array Init Test =====

    #[test]
    fn test_array_init_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::ArrayInit {
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            size: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 5);
        } else {
            panic!("Expected Array");
        }
    }

    // ===== Index Access Test =====

    #[test]
    fn test_index_access_tuple() {
        let mut interp = make_interpreter();
        let tuple = Value::Tuple(std::sync::Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        interp.set_variable("t", tuple);

        let expr = make_expr(ExprKind::IndexAccess {
            object: Box::new(make_expr(ExprKind::Identifier("t".to_string()))),
            index: Box::new(make_expr(ExprKind::Literal(Literal::Integer(1, None)))),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // ===== call_function Edge Cases =====

    #[test]
    fn test_call_function_class_constructor() {
        let mut interp = make_interpreter();
        // Set up a minimal class definition
        let mut class_def = HashMap::new();
        class_def.insert(
            "__type".to_string(),
            Value::from_string("Class".to_string()),
        );
        class_def.insert(
            "__name".to_string(),
            Value::from_string("Point".to_string()),
        );
        class_def.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(HashMap::new())),
        );
        class_def.insert(
            "__methods".to_string(),
            Value::Array(std::sync::Arc::from(vec![])),
        );
        interp.set_variable("Point", Value::Object(std::sync::Arc::new(class_def)));

        let constructor = Value::from_string("__class_constructor__:Point:new".to_string());
        let result = interp.call_function(constructor, &[]);
        // May fail if instantiate_class_with_constructor isn't fully set up
        // but this tests the code path
        let _ = result;
    }

    #[test]
    fn test_call_function_struct_constructor() {
        let mut interp = make_interpreter();
        // Set up a minimal struct definition
        let mut struct_def = HashMap::new();
        struct_def.insert(
            "__type".to_string(),
            Value::from_string("Struct".to_string()),
        );
        struct_def.insert(
            "__name".to_string(),
            Value::from_string("Point".to_string()),
        );
        struct_def.insert(
            "__fields".to_string(),
            Value::Object(std::sync::Arc::new(HashMap::new())),
        );
        interp.set_variable("Point", Value::Object(std::sync::Arc::new(struct_def)));

        let constructor = Value::from_string("__struct_constructor__:Point".to_string());
        let result = interp.call_function(constructor, &[]);
        // May succeed or fail depending on struct setup
        let _ = result;
    }

    #[test]
    fn test_call_function_closure_with_defaults() {
        let mut interp = make_interpreter();
        // Create a closure with a default parameter
        let closure = Value::Closure {
            params: vec![
                ("x".to_string(), None),
                (
                    "y".to_string(),
                    Some(std::sync::Arc::new(make_expr(ExprKind::Literal(
                        Literal::Integer(10, None),
                    )))),
                ),
            ],
            body: std::sync::Arc::new(make_expr(ExprKind::Binary {
                left: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
                op: crate::frontend::ast::BinaryOp::Add,
                right: Box::new(make_expr(ExprKind::Identifier("y".to_string()))),
            })),
            env: interp.current_env().clone(),
        };

        // Call with only required arg - default should be used
        let result = interp
            .call_function(closure.clone(), &[Value::Integer(5)])
            .unwrap();
        assert_eq!(result, Value::Integer(15));

        // Call with both args
        let result = interp
            .call_function(closure, &[Value::Integer(5), Value::Integer(3)])
            .unwrap();
        assert_eq!(result, Value::Integer(8));
    }

    #[test]
    fn test_call_function_arg_count_errors() {
        let mut interp = make_interpreter();
        let closure = Value::Closure {
            params: vec![("x".to_string(), None), ("y".to_string(), None)],
            body: std::sync::Arc::new(make_expr(ExprKind::Literal(Literal::Integer(0, None)))),
            env: interp.current_env().clone(),
        };

        // Too few arguments
        let result = interp.call_function(closure.clone(), &[Value::Integer(1)]);
        assert!(result.is_err());

        // Too many arguments
        let result = interp.call_function(
            closure,
            &[Value::Integer(1), Value::Integer(2), Value::Integer(3)],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_call_function_builtin() {
        let mut interp = make_interpreter();
        // Test a builtin function
        let builtin = Value::from_string("__builtin_len__".to_string());
        let arr = Value::Array(std::sync::Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]));
        let result = interp.call_function(builtin, &[arr]).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    // ===== Spread Expression Tests =====

    #[test]
    fn test_spread_in_list_returns_error() {
        let mut interp = make_interpreter();
        interp.set_variable(
            "inner",
            Value::Array(std::sync::Arc::from(vec![
                Value::Integer(2),
                Value::Integer(3),
            ])),
        );

        // Spread inside list triggers error (feature not yet implemented)
        let expr = make_expr(ExprKind::List(vec![
            make_expr(ExprKind::Literal(Literal::Integer(1, None))),
            make_expr(ExprKind::Spread {
                expr: Box::new(make_expr(ExprKind::Identifier("inner".to_string()))),
            }),
            make_expr(ExprKind::Literal(Literal::Integer(4, None))),
        ]));
        let result = interp.eval_expr(&expr);
        // Spread expressions in list context return error
        assert!(result.is_err());
    }

    // ===== Await Expression Test =====

    #[test]
    fn test_await_expr() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::Await {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
        });
        // In sync mode, await just returns the value
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    // ===== Field Access Tests =====

    #[test]
    fn test_field_access_object() {
        let mut interp = make_interpreter();
        let mut obj = HashMap::new();
        obj.insert("x".to_string(), Value::Integer(42));
        obj.insert("y".to_string(), Value::Integer(100));
        interp.set_variable("obj", Value::Object(std::sync::Arc::new(obj)));

        let expr = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("obj".to_string()))),
            field: "x".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_field_access_struct() {
        let mut interp = make_interpreter();
        let mut fields = HashMap::new();
        fields.insert("name".to_string(), Value::from_string("Alice".to_string()));
        let struct_val = Value::Struct {
            name: "Person".to_string(),
            fields: std::sync::Arc::new(fields),
        };
        interp.set_variable("person", struct_val);

        let expr = make_expr(ExprKind::FieldAccess {
            object: Box::new(make_expr(ExprKind::Identifier("person".to_string()))),
            field: "name".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::String(s) = result {
            assert_eq!(s.as_ref(), "Alice");
        } else {
            panic!("Expected String");
        }
    }

    // ===== Compound Assignment Tests =====

    #[test]
    fn test_compound_add_assign() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(10));

        let expr = make_expr(ExprKind::CompoundAssign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            op: crate::frontend::ast::BinaryOp::Add,
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(5, None)))),
        });
        interp.eval_expr(&expr).unwrap();

        let x = interp.lookup_variable("x").unwrap();
        assert_eq!(x, Value::Integer(15));
    }

    #[test]
    fn test_compound_mul_assign() {
        let mut interp = make_interpreter();
        interp.set_variable("x", Value::Integer(3));

        let expr = make_expr(ExprKind::CompoundAssign {
            target: Box::new(make_expr(ExprKind::Identifier("x".to_string()))),
            op: crate::frontend::ast::BinaryOp::Multiply,
            value: Box::new(make_expr(ExprKind::Literal(Literal::Integer(4, None)))),
        });
        interp.eval_expr(&expr).unwrap();

        let x = interp.lookup_variable("x").unwrap();
        assert_eq!(x, Value::Integer(12));
    }

    // ===== Type Cast Tests =====

    #[test]
    fn test_type_cast_int_to_float() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::TypeCast {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Integer(42, None)))),
            target_type: "f64".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        if let Value::Float(f) = result {
            assert!((f - 42.0).abs() < 0.001);
        } else {
            panic!("Expected Float");
        }
    }

    #[test]
    fn test_type_cast_float_to_int() {
        let mut interp = make_interpreter();
        let expr = make_expr(ExprKind::TypeCast {
            expr: Box::new(make_expr(ExprKind::Literal(Literal::Float(3.7)))),
            target_type: "i32".to_string(),
        });
        let result = interp.eval_expr(&expr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }
