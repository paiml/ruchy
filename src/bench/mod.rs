//! Benchmarking module for HTTP/HTTPS, WASM, and CLI targets
//!
//! This module provides ApacheBench-style performance testing capabilities
//! for various Ruchy targets. Each benchmark produces comprehensive statistics
//! including latency percentiles, throughput, and success rates.
//!
//! # Examples
//!
//! ```no_run
//! use ruchy::bench::{BenchmarkResults, http_benchmark};
//! use std::time::Duration;
//!
//! let results = http_benchmark(
//!     "https://api.example.com",
//!     100,  // requests
//!     10,   // concurrency
//!     "GET",
//!     None,
//!     Vec::new()
//! ).unwrap();
//!
//! println!("Requests/sec: {:.2}", results.requests_per_second());
//! ```

pub mod cli;
pub mod http;
pub mod stats;
pub mod wasm;

use std::time::Duration;

/// Results from a benchmark run
///
/// # Complexity
/// Cyclomatic complexity: ≤10 (target)
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Total number of requests attempted
    pub total_requests: usize,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Total duration of benchmark
    pub total_duration: Duration,
    /// Individual request times
    pub request_times: Vec<Duration>,
}

impl BenchmarkResults {
    /// Calculate requests per second (throughput)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::BenchmarkResults;
    /// use std::time::Duration;
    ///
    /// let results = BenchmarkResults {
    ///     total_requests: 100,
    ///     successful_requests: 100,
    ///     failed_requests: 0,
    ///     total_duration: Duration::from_secs(10),
    ///     request_times: vec![Duration::from_millis(100); 100],
    /// };
    ///
    /// assert_eq!(results.requests_per_second(), 10.0);
    /// ```
    #[must_use]
    pub fn requests_per_second(&self) -> f64 {
        let secs = self.total_duration.as_secs_f64();
        if secs == 0.0 {
            0.0
        } else {
            self.total_requests as f64 / secs
        }
    }

    /// Calculate mean request time
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::BenchmarkResults;
    /// use std::time::Duration;
    ///
    /// let results = BenchmarkResults {
    ///     total_requests: 3,
    ///     successful_requests: 3,
    ///     failed_requests: 0,
    ///     total_duration: Duration::from_secs(1),
    ///     request_times: vec![
    ///         Duration::from_millis(100),
    ///         Duration::from_millis(200),
    ///         Duration::from_millis(300),
    ///     ],
    /// };
    ///
    /// assert_eq!(results.mean_time(), Duration::from_millis(200));
    /// ```
    #[must_use]
    pub fn mean_time(&self) -> Duration {
        if self.request_times.is_empty() {
            return Duration::ZERO;
        }

        let sum: Duration = self.request_times.iter().sum();
        sum / self.request_times.len() as u32
    }

