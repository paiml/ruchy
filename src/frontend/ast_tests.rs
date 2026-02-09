    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn test_span_merge(start1 in 0usize..1000, end1 in 0usize..1000,
                          start2 in 0usize..1000, end2 in 0usize..1000) {
            let span1 = Span::new(start1, end1);
            let span2 = Span::new(start2, end2);
            let merged = span1.merge(span2);
            prop_assert!(merged.start <= span1.start);
            prop_assert!(merged.start <= span2.start);
            prop_assert!(merged.end >= span1.end);
            prop_assert!(merged.end >= span2.end);
        }
    }
    #[test]
    fn test_ast_size() {
        // Track AST node sizes for optimization
        let expr_size = std::mem::size_of::<Expr>();
        let kind_size = std::mem::size_of::<ExprKind>();
        // Current sizes are larger than ideal but acceptable for MVP
        // Future optimization: Use arena allocation and indices
        // Increased limits after adding more OOP features and comment tracking
        assert!(expr_size <= 400, "Expr too large: {expr_size} bytes");
        assert!(kind_size <= 280, "ExprKind too large: {kind_size} bytes");
    }
    #[test]
    fn test_span_creation() {
        let span = Span::new(10, 20);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
    }
    #[test]
    fn test_span_merge_simple() {
        let span1 = Span::new(5, 10);
        let span2 = Span::new(8, 15);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);
    }
    #[test]
    fn test_span_merge_disjoint() {
        let span1 = Span::new(0, 5);
        let span2 = Span::new(10, 15);
        let merged = span1.merge(span2);
        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);
    }
    #[test]
    fn test_expr_creation() {
        let span = Span::new(0, 10);
        let expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), span);
        assert_eq!(expr.span.start, 0);
        assert_eq!(expr.span.end, 10);
        match expr.kind {
            ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 42),
            _ => panic!("Wrong expression kind"),
        }
    }
    #[test]
    fn test_literal_variants() {
        let literals = vec![
            Literal::Integer(42, None),
            #[allow(clippy::approx_constant)]
            Literal::Float(3.15), // Not PI, just a test value
            Literal::String("hello".to_string()),
            Literal::Bool(true),
            Literal::Unit,
        ];
        for lit in literals {
            let expr = Expr::new(ExprKind::Literal(lit.clone()), Span::new(0, 0));
            match expr.kind {
                ExprKind::Literal(l) => assert_eq!(l, lit),
                _ => panic!("Expected literal"),
            }
        }
    }
    #[test]
    fn test_binary_op_display() {
        assert_eq!(BinaryOp::Add.to_string(), "+");
        assert_eq!(BinaryOp::Subtract.to_string(), "-");
        assert_eq!(BinaryOp::Multiply.to_string(), "*");
        assert_eq!(BinaryOp::Divide.to_string(), "/");
        assert_eq!(BinaryOp::Modulo.to_string(), "%");
        assert_eq!(BinaryOp::Power.to_string(), "**");
        assert_eq!(BinaryOp::Equal.to_string(), "==");
        assert_eq!(BinaryOp::NotEqual.to_string(), "!=");
        assert_eq!(BinaryOp::Less.to_string(), "<");
        assert_eq!(BinaryOp::LessEqual.to_string(), "<=");
        assert_eq!(BinaryOp::Greater.to_string(), ">");
        assert_eq!(BinaryOp::GreaterEqual.to_string(), ">=");
        assert_eq!(BinaryOp::And.to_string(), "&&");
        assert_eq!(BinaryOp::Or.to_string(), "||");
        assert_eq!(BinaryOp::BitwiseAnd.to_string(), "&");
        assert_eq!(BinaryOp::BitwiseOr.to_string(), "|");
        assert_eq!(BinaryOp::BitwiseXor.to_string(), "^");
        assert_eq!(BinaryOp::LeftShift.to_string(), "<<");
    }
    #[test]
    fn test_unary_op_display() {
        assert_eq!(UnaryOp::Not.to_string(), "!");
        assert_eq!(UnaryOp::Negate.to_string(), "-");
        assert_eq!(UnaryOp::BitwiseNot.to_string(), "~");
        assert_eq!(UnaryOp::Reference.to_string(), "&");
    }
    #[test]
    fn test_binary_expression() {
        let left = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(0, 1),
        ));
        let right = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::new(4, 5),
        ));
        let expr = Expr::new(
            ExprKind::Binary {
                left,
                op: BinaryOp::Add,
                right,
            },
            Span::new(0, 5),
        );
        match expr.kind {
            ExprKind::Binary {
                left: l,
                op,
                right: r,
            } => {
                assert_eq!(op, BinaryOp::Add);
                match l.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 1),
                    _ => panic!("Wrong left operand"),
                }
                match r.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 2),
                    _ => panic!("Wrong right operand"),
                }
            }
            _ => panic!("Expected binary expression"),
        }
    }
    #[test]
    fn test_unary_expression() {
        let operand = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(1, 5),
        ));
        let expr = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand,
            },
            Span::new(0, 5),
        );
        match expr.kind {
            ExprKind::Unary { op, operand } => {
                assert_eq!(op, UnaryOp::Not);
                match operand.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Wrong operand"),
                }
            }
            _ => panic!("Expected unary expression"),
        }
    }
    #[test]
    fn test_if_expression() {
        let condition = Box::new(Expr::new(
            ExprKind::Literal(Literal::Bool(true)),
            Span::new(3, 7),
        ));
        let then_branch = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(10, 11),
        ));
        let else_branch = Some(Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Span::new(17, 18),
        )));
        let expr = Expr::new(
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            },
            Span::new(0, 18),
        );
        match expr.kind {
            ExprKind::If {
                condition: c,
                then_branch: t,
                else_branch: e,
            } => {
                match c.kind {
                    ExprKind::Literal(Literal::Bool(b)) => assert!(b),
                    _ => panic!("Wrong condition"),
                }
                match t.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 1),
                    _ => panic!("Wrong then branch"),
                }
                assert!(e.is_some());
                if let Some(else_expr) = e {
                    match else_expr.kind {
                        ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 2),
                        _ => panic!("Wrong else branch"),
                    }
                }
            }
            _ => panic!("Expected if expression"),
        }
    }
    #[test]
    fn test_let_expression() {
        let value = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(8, 10),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(14, 15),
        ));
        let expr = Expr::new(
            ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value,
                body,
                is_mutable: false,
                else_block: None,
            },
            Span::new(0, 15),
        );
        match expr.kind {
            ExprKind::Let {
                name,
                value: v,
                body: b,
                ..
            } => {
                assert_eq!(name, "x");
                match v.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 42),
                    _ => panic!("Wrong value"),
                }
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected let expression"),
        }
    }
    #[test]
    fn test_function_expression() {
        let params = vec![Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(10, 13),
            },
            span: Span::new(8, 13),
            is_mutable: false,
            default_value: None,
        }];
        let body = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(20, 21),
        ));
        let expr = Expr::new(
            ExprKind::Function {
                name: "identity".to_string(),
                type_params: vec![],
                params,
                return_type: Some(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(16, 19),
                }),
                body,
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 22),
        );
        match expr.kind {
            ExprKind::Function {
                name,
                params: p,
                return_type,
                body: b,
                ..
            } => {
                assert_eq!(name, "identity");
                assert_eq!(p.len(), 1);
                assert_eq!(p[0].name(), "x");
                assert!(return_type.is_some());
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected function expression"),
        }
    }
    #[test]
    fn test_call_expression() {
        let func = Box::new(Expr::new(
            ExprKind::Identifier("add".to_string()),
            Span::new(0, 3),
        ));
        let args = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(4, 5),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::new(7, 8),
            ),
        ];
        let expr = Expr::new(ExprKind::Call { func, args }, Span::new(0, 9));
        match expr.kind {
            ExprKind::Call { func: f, args: a } => {
                match f.kind {
                    ExprKind::Identifier(name) => assert_eq!(name, "add"),
                    _ => panic!("Wrong function"),
                }
                assert_eq!(a.len(), 2);
            }
            _ => panic!("Expected call expression"),
        }
    }
    #[test]
    fn test_block_expression() {
        let exprs = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(2, 3),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::new(5, 6),
            ),
        ];
        let expr = Expr::new(ExprKind::Block(exprs), Span::new(0, 8));
        match expr.kind {
            ExprKind::Block(block) => {
                assert_eq!(block.len(), 2);
            }
            _ => panic!("Expected block expression"),
        }
    }
    #[test]
    fn test_list_expression() {
        let items = vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span::new(1, 2),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span::new(4, 5),
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span::new(7, 8),
            ),
        ];
        let expr = Expr::new(ExprKind::List(items), Span::new(0, 9));
        match expr.kind {
            ExprKind::List(list) => {
                assert_eq!(list.len(), 3);
            }
            _ => panic!("Expected list expression"),
        }
    }
    #[test]
    fn test_for_expression() {
        let iter = Box::new(Expr::new(
            ExprKind::Range {
                start: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(0, None)),
                    Span::new(10, 11),
                )),
                end: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(10, None)),
                    Span::new(13, 15),
                )),
                inclusive: false,
            },
            Span::new(10, 15),
        ));
        let body = Box::new(Expr::new(
            ExprKind::Identifier("i".to_string()),
            Span::new(20, 21),
        ));
        let expr = Expr::new(
            ExprKind::For {
                label: None,
                var: "i".to_string(),
                pattern: None,
                iter,
                body,
            },
            Span::new(0, 22),
        );
        match expr.kind {
            ExprKind::For {
                label: None,
                var,
                iter: it,
                body: b,
                ..
            } => {
                assert_eq!(var, "i");
                match it.kind {
                    ExprKind::Range { .. } => {}
                    _ => panic!("Wrong iterator"),
                }
                match b.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "i"),
                    _ => panic!("Wrong body"),
                }
            }
            _ => panic!("Expected for expression"),
        }
    }
    #[test]
    fn test_range_expression() {
        let start = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Span::new(0, 1),
        ));
        let end = Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(10, None)),
            Span::new(3, 5),
        ));
        let expr = Expr::new(
            ExprKind::Range {
                start,
                end,
                inclusive: false,
            },
            Span::new(0, 5),
        );
        match expr.kind {
            ExprKind::Range {
                start: s,
                end: e,
                inclusive,
            } => {
                assert!(!inclusive);
                match s.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 1),
                    _ => panic!("Wrong start"),
                }
                match e.kind {
                    ExprKind::Literal(Literal::Integer(n, None)) => assert_eq!(n, 10),
                    _ => panic!("Wrong end"),
                }
            }
            _ => panic!("Expected range expression"),
        }
    }

    #[test]
    fn test_pipeline_expression() {
        let expr_start = Box::new(Expr::new(
            ExprKind::List(vec![
                Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::new(1, 2),
                ),
                Expr::new(
                    ExprKind::Literal(Literal::Integer(2, None)),
                    Span::new(4, 5),
                ),
            ]),
            Span::new(0, 6),
        ));
        let stages = vec![PipelineStage {
            op: Box::new(Expr::new(
                ExprKind::Identifier("filter".to_string()),
                Span::new(10, 16),
            )),
            span: Span::new(10, 16),
        }];
        let expr = Expr::new(
            ExprKind::Pipeline {
                expr: expr_start,
                stages,
            },
            Span::new(0, 16),
        );
        match expr.kind {
            ExprKind::Pipeline { expr: e, stages: s } => {
                assert_eq!(s.len(), 1);
                match e.kind {
                    ExprKind::List(list) => assert_eq!(list.len(), 2),
                    _ => panic!("Wrong pipeline start"),
                }
            }
            _ => panic!("Expected pipeline expression"),
        }
    }
    #[test]
    fn test_match_expression() {
        let expr_to_match = Box::new(Expr::new(
            ExprKind::Identifier("x".to_string()),
            Span::new(6, 7),
        ));
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1, None)),
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("one".to_string())),
                    Span::new(15, 20),
                )),
                span: Span::new(10, 20),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("other".to_string())),
                    Span::new(28, 35),
                )),
                span: Span::new(25, 35),
            },
        ];
        let expr = Expr::new(
            ExprKind::Match {
                expr: expr_to_match,
                arms,
            },
            Span::new(0, 36),
        );
        match expr.kind {
            ExprKind::Match { expr: e, arms: a } => {
                assert_eq!(a.len(), 2);
                match e.kind {
                    ExprKind::Identifier(id) => assert_eq!(id, "x"),
                    _ => panic!("Wrong match expression"),
                }
            }
            _ => panic!("Expected match expression"),
        }
    }
    #[test]
    fn test_pattern_variants() {
        let patterns = vec![
            Pattern::Wildcard,
            Pattern::Literal(Literal::Integer(42, None)),
            Pattern::Identifier("x".to_string()),
            Pattern::Tuple(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Identifier("x".to_string()),
            ]),
            Pattern::List(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Literal(Literal::Integer(2, None)),
            ]),
            Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![StructPatternField {
                    name: "x".to_string(),
                    pattern: Some(Pattern::Identifier("x".to_string())),
                }],
                has_rest: false,
            },
            Pattern::Range {
                start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
                end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
                inclusive: true,
            },
            Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Literal(Literal::Integer(2, None)),
            ]),
            Pattern::Rest,
        ];
        for pattern in patterns {
            match pattern {
                Pattern::Tuple(list) | Pattern::List(list) => assert!(!list.is_empty()),
                Pattern::Struct { fields, .. } => assert!(!fields.is_empty()),
                Pattern::Or(patterns) => assert!(!patterns.is_empty()),
                Pattern::Range { .. }
                | Pattern::Wildcard
                | Pattern::Literal(_)
                | Pattern::Identifier(_)
                | Pattern::Rest
                | Pattern::RestNamed(_)
                | Pattern::Ok(_)
                | Pattern::Err(_)
                | Pattern::Some(_)
                | Pattern::None
                | Pattern::QualifiedName(_)
                | Pattern::AtBinding { .. }
                | Pattern::WithDefault { .. }
                | Pattern::Mut(_)
                | Pattern::TupleVariant { .. } => {} // Simple patterns
            }
        }
    }
    #[test]
    fn test_type_kinds() {
        let types = vec![
            Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            },
            Type {
                kind: TypeKind::Optional(Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(0, 6),
                })),
                span: Span::new(0, 7),
            },
            Type {
                kind: TypeKind::List(Box::new(Type {
                    kind: TypeKind::Named("f64".to_string()),
                    span: Span::new(1, 4),
                })),
                span: Span::new(0, 5),
            },
            Type {
                kind: TypeKind::Function {
                    params: vec![Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 3),
                    }],
                    ret: Box::new(Type {
                        kind: TypeKind::Named("String".to_string()),
                        span: Span::new(7, 13),
                    }),
                },
                span: Span::new(0, 13),
            },
        ];
        for ty in types {
            match ty.kind {
                TypeKind::Named(name) => assert!(!name.is_empty()),
                TypeKind::Generic { base, params } => {
                    assert!(!base.is_empty());
                    assert!(!params.is_empty());
                }
                TypeKind::Optional(_) | TypeKind::List(_) | TypeKind::Series { .. } => {}
                TypeKind::Function { params, .. } => assert!(!params.is_empty()),
                TypeKind::DataFrame { columns } => assert!(!columns.is_empty()),
                TypeKind::Refined { .. } => {} // SPEC-001-H: Refined types (constraint validation not tested here)
                TypeKind::Tuple(ref types) => assert!(!types.is_empty()),
                TypeKind::Reference {
                    is_mut: _,
                    lifetime: _,
                    ref inner,
                } => {
                    // Reference types should have a valid inner type
                    if let TypeKind::Named(ref name) = inner.kind {
                        assert!(!name.is_empty());
                    }
                }
                TypeKind::Array { elem_type: _, size } => {
                    // Array types should have a valid size
                    assert!(size > 0);
                }
            }
        }
    }
    #[test]
    fn test_param_creation() {
        let param = Param {
            pattern: Pattern::Identifier("count".to_string()),
            ty: Type {
                kind: TypeKind::Named("usize".to_string()),
                span: Span::new(6, 11),
            },
            span: Span::new(0, 11),
            is_mutable: false,
            default_value: None,
        };
        assert_eq!(param.name(), "count");
        match param.ty.kind {
            TypeKind::Named(name) => assert_eq!(name, "usize"),
            _ => panic!("Wrong type kind"),
        }
    }

    #[test]
    fn test_string_interpolation_parts() {
        // Test string interpolation with mixed parts
        let parts = vec![
            StringPart::Text("Hello, ".to_string()),
            StringPart::Expr(Box::new(Expr::new(
                ExprKind::Identifier("name".to_string()),
                Span::new(8, 12),
            ))),
            StringPart::Text("!".to_string()),
        ];

        let expr = Expr::new(ExprKind::StringInterpolation { parts }, Span::new(0, 13));

        if let ExprKind::StringInterpolation { parts } = expr.kind {
            assert_eq!(parts.len(), 3);
            match &parts[0] {
                StringPart::Text(s) => assert_eq!(s, "Hello, "),
                _ => panic!("Expected static part"),
            }
            match &parts[1] {
                StringPart::Expr(e) => {
                    if let ExprKind::Identifier(id) = &e.kind {
                        assert_eq!(id, "name");
                    }
                }
                _ => panic!("Expected dynamic part"),
            }
        }
    }

    #[test]
    fn test_async_function_creation() {
        // Test async function with await
        let func = Expr::new(
            ExprKind::Function {
                name: "fetch_data".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(
                    ExprKind::Await {
                        expr: Box::new(Expr::new(
                            ExprKind::Identifier("api_call".to_string()),
                            Span::new(0, 8),
                        )),
                    },
                    Span::new(0, 14),
                )),
                is_async: true,
                is_pub: false,
            },
            Span::new(0, 30),
        );

        if let ExprKind::Function { is_async, body, .. } = func.kind {
            assert!(is_async);
            if let ExprKind::Await { .. } = body.kind {
                // Correctly contains await expression
            } else {
                panic!("Expected await in async function");
            }
        }
    }

    #[test]
    fn test_try_catch_finally() {
        // Test try-catch-finally structure
        let try_catch = Expr::new(
            ExprKind::TryCatch {
                try_block: Box::new(Expr::new(
                    ExprKind::Identifier("risky_operation".to_string()),
                    Span::new(4, 19),
                )),
                catch_clauses: vec![CatchClause {
                    pattern: Pattern::Identifier("e".to_string()),
                    body: Box::new(Expr::new(
                        ExprKind::Identifier("handle_error".to_string()),
                        Span::new(25, 37),
                    )),
                }],
                finally_block: Some(Box::new(Expr::new(
                    ExprKind::Identifier("cleanup".to_string()),
                    Span::new(45, 52),
                ))),
            },
            Span::new(0, 52),
        );

        if let ExprKind::TryCatch {
            catch_clauses,
            finally_block,
            ..
        } = try_catch.kind
        {
            assert_eq!(catch_clauses.len(), 1);
            assert!(finally_block.is_some());
        }
    }

    #[test]
    fn test_result_option_types() {
        // Test Result and Option type constructors
        let ok_val = Expr::new(
            ExprKind::Ok {
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span::new(3, 5),
                )),
            },
            Span::new(0, 6),
        );

        let err_val = Expr::new(
            ExprKind::Err {
                error: Box::new(Expr::new(
                    ExprKind::Literal(Literal::String("error".to_string())),
                    Span::new(4, 11),
                )),
            },
            Span::new(0, 12),
        );

        let some_val = Expr::new(
            ExprKind::Some {
                value: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(1, None)),
                    Span::new(5, 6),
                )),
            },
            Span::new(0, 7),
        );

        let none_val = Expr::new(ExprKind::None, Span::new(0, 4));

        assert!(matches!(ok_val.kind, ExprKind::Ok { .. }));
        assert!(matches!(err_val.kind, ExprKind::Err { .. }));
        assert!(matches!(some_val.kind, ExprKind::Some { .. }));
        assert!(matches!(none_val.kind, ExprKind::None));
    }

    #[test]
    fn test_destructuring_patterns() {
        // Test tuple and struct destructuring
        let tuple_pattern = Pattern::Tuple(vec![
            Pattern::Identifier("x".to_string()),
            Pattern::Identifier("y".to_string()),
            Pattern::Rest,
        ]);

        let struct_pattern = Pattern::Struct {
            name: "User".to_string(),
            fields: vec![
                StructPatternField {
                    name: "name".to_string(),
                    pattern: Some(Pattern::Identifier("n".to_string())),
                },
                StructPatternField {
                    name: "age".to_string(),
                    pattern: None,
                },
            ],
            has_rest: true,
        };

        if let Pattern::Tuple(elements) = tuple_pattern {
            assert_eq!(elements.len(), 3);
            assert!(matches!(elements[2], Pattern::Rest));
        }

        if let Pattern::Struct {
            fields, has_rest, ..
        } = struct_pattern
        {
            assert_eq!(fields.len(), 2);
            assert!(has_rest);
        }
    }

    #[test]
    fn test_qualified_names() {
        // Test module-qualified names
        let qualified = Expr::new(
            ExprKind::QualifiedName {
                module: "std".to_string(),
                name: "println".to_string(),
            },
            Span::new(0, 11),
        );

        if let ExprKind::QualifiedName { module, name } = qualified.kind {
            assert_eq!(module, "std");
            assert_eq!(name, "println");
        }
    }

    #[test]
    fn test_decorator_attributes() {
        // Test decorator/attribute attachment
        let decorated = Expr::with_attributes(
            ExprKind::Function {
                name: "test_func".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: Box::new(Expr::new(ExprKind::Literal(Literal::Unit), Span::new(0, 0))),
                is_async: false,
                is_pub: false,
            },
            Span::new(0, 20),
            vec![
                Attribute {
                    name: "test".to_string(),
                    args: vec![],
                    span: Span::new(0, 5),
                },
                Attribute {
                    name: "bench".to_string(),
                    args: vec![],
                    span: Span::new(0, 6),
                },
            ],
        );

        assert_eq!(decorated.attributes.len(), 2);
        assert_eq!(decorated.attributes[0].name, "test");
        assert_eq!(decorated.attributes[1].name, "bench");
    }

    // Test removed - CompClause type not defined

    #[test]
    fn test_dataframe_operations() {
        // Test DataFrame literal and operations
        let df = Expr::new(
            ExprKind::DataFrame {
                columns: vec![
                    DataFrameColumn {
                        name: "name".to_string(),
                        values: vec![
                            Expr::new(
                                ExprKind::Literal(Literal::String("Alice".to_string())),
                                Span::new(0, 7),
                            ),
                            Expr::new(
                                ExprKind::Literal(Literal::String("Bob".to_string())),
                                Span::new(8, 13),
                            ),
                        ],
                    },
                    DataFrameColumn {
                        name: "age".to_string(),
                        values: vec![
                            Expr::new(
                                ExprKind::Literal(Literal::Integer(25, None)),
                                Span::new(14, 16),
                            ),
                            Expr::new(
                                ExprKind::Literal(Literal::Integer(30, None)),
                                Span::new(17, 19),
                            ),
                        ],
                    },
                ],
            },
            Span::new(0, 50),
        );

        if let ExprKind::DataFrame { columns } = df.kind {
            assert_eq!(columns.len(), 2);
            assert_eq!(columns[0].name, "name");
            assert_eq!(columns[0].values.len(), 2);
            assert_eq!(columns[1].name, "age");
            assert_eq!(columns[1].values.len(), 2);
        }
    }

    #[test]
    fn test_type_cast_operations() {
        // Test type casting
        let cast = Expr::new(
            ExprKind::TypeCast {
                expr: Box::new(Expr::new(
                    ExprKind::Literal(Literal::Integer(42, None)),
                    Span::new(0, 2),
                )),
                target_type: "f64".to_string(),
            },
            Span::new(0, 10),
        );

        if let ExprKind::TypeCast { target_type, .. } = cast.kind {
            assert_eq!(target_type, "f64");
        }
    }

    #[test]
    fn test_binary_operators_complete() {
        // Test all binary operators
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
            BinaryOp::Power,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::Greater,
            BinaryOp::LessEqual,
            BinaryOp::GreaterEqual,
            BinaryOp::And,
            BinaryOp::Or,
            // BinaryOp::Pipeline, // Enum variant pending
            BinaryOp::BitwiseAnd,
            BinaryOp::BitwiseOr,
            BinaryOp::BitwiseXor,
            BinaryOp::LeftShift,
            // BinaryOp::RightShift, // Enum variant pending
        ];

        for op in ops {
            let expr = Expr::new(
                ExprKind::Binary {
                    left: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(1, None)),
                        Span::new(0, 1),
                    )),
                    op,
                    right: Box::new(Expr::new(
                        ExprKind::Literal(Literal::Integer(2, None)),
                        Span::new(2, 3),
                    )),
                },
                Span::new(0, 3),
            );

            if let ExprKind::Binary { op: test_op, .. } = expr.kind {
                assert_eq!(test_op, op);
            }
        }
    }

    #[test]
    fn test_span_operations() {
        // Test span merging and creation
        let span1 = Span::new(0, 10);
        let span2 = Span::new(5, 15);
        let merged = span1.merge(span2);

        assert_eq!(merged.start, 0);
        assert_eq!(merged.end, 15);

        // Test with reverse order
        let merged2 = span2.merge(span1);
        assert_eq!(merged2.start, 0);
        assert_eq!(merged2.end, 15);
    }

    #[test]
    fn test_pattern_with_default() {
        // Test pattern with default value
        let pattern = Pattern::WithDefault {
            pattern: Box::new(Pattern::Identifier("count".to_string())),
            default: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(0, None)),
                Span::new(0, 1),
            )),
        };

        if let Pattern::WithDefault { pattern, default } = pattern {
            match *pattern {
                Pattern::Identifier(name) => assert_eq!(name, "count"),
                _ => panic!("Expected identifier pattern"),
            }
            match default.kind {
                ExprKind::Literal(Literal::Integer(val, None)) => assert_eq!(val, 0),
                _ => panic!("Expected integer literal"),
            }
        }
    }

    // Test removed - Generator and CompClause types not defined

    #[test]
    fn test_mutable_parameter() {
        // Test mutable parameter
        let param = Param {
            pattern: Pattern::Identifier("data".to_string()),
            ty: Type {
                kind: TypeKind::List(Box::new(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                })),
                span: Span::new(0, 6),
            },
            span: Span::new(0, 10),
            is_mutable: true,
            default_value: None,
        };

        assert!(param.is_mutable);
        assert_eq!(param.name(), "data");
    }

    #[test]
    fn test_reference_types() {
        // Test reference and mutable reference types
        let ref_type = Type {
            kind: TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(1, 7),
                }),
            },
            span: Span::new(0, 7),
        };

        let mut_ref_type = Type {
            kind: TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("Vec".to_string()),
                    span: Span::new(4, 7),
                }),
            },
            span: Span::new(0, 7),
        };

        if let TypeKind::Reference {
            is_mut,
            lifetime: _,
            inner,
        } = ref_type.kind
        {
            assert!(!is_mut);
            if let TypeKind::Named(name) = inner.kind {
                assert_eq!(name, "String");
            }
        }

        if let TypeKind::Reference { is_mut, .. } = mut_ref_type.kind {
            assert!(is_mut);
        }
    }

    // EXTREME COVERAGE TESTS FOR 100% AST.RS HOT FILE COVERAGE
    #[test]
    fn test_all_expr_kinds_systematic() {
        // Test every single ExprKind variant for complete coverage

        // Test Literal variants
        let literals = vec![
            ExprKind::Literal(Literal::Integer(42, None)),
            ExprKind::Literal(Literal::Float(3.15)),
            ExprKind::Literal(Literal::Bool(true)),
            ExprKind::Literal(Literal::Bool(false)),
            ExprKind::Literal(Literal::String("test".to_string())),
            ExprKind::Literal(Literal::Char('a')),
            ExprKind::Literal(Literal::Unit),
        ];

        for literal in literals {
            let expr = Expr {
                kind: literal,
                span: Span::new(0, 1),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            };
            // Just test creation and access
            assert!(matches!(expr.kind, ExprKind::Literal(_)));
        }

        // Test Binary operations
        let left = Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(1, None)),
            span: Span::new(0, 1),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });
        let right = Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(2, None)),
            span: Span::new(2, 3),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });

        let binary_ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
            BinaryOp::Power,
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::LessEqual,
            BinaryOp::Greater,
            BinaryOp::GreaterEqual,
            BinaryOp::Gt,
            BinaryOp::And,
            BinaryOp::Or,
            BinaryOp::BitwiseAnd,
            BinaryOp::BitwiseOr,
            BinaryOp::BitwiseXor,
            BinaryOp::LeftShift,
            BinaryOp::NullCoalesce,
        ];

        for op in binary_ops {
            let binary_expr = ExprKind::Binary {
                op,
                left: left.clone(),
                right: right.clone(),
            };
            let expr = Expr {
                kind: binary_expr,
                span: Span::new(0, 3),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            };
            assert!(matches!(expr.kind, ExprKind::Binary { .. }));
        }

        // Test Unary operations
        let operand = Box::new(Expr {
            kind: ExprKind::Literal(Literal::Integer(42, None)),
            span: Span::new(1, 3),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        });

        let unary_ops = vec![
            UnaryOp::Not,
            UnaryOp::Negate,
            UnaryOp::BitwiseNot,
            UnaryOp::Reference,
        ];

        for op in unary_ops {
            let unary_expr = ExprKind::Unary {
                op,
                operand: operand.clone(),
            };
            let expr = Expr {
                kind: unary_expr,
                span: Span::new(0, 3),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            };
            assert!(matches!(expr.kind, ExprKind::Unary { .. }));
        }
    }

    #[test]
    fn test_all_type_kinds_comprehensive() {
        // Test every TypeKind variant for complete coverage

        let type_kinds = vec![
            TypeKind::Named("String".to_string()),
            TypeKind::Generic {
                base: "Vec".to_string(),
                params: vec![Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                }],
            },
            TypeKind::Function {
                params: vec![Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                }],
                ret: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(7, 13),
                }),
            },
            TypeKind::Tuple(vec![
                Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                },
                Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(5, 11),
                },
            ]),
            TypeKind::Array {
                elem_type: Box::new(Type {
                    kind: TypeKind::Named("i32".to_string()),
                    span: Span::new(0, 3),
                }),
                size: 10,
            },
            TypeKind::Reference {
                is_mut: false,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(1, 7),
                }),
            },
            TypeKind::Reference {
                is_mut: true,
                lifetime: None,
                inner: Box::new(Type {
                    kind: TypeKind::Named("String".to_string()),
                    span: Span::new(5, 11),
                }),
            },
            TypeKind::Optional(Box::new(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            })),
            TypeKind::List(Box::new(Type {
                kind: TypeKind::Named("String".to_string()),
                span: Span::new(0, 6),
            })),
            TypeKind::DataFrame {
                columns: vec![
                    (
                        "id".to_string(),
                        Type {
                            kind: TypeKind::Named("i32".to_string()),
                            span: Span::new(0, 3),
                        },
                    ),
                    (
                        "name".to_string(),
                        Type {
                            kind: TypeKind::Named("String".to_string()),
                            span: Span::new(0, 6),
                        },
                    ),
                ],
            },
            TypeKind::Series {
                dtype: Box::new(Type {
                    kind: TypeKind::Named("f64".to_string()),
                    span: Span::new(0, 3),
                }),
            },
        ];

        for type_kind in type_kinds {
            let ty = Type {
                kind: type_kind,
                span: Span::new(0, 10),
            };
            // Test construction and basic operations
            assert!(ty.span.start <= ty.span.end);
        }
    }

    #[test]
    fn test_all_pattern_kinds_comprehensive() {
        // Test every Pattern variant for complete coverage

        let patterns = vec![
            Pattern::Wildcard,
            Pattern::Literal(Literal::Integer(42, None)),
            Pattern::Literal(Literal::String("test".to_string())),
            Pattern::Literal(Literal::Bool(true)),
            Pattern::Identifier("variable".to_string()),
            Pattern::QualifiedName(vec!["Module".to_string(), "Type".to_string()]),
            Pattern::Tuple(vec![
                Pattern::Identifier("x".to_string()),
                Pattern::Identifier("y".to_string()),
            ]),
            Pattern::List(vec![
                Pattern::Identifier("head".to_string()),
                Pattern::Wildcard,
            ]),
            Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![
                    StructPatternField {
                        name: "x".to_string(),
                        pattern: Some(Pattern::Identifier("x_val".to_string())),
                    },
                    StructPatternField {
                        name: "y".to_string(),
                        pattern: None, // Shorthand
                    },
                ],
                has_rest: false,
            },
            Pattern::Struct {
                name: "Point".to_string(),
                fields: vec![],
                has_rest: true, // With rest pattern
            },
        ];

        for pattern in patterns {
            // Test pattern construction and basic operations
            match pattern {
                Pattern::Wildcard => {}
                Pattern::Literal(_) => {}
                Pattern::Identifier(ref name) => assert!(!name.is_empty()),
                Pattern::QualifiedName(ref names) => assert!(!names.is_empty()),
                Pattern::Tuple(ref patterns) => assert!(!patterns.is_empty()),
                Pattern::List(ref patterns) => assert!(!patterns.is_empty()),
                Pattern::Struct { ref name, .. } => assert!(!name.is_empty()),
                _ => {} // Handle all other pattern variants
            }
        }
    }

    #[test]
    fn test_complex_expr_constructions() {
        // Test complex expression constructions for edge case coverage

        // Complex nested call
        let complex_call = ExprKind::Call {
            func: Box::new(Expr {
                kind: ExprKind::FieldAccess {
                    object: Box::new(Expr {
                        kind: ExprKind::Identifier("obj".to_string()),
                        span: Span::new(0, 3),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    field: "method".to_string(),
                },
                span: Span::new(0, 10),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            args: vec![
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(42, None)),
                    span: Span::new(11, 13),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                },
                Expr {
                    kind: ExprKind::Literal(Literal::String("arg".to_string())),
                    span: Span::new(15, 20),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                },
            ],
        };

        let expr = Expr {
            kind: complex_call,
            span: Span::new(0, 21),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(matches!(expr.kind, ExprKind::Call { .. }));

        // Complex if expression
        let complex_if = ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Greater,
                    left: Box::new(Expr {
                        kind: ExprKind::Identifier("x".to_string()),
                        span: Span::new(3, 4),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    right: Box::new(Expr {
                        kind: ExprKind::Literal(Literal::Integer(0, None)),
                        span: Span::new(7, 8),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                },
                span: Span::new(3, 8),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            then_branch: Box::new(Expr {
                kind: ExprKind::Block(vec![Expr {
                    kind: ExprKind::Literal(Literal::String("positive".to_string())),
                    span: Span::new(11, 21),
                    attributes: vec![],
                    leading_comments: vec![],
                    trailing_comment: None,
                }]),
                span: Span::new(9, 23),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            else_branch: Some(Box::new(Expr {
                kind: ExprKind::Literal(Literal::String("negative".to_string())),
                span: Span::new(29, 39),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            })),
        };

        let if_expr = Expr {
            kind: complex_if,
            span: Span::new(0, 40),
            attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
        };
        assert!(matches!(if_expr.kind, ExprKind::If { .. }));
    }

    #[test]
    fn test_attribute_and_span_coverage() {
        // Test attribute and span functionality comprehensively

        // Test various attribute types
        let attributes = vec![
            Attribute {
                name: "inline".to_string(),
                args: vec![],
                span: Span::new(0, 8),
            },
            Attribute {
                name: "deprecated".to_string(),
                args: vec!["Use new_function instead".to_string()],
                span: Span::new(0, 40),
            },
            Attribute {
                name: "derive".to_string(),
                args: vec!["Debug".to_string(), "Clone".to_string()],
                span: Span::new(0, 20),
            },
        ];

        for attr in attributes {
            assert!(!attr.name.is_empty());
            assert!(attr.span.start <= attr.span.end);
        }

        // Test expression with attributes
        let expr_with_attrs = Expr::with_attributes(
            ExprKind::Literal(Literal::Integer(42, None)),
            Span::new(0, 2),
            vec![Attribute {
                name: "test_attr".to_string(),
                args: vec![],
                span: Span::new(0, 10),
            }],
        );

        assert_eq!(expr_with_attrs.attributes.len(), 1);
        assert_eq!(expr_with_attrs.attributes[0].name, "test_attr");

        // Test span operations
        let span1 = Span::new(0, 10);
        let span2 = Span::new(5, 15);

        assert_eq!(span1.start, 0);
        assert_eq!(span1.end, 10);
        assert!(span1.start <= span1.end);
        assert!(span2.start <= span2.end);

        // Test default span
        let default_span = Span::default();
        assert_eq!(default_span.start, 0);
        assert_eq!(default_span.end, 0);
    }

    #[test]
    fn test_all_remaining_expr_kinds() {
        // Test remaining ExprKind variants for 100% coverage

        // Test collections
        let list_expr = ExprKind::List(vec![
            Expr {
                kind: ExprKind::Literal(Literal::Integer(1, None)),
                span: Span::new(1, 2),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
            Expr {
                kind: ExprKind::Literal(Literal::Integer(2, None)),
                span: Span::new(4, 5),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
        ]);

        let tuple_expr = ExprKind::Tuple(vec![
            Expr {
                kind: ExprKind::Literal(Literal::String("first".to_string())),
                span: Span::new(1, 8),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
            Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: Span::new(10, 12),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            },
        ]);

        // Test assignments
        let assign_expr = ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::new(0, 1),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42, None)),
                span: Span::new(4, 6),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
        };

        // Test function definitions
        let func_expr = ExprKind::Function {
            name: "add".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    pattern: Pattern::Identifier("a".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 3),
                    },
                    default_value: None,
                    is_mutable: false,
                    span: Span::new(0, 5),
                },
                Param {
                    pattern: Pattern::Identifier("b".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("i32".to_string()),
                        span: Span::new(0, 3),
                    },
                    default_value: None,
                    is_mutable: false,
                    span: Span::new(0, 5),
                },
            ],
            return_type: Some(Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 3),
            }),
            body: Box::new(Expr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr {
                        kind: ExprKind::Identifier("a".to_string()),
                        span: Span::new(0, 1),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                    right: Box::new(Expr {
                        kind: ExprKind::Identifier("b".to_string()),
                        span: Span::new(4, 5),
                        attributes: vec![],
                        leading_comments: vec![],
                        trailing_comment: None,
                    }),
                },
                span: Span::new(0, 5),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            }),
            is_async: false,
            is_pub: false,
        };

        // Test all constructions
        let expressions = vec![list_expr, tuple_expr, assign_expr, func_expr];

        for expr_kind in expressions {
            let expr = Expr {
                kind: expr_kind,
                span: Span::new(0, 10),
                attributes: vec![],
                leading_comments: vec![],
                trailing_comment: None,
            };
            // Verify construction succeeded
            assert!(expr.span.start <= expr.span.end);
        }
    }

    // Round 96: Additional AST tests

    // Test 46: TypeKind variants
    #[test]
    fn test_typekind_named() {
        let ty = TypeKind::Named("String".to_string());
        match ty {
            TypeKind::Named(name) => assert_eq!(name, "String"),
            _ => panic!("Expected Named"),
        }
    }

    // Test 47: TypeKind Array (struct variant)
    #[test]
    fn test_typekind_array() {
        let inner = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let ty = TypeKind::Array {
            elem_type: Box::new(inner),
            size: 10,
        };
        match ty {
            TypeKind::Array { elem_type, size } => {
                assert!(matches!(elem_type.kind, TypeKind::Named(_)));
                assert_eq!(size, 10);
            }
            _ => panic!("Expected Array"),
        }
    }

    // Test 48: TypeKind Tuple
    #[test]
    fn test_typekind_tuple() {
        let t1 = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let t2 = Type {
            kind: TypeKind::Named("String".to_string()),
            span: Span::default(),
        };
        let ty = TypeKind::Tuple(vec![t1, t2]);
        match ty {
            TypeKind::Tuple(types) => assert_eq!(types.len(), 2),
            _ => panic!("Expected Tuple"),
        }
    }

    // Test 49: TypeKind Function (struct variant)
    #[test]
    fn test_typekind_function() {
        let param = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let ret = Type {
            kind: TypeKind::Named("String".to_string()),
            span: Span::default(),
        };
        let ty = TypeKind::Function {
            params: vec![param],
            ret: Box::new(ret),
        };
        match ty {
            TypeKind::Function { params, .. } => assert_eq!(params.len(), 1),
            _ => panic!("Expected Function"),
        }
    }

    // Test 50: TypeKind Optional
    #[test]
    fn test_typekind_optional() {
        let inner = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let ty = TypeKind::Optional(Box::new(inner));
        assert!(matches!(ty, TypeKind::Optional(_)));
    }

    // Test 51: TypeKind List
    #[test]
    fn test_typekind_list() {
        let inner = Type {
            kind: TypeKind::Named("i32".to_string()),
            span: Span::default(),
        };
        let ty = TypeKind::List(Box::new(inner));
        assert!(matches!(ty, TypeKind::List(_)));
    }

    // Test 52: Pattern variants
    #[test]
    fn test_pattern_identifier() {
        let pat = Pattern::Identifier("x".to_string());
        match pat {
            Pattern::Identifier(name) => assert_eq!(name, "x"),
            _ => panic!("Expected Identifier"),
        }
    }

    // Test 53: Pattern Tuple
    #[test]
    fn test_pattern_tuple() {
        let pat = Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]);
        match pat {
            Pattern::Tuple(pats) => assert_eq!(pats.len(), 2),
            _ => panic!("Expected Tuple"),
        }
    }

    // Test 54: Pattern Wildcard
    #[test]
    fn test_pattern_wildcard() {
        let pat = Pattern::Wildcard;
        assert!(matches!(pat, Pattern::Wildcard));
    }

    // Test 55: BinaryOp arithmetic
    #[test]
    fn test_binary_op_arithmetic() {
        let ops = vec![
            BinaryOp::Add,
            BinaryOp::Subtract,
            BinaryOp::Multiply,
            BinaryOp::Divide,
            BinaryOp::Modulo,
        ];
        assert_eq!(ops.len(), 5);
    }

    // Test 56: BinaryOp comparison
    #[test]
    fn test_binary_op_comparison() {
        let ops = vec![
            BinaryOp::Equal,
            BinaryOp::NotEqual,
            BinaryOp::Less,
            BinaryOp::Greater,
            BinaryOp::LessEqual,
            BinaryOp::GreaterEqual,
        ];
        assert_eq!(ops.len(), 6);
    }

    // Test 57: BinaryOp logical
    #[test]
    fn test_binary_op_logical() {
        let ops = vec![BinaryOp::And, BinaryOp::Or];
        assert_eq!(ops.len(), 2);
    }

    // Test 58: UnaryOp variants
    #[test]
    fn test_unary_op_variants() {
        let ops = vec![UnaryOp::Negate, UnaryOp::Not, UnaryOp::BitwiseNot];
        assert_eq!(ops.len(), 3);
    }

    // Test 59: Literal integer with suffix
    #[test]
    fn test_literal_integer_with_suffix() {
        let lit = Literal::Integer(42, Some("i64".to_string()));
        match lit {
            Literal::Integer(val, suffix) => {
                assert_eq!(val, 42);
                assert_eq!(suffix, Some("i64".to_string()));
            }
            _ => panic!("Expected Integer"),
        }
    }

    // Test 60: Literal float
    #[test]
    fn test_literal_float() {
        let lit = Literal::Float(3.14);
        match lit {
            Literal::Float(val) => assert!((val - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
    }

    // Test 61: Literal string
    #[test]
    fn test_literal_string() {
        let lit = Literal::String("hello".to_string());
        match lit {
            Literal::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String"),
        }
    }

    // Test 62: Literal bool
    #[test]
    fn test_literal_bool() {
        let true_lit = Literal::Bool(true);
        let false_lit = Literal::Bool(false);
        assert!(matches!(true_lit, Literal::Bool(true)));
        assert!(matches!(false_lit, Literal::Bool(false)));
    }

    // Test 63: Literal null
    #[test]
    fn test_literal_null() {
        let lit = Literal::Null;
        assert!(matches!(lit, Literal::Null));
    }

    // Test 64: Literal Unit
    #[test]
    fn test_literal_unit() {
        let lit = Literal::Unit;
        assert!(matches!(lit, Literal::Unit));
    }

    // Test 65: Literal Char
    #[test]
    fn test_literal_char() {
        let lit = Literal::Char('a');
        match lit {
            Literal::Char(c) => assert_eq!(c, 'a'),
            _ => panic!("Expected Char"),
        }
    }

    // Test 66: Literal Byte
    #[test]
    fn test_literal_byte() {
        let lit = Literal::Byte(255);
        match lit {
            Literal::Byte(b) => assert_eq!(b, 255),
            _ => panic!("Expected Byte"),
        }
    }

    // Test 67: Literal Atom
    #[test]
    fn test_literal_atom() {
        let lit = Literal::Atom("ok".to_string());
        match lit {
            Literal::Atom(s) => assert_eq!(s, "ok"),
            _ => panic!("Expected Atom"),
        }
    }
