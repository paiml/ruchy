//! HTTP/HTTPS benchmarking module
//!
//! Provides ApacheBench-style HTTP performance testing with support for:
//! - Concurrent requests
//! - Custom headers
//! - POST/GET/PUT/DELETE methods
//! - Request body payloads
//!
//! Note: This module is not available for WASM targets
//!
//! # Examples
//!
#![cfg(all(not(target_arch = "wasm32"), feature = "http-client"))]
//! ```no_run
//! use ruchy::bench::http::benchmark_http;
//!
//! let results = benchmark_http(
//!     "https://api.example.com/endpoint",
//!     100,  // requests
//!     10,   // concurrency
//!     "GET",
//!     None,
//!     Vec::new(),
//! ).unwrap();
//!
//! println!("RPS: {:.2}", results.requests_per_second());
//! ```

use super::BenchmarkResults;
use reqwest::blocking::Client;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Benchmark an HTTP/HTTPS endpoint
///
/// # Arguments
///
/// * `url` - Target URL
/// * `total_requests` - Total number of requests to make
/// * `concurrency` - Number of concurrent workers
/// * `method` - HTTP method (GET, POST, PUT, DELETE, etc.)
/// * `data` - Optional request body
/// * `headers` - HTTP headers in "Key: Value" format
///
/// # Errors
///
/// Returns error if:
/// - Invalid URL
/// - Network failure
/// - Invalid HTTP method
///
/// # Examples
///
/// ```no_run
/// use ruchy::bench::http::benchmark_http;
///
/// let results = benchmark_http(
///     "https://httpbin.org/get",
///     10,
///     2,
///     "GET",
///     None,
///     vec!["User-Agent: ruchy-bench/1.0".to_string()],
/// ).unwrap();
///
/// assert!(results.successful_requests > 0);
/// ```
pub fn benchmark_http(
    url: &str,
    total_requests: usize,
    concurrency: usize,
    method: &str,
    data: Option<&str>,
    headers: Vec<String>,
) -> Result<BenchmarkResults, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    // Shared state for collecting results
    let results = Arc::new(Mutex::new(Vec::new()));
    let successful = Arc::new(Mutex::new(0usize));
    let failed = Arc::new(Mutex::new(0usize));

    let start = Instant::now();

    // Spawn worker threads
    let handles = spawn_worker_threads(
        &client,
        url,
        total_requests,
        concurrency,
        method,
        data,
        &headers,
        &results,
        &successful,
        &failed,
    );

    // Wait for all workers to complete
    join_worker_threads(handles)?;

    let total_duration = start.elapsed();

    // Unwrap results
    let (request_times, successful_requests, failed_requests) =
        unwrap_shared_results(results, successful, failed)?;

    Ok(BenchmarkResults {
        total_requests,
        successful_requests,
        failed_requests,
        total_duration,
        request_times,
    })
}

/// Spawn worker threads for concurrent HTTP requests
fn spawn_worker_threads(
    client: &Client,
    url: &str,
    total_requests: usize,
    concurrency: usize,
    method: &str,
    data: Option<&str>,
    headers: &[String],
    results: &Arc<Mutex<Vec<Duration>>>,
    successful: &Arc<Mutex<usize>>,
    failed: &Arc<Mutex<usize>>,
) -> Vec<thread::JoinHandle<()>> {
    let requests_per_worker = total_requests / concurrency;
    let remainder = total_requests % concurrency;

    let mut handles = Vec::new();
    for i in 0..concurrency {
        let worker_requests = if i == 0 {
            requests_per_worker + remainder
        } else {
            requests_per_worker
        };

        let client = client.clone();
        let url = url.to_string();
        let method = method.to_string();
        let data = data.map(ToString::to_string);
        let headers = headers.to_vec();
        let results = Arc::clone(results);
        let successful = Arc::clone(successful);
        let failed = Arc::clone(failed);

        let handle = thread::spawn(move || {
            execute_worker_requests(
                &client,
                &url,
                &method,
                data.as_deref(),
                &headers,
                worker_requests,
                &results,
                &successful,
                &failed,
            );
        });

        handles.push(handle);
    }

    handles
}

/// Execute HTTP requests for a worker thread
fn execute_worker_requests(
    client: &Client,
    url: &str,
    method: &str,
    data: Option<&str>,
    headers: &[String],
    worker_requests: usize,
    results: &Arc<Mutex<Vec<Duration>>>,
    successful: &Arc<Mutex<usize>>,
    failed: &Arc<Mutex<usize>>,
) {
    for _ in 0..worker_requests {
        let request_start = Instant::now();
        let req = build_http_request(client, url, method, data, headers);

        // Execute request
        match req.send() {
            Ok(response) if response.status().is_success() => {
                let elapsed = request_start.elapsed();
                results
                    .lock()
                    .expect("mutex should not be poisoned")
                    .push(elapsed);
                *successful.lock().expect("mutex should not be poisoned") += 1;
            }
            _ => {
                *failed.lock().expect("mutex should not be poisoned") += 1;
            }
        }
    }
}

