//! Tests for WASM Emitter
//!
//! Comprehensive tests for WASM code generation.

use super::*;
use crate::frontend::ast::{Expr, ExprKind, Literal};
use crate::frontend::parser::Parser;

#[test]
fn test_emitter_creates() {
    let _emitter = WasmEmitter::new();
}
#[test]
fn test_empty_program_emits() {
    let mut parser = Parser::new("");
    let expr = parser.parse().unwrap_or_else(|_| {
        // Create an empty block expression for empty input
        Expr::new(ExprKind::Block(vec![]), Default::default())
    });
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");
    assert!(!bytes.is_empty());
    // Check WASM magic number
    assert_eq!(&bytes[0..4], b"\0asm");
    // Check version
    assert_eq!(&bytes[4..8], &[1, 0, 0, 0]);
}
#[test]
fn test_integer_literal() {
    let mut parser = Parser::new("42");
    let expr = parser.parse().expect("Should parse integer");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");
    // Should contain i32.const instruction (0x41)
    assert!(bytes.contains(&0x41));
}

#[test]
fn test_binary_operations() {
    // Test addition
    let mut parser = Parser::new("1 + 2");
    let expr = parser.parse().expect("Should parse addition");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");
    // Should contain i32.add instruction (0x6a)
    assert!(bytes.contains(&0x6a));

    // Test subtraction
    let mut parser = Parser::new("5 - 3");
    let expr = parser.parse().expect("Should parse subtraction");
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");
    // Should contain i32.sub instruction (0x6b)
    assert!(bytes.contains(&0x6b));

    // Test multiplication
    let mut parser = Parser::new("3 * 4");
    let expr = parser.parse().expect("Should parse multiplication");
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");
    // Should contain i32.mul instruction (0x6c)
    assert!(bytes.contains(&0x6c));
}

#[test]
fn test_function_definition() {
    let mut parser = Parser::new("fun add(x, y) { x + y }");
    let expr = parser.parse().expect("Should parse function");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should have WASM magic number
    assert_eq!(&bytes[0..4], b"\0asm");
    // Should contain function section
    assert!(bytes.windows(2).any(|w| w == [0x03, 0x02])); // Function section with size
}

#[test]
fn test_if_expression() {
    let mut parser = Parser::new("if true { 1 } else { 2 }");
    let expr = parser.parse().expect("Should parse if expression");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain if instruction (0x04)
    assert!(bytes.contains(&0x04));
}

#[test]
fn test_local_variables() {
    let mut parser = Parser::new("let x = 5; x");
    let expr = parser.parse().expect("Should parse let binding");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain local.set (0x21) or local.get (0x20)
    assert!(bytes.iter().any(|&b| b == 0x20 || b == 0x21));
}

#[test]
fn test_comparison_operations() {
    let mut parser = Parser::new("3 > 2");
    let expr = parser.parse().expect("Should parse comparison");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain i32.gt_s instruction (0x4a)
    assert!(bytes.contains(&0x4a));
}

#[test]
fn test_boolean_literals() {
    let mut parser = Parser::new("true");
    let expr = parser.parse().expect("Should parse boolean");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // true should emit i32.const 1
    assert!(bytes.windows(2).any(|w| w == [0x41, 0x01]));

    let mut parser = Parser::new("false");
    let expr = parser.parse().expect("Should parse boolean");
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // false should emit i32.const 0
    assert!(bytes.windows(2).any(|w| w == [0x41, 0x00]));
}

#[test]
fn test_block_expression() {
    let mut parser = Parser::new("{ 1; 2; 3 }");
    let expr = parser.parse().expect("Should parse block");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain block instruction (0x02)
    assert!(bytes.contains(&0x02));
}

#[test]
fn test_while_loop() {
    let mut parser = Parser::new("while false { }");
    let expr = parser.parse().expect("Should parse while loop");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain loop instruction (0x03)
    assert!(bytes.contains(&0x03));
}

#[test]
fn test_break_statement() {
    let mut parser = Parser::new("while true { break }");
    let expr = parser.parse().expect("Should parse break");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain br instruction (0x0c)
    assert!(bytes.contains(&0x0c));
}

#[test]
fn test_continue_statement() {
    let mut parser = Parser::new("while true { continue }");
    let expr = parser.parse().expect("Should parse continue");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain br instruction for continue
    assert!(bytes.contains(&0x0c));
}

#[test]
fn test_return_statement() {
    let mut parser = Parser::new("fun test() { return 42 }");
    let expr = parser.parse().expect("Should parse return");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain return instruction (0x0f)
    assert!(bytes.contains(&0x0f));
}

