//! Comprehensive TDD test suite for statements.rs transpiler module
//! Target: Transform 0% → 70%+ coverage via systematic testing
//! Toyota Way: Every statement type must be tested comprehensively

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Param, Pattern, PipelineStage, ImportItem, Span, Type, TypeKind, StructPatternField};
use ruchy::frontend::ast::BinaryOp;

/// Helper function to create a test transpiler
fn create_test_transpiler() -> Transpiler {
    Transpiler::new()
}

/// Helper to create a simple identifier expression
fn create_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::new(0, name.len()),
        attributes: vec![],
    }
}

/// Helper to create a literal integer expression
fn create_integer(value: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(value)),
        span: Span::new(0, 0),
        attributes: vec![],
    }
}

/// Helper to create a literal string expression
fn create_string(value: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(value.to_string())),
        span: Span::new(0, value.len()),
        attributes: vec![],
    }
}

/// Helper to create a block expression
fn create_block(exprs: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Block(exprs),
        span: Span::new(0, 0),
        attributes: vec![],
    }
}

/// Helper to create unit literal
fn create_unit() -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Unit),
        span: Span::new(0, 0),
        attributes: vec![],
    }
}

// ==================== IF STATEMENT TESTS ====================

#[test]
fn test_transpile_if_simple() {
    let transpiler = create_test_transpiler();
    let condition = create_identifier("condition");
    let then_branch = create_integer(42);
    
    let result = transpiler.transpile_if(&condition, &then_branch, None);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("if"));
    assert!(code.contains("condition"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_if_with_else() {
    let transpiler = create_test_transpiler();
    let condition = create_identifier("x");
    let then_branch = create_integer(1);
    let else_branch = create_integer(2);
    
    let result = transpiler.transpile_if(&condition, &then_branch, Some(&else_branch));
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("if"));
    assert!(code.contains("else"));
    assert!(code.contains("1"));
    assert!(code.contains("2"));
}

#[test]
fn test_transpile_if_complex_condition() {
    let transpiler = create_test_transpiler();
    let condition = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Greater,
            left: Box::new(create_identifier("x")),
            right: Box::new(create_integer(10)),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    let then_branch = create_string("greater");
    let else_branch = create_string("lesser");
    
    let result = transpiler.transpile_if(&condition, &then_branch, Some(&else_branch));
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("if"));
    assert!(code.contains(">"));
    assert!(code.contains("10"));
}

// ==================== LET BINDING TESTS ====================

#[test]
fn test_transpile_let_simple() {
    let transpiler = create_test_transpiler();
    let value = create_integer(42);
    let body = create_identifier("x");
    
    let result = transpiler.transpile_let("x", &value, &body, false);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let x"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_let_mutable() {
    let transpiler = create_test_transpiler();
    let value = create_integer(0);
    let body = create_identifier("counter");
    
    let result = transpiler.transpile_let("counter", &value, &body, true);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let mut counter"));
    assert!(code.contains("0"));
}

#[test]
fn test_transpile_let_with_string_coercion() {
    let transpiler = create_test_transpiler();
    let value = create_string("hello");
    let body = create_identifier("msg");
    
    let result = transpiler.transpile_let("msg", &value, &body, false);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let msg"));
    assert!(code.contains("to_string"));
}

#[test]
fn test_transpile_let_reserved_keyword() {
    let transpiler = create_test_transpiler();
    let value = create_integer(1);
    let body = create_identifier("match");
    
    let result = transpiler.transpile_let("match", &value, &body, false);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("r#match"));
}

#[test]
fn test_transpile_let_top_level() {
    let transpiler = create_test_transpiler();
    let value = create_integer(100);
    let body = create_unit(); // Unit body means top-level let
    
    let result = transpiler.transpile_let("global", &value, &body, false);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let global"));
    assert!(!code.contains("{")); // No scoping block
}

// ==================== LET PATTERN TESTS ====================

#[test]
fn test_transpile_let_pattern_tuple() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string()),
    ]);
    let value = create_identifier("pair");
    let body = create_identifier("x");
    
    let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let"));
    assert!(code.contains("("));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
}

#[test]
fn test_transpile_let_pattern_struct() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            StructPatternField {
                name: "x".to_string(),
                pattern: Some(Pattern::Identifier("px".to_string())),
            },
            StructPatternField {
                name: "y".to_string(),
                pattern: Some(Pattern::Identifier("py".to_string())),
            },
        ],
        has_rest: false,
    };
    let value = create_identifier("point");
    let body = create_identifier("px");
    
    let result = transpiler.transpile_let_pattern(&pattern, &value, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("let"));
    assert!(code.contains("Point"));
}

