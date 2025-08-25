#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::approx_constant)]
//! Comprehensive tests for the transpiler dispatcher module
//!
//! This test suite provides extensive coverage for the dispatcher module,
//! focusing on delegated transpilation functions.

#![allow(clippy::unwrap_used)]  // Tests are allowed to use unwrap
#![allow(clippy::approx_constant)]  // Tests use PI values for testing

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, 
    MatchArm, Param, Pattern, PipelineStage, Span, Type, TypeKind, UnaryOp
};

/// Helper function to create test expressions
fn create_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::new(0, 10))
}

/// Test transpiling basic expressions (literals, identifiers)
#[test]
fn test_transpile_basic_expressions() {
    let transpiler = Transpiler::new();
    
    // Integer literal
    let int_expr = create_expr(ExprKind::Literal(Literal::Integer(42)));
    let result = transpiler.transpile_expr(&int_expr).unwrap();
    assert_eq!(result.to_string(), "42i32");
    
    // Float literal
    let float_expr = create_expr(ExprKind::Literal(Literal::Float(3.14159265)));
    let result = transpiler.transpile_expr(&float_expr).unwrap();
    assert_eq!(result.to_string(), "3.14159265f64");
    
    // Boolean literal
    let bool_expr = create_expr(ExprKind::Literal(Literal::Bool(true)));
    let result = transpiler.transpile_expr(&bool_expr).unwrap();
    assert_eq!(result.to_string(), "true");
    
    // String literal
    let string_expr = create_expr(ExprKind::Literal(Literal::String("hello".to_string())));
    let result = transpiler.transpile_expr(&string_expr).unwrap();
    assert_eq!(result.to_string(), "\"hello\"");
    
    // Identifier
    let ident_expr = create_expr(ExprKind::Identifier("variable".to_string()));
    let result = transpiler.transpile_expr(&ident_expr).unwrap();
    assert_eq!(result.to_string(), "variable");
}

/// Test transpiling qualified names
#[test]
fn test_transpile_qualified_names() {
    let transpiler = Transpiler::new();
    
    let qualified_expr = create_expr(ExprKind::QualifiedName {
        module: "std".to_string(),
        name: "println".to_string(),
    });
    let result = transpiler.transpile_expr(&qualified_expr).unwrap();
    assert_eq!(result.to_string(), "std :: println");
}

/// Test transpiling binary operations
#[test]
fn test_transpile_binary_operations() {
    let transpiler = Transpiler::new();
    
    // Addition
    let add_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        op: BinaryOp::Add,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(20)))),
    });
    let result = transpiler.transpile_expr(&add_expr).unwrap();
    assert!(result.to_string().contains("10") && result.to_string().contains("20"));
    
    // Multiplication
    let mul_expr = create_expr(ExprKind::Binary {
        left: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        op: BinaryOp::Multiply,
        right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(6)))),
    });
    let result = transpiler.transpile_expr(&mul_expr).unwrap();
    assert!(result.to_string().contains('5') && result.to_string().contains('6'));
}

/// Test transpiling unary operations
#[test]
fn test_transpile_unary_operations() {
    let transpiler = Transpiler::new();
    
    // Negation
    let neg_expr = create_expr(ExprKind::Unary {
        op: UnaryOp::Negate,
        operand: Box::new(create_expr(ExprKind::Literal(Literal::Integer(42)))),
    });
    let result = transpiler.transpile_expr(&neg_expr).unwrap();
    assert!(result.to_string().contains("42"));
    
    // Logical NOT
    let not_expr = create_expr(ExprKind::Unary {
        op: UnaryOp::Not,
        operand: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
    });
    let result = transpiler.transpile_expr(&not_expr).unwrap();
    assert!(result.to_string().contains("true"));
}

/// Test transpiling if expressions
#[test]
fn test_transpile_if_expressions() {
    let transpiler = Transpiler::new();
    
    let if_expr = create_expr(ExprKind::If {
        condition: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
        then_branch: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        else_branch: Some(Box::new(create_expr(ExprKind::Literal(Literal::Integer(20))))),
    });
    let result = transpiler.transpile_expr(&if_expr).unwrap();
    assert!(result.to_string().contains("if"));
    assert!(result.to_string().contains("true"));
}

/// Test transpiling match expressions
#[test]
fn test_transpile_match_expressions() {
    let transpiler = Transpiler::new();
    
    let match_expr = create_expr(ExprKind::Match {
        expr: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
        arms: vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String("one".to_string())))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(create_expr(ExprKind::Literal(Literal::String("other".to_string())))),
                span: Span::new(0, 10),
            },
        ],
    });
    let result = transpiler.transpile_expr(&match_expr).unwrap();
    assert!(result.to_string().contains("match"));
}

