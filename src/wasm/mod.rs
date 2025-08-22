//! WebAssembly component toolkit for Ruchy (RUCHY-0819)
//!
//! This module provides WebAssembly component generation, WIT interface generation,
//! platform-specific deployment, and portability scoring for Ruchy code.

pub mod component;
pub mod wit;
pub mod deployment;
pub mod portability;

pub use component::{WasmComponent, ComponentBuilder, ComponentConfig};
pub use wit::{WitInterface, WitGenerator, InterfaceDefinition};
pub use deployment::{DeploymentTarget, Deployer, DeploymentConfig};
pub use portability::{PortabilityScore, PortabilityAnalyzer, PortabilityReport};