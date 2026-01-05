//! WebAssembly REPL Module
//!
//! Provides interactive Ruchy evaluation in the browser with progressive enhancement.

// Submodules
#[cfg(test)]
mod tests;
mod wasm_repl;

// Re-exports
pub use wasm_repl::WasmRepl;
