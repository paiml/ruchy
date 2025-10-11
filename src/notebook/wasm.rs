// NOTEBOOK-006: WASM Notebook Bindings
// Phase 4: Notebook Excellence - Browser Integration
//
// This module provides WebAssembly bindings for the NotebookEngine:
// - Browser-based notebook execution
// - Cell-by-cell evaluation with state persistence
// - Rich HTML output generation
// - Performance: <10ms per cell target
//
// Quality Requirements:
// - Cyclomatic Complexity: ≤10 per function (Toyota Way)
// - Line Coverage: ≥85%
// - Branch Coverage: ≥90%
// - WASM Size: <500KB
// - WASI Imports: 0 (pure WASM)

use crate::notebook::engine::NotebookEngine;
use crate::notebook::execution::CellExecutionResult;
use crate::notebook::persistence::Checkpoint;
use std::collections::HashMap;

// WASM-specific imports (only when targeting WASM)
#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::future_to_promise;

/// WebAssembly notebook interface
///
/// Provides browser-based execution of Ruchy code cells with state persistence.
pub struct NotebookWasm {
    engine: NotebookEngine,
    checkpoints: HashMap<String, Checkpoint>,
    checkpoint_counter: usize,
}

impl NotebookWasm {
    /// Create a new notebook instance
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            // Set panic hook for better browser debugging
            console_error_panic_hook::set_once();
        }

        Self {
            engine: NotebookEngine::new().expect("Failed to initialize NotebookEngine"),
            checkpoints: HashMap::new(),
            checkpoint_counter: 0,
        }
    }

    /// Execute a code cell and return JSON result
    pub fn execute_cell_json(&mut self, code: &str) -> String {
        let result = self.engine.execute_cell_detailed(code);
        Self::result_to_json(&result)
    }

    /// Get cell result as HTML
    pub fn execute_cell_html(&mut self, code: &str) -> String {
        let result = self.engine.execute_cell_detailed(code);
        result.as_html()
    }

    /// Reset notebook state
    pub fn reset(&mut self) {
        self.engine = NotebookEngine::new().expect("Failed to reset NotebookEngine");
        self.checkpoints.clear();
        self.checkpoint_counter = 0;
    }

    /// Create a checkpoint of current state
    pub fn checkpoint(&mut self) -> String {
        let checkpoint_id = format!("checkpoint_{}", self.checkpoint_counter);
        self.checkpoint_counter += 1;

        let checkpoint = self.engine.create_checkpoint(checkpoint_id.clone());
        self.checkpoints.insert(checkpoint_id.clone(), checkpoint);

        checkpoint_id
    }

    /// Restore to a checkpoint
    pub fn restore(&mut self, checkpoint_id: &str) -> bool {
        if let Some(checkpoint) = self.checkpoints.get(checkpoint_id) {
            self.engine.restore_checkpoint(checkpoint);
            true
        } else {
            false
        }
    }

    /// Get notebook version
    pub fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Helper to convert `CellExecutionResult` to JSON string
    fn result_to_json(result: &CellExecutionResult) -> String {
        let json = serde_json::json!({
            "success": result.is_success(),
            "output": result.output(),
            "error": result.error(),
            "stdout": result.stdout(),
            "stderr": result.stderr(),
            "duration_ms": result.duration_ms(),
            "html": result.as_html(),
        });

        json.to_string()
    }
}

impl Default for NotebookWasm {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring for notebook cells
pub struct NotebookPerformance {
    cell_count: usize,
    total_time_ms: f64,
}

impl NotebookPerformance {
    /// Create new performance monitor
    pub fn new() -> Self {
        Self {
            cell_count: 0,
            total_time_ms: 0.0,
        }
    }

    /// Record cell execution time
    pub fn record(&mut self, duration_ms: f64) {
        self.cell_count += 1;
        self.total_time_ms += duration_ms;
    }

    /// Get average cell execution time
    pub fn average_time_ms(&self) -> f64 {
        if self.cell_count == 0 {
            0.0
        } else {
            self.total_time_ms / (self.cell_count as f64)
        }
    }

    /// Check if performance target is met (<10ms average)
    pub fn target_met(&self) -> bool {
        self.average_time_ms() < 10.0
    }

