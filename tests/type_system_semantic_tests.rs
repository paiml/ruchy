// TYPE SYSTEM & SEMANTIC ANALYZER TESTS
// Sprint 80 Phase 29: Deep type checking coverage
// ALL NIGHT MARATHON - NO STOPPING!

use ruchy::frontend::type_checker::{TypeChecker, Type, TypeEnvironment};
use ruchy::frontend::semantic_analyzer::SemanticAnalyzer;
use ruchy::frontend::ast::*;
use ruchy::Parser;
use std::collections::HashMap;

// Type system tests
#[test]
fn test_type_checker_new() {
    let checker = TypeChecker::new();
    let _ = checker;
}

#[test]
fn test_type_checker_default() {
    let checker = TypeChecker::default();
    let _ = checker;
}

#[test]
fn test_type_integer() {
    let t = Type::Integer;
    assert!(matches!(t, Type::Integer));
}

#[test]
fn test_type_float() {
    let t = Type::Float;
    assert!(matches!(t, Type::Float));
}

#[test]
fn test_type_string() {
    let t = Type::String;
    assert!(matches!(t, Type::String));
}

#[test]
fn test_type_bool() {
    let t = Type::Bool;
    assert!(matches!(t, Type::Bool));
}

#[test]
fn test_type_unit() {
    let t = Type::Unit;
    assert!(matches!(t, Type::Unit));
}

#[test]
fn test_type_list() {
    let t = Type::List(Box::new(Type::Integer));
    assert!(matches!(t, Type::List(_)));
}

#[test]
fn test_type_tuple() {
    let t = Type::Tuple(vec![Type::Integer, Type::String]);
    assert!(matches!(t, Type::Tuple(_)));
}

#[test]
fn test_type_function() {
    let t = Type::Function(
        vec![Type::Integer, Type::Integer],
        Box::new(Type::Integer)
    );
    assert!(matches!(t, Type::Function(_, _)));
}

#[test]
fn test_type_option() {
    let t = Type::Option(Box::new(Type::Integer));
    assert!(matches!(t, Type::Option(_)));
}

#[test]
fn test_type_result() {
    let t = Type::Result(
        Box::new(Type::Integer),
        Box::new(Type::String)
    );
    assert!(matches!(t, Type::Result(_, _)));
}

#[test]
fn test_type_any() {
    let t = Type::Any;
    assert!(matches!(t, Type::Any));
}

#[test]
fn test_type_never() {
    let t = Type::Never;
    assert!(matches!(t, Type::Never));
}

#[test]
fn test_type_generic() {
    let t = Type::Generic("T".to_string());
    assert!(matches!(t, Type::Generic(_)));
}

#[test]
fn test_type_struct() {
    let mut fields = HashMap::new();
    fields.insert("x".to_string(), Type::Integer);
    fields.insert("y".to_string(), Type::Integer);
    let t = Type::Struct("Point".to_string(), fields);
    assert!(matches!(t, Type::Struct(_, _)));
}

#[test]
fn test_type_enum() {
    let mut variants = HashMap::new();
    variants.insert("Some".to_string(), Some(Type::Integer));
    variants.insert("None".to_string(), None);
    let t = Type::Enum("Option".to_string(), variants);
    assert!(matches!(t, Type::Enum(_, _)));
}

#[test]
fn test_type_alias() {
    let t = Type::Alias(
        "MyInt".to_string(),
        Box::new(Type::Integer)
    );
    assert!(matches!(t, Type::Alias(_, _)));
}

#[test]
fn test_type_environment_new() {
    let env = TypeEnvironment::new();
    let _ = env;
}

#[test]
fn test_type_environment_default() {
    let env = TypeEnvironment::default();
    let _ = env;
}

#[test]
fn test_type_environment_define() {
    let mut env = TypeEnvironment::new();
    env.define("x", Type::Integer);
    assert_eq!(env.lookup("x"), Some(&Type::Integer));
}

#[test]
fn test_type_environment_lookup() {
    let mut env = TypeEnvironment::new();
    env.define("x", Type::Integer);
    assert_eq!(env.lookup("x"), Some(&Type::Integer));
    assert_eq!(env.lookup("y"), None);
}

