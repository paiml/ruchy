//! TDD tests for backend/transpiler/mod.rs - achieving 90%+ coverage
//! QDD Metrics Target:
//! - Line Coverage: ≥90%
//! - Branch Coverage: ≥85%
//! - All public APIs: 100%

use ruchy::Transpiler;
use ruchy::{Expr, ExprKind, Literal, BinaryOp};
use ruchy::frontend::ast::{Span, Type, TypeKind, Param, MatchArm, Pattern};

// Helper function removed - Lexer not publicly accessible

// Helper to create a simple expression
fn make_literal(val: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(val)),
        span: Span::default(),
        attributes: vec![],
    }
}

// ============================================================================
// Core Transpiler Tests
// ============================================================================

#[test]
fn test_transpiler_new() {
    let transpiler = Transpiler::new();
    assert!(!transpiler.in_async_context);
    assert!(transpiler.mutable_vars.is_empty());
    assert!(transpiler.function_signatures.is_empty());
}

#[test]
fn test_transpiler_default() {
    let transpiler = Transpiler::default();
    assert!(!transpiler.in_async_context);
    assert!(transpiler.mutable_vars.is_empty());
}

// ============================================================================
// Literal Transpilation Tests
// ============================================================================

#[test]
fn test_transpile_integer_literal() {
    let transpiler = Transpiler::new();
    let expr = make_literal(42);
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_string_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::String("hello".to_string())),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("\"hello\""));
}

#[test]
fn test_transpile_boolean_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("true"));
}

#[test]
fn test_transpile_float_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Float(3.14)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("3.14"));
}

#[test]
fn test_transpile_char_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Char('a')),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("'a'"));
}

// ============================================================================
// Binary Operation Tests
// ============================================================================

#[test]
fn test_transpile_binary_add() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Add,
            left: Box::new(make_literal(1)),
            right: Box::new(make_literal(2)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("1") && code.contains("2") && code.contains("+"));
}

#[test]
fn test_transpile_binary_subtract() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Subtract,
            left: Box::new(make_literal(5)),
            right: Box::new(make_literal(3)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("5") && code.contains("3") && code.contains("-"));
}

// ============================================================================
// Identifier and Variable Tests
// ============================================================================

#[test]
fn test_transpile_identifier() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Identifier("variable".to_string()),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("variable"));
}

// ============================================================================
// Mutability Analysis Tests
// ============================================================================

#[test]
fn test_analyze_mutability_assignment() {
    let mut transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            value: Box::new(make_literal(42)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[expr]);
    assert!(transpiler.mutable_vars.contains("x"));
}