/// Test transpiling for loops
#[test]
fn test_transpile_for_loops() {
    let transpiler = Transpiler::new();
    
    let for_expr = create_expr(ExprKind::For {
        var: "i".to_string(),
        pattern: Some(Pattern::Identifier("i".to_string())),
        iter: Box::new(create_expr(ExprKind::Range {
            start: Box::new(create_expr(ExprKind::Literal(Literal::Integer(0)))),
            end: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
            inclusive: false,
        })),
        body: Box::new(create_expr(ExprKind::Block(vec![]))),
    });
    let result = transpiler.transpile_expr(&for_expr).unwrap();
    assert!(result.to_string().contains("for"));
}

/// Test transpiling while loops
#[test]
fn test_transpile_while_loops() {
    let transpiler = Transpiler::new();
    
    let while_expr = create_expr(ExprKind::While {
        condition: Box::new(create_expr(ExprKind::Literal(Literal::Bool(true)))),
        body: Box::new(create_expr(ExprKind::Block(vec![]))),
    });
    let result = transpiler.transpile_expr(&while_expr).unwrap();
    assert!(result.to_string().contains("while"));
}

/// Test transpiling function definitions
#[test]
fn test_transpile_function_definitions() {
    let transpiler = Transpiler::new();
    
    let func_expr = create_expr(ExprKind::Function {
        name: "test_func".to_string(),
        type_params: vec![],
        params: vec![
            Param {
                pattern: Pattern::Identifier("x".to_string()),
                ty: Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 10) },
                span: Span::new(0, 10),
                is_mutable: false,
                default_value: None,
            },
        ],
        body: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
        is_async: false,
        return_type: Some(Type { kind: TypeKind::Named("i32".to_string()), span: Span::new(0, 10) }),
        is_pub: false,
    });
    let result = transpiler.transpile_expr(&func_expr).unwrap();
    assert!(result.to_string().contains("fn"));
    assert!(result.to_string().contains("test_func"));
}

/// Test transpiling lambda expressions
#[test]
fn test_transpile_lambda_expressions() {
    let transpiler = Transpiler::new();
    
    let lambda_expr = create_expr(ExprKind::Lambda {
        params: vec![
            Param {
                pattern: Pattern::Identifier("x".to_string()),
                ty: Type { kind: TypeKind::Named("_".to_string()), span: Span::new(0, 10) },
                span: Span::new(0, 10),
                is_mutable: false,
                default_value: None,
            },
        ],
        body: Box::new(create_expr(ExprKind::Binary {
            left: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
            op: BinaryOp::Add,
            right: Box::new(create_expr(ExprKind::Literal(Literal::Integer(1)))),
        })),
    });
    let result = transpiler.transpile_expr(&lambda_expr).unwrap();
    assert!(result.to_string().contains('|'));
}

/// Test transpiling function calls
#[test]
fn test_transpile_function_calls() {
    let transpiler = Transpiler::new();
    
    let call_expr = create_expr(ExprKind::Call {
        func: Box::new(create_expr(ExprKind::Identifier("println".to_string()))),
        args: vec![
            create_expr(ExprKind::Literal(Literal::String("Hello".to_string()))),
        ],
    });
    let result = transpiler.transpile_expr(&call_expr).unwrap();
    assert!(result.to_string().contains("println"));
}

/// Test transpiling macro expressions (QUALITY-013 Refactored)
///
/// This test verifies that our macro refactoring from complexity 73 to <10 
/// maintains correct transpilation behavior for all macro categories:
/// - Print macros: println, print, panic
/// - Collection macros: vec
/// - Assertion macros: assert, assert_eq, assert_ne
#[test]
fn test_transpile_macro_expressions() {
    let transpiler = Transpiler::new();
    
    // Test print macros category
    test_print_macros(&transpiler);
    test_collection_macros(&transpiler);
    test_assertion_macros(&transpiler);
}

