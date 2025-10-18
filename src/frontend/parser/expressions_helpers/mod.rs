//! Expression parsing helper modules
//!
//! This module contains focused submodules extracted from the monolithic
//! expressions.rs file to improve maintainability and enable effective testing.
//!
//! Modularization improves TDG Structural score from 0/25 to target ≥21/25.

pub mod control_flow;
pub mod identifiers;
pub mod literals;
pub mod visibility_modifiers;
