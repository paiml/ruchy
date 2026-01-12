//! REPL Expression Evaluation Engine
//!
//! Handles evaluation of user input with proper error handling and multiline support.

use crate::runtime::interpreter::{Interpreter, Value};
use anyhow::Result;

/// Result of evaluating a line of input
#[derive(Debug, Clone)]
pub enum EvalResult {
    /// Successfully evaluated to a value
    Value(Value),
    /// Input incomplete, needs more input
    NeedMoreInput,
    /// Evaluation error with message
    Error(String),
}

/// Expression evaluator for the REPL
#[derive(Debug)]
pub struct Evaluator {
    /// Internal interpreter instance
    interpreter: Interpreter,
    /// Tracks if we're in multiline mode
    multiline_buffer: String,
}

impl Evaluator {
    /// Create a new evaluator (complexity: 2)
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            multiline_buffer: String::new(),
        }
    }

    /// Evaluate a line of input with state synchronization (complexity: 9)
    pub fn evaluate_line(
        &mut self,
        line: &str,
        state: &mut crate::runtime::repl::state::ReplState,
    ) -> Result<EvalResult> {
        // Handle multiline continuation
        if self.multiline_buffer.is_empty() {
            self.multiline_buffer = line.to_string();
        } else {
            self.multiline_buffer.push('\n');
            self.multiline_buffer.push_str(line);
        }

        // Try to parse and evaluate the accumulated input
        use crate::frontend::parser::Parser;

        let mut parser = Parser::new(&self.multiline_buffer);
        match parser.parse() {
            Ok(expr) => {
                // [RUNTIME-083] Catch InterpreterError::Return and extract value (early return support)
                // This matches the pattern in interpreter.rs:2044-2046 for function calls
                let result = match self.interpreter.eval_expr(&expr) {
                    Err(crate::runtime::InterpreterError::Return(val)) => Ok(val),
                    other => other,
                };

                match result {
                    Ok(value) => {
                        self.multiline_buffer.clear();

                        // Synchronize interpreter bindings with REPL state
                        let interpreter_bindings = self.interpreter.get_current_bindings();
                        let state_bindings = state.get_bindings_mut();
                        for (name, val) in interpreter_bindings {
                            state_bindings.insert(name, val);
                        }

                        Ok(EvalResult::Value(value))
                    }
                    Err(e) => {
                        self.multiline_buffer.clear();
                        Ok(EvalResult::Error(e.to_string()))
                    }
                }
            }
            Err(e) => {
                // Check if this looks like an incomplete expression
                let error_str = e.to_string();
                if self.is_incomplete_error(&error_str) {
                    Ok(EvalResult::NeedMoreInput)
                } else {
                    self.multiline_buffer.clear();
                    Ok(EvalResult::Error(error_str))
                }
            }
        }
    }

    /// Check if we're in multiline mode (complexity: 1)
    pub fn is_multiline(&self) -> bool {
        !self.multiline_buffer.is_empty()
    }

    /// Clear interpreter variables for checkpoint restore (complexity: 2)
    pub fn clear_interpreter_variables(&mut self) {
        self.interpreter.clear_user_variables();
    }

    /// Set a variable in the interpreter (complexity: 1)
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.interpreter.set_global_binding(name, value);
    }

    /// Check if error indicates incomplete input (complexity: 3)
    fn is_incomplete_error(&self, error_msg: &str) -> bool {
        error_msg.contains("unexpected end of input")
            || error_msg.contains("incomplete")
            || (error_msg.contains("Expected") && error_msg.contains("EOF"))
            || (error_msg.contains("expected") && error_msg.contains("EOF"))
            || error_msg.contains("found EOF")
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::new();
        assert!(!evaluator.is_multiline());
    }

    #[test]
    fn test_simple_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("2 + 2", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(4)) => {}
            result => panic!("Expected Integer(4), got {result:?}"),
        }
    }

    #[test]
    fn test_error_handling() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator
            .evaluate_line("undefined_variable", &mut state)
            .unwrap()
        {
            EvalResult::Error(_) => {}
            result => panic!("Expected Error, got {result:?}"),
        }
    }

    // === EXTREME TDD Round 15 tests ===

    #[test]
    fn test_evaluator_default() {
        let evaluator = Evaluator::default();
        assert!(!evaluator.is_multiline());
    }

    #[test]
    fn test_eval_result_debug() {
        let value_result = EvalResult::Value(Value::Integer(42));
        let debug_str = format!("{:?}", value_result);
        assert!(debug_str.contains("Value"));

        let need_more = EvalResult::NeedMoreInput;
        let debug_str = format!("{:?}", need_more);
        assert!(debug_str.contains("NeedMoreInput"));

        let error_result = EvalResult::Error("test error".to_string());
        let debug_str = format!("{:?}", error_result);
        assert!(debug_str.contains("Error"));
    }

    #[test]
    fn test_eval_result_clone() {
        let original = EvalResult::Value(Value::Integer(100));
        let cloned = original.clone();
        match cloned {
            EvalResult::Value(Value::Integer(100)) => {}
            _ => panic!("Clone should preserve value"),
        }
    }

    #[test]
    fn test_is_incomplete_error() {
        let evaluator = Evaluator::new();

        // These should be detected as incomplete
        assert!(evaluator.is_incomplete_error("unexpected end of input"));
        assert!(evaluator.is_incomplete_error("incomplete expression"));
        assert!(evaluator.is_incomplete_error("Expected } but found EOF"));
        assert!(evaluator.is_incomplete_error("expected ) at EOF"));
        assert!(evaluator.is_incomplete_error("found EOF unexpectedly"));

        // These should NOT be incomplete
        assert!(!evaluator.is_incomplete_error("undefined variable"));
        assert!(!evaluator.is_incomplete_error("type mismatch"));
    }

    #[test]
    fn test_string_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("\"hello\"", &mut state).unwrap() {
            EvalResult::Value(Value::String(s)) => {
                assert_eq!(&*s, "hello");
            }
            result => panic!("Expected String, got {result:?}"),
        }
    }

    #[test]
    fn test_let_binding_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        // Define a variable
        let result = evaluator.evaluate_line("let x = 10", &mut state).unwrap();
        match result {
            EvalResult::Value(_) => {}
            result => panic!("Expected Value for let binding, got {result:?}"),
        }
    }

    // === Additional tests for improved coverage ===

    #[test]
    fn test_evaluator_debug_trait() {
        let evaluator = Evaluator::new();
        let debug_str = format!("{:?}", evaluator);
        assert!(debug_str.contains("Evaluator"));
    }

    #[test]
    fn test_evaluator_is_multiline_false_initially() {
        let evaluator = Evaluator::new();
        assert!(!evaluator.is_multiline());
    }

    #[test]
    fn test_float_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("3.14", &mut state).unwrap() {
            EvalResult::Value(Value::Float(f)) => {
                assert!((f - 3.14).abs() < 0.0001);
            }
            result => panic!("Expected Float, got {result:?}"),
        }
    }

    #[test]
    fn test_boolean_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("true", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true), got {result:?}"),
        }

        match evaluator.evaluate_line("false", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(false)) => {}
            result => panic!("Expected Bool(false), got {result:?}"),
        }
    }

    #[test]
    fn test_nil_evaluation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("nil", &mut state).unwrap() {
            EvalResult::Value(Value::Nil) => {}
            result => panic!("Expected Nil, got {result:?}"),
        }
    }

    #[test]
    fn test_arithmetic_expressions() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        // Addition
        match evaluator.evaluate_line("10 + 5", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(15)) => {}
            result => panic!("Expected Integer(15), got {result:?}"),
        }

        // Subtraction
        match evaluator.evaluate_line("10 - 3", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(7)) => {}
            result => panic!("Expected Integer(7), got {result:?}"),
        }

        // Multiplication
        match evaluator.evaluate_line("4 * 5", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(20)) => {}
            result => panic!("Expected Integer(20), got {result:?}"),
        }

        // Division
        match evaluator.evaluate_line("20 / 4", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(5)) => {}
            result => panic!("Expected Integer(5), got {result:?}"),
        }
    }

    #[test]
    fn test_comparison_expressions() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("5 > 3", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true), got {result:?}"),
        }

        match evaluator.evaluate_line("3 < 5", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true), got {result:?}"),
        }

        match evaluator.evaluate_line("5 == 5", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true), got {result:?}"),
        }

        match evaluator.evaluate_line("5 != 3", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true), got {result:?}"),
        }
    }

    #[test]
    fn test_parenthesized_expressions() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("(2 + 3) * 4", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(20)) => {}
            result => panic!("Expected Integer(20), got {result:?}"),
        }
    }

    #[test]
    fn test_negative_numbers() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("-5", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(-5)) => {}
            result => panic!("Expected Integer(-5), got {result:?}"),
        }
    }

    #[test]
    fn test_set_variable() {
        let mut evaluator = Evaluator::new();
        evaluator.set_variable("test_var".to_string(), Value::Integer(42));
        // Verify the variable was set by evaluating it
        let mut state = crate::runtime::repl::state::ReplState::new();
        match evaluator.evaluate_line("test_var", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(42)) => {}
            result => panic!("Expected Integer(42), got {result:?}"),
        }
    }

    #[test]
    fn test_clear_interpreter_variables() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        // Define a variable
        evaluator
            .evaluate_line("let my_var = 100", &mut state)
            .unwrap();

        // Clear variables
        evaluator.clear_interpreter_variables();

        // Variable should no longer exist
        match evaluator.evaluate_line("my_var", &mut state).unwrap() {
            EvalResult::Error(_) => {} // Expected: variable undefined
            result => panic!("Expected Error (undefined variable), got {result:?}"),
        }
    }

    #[test]
    fn test_is_incomplete_error_patterns() {
        let evaluator = Evaluator::new();

        // Should detect as incomplete
        assert!(evaluator.is_incomplete_error("unexpected end of input"));
        assert!(evaluator.is_incomplete_error("expression is incomplete"));
        assert!(evaluator.is_incomplete_error("Expected } but found EOF"));
        assert!(evaluator.is_incomplete_error("expected ) at EOF"));
        assert!(evaluator.is_incomplete_error("found EOF while parsing"));

        // Should NOT detect as incomplete
        assert!(!evaluator.is_incomplete_error("syntax error"));
        assert!(!evaluator.is_incomplete_error("invalid token"));
        assert!(!evaluator.is_incomplete_error("type error: cannot add"));
    }

    #[test]
    fn test_logical_expressions() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("true && true", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true) for AND, got {result:?}"),
        }

        match evaluator
            .evaluate_line("true || false", &mut state)
            .unwrap()
        {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true) for OR, got {result:?}"),
        }

        match evaluator.evaluate_line("!false", &mut state).unwrap() {
            EvalResult::Value(Value::Bool(true)) => {}
            result => panic!("Expected Bool(true) for NOT, got {result:?}"),
        }
    }

    #[test]
    fn test_if_expression() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator
            .evaluate_line("if true { 1 } else { 2 }", &mut state)
            .unwrap()
        {
            EvalResult::Value(Value::Integer(1)) => {}
            result => panic!("Expected Integer(1), got {result:?}"),
        }

        match evaluator
            .evaluate_line("if false { 1 } else { 2 }", &mut state)
            .unwrap()
        {
            EvalResult::Value(Value::Integer(2)) => {}
            result => panic!("Expected Integer(2), got {result:?}"),
        }
    }

    #[test]
    fn test_state_synchronization() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        // Define a variable via evaluator
        evaluator
            .evaluate_line("let sync_var = 999", &mut state)
            .unwrap();

        // State should have the binding now
        assert!(state.get_bindings().contains_key("sync_var"));
    }

    #[test]
    fn test_string_concatenation() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator
            .evaluate_line("\"hello\" + \" \" + \"world\"", &mut state)
            .unwrap()
        {
            EvalResult::Value(Value::String(s)) => {
                assert_eq!(&*s, "hello world");
            }
            result => panic!("Expected String, got {result:?}"),
        }
    }

    #[test]
    fn test_modulo_operator() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("10 % 3", &mut state).unwrap() {
            EvalResult::Value(Value::Integer(1)) => {}
            result => panic!("Expected Integer(1), got {result:?}"),
        }
    }

    #[test]
    fn test_eval_result_clone_variants() {
        // Test clone for all EvalResult variants
        let value = EvalResult::Value(Value::String("test".into()));
        let cloned = value.clone();
        match cloned {
            EvalResult::Value(Value::String(s)) => assert_eq!(&*s, "test"),
            _ => panic!("Clone failed for Value variant"),
        }

        let need_more = EvalResult::NeedMoreInput;
        let cloned = need_more.clone();
        assert!(matches!(cloned, EvalResult::NeedMoreInput));

        let error = EvalResult::Error("error msg".to_string());
        let cloned = error.clone();
        match cloned {
            EvalResult::Error(msg) => assert_eq!(msg, "error msg"),
            _ => panic!("Clone failed for Error variant"),
        }
    }
}
