//! WebAssembly bindings for Ruchy
//!
//! This crate provides WASM bindings for the Ruchy compiler, enabling:
//! - Browser-based Ruchy compilation
//! - Interactive code playgrounds
//! - Educational tools and documentation
//! - Real-time syntax validation
//!
//! # Example
//!
//! ```javascript
//! import init, { RuchyCompiler } from './ruchy_wasm.js';
//!
//! async function compile() {
//!     await init();
//!     const compiler = new RuchyCompiler();
//!     const rustCode = compiler.compile('fn add(a, b) { a + b }');
//!     console.log(rustCode);
//! }
//! ```

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;
use wasm_bindgen::prelude::*;

/// WebAssembly compiler interface for Ruchy
#[wasm_bindgen]
pub struct RuchyCompiler {
    transpiler: Transpiler,
}

#[wasm_bindgen]
impl RuchyCompiler {
    /// Create a new Ruchy compiler instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Set panic hook for better browser debugging
        console_error_panic_hook::set_once();

        Self {
            transpiler: Transpiler::new(),
        }
    }

    /// Compile Ruchy code to Rust
    ///
    /// # Arguments
    ///
    /// * `source` - Ruchy source code as a string
    ///
    /// # Returns
    ///
    /// Transpiled Rust code as a string, or error message
    #[wasm_bindgen]
    pub fn compile(&mut self, source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        let rust_code = self
            .transpiler
            .transpile(&ast)
            .map_err(|e| JsValue::from_str(&format!("Transpile error: {}", e)))?;

        Ok(rust_code.to_string())
    }

    /// Validate Ruchy syntax without compilation
    ///
    /// # Arguments
    ///
    /// * `source` - Ruchy source code to validate
    ///
    /// # Returns
    ///
    /// `true` if syntax is valid, `false` otherwise
    #[wasm_bindgen]
    pub fn validate(&self, source: &str) -> bool {
        Parser::new(source).parse().is_ok()
    }

    /// Get Ruchy compiler version
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Parse Ruchy code and return AST as JSON
    ///
    /// # Arguments
    ///
    /// * `source` - Ruchy source code to parse
    ///
    /// # Returns
    ///
    /// AST representation as JSON string
    #[wasm_bindgen]
    pub fn parse_to_json(&self, source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

        serde_json::to_string_pretty(&ast)
            .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
    }
}

impl Default for RuchyCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_compile_simple_function() {
        let mut compiler = RuchyCompiler::new();
        let result = compiler.compile("fn add(a, b) { a + b }");
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_valid_syntax() {
        let mut compiler = RuchyCompiler::new();
        assert!(compiler.validate("let x = 42"));
    }

    #[wasm_bindgen_test]
    fn test_validate_invalid_syntax() {
        let mut compiler = RuchyCompiler::new();
        assert!(!compiler.validate("let x = "));
    }

    #[wasm_bindgen_test]
    fn test_version() {
        let mut compiler = RuchyCompiler::new();
        assert!(!compiler.version().is_empty());
    }
}
