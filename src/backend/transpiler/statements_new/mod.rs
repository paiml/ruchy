//! Statement and control flow transpilation - modularized
//!
//! This module is organized into focused submodules:
//! - control_flow: if, while, for, loop statements
//! - bindings: let, pattern matching, destructuring
//! - functions: function definitions, lambdas, calls
//! - blocks: blocks, pipelines, comprehensions
//! - modules: import, export, module definitions

pub mod control_flow;
pub mod bindings;
pub mod functions;
pub mod blocks;
pub mod modules;

// Re-export the main transpiler implementation
pub use control_flow::*;
pub use bindings::*;
pub use functions::*;
pub use blocks::*;
pub use modules::*;