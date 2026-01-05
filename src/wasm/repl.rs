//! WebAssembly REPL implementation for browser-based evaluation
//!
//! Provides interactive Ruchy evaluation in the browser with progressive enhancement.

use crate::runtime::{Interpreter, Value};
use crate::wasm::helpers::{generate_session_id, get_timestamp, JsValue};
use crate::wasm::output::{ReplOutput, TimingInfo};
use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WebAssembly REPL for browser-based Ruchy evaluation
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct WasmRepl {
    bindings: HashMap<String, String>,
    history: Vec<String>,
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
    fn format_value_for_display(value: &Value) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_repl_creation() {
        let repl = WasmRepl::new();
        assert!(repl.is_ok());
    }

    #[test]
    fn test_wasm_repl_default() {
        let repl = WasmRepl::default();
        assert!(repl.session_id().starts_with("session-"));
    }

    #[test]
    fn test_session_id() {
        let repl = WasmRepl::new().expect("repl");
        assert!(repl.session_id().starts_with("session-"));
    }

    #[test]
    fn test_session_id_unique() {
        let repl1 = WasmRepl::new().expect("repl1");
        std::thread::sleep(std::time::Duration::from_millis(1));
        let repl2 = WasmRepl::new().expect("repl2");
        assert_ne!(repl1.session_id(), repl2.session_id());
    }

    #[test]
    fn test_eval_simple_integer() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("42").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("42".to_string()));
    }

    #[test]
    fn test_eval_simple_float() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("3.14").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
    }

    #[test]
    fn test_eval_simple_bool() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("true").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("true".to_string()));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("1 + 2 * 3").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("7".to_string()));
    }

    #[test]
    fn test_eval_parse_error() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("let = invalid").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(!output.success);
        assert!(output.error.is_some());
        assert!(output.error.unwrap().contains("Parse error"));
    }

    #[test]
    fn test_eval_adds_to_history() {
        let mut repl = WasmRepl::new().expect("repl");
        let _ = repl.eval("1");
        let _ = repl.eval("2");
        let _ = repl.eval("3");
        let history = repl.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], "1");
        assert_eq!(history[1], "2");
        assert_eq!(history[2], "3");
    }

    #[test]
    fn test_clear_history() {
        let mut repl = WasmRepl::new().expect("repl");
        let _ = repl.eval("1");
        let _ = repl.eval("2");
        assert_eq!(repl.get_history().len(), 2);
        repl.clear();
        assert!(repl.get_history().is_empty());
    }

    #[test]
    fn test_clear_bindings() {
        let mut repl = WasmRepl::new().expect("repl");
        repl.bindings.insert("x".to_string(), "10".to_string());
        assert!(!repl.bindings.is_empty());
        repl.clear();
        assert!(repl.bindings.is_empty());
    }

    #[test]
    fn test_format_value_string() {
        use std::sync::Arc;
        let value = Value::String(Arc::from("hello"));
        let result = WasmRepl::format_value_for_display(&value);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_format_value_nil() {
        let value = Value::Nil;
        let result = WasmRepl::format_value_for_display(&value);
        assert_eq!(result, "nil");
    }

    #[test]
    fn test_format_value_integer() {
        let value = Value::Integer(42);
        let result = WasmRepl::format_value_for_display(&value);
        assert_eq!(result, "42");
    }

    #[test]
    fn test_format_value_bool() {
        let value = Value::Bool(true);
        let result = WasmRepl::format_value_for_display(&value);
        assert_eq!(result, "true");
    }

    #[test]
    fn test_format_value_float() {
        let value = Value::Float(3.14);
        let result = WasmRepl::format_value_for_display(&value);
        assert!(result.contains("3.14"));
    }

    #[test]
    fn test_eval_complex_expression() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("(1 + 2) * (3 + 4)").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("21".to_string()));
    }

    #[test]
    fn test_eval_string_literal() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval(r#""hello world""#).expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("hello world".to_string()));
    }

    #[test]
    fn test_eval_array_literal() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("[1, 2, 3]").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
    }

    #[test]
    fn test_timing_info_present() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("42").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.timing.total_ms >= 0.0);
        assert!(output.timing.parse_ms >= 0.0);
        assert!(output.timing.eval_ms >= 0.0);
    }

    #[test]
    fn test_eval_boolean_false() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("false").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("false".to_string()));
    }

    #[test]
    fn test_eval_comparison() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("5 > 3").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("true".to_string()));
    }

    #[test]
    fn test_eval_logical_and() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("true && false").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("false".to_string()));
    }

    #[test]
    fn test_eval_logical_or() {
        let mut repl = WasmRepl::new().expect("repl");
        let result = repl.eval("true || false").expect("eval");
        let output: ReplOutput = serde_json::from_str(&result).expect("parse");
        assert!(output.success);
        assert_eq!(output.display, Some("true".to_string()));
    }

    #[test]
    fn test_history_accumulation() {
        let mut repl = WasmRepl::new().expect("repl");
        for i in 1..=10 {
            let _ = repl.eval(&format!("{i}"));
        }
        let history = repl.get_history();
        assert_eq!(history.len(), 10);
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(50))]

            #[test]
            fn prop_wasm_repl_new_never_panics(_dummy: u8) {
                let repl = WasmRepl::new();
                prop_assert!(repl.is_ok());
            }

            #[test]
            fn prop_eval_simple_never_panics(x in -1000i64..1000) {
                let mut repl = WasmRepl::new().unwrap();
                let code = format!("{x}");
                let _ = repl.eval(&code);
            }

            #[test]
            fn prop_eval_arithmetic_never_panics(
                a in -100i64..100,
                b in 1i64..100
            ) {
                let mut repl = WasmRepl::new().unwrap();
                let _ = repl.eval(&format!("{a} + {b}"));
                let _ = repl.eval(&format!("{a} - {b}"));
                let _ = repl.eval(&format!("{a} * {b}"));
                let _ = repl.eval(&format!("{a} / {b}"));
            }

            #[test]
            fn prop_get_history_valid(
                code1 in "[a-z0-9 +\\-*/]{1,20}",
                code2 in "[a-z0-9 +\\-*/]{1,20}"
            ) {
                let mut repl = WasmRepl::new().unwrap();
                let _ = repl.eval(&code1);
                let _ = repl.eval(&code2);
                let history = repl.get_history();
                prop_assert!(history.len() <= 2);
            }
        }
    }
}
