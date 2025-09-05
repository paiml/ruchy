//! Interpreter error types and results
//! Extracted from interpreter.rs for modularity (complexity: ≤10 per function)

use std::fmt;

/// Result type for interpreter operations
pub type InterpreterResult<T> = Result<T, InterpreterError>;

/// Error types for the interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    /// Variable not found in environment
    UndefinedVariable(String),
    /// Type mismatch in operation
    TypeMismatch { expected: String, found: String },
    /// Division by zero
    DivisionByZero,
    /// Index out of bounds
    IndexOutOfBounds { index: i64, length: usize },
    /// Function call with wrong number of arguments
    ArgumentCountMismatch { expected: usize, found: usize },
    /// Stack overflow
    StackOverflow,
    /// Generic runtime error
    RuntimeError(String),
    /// Pattern match failure
    PatternMatchFailure,
    /// Module not found
    ModuleNotFound(String),
    /// Import error
    ImportError(String),
    /// Invalid operation
    InvalidOperation(String),
}

impl InterpreterError {
    /// Create an undefined variable error
    pub fn undefined_variable(name: impl Into<String>) -> Self {
        InterpreterError::UndefinedVariable(name.into())
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: impl Into<String>, found: impl Into<String>) -> Self {
        InterpreterError::TypeMismatch {
            expected: expected.into(),
            found: found.into(),
        }
    }

    /// Create an index out of bounds error
    pub fn index_out_of_bounds(index: i64, length: usize) -> Self {
        InterpreterError::IndexOutOfBounds { index, length }
    }

    /// Create an argument count mismatch error
    pub fn argument_count_mismatch(expected: usize, found: usize) -> Self {
        InterpreterError::ArgumentCountMismatch { expected, found }
    }

    /// Create a runtime error with a message
    pub fn runtime(message: impl Into<String>) -> Self {
        InterpreterError::RuntimeError(message.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(message: impl Into<String>) -> Self {
        InterpreterError::InvalidOperation(message.into())
    }

    /// Create a module not found error
    pub fn module_not_found(name: impl Into<String>) -> Self {
        InterpreterError::ModuleNotFound(name.into())
    }

    /// Create an import error
    pub fn import_error(message: impl Into<String>) -> Self {
        InterpreterError::ImportError(message.into())
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            InterpreterError::UndefinedVariable(name) => {
                format!("Undefined variable: '{}'", name)
            }
            InterpreterError::TypeMismatch { expected, found } => {
                format!("Type mismatch: expected {}, found {}", expected, found)
            }
            InterpreterError::DivisionByZero => "Division by zero".to_string(),
            InterpreterError::IndexOutOfBounds { index, length } => {
                format!("Index {} out of bounds for length {}", index, length)
            }
            InterpreterError::ArgumentCountMismatch { expected, found } => {
                format!("Wrong number of arguments: expected {}, found {}", expected, found)
            }
            InterpreterError::StackOverflow => "Stack overflow".to_string(),
            InterpreterError::RuntimeError(msg) => msg.clone(),
            InterpreterError::PatternMatchFailure => "Pattern match failure".to_string(),
            InterpreterError::ModuleNotFound(name) => {
                format!("Module not found: '{}'", name)
            }
            InterpreterError::ImportError(msg) => format!("Import error: {}", msg),
            InterpreterError::InvalidOperation(msg) => format!("Invalid operation: {}", msg),
        }
    }
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for InterpreterError {}