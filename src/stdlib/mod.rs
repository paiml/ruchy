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
//! - `http`: HTTP client operations (STD-002)
//! - `json`: JSON parsing and manipulation (STD-003)
//! - `path`: Path manipulation operations (STD-004)

pub mod fs;
pub mod http;
pub mod json;
pub mod path;
