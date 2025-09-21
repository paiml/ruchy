//! Example: Compile Ruchy code to WebAssembly
//!
//! This example demonstrates how to use the Ruchy compiler to generate
//! WebAssembly modules from Ruchy source code.
//!
//! Run with: cargo run --example wasm_compile

use ruchy::{Parser, WasmEmitter};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Ruchy to WebAssembly Compilation Example\n");

    // Example 1: Simple expression
    compile_example("Simple Expression", "42 + 58", "simple.wasm")?;

    // Example 2: Function definition
    compile_example(
        "Function Definition",
        r#"
        fun add(a, b) {
            a + b
        }
        add(10, 20)
        "#,
        "function.wasm",
    )?;

    // Example 3: Multiple operations
    compile_example(
        "Multiple Operations",
        r#"
        fun multiply(x, y) {
            x * y
        }
        
        multiply(8, 8)
        "#,
        "multiple.wasm",
    )?;

    // Example 4: Mathematical expression
    compile_example(
        "Mathematical Expression",
        r#"
        let x = 10
        let y = 20
        x * y + 50
        "#,
        "math.wasm",
    )?;

    println!("\n✅ All examples compiled successfully!");
    println!("📁 WASM files saved to current directory");

    Ok(())
}

fn compile_example(
    name: &str,
    source: &str,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Compiling: {}", name);
    println!("   Source: {}", source.lines().next().unwrap_or("").trim());

    // Parse the source
    let mut parser = Parser::new(source);
    let ast = parser.parse().map_err(|e| format!("Parse error: {}", e))?;

    // Generate WASM
    let emitter = WasmEmitter::new();
    let wasm_bytes = emitter
        .emit(&ast)
        .map_err(|e| format!("WASM generation error: {}", e))?;

    // Validate WASM
    wasmparser::validate(&wasm_bytes).map_err(|e| format!("WASM validation error: {}", e))?;

    // Save to file
    fs::write(output_file, &wasm_bytes)?;

    println!(
        "   ✓ Generated {} ({} bytes)",
        output_file,
        wasm_bytes.len()
    );

    Ok(())
}
