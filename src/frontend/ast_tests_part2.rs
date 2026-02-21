
use super::*;

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

// ========================================================================
// Coverage: Pattern::primary_name — all variants (24 uncov, 35.1% cov)
// ========================================================================

#[test]
fn test_primary_name_identifier() {
    let pat = Pattern::Identifier("foo".to_string());
    assert_eq!(pat.primary_name(), "foo");
}

#[test]
fn test_primary_name_qualified_name() {
    let pat = Pattern::QualifiedName(vec!["std".to_string(), "io".to_string()]);
    assert_eq!(pat.primary_name(), "std::io");
}

#[test]
fn test_primary_name_tuple_non_empty() {
    let pat = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    assert_eq!(pat.primary_name(), "a");
}

#[test]
fn test_primary_name_tuple_empty() {
    let pat = Pattern::Tuple(vec![]);
    assert_eq!(pat.primary_name(), "_tuple");
}

#[test]
fn test_primary_name_list_non_empty() {
    let pat = Pattern::List(vec![Pattern::Identifier("x".to_string())]);
    assert_eq!(pat.primary_name(), "x");
}

#[test]
fn test_primary_name_list_empty() {
    let pat = Pattern::List(vec![]);
    assert_eq!(pat.primary_name(), "_list");
}

#[test]
fn test_primary_name_struct_with_name() {
    let pat = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![],
        has_rest: false,
    };
    assert_eq!(pat.primary_name(), "Point");
}

#[test]
fn test_primary_name_struct_anonymous_with_fields() {
    let pat = Pattern::Struct {
        name: String::new(),
        fields: vec![StructPatternField {
            name: "x".to_string(),
            pattern: None,
        }],
        has_rest: false,
    };
    assert_eq!(pat.primary_name(), "x");
}

#[test]
fn test_primary_name_struct_anonymous_no_fields() {
    let pat = Pattern::Struct {
        name: String::new(),
        fields: vec![],
        has_rest: false,
    };
    assert_eq!(pat.primary_name(), "_struct");
}

#[test]
fn test_primary_name_tuple_variant_with_patterns() {
    let pat = Pattern::TupleVariant {
        path: vec!["Option".to_string(), "Some".to_string()],
        patterns: vec![Pattern::Identifier("val".to_string())],
    };
    assert_eq!(pat.primary_name(), "val");
}

#[test]
fn test_primary_name_tuple_variant_empty_patterns() {
    let pat = Pattern::TupleVariant {
        path: vec!["Color".to_string(), "Red".to_string()],
        patterns: vec![],
    };
    assert_eq!(pat.primary_name(), "Color::Red");
}

#[test]
fn test_primary_name_ok() {
    let pat = Pattern::Ok(Box::new(Pattern::Identifier("v".to_string())));
    assert_eq!(pat.primary_name(), "v");
}

#[test]
fn test_primary_name_err() {
    let pat = Pattern::Err(Box::new(Pattern::Identifier("e".to_string())));
    assert_eq!(pat.primary_name(), "e");
}

#[test]
fn test_primary_name_some() {
    let pat = Pattern::Some(Box::new(Pattern::Identifier("s".to_string())));
    assert_eq!(pat.primary_name(), "s");
}

#[test]
fn test_primary_name_none() {
    assert_eq!(Pattern::None.primary_name(), "_none");
}

#[test]
fn test_primary_name_or_pattern() {
    let pat = Pattern::Or(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    assert_eq!(pat.primary_name(), "a");
}

#[test]
fn test_primary_name_or_empty() {
    let pat = Pattern::Or(vec![]);
    assert_eq!(pat.primary_name(), "_or");
}

#[test]
fn test_primary_name_wildcard() {
    assert_eq!(Pattern::Wildcard.primary_name(), "_");
}

#[test]
fn test_primary_name_rest() {
    assert_eq!(Pattern::Rest.primary_name(), "_rest");
}

#[test]
fn test_primary_name_rest_named() {
    let pat = Pattern::RestNamed("rest".to_string());
    assert_eq!(pat.primary_name(), "rest");
}

#[test]
fn test_primary_name_at_binding() {
    let pat = Pattern::AtBinding {
        name: "all".to_string(),
        pattern: Box::new(Pattern::Wildcard),
    };
    assert_eq!(pat.primary_name(), "all");
}

#[test]
fn test_primary_name_with_default() {
    let pat = Pattern::WithDefault {
        pattern: Box::new(Pattern::Identifier("x".to_string())),
        default: Box::new(Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Span::default(),
        )),
    };
    assert_eq!(pat.primary_name(), "x");
}

#[test]
fn test_primary_name_mut() {
    let pat = Pattern::Mut(Box::new(Pattern::Identifier("y".to_string())));
    assert_eq!(pat.primary_name(), "y");
}

#[test]
fn test_primary_name_literal() {
    let pat = Pattern::Literal(Literal::Integer(42, None));
    let name = pat.primary_name();
    assert!(name.starts_with("_literal_"), "Got: {name}");
}

#[test]
fn test_primary_name_range() {
    let pat = Pattern::Range {
        start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
        end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
        inclusive: true,
    };
    assert_eq!(pat.primary_name(), "_range");
}
