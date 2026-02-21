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

// ============================================================================
// COVERAGE: Tests for uncovered functions in emitter.rs
// Targets: lower_match, lower_field_access, lower_assign, store_pattern_values,
//          lower_string_interpolation, lower_call, register_pattern_symbols
// ============================================================================

fn span() -> Span {
    Span { start: 0, end: 1 }
}

fn int_expr(n: i64) -> Expr {
    Expr::new(ExprKind::Literal(Literal::Integer(n, None)), span())
}

fn float_expr(f: f64) -> Expr {
    Expr::new(ExprKind::Literal(Literal::Float(f)), span())
}

fn ident_expr(name: &str) -> Expr {
    Expr::new(ExprKind::Identifier(name.to_string()), span())
}

fn string_expr(s: &str) -> Expr {
    Expr::new(ExprKind::Literal(Literal::String(s.to_string())), span())
}

fn unit_expr() -> Expr {
    Expr::new(ExprKind::Literal(Literal::Unit), span())
}

// --- register_pattern_symbols tests ---

#[test]
fn test_register_pattern_symbols_identifier() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Identifier("x".to_string());
    emitter.register_pattern_symbols(&pattern);
    let sym = emitter.symbols.borrow();
    assert!(sym.lookup("x").is_some());
    assert_eq!(sym.lookup_type("x"), Some(WasmType::I32));
}

#[test]
fn test_register_pattern_symbols_tuple() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    emitter.register_pattern_symbols(&pattern);
    let sym = emitter.symbols.borrow();
    assert!(sym.lookup("a").is_some());
    assert!(sym.lookup("b").is_some());
}

#[test]
fn test_register_pattern_symbols_wildcard() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Wildcard;
    emitter.register_pattern_symbols(&pattern);
    assert_eq!(emitter.symbols.borrow().local_count(), 0);
}

#[test]
fn test_register_pattern_symbols_nested_tuple() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Tuple(vec![
            Pattern::Identifier("y".to_string()),
            Pattern::Identifier("z".to_string()),
        ]),
    ]);
    emitter.register_pattern_symbols(&pattern);
    let sym = emitter.symbols.borrow();
    assert!(sym.lookup("x").is_some());
    assert!(sym.lookup("y").is_some());
    assert!(sym.lookup("z").is_some());
    assert_eq!(sym.local_count(), 3);
}

#[test]
fn test_register_pattern_symbols_other_pattern() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Literal(Literal::Integer(42, None));
    emitter.register_pattern_symbols(&pattern);
    assert_eq!(emitter.symbols.borrow().local_count(), 0);
}

#[test]
fn test_register_pattern_symbols_tuple_with_wildcard() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Wildcard,
        Pattern::Identifier("c".to_string()),
    ]);
    emitter.register_pattern_symbols(&pattern);
    let sym = emitter.symbols.borrow();
    assert!(sym.lookup("a").is_some());
    assert!(sym.lookup("c").is_some());
    assert_eq!(sym.local_count(), 2);
}

// --- store_pattern_values tests ---

#[test]
fn test_store_pattern_values_identifier() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("x".to_string(), WasmType::I32);
    let pattern = Pattern::Identifier("x".to_string());
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_ok());
    assert!(!instructions.is_empty());
}

#[test]
fn test_store_pattern_values_wildcard() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Wildcard;
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_ok());
    assert_eq!(instructions.len(), 1);
}

#[test]
fn test_store_pattern_values_tuple() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("a".to_string(), WasmType::I32);
    emitter
        .symbols
        .borrow_mut()
        .insert("b".to_string(), WasmType::I32);
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Identifier("b".to_string()),
    ]);
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_ok());
    assert!(instructions.len() >= 4);
}

#[test]
fn test_store_pattern_values_tuple_single_element() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("x".to_string(), WasmType::I32);
    let pattern = Pattern::Tuple(vec![Pattern::Identifier("x".to_string())]);
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_ok());
    assert!(!instructions.is_empty());
}

#[test]
fn test_store_pattern_values_unsupported_pattern() {
    let emitter = WasmEmitter::new();
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![],
        has_rest: false,
    };
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not yet supported"));
}

#[test]
fn test_store_pattern_values_nested_tuple() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("a".to_string(), WasmType::I32);
    emitter
        .symbols
        .borrow_mut()
        .insert("b".to_string(), WasmType::I32);
    emitter
        .symbols
        .borrow_mut()
        .insert("c".to_string(), WasmType::I32);
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("a".to_string()),
        Pattern::Tuple(vec![
            Pattern::Identifier("b".to_string()),
            Pattern::Identifier("c".to_string()),
        ]),
    ]);
    let mut instructions = vec![];
    let result = emitter.store_pattern_values(&pattern, &mut instructions);
    assert!(result.is_ok());
    assert!(instructions.len() >= 6);
}

