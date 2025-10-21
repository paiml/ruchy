//! WASM module benchmarking
//!
//! Provides performance testing for WebAssembly modules with:
//! - Function invocation benchmarking
//! - Memory allocation tracking
//! - Execution time percentiles
//!
//! Note: This module is not available for WASM targets (it benchmarks WASM from native code)
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::bench::wasm::benchmark_wasm;
//! use std::path::PathBuf;
//!
//! let results = benchmark_wasm(
//!     &PathBuf::from("module.wasm"),
//!     "add",
//!     &[1, 2],  // Function arguments
//!     1000,     // iterations
//! ).unwrap();
//!
//! println!("Mean: {:.2}µs", results.mean_time().as_micros());
//! ```
#![cfg(not(target_arch = "wasm32"))]

use super::BenchmarkResults;
use std::path::Path;
use std::time::Instant;

/// Benchmark a WASM module function
///
/// # Arguments
///
/// * `module_path` - Path to .wasm file
/// * `function_name` - Name of exported function to benchmark
/// * `args` - Function arguments (i32 values)
/// * `iterations` - Number of iterations to run
///
/// # Errors
///
/// Returns error if:
/// - WASM file not found
/// - Function not exported
/// - Invalid arguments
/// - WASM runtime error
///
/// # Examples
///
/// ```no_run
/// use ruchy::bench::wasm::benchmark_wasm;
/// use std::path::PathBuf;
///
/// let results = benchmark_wasm(
///     &PathBuf::from("test.wasm"),
///     "factorial",
///     &[5],
///     100,
/// ).unwrap();
/// ```
#[cfg(feature = "notebook")]  // WASM benchmarking requires wasmtime
pub fn benchmark_wasm(
    module_path: &Path,
    function_name: &str,
    args: &[i32],
    iterations: usize,
) -> Result<BenchmarkResults, String> {
    use wasmtime::{Engine, Instance, Module, Store, Val};

    // Load WASM module
    let engine = Engine::default();
    let module = Module::from_file(&engine, module_path)
        .map_err(|e| format!("Failed to load WASM module: {e}"))?;

    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[])
        .map_err(|e| format!("Failed to instantiate WASM module: {e}"))?;

    // Get function
    let func = instance
        .get_func(&mut store, function_name)
        .ok_or_else(|| format!("Function '{function_name}' not found in WASM module"))?;

    // Validate function signature
    let func_ty = func.ty(&store);
    if func_ty.params().len() != args.len() {
        return Err(format!(
            "Function expects {} arguments, got {}",
            func_ty.params().len(),
            args.len()
        ));
    }

    // Convert args to Val
    let wasm_args: Vec<Val> = args.iter().map(|&v| Val::I32(v)).collect();
    let mut results_buffer = vec![Val::I32(0); func_ty.results().len()];

    // Benchmark iterations
    let mut request_times = Vec::with_capacity(iterations);
    let mut successful = 0;
    let mut failed = 0;

    let start = Instant::now();

    for _ in 0..iterations {
        let iteration_start = Instant::now();

        match func.call(&mut store, &wasm_args, &mut results_buffer) {
            Ok(()) => {
                let elapsed = iteration_start.elapsed();
                request_times.push(elapsed);
                successful += 1;
            }
            Err(_) => {
                failed += 1;
            }
        }
    }

    let total_duration = start.elapsed();

    Ok(BenchmarkResults {
        total_requests: iterations,
        successful_requests: successful,
        failed_requests: failed,
        total_duration,
        request_times,
    })
}

/// Stub implementation for when notebook feature is disabled
#[cfg(not(feature = "notebook"))]
pub fn benchmark_wasm(
    _module_path: &Path,
    _function_name: &str,
    _args: &[i32],
    _iterations: usize,
) -> Result<BenchmarkResults, String> {
    Err("WASM benchmarking requires 'notebook' feature (wasmtime dependency)".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Test WASM benchmark (requires test fixture)
    ///
    /// RED: This test should FAIL because benchmark_wasm() is unimplemented
    #[test]
    #[ignore = "Requires WASM test fixture - run with: cargo test -- --ignored"]
    #[cfg(feature = "notebook")]
    fn test_wasm_benchmark() {
        // Note: This requires a test WASM module
        // For now, this is a placeholder test
        let module_path = PathBuf::from("tests/fixtures/test.wasm");
        if !module_path.exists() {
            eprintln!("Skipping test - no WASM fixture at {:?}", module_path);
            return;
        }

        let results = benchmark_wasm(
            &module_path,
            "add",
            &[1, 2],
            10,
        ).unwrap();

        assert_eq!(results.total_requests, 10);
        assert!(results.successful_requests > 0);
    }

    /// Test WASM benchmark without notebook feature
    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled() {
        let result = benchmark_wasm(
            &PathBuf::from("test.wasm"),
            "add",
            &[1, 2],
            10,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("notebook"));
    }
}
