//! Backend code generation and transpilation
//!
//! This module handles the conversion of Ruchy AST to Rust code.

pub mod compiler;
pub mod module_loader;
pub mod module_resolver;
pub mod transpiler;
pub mod wasm;

pub use compiler::{compile_to_binary, compile_source_to_binary, CompileOptions};
pub use module_loader::{ModuleLoader, ParsedModule, ModuleLoaderStats};
pub use module_resolver::ModuleResolver;
pub use transpiler::Transpiler;