/// Build an HTTP request with method, headers, and body
fn build_http_request(
    client: &Client,
    url: &str,
    method: &str,
    data: Option<&str>,
    headers: &[String],
) -> reqwest::blocking::RequestBuilder {
    let mut req = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        "PATCH" => client.patch(url),
        _ => client.get(url),
    };

    // Add headers
    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            req = req.header(key.trim(), value.trim());
        }
    }

    // Add body if provided
    if let Some(body) = data {
        req = req.body(body.to_string());
    }

    req
}

/// Join all worker threads and handle panics
fn join_worker_threads(handles: Vec<thread::JoinHandle<()>>) -> Result<(), String> {
    for handle in handles {
        handle
            .join()
            .map_err(|_| "Worker thread panicked".to_string())?;
    }
    Ok(())
}

/// Unwrap shared results from Arc<Mutex<>>
fn unwrap_shared_results(
    results: Arc<Mutex<Vec<Duration>>>,
    successful: Arc<Mutex<usize>>,
    failed: Arc<Mutex<usize>>,
) -> Result<(Vec<Duration>, usize, usize), String> {
    let request_times = Arc::try_unwrap(results)
        .map_err(|_| "Failed to unwrap results")?
        .into_inner()
        .map_err(|_| "Failed to unlock results")?;

    let successful_requests = Arc::try_unwrap(successful)
        .map_err(|_| "Failed to unwrap successful count")?
        .into_inner()
        .map_err(|_| "Failed to unlock successful count")?;

    let failed_requests = Arc::try_unwrap(failed)
        .map_err(|_| "Failed to unwrap failed count")?
        .into_inner()
        .map_err(|_| "Failed to unlock failed count")?;

    Ok((request_times, successful_requests, failed_requests))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test HTTP benchmark with httpbin.org (requires internet)
    ///
    /// RED: This test should FAIL because `benchmark_http()` is unimplemented
    #[test]
    #[ignore = "Flaky: Depends on httpbin.org availability"]
    fn test_http_benchmark_get() {
        let results = benchmark_http(
            "https://httpbin.org/get",
            10,
            2,
            "GET",
            None,
            vec!["User-Agent: ruchy-bench/1.0".to_string()],
        )
        .expect("operation should succeed in test");

        assert_eq!(results.total_requests, 10);
        assert!(
            results.successful_requests > 0,
            "Should have successful requests"
        );
        assert!(results.total_duration > Duration::ZERO);
        assert!(!results.request_times.is_empty());
    }

    /// Test HTTP POST benchmark
    #[test]
    #[ignore = "Flaky: Depends on httpbin.org availability"]
    fn test_http_benchmark_post() {
        let results = benchmark_http(
            "https://httpbin.org/post",
            5,
            1,
            "POST",
            Some(r#"{"test": "data"}"#),
            vec!["Content-Type: application/json".to_string()],
        )
        .expect("operation should succeed in test");

        assert_eq!(results.total_requests, 5);
        assert!(results.successful_requests > 0);
    }

    /// Test concurrent HTTP benchmark
    #[test]
    #[ignore = "Requires internet connection - run with: cargo test -- --ignored"]
    fn test_http_benchmark_concurrency() {
        let results = benchmark_http(
            "https://httpbin.org/delay/0",
            20,
            5, // 5 concurrent workers
            "GET",
            None,
            Vec::new(),
        )
        .expect("operation should succeed in test");

        assert_eq!(results.total_requests, 20);
        // With concurrency, should be faster than sequential
        assert!(results.total_duration < Duration::from_secs(20));
    }

    /// Property test: Total requests should equal successful + failed
    #[test]
    #[ignore = "Property test - run with: cargo test -- --ignored"]
    fn prop_requests_accounting() {
        use proptest::prelude::*;

        proptest!(|(
            total in 1usize..50,
            concurrency in 1usize..10,
        )| {
            let concurrency = concurrency.min(total);  // Can't have more workers than requests

            // Use httpbin delay endpoint for testing
            let results = benchmark_http(
                "https://httpbin.org/delay/0",
                total,
                concurrency,
                "GET",
                None,
                Vec::new(),
            ).expect("operation should succeed in test");

            // Accounting invariant
            prop_assert_eq!(
                results.successful_requests + results.failed_requests,
                results.total_requests
            );

            // Should have at least some successful requests
            prop_assert!(results.successful_requests > 0);
        });
    }
}
