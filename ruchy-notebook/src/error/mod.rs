pub mod types;
pub mod suggestions;
pub mod stack_trace;

pub use types::{NotebookError, ErrorKind, ErrorSpan, ErrorSeverity};
pub use suggestions::SuggestionEngine;
pub use stack_trace::{StackTrace, StackFrame};