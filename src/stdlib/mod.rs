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
//! - `env`: Environment operations (STD-005)
//! - `process`: Process operations (STD-006)
//! - `dataframe`: `DataFrame` operations (STD-007, requires `dataframe` feature)
//! - `time`: Time operations (STD-008)
//! - `logging`: Logging operations (STD-009)
//! - `regex`: Regular expression operations (STD-010)

pub mod env;
pub mod fs;
pub mod http;
pub mod json;
pub mod logging;
pub mod path;
pub mod process;
pub mod regex;
pub mod time;

#[cfg(feature = "dataframe")]
pub mod dataframe;
