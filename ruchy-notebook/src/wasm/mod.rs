use wasm_bindgen::prelude::*;
use crate::vm::{VirtualMachine, BytecodeModule, Compiler};

#[wasm_bindgen]
pub struct WasmNotebook {
    vm: VirtualMachine,
}

#[wasm_bindgen]
impl WasmNotebook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            vm: VirtualMachine::new(),
        }
    }
    
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> Result<String, JsValue> {
        let mut compiler = Compiler::new();
        
        let module = compiler.compile_expression(code)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        let result = self.vm.execute(&module)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        if !result.output.is_empty() {
            Ok(result.output.join("\n"))
        } else if let Some(value) = result.value {
            Ok(format!("{:?}", value))
        } else {
            Ok("".to_string())
        }
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.vm = VirtualMachine::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    #[wasm_bindgen_test]
    fn test_wasm_notebook_creation() {
        let notebook = WasmNotebook::new();
        assert!(true); // Just verify it compiles
    }
}