#[test]
fn test_type_environment_push_scope() {
    let mut env = TypeEnvironment::new();
    env.push_scope();
    env.define("x", Type::Integer);
    assert_eq!(env.lookup("x"), Some(&Type::Integer));
}

#[test]
fn test_type_environment_pop_scope() {
    let mut env = TypeEnvironment::new();
    env.define("outer", Type::Integer);
    env.push_scope();
    env.define("inner", Type::String);
    assert_eq!(env.lookup("inner"), Some(&Type::String));
    env.pop_scope();
    assert_eq!(env.lookup("inner"), None);
    assert_eq!(env.lookup("outer"), Some(&Type::Integer));
}

#[test]
fn test_type_checker_infer_literal_integer() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Integer);
}

#[test]
fn test_type_checker_infer_literal_float() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Float(3.14)),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Float);
}

#[test]
fn test_type_checker_infer_literal_string() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::String("hello".to_string())),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::String);
}

#[test]
fn test_type_checker_infer_literal_bool() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Bool(true)),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Bool);
}

#[test]
fn test_type_checker_infer_literal_unit() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Unit);
}

#[test]
fn test_type_checker_infer_binary_add() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span::default(),
                attributes: vec![],
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(2)),
                span: Span::default(),
                attributes: vec![],
            }),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert_eq!(ty.unwrap(), Type::Integer);
}

#[test]
fn test_type_checker_infer_list() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::List(vec![
            Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span::default(),
                attributes: vec![],
            },
            Expr {
                kind: ExprKind::Literal(Literal::Integer(2)),
                span: Span::default(),
                attributes: vec![],
            },
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::List(_)));
}

#[test]
fn test_type_checker_infer_tuple() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::Tuple(vec![
            Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span::default(),
                attributes: vec![],
            },
            Expr {
                kind: ExprKind::Literal(Literal::String("hello".to_string())),
                span: Span::default(),
                attributes: vec![],
            },
        ]),
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
    assert!(matches!(ty.unwrap(), Type::Tuple(_)));
}

#[test]
fn test_type_checker_infer_if() {
    let mut checker = TypeChecker::new();
    let expr = Expr {
        kind: ExprKind::If {
            condition: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Bool(true)),
                span: Span::default(),
                attributes: vec![],
            }),
            then_branch: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span::default(),
                attributes: vec![],
            }),
            else_branch: Some(Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(2)),
                span: Span::default(),
                attributes: vec![],
            })),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let ty = checker.infer(&expr);
    assert!(ty.is_ok());
}

#[test]
fn test_type_checker_unify() {
    let mut checker = TypeChecker::new();
    assert!(checker.unify(Type::Integer, Type::Integer).is_ok());
    assert!(checker.unify(Type::Float, Type::Float).is_ok());
    assert!(checker.unify(Type::String, Type::String).is_ok());
    assert!(checker.unify(Type::Bool, Type::Bool).is_ok());
    assert!(checker.unify(Type::Unit, Type::Unit).is_ok());
}

#[test]
fn test_type_checker_unify_mismatch() {
    let mut checker = TypeChecker::new();
    assert!(checker.unify(Type::Integer, Type::String).is_err());
    assert!(checker.unify(Type::Float, Type::Bool).is_err());
}

