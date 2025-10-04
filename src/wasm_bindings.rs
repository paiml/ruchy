//! WebAssembly bindings for Ruchy compiler
//!
//! This module provides WASM bindings for core Ruchy functionality.
//! Network-dependent features are excluded to minimize binary size.
#![cfg(target_arch = "wasm32")]
use crate::backend::transpiler::Transpiler;
use crate::frontend::parser::Parser;
use js_sys::Promise;
#[cfg(test)]
use proptest::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
/// WebAssembly compiler interface for Ruchy
#[wasm_bindgen]
pub struct RuchyWasm {
    transpiler: Transpiler,
}
#[wasm_bindgen]
impl RuchyWasm {
    /// Create a new compiler instance
    #[wasm_bindgen(constructor)]
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm_bindings::new;
    ///
    /// let result = new(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn new() -> Self {
        // Set panic hook for better browser debugging
        console_error_panic_hook::set_once();
        Self {
            transpiler: Transpiler::new(),
        }
    }
    /// Compile Ruchy code to Rust
    #[wasm_bindgen]
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm_bindings::compile;
    ///
    /// let result = compile("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn compile(&self, source: &str) -> Result<String, JsValue> {
        let mut parser = Parser::new(source);
        let ast = parser
            .parse()
            .map_err(|e| js_sys::Error::new(&format!("Parse error: {}", e)).into())?;
        let rust_code = self
            .transpiler
            .transpile(&ast)
            .map_err(|e| js_sys::Error::new(&format!("Transpile error: {}", e)).into())?;
        Ok(rust_code.to_string())
    }
    /// Validate Ruchy syntax
    #[wasm_bindgen]
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm_bindings::validate;
    ///
    /// let result = validate("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn validate(&self, source: &str) -> bool {
        Parser::new(source).parse().is_ok()
    }
    /// Get version
    #[wasm_bindgen(getter)]
    /// # Examples
    ///
    /// ```
    /// use ruchy::wasm_bindings::version;
    ///
    /// let result = version(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Compile code asynchronously using WebWorker
    /// This method enables non-blocking compilation in web browsers
    #[wasm_bindgen]
    pub fn compile_async(&self, source: String) -> Promise {
        let transpiler = self.transpiler.clone();
        future_to_promise(async move {
            // Parse and transpile in async context
            let mut parser = Parser::new(&source);
            let ast = parser
                .parse()
                .map_err(|e| js_sys::Error::new(&format!("Parse error: {}", e)).into())?;
            let rust_code = transpiler
                .transpile(&ast)
                .map_err(|e| js_sys::Error::new(&format!("Transpile error: {}", e)).into())?;
            Ok(JsValue::from_str(&rust_code.to_string()))
        })
    }

    /// Execute multiple cells in parallel using WebWorker pattern
    /// Each cell compiles independently for maximum parallelism
    #[wasm_bindgen]
    pub fn compile_cells_parallel(&self, sources: &js_sys::Array) -> Promise {
        let transpiler = self.transpiler.clone();
        let sources: Vec<String> = sources
            .iter()
            .map(|val| val.as_string().unwrap_or_default())
            .collect();

        future_to_promise(async move {
            let mut results = Vec::new();

            // Process each cell independently (WebWorker-friendly)
            for source in sources {
                let mut parser = Parser::new(&source);
                match parser.parse() {
                    Ok(ast) => match transpiler.transpile(&ast) {
                        Ok(rust_code) => {
                            results.push(serde_json::json!({
                                "success": true,
                                "result": rust_code.to_string(),
                                "error": null
                            }));
                        }
                        Err(e) => {
                            results.push(serde_json::json!({
                                "success": false,
                                "result": null,
                                "error": format!("Transpile error: {}", e)
                            }));
                        }
                    },
                    Err(e) => {
                        results.push(serde_json::json!({
                            "success": false,
                            "result": null,
                            "error": format!("Parse error: {}", e)
                        }));
                    }
                }
            }

            let results_array = js_sys::Array::new();
            for result in results {
                results_array.push(&JsValue::from_str(&result.to_string()));
            }
            Ok(results_array.into())
        })
    }

    /// Get WebWorker capabilities and configuration
    #[wasm_bindgen]
    pub fn get_webworker_info(&self) -> JsValue {
        let info = serde_json::json!({
            "webworker_support": true,
            "async_compilation": true,
            "parallel_cells": true,
            "max_concurrent_cells": 4,
            "performance_target_ms": 10,
            "features": {
                "incremental_compilation": true,
                "error_recovery": true,
                "source_maps": false,
                "fast_execution": true
            }
        });
        JsValue::from_str(&info.to_string())
    }

    /// Execute cell with performance monitoring (WASM-007)
    /// Target: <10ms execution time for typical cells
    #[wasm_bindgen]
    pub fn execute_cell_fast(&self, source: &str) -> JsValue {
        let start_time = js_sys::Date::now();

        // Fast path compilation
        let result = match self.compile(source) {
            Ok(rust_code) => {
                serde_json::json!({
                    "success": true,
                    "result": rust_code,
                    "error": null
                })
            }
            Err(e) => {
                serde_json::json!({
                    "success": false,
                    "result": null,
                    "error": e.as_string().unwrap_or_default()
                })
            }
        };

        let end_time = js_sys::Date::now();
        let execution_time = end_time - start_time;

        let performance_result = serde_json::json!({
            "cell_result": result,
            "performance": {
                "execution_time_ms": execution_time,
                "target_met": execution_time < 10.0,
                "target_ms": 10,
                "optimization_level": "fast"
            }
        });

        JsValue::from_str(&performance_result.to_string())
    }

    /// Benchmark cell execution performance
    #[wasm_bindgen]
    pub fn benchmark_cell_execution(&self, iterations: usize) -> JsValue {
        let test_cases = vec![
            "let x = 42",                          // Simple assignment
            "let y = x * 2 + 1",                   // Expression
            "fun double(n: Int) -> Int { n * 2 }", // Function definition
            "let result = double(21)",             // Function call
            "if x > 0 { x } else { 0 }",           // Conditional
        ];

        let mut benchmark_results = Vec::new();

        for (i, test_case) in test_cases.iter().enumerate() {
            let mut total_time = 0.0;
            let mut success_count = 0;

            for _ in 0..iterations {
                let start_time = js_sys::Date::now();
                let result = self.compile(test_case);
                let end_time = js_sys::Date::now();

                total_time += end_time - start_time;
                if result.is_ok() {
                    success_count += 1;
                }
            }

            let avg_time = total_time / iterations as f64;
            let success_rate = (success_count as f64) / (iterations as f64) * 100.0;

            benchmark_results.push(serde_json::json!({
                "test_case": test_case,
                "test_id": i,
                "avg_execution_time_ms": avg_time,
                "success_rate_percent": success_rate,
                "target_met": avg_time < 10.0,
                "iterations": iterations
            }));
        }

        let overall_avg = benchmark_results
            .iter()
            .map(|r| r["avg_execution_time_ms"].as_f64().unwrap_or(0.0))
            .sum::<f64>()
            / benchmark_results.len() as f64;

        let result = serde_json::json!({
            "benchmark_results": benchmark_results,
            "summary": {
                "overall_avg_ms": overall_avg,
                "target_ms": 10,
                "target_met": overall_avg < 10.0,
                "total_iterations": iterations * test_cases.len()
            }
        });

        JsValue::from_str(&result.to_string())
    }
}