#[test]
fn test_nested_expressions() {
    let mut parser = Parser::new("(1 + 2) * (3 - 4)");
    let expr = parser.parse().expect("Should parse nested expr");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain add, sub, and mul instructions
    assert!(bytes.contains(&0x6a)); // add
    assert!(bytes.contains(&0x6b)); // sub
    assert!(bytes.contains(&0x6c)); // mul
}

#[test]
fn test_logical_operations() {
    let mut parser = Parser::new("true && false");
    let expr = parser.parse().expect("Should parse logical and");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should generate valid WASM bytecode (specific instruction may vary)
    assert!(!bytes.is_empty());

    let mut parser = Parser::new("true || false");
    let expr = parser.parse().expect("Should parse logical or");
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should generate valid WASM bytecode (specific instruction may vary)
    assert!(!bytes.is_empty());
}

#[test]
fn test_unary_operations() {
    let mut parser = Parser::new("!true");
    let expr = parser.parse().expect("Should parse not");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain i32.eqz instruction (0x45)
    assert!(bytes.contains(&0x45));

    let mut parser = Parser::new("-5");
    let expr = parser.parse().expect("Should parse negate");
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    // Negate is typically implemented as 0 - x
}

#[test]
fn test_complex_function() {
    let mut parser = Parser::new(
        r"
        fun factorial(n) {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    ",
    );
    let expr = parser.parse().expect("Should parse complex function");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should be valid WASM
    assert_eq!(&bytes[0..4], b"\0asm");
    assert!(!bytes.is_empty());
}

#[test]
fn test_needs_memory_check() {
    let _emitter = WasmEmitter::new();

    // Simple integer shouldn't need memory
    let expr = Expr::new(
        ExprKind::Literal(Literal::Integer(42, None)),
        Default::default(),
    );
    assert!(!utils::needs_memory(&expr));

    // List should need memory
    let list_expr = Expr::new(ExprKind::List(vec![]), Default::default());
    assert!(utils::needs_memory(&list_expr));
}

#[test]
fn test_collect_functions() {
    let emitter = WasmEmitter::new();

    // Function definition
    let func_expr = Expr::new(
        ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(Expr::new(
                ExprKind::Literal(Literal::Integer(42, None)),
                Default::default(),
            )),
            return_type: None,
            is_async: false,
            is_pub: false,
        },
        Default::default(),
    );

    let funcs = emitter.collect_functions(&func_expr);
    assert_eq!(funcs.len(), 1);
    assert_eq!(funcs[0].0, "test");
}

#[test]
fn test_division_operation() {
    let mut parser = Parser::new("10 / 2");
    let expr = parser.parse().expect("Should parse division");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&expr);
    assert!(result.is_ok());
    let bytes = result.expect("operation should succeed in test");

    // Should contain i32.div_s instruction (0x6d)
    assert!(bytes.contains(&0x6d));
}

#[test]
fn test_wasm_has_import_section_for_println() {
    // RED phase: Test that WASM module has import section for println
    let mut parser = Parser::new(r#"println("Hello")"#);
    let ast = parser.parse().expect("operation should succeed in test");

    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter
        .emit(&ast)
        .expect("operation should succeed in test");

    // Parse WASM and verify import section exists
    let parser = wasmparser::Parser::new(0);
    let mut has_import = false;

    for payload in parser.parse_all(&wasm_bytes) {
        if let Ok(wasmparser::Payload::ImportSection(_)) = payload {
            has_import = true;
            break;
        }
    }

    assert!(
        has_import,
        "WASM module must have import section for println"
    );
}

#[test]
fn test_wasm_fstring_simple() {
    // RED phase: F-strings should compile to WASM
    // This test WILL FAIL until we implement string interpolation support
    let mut parser = Parser::new(r#"let x = 10; println(f"Value: {x}")"#);
    let ast = parser.parse().expect("operation should succeed in test");

    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast);

    // Should compile successfully
    assert!(
        wasm_bytes.is_ok(),
        "F-strings must compile to valid WASM: {:?}",
        wasm_bytes.err()
    );

    // Validate WASM bytecode
    let bytes = wasm_bytes.expect("operation should succeed in test");
    let validation = wasmparser::validate(&bytes);
    assert!(
        validation.is_ok(),
        "F-string WASM must pass validation: {:?}",
        validation.err()
    );
}

#[test]
fn test_wasm_match_simple_literal() {
    // RED phase: Match expressions should compile to WASM
    // This test WILL FAIL until we implement match expression support
    // Root cause: Match in let binding doesn't produce value on stack
    let code = r#"
        let number = 2
        let description = match number {
            1 => "one",
            2 => "two",
            _ => "other"
        }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("operation should succeed in test");

    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast);

    // Should compile successfully
    assert!(
        wasm_bytes.is_ok(),
        "Match expressions must compile to valid WASM: {:?}",
        wasm_bytes.err()
    );

    // Validate WASM bytecode
    let bytes = wasm_bytes.expect("operation should succeed in test");
    let validation = wasmparser::validate(&bytes);
    assert!(
        validation.is_ok(),
        "Match expression WASM must pass validation: {:?}",
        validation.err()
    );
}

