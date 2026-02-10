//! Ruchy Standard Library (STD-XXX Series)
//!
//! **Accelerated Computing First Data Science** - Six Pillars:
//!
//! | Pillar | Module | Purpose | Acceleration |
//! |--------|--------|---------|--------------|
//! | 1. Compute | `trueno_bridge` | SIMD tensor operations | AVX-512/NEON/CUDA |
//! | 2. Data Loading | `alimentar_bridge` | Dataset loading, transforms | Zero-copy Arrow |
//! | 3. Analytics | `dataframe` | Embedded analytics database | Vectorized execution |
//! | 4. Learning | `aprender_bridge` | ML primitives | SIMD matmul |
//! | 5. Visualization | `viz_bridge` | GPU/WASM charts | WebGPU |
//! | 6. Interaction | `presentar_bridge` | WASM notebook widgets | WASM-native |
//!
//! # Design Philosophy
//!
//! - **Accelerated Computing First**: SIMD/GPU/WASM is the default execution model
//! - **Zero Reinvention**: Leverage existing Rust ecosystem
//! - **Thin Wrappers**: Minimal complexity, maximum reliability
//! - **Ruchy-Friendly**: Clean API that feels natural in Ruchy code
//! - **Toyota Way**: ≤10 complexity per function, comprehensive tests
//! - **Batteries-Included**: Everything needed for data science out of the box
//!
//! # Core Modules (Six Pillars)
//!
//! - `trueno_bridge`: SIMD tensor operations (Pillar 1: Compute)
//! - `alimentar_bridge`: Dataset loading and transforms (Pillar 2: Data Loading)
//! - `aprender_bridge`: ML primitives (Pillar 4: Learning)
//! - `viz_bridge`: GPU/WASM visualization (Pillar 5: Visualization)
//! - `presentar_bridge`: WASM-native widgets (Pillar 6: Interaction)
//!
//! # Utility Modules
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

// === Six Pillars Core (Accelerated Computing First Data Science) ===
#[cfg(feature = "data-loading")]
pub mod alimentar_bridge; // Pillar 2: Data Loading
pub mod aprender_bridge; // Pillar 4: ML Primitives
#[cfg(feature = "widgets")]
pub mod presentar_bridge; // Pillar 6: WASM Widgets
pub mod trueno_bridge; // Pillar 1: SIMD Compute
#[cfg(feature = "visualization")]
pub mod viz_bridge; // Pillar 5: Visualization
                    // Pillar 3 (trueno-db) exposed via dataframe module

// === Utility Modules ===
pub mod env;
pub mod fs;
pub mod json;
pub mod logging;
pub mod path;
pub mod regex;
pub mod time;

// HTTP and process modules require blocking I/O (not available in WASM)
#[cfg(not(target_arch = "wasm32"))]
pub mod http;
#[cfg(not(target_arch = "wasm32"))]
pub mod process;

// HTML parsing (HTTP-002-C, STD-011)
#[cfg(not(target_arch = "wasm32"))]
pub mod html;

#[cfg(feature = "dataframe")]
pub mod dataframe;
