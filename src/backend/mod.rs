//! Backend code generation and transpilation
//!
//! This module handles the conversion of Ruchy AST to Rust code, WebAssembly
//! compilation, and module system management.
//!
//! # Architecture
//!
//! The backend follows a multi-stage compilation pipeline:
//!
//! ```text
//! Ruchy AST → Type Checking → Code Generation → Output
//!     ↓           ↓               ↓            ↓
//!   Frontend   Middleend      Backend      Rust/WASM
//! ```
//!
//! # Components
//!
//! ## Transpiler
//! Converts Ruchy AST nodes to equivalent Rust code:
//! - Expression transpilation with proper precedence
//! - Statement handling and control flow
//! - Pattern matching compilation
//! - Actor system code generation
//!
//! ## Module System
//! Manages Ruchy module loading and dependency resolution:
//! - Module discovery and caching
//! - Import/export resolution
//! - Circular dependency detection
//! - Module compilation ordering
//!
//! ## WebAssembly Support
//! Compiles Ruchy to WebAssembly for browser deployment:
//! - WASM module generation
//! - JavaScript interop
//! - Memory management for WASM
//! - Component model support
//!
//! ## `DataFrame` Integration
//! Optional Apache Arrow integration for data science:
//! - `DataFrame` ↔ Arrow conversion
//! - Columnar data processing
//! - Memory-efficient operations
//!
//! # Examples
//!
//! ```
//! use ruchy::backend::{Transpiler, CompileOptions};
//! use ruchy::frontend::Parser;
//!
//! // Basic transpilation
//! let mut parser = Parser::new("let x = 42");
//! let ast = parser.parse().unwrap();
//! 
//! let mut transpiler = Transpiler::new();
//! let rust_code = transpiler.transpile_to_program(&ast).unwrap();
//! 
//! println!("Generated Rust:\n{}", rust_code);
//! ```
//!
//! ```no_run
//! use ruchy::backend::{compile_to_binary, CompileOptions};
//!
//! // Compile to executable binary
//! let options = CompileOptions::default();
//! let binary_path = compile_to_binary("main.ruchy", &options).unwrap();
//! println!("Binary created: {}", binary_path.display());
//! ```
pub mod compiler;
pub mod module_loader;
pub mod module_resolver;
pub mod transpiler;
pub mod wasm;
#[cfg(feature = "dataframe")]
pub mod arrow_integration;
pub use compiler::{compile_to_binary, compile_source_to_binary, CompileOptions};
pub use module_loader::{ModuleLoader, ParsedModule, ModuleLoaderStats};
pub use module_resolver::ModuleResolver;
pub use transpiler::Transpiler;

// Tests removed: This module only re-exports from submodules.
// Actual implementations and tests belong in the submodules.