// --- lower_call tests ---

#[test]
fn test_lower_call_println_integer() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let args = vec![int_expr(42)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 2);
}

#[test]
fn test_lower_call_println_float() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let args = vec![float_expr(3.14)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 2);
}

#[test]
fn test_lower_call_println_string_then_value() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let args = vec![string_expr("value: {}"), int_expr(42)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_call_println_only_string() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let args = vec![string_expr("hello world")];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_println_no_args() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let args: Vec<Expr> = vec![];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_print_builtin() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("print");
    let args = vec![int_expr(10)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_eprintln_builtin() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("eprintln");
    let args = vec![int_expr(10)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_eprint_builtin() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("eprint");
    let args = vec![float_expr(2.5)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_user_function() {
    let emitter = WasmEmitter::new();
    emitter
        .functions
        .borrow_mut()
        .insert("add".to_string(), (2, false));
    let func = ident_expr("add");
    let args = vec![int_expr(1), int_expr(2)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 3);
}

#[test]
fn test_lower_call_unknown_function() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("unknown_func");
    let args = vec![int_expr(1)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown function"));
}

#[test]
fn test_lower_call_non_identifier_function() {
    let emitter = WasmEmitter::new();
    let func = int_expr(42);
    let args = vec![int_expr(1)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must use identifiers"));
}

#[test]
fn test_lower_call_println_skip_string_interpolation() {
    let emitter = WasmEmitter::new();
    let func = ident_expr("println");
    let interp_arg = Expr::new(
        ExprKind::StringInterpolation {
            parts: vec![StringPart::Text("hello".to_string())],
        },
        span(),
    );
    let args = vec![interp_arg, int_expr(42)];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

#[test]
fn test_lower_call_user_function_no_args() {
    let emitter = WasmEmitter::new();
    emitter
        .functions
        .borrow_mut()
        .insert("no_args".to_string(), (3, true));
    let func = ident_expr("no_args");
    let args: Vec<Expr> = vec![];
    let result = emitter.lower_call(&func, &args);
    assert!(result.is_ok());
}

// --- lower_field_access tests ---

#[test]
fn test_lower_field_access_tuple_i32() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    emitter
        .tuple_types
        .borrow_mut()
        .insert("t".to_string(), vec![WasmType::I32, WasmType::I32]);
    let object = ident_expr("t");
    let result = emitter.lower_field_access(&object, "0");
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_field_access_tuple_f32() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    emitter
        .tuple_types
        .borrow_mut()
        .insert("t".to_string(), vec![WasmType::I32, WasmType::F32]);
    let object = ident_expr("t");
    let result = emitter.lower_field_access(&object, "1");
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_field_access_tuple_no_type_info() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    let object = ident_expr("t");
    let result = emitter.lower_field_access(&object, "0");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_tuple_non_identifier_object() {
    let emitter = WasmEmitter::new();
    let object = int_expr(100);
    let result = emitter.lower_field_access(&object, "0");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_struct_field() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    let object = ident_expr("p");
    let result = emitter.lower_field_access(&object, "x");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_struct_second_field() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    let object = ident_expr("p");
    let result = emitter.lower_field_access(&object, "y");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_unknown_struct_field() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    let object = ident_expr("p");
    let result = emitter.lower_field_access(&object, "z");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_no_struct_registered() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    let object = ident_expr("p");
    let result = emitter.lower_field_access(&object, "name");
    assert!(result.is_ok());
}

#[test]
fn test_lower_field_access_tuple_index_out_of_range() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    emitter
        .tuple_types
        .borrow_mut()
        .insert("t".to_string(), vec![WasmType::I32]);
    let object = ident_expr("t");
    let result = emitter.lower_field_access(&object, "5");
    assert!(result.is_ok());
}

// --- lower_assign tests ---

#[test]
fn test_lower_assign_identifier() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("x".to_string(), WasmType::I32);
    let target = ident_expr("x");
    let value = int_expr(42);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_assign_field_access_tuple() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    let target = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(ident_expr("t")),
            field: "0".to_string(),
        },
        span(),
    );
    let value = int_expr(99);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_assign_field_access_struct() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    let target = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(ident_expr("p")),
            field: "y".to_string(),
        },
        span(),
    );
    let value = int_expr(10);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
}

