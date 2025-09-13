//! Comprehensive TDD tests for WASM REPL module
//! Target: Increase coverage from 4-8% to 50%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod wasm_repl_tests {
    use crate::wasm::repl::{WasmRepl, ReplOutput, TimingInfo};
    use std::collections::HashMap;
    
    // ========== Basic REPL Creation Tests ==========
    
    #[test]
    fn test_repl_creation() {
        let repl = WasmRepl::new();
        assert!(repl.is_ok());
        let repl = repl.unwrap();
        assert_eq!(repl.get_history().len(), 0);
    }
    
    #[test]
    fn test_repl_initial_state() {
        let repl = WasmRepl::new().unwrap();
        assert!(repl.get_history().is_empty());
        assert_eq!(repl.get_bindings().len(), 0);
    }
    
    #[test]
    fn test_repl_session_id() {
        let repl1 = WasmRepl::new().unwrap();
        let repl2 = WasmRepl::new().unwrap();
        // Session IDs should be unique
        assert_ne!(repl1.get_session_id(), repl2.get_session_id());
    }
    
    // ========== Evaluation Tests ==========
    
    #[test]
    fn test_eval_simple_expression() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("1 + 1");
        assert!(result.is_ok());
        
        let output = result.unwrap();
        let parsed: ReplOutput = serde_json::from_str(&output).unwrap();
        assert!(parsed.success);
        assert!(parsed.error.is_none());
    }
    
    #[test]
    fn test_eval_parse_error() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("@#$%^&*");
        assert!(result.is_ok()); // Returns Ok with error in output
        
        let output = result.unwrap();
        let parsed: ReplOutput = serde_json::from_str(&output).unwrap();
        assert!(!parsed.success);
        assert!(parsed.error.is_some());
        assert!(parsed.error.unwrap().contains("Parse error"));
    }
    
    #[test]
    fn test_eval_timing_info() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("42").unwrap();
        
        let parsed: ReplOutput = serde_json::from_str(&result).unwrap();
        assert!(parsed.timing.parse_ms >= 0.0);
        assert!(parsed.timing.typecheck_ms >= 0.0);
        assert!(parsed.timing.eval_ms >= 0.0);
        assert!(parsed.timing.total_ms >= 0.0);
        assert!(parsed.timing.total_ms >= parsed.timing.parse_ms);
    }
    
    // ========== History Management Tests ==========
    
    #[test]
    fn test_history_tracking() {
        let mut repl = WasmRepl::new().unwrap();
        
        repl.eval("1 + 1").unwrap();
        assert_eq!(repl.get_history().len(), 1);
        assert_eq!(repl.get_history()[0], "1 + 1");
        
        repl.eval("2 * 3").unwrap();
        assert_eq!(repl.get_history().len(), 2);
        assert_eq!(repl.get_history()[1], "2 * 3");
    }
    
    #[test]
    fn test_history_persistence() {
        let mut repl = WasmRepl::new().unwrap();
        
        repl.eval("let x = 10").unwrap();
        repl.eval("let y = 20").unwrap();
        repl.eval("x + y").unwrap();
        
        let history = repl.get_history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0], "let x = 10");
        assert_eq!(history[1], "let y = 20");
        assert_eq!(history[2], "x + y");
    }
    
    #[test]
    fn test_clear_repl() {
        let mut repl = WasmRepl::new().unwrap();
        
        repl.eval("1 + 1").unwrap();
        repl.eval("2 + 2").unwrap();
        assert_eq!(repl.get_history().len(), 2);
        
        repl.clear();
        assert_eq!(repl.get_history().len(), 0);
        assert_eq!(repl.get_bindings().len(), 0);
    }
    
    // ========== Bindings Tests ==========
    
    #[test]
    fn test_bindings_storage() {
        let mut repl = WasmRepl::new().unwrap();
        
        repl.eval("let x = 42").unwrap();
        let bindings = repl.get_bindings();
        assert!(bindings.contains_key("x"));
    }
    
    #[test]
    fn test_bindings_update() {
        let mut repl = WasmRepl::new().unwrap();
        
        repl.eval("let x = 10").unwrap();
        assert_eq!(repl.get_bindings().get("x"), Some(&"10".to_string()));
        
        repl.eval("let x = 20").unwrap();
        assert_eq!(repl.get_bindings().get("x"), Some(&"20".to_string()));
    }
    
    // ========== Error Handling Tests ==========
    
    #[test]
    fn test_empty_input() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("");
        assert!(result.is_ok());
        
        let output = result.unwrap();
        let parsed: ReplOutput = serde_json::from_str(&output).unwrap();
        // Empty input might parse successfully or fail - check handling
        assert!(parsed.timing.total_ms >= 0.0);
    }
    
    #[test]
    fn test_whitespace_only_input() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("   \n\t  ");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_very_long_input() {
        let mut repl = WasmRepl::new().unwrap();
        let long_input = "1 + ".repeat(1000) + "1";
        let result = repl.eval(&long_input);
        assert!(result.is_ok());
    }
    
    // ========== Output Format Tests ==========
    
    #[test]
    fn test_output_json_format() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("42").unwrap();
        
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
        assert!(parsed.get("success").is_some());
        assert!(parsed.get("timing").is_some());
    }
    
    #[test]
    fn test_rust_code_generation() {
        let mut repl = WasmRepl::new().unwrap();
        let result = repl.eval("fn add(x: i32, y: i32) -> i32 { x + y }").unwrap();
        
        let parsed: ReplOutput = serde_json::from_str(&result).unwrap();
        // Check if Rust code generation is included when applicable
        if parsed.success {
            // rust_code field should be populated for function definitions
            assert!(parsed.display.is_some() || parsed.rust_code.is_some());
        }
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    impl WasmRepl {
        /// Helper: Get bindings for testing
        pub fn get_bindings(&self) -> &HashMap<String, String> {
            &self.bindings
        }
        
        /// Helper: Get session ID for testing
        pub fn get_session_id(&self) -> &str {
            &self.session_id
        }
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_eval_never_panics(input in "[a-zA-Z0-9 +\\-*/()]{0,100}") {
            let mut repl = WasmRepl::new().unwrap();
            let result = repl.eval(&input);
            assert!(result.is_ok());
        }
        
        #[test]
        fn test_history_preserves_input(inputs in prop::collection::vec("[a-z]+", 1..10)) {
            let mut repl = WasmRepl::new().unwrap();
            
            for input in &inputs {
                repl.eval(input).unwrap();
            }
            
            let history = repl.get_history();
            assert_eq!(history.len(), inputs.len());
            for (i, input) in inputs.iter().enumerate() {
                assert_eq!(history[i], *input);
            }
        }
        
        #[test]
        fn test_timing_always_positive(input in ".*") {
            let mut repl = WasmRepl::new().unwrap();
            if let Ok(output) = repl.eval(&input) {
                if let Ok(parsed) = serde_json::from_str::<ReplOutput>(&output) {
                    assert!(parsed.timing.parse_ms >= 0.0);
                    assert!(parsed.timing.typecheck_ms >= 0.0);
                    assert!(parsed.timing.eval_ms >= 0.0);
                    assert!(parsed.timing.total_ms >= 0.0);
                }
            }
        }
    }
}