//! WebAssembly component toolkit for Ruchy (RUCHY-0819)
//!
//! This module provides WebAssembly component generation, WIT interface generation,
//! platform-specific deployment, and portability scoring for Ruchy code.

// Module declarations
pub mod compiler;
pub mod component;
pub mod demo_converter;
pub mod deployment;
pub mod heap;
pub mod portability;
pub mod repl;
pub mod wit;

// Re-exports from compiler
pub use compiler::{WasmCompiler, WasmModule};

// Re-exports from component
pub use component::{ComponentBuilder, ComponentConfig, WasmComponent};

// Re-exports from demo_converter
pub use demo_converter::{
    convert_demo_to_notebook, find_demo_files, Notebook as DemoNotebook,
    NotebookCell as DemoNotebookCell,
};

// Re-exports from deployment
pub use deployment::{Deployer, DeploymentConfig, DeploymentTarget};

// Re-exports from heap
pub use heap::WasmHeap;

// Re-exports from portability
pub use portability::{PortabilityAnalyzer, PortabilityReport, PortabilityScore};

// Re-exports from repl
pub use repl::{ReplOutput, TimingInfo, WasmRepl};

// Re-exports from wit
pub use wit::{InterfaceDefinition, WitGenerator, WitInterface};
