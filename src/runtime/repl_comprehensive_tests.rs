//! Comprehensive TDD tests for REPL module
//! Target: Increase coverage from 10.73% to 80%
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod repl_comprehensive_tests {
    use crate::runtime::repl::{Repl, Value};
    use std::time::{Duration, Instant};

    // ========== Basic REPL Functionality Tests ==========

    #[test]
    fn test_repl_creation() {
        let repl = Repl::new().unwrap();
        assert!(repl.can_accept_input());
        assert!(repl.bindings_valid());
        assert_eq!(repl.memory_used(), 0);
    }

    #[test]
    fn test_sandboxed_repl() {
        let repl = Repl::sandboxed().unwrap();
        assert!(repl.can_accept_input());
        assert_eq!(repl.memory_used(), 0);
    }

    // ========== Memory Management Tests ==========

    #[test]
    fn test_memory_tracking() {
        let mut repl = Repl::new().unwrap();
        let initial_memory = repl.memory_used();
        
        // Evaluate something that uses memory
        let result = repl.eval_transactional("let x = [1, 2, 3, 4, 5]");
        if result.is_ok() {
            assert!(repl.memory_used() >= initial_memory);
            assert!(repl.peak_memory() >= repl.memory_used());
            assert!(repl.memory_pressure() >= 0.0);
        }
    }

    #[test]
    fn test_memory_bounded_evaluation() {
        let mut repl = Repl::new().unwrap();
        let max_memory = 1024;
        let timeout = Duration::from_millis(100);
        
        let result = repl.eval_bounded("2 + 2", max_memory, timeout);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "4");
    }

    // ========== Basic Evaluation Tests ==========

    #[test]
    fn test_basic_arithmetic() {
        let mut repl = Repl::new().unwrap();
        
        test_eval_success(&mut repl, "2 + 2", "4");
        test_eval_success(&mut repl, "10 - 3", "7");
        test_eval_success(&mut repl, "4 * 5", "20");
    }

    #[test]
    fn test_variable_definitions() {
        let mut repl = Repl::new().unwrap();
        
        test_eval_success(&mut repl, "let x = 10", "10");
        test_eval_success(&mut repl, "x", "10");
        test_eval_success(&mut repl, "let y = x + 5", "15");
        test_eval_success(&mut repl, "y", "15");
    }

    // ========== State Management Tests ==========

    #[test]
    fn test_checkpoint_creation() {
        let mut repl = Repl::new().unwrap();
        
        test_eval_success(&mut repl, "let x = 10", "10");
        let checkpoint = repl.checkpoint();
        
        test_eval_success(&mut repl, "let y = 20", "20");
        test_eval_success(&mut repl, "x + y", "30");
        
        repl.restore_checkpoint(&checkpoint);
        test_eval_success(&mut repl, "x", "10");
        
        // y should no longer be defined
        let result = repl.eval_transactional("y");
        assert!(result.is_err());
    }

    #[test]
    fn test_state_management() {
        let mut repl = Repl::new().unwrap();
        
        test_eval_success(&mut repl, "let x = 42", "42");
        
        let bindings = repl.get_bindings();
        assert!(bindings.contains_key("x"));
        assert_eq!(bindings.get("x").unwrap().to_string(), "42");
        
        // Test history
        assert!(repl.result_history_len() > 0);
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_syntax_error_recovery() {
        let mut repl = Repl::new().unwrap();
        
        let result = repl.eval_transactional("2 +"); // Incomplete expression
        assert!(result.is_err());
        
        // REPL should still be functional
        test_eval_success(&mut repl, "2 + 2", "4");
    }

    #[test]
    fn test_undefined_variable_error() {
        let mut repl = Repl::new().unwrap();
        
        let result = repl.eval_transactional("undefined_var");
        assert!(result.is_err());
        
        // REPL should still be functional
        test_eval_success(&mut repl, "let x = 1", "1");
        test_eval_success(&mut repl, "x", "1");
    }

    // ========== Timeout and Resource Tests ==========

    #[test]
    fn test_evaluate_with_deadline() {
        let mut repl = Repl::new().unwrap();
        let deadline = Some(Instant::now() + Duration::from_millis(100));
        
        let result = repl.evaluate_expr_str("5 + 3", deadline);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "8");
    }

    #[test]
    fn test_transactional_evaluation() {
        let mut repl = Repl::new().unwrap();
        
        let result = repl.eval_transactional("let x = 42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
        
        let result = repl.eval_transactional("x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    // ========== Helper Functions (≤10 complexity each) ==========

    /// Helper: Test successful evaluation with expected result
    fn test_eval_success(repl: &mut Repl, input: &str, expected: &str) {
        let result = repl.eval_transactional(input);
        match &result {
            Ok(val) => {
                assert_eq!(val, expected, "Wrong result for: {} - got {}", input, val);
            },
            Err(e) => {
                panic!("Failed to evaluate: {} - Error: {}", input, e);
            }
        }
    }

    #[test]
    fn test_helper_functions() {
        let mut repl = Repl::new().unwrap();
        
        test_eval_success(&mut repl, "let x = 10", "10");
        test_eval_success(&mut repl, "let y = 20", "20");
        test_eval_success(&mut repl, "x + y", "30");
    }
}