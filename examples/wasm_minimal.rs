//! Minimal WASM example showing core Ruchy functionality
//!
//! This demonstrates the smallest possible WASM module
//! that can parse and transpile Ruchy code.

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

fn main() {
    // Example Ruchy code
    let source = r"
        fun factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    ";

    // Parse the code
    let mut parser = Parser::new(source);
    match parser.parse() {
        Ok(ast) => {
            println!("Parse successful!");

            // Transpile to Rust
            let transpiler = Transpiler::new();
            match transpiler.transpile(&ast) {
                Ok(rust_code) => {
                    println!("Transpilation successful!");
                    println!("Generated Rust code:");
                    println!("{rust_code}");
                }
                Err(e) => {
                    eprintln!("Transpilation error: {e}");
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {e}");
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn compile_ruchy(source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let transpiler = Transpiler::new();
        let rust_code = transpiler
            .transpile(&ast)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(rust_code.to_string())
    }
}
