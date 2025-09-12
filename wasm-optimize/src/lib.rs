use wasm_bindgen::prelude::*;
use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

#[wasm_bindgen]
pub struct RuchyCompiler {
    transpiler: Transpiler,
}

#[wasm_bindgen]
impl RuchyCompiler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            transpiler: Transpiler::new(),
        }
    }

    /// Compile Ruchy code to Rust
    #[wasm_bindgen]
    pub fn compile(&self, source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser.parse()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        let rust_code = self.transpiler.transpile(&ast)
            .map_err(|e| JsValue::from_str(&format!("Transpile error: {}", e)))?;
        
        Ok(rust_code.to_string())
    }

    /// Parse Ruchy code and return AST as JSON
    #[wasm_bindgen]
    pub fn parse(&self, source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser.parse()
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        // Serialize AST to JSON
        serde_json::to_string_pretty(&ast)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }

    /// Validate Ruchy syntax
    #[wasm_bindgen]
    pub fn validate(&self, source: &str) -> bool {
        let mut parser = Parser::new(source);
        parser.parse().is_ok()
    }
}

/// Simple version info
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}