// ==================== FUNCTION TESTS ====================

#[test]
fn test_transpile_function_simple() {
    let transpiler = create_test_transpiler();
    let body = create_integer(42);
    
    let result = transpiler.transpile_function(
        "get_answer",
        &[],
        &[],
        &body,
        false,
        None,
        false,
        &[]
    );
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("fn get_answer"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_function_with_params() {
    let transpiler = create_test_transpiler();
    let params = vec![
        Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
        Param {
            pattern: Pattern::Identifier("y".to_string()),
            ty: Type {
                kind: TypeKind::Named("i32".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
    ];
    let body = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Add,
            left: Box::new(create_identifier("x")),
            right: Box::new(create_identifier("y")),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    
    let return_type = Type {
        kind: TypeKind::Named("i32".to_string()),
        span: Span::new(0, 0),
    };
    let result = transpiler.transpile_function(
        "add",
        &[],
        &params,
        &body,
        false,
        Some(&return_type),
        false,
        &[]
    );
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("fn add"));
    assert!(code.contains("x : i32"));
    assert!(code.contains("y : i32"));
    assert!(code.contains("-> i32"));
}

#[test]
fn test_transpile_function_generic() {
    let transpiler = create_test_transpiler();
    let type_params = vec!["T".to_string()];
    let params = vec![
        Param {
            pattern: Pattern::Identifier("value".to_string()),
            ty: Type {
                kind: TypeKind::Named("T".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
    ];
    let body = create_identifier("value");
    
    let return_type = Type {
        kind: TypeKind::Named("T".to_string()),
        span: Span::new(0, 0),
    };
    let result = transpiler.transpile_function(
        "identity",
        &type_params,
        &params,
        &body,
        false,
        Some(&return_type),
        false,
        &[]
    );
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("fn identity"));
    assert!(code.contains("< T >") || code.contains("<T>"));  // Account for formatting
}

// ==================== LAMBDA TESTS ====================

#[test]
fn test_transpile_lambda_simple() {
    let transpiler = create_test_transpiler();
    let params = vec![
        Param {
            pattern: Pattern::Identifier("x".to_string()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
    ];
    let body = create_identifier("x");
    
    let result = transpiler.transpile_lambda(&params, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("|"));
    assert!(code.contains("x"));
}

#[test]
fn test_transpile_lambda_multiple_params() {
    let transpiler = create_test_transpiler();
    let params = vec![
        Param {
            pattern: Pattern::Identifier("a".to_string()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
        Param {
            pattern: Pattern::Identifier("b".to_string()),
            ty: Type {
                kind: TypeKind::Named("_".to_string()),
                span: Span::new(0, 0),
            },
            span: Span::new(0, 0),
            is_mutable: false,
            default_value: None,
        },
    ];
    let body = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Add,
            left: Box::new(create_identifier("a")),
            right: Box::new(create_identifier("b")),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_lambda(&params, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("|"));
    assert!(code.contains("a"));
    assert!(code.contains("b"));
    assert!(code.contains("+"));
}

// ==================== CALL TESTS ====================

#[test]
fn test_transpile_call_simple() {
    let transpiler = create_test_transpiler();
    let func = create_identifier("print");
    let args = vec![create_string("Hello")];
    
    let result = transpiler.transpile_call(&func, &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("print"));
    assert!(code.contains("Hello"));
}

#[test]
fn test_transpile_call_builtin_println() {
    let transpiler = create_test_transpiler();
    let func = create_identifier("println");
    let args = vec![create_string("Debug")];
    
    let result = transpiler.transpile_call(&func, &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    // println is a regular function call, not a macro
    assert!(code.contains("println"));
}

#[test]
fn test_transpile_call_multiple_args() {
    let transpiler = create_test_transpiler();
    let func = create_identifier("max");
    let args = vec![create_integer(10), create_integer(20)];
    
    let result = transpiler.transpile_call(&func, &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("max"));
    assert!(code.contains("10"));
    assert!(code.contains("20"));
}

// ==================== BLOCK TESTS ====================

#[test]
fn test_transpile_block_empty() {
    let transpiler = create_test_transpiler();
    let exprs = vec![];
    
    let result = transpiler.transpile_block(&exprs);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("{"));
    assert!(code.contains("}"));
}

#[test]
fn test_transpile_block_single_expr() {
    let transpiler = create_test_transpiler();
    let exprs = vec![create_integer(42)];
    
    let result = transpiler.transpile_block(&exprs);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("{"));
    assert!(code.contains("42"));
    assert!(code.contains("}"));
}

#[test]
fn test_transpile_block_multiple_statements() {
    let transpiler = create_test_transpiler();
    let exprs = vec![
        create_identifier("x"),
        create_identifier("y"),
        create_integer(100),
    ];
    
    let result = transpiler.transpile_block(&exprs);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("{"));
    assert!(code.contains("x"));
    assert!(code.contains("y"));
    assert!(code.contains("100"));
}

// ==================== PIPELINE TESTS ====================

#[test]
fn test_transpile_pipeline_simple() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("data");
    let stages = vec![
        PipelineStage {
            op: Box::new(create_identifier("filter")),
            span: Span::new(0, 0),
        },
    ];
    
    let result = transpiler.transpile_pipeline(&expr, &stages);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("data"));
    assert!(code.contains("filter"));
}

#[test]
fn test_transpile_pipeline_chain() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("numbers");
    let stages = vec![
        PipelineStage {
            op: Box::new(create_identifier("map")),
            span: Span::new(0, 0),
        },
        PipelineStage {
            op: Box::new(create_identifier("filter")),
            span: Span::new(0, 0),
        },
    ];
    
    let result = transpiler.transpile_pipeline(&expr, &stages);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("numbers"));
    assert!(code.contains("map"));
    assert!(code.contains("filter"));
}

// ==================== FOR LOOP TESTS ====================

#[test]
fn test_transpile_for_simple() {
    let transpiler = create_test_transpiler();
    let iter = create_identifier("items");
    let body = create_identifier("x");
    
    let result = transpiler.transpile_for("x", None, &iter, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("for"));
    assert!(code.contains("x"));
    assert!(code.contains("in"));
    assert!(code.contains("items"));
}

#[test]
fn test_transpile_for_with_pattern() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("key".to_string()),
        Pattern::Identifier("value".to_string()),
    ]);
    let iter = create_identifier("map");
    let body = create_identifier("value");
    
    let result = transpiler.transpile_for("_", Some(&pattern), &iter, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("for"));
    assert!(code.contains("key"));
    assert!(code.contains("value"));
}

// ==================== WHILE LOOP TESTS ====================

#[test]
fn test_transpile_while_simple() {
    let transpiler = create_test_transpiler();
    let condition = create_identifier("running");
    let body = create_identifier("process");
    
    let result = transpiler.transpile_while(&condition, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("while"));
    assert!(code.contains("running"));
    assert!(code.contains("process"));
}

#[test]
fn test_transpile_while_complex_condition() {
    let transpiler = create_test_transpiler();
    let condition = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Less,
            left: Box::new(create_identifier("count")),
            right: Box::new(create_integer(10)),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    let body = create_block(vec![create_identifier("work")]);
    
    let result = transpiler.transpile_while(&condition, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("while"));
    assert!(code.contains("count"));
    assert!(code.contains("<"));
    assert!(code.contains("10"));
}

// ==================== IF-LET TESTS ====================

#[test]
fn test_transpile_if_let_some() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::QualifiedName(vec!["Some".to_string()]);
    let expr = create_identifier("maybe");
    let then_branch = create_identifier("x");
    let else_branch = Some(create_integer(0));
    
    let result = transpiler.transpile_if_let(&pattern, &expr, &then_branch, else_branch.as_ref());
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("if let"));
    assert!(code.contains("Some"));
    assert!(code.contains("maybe"));
}

