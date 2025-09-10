use ruchy::backend::wasm::WasmEmitter;
use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{Expr, ExprKind};

fn main() {
    // Test empty module
    let emitter = WasmEmitter::new();
    let expr = Expr::new(ExprKind::Block(vec![]), Default::default());
    
    match emitter.emit(&expr) {
        Ok(bytes) => {
            println!("Generated {} bytes", bytes.len());
            println!("Magic: {:?}", &bytes[0..4]);
            println!("Version: {:?}", &bytes[4..8]);
            
            // Print hex dump
            for (i, chunk) in bytes.chunks(16).enumerate() {
                print!("{:04x}: ", i * 16);
                for byte in chunk {
                    print!("{:02x} ", byte);
                }
                println!();
            }
            
            // Try to validate
            match wasmparser::validate(&bytes) {
                Ok(()) => println!("✅ Valid WASM!"),
                Err(e) => println!("❌ Validation error: {}", e),
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}