//! Simple TDD tests for backend/transpiler/statements.rs
//! Focus on testable parts without complex type construction

use ruchy::frontend::ast::{ImportItem, Pattern, PipelineStage, Span};
use ruchy::Transpiler;
use ruchy::{Expr, ExprKind, Literal};

// ============================================================================
// Test Helpers
// ============================================================================

fn make_literal(val: i64) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Integer(val)),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_bool_literal(val: bool) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::Bool(val)),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_string_literal(s: &str) -> Expr {
    Expr {
        kind: ExprKind::Literal(Literal::String(s.to_string())),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_identifier(name: &str) -> Expr {
    Expr {
        kind: ExprKind::Identifier(name.to_string()),
        span: Span::default(),
        attributes: vec![],
    }
}

fn make_block(exprs: Vec<Expr>) -> Expr {
    Expr {
        kind: ExprKind::Block(exprs),
        span: Span::default(),
        attributes: vec![],
    }
}

// ============================================================================
// If Statement Tests
// ============================================================================

#[test]
fn test_transpile_if_simple() {
    let transpiler = Transpiler::new();
    let condition = make_bool_literal(true);
    let then_branch = make_literal(42);
    let result = transpiler
        .transpile_if(&condition, &then_branch, None)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("if"));
    assert!(code.contains("true"));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_if_with_else() {
    let transpiler = Transpiler::new();
    let condition = make_bool_literal(false);
    let then_branch = make_literal(1);
    let else_branch = make_literal(2);
    let result = transpiler
        .transpile_if(&condition, &then_branch, Some(&else_branch))
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("if"));
    assert!(code.contains("else"));
    assert!(code.contains('1'));
    assert!(code.contains('2'));
}

// ============================================================================
// Let Statement Tests (Simplified API)
// ============================================================================

#[test]
fn test_transpile_let_simple() {
    let transpiler = Transpiler::new();
    let name = "x";
    let value = make_literal(100);
    let body = make_identifier("x");
    let result = transpiler
        .transpile_let(name, &value, &body, false)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains('x'));
    assert!(code.contains("100"));
}

#[test]
fn test_transpile_let_mutable() {
    let transpiler = Transpiler::new();
    let name = "counter";
    let value = make_literal(0);
    let body = make_identifier("counter");
    let result = transpiler.transpile_let(name, &value, &body, true).unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains("mut"));
    assert!(code.contains("counter"));
}

// ============================================================================
// Let Pattern Tests
// ============================================================================

#[test]
fn test_transpile_let_pattern_identifier() {
    let transpiler = Transpiler::new();
    let pattern = Pattern::Identifier("x".to_string());
    let value = make_literal(42);
    let body = make_identifier("x");
    let result = transpiler
        .transpile_let_pattern(&pattern, &value, &body)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains('x'));
    assert!(code.contains("42"));
}

#[test]
fn test_transpile_let_pattern_tuple() {
    let transpiler = Transpiler::new();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let value = Expr {
        kind: ExprKind::Tuple(vec![make_literal(1), make_literal(2)]),
        span: Span::default(),
        attributes: vec![],
    };
    let body = make_identifier("a");
    let result = transpiler
        .transpile_let_pattern(&pattern, &value, &body)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains('a'));
    assert!(code.contains('b'));
}

#[test]
fn test_transpile_let_pattern_wildcard() {
    let transpiler = Transpiler::new();
    let pattern = Pattern::Wildcard;
    let value = make_literal(999);
    let body = make_literal(0);
    let result = transpiler
        .transpile_let_pattern(&pattern, &value, &body)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("let"));
    assert!(code.contains('_'));
}

// ============================================================================
// Call Tests
// ============================================================================

#[test]
fn test_transpile_call_simple() {
    let transpiler = Transpiler::new();
    let func = make_identifier("print");
    let args = vec![make_string_literal("Hello")];
    let result = transpiler.transpile_call(&func, &args).unwrap();
    let code = result.to_string();
    assert!(code.contains("print"));
    assert!(code.contains("Hello"));
}

#[test]
fn test_transpile_call_no_args() {
    let transpiler = Transpiler::new();
    let func = make_identifier("get_time");
    let args = vec![];
    let result = transpiler.transpile_call(&func, &args).unwrap();
    let code = result.to_string();
    assert!(code.contains("get_time"));
    assert!(code.contains("()"));
}

