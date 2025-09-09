#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::vm::{VirtualMachine, BytecodeModule, Compiler};

#[cfg(feature = "wasm")]
use wee_alloc;

#[cfg(feature = "wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = performance, js_name = now)]
    fn performance_now() -> f64;
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[cfg(feature = "wasm")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmNotebook {
    vm: VirtualMachine,
    start_time: f64,
    memory_used: usize,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct ExecutionResult {
    output: String,
    success: bool,
    execution_time_ms: f64,
    memory_used: usize,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ExecutionResult {
    #[wasm_bindgen(getter)]
    pub fn output(&self) -> String { self.output.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool { self.success }
    
    #[wasm_bindgen(getter)]
    pub fn execution_time_ms(&self) -> f64 { self.execution_time_ms }
    
    #[wasm_bindgen(getter)]
    pub fn memory_used(&self) -> usize { self.memory_used }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmNotebook {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        init_panic_hook();
        console_log!("Initializing Ruchy Notebook WASM runtime");
        
        Self {
            vm: VirtualMachine::new(),
            start_time: performance_now(),
            memory_used: 0,
        }
    }
    
    #[wasm_bindgen]
    pub fn execute(&mut self, code: &str) -> ExecutionResult {
        let start = performance_now();
        
        let mut compiler = Compiler::new();
        
        let result = match compiler.compile_expression(code) {
            Ok(module) => {
                match self.vm.execute(&module) {
                    Ok(exec_result) => {
                        let output = if !exec_result.output.is_empty() {
                            exec_result.output.join("\n")
                        } else if let Some(value) = exec_result.value {
                            format!("{:?}", value)
                        } else {
                            String::new()
                        };
                        
                        ExecutionResult {
                            output,
                            success: true,
                            execution_time_ms: performance_now() - start,
                            memory_used: self.memory_used,
                        }
                    }
                    Err(e) => {
                        console_log!("Runtime error: {}", e);
                        ExecutionResult {
                            output: format!("RuntimeError: {}", e),
                            success: false,
                            execution_time_ms: performance_now() - start,
                            memory_used: self.memory_used,
                        }
                    }
                }
            }
            Err(e) => {
                console_log!("Compilation error: {}", e);
                ExecutionResult {
                    output: format!("CompileError: {}", e),
                    success: false,
                    execution_time_ms: performance_now() - start,
                    memory_used: self.memory_used,
                }
            }
        };
        
        result
    }
    
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.vm = VirtualMachine::new();
        self.start_time = performance_now();
        self.memory_used = 0;
        console_log!("Notebook runtime reset");
    }
    
    #[wasm_bindgen]
    pub fn get_memory_usage(&self) -> usize {
        self.memory_used
    }
    
    #[wasm_bindgen]
    pub fn get_runtime_ms(&self) -> f64 {
        performance_now() - self.start_time
    }
}

// WebWorker message handling
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WorkerMessage {
    id: String,
    code: String,
    timeout_ms: Option<u32>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WorkerMessage {
    #[wasm_bindgen(constructor)]
    pub fn new(id: String, code: String, timeout_ms: Option<u32>) -> Self {
        Self { id, code, timeout_ms }
    }
    
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String { self.code.clone() }
    
    #[wasm_bindgen(getter)]
    pub fn timeout_ms(&self) -> Option<u32> { self.timeout_ms }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn handle_worker_message(msg: &WorkerMessage) -> ExecutionResult {
    let mut notebook = WasmNotebook::new();
    
    // TODO: Implement timeout using web APIs
    if let Some(_timeout) = msg.timeout_ms() {
        console_log!("Executing with timeout: {}ms", _timeout);
    }
    
    notebook.execute(&msg.code())
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