//! WASM Backend Module
//!
//! Provides WASM code generation from Ruchy AST.
//!
//! ## Module Structure
//! - `emitter` - Core WASM emitter implementation
//! - `wasm_module` - Compiled WASM module representation
//! - `symbol_table` - Variable tracking across scopes
//! - `types` - WASM type definitions
//! - `utils` - Pure utility functions for AST analysis

// Submodules
#[cfg(test)]
mod debug;
pub mod emitter;
#[cfg(test)]
mod emitter_tests;
pub mod symbol_table;
pub mod types;
pub mod utils;
pub mod wasm_module;

// Re-exports for convenient access
pub use emitter::WasmEmitter;
pub use symbol_table::SymbolTable;
pub use types::WasmType;
pub use wasm_module::WasmModule;
