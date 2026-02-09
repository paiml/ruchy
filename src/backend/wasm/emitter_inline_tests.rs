use super::*;
use crate::frontend::ast::Span;
use crate::frontend::parser::Parser;


#[test]
fn test_wasm_emitter_new() {
    let emitter = WasmEmitter::new();
    // Just verify creation works
    assert!(emitter.functions.borrow().is_empty());
}

#[test]
fn test_wasm_emitter_default() {
    let emitter = WasmEmitter::default();
    assert!(emitter.functions.borrow().is_empty());
}

#[test]
fn test_lower_literal_integer() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Integer(42, None));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_float() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Float(3.14));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_bool_true() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Bool(true));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_bool_false() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Bool(false));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_string() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::String("test".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_unit() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Unit);
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_char() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Char('A'));
    assert!(result.is_ok());
}

#[test]
fn test_infer_element_type_via_parser() {
    let emitter = WasmEmitter::new();
    let mut parser = Parser::new("42");
    if let Ok(ast) = parser.parse() {
        let ty = emitter.infer_element_type(&ast);
        // Integer literals infer to I32
        let _ = ty;
    }
}

#[test]
fn test_infer_element_type_float_via_parser() {
    let emitter = WasmEmitter::new();
    let mut parser = Parser::new("3.14");
    if let Ok(ast) = parser.parse() {
        let ty = emitter.infer_element_type(&ast);
        let _ = ty;
    }
}

#[test]
fn test_collect_tuple_types_via_parser() {
    let emitter = WasmEmitter::new();
    let mut parser = Parser::new("let x = (1, 2.0); x");
    if let Ok(ast) = parser.parse() {
        emitter.collect_tuple_types(&ast);
    }
}

#[test]
fn test_structs_registration() {
    let emitter = WasmEmitter::new();
    // Register a struct manually
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    assert!(emitter.structs.borrow().contains_key("Point"));
}

#[test]
fn test_functions_registration() {
    let emitter = WasmEmitter::new();
    // Register a function manually
    emitter
        .functions
        .borrow_mut()
        .insert("main".to_string(), (0, false));
    let funcs = emitter.functions.borrow();
    assert!(funcs.contains_key("main"));
    assert_eq!(funcs["main"], (0, false));
}

#[test]
fn test_tuple_types_registration() {
    let emitter = WasmEmitter::new();
    emitter
        .tuple_types
        .borrow_mut()
        .insert("pair".to_string(), vec![WasmType::I32, WasmType::F32]);
    let types = emitter.tuple_types.borrow();
    assert!(types.contains_key("pair"));
    assert_eq!(types["pair"].len(), 2);
}

// === COVERAGE: Additional tests for uncovered branches ===

#[test]
fn test_infer_element_type_bool() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::Bool(true)),
        Span { start: 0, end: 4 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::I32);
}

#[test]
fn test_infer_element_type_string() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Literal(Literal::String("hello".to_string())),
        Span { start: 0, end: 7 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::I32); // Address
}

#[test]
fn test_infer_element_type_binary_add() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span { start: 0, end: 1 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 4, end: 5 },
            )),
        },
        Span { start: 0, end: 5 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::F32); // Binary ops return F32
}

#[test]
fn test_infer_element_type_binary_comparison() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Less,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span { start: 0, end: 1 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 4, end: 5 },
            )),
        },
        Span { start: 0, end: 5 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::I32); // Comparisons return I32
}

#[test]
fn test_infer_element_type_identifier() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Identifier("x".to_string()),
        Span { start: 0, end: 1 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::I32); // Default for complex expressions
}

#[test]
fn test_collect_tuple_types_block() {
    let emitter = WasmEmitter::new();
    let block = Expr::new(
        ExprKind::Block(vec![
            Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span { start: 0, end: 1 },
            ),
            Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 3, end: 4 },
            ),
        ]),
        Span { start: 0, end: 5 },
    );
    emitter.collect_tuple_types(&block);
    // Should not panic on block traversal
}

#[test]
fn test_collect_tuple_types_if() {
    let emitter = WasmEmitter::new();
    let if_expr = Expr::new(
        ExprKind::If {
            condition: Box::new(Expr::new(
                ExprKind::Literal(Literal::Bool(true)),
                Span { start: 0, end: 4 },
            )),
            then_branch: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(1, None)),
                Span { start: 6, end: 7 },
            )),
            else_branch: Some(Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 13, end: 14 },
            ))),
        },
        Span { start: 0, end: 15 },
    );
    emitter.collect_tuple_types(&if_expr);
    // Should not panic on if traversal
}

#[test]
fn test_lower_literal_null() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Null);
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_byte() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Byte(255));
    assert!(result.is_ok());
}

#[test]
fn test_lower_literal_atom() {
    let emitter = WasmEmitter::new();
    let result = emitter.lower_literal(&Literal::Atom("ok".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_symbols_operations() {
    let emitter = WasmEmitter::new();
    // Verify symbols table starts with zero local count and depth 1
    assert_eq!(emitter.symbols.borrow().local_count(), 0);
    assert_eq!(emitter.symbols.borrow().scope_depth(), 1);
}

#[test]
fn test_multiple_structs_registration() {
    let emitter = WasmEmitter::new();
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    emitter.structs.borrow_mut().insert(
        "Rect".to_string(),
        vec!["width".to_string(), "height".to_string()],
    );
    assert_eq!(emitter.structs.borrow().len(), 2);
}

#[test]
fn test_multiple_functions_registration() {
    let emitter = WasmEmitter::new();
    emitter
        .functions
        .borrow_mut()
        .insert("main".to_string(), (0, false));
    emitter
        .functions
        .borrow_mut()
        .insert("helper".to_string(), (1, true));
    assert_eq!(emitter.functions.borrow().len(), 2);
    assert_eq!(emitter.functions.borrow()["helper"], (1, true));
}

#[test]
fn test_infer_element_type_binary_subtract() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Subtract,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(5, None)),
                Span { start: 0, end: 1 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span { start: 4, end: 5 },
            )),
        },
        Span { start: 0, end: 5 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::F32);
}

#[test]
fn test_infer_element_type_binary_multiply() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Multiply,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 0, end: 1 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span { start: 4, end: 5 },
            )),
        },
        Span { start: 0, end: 5 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::F32);
}

#[test]
fn test_infer_element_type_binary_divide() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Divide,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(10, None)),
                Span { start: 0, end: 2 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(2, None)),
                Span { start: 5, end: 6 },
            )),
        },
        Span { start: 0, end: 6 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::F32);
}

#[test]
fn test_infer_element_type_binary_modulo() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: BinaryOp::Modulo,
            left: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(10, None)),
                Span { start: 0, end: 2 },
            )),
            right: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(3, None)),
                Span { start: 5, end: 6 },
            )),
        },
        Span { start: 0, end: 6 },
    );
    let ty = emitter.infer_element_type(&expr);
    assert_eq!(ty, WasmType::F32);
}