/// Test print-style macros (println, print, panic) after refactoring
fn test_print_macros(transpiler: &Transpiler) {
    // println macro with string literal
    let println_expr = create_expr(ExprKind::Macro {
        name: "println".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::String("Test".to_string()))),
        ],
    });
    let result = transpiler.transpile_expr(&println_expr).unwrap();
    assert!(result.to_string().contains("println!"));
    assert!(result.to_string().contains("Test"));
    
    // print macro with no args
    let print_expr = create_expr(ExprKind::Macro {
        name: "print".to_string(),
        args: vec![],
    });
    let result = transpiler.transpile_expr(&print_expr).unwrap();
    assert!(result.to_string().contains("print!()"));
    
    // panic macro with string literal
    let panic_expr = create_expr(ExprKind::Macro {
        name: "panic".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::String("Error".to_string()))),
        ],
    });
    let result = transpiler.transpile_expr(&panic_expr).unwrap();
    assert!(result.to_string().contains("panic!"));
    assert!(result.to_string().contains("Error"));
}

/// Test collection-style macros (vec) after refactoring  
fn test_collection_macros(transpiler: &Transpiler) {
    // vec macro with multiple elements
    let vec_expr = create_expr(ExprKind::Macro {
        name: "vec".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(2))),
            create_expr(ExprKind::Literal(Literal::Integer(3))),
        ],
    });
    let result = transpiler.transpile_expr(&vec_expr).unwrap();
    assert!(result.to_string().contains("vec!"));
    assert!(result.to_string().contains("1"));
    assert!(result.to_string().contains("2"));
    assert!(result.to_string().contains("3"));
    
    // vec macro with no elements
    let empty_vec_expr = create_expr(ExprKind::Macro {
        name: "vec".to_string(),
        args: vec![],
    });
    let result = transpiler.transpile_expr(&empty_vec_expr).unwrap();
    assert!(result.to_string().contains("vec![]"));
}

/// Test assertion-style macros (assert, assert_eq, assert_ne) after refactoring
fn test_assertion_macros(transpiler: &Transpiler) {
    // assert macro with boolean expression
    let assert_expr = create_expr(ExprKind::Macro {
        name: "assert".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Bool(true))),
        ],
    });
    let result = transpiler.transpile_expr(&assert_expr).unwrap();
    assert!(result.to_string().contains("assert!"));
    assert!(result.to_string().contains("true"));
    
    // assert_eq macro with two arguments
    let assert_eq_expr = create_expr(ExprKind::Macro {
        name: "assert_eq".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(1))),
        ],
    });
    let result = transpiler.transpile_expr(&assert_eq_expr).unwrap();
    assert!(result.to_string().contains("assert_eq!"));
    
    // assert_ne macro with two arguments
    let assert_ne_expr = create_expr(ExprKind::Macro {
        name: "assert_ne".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            create_expr(ExprKind::Literal(Literal::Integer(2))),
        ],
    });
    let result = transpiler.transpile_expr(&assert_ne_expr).unwrap();
    assert!(result.to_string().contains("assert_ne!"));
}

/// Test macro validation - assert_eq requires >= 2 arguments  
#[test]
fn test_macro_validation() {
    let transpiler = Transpiler::new();
    
    // assert_eq with insufficient arguments should fail
    let invalid_assert_eq = create_expr(ExprKind::Macro {
        name: "assert_eq".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            // Missing second argument
        ],
    });
    let result = transpiler.transpile_expr(&invalid_assert_eq);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires at least 2 arguments"));
    
    // assert_ne with insufficient arguments should fail
    let invalid_assert_ne = create_expr(ExprKind::Macro {
        name: "assert_ne".to_string(),
        args: vec![
            create_expr(ExprKind::Literal(Literal::Integer(1))),
            // Missing second argument
        ],
    });
    let result = transpiler.transpile_expr(&invalid_assert_ne);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires at least 2 arguments"));
    
    // Unknown macro should fail
    let unknown_macro = create_expr(ExprKind::Macro {
        name: "unknown_macro".to_string(),
        args: vec![],
    });
    let result = transpiler.transpile_expr(&unknown_macro);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unknown macro"));
}

/// Test transpiling struct literal expressions
#[test]
fn test_transpile_struct_literal() {
    let transpiler = Transpiler::new();
    
    let struct_literal = create_expr(ExprKind::StructLiteral {
        name: "TestStruct".to_string(),
        fields: vec![
            ("value".to_string(), create_expr(ExprKind::Literal(Literal::Integer(42)))),
        ],
    });
    let result = transpiler.transpile_expr(&struct_literal).unwrap();
    assert!(result.to_string().contains("TestStruct"));
    assert!(result.to_string().contains("value"));
}