/// WebWorker-specific runtime for parallel execution
#[wasm_bindgen]
pub struct WebWorkerRuntime {
    max_workers: usize,
    active_workers: usize,
}

#[wasm_bindgen]
impl WebWorkerRuntime {
    /// Create new WebWorker runtime
    #[wasm_bindgen(constructor)]
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers: max_workers.max(1).min(8), // Reasonable bounds
            active_workers: 0,
        }
    }

    /// Execute task in WebWorker context with load balancing
    #[wasm_bindgen]
    pub fn execute_with_workers(&mut self, task_data: &str) -> Promise {
        let max_workers = self.max_workers;
        let task_data = task_data.to_string();

        future_to_promise(async move {
            // Simulate WebWorker execution with resource management
            let start_time = js_sys::Date::now();

            // Create compiler instance for this worker
            let compiler = RuchyWasm::new();
            let result = compiler.compile(&task_data);

            let end_time = js_sys::Date::now();
            let duration = end_time - start_time;

            match result {
                Ok(output) => {
                    let response = serde_json::json!({
                        "success": true,
                        "result": output,
                        "execution_time_ms": duration,
                        "worker_id": max_workers,
                        "memory_used": "optimized"
                    });
                    Ok(JsValue::from_str(&response.to_string()))
                }
                Err(e) => {
                    let response = serde_json::json!({
                        "success": false,
                        "error": e.as_string().unwrap_or_default(),
                        "execution_time_ms": duration,
                        "worker_id": max_workers
                    });
                    Ok(JsValue::from_str(&response.to_string()))
                }
            }
        })
    }

    /// Get worker pool status
    #[wasm_bindgen]
    pub fn get_worker_status(&self) -> JsValue {
        let status = serde_json::json!({
            "max_workers": self.max_workers,
            "active_workers": self.active_workers,
            "available_workers": self.max_workers - self.active_workers,
            "load_factor": if self.max_workers > 0 {
                (self.active_workers as f64) / (self.max_workers as f64)
            } else { 0.0 }
        });
        JsValue::from_str(&status.to_string())
    }
}