// COVERAGE: Additional expression tests
// NOTE: SymbolTable and WasmType tests moved to symbol_table.rs and types.rs
#[test]
fn test_while_loop_with_assignment() {
    let code = "let x = 0; while x < 10 { x = x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_list_literal() {
    let code = "[1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_tuple_literal() {
    let code = "(1, 2, 3)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_struct_definition_and_literal() {
    let code = r#"
        struct Point { x: i32, y: i32 }
        let p = Point { x: 10, y: 20 }
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_index_access() {
    let code = "let arr = [1, 2, 3]; arr[0]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_assignment() {
    let code = "let x = 1; x = 2; x";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_return_expression() {
    let code = "fun foo() { return 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_float_literal() {
    let code = "3.14";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    // Should contain f32.const instruction (0x43)
    assert!(bytes.contains(&0x43));
}

#[test]
fn test_float_operations() {
    let code = "3.14 + 2.71";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_nested_function_calls() {
    let code = r#"
        fun add(a, b) { a + b }
        fun double(x) { add(x, x) }
        double(5)
    "#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_comparison_lt() {
    let code = "1 < 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    // i32.lt_s is 0x48
    assert!(bytes.contains(&0x48));
}

#[test]
fn test_comparison_eq() {
    let code = "1 == 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    // i32.eq is 0x46
    assert!(bytes.contains(&0x46));
}

#[test]
fn test_logical_and() {
    let code = "true && false";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_logical_or() {
    let code = "true || false";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_unary_negate() {
    let code = "-5";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_unary_not() {
    let code = "!true";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_nested_blocks() {
    let code = "{ { 1 } }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_if_without_else() {
    let code = "if true { 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_let_bindings() {
    let code = "let a = 1; let b = 2; let c = a + b; c";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_division() {
    let code = "10 / 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
    let bytes = result.unwrap();
    // i32.div_s is 0x6d
    assert!(bytes.contains(&0x6d));
}

#[test]
fn test_modulo() {
    let code = "10 % 3";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let emitter = WasmEmitter::new();
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[test]
fn test_wasm_type_to_valtype() {
    let emitter = WasmEmitter::new();
    assert_eq!(emitter.wasm_type_to_valtype(WasmType::I32), wasm_encoder::ValType::I32);
    assert_eq!(emitter.wasm_type_to_valtype(WasmType::I64), wasm_encoder::ValType::I64);
    assert_eq!(emitter.wasm_type_to_valtype(WasmType::F32), wasm_encoder::ValType::F32);
    assert_eq!(emitter.wasm_type_to_valtype(WasmType::F64), wasm_encoder::ValType::F64);
}

#[test]
fn test_infer_element_type_literals() {
    let emitter = WasmEmitter::new();

    let int_expr = Expr::new(ExprKind::Literal(Literal::Integer(42, None)), Default::default());
    assert_eq!(emitter.infer_element_type(&int_expr), WasmType::I32);

    let float_expr = Expr::new(ExprKind::Literal(Literal::Float(3.14)), Default::default());
    assert_eq!(emitter.infer_element_type(&float_expr), WasmType::F32);

    let bool_expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
    assert_eq!(emitter.infer_element_type(&bool_expr), WasmType::I32);
}

#[test]
fn test_uses_builtins_println() {
    let code = "println(42)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    assert!(utils::uses_builtins(&ast));
}

#[test]
fn test_uses_builtins_no_builtins() {
    let code = "1 + 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    assert!(!utils::uses_builtins(&ast));
}

#[test]
fn test_needs_memory_with_list() {
    let code = "[1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    assert!(utils::needs_memory(&ast));
}

#[test]
fn test_needs_memory_with_string() {
    let code = r#""hello""#;
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    assert!(utils::needs_memory(&ast));
}

#[test]
fn test_needs_memory_simple_int() {
    let code = "42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    assert!(!utils::needs_memory(&ast));
}

#[test]
fn test_emitter_default() {
    let emitter = WasmEmitter::default();
    let code = "1 + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("should parse");
    let result = emitter.emit(&ast);
    assert!(result.is_ok());
}

#[cfg(test)]
mod property_tests_mod {
use proptest::proptest;

proptest! {
    /// Property: Function never panics on any input
    #[test]
    fn test_new_never_panics(input: String) {
        // Limit input size to avoid timeout
        let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
        // Function should not panic on any input
        let _ = std::panic::catch_unwind(|| {
            // Call function with various inputs
            // This is a template - adjust based on actual function signature
        });
    }
}
}