    /// Calculate percentile of request times
    ///
    /// # Arguments
    ///
    /// * `p` - Percentile value (0.0-100.0)
    ///
    /// # Examples
    ///
    /// ```
    /// use ruchy::bench::BenchmarkResults;
    /// use std::time::Duration;
    ///
    /// let results = BenchmarkResults {
    ///     total_requests: 5,
    ///     successful_requests: 5,
    ///     failed_requests: 0,
    ///     total_duration: Duration::from_secs(1),
    ///     request_times: vec![
    ///         Duration::from_millis(10),
    ///         Duration::from_millis(20),
    ///         Duration::from_millis(30),
    ///         Duration::from_millis(40),
    ///         Duration::from_millis(50),
    ///     ],
    /// };
    ///
    /// assert_eq!(results.percentile(50.0), Duration::from_millis(30));
    /// ```
    #[must_use]
    pub fn percentile(&self, p: f64) -> Duration {
        if self.request_times.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.request_times.clone();
        sorted.sort();

        let index = ((p / 100.0) * sorted.len() as f64) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// Print ApacheBench-style summary to stdout
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use ruchy::bench::BenchmarkResults;
    /// use std::time::Duration;
    ///
    /// let results = BenchmarkResults {
    ///     total_requests: 100,
    ///     successful_requests: 98,
    ///     failed_requests: 2,
    ///     total_duration: Duration::from_secs(10),
    ///     request_times: vec![Duration::from_millis(100); 98],
    /// };
    ///
    /// results.print_summary();
    /// ```
    pub fn print_summary(&self) {
        println!(
            "Requests per second:    {:.2} [#/sec] (mean)",
            self.requests_per_second()
        );
        println!(
            "Time per request:       {:.3} [ms] (mean)",
            self.mean_time().as_secs_f64() * 1000.0
        );

        if self.failed_requests > 0 {
            println!("Failed requests:        {}", self.failed_requests);
        }

        println!("\nPercentage of requests served within a certain time (ms)");
        for p in &[50, 66, 75, 80, 90, 95, 98, 99, 100] {
            let time = self.percentile(f64::from(*p));
            println!("  {:3}%  {:6.0}", p, time.as_secs_f64() * 1000.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `requests_per_second` calculation
    ///
    /// RED: This test should PASS because `requests_per_second()` is implemented
    #[test]
    fn test_requests_per_second() {
        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 100,
            failed_requests: 0,
            total_duration: Duration::from_secs(10),
            request_times: vec![Duration::from_millis(100); 100],
        };

        assert_eq!(results.requests_per_second(), 10.0);
    }

    /// Test `mean_time` calculation
    ///
    /// RED: This test should PASS because `mean_time()` is implemented
    #[test]
    fn test_mean_time() {
        let results = BenchmarkResults {
            total_requests: 3,
            successful_requests: 3,
            failed_requests: 0,
            total_duration: Duration::from_secs(1),
            request_times: vec![
                Duration::from_millis(100),
                Duration::from_millis(200),
                Duration::from_millis(300),
            ],
        };

        assert_eq!(results.mean_time(), Duration::from_millis(200));
    }

    /// Test percentile calculation
    ///
    /// RED: This test should PASS because `percentile()` is implemented
    #[test]
    fn test_percentile() {
        let results = BenchmarkResults {
            total_requests: 5,
            successful_requests: 5,
            failed_requests: 0,
            total_duration: Duration::from_secs(1),
            request_times: vec![
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(30),
                Duration::from_millis(40),
                Duration::from_millis(50),
            ],
        };

        assert_eq!(results.percentile(50.0), Duration::from_millis(30));
        assert_eq!(results.percentile(100.0), Duration::from_millis(50));
    }

    /// Test edge case: empty `request_times`
    #[test]
    fn test_empty_results() {
        let results = BenchmarkResults {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            request_times: vec![],
        };

        assert_eq!(results.requests_per_second(), 0.0);
        assert_eq!(results.mean_time(), Duration::ZERO);
        assert_eq!(results.percentile(50.0), Duration::ZERO);
    }

    /// Property test: Percentiles should be monotonically increasing
    #[test]
    fn prop_percentiles_monotonic() {
        use proptest::prelude::*;

        proptest!(|(times in prop::collection::vec(0u64..10000, 10..100))| {
            let request_times: Vec<Duration> = times.iter()
                .map(|&t| Duration::from_millis(t))
                .collect();

            let results = BenchmarkResults {
                total_requests: request_times.len(),
                successful_requests: request_times.len(),
                failed_requests: 0,
                total_duration: Duration::from_secs(1),
                request_times,
            };

            // Percentiles should be monotonically increasing
            let p50 = results.percentile(50.0);
            let p75 = results.percentile(75.0);
            let p90 = results.percentile(90.0);
            let p99 = results.percentile(99.0);

            prop_assert!(p50 <= p75);
            prop_assert!(p75 <= p90);
            prop_assert!(p90 <= p99);
        });
    }

    #[test]
    fn test_zero_duration_rps() {
        let results = BenchmarkResults {
            total_requests: 100,
            successful_requests: 100,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            request_times: vec![Duration::from_millis(100); 100],
        };
        // Zero duration should return 0.0 to avoid div by zero
        assert_eq!(results.requests_per_second(), 0.0);
    }

    #[test]
    fn test_single_request() {
        let results = BenchmarkResults {
            total_requests: 1,
            successful_requests: 1,
            failed_requests: 0,
            total_duration: Duration::from_millis(500),
            request_times: vec![Duration::from_millis(100)],
        };
        assert_eq!(results.mean_time(), Duration::from_millis(100));
        assert_eq!(results.percentile(50.0), Duration::from_millis(100));
        assert_eq!(results.percentile(100.0), Duration::from_millis(100));
    }

    #[test]
    fn test_failed_requests_ratio() {
        let results = BenchmarkResults {
            total_requests: 10,
            successful_requests: 7,
            failed_requests: 3,
            total_duration: Duration::from_secs(1),
            request_times: vec![Duration::from_millis(100); 7],
        };
        assert_eq!(results.total_requests, 10);
        assert_eq!(results.successful_requests + results.failed_requests, 10);
    }

    #[test]
    fn test_benchmark_results_clone() {
        let results = BenchmarkResults {
            total_requests: 5,
            successful_requests: 5,
            failed_requests: 0,
            total_duration: Duration::from_millis(500),
            request_times: vec![Duration::from_millis(100); 5],
        };
        let cloned = results.clone();
        assert_eq!(cloned.total_requests, results.total_requests);
        assert_eq!(cloned.request_times.len(), results.request_times.len());
    }

    #[test]
    fn test_percentile_boundary_0() {
        let results = BenchmarkResults {
            total_requests: 5,
            successful_requests: 5,
            failed_requests: 0,
            total_duration: Duration::from_secs(1),
            request_times: vec![
                Duration::from_millis(10),
                Duration::from_millis(20),
                Duration::from_millis(30),
                Duration::from_millis(40),
                Duration::from_millis(50),
            ],
        };
        // 0th percentile should be first element
        assert_eq!(results.percentile(0.0), Duration::from_millis(10));
    }

    #[test]
    fn test_high_throughput() {
        let results = BenchmarkResults {
            total_requests: 1000,
            successful_requests: 1000,
            failed_requests: 0,
            total_duration: Duration::from_millis(100),
            request_times: vec![Duration::from_micros(100); 1000],
        };
        // 1000 requests in 100ms = 10000 RPS
        assert_eq!(results.requests_per_second(), 10000.0);
    }

    #[test]
    fn test_percentile_90th() {
        let results = BenchmarkResults {
            total_requests: 10,
            successful_requests: 10,
            failed_requests: 0,
            total_duration: Duration::from_secs(1),
            request_times: (1..=10).map(|i| Duration::from_millis(i * 10)).collect(),
        };
        // 90th percentile of [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
        assert_eq!(results.percentile(90.0), Duration::from_millis(100));
    }

    #[test]
    fn test_mean_time_large_values() {
        let results = BenchmarkResults {
            total_requests: 2,
            successful_requests: 2,
            failed_requests: 0,
            total_duration: Duration::from_secs(10),
            request_times: vec![Duration::from_secs(2), Duration::from_secs(4)],
        };
        assert_eq!(results.mean_time(), Duration::from_secs(3));
    }

    #[test]
    fn test_debug_impl() {
        let results = BenchmarkResults {
            total_requests: 1,
            successful_requests: 1,
            failed_requests: 0,
            total_duration: Duration::from_millis(100),
            request_times: vec![Duration::from_millis(100)],
        };
        let debug_str = format!("{results:?}");
        assert!(debug_str.contains("BenchmarkResults"));
        assert!(debug_str.contains("total_requests"));
    }

    #[test]
    fn test_fractional_rps() {
        let results = BenchmarkResults {
            total_requests: 3,
            successful_requests: 3,
            failed_requests: 0,
            total_duration: Duration::from_secs(2),
            request_times: vec![Duration::from_millis(100); 3],
        };
        // 3 requests in 2 seconds = 1.5 RPS
        assert_eq!(results.requests_per_second(), 1.5);
    }
}