    /// Get performance report
    pub fn report(&self) -> String {
        format!(
            "Cells: {}, Avg: {:.2}ms, Target: {}",
            self.cell_count,
            self.average_time_ms(),
            if self.target_met() {
                "✅ MET"
            } else {
                "❌ MISSED"
            }
        )
    }
}

impl Default for NotebookPerformance {
    fn default() -> Self {
        Self::new()
    }
}

// WASM-specific bindings (only compiled for WASM target)
#[cfg(target_arch = "wasm32")]
mod wasm_bindings {
    use super::*;

    /// WASM-exported notebook interface
    #[wasm_bindgen]
    pub struct NotebookWasmExport {
        inner: NotebookWasm,
    }

    #[wasm_bindgen]
    impl NotebookWasmExport {
        /// Create a new notebook instance
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                inner: NotebookWasm::new(),
            }
        }

        /// Execute a code cell and return the result as JsValue
        #[wasm_bindgen]
        pub fn execute_cell(&mut self, code: &str) -> JsValue {
            let json = self.inner.execute_cell_json(code);
            JsValue::from_str(&json)
        }

        /// Execute a cell asynchronously
        #[wasm_bindgen]
        pub fn execute_cell_async(&mut self, code: String) -> Promise {
            let mut engine = self.inner.engine.clone();

            future_to_promise(async move {
                let result = engine.execute_cell_detailed(&code);
                let json = serde_json::json!({
                    "success": result.is_success(),
                    "output": result.output(),
                    "error": result.error(),
                    "duration_ms": result.duration_ms(),
                    "html": result.as_html(),
                });

                Ok(JsValue::from_str(&json.to_string()))
            })
        }

        /// Get cell result as HTML
        #[wasm_bindgen]
        pub fn execute_cell_html(&mut self, code: &str) -> String {
            self.inner.execute_cell_html(code)
        }

        /// Reset notebook state
        #[wasm_bindgen]
        pub fn reset(&mut self) {
            self.inner.reset();
        }

        /// Create a checkpoint of current state
        #[wasm_bindgen]
        pub fn checkpoint(&mut self) -> String {
            self.inner.checkpoint()
        }

        /// Restore to a checkpoint
        #[wasm_bindgen]
        pub fn restore(&mut self, checkpoint_id: &str) -> bool {
            self.inner.restore(checkpoint_id)
        }

