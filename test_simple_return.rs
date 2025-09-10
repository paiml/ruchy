extern crate ruchy;
extern crate wasmparser;

use ruchy::backend::wasm::WasmEmitter;
use ruchy::frontend::parser::Parser;

fn main() {
    let source = "return 42";
    let mut parser = Parser::new(source);
    let ast = parser.parse().expect("Should parse");
    
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter.emit(&ast).expect("Should emit");
    
    println!("WASM bytes: {} total", wasm_bytes.len());
    for (i, b) in wasm_bytes.iter().enumerate() {
        print!("{:02x} ", b);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();
    
    // Validate
    let mut validator = wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all());
    match validator.validate_all(&wasm_bytes) {
        Ok(()) => println!("✅ Valid WASM"),
        Err(e) => println!("❌ Invalid: {}", e),
    }
}