#[test]
fn test_type_checker_unify_generic() {
    let mut checker = TypeChecker::new();
    let result = checker.unify(
        Type::Generic("T".to_string()),
        Type::Integer
    );
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_unify_list() {
    let mut checker = TypeChecker::new();
    let result = checker.unify(
        Type::List(Box::new(Type::Integer)),
        Type::List(Box::new(Type::Integer))
    );
    assert!(result.is_ok());
}

// Semantic analyzer tests
#[test]
fn test_semantic_analyzer_new() {
    let analyzer = SemanticAnalyzer::new();
    let _ = analyzer;
}

#[test]
fn test_semantic_analyzer_default() {
    let analyzer = SemanticAnalyzer::default();
    let _ = analyzer;
}

#[test]
fn test_semantic_analyzer_analyze_literal() {
    let mut analyzer = SemanticAnalyzer::new();
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_analyze_identifier() {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.define_variable("x", Type::Integer);
    let expr = Expr {
        kind: ExprKind::Identifier("x".to_string()),
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_undefined_identifier() {
    let mut analyzer = SemanticAnalyzer::new();
    let expr = Expr {
        kind: ExprKind::Identifier("undefined".to_string()),
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_err());
}

#[test]
fn test_semantic_analyzer_analyze_binary() {
    let mut analyzer = SemanticAnalyzer::new();
    let expr = Expr {
        kind: ExprKind::Binary {
            left: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span::default(),
                attributes: vec![],
            }),
            op: BinaryOp::Add,
            right: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(2)),
                span: Span::default(),
                attributes: vec![],
            }),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_analyze_let() {
    let mut analyzer = SemanticAnalyzer::new();
    let expr = Expr {
        kind: ExprKind::Let {
            pattern: Pattern::Identifier("x".to_string()),
            value: Box::new(Expr {
                kind: ExprKind::Literal(Literal::Integer(42)),
                span: Span::default(),
                attributes: vec![],
            }),
            type_annotation: None,
            body: None,
            mutable: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_analyze_function() {
    let mut analyzer = SemanticAnalyzer::new();
    let expr = Expr {
        kind: ExprKind::Function {
            name: "add".to_string(),
            params: vec![
                (Pattern::Identifier("x".to_string()), None),
                (Pattern::Identifier("y".to_string()), None),
            ],
            body: Box::new(Expr {
                kind: ExprKind::Binary {
                    left: Box::new(Expr {
                        kind: ExprKind::Identifier("x".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                    op: BinaryOp::Add,
                    right: Box::new(Expr {
                        kind: ExprKind::Identifier("y".to_string()),
                        span: Span::default(),
                        attributes: vec![],
                    }),
                },
                span: Span::default(),
                attributes: vec![],
            }),
            return_type: None,
            generic_params: vec![],
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_analyze_call() {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.define_function(
        "add",
        Type::Function(
            vec![Type::Integer, Type::Integer],
            Box::new(Type::Integer)
        )
    );
    let expr = Expr {
        kind: ExprKind::Call {
            callee: Box::new(Expr {
                kind: ExprKind::Identifier("add".to_string()),
                span: Span::default(),
                attributes: vec![],
            }),
            args: vec![
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(1)),
                    span: Span::default(),
                    attributes: vec![],
                },
                Expr {
                    kind: ExprKind::Literal(Literal::Integer(2)),
                    span: Span::default(),
                    attributes: vec![],
                },
            ],
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = analyzer.analyze(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_semantic_analyzer_scope_management() {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.push_scope();
    analyzer.define_variable("inner", Type::Integer);
    assert!(analyzer.lookup_variable("inner").is_some());
    analyzer.pop_scope();
    assert!(analyzer.lookup_variable("inner").is_none());
}

#[test]
fn test_semantic_analyzer_duplicate_definition() {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.define_variable("x", Type::Integer);
    let result = analyzer.define_variable("x", Type::String);
    // Should handle duplicate gracefully
    let _ = result;
}

// Type equality and display tests
#[test]
fn test_type_equality() {
    assert_eq!(Type::Integer, Type::Integer);
    assert_eq!(Type::Float, Type::Float);
    assert_eq!(Type::String, Type::String);
    assert_eq!(Type::Bool, Type::Bool);
    assert_eq!(Type::Unit, Type::Unit);
    assert_ne!(Type::Integer, Type::Float);
    assert_ne!(Type::String, Type::Bool);
}

#[test]
fn test_type_display() {
    assert_eq!(format!("{:?}", Type::Integer), "Integer");
    assert_eq!(format!("{:?}", Type::Float), "Float");
    assert_eq!(format!("{:?}", Type::String), "String");
    assert_eq!(format!("{:?}", Type::Bool), "Bool");
    assert_eq!(format!("{:?}", Type::Unit), "Unit");
}

#[test]
fn test_type_clone() {
    let t = Type::Integer;
    let cloned = t.clone();
    assert_eq!(t, cloned);
}

// ALL NIGHT continues...
