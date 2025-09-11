use ruchy::backend::wasm::WasmEmitter;
use ruchy::frontend::parser::Parser;
use wasmparser::{Validator, WasmFeatures};

fn main() {
    // Test the simple return case
    let source = "fun early_return(x) { if x > 10 { return 42 } return 0 }";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse return");
    
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit return");
    
    println!("Generated WASM module size: {} bytes", wasm_bytes.len());
    
    // Try to validate
    let mut validator = Validator::new_with_features(WasmFeatures::all());
    match validator.validate_all(&wasm_bytes) {
        Ok(_) => println!("✓ Module is valid"),
        Err(e) => println!("✗ Validation error: {}", e),
    }
    
    // Let's also try to examine the binary
    println!("First 20 bytes: {:02x?}", &wasm_bytes[..20.min(wasm_bytes.len())]);
}