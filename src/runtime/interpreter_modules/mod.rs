//! Modularized interpreter implementation
//! Refactored from monolithic 5,130-line interpreter.rs

pub mod value;
pub mod cache;
pub mod type_feedback;
pub mod gc;
pub mod threaded;
pub mod evaluator;
pub mod builtin;
pub mod error;

// Re-export commonly used types
pub use value::Value;
pub use cache::{InlineCache, CacheEntry, CacheState};
pub use type_feedback::{TypeFeedback, OperationFeedback, CallSiteFeedback};
pub use gc::{ConservativeGC, GCObject, GCStats};
pub use threaded::{DirectThreadedInterpreter, Instruction, Operand};
pub use evaluator::Evaluator;
pub use builtin::BuiltinFunction;
pub use error::{InterpreterError, InterpreterResult};