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
            println!("Section at {}: ID={} ({:02x})", offset, section_id, section_id);
            offset += 1;
            
            // Read section size (LEB128)
            let mut size = 0u32;
            let mut shift = 0;
            loop {
                let byte = bytes[offset];
                offset += 1;
                size |= ((byte & 0x7f) as u32) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            println!("  Size: {}", size);
            offset += size as usize;
        }
        
        // Validate
        match wasmparser::validate(&bytes) {
            Ok(_types) => println!("✅ Valid WASM"),
            Err(e) => panic!("❌ Invalid: {}", e),
        }
    }
}