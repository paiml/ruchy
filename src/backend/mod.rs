//! Backend code generation and transpilation
//!
//! This module handles the conversion of Ruchy AST to Rust code.

pub mod compiler;
pub mod transpiler;

pub use compiler::{compile_to_binary, compile_source_to_binary, CompileOptions};
pub use transpiler::Transpiler;