#[test]
fn test_lower_assign_field_access_unknown_field() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("p".to_string(), WasmType::I32);
    emitter
        .structs
        .borrow_mut()
        .insert("Point".to_string(), vec!["x".to_string(), "y".to_string()]);
    let target = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(ident_expr("p")),
            field: "unknown".to_string(),
        },
        span(),
    );
    let value = int_expr(5);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
}

#[test]
fn test_lower_assign_index_access() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("arr".to_string(), WasmType::I32);
    let target = Expr::new(
        ExprKind::IndexAccess {
            object: Box::new(ident_expr("arr")),
            index: Box::new(int_expr(2)),
        },
        span(),
    );
    let value = int_expr(100);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 5);
}

#[test]
fn test_lower_assign_unsupported_target() {
    let emitter = WasmEmitter::new();
    let target = int_expr(42);
    let value = int_expr(10);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not supported"));
}

#[test]
fn test_lower_assign_via_lower_expression() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("x".to_string(), WasmType::I32);
    let assign_expr = Expr::new(
        ExprKind::Assign {
            target: Box::new(ident_expr("x")),
            value: Box::new(int_expr(55)),
        },
        span(),
    );
    let result = emitter.lower_expression(&assign_expr);
    assert!(result.is_ok());
}

// --- lower_string_interpolation tests ---

#[test]
fn test_lower_string_interpolation_all_text() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text("hello ".to_string()),
        StringPart::Text("world".to_string()),
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_single_text() {
    let emitter = WasmEmitter::new();
    let parts = vec![StringPart::Text("hello".to_string())];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_single_expr() {
    let emitter = WasmEmitter::new();
    let parts = vec![StringPart::Expr(Box::new(int_expr(42)))];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_string_interpolation_text_and_expr() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text("value: ".to_string()),
        StringPart::Expr(Box::new(int_expr(42))),
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_expr_with_format() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text("pi = ".to_string()),
        StringPart::ExprWithFormat {
            expr: Box::new(float_expr(3.14)),
            format_spec: ".2".to_string(),
        },
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_empty_text_parts() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text(String::new()),
        StringPart::Text(String::new()),
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_multi_text_parts() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text("a".to_string()),
        StringPart::Text("b".to_string()),
        StringPart::Text("c".to_string()),
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_via_lower_expression() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::StringInterpolation {
            parts: vec![
                StringPart::Text("count: ".to_string()),
                StringPart::Expr(Box::new(int_expr(10))),
            ],
        },
        span(),
    );
    let result = emitter.lower_expression(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_lower_string_interpolation_text_only_concatenation() {
    let emitter = WasmEmitter::new();
    let parts = vec![
        StringPart::Text("Hello, ".to_string()),
        StringPart::Text("World!".to_string()),
    ];
    let result = emitter.lower_string_interpolation(&parts);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

// --- lower_match tests ---

#[test]
fn test_lower_match_empty_arms() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms: Vec<crate::frontend::ast::MatchArm> = vec![];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert_eq!(instructions.len(), 1);
}

#[test]
fn test_lower_match_wildcard_only() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Wildcard,
        guard: None,
        body: Box::new(int_expr(42)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(!instructions.is_empty());
}

#[test]
fn test_lower_match_literal_single_arm() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Literal(Literal::Integer(1, None)),
        guard: None,
        body: Box::new(int_expr(10)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_literal_with_wildcard() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1, None)),
            guard: None,
            body: Box::new(int_expr(10)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 3);
}

#[test]
fn test_lower_match_multiple_literal_arms() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(2);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1, None)),
            guard: None,
            body: Box::new(int_expr(10)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Literal(Literal::Integer(2, None)),
            guard: None,
            body: Box::new(int_expr(20)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 5);
}

#[test]
fn test_lower_match_or_pattern() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(3);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Literal(Literal::Integer(2, None)),
            ]),
            guard: None,
            body: Box::new(int_expr(100)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 5);
}