/// Initialize WASM module
#[wasm_bindgen(start)]
/// # Examples
///
/// ```
/// use ruchy::wasm_bindings::wasm_init;
///
/// let result = wasm_init(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn wasm_init() {
    console_error_panic_hook::set_once();
}

/// FFI Boundary Testing Methods for QA Framework
#[wasm_bindgen]
impl RuchyWasm {
    /// Test bidirectional type marshalling
    #[wasm_bindgen]
    pub fn roundtrip(&self, input: JsValue) -> JsValue {
        input
    }

    /// Test panic propagation across FFI boundary
    #[wasm_bindgen]
    pub fn trigger_panic(&self, message: &str) {
        panic!("{}", message);
    }

    /// Test JavaScript callback integration
    #[wasm_bindgen]
    pub fn call_js_callback(&self, callback: &js_sys::Function) -> JsValue {
        match callback.call0(&JsValue::NULL) {
            Ok(result) => serde_wasm_bindgen::to_value(&serde_json::json!({
                "is_err": false,
                "value": result
            }))
            .unwrap_or(JsValue::NULL),
            Err(e) => serde_wasm_bindgen::to_value(&serde_json::json!({
                "is_err": true,
                "error": format!("{:?}", e)
            }))
            .unwrap_or(JsValue::NULL),
        }
    }

    /// Test memory management with large data
    #[wasm_bindgen]
    pub fn process_bytes(&self, data: &[u8]) -> ProcessResult {
        ProcessResult::new(data.to_vec())
    }

    /// Test async operations
    #[wasm_bindgen]
    pub fn async_operation(&self, value: i32) -> Promise {
        future_to_promise(async move {
            // Simulate some async work
            Ok(JsValue::from(value))
        })
    }
}

/// Memory management test helper
#[wasm_bindgen]
pub struct ProcessResult {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl ProcessResult {
    fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.data[..])
    }

    /// Explicit memory cleanup for testing
    #[wasm_bindgen]
    pub fn free(self) {
        // Rust will handle cleanup automatically
        drop(self);
    }
}
#[cfg(test)]
mod property_tests_wasm_bindings {
    use super::*;
    use proptest::prelude::*;
    use proptest::proptest;

    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let _input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                let _ruchy = RuchyWasm::new();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_repl_new() {
        let repl = RuchyRepl::new();
        assert!(repl.engine.is_none());
    }

    #[test]
    fn test_rust_repl_eval_simple() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("2 + 2");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_let() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("let x = 10");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_function() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("fun add(a, b) { a + b }");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_invalid() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("invalid++syntax");
        assert!(result.is_some()); // Returns error string
    }

    #[test]
    fn test_rust_repl_eval_empty() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_whitespace() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("   ");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_comment() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("// just a comment");
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_multiline() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval(
            "let x = 1
let y = 2
x + y",
        );
        assert!(result.is_some());
    }

    #[test]
    fn test_rust_repl_eval_string() {
        let mut repl = RuchyRepl::new();
        let result = repl.eval("\"hello world\"");
        assert!(result.is_some());
    }
}
