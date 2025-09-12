//! WebAssembly component toolkit for Ruchy (RUCHY-0819)
//!
//! This module provides WebAssembly component generation, WIT interface generation,
//! platform-specific deployment, and portability scoring for Ruchy code.
pub mod component;
pub mod wit;
pub mod deployment;
pub mod portability;
pub mod repl;
pub mod notebook;
pub mod shared_session;
pub mod demo_converter;

#[cfg(test)]
pub mod webworker_tests;
pub use component::{WasmComponent, ComponentBuilder, ComponentConfig};
pub use wit::{WitInterface, WitGenerator, InterfaceDefinition};
pub use deployment::{DeploymentTarget, Deployer, DeploymentConfig};
pub use portability::{PortabilityScore, PortabilityAnalyzer, PortabilityReport};
pub use repl::{WasmRepl, ReplOutput, TimingInfo};
pub use notebook::{NotebookRuntime, NotebookCell, Notebook, CellType, CellOutput};
pub use shared_session::{SharedSession, GlobalRegistry, DefId, ExecutionMode, ExecuteResponse};
pub use demo_converter::{convert_demo_to_notebook, find_demo_files, NotebookCell as DemoNotebookCell, Notebook as DemoNotebook};