//! WebAssembly REPL Core Implementation
//!
//! Contains the `WasmRepl` struct and its core methods.

use crate::runtime::{Interpreter, Value};
use crate::wasm::helpers::{generate_session_id, get_timestamp, JsValue};
use crate::wasm::output::{ReplOutput, TimingInfo};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WebAssembly REPL for browser-based Ruchy evaluation
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WasmRepl {
    pub(crate) bindings: HashMap<String, String>,
    pub(crate) history: Vec<String>,
    session_id: String,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl WasmRepl {
    /// Create a new WASM REPL instance
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn new() -> Result<WasmRepl, JsValue> {
        #[cfg(target_arch = "wasm32")]
        console_error_panic_hook::set_once();

        Ok(WasmRepl {
            bindings: HashMap::new(),
            history: Vec::new(),
            session_id: generate_session_id(),
        })
    }

    /// Evaluate a Ruchy expression
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn eval(&mut self, input: &str) -> Result<String, JsValue> {
        let start = get_timestamp();

        // Parse the input
        let parse_start = get_timestamp();
        let mut parser = crate::Parser::new(input);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                let timing = TimingInfo::with_eval(
                    get_timestamp() - parse_start,
                    0.0,
                    get_timestamp() - start,
                );
                let output = ReplOutput::parse_error(e.to_string(), timing);
                return Ok(serde_json::to_string(&output)
                    .unwrap_or_else(|_| "Error serializing output".to_string()));
            }
        };
        let parse_time = get_timestamp() - parse_start;

        // Evaluate using interpreter
        let eval_start = get_timestamp();
        let mut interpreter = Interpreter::new();

        // Clear OUTPUT_BUFFER before evaluation
        if let Ok(mut buf) = crate::runtime::builtins::OUTPUT_BUFFER.lock() {
            buf.clear();
        }

        // Evaluate the expression
        let result = match interpreter.eval_expr(&ast) {
            Ok(value) => Self::format_value_for_display(&value),
            Err(e) => {
                let timing = TimingInfo::with_eval(
                    parse_time,
                    get_timestamp() - eval_start,
                    get_timestamp() - start,
                );
                let output = ReplOutput::runtime_error(e.to_string(), timing);
                return Ok(serde_json::to_string(&output)
                    .unwrap_or_else(|_| "Error serializing output".to_string()));
            }
        };
        let eval_time = get_timestamp() - eval_start;

        // Get captured stdout from OUTPUT_BUFFER
        let stdout = if let Ok(buf) = crate::runtime::builtins::OUTPUT_BUFFER.lock() {
            if buf.is_empty() {
                None
            } else {
                Some(buf.clone())
            }
        } else {
            None
        };

        // Determine what to display: stdout if available, otherwise return value
        let display = if let Some(stdout_output) = stdout {
            stdout_output.trim_end().to_string()
        } else {
            result
        };

        // Add to history
        self.history.push(input.to_string());

        // Return result
        let timing = TimingInfo::with_eval(parse_time, eval_time, get_timestamp() - start);
        let output = ReplOutput::success(display, timing);
        Ok(serde_json::to_string(&output)
            .unwrap_or_else(|_| "Error serializing output".to_string()))
    }

    /// Format a Value for display in REPL output
    pub(crate) fn format_value_for_display(value: &Value) -> String {
        match value {
            Value::String(s) => s.to_string(),
            Value::Nil => "nil".to_string(),
            _ => format!("{value}"),
        }
    }

    /// Get command history
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }

    /// Clear the REPL state
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn clear(&mut self) {
        self.bindings.clear();
        self.history.clear();
    }

    /// Get session ID
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn session_id(&self) -> String {
        self.session_id.clone()
    }
}

impl Default for WasmRepl {
    fn default() -> Self {
        Self::new().expect("WasmRepl::new() should succeed in Default impl")
    }
}