// ==================== WHILE-LET TESTS ====================

#[test]
fn test_transpile_while_let() {
    let transpiler = create_test_transpiler();
    let pattern = Pattern::QualifiedName(vec!["Some".to_string()]);
    let expr = create_identifier("iterator");
    let body = create_identifier("item");
    
    let result = transpiler.transpile_while_let(&pattern, &expr, &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("while let"));
    assert!(code.contains("Some"));
    assert!(code.contains("item"));
}

// ==================== LOOP TESTS ====================

#[test]
fn test_transpile_loop() {
    let transpiler = create_test_transpiler();
    let body = create_block(vec![
        create_identifier("work"),
        Expr {
            kind: ExprKind::Break { label: None },
            span: Span::new(0, 0),
            attributes: vec![],
        },
    ]);
    
    let result = transpiler.transpile_loop(&body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("loop"));
    assert!(code.contains("work"));
}

// ==================== LIST COMPREHENSION TESTS ====================

#[test]
fn test_transpile_list_comprehension_simple() {
    let transpiler = create_test_transpiler();
    let expr = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(create_identifier("x")),
            right: Box::new(create_integer(2)),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    let iter = create_identifier("numbers");
    
    let result = transpiler.transpile_list_comprehension(&expr, "x", &iter, None);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("numbers"));
    assert!(code.contains("iter"));
    assert!(code.contains("map"));
}