/// Test transpiling list expressions
#[test]
fn test_transpile_list_expressions() {
    let transpiler = Transpiler::new();
    
    let list_expr = create_expr(ExprKind::List(vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
        create_expr(ExprKind::Literal(Literal::Integer(3))),
    ]));
    let result = transpiler.transpile_expr(&list_expr).unwrap();
    assert!(result.to_string().contains("vec"));
    assert!(result.to_string().contains('!'));
}

/// Test transpiling tuple expressions
#[test]
fn test_transpile_tuple_expressions() {
    let transpiler = Transpiler::new();
    
    let tuple_expr = create_expr(ExprKind::Tuple(vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::String("test".to_string()))),
    ]));
    let result = transpiler.transpile_expr(&tuple_expr).unwrap();
    assert!(result.to_string().contains('1'));
    assert!(result.to_string().contains("\"test\""));
}

/// Test transpiling range expressions
#[test]
fn test_transpile_range_expressions() {
    let transpiler = Transpiler::new();
    
    // Exclusive range
    let range_expr = create_expr(ExprKind::Range {
        start: Box::new(create_expr(ExprKind::Literal(Literal::Integer(0)))),
        end: Box::new(create_expr(ExprKind::Literal(Literal::Integer(10)))),
        inclusive: false,
    });
    let result = transpiler.transpile_expr(&range_expr).unwrap();
    assert!(result.to_string().contains('0'));
    assert!(result.to_string().contains("10"));
}

/// Test transpiling Result types
#[test]
fn test_transpile_result_types() {
    let transpiler = Transpiler::new();
    
    // Ok variant
    let ok_expr = create_expr(ExprKind::Ok {
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(42)))),
    });
    let result = transpiler.transpile_expr(&ok_expr).unwrap();
    assert!(result.to_string().contains("Ok"));
    assert!(result.to_string().contains("42"));
    
    // Err variant
    let err_expr = create_expr(ExprKind::Err {
        error: Box::new(create_expr(ExprKind::Literal(Literal::String("error".to_string())))),
    });
    let result = transpiler.transpile_expr(&err_expr).unwrap();
    assert!(result.to_string().contains("Err"));
}

/// Test transpiling Option types
#[test]
fn test_transpile_option_types() {
    let transpiler = Transpiler::new();
    
    // Some variant
    let some_expr = create_expr(ExprKind::Some {
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(42)))),
    });
    let result = transpiler.transpile_expr(&some_expr).unwrap();
    assert!(result.to_string().contains("Some"));
    assert!(result.to_string().contains("42"));
    
    // None variant
    let none_expr = create_expr(ExprKind::None);
    let result = transpiler.transpile_expr(&none_expr).unwrap();
    assert_eq!(result.to_string(), "None");
}

/// Test transpiling let bindings
#[test]
fn test_transpile_let_bindings() {
    let transpiler = Transpiler::new();
    
    let let_expr = create_expr(ExprKind::Let {
        name: "x".to_string(),
        type_annotation: None,
        value: Box::new(create_expr(ExprKind::Literal(Literal::Integer(42)))),
        body: Box::new(create_expr(ExprKind::Identifier("x".to_string()))),
        is_mutable: false,
    });
    let result = transpiler.transpile_expr(&let_expr).unwrap();
    assert!(result.to_string().contains("let"));
}

/// Test transpiling block expressions
#[test]
fn test_transpile_block_expressions() {
    let transpiler = Transpiler::new();
    
    let block_expr = create_expr(ExprKind::Block(vec![
        create_expr(ExprKind::Literal(Literal::Integer(1))),
        create_expr(ExprKind::Literal(Literal::Integer(2))),
    ]));
    let result = transpiler.transpile_expr(&block_expr).unwrap();
    assert!(result.to_string().contains('{'));
    assert!(result.to_string().contains('}'));
}

/// Test transpiling pipeline expressions
#[test]
fn test_transpile_pipeline_expressions() {
    let transpiler = Transpiler::new();
    
    let pipeline_expr = create_expr(ExprKind::Pipeline {
        expr: Box::new(create_expr(ExprKind::Literal(Literal::Integer(5)))),
        stages: vec![
            PipelineStage {
                op: Box::new(create_expr(ExprKind::Identifier("double".to_string()))),
                span: Span::new(0, 10),
            },
            PipelineStage {
                op: Box::new(create_expr(ExprKind::Identifier("increment".to_string()))),
                span: Span::new(0, 10),
            },
        ],
    });
    let result = transpiler.transpile_expr(&pipeline_expr).unwrap();
    // Pipeline should transpile to nested function calls
    assert!(result.to_string().contains("increment"));
    assert!(result.to_string().contains("double"));
}