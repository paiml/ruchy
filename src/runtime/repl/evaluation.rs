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
    pub fn evaluate_line(&mut self, line: &str, state: &mut crate::runtime::repl::state::ReplState) -> Result<EvalResult> {
        // Handle multiline continuation
        if !self.multiline_buffer.is_empty() {
            self.multiline_buffer.push('\n');
            self.multiline_buffer.push_str(line);
        } else {
            self.multiline_buffer = line.to_string();
        }

        // Try to parse and evaluate the accumulated input
        use crate::frontend::parser::Parser;

        let mut parser = Parser::new(&self.multiline_buffer);
        match parser.parse_expr() {
            Ok(expr) => {
                match self.interpreter.eval_expr(&expr) {
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

    /// Set a variable in the interpreter (complexity: 1)
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.interpreter.set_global_binding(name, value);
    }

    /// Check if error indicates incomplete input (complexity: 3)
    fn is_incomplete_error(&self, error_msg: &str) -> bool {
        error_msg.contains("unexpected end of input") ||
        error_msg.contains("expected") ||
        error_msg.contains("incomplete")
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
            EvalResult::Value(Value::Integer(4)) => {},
            result => panic!("Expected Integer(4), got {:?}", result),
        }
    }

    #[test]
    fn test_error_handling() {
        let mut evaluator = Evaluator::new();
        let mut state = crate::runtime::repl::state::ReplState::new();

        match evaluator.evaluate_line("undefined_variable", &mut state).unwrap() {
            EvalResult::Error(_) => {},
            result => panic!("Expected Error, got {:?}", result),
        }
    }
}