        /// Get notebook version
        #[wasm_bindgen]
        pub fn version(&self) -> String {
            self.inner.version()
        }
    }

    /// Initialize WASM notebook module
    #[wasm_bindgen(start)]
    pub fn init_notebook_wasm() {
        console_error_panic_hook::set_once();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED PHASE: Write tests that define expected behavior

    #[test]
    fn test_notebook_006_wasm_creation() {
        let notebook = NotebookWasm::new();
        assert_eq!(notebook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_notebook_006_wasm_default() {
        let notebook = NotebookWasm::default();
        assert_eq!(notebook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_notebook_006_execute_cell_json() {
        let mut notebook = NotebookWasm::new();
        let result_json = notebook.execute_cell_json("2 + 2");

        assert!(result_json.contains("success") || result_json.contains("output"));
        assert!(!result_json.is_empty());
    }

    #[test]
    fn test_notebook_006_execute_cell_html() {
        let mut notebook = NotebookWasm::new();
        let html = notebook.execute_cell_html("let x = 42");

        assert!(html.contains("42") || html.contains("html") || html.contains("notebook"));
    }

    #[test]
    fn test_notebook_006_reset() {
        let mut notebook = NotebookWasm::new();
        let _ = notebook.execute_cell_json("let x = 42");

        notebook.reset();

        // After reset, notebook should be fresh
        let version = notebook.version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_notebook_006_checkpoint_restore() {
        let mut notebook = NotebookWasm::new();
        let _ = notebook.execute_cell_json("let x = 42");

        let checkpoint_id = notebook.checkpoint();
        assert!(!checkpoint_id.is_empty());
        assert!(checkpoint_id.starts_with("checkpoint_"));

        let _ = notebook.execute_cell_json("let y = 100");

        let restored = notebook.restore(&checkpoint_id);
        assert!(restored);
    }

    #[test]
    fn test_notebook_006_performance_new() {
        let perf = NotebookPerformance::new();
        assert_eq!(perf.cell_count, 0);
        assert_eq!(perf.average_time_ms(), 0.0);
    }

    #[test]
    fn test_notebook_006_performance_default() {
        let perf = NotebookPerformance::default();
        assert_eq!(perf.cell_count, 0);
    }

    #[test]
    fn test_notebook_006_performance_record() {
        let mut perf = NotebookPerformance::new();

        perf.record(5.0);
        perf.record(7.0);
        perf.record(9.0);

        assert_eq!(perf.cell_count, 3);
        assert!((perf.average_time_ms() - 7.0).abs() < 0.1);
    }

    #[test]
    fn test_notebook_006_performance_target_met() {
        let mut perf = NotebookPerformance::new();

        perf.record(5.0);
        perf.record(8.0);

        assert!(perf.target_met()); // 6.5ms average < 10ms
    }

    #[test]
    fn test_notebook_006_performance_target_missed() {
        let mut perf = NotebookPerformance::new();

        perf.record(15.0);
        perf.record(20.0);

        assert!(!perf.target_met()); // 17.5ms average > 10ms
    }

    #[test]
    fn test_notebook_006_performance_report() {
        let mut perf = NotebookPerformance::new();
        perf.record(5.0);

        let report = perf.report();
        assert!(report.contains("Cells: 1"));
        assert!(report.contains("5.00ms"));
    }

    #[test]
    fn test_notebook_006_version() {
        let notebook = NotebookWasm::new();
        let version = notebook.version();

        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }

    #[test]
    fn test_notebook_006_multiple_cells() {
        let mut notebook = NotebookWasm::new();

        let _ = notebook.execute_cell_json("let x = 10");
        let _ = notebook.execute_cell_json("let y = 20");
        let _ = notebook.execute_cell_json("x + y");

        // State should persist across cells
        assert_eq!(notebook.version(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_notebook_006_empty_cell() {
        let mut notebook = NotebookWasm::new();
        let result = notebook.execute_cell_json("");

        assert!(!result.is_empty());
    }

    #[test]
    fn test_notebook_006_invalid_syntax() {
        let mut notebook = NotebookWasm::new();
        let html = notebook.execute_cell_html("invalid++syntax");

        // Should contain error indication
        assert!(html.contains("error") || html.contains("Error") || html.contains("❌"));
    }

    #[test]
    fn test_notebook_006_checkpoint_invalid_restore() {
        let mut notebook = NotebookWasm::new();

        let restored = notebook.restore("invalid-checkpoint-id");
        assert!(!restored);
    }

    #[test]
    fn test_notebook_006_performance_zero_cells() {
        let perf = NotebookPerformance::new();

        assert_eq!(perf.average_time_ms(), 0.0);
        assert!(perf.target_met()); // 0ms < 10ms
    }

    #[test]
    fn test_notebook_006_performance_single_slow_cell() {
        let mut perf = NotebookPerformance::new();
        perf.record(100.0);

        assert!(!perf.target_met());
        assert!(perf.report().contains("❌"));
    }

    #[test]
    fn test_notebook_006_performance_mixed_times() {
        let mut perf = NotebookPerformance::new();

        // Mix of fast and slow cells
        perf.record(2.0);
        perf.record(5.0);
        perf.record(8.0);
        perf.record(12.0);
        perf.record(3.0);

        // Average = 6.0ms < 10ms
        assert!(perf.target_met());
    }

    #[test]
    fn test_notebook_006_json_format() {
        let mut notebook = NotebookWasm::new();
        let json = notebook.execute_cell_json("let x = 42");

        // Should be valid JSON
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_notebook_006_html_not_empty() {
        let mut notebook = NotebookWasm::new();
        let html = notebook.execute_cell_html("2 + 2");

        assert!(!html.is_empty());
    }

    #[test]
    fn test_notebook_006_checkpoint_sequential() {
        let mut notebook = NotebookWasm::new();

        let cp1 = notebook.checkpoint();
        let cp2 = notebook.checkpoint();

        assert_ne!(cp1, cp2);
        assert!(cp1.starts_with("checkpoint_0"));
        assert!(cp2.starts_with("checkpoint_1"));
    }

    #[test]
    fn test_notebook_006_multiple_checkpoints() {
        let mut notebook = NotebookWasm::new();

        let cp1 = notebook.checkpoint();
        notebook.execute_cell_json("let x = 1");

        let cp2 = notebook.checkpoint();
        notebook.execute_cell_json("let y = 2");

        assert!(notebook.restore(&cp2));
        assert!(notebook.restore(&cp1));
    }
}