#[test]
fn test_transpile_list_comprehension_with_filter() {
    let transpiler = create_test_transpiler();
    let expr = create_identifier("x");
    let iter = create_identifier("numbers");
    let filter = Expr {
        kind: ExprKind::Binary {
            op: BinaryOp::Greater,
            left: Box::new(create_identifier("x")),
            right: Box::new(create_integer(5)),
        },
        span: Span::new(0, 0),
        attributes: vec![],
    };
    
    let result = transpiler.transpile_list_comprehension(&expr, "x", &iter, Some(&filter));
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("filter"));
    assert!(code.contains("map"));
}

// ==================== MODULE TESTS ====================

#[test]
fn test_transpile_module() {
    let transpiler = create_test_transpiler();
    let body = create_block(vec![
        create_identifier("x"),
    ]);
    
    let result = transpiler.transpile_module("mymod", &body);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("mod mymod"));
}

// ==================== IMPORT TESTS ====================

#[test]
fn test_transpile_import_simple() {
    let items = vec![
        ImportItem::Named("HashMap".to_string()),
    ];
    
    let tokens = Transpiler::transpile_import("std::collections", &items);
    let code = tokens.to_string();
    
    assert!(code.contains("use"));
    assert!(code.contains("std"));
    assert!(code.contains("collections"));
    assert!(code.contains("HashMap"));
}

#[test]
fn test_transpile_import_multiple() {
    let items = vec![
        ImportItem::Named("Vec".to_string()),
        ImportItem::Named("HashMap".to_string()),
        ImportItem::Named("HashSet".to_string()),
    ];
    
    let tokens = Transpiler::transpile_import("std::collections", &items);
    let code = tokens.to_string();
    
    assert!(code.contains("Vec"));
    assert!(code.contains("HashMap"));
    assert!(code.contains("HashSet"));
}

#[test]
fn test_transpile_import_aliased() {
    let items = vec![
        ImportItem::Aliased {
            name: "HashMap".to_string(),
            alias: "Map".to_string(),
        },
    ];
    
    let tokens = Transpiler::transpile_import("std::collections", &items);
    let code = tokens.to_string();
    
    assert!(code.contains("HashMap"));
    assert!(code.contains("as"));
    assert!(code.contains("Map"));
}

#[test]
fn test_transpile_import_inline() {
    let items = vec![
        ImportItem::Named("File".to_string()),
    ];
    
    let tokens = Transpiler::transpile_import_inline("std::fs", &items);
    let code = tokens.to_string();
    
    assert!(code.contains("use"));
    assert!(code.contains("std") && code.contains("fs"));  // Should contain std::fs
}

// ==================== EXPORT TESTS ====================

#[test]
fn test_transpile_export_single() {
    let items = vec!["my_function".to_string()];
    
    let tokens = Transpiler::transpile_export(&items);
    let code = tokens.to_string();
    
    assert!(code.contains("pub use"));
    assert!(code.contains("my_function"));
}

#[test]
fn test_transpile_export_multiple() {
    let items = vec![
        "func1".to_string(),
        "func2".to_string(),
        "MyStruct".to_string(),
    ];
    
    let tokens = Transpiler::transpile_export(&items);
    let code = tokens.to_string();
    
    assert!(code.contains("pub use"));
    assert!(code.contains("func1"));
    assert!(code.contains("func2"));
    assert!(code.contains("MyStruct"));
}

// ==================== METHOD CALL TESTS ====================

#[test]
fn test_transpile_method_call_simple() {
    let transpiler = create_test_transpiler();
    let object = create_identifier("vec");
    let args = vec![create_integer(42)];
    
    let result = transpiler.transpile_method_call(&object, "push", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("vec"));
    assert!(code.contains("push"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_method_call_no_args() {
    let transpiler = create_test_transpiler();
    let object = create_identifier("s");
    let args = vec![];
    
    let result = transpiler.transpile_method_call(&object, "len", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("s"));
    assert!(code.contains("len"));
}

#[test]
fn test_transpile_method_call_chain() {
    let transpiler = create_test_transpiler();
    let object = create_identifier("text");
    let args = vec![];
    
    let result = transpiler.transpile_method_call(&object, "trim", &args);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let code = tokens.to_string();
    
    assert!(code.contains("text"));
    assert!(code.contains("trim"));
}

// Run all tests with: cargo test statements_transpiler_tdd --lib