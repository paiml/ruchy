//! REPL Evaluation Engine - EXTREME Quality
//!
//! Handles expression and statement evaluation with <10 complexity

use anyhow::{Context, Result};
use crate::frontend::parser::Parser;
use crate::frontend::ast::Expr;
use crate::runtime::interpreter::Interpreter;
use crate::runtime::value::Value;

/// Evaluation context for REPL expressions
pub struct Evaluator {
    interpreter: Interpreter,
    multiline_buffer: Vec<String>,
    in_multiline: bool,
}

impl Evaluator {
    /// Create a new evaluator (complexity: 2)
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            multiline_buffer: Vec::new(),
            in_multiline: false,
        }
    }

    /// Evaluate a single line of input (complexity: 8)
    pub fn evaluate_line(&mut self, line: &str) -> Result<EvalResult> {
        // Check for multiline start
        if self.should_start_multiline(line) {
            self.start_multiline(line);
            return Ok(EvalResult::NeedMoreInput);
        }

        // If in multiline, add to buffer
        if self.in_multiline {
            return self.handle_multiline(line);
        }

        // Single line evaluation
        self.evaluate_complete_input(line)
    }

    /// Check if input should start multiline mode (complexity: 5)
    fn should_start_multiline(&self, line: &str) -> bool {
        let trimmed = line.trim();
        trimmed.ends_with('{') ||
        trimmed.starts_with("fn ") ||
        trimmed.starts_with("if ") ||
        trimmed.starts_with("for ") ||
        trimmed.starts_with("while ")
    }

    /// Start multiline input mode (complexity: 2)
    fn start_multiline(&mut self, line: &str) {
        self.multiline_buffer.clear();
        self.multiline_buffer.push(line.to_string());
        self.in_multiline = true;
    }

    /// Handle multiline input (complexity: 6)
    fn handle_multiline(&mut self, line: &str) -> Result<EvalResult> {
        self.multiline_buffer.push(line.to_string());

        // Check if multiline is complete
        if self.is_multiline_complete() {
            let full_input = self.multiline_buffer.join("\n");
            self.multiline_buffer.clear();
            self.in_multiline = false;
            self.evaluate_complete_input(&full_input)
        } else {
            Ok(EvalResult::NeedMoreInput)
        }
    }

    /// Check if multiline input is complete (complexity: 4)
    fn is_multiline_complete(&self) -> bool {
        let mut brace_count = 0;
        for line in &self.multiline_buffer {
            for ch in line.chars() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => brace_count -= 1,
                    _ => {}
                }
            }
        }
        brace_count == 0
    }

    /// Evaluate complete input (complexity: 7)
    fn evaluate_complete_input(&mut self, input: &str) -> Result<EvalResult> {
        // Parse the input
        let mut parser = Parser::new(input);
        let ast = match parser.parse() {
            Ok(ast) => ast,
            Err(e) => {
                return Ok(EvalResult::Error(format!("Parse error: {}", e)));
            }
        };

        // Evaluate the AST
        match self.interpreter.evaluate(&ast) {
            Ok(value) => Ok(EvalResult::Value(value)),
            Err(e) => Ok(EvalResult::Error(format!("Evaluation error: {}", e))),
        }
    }

    /// Cancel multiline input (complexity: 2)
    pub fn cancel_multiline(&mut self) {
        self.multiline_buffer.clear();
        self.in_multiline = false;
    }

    /// Check if currently in multiline mode (complexity: 1)
    pub fn is_multiline(&self) -> bool {
        self.in_multiline
    }

    /// Get interpreter reference (complexity: 1)
    pub fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }

    /// Get mutable interpreter reference (complexity: 1)
    pub fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }
}

/// Result of evaluation
#[derive(Debug)]
pub enum EvalResult {
    /// Successfully evaluated to a value
    Value(Value),
    /// Need more input (multiline)
    NeedMoreInput,
    /// Evaluation error
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let eval = Evaluator::new();
        assert!(!eval.is_multiline());
    }

    #[test]
    fn test_simple_evaluation() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate_line("2 + 2").unwrap();

        match result {
            EvalResult::Value(Value::Int(4)) => {},
            _ => panic!("Expected Value(Int(4))"),
        }
    }

    #[test]
    fn test_multiline_detection() {
        let eval = Evaluator::new();
        assert!(eval.should_start_multiline("fn test() {"));
        assert!(eval.should_start_multiline("if true {"));
        assert!(eval.should_start_multiline("for x in list {"));
        assert!(!eval.should_start_multiline("2 + 2"));
    }

    #[test]
    fn test_multiline_input() {
        let mut eval = Evaluator::new();

        // Start multiline
        let result = eval.evaluate_line("fn test() {").unwrap();
        assert!(matches!(result, EvalResult::NeedMoreInput));
        assert!(eval.is_multiline());

        // Continue multiline
        let result = eval.evaluate_line("  42").unwrap();
        assert!(matches!(result, EvalResult::NeedMoreInput));

        // Complete multiline
        let result = eval.evaluate_line("}").unwrap();
        assert!(!eval.is_multiline());
    }

    #[test]
    fn test_multiline_cancellation() {
        let mut eval = Evaluator::new();

        eval.evaluate_line("fn test() {").unwrap();
        assert!(eval.is_multiline());

        eval.cancel_multiline();
        assert!(!eval.is_multiline());
    }

    #[test]
    fn test_brace_counting() {
        let mut eval = Evaluator::new();
        eval.multiline_buffer = vec![
            "fn test() {".to_string(),
            "  if true {".to_string(),
            "    42".to_string(),
            "  }".to_string(),
        ];
        assert!(!eval.is_multiline_complete());

        eval.multiline_buffer.push("}".to_string());
        assert!(eval.is_multiline_complete());
    }

    #[test]
    fn test_error_handling() {
        let mut eval = Evaluator::new();
        let result = eval.evaluate_line("invalid syntax @#$").unwrap();

        match result {
            EvalResult::Error(msg) => assert!(msg.contains("error")),
            _ => panic!("Expected error result"),
        }
    }

    // Property tests
    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_evaluator_never_panics(input: String) {
                let mut eval = Evaluator::new();
                let _ = eval.evaluate_line(&input);
            }

            #[test]
            fn test_multiline_state_consistency(lines in prop::collection::vec(".*", 0..10)) {
                let mut eval = Evaluator::new();
                for line in lines {
                    let _ = eval.evaluate_line(&line);
                }
                // State should always be consistent
                assert!(eval.multiline_buffer.is_empty() || eval.is_multiline());
            }
        }
    }
}