//! CLI command benchmarking module
//!
//! Provides performance testing for shell commands and CLI tools with:
//! - Warmup iterations to eliminate cold-start effects
//! - Statistical analysis of execution times
//! - Standard deviation and percentile reporting
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::bench::cli::benchmark_cli;
//!
//! let results = benchmark_cli(
//!     "echo 'hello world'",
//!     10,  // iterations
//!     3,   // warmup runs
//! ).unwrap();
//!
//! println!("Mean: {:.2}ms", results.mean_time().as_secs_f64() * 1000.0);
//! ```

use super::BenchmarkResults;
use std::process::Command;
use std::time::Instant;

/// Benchmark a CLI command
///
/// # Arguments
///
/// * `command` - Shell command to execute
/// * `iterations` - Number of timed iterations
/// * `warmup` - Number of warmup runs (not timed)
///
/// # Errors
///
/// Returns error if:
/// - Command fails to execute
/// - Command returns non-zero exit code
///
/// # Examples
///
/// ```no_run
/// use ruchy::bench::cli::benchmark_cli;
///
/// let results = benchmark_cli("echo test", 5, 2).unwrap();
/// assert_eq!(results.total_requests, 5);
/// ```
pub fn benchmark_cli(
    command: &str,
    iterations: usize,
    warmup: usize,
) -> Result<BenchmarkResults, String> {
    // Warmup runs (not timed)
    for _ in 0..warmup {
        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| format!("Failed to execute warmup command: {e}"))?;

        if !status.status.success() {
            return Err(format!(
                "Command failed during warmup: {}",
                String::from_utf8_lossy(&status.stderr)
            ));
        }
    }

    // Timed iterations
    let mut request_times = Vec::with_capacity(iterations);
    let mut successful = 0;
    let mut failed = 0;

    let start = Instant::now();

    for _ in 0..iterations {
        let iteration_start = Instant::now();

        let status = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| format!("Failed to execute command: {e}"))?;

        let elapsed = iteration_start.elapsed();

        if status.status.success() {
            request_times.push(elapsed);
            successful += 1;
        } else {
            failed += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Test CLI benchmark with echo command
    ///
    /// RED: This test should PASS because `benchmark_cli()` is implemented
    #[test]
    fn test_cli_benchmark_echo() {
        let results = benchmark_cli("echo 'test'", 5, 2).expect("operation should succeed in test");

        assert_eq!(results.total_requests, 5);
        assert_eq!(results.successful_requests, 5);
        assert_eq!(results.failed_requests, 0);
        assert!(!results.request_times.is_empty());
    }

    /// Test CLI benchmark with failing command
    #[test]
    fn test_cli_benchmark_failure() {
        // Command that fails during warmup
        let results = benchmark_cli("exit 1", 3, 1);

        // Warmup should fail and propagate error
        assert!(results.is_err());
        assert!(results.unwrap_err().contains("warmup"));
    }

    /// Test CLI benchmark with sleep command (timing validation)
    #[test]
    fn test_cli_benchmark_timing() {
        let results = benchmark_cli("sleep 0.01", 3, 1).expect("operation should succeed in test");

        assert_eq!(results.total_requests, 3);
        assert_eq!(results.successful_requests, 3);

        // Each iteration should take at least 10ms
        for time in &results.request_times {
            assert!(time.as_millis() >= 10, "Each sleep should take ≥10ms");
        }
    }

    /// Test warmup iterations don't affect timing
    #[test]
    fn test_cli_benchmark_warmup() {
        let with_warmup =
            benchmark_cli("echo 'test'", 10, 5).expect("operation should succeed in test");
        let without_warmup =
            benchmark_cli("echo 'test'", 10, 0).expect("operation should succeed in test");

        // Both should have same number of timed iterations
        assert_eq!(with_warmup.total_requests, 10);
        assert_eq!(without_warmup.total_requests, 10);
        assert_eq!(with_warmup.request_times.len(), 10);
        assert_eq!(without_warmup.request_times.len(), 10);
    }

    /// Property test: All successful iterations should be recorded
    #[test]
    fn prop_all_iterations_recorded() {
        use proptest::prelude::*;

        proptest!(|(
            iterations in 1usize..20,
            warmup in 0usize..5,
        )| {
            let results = benchmark_cli("echo 'test'", iterations, warmup).expect("operation should succeed in test");

            // All iterations should succeed
            prop_assert_eq!(results.successful_requests, iterations);
            prop_assert_eq!(results.failed_requests, 0);
            prop_assert_eq!(results.request_times.len(), iterations);

            // Total duration should be sum of all iterations (approximately)
            let sum: Duration = results.request_times.iter().sum();
            prop_assert!(results.total_duration >= sum);
        });
    }

    // === EXTREME TDD Round 17 tests ===

    #[test]
    fn test_benchmark_results_fields() {
        let results = benchmark_cli("echo 'hello'", 3, 0).expect("operation should succeed in test");

        assert_eq!(results.total_requests, 3);
        assert!(results.total_duration > Duration::ZERO);
        assert!(!results.request_times.is_empty());
    }

    #[test]
    fn test_benchmark_zero_warmup() {
        let results = benchmark_cli("echo 'no warmup'", 2, 0).expect("operation should succeed in test");

        assert_eq!(results.total_requests, 2);
        assert_eq!(results.successful_requests, 2);
        assert_eq!(results.failed_requests, 0);
    }

    #[test]
    fn test_benchmark_command_output_ignored() {
        // Command output should not affect benchmarking
        let results =
            benchmark_cli("echo 'lots of output'; echo 'more output'", 2, 0).expect("should succeed");

        assert_eq!(results.successful_requests, 2);
    }
}