#[test]
fn test_transpile_call_multiple_args() {
    let transpiler = Transpiler::new();
    let func = make_identifier("add");
    let args = vec![make_literal(10), make_literal(20), make_literal(30)];
    let result = transpiler.transpile_call(&func, &args).unwrap();
    let code = result.to_string();
    assert!(code.contains("add"));
    assert!(code.contains("10"));
    assert!(code.contains("20"));
    assert!(code.contains("30"));
}

// ============================================================================
// Block Tests
// ============================================================================

#[test]
fn test_transpile_block_empty() {
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_block(&[]).unwrap();
    let code = result.to_string();
    assert!(code.contains('{'));
    assert!(code.contains('}'));
}

#[test]
fn test_transpile_block_single() {
    let transpiler = Transpiler::new();
    let exprs = vec![make_literal(42)];
    let result = transpiler.transpile_block(&exprs).unwrap();
    let code = result.to_string();
    assert!(code.contains('{'));
    assert!(code.contains("42"));
    assert!(code.contains('}'));
}

#[test]
fn test_transpile_block_multiple() {
    let transpiler = Transpiler::new();
    let exprs = vec![make_literal(1), make_literal(2), make_literal(3)];
    let result = transpiler.transpile_block(&exprs).unwrap();
    let code = result.to_string();
    assert!(code.contains('{'));
    assert!(code.contains('1'));
    assert!(code.contains('2'));
    assert!(code.contains('3'));
    assert!(code.contains('}'));
}

#[test]
fn test_transpile_block_with_semicolons() {
    let transpiler = Transpiler::new();
    let exprs = vec![
        Expr {
            kind: ExprKind::Let {
                name: "x".to_string(),
                type_annotation: None,
                value: Box::new(make_literal(10)),
                body: Box::new(make_identifier("x")),
                is_mutable: false,
            },
            span: Span::default(),
            attributes: vec![],
        },
        make_literal(20),
    ];
    let result = transpiler.transpile_block(&exprs).unwrap();
    let code = result.to_string();
    assert!(code.contains('{'));
    assert!(code.contains('}'));
}

// ============================================================================
// Loop Tests
// ============================================================================

#[test]
fn test_transpile_for_simple() {
    let transpiler = Transpiler::new();
    let var = "i";
    let iter = Expr {
        kind: ExprKind::Range {
            start: Box::new(make_literal(0)),
            end: Box::new(make_literal(10)),
            inclusive: false,
        },
        span: Span::default(),
        attributes: vec![],
    };
    let body = make_identifier("i");
    let result = transpiler.transpile_for(var, None, &iter, &body).unwrap();
    let code = result.to_string();
    assert!(code.contains("for"));
    assert!(code.contains('i'));
    assert!(code.contains("in"));
}

#[test]
fn test_transpile_while() {
    let transpiler = Transpiler::new();
    let condition = make_bool_literal(true);
    let body = make_block(vec![make_literal(1)]);
    let result = transpiler.transpile_while(&condition, &body).unwrap();
    let code = result.to_string();
    assert!(code.contains("while"));
    assert!(code.contains("true"));
}

#[test]
fn test_transpile_loop() {
    let transpiler = Transpiler::new();
    let body = make_block(vec![
        make_literal(42),
        Expr {
            kind: ExprKind::Break { label: None },
            span: Span::default(),
            attributes: vec![],
        },
    ]);
    let result = transpiler.transpile_loop(&body).unwrap();
    let code = result.to_string();
    assert!(code.contains("loop"));
    assert!(code.contains("42"));
    assert!(code.contains("break"));
}

// ============================================================================
// If-Let and While-Let Tests
// ============================================================================

#[test]
fn test_transpile_if_let() {
    let transpiler = Transpiler::new();
    let pattern = Pattern::Identifier("x".to_string());
    let expr = make_identifier("maybe_value");
    let then_branch = make_identifier("x");
    let result = transpiler
        .transpile_if_let(&pattern, &expr, &then_branch, None)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("if"));
    assert!(code.contains("let"));
}

