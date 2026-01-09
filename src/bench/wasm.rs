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
#[cfg(feature = "notebook")]
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
#[cfg(feature = "notebook")] // WASM benchmarking requires wasmtime
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
    /// RED: This test should FAIL because `benchmark_wasm()` is unimplemented
    #[test]
    #[cfg(feature = "notebook")]
    fn test_wasm_benchmark() {
        // Note: This requires a test WASM module
        // For now, this is a placeholder test
        let module_path = PathBuf::from("tests/fixtures/test.wasm");
        if !module_path.exists() {
            eprintln!("Skipping test - no WASM fixture at {module_path:?}");
            return;
        }

        let results = benchmark_wasm(&module_path, "add", &[1, 2], 10).unwrap();

        assert_eq!(results.total_requests, 10);
        assert!(results.successful_requests > 0);
    }

    /// Test WASM benchmark without notebook feature
    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled() {
        let result = benchmark_wasm(&PathBuf::from("test.wasm"), "add", &[1, 2], 10);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("notebook"));
    }

    #[test]
    #[cfg(feature = "notebook")]
    fn test_wasm_benchmark_invalid_path() {
        let result = benchmark_wasm(
            &PathBuf::from("/nonexistent/path/module.wasm"),
            "add",
            &[1, 2],
            10,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to load"));
    }

    #[test]
    #[cfg(feature = "notebook")]
    fn test_wasm_benchmark_zero_iterations() {
        let module_path = PathBuf::from("tests/fixtures/test.wasm");
        if !module_path.exists() {
            return; // Skip if no fixture
        }
        let results = benchmark_wasm(&module_path, "add", &[1, 2], 0).unwrap();
        assert_eq!(results.total_requests, 0);
    }

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled_error_message() {
        let result = benchmark_wasm(&PathBuf::from("any.wasm"), "func", &[], 1);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("wasmtime"));
    }

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled_with_args() {
        let result = benchmark_wasm(&PathBuf::from("test.wasm"), "multiply", &[5, 10], 100);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled_empty_args() {
        let result = benchmark_wasm(&PathBuf::from("test.wasm"), "noop", &[], 50);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(feature = "notebook"))]
    fn test_wasm_benchmark_disabled_many_iterations() {
        let result = benchmark_wasm(&PathBuf::from("test.wasm"), "sum", &[1, 2, 3], 10000);
        assert!(result.is_err());
    }

    // Additional coverage tests for BenchmarkResults usage
    #[test]
    fn test_benchmark_results_struct() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 95,
            failed_requests: 5,
            total_duration: Duration::from_secs(10),
            request_times: vec![Duration::from_millis(100); 95],
        };

        assert_eq!(results.total_requests, 100);
        assert_eq!(results.successful_requests, 95);
        assert_eq!(results.failed_requests, 5);
        assert_eq!(results.total_duration, Duration::from_secs(10));
        assert_eq!(results.request_times.len(), 95);
    }

    #[test]
    fn test_benchmark_results_mean_time() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            total_duration: Duration::from_secs(1),
            request_times: vec![Duration::from_millis(100); 10],
        };

        let mean = results.mean_time();
        assert_eq!(mean, Duration::from_millis(100));
    }

    #[test]
    fn test_benchmark_results_empty_times() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            request_times: vec![],
        };

        // Empty results should return zero duration
        let mean = results.mean_time();
        assert_eq!(mean, Duration::ZERO);
    }

    #[test]
    fn test_benchmark_results_percentiles() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 100,
            failed_requests: 0,
            total_duration: Duration::from_secs(10),
            request_times: (1..=100).map(|i| Duration::from_millis(i)).collect(),
        };

        let p50 = results.percentile(50.0);
        let p90 = results.percentile(90.0);
        let p99 = results.percentile(99.0);

        assert!(p50 <= p90);
        assert!(p90 <= p99);
    }

    #[test]
    fn test_benchmark_results_success_ratio() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 80,
            failed_requests: 20,
            total_duration: Duration::from_secs(10),
            request_times: vec![Duration::from_millis(100); 80],
        };

        // Calculate success rate manually
        let rate = results.successful_requests as f64 / results.total_requests as f64;
        assert!((rate - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_benchmark_results_requests_per_second() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 100,
            failed_requests: 0,
            total_duration: Duration::from_secs(10),
            request_times: vec![Duration::from_millis(100); 100],
        };

        let rps = results.requests_per_second();
        assert!((rps - 10.0).abs() < 0.001); // 100 requests / 10 seconds = 10 req/s
    }

    #[test]
    fn test_benchmark_results_zero_duration_rps() {
        use super::super::BenchmarkResults;
        use std::time::Duration;

        let results = BenchmarkResults {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            request_times: vec![],
        };

        // Zero duration should return 0.0 (handles gracefully)
        let rps = results.requests_per_second();
        assert_eq!(rps, 0.0);
    }
}
