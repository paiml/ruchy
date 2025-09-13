//! Tests for WASM emitter module
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

use super::*;
use crate::frontend::ast::{BinaryOp, Expr, ExprKind, Literal, Span};

#[cfg(test)]
mod basic_tests {
    use super::*;

    fn create_test_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::new(0, 10),
            attributes: vec![],
        }
    }

    #[test]
    fn test_wasm_emitter_creation() {
        let emitter = WasmEmitter::new();
        // Should be able to create a WASM emitter
        let _ = emitter;
    }

    #[test]
    fn test_emit_integer_literal() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        
        // Basic WASM module should start with magic bytes
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_float_literal() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Float(3.14)));
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_boolean_literal_true() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Bool(true)));
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_boolean_literal_false() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Bool(false)));
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_string_literal() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::String("hello".to_string())));
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_binary_addition() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1)))),
            op: BinaryOp::Add,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2)))),
        });
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_binary_subtraction() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(5)))),
            op: BinaryOp::Subtract,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(3)))),
        });
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_binary_multiplication() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(6)))),
            op: BinaryOp::Multiply,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(7)))),
        });
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_emit_binary_division() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(8)))),
            op: BinaryOp::Divide,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2)))),
        });
        
        let result = emitter.emit(&expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }

    #[test]
    fn test_collect_functions_empty_expr() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
        
        let functions = emitter.collect_functions(&expr);
        // Simple literal should not contain function definitions
        assert!(functions.is_empty());
    }

    #[test]
    fn test_emit_to_instruction_integer() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(123)));
        
        let instructions = emitter.emit_to_instruction(&expr);
        assert!(instructions.is_ok());
        
        let instr_vec = instructions.unwrap();
        assert!(!instr_vec.is_empty());
    }

    #[test]
    fn test_emit_to_instruction_float() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Float(2.5)));
        
        let instructions = emitter.emit_to_instruction(&expr);
        assert!(instructions.is_ok());
        
        let instr_vec = instructions.unwrap();
        assert!(!instr_vec.is_empty());
    }

    #[test]
    fn test_emit_nested_binary_operations() {
        let emitter = WasmEmitter::new();
        // (1 + 2) * 3
        let nested_expr = create_test_expr(ExprKind::Binary {
            left: Box::new(create_test_expr(ExprKind::Binary {
                left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1)))),
                op: BinaryOp::Add,
                right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2)))),
            })),
            op: BinaryOp::Multiply,
            right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(3)))),
        });
        
        let result = emitter.emit(&nested_expr);
        assert!(result.is_ok());
        
        let wasm_bytes = result.unwrap();
        assert!(!wasm_bytes.is_empty());
        assert_eq!(&wasm_bytes[0..4], b"\x00asm");
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_wasm_module_validation_integer() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
        
        let wasm_bytes = emitter.emit(&expr).unwrap();
        
        // Basic validation: check WASM magic number and version
        assert_eq!(&wasm_bytes[0..4], b"\x00asm"); // Magic number
        assert_eq!(&wasm_bytes[4..8], b"\x01\x00\x00\x00"); // Version 1
    }

    #[test]
    fn test_wasm_module_has_correct_sections() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(100)));
        
        let wasm_bytes = emitter.emit(&expr).unwrap();
        
        // Should have proper WASM header
        assert!(wasm_bytes.len() >= 8); // At least magic + version
        
        // Validate that it's properly formed WASM
        let validation_result = wasmparser::validate(&wasm_bytes);
        assert!(validation_result.is_ok(), "Generated WASM should be valid");
    }

    #[test]
    fn test_wasm_module_export_validation() {
        let emitter = WasmEmitter::new();
        let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
        
        let wasm_bytes = emitter.emit(&expr).unwrap();
        
        // Parse and validate the module has expected exports
        let mut parser = wasmparser::Parser::new(0);
        let mut has_exports = false;
        
        for payload in parser.parse_all(&wasm_bytes) {
            match payload.unwrap() {
                wasmparser::Payload::ExportSection(exports) => {
                    has_exports = true;
                    // Should have at least the main export
                    assert!(exports.count() > 0);
                }
                _ => {}
            }
        }
        
        assert!(has_exports, "WASM module should have exports");
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_emit_integer_never_panics(value in i64::MIN..i64::MAX) {
            let emitter = WasmEmitter::new();
            let expr = create_test_expr(ExprKind::Literal(Literal::Integer(value)));
            
            let result = emitter.emit(&expr);
            // Should not panic and should produce valid WASM
            if let Ok(bytes) = result {
                prop_assert!(!bytes.is_empty());
                prop_assert_eq!(&bytes[0..4], b"\x00asm");
            }
        }

        #[test]
        fn test_emit_float_never_panics(value in -1000.0f64..1000.0f64) {
            let emitter = WasmEmitter::new();
            let expr = create_test_expr(ExprKind::Literal(Literal::Float(value)));
            
            let result = emitter.emit(&expr);
            // Should not panic
            if let Ok(bytes) = result {
                prop_assert!(!bytes.is_empty());
                prop_assert_eq!(&bytes[0..4], b"\x00asm");
            }
        }

        #[test]
        fn test_emit_boolean_never_panics(value in any::<bool>()) {
            let emitter = WasmEmitter::new();
            let expr = create_test_expr(ExprKind::Literal(Literal::Bool(value)));
            
            let result = emitter.emit(&expr);
            if let Ok(bytes) = result {
                prop_assert!(!bytes.is_empty());
                prop_assert_eq!(&bytes[0..4], b"\x00asm");
            }
        }

        #[test]
        fn test_emit_string_never_panics(value in "[a-zA-Z0-9 ]{0,100}") {
            let emitter = WasmEmitter::new();
            let expr = create_test_expr(ExprKind::Literal(Literal::String(value)));
            
            let result = emitter.emit(&expr);
            if let Ok(bytes) = result {
                prop_assert!(!bytes.is_empty());
                prop_assert_eq!(&bytes[0..4], b"\x00asm");
            }
        }

        #[test]
        fn test_binary_operations_never_panic(
            left in -100i64..100i64,
            right in 1i64..100i64, // Avoid division by zero
            op_choice in 0u8..4u8
        ) {
            let emitter = WasmEmitter::new();
            let op = match op_choice {
                0 => BinaryOp::Add,
                1 => BinaryOp::Subtract,
                2 => BinaryOp::Multiply,
                _ => BinaryOp::Divide,
            };
            
            let expr = create_test_expr(ExprKind::Binary {
                left: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(left)))),
                op,
                right: Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(right)))),
            });
            
            let result = emitter.emit(&expr);
            if let Ok(bytes) = result {
                prop_assert!(!bytes.is_empty());
                prop_assert_eq!(&bytes[0..4], b"\x00asm");
            }
        }
    }
}

// Helper function used in tests
fn create_test_expr(kind: ExprKind) -> Expr {
    Expr {
        kind,
        span: Span::new(0, 10),
        attributes: vec![],
    }
}