#[test]
fn test_lower_match_or_pattern_last_arm() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Or(vec![
            Pattern::Literal(Literal::Integer(1, None)),
            Pattern::Literal(Literal::Integer(2, None)),
        ]),
        guard: None,
        body: Box::new(int_expr(99)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_tuple_pattern() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Tuple(vec![
            Pattern::Identifier("a".to_string()),
            Pattern::Identifier("b".to_string()),
        ]),
        guard: None,
        body: Box::new(int_expr(42)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_tuple_pattern_not_last() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Tuple(vec![Pattern::Identifier("a".to_string())]),
            guard: None,
            body: Box::new(int_expr(10)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_identifier_pattern() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Identifier("x".to_string()),
        guard: None,
        body: Box::new(int_expr(42)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_unsupported_pattern() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![crate::frontend::ast::MatchArm {
        pattern: Pattern::Range {
            start: Box::new(Pattern::Literal(Literal::Integer(1, None))),
            end: Box::new(Pattern::Literal(Literal::Integer(10, None))),
            inclusive: true,
        },
        guard: None,
        body: Box::new(int_expr(42)),
        span: span(),
    }];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not yet supported"));
}

#[test]
fn test_lower_match_via_lower_expression() {
    let emitter = WasmEmitter::new();
    let match_ast = Expr::new(
        ExprKind::Match {
            expr: Box::new(int_expr(1)),
            arms: vec![
                crate::frontend::ast::MatchArm {
                    pattern: Pattern::Literal(Literal::Integer(1, None)),
                    guard: None,
                    body: Box::new(int_expr(10)),
                    span: span(),
                },
                crate::frontend::ast::MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(int_expr(0)),
                    span: span(),
                },
            ],
        },
        span(),
    );
    let result = emitter.lower_expression(&match_ast);
    assert!(result.is_ok());
}

#[test]
fn test_lower_match_or_pattern_non_literal_error() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(1);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Or(vec![Pattern::Identifier("x".to_string())]),
            guard: None,
            body: Box::new(int_expr(100)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Non-literal patterns in OR"));
}

#[test]
fn test_lower_match_three_or_patterns() {
    let emitter = WasmEmitter::new();
    let match_expr = int_expr(5);
    let arms = vec![
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Or(vec![
                Pattern::Literal(Literal::Integer(1, None)),
                Pattern::Literal(Literal::Integer(2, None)),
                Pattern::Literal(Literal::Integer(3, None)),
            ]),
            guard: None,
            body: Box::new(int_expr(100)),
            span: span(),
        },
        crate::frontend::ast::MatchArm {
            pattern: Pattern::Wildcard,
            guard: None,
            body: Box::new(int_expr(0)),
            span: span(),
        },
    ];
    let result = emitter.lower_match(&match_expr, &arms);
    assert!(result.is_ok());
    let instructions = result.unwrap();
    assert!(instructions.len() >= 8);
}

// --- Integration tests via lower_expression ---

#[test]
fn test_lower_expression_field_access() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("t".to_string(), WasmType::I32);
    let field_access = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(ident_expr("t")),
            field: "0".to_string(),
        },
        span(),
    );
    let result = emitter.lower_expression(&field_access);
    assert!(result.is_ok());
}

#[test]
fn test_lower_expression_assign_index() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("arr".to_string(), WasmType::I32);
    let assign = Expr::new(
        ExprKind::Assign {
            target: Box::new(Expr::new(
                ExprKind::IndexAccess {
                    object: Box::new(ident_expr("arr")),
                    index: Box::new(int_expr(0)),
                },
                span(),
            )),
            value: Box::new(int_expr(42)),
        },
        span(),
    );
    let result = emitter.lower_expression(&assign);
    assert!(result.is_ok());
}

#[test]
fn test_lower_expression_let_pattern_tuple() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("a".to_string(), WasmType::I32);
    emitter
        .symbols
        .borrow_mut()
        .insert("b".to_string(), WasmType::I32);
    let let_pattern = Expr::new(
        ExprKind::LetPattern {
            pattern: Pattern::Tuple(vec![
                Pattern::Identifier("a".to_string()),
                Pattern::Identifier("b".to_string()),
            ]),
            type_annotation: None,
            value: Box::new(Expr::new(
                ExprKind::Tuple(vec![int_expr(1), int_expr(2)]),
                span(),
            )),
            body: Box::new(unit_expr()),
            is_mutable: false,
            else_block: None,
        },
        span(),
    );
    let result = emitter.lower_expression(&let_pattern);
    assert!(result.is_ok());
}

#[test]
fn test_lower_expression_string_interpolation_single_text() {
    let emitter = WasmEmitter::new();
    let expr = Expr::new(
        ExprKind::StringInterpolation {
            parts: vec![StringPart::Text("just text".to_string())],
        },
        span(),
    );
    let result = emitter.lower_expression(&expr);
    assert!(result.is_ok());
}

#[test]
fn test_lower_assign_field_access_no_structs() {
    let emitter = WasmEmitter::new();
    emitter
        .symbols
        .borrow_mut()
        .insert("obj".to_string(), WasmType::I32);
    let target = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(ident_expr("obj")),
            field: "field_name".to_string(),
        },
        span(),
    );
    let value = int_expr(7);
    let result = emitter.lower_assign(&target, &value);
    assert!(result.is_ok());
}
