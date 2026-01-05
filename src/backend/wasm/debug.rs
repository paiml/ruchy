#[cfg(test)]
mod debug_tests {
    use super::super::*;
    use crate::frontend::ast::{Expr, ExprKind};
    #[test]
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
        assert_eq!(
            &bytes[4..8],
            &[1, 0, 0, 0],
            "Should have WASM version 1"
        );
    }
}
