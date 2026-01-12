#[cfg(test)]
mod debug_tests {
    use super::super::*;
    use crate::frontend::ast::{Expr, ExprKind};
    #[test]
    #[cfg(feature = "notebook")]
    fn debug_empty_module() {
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        println!("Generated {} bytes", bytes.len());
        // Print sections
        let mut offset = 8; // Skip magic and version
        while offset < bytes.len() {
            let section_id = bytes[offset];
            println!("Section at {offset}: ID={section_id} ({section_id:02x})");
            offset += 1;
            // Read section size (LEB128)
            let mut size = 0u32;
            let mut shift = 0;
            loop {
                let byte = bytes[offset];
                offset += 1;
                size |= u32::from(byte & 0x7f) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            println!("  Size: {size}");
            offset += size as usize;
        }
        // Validate
        match wasmparser::validate(&bytes) {
            Ok(_types) => println!("✅ Valid WASM"),
            Err(e) => panic!("❌ Invalid: {e}"),
        }
    }

    // === EXTREME TDD Round 17 tests ===

    #[test]
    fn test_wasm_emitter_new() {
        let emitter = WasmEmitter::new();
        // Emitter should be created successfully
        // Just verify it doesn't panic
        let _ = emitter;
    }

    #[test]
    fn test_wasm_magic_and_version() {
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");

        // WASM magic number: \0asm
        assert_eq!(&bytes[0..4], b"\0asm", "Should have WASM magic number");

        // WASM version 1
        assert_eq!(&bytes[4..8], &[1, 0, 0, 0], "Should have WASM version 1");
    }

    // === EXTREME TDD Round 124 tests ===

    #[test]
    fn test_emit_integer_zero() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(0, None)),
            Default::default(),
        );
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..4], b"\0asm");
    }

    #[test]
    fn test_emit_integer_negative() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(-42, None)),
            Default::default(),
        );
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_integer_max() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Integer(i32::MAX as i64, None)),
            Default::default(),
        );
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_float_zero() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Float(0.0)), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_float_pi() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(
            ExprKind::Literal(Literal::Float(3.14159)),
            Default::default(),
        );
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_bool_true() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Bool(true)), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_bool_false() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Bool(false)), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_unit_literal() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Literal(Literal::Unit), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_empty_block() {
        let emitter = WasmEmitter::new();
        let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_single_item_block() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let inner = Expr::new(
            ExprKind::Literal(Literal::Integer(42, None)),
            Default::default(),
        );
        let expr = Expr::new(ExprKind::Block(vec![inner]), Default::default());
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_emit_multi_item_block() {
        use crate::frontend::ast::Literal;
        let emitter = WasmEmitter::new();
        let item1 = Expr::new(
            ExprKind::Literal(Literal::Integer(1, None)),
            Default::default(),
        );
        let item2 = Expr::new(
            ExprKind::Literal(Literal::Integer(2, None)),
            Default::default(),
        );
        let item3 = Expr::new(
            ExprKind::Literal(Literal::Integer(3, None)),
            Default::default(),
        );
        let expr = Expr::new(
            ExprKind::Block(vec![item1, item2, item3]),
            Default::default(),
        );
        let bytes = emitter.emit(&expr).expect("Should emit");
        assert!(!bytes.is_empty());
    }
}