#[test]
fn test_transpile_while_let() {
    let transpiler = Transpiler::new();
    let pattern = Pattern::Identifier("item".to_string());
    let expr = make_identifier("iter");
    let body = make_identifier("item");
    let result = transpiler
        .transpile_while_let(&pattern, &expr, &body)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("while"));
    assert!(code.contains("let"));
}

// ============================================================================
// Pipeline Tests
// ============================================================================

#[test]
fn test_transpile_pipeline_simple() {
    let transpiler = Transpiler::new();
    let expr = make_literal(5);
    let stages = vec![
        PipelineStage {
            op: Box::new(make_identifier("double")),
            span: Span::default(),
        },
        PipelineStage {
            op: Box::new(make_identifier("increment")),
            span: Span::default(),
        },
    ];
    let result = transpiler.transpile_pipeline(&expr, &stages).unwrap();
    let code = result.to_string();
    assert!(code.contains('5'));
}

#[test]
fn test_transpile_pipeline_empty() {
    let transpiler = Transpiler::new();
    let expr = make_literal(42);
    let stages = vec![];
    let result = transpiler.transpile_pipeline(&expr, &stages).unwrap();
    let code = result.to_string();
    assert!(code.contains("42"));
}

// ============================================================================
// List Comprehension Tests
// ============================================================================

#[test]
fn test_transpile_list_comprehension() {
    let transpiler = Transpiler::new();
    let output = make_identifier("x");
    let var = "x";
    let iter = Expr {
        kind: ExprKind::List(vec![make_literal(1), make_literal(2), make_literal(3)]),
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler
        .transpile_list_comprehension(&output, var, &iter, None)
        .unwrap();
    let code = result.to_string();
    assert!(code.contains('x'));
}

#[test]
fn test_transpile_list_comprehension_with_filter() {
    let transpiler = Transpiler::new();
    let output = make_identifier("x");
    let var = "x";
    let iter = make_identifier("numbers");
    let filter = Expr {
        kind: ExprKind::Binary {
            op: ruchy::BinaryOp::Greater,
            left: Box::new(make_identifier("x")),
            right: Box::new(make_literal(5)),
        },
        span: Span::default(),
        attributes: vec![],
    };
    let result = transpiler
        .transpile_list_comprehension(&output, var, &iter, Some(&filter))
        .unwrap();
    let code = result.to_string();
    assert!(code.contains("filter"));
}

// ============================================================================
// Module Tests
// ============================================================================

#[test]
fn test_transpile_module() {
    let transpiler = Transpiler::new();
    let name = "math_utils";
    let body = make_block(vec![make_literal(42)]);
    let result = transpiler.transpile_module(name, &body).unwrap();
    let code = result.to_string();
    assert!(code.contains("mod"));
    assert!(code.contains("math_utils"));
}

// ============================================================================
// Import/Export Tests
// ============================================================================

#[test]
fn test_transpile_import_wildcard() {
    let path = "std::collections";
    let result = Transpiler::transpile_import_all(path, "*");
    let code = result.to_string();
    assert!(code.contains("use"));
    assert!(code.contains("std"));
    assert!(code.contains("collections"));
    assert!(code.contains('*'));
}

#[test]
fn test_transpile_import_specific() {
    let path = "std::vec";
    let items = Some(&vec!["Vec".to_string()][..]);
    let result = Transpiler::transpile_import(path, items);
    let code = result.to_string();
    assert!(code.contains("use"));
    assert!(code.contains("Vec"));
}

#[test]
fn test_transpile_import_aliased() {
    let path = "std::collections";
    let result = Transpiler::transpile_import_all(path, "collections");
    let code = result.to_string();
    assert!(code.contains("use"));
    assert!(code.contains("HashMap"));
    assert!(code.contains("as"));
    assert!(code.contains("Map"));
}

#[test]
fn test_transpile_export() {
    let items = vec!["my_function".to_string(), "MyStruct".to_string()];
    let result = Transpiler::transpile_export_list(&items);
    let code = result.to_string();
    assert!(code.contains("pub"));
}

#[test]
fn test_transpile_import_inline() {
    let path = "utils";
    let items = vec![ImportItem::Named("helper".to_string())];
    let result = Transpiler::transpile_import_inline(path, &items);
    let code = result.to_string();
    // Inline imports should still generate use statements
    assert!(code.contains("use"));
}
