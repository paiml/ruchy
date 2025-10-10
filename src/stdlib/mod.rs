//! Ruchy Standard Library (STD-XXX Series)
//!
//! Thin wrappers around proven Rust crates for Ruchy-friendly API.
//!
//! # Design Philosophy
//!
//! - **Zero Reinvention**: Leverage existing Rust ecosystem
//! - **Thin Wrappers**: Minimal complexity, maximum reliability
//! - **Ruchy-Friendly**: Clean API that feels natural in Ruchy code
//! - **Toyota Way**: ≤10 complexity per function, comprehensive tests
//!
//! # Modules
//!
//! - `fs`: File system operations (STD-001)

pub mod fs;
