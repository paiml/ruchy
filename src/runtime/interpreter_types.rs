//! Interpreter Type Definitions
//!
//! This module contains the core type definitions for the Ruchy interpreter:
//! - `InterpreterError`: All error types that can occur during interpretation
//! - `InterpreterResult`: Execution result enum
//! - `CallFrame`: Function call frame for stack management
//!
//! **EXTREME TDD Round 52**: Extracted from interpreter.rs for modularization.

#![allow(unsafe_code)] // Required for CallFrame Send implementation - see DEFECT-001-B

use super::value::Value;

/// Call frame for function invocation (will be used in Phase 1)
#[derive(Debug)]
#[allow(dead_code)]
pub struct CallFrame {
    /// Function being executed
    pub(crate) closure: Value,

    /// Instruction pointer
    pub(crate) ip: *const u8,

    /// Base of stack frame
    pub(crate) base: usize,

    /// Number of locals in this frame
    pub(crate) locals: usize,
}

// SAFETY: CallFrame can safely be Send because:
// 1. The `ip` raw pointer points to immutable bytecode that never changes
// 2. CallFrame has exclusive ownership of the data (no sharing)
// 3. The pointer is never dereferenced across thread boundaries
// 4. CallFrame is only used in single-threaded execution contexts within each thread
// 5. When Repl is shared across threads, each thread gets its own CallFrame instance
// SAFETY: see above -- ip is immutable bytecode, no cross-thread dereference
unsafe impl Send for CallFrame {}

/// Interpreter execution result
#[derive(Debug)]
pub enum InterpreterResult {
    Continue,
    Jump(usize),
    Return(Value),
    Error(InterpreterError),
}

/// Errors that can occur during interpretation.
///
/// # Examples
///
/// ```
/// use ruchy::runtime::interpreter::InterpreterError;
///
/// let err = InterpreterError::TypeError("Expected integer".to_string());
/// assert_eq!(err.to_string(), "Type error: Expected integer");
/// ```
#[derive(Debug, Clone)]
pub enum InterpreterError {
    /// Type mismatch error
    TypeError(String),
    /// General runtime error
    RuntimeError(String),
    /// Stack overflow (too many nested calls)
    StackOverflow,
    /// Stack underflow (pop from empty stack)
    StackUnderflow,
    /// Invalid bytecode instruction
    InvalidInstruction,
    /// Division by zero
    DivisionByZero,
    /// Array/string index out of bounds
    IndexOutOfBounds,
    /// Break statement with optional label and value
    Break(Option<String>, Value),
    /// Continue statement with optional label
    Continue(Option<String>),
    /// Return statement with value
    Return(Value),
    /// Throw statement (exception handling)
    Throw(Value),
    /// Assertion failed (BUG-037: Test assertions)
    AssertionFailed(String),
    /// Recursion depth limit exceeded (`current_depth`, `max_depth`)
    /// Added via [RUNTIME-001] fix for stack overflow crashes
    RecursionLimitExceeded(usize, usize),
}

// Display implementation is in eval_display.rs

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_error_type_error() {
        let err = InterpreterError::TypeError("expected i64".to_string());
        match err {
            InterpreterError::TypeError(msg) => assert_eq!(msg, "expected i64"),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_interpreter_error_runtime_error() {
        let err = InterpreterError::RuntimeError("undefined variable".to_string());
        match err {
            InterpreterError::RuntimeError(msg) => assert_eq!(msg, "undefined variable"),
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_interpreter_error_stack_overflow() {
        let err = InterpreterError::StackOverflow;
        assert!(matches!(err, InterpreterError::StackOverflow));
    }

    #[test]
    fn test_interpreter_error_stack_underflow() {
        let err = InterpreterError::StackUnderflow;
        assert!(matches!(err, InterpreterError::StackUnderflow));
    }

    #[test]
    fn test_interpreter_error_division_by_zero() {
        let err = InterpreterError::DivisionByZero;
        assert!(matches!(err, InterpreterError::DivisionByZero));
    }

    #[test]
    fn test_interpreter_error_index_out_of_bounds() {
        let err = InterpreterError::IndexOutOfBounds;
        assert!(matches!(err, InterpreterError::IndexOutOfBounds));
    }

    #[test]
    fn test_interpreter_error_break() {
        let err = InterpreterError::Break(Some("outer".to_string()), Value::Integer(42));
        match err {
            InterpreterError::Break(Some(label), Value::Integer(val)) => {
                assert_eq!(label, "outer");
                assert_eq!(val, 42);
            }
            _ => panic!("Expected Break"),
        }
    }

    #[test]
    fn test_interpreter_error_continue() {
        let err = InterpreterError::Continue(None);
        assert!(matches!(err, InterpreterError::Continue(None)));
    }

    #[test]
    fn test_interpreter_error_return() {
        let err = InterpreterError::Return(Value::Bool(true));
        match err {
            InterpreterError::Return(Value::Bool(b)) => assert!(b),
            _ => panic!("Expected Return"),
        }
    }

    #[test]
    fn test_interpreter_error_throw() {
        let err = InterpreterError::Throw(Value::from_string("error message".to_string()));
        assert!(matches!(err, InterpreterError::Throw(_)));
    }

    #[test]
    fn test_interpreter_error_assertion_failed() {
        let err = InterpreterError::AssertionFailed("expected true".to_string());
        match err {
            InterpreterError::AssertionFailed(msg) => assert_eq!(msg, "expected true"),
            _ => panic!("Expected AssertionFailed"),
        }
    }

    #[test]
    fn test_interpreter_error_recursion_limit() {
        let err = InterpreterError::RecursionLimitExceeded(101, 100);
        match err {
            InterpreterError::RecursionLimitExceeded(current, max) => {
                assert_eq!(current, 101);
                assert_eq!(max, 100);
            }
            _ => panic!("Expected RecursionLimitExceeded"),
        }
    }

    #[test]
    fn test_interpreter_error_clone() {
        let err = InterpreterError::TypeError("test".to_string());
        let cloned = err.clone();
        assert!(matches!(cloned, InterpreterError::TypeError(_)));
    }

    #[test]
    fn test_interpreter_result_continue() {
        let result = InterpreterResult::Continue;
        assert!(matches!(result, InterpreterResult::Continue));
    }

    #[test]
    fn test_interpreter_result_jump() {
        let result = InterpreterResult::Jump(42);
        match result {
            InterpreterResult::Jump(addr) => assert_eq!(addr, 42),
            _ => panic!("Expected Jump"),
        }
    }

    #[test]
    fn test_interpreter_result_return() {
        let result = InterpreterResult::Return(Value::Integer(100));
        match result {
            InterpreterResult::Return(Value::Integer(val)) => assert_eq!(val, 100),
            _ => panic!("Expected Return"),
        }
    }

    #[test]
    fn test_interpreter_result_error() {
        let result = InterpreterResult::Error(InterpreterError::DivisionByZero);
        assert!(matches!(
            result,
            InterpreterResult::Error(InterpreterError::DivisionByZero)
        ));
    }

    #[test]
    fn test_call_frame_creation() {
        let frame = CallFrame {
            closure: Value::Nil,
            ip: std::ptr::null(),
            base: 0,
            locals: 5,
        };
        assert_eq!(frame.base, 0);
        assert_eq!(frame.locals, 5);
    }

    #[test]
    fn test_call_frame_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<CallFrame>();
    }
}