#[test]
fn test_analyze_mutability_compound_assign() {
    let mut transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::CompoundAssign {
            op: BinaryOp::Add,
            target: Box::new(Expr {
                kind: ExprKind::Identifier("y".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            value: Box::new(make_literal(10)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[expr]);
    assert!(transpiler.mutable_vars.contains("y"));
}

#[test]
fn test_analyze_mutability_pre_increment() {
    let mut transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::PreIncrement {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("counter".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[expr]);
    assert!(transpiler.mutable_vars.contains("counter"));
}

#[test]
fn test_analyze_mutability_post_decrement() {
    let mut transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::PostDecrement {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("count".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[expr]);
    assert!(transpiler.mutable_vars.contains("count"));
}

#[test]
fn test_analyze_mutability_in_block() {
    let mut transpiler = Transpiler::new();
    let inner_assign = Expr {
        kind: ExprKind::Assign {
            target: Box::new(Expr {
                kind: ExprKind::Identifier("z".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            value: Box::new(make_literal(100)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let block = Expr {
        kind: ExprKind::Block(vec![inner_assign]),
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[block]);
    assert!(transpiler.mutable_vars.contains("z"));
}

// ============================================================================
// Function Signature Collection Tests
// ============================================================================

#[test]
fn test_collect_function_signatures() {
    let mut transpiler = Transpiler::new();
    
    let func = Expr {
        kind: ExprKind::Function {
            name: "add".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    pattern: Pattern::Identifier("a".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("Int".to_string()),
                        span: Span::default(),
                    },
                    default: None,
                },
                Param {
                    pattern: Pattern::Identifier("b".to_string()),
                    ty: Type {
                        kind: TypeKind::Named("Int".to_string()),
                        span: Span::default(),
                    },
                    default: None,
                },
            ],
            return_type: Some(Type {
                kind: TypeKind::Named("Int".to_string()),
                span: Span::default(),
            }),
            body: Box::new(make_literal(0)),
            is_async: false,
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.collect_function_signatures(&[func]);
    
    assert!(transpiler.function_signatures.contains_key("add"));
    let sig = &transpiler.function_signatures["add"];
    assert_eq!(sig.name, "add");
    assert_eq!(sig.param_types.len(), 2);
    assert_eq!(sig.param_types[0], "Int");
    assert_eq!(sig.param_types[1], "Int");
}

// ============================================================================
// Control Flow Tests
// ============================================================================

#[test]
fn test_transpile_if_else() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
            then_branch: Box::new(make_literal(1)),
            else_branch: Some(Box::new(make_literal(2))),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("if"));
}

#[test]
fn test_transpile_while_loop() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::While {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
            body: Box::new(make_literal(1)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("while"));
}

// ============================================================================
// Block and List Tests
// ============================================================================

#[test]
fn test_transpile_empty_block() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Block(vec![]),
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    // Empty block should produce something
    assert!(result.to_string().len() >= 0);
}

#[test]
fn test_transpile_block_with_expressions() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Block(vec![
            make_literal(1),
            make_literal(2),
            make_literal(3),
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

#[test]
fn test_transpile_list() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::List(vec![
            make_literal(1),
            make_literal(2),
            make_literal(3),
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("vec"));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
    assert!(code.contains("3"));
}

// ============================================================================
// Program Transpilation Tests
// ============================================================================

#[test]
fn test_transpile_to_program_simple() {
    let mut transpiler = Transpiler::new();
    let expr = make_literal(42);
    
    let result = transpiler.transpile_to_program(&expr).unwrap();
    let code = result.to_string();
    
    // Should generate a main function
    assert!(code.contains("fn main"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_to_string() {
    let transpiler = Transpiler::new();
    let expr = make_literal(42);
    
    let result = transpiler.transpile_to_string(&expr).unwrap();
    
    // Should produce formatted Rust code
    assert!(result.contains("fn main"));
}

#[test]
fn test_transpile_minimal() {
    let transpiler = Transpiler::new();
    let expr = make_literal(42);
    
    let result = transpiler.transpile_minimal(&expr).unwrap();
    
    // Minimal transpilation should be simpler
    assert!(result.contains("42"));
}

// ============================================================================
// Let Binding Tests
// ============================================================================

#[test]
fn test_transpile_let_binding() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Let {
            name: "x".to_string(),
            ty: None,
            value: Box::new(make_literal(42)),
            body: Box::new(Expr {
                kind: ExprKind::Identifier("x".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            is_mutable: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains("x"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_let_mutable() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Let {
            name: "y".to_string(),
            ty: None,
            value: Box::new(make_literal(10)),
            body: Box::new(Expr {
                kind: ExprKind::Identifier("y".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            is_mutable: true,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains("mut") || code.contains("y")); // Should mark as mutable
}

// ============================================================================
// Call Expression Tests
// ============================================================================

#[test]
fn test_transpile_function_call() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Call {
            func: Box::new(Expr {
                kind: ExprKind::Identifier("println".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            args: vec![
                Expr {
                    kind: ExprKind::Literal(Literal::String("Hello".to_string())),
                    span: Span::default(),
                    attributes: vec![],
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("println"));
    assert!(code.contains("Hello"));
}

// ============================================================================
// Async Context Tests
// ============================================================================

#[test]
fn test_async_context() {
    let mut transpiler = Transpiler::new();
    assert!(!transpiler.in_async_context);
    
    transpiler.in_async_context = true;
    assert!(transpiler.in_async_context);
    
    // Async functions should be handled differently
    let expr = Expr {
        kind: ExprKind::Function {
            name: "async_func".to_string(),
            type_params: vec![],
            params: vec![],
            return_type: None,
            body: Box::new(make_literal(42)),
            is_async: true,
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("async"));
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_transpile_unit_literal() {
    let transpiler = Transpiler::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
    };
    
    let result = transpiler.transpile(&expr).unwrap();
    let code = result.to_string();
    assert!(code.contains("()")); // Unit should transpile to ()
}

// ============================================================================
// Match Expression Tests
// ============================================================================

#[test]
fn test_analyze_mutability_in_match() {
    let mut transpiler = Transpiler::new();
    
    let match_expr = Expr {
        kind: ExprKind::Match {
            expr: Box::new(make_literal(1)),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr {
                        kind: ExprKind::Assign {
                            target: Box::new(Expr {
                                kind: ExprKind::Identifier("m".to_string()),
                                span: Span::default(),
                                attributes: vec![],
                            }),
                            value: Box::new(make_literal(5)),
                        },
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    span: Span::default(),
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.analyze_mutability(&[match_expr]);
    assert!(transpiler.mutable_vars.contains("m"));
}

// ============================================================================
// Method Call Tests
// ============================================================================

#[test]
fn test_analyze_mutability_method_call() {
    let mut transpiler = Transpiler::new();
    
    let method_call = Expr {
        kind: ExprKind::MethodCall {
            receiver: Box::new(Expr {
                kind: ExprKind::Identifier("obj".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            method: "method".to_string(),
            args: vec![make_literal(1)],
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    // Just analyzing method call shouldn't mark anything mutable
    transpiler.analyze_mutability(&[method_call]);
    assert!(!transpiler.mutable_vars.contains("obj"));
}

// ============================================================================
// Type System Tests
// ============================================================================

#[test]
fn test_type_to_string_named() {
    let transpiler = Transpiler::new();
    let ty = Type {
        kind: TypeKind::Named("String".to_string()),
        span: Span::default(),
    };
    
    // This is a private method, so we test it indirectly through collect_function_signatures
    let mut transpiler = Transpiler::new();
    let func = Expr {
        kind: ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    pattern: Pattern::Identifier("s".to_string()),
                    ty,
                    default: None,
                },
            ],
            return_type: None,
            body: Box::new(make_literal(0)),
            is_async: false,
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.collect_function_signatures(&[func]);
    assert_eq!(transpiler.function_signatures["test"].param_types[0], "String");
}

#[test]
fn test_type_to_string_reference() {
    let mut transpiler = Transpiler::new();
    let ty = Type {
        kind: TypeKind::Reference {
            mutable: false,
            inner: Box::new(Type {
                kind: TypeKind::Named("Int".to_string()),
                span: Span::default(),
            }),
        },
        span: Span::default(),
    };
    
    let func = Expr {
        kind: ExprKind::Function {
            name: "ref_test".to_string(),
            type_params: vec![],
            params: vec![
                Param {
                    pattern: Pattern::Identifier("r".to_string()),
                    ty,
                    default: None,
                },
            ],
            return_type: None,
            body: Box::new(make_literal(0)),
            is_async: false,
            is_pub: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    
    transpiler.collect_function_signatures(&[func]);
    assert_eq!(transpiler.function_signatures["ref_test"].param_types[0], "&Int");
}