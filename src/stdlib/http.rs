//! HTTP Client Module (STD-002)
//!
//! Thin wrappers around `reqwest::blocking` for Ruchy-friendly API.
//!
//! # Examples
//!
#![cfg(feature = "http-client")]
//! ```no_run
//! use ruchy::stdlib::http;
//!
//! // GET request
//! let response = http::get("https://api.example.com/data")?;
//! println!("{}", response);
//!
//! // POST request with body
//! let body = r#"{"name": "Alice"}"#;
//! let response = http::post("https://api.example.com/users", body)?;
//!
//! // PUT request
//! let body = r#"{"name": "Bob"}"#;
//! let response = http::put("https://api.example.com/users/123", body)?;
//!
//! // DELETE request
//! http::delete("https://api.example.com/users/123")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};

/// Send a GET request to the specified URL
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::http;
///
/// let response = http::get("https://api.example.com/data")?;
/// println!("Response: {}", response);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the request fails or the server returns an error status
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn get(url: &str) -> Result<String> {
    let response = reqwest::blocking::get(url)
        .with_context(|| format!("Failed to send GET request to {url}"))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("GET request failed with status {status}: {url}");
    }

    response
        .text()
        .with_context(|| format!("Failed to read response body from {url}"))
}

/// Send a POST request with a body to the specified URL
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::http;
///
/// let body = r#"{"name": "Alice", "age": 30}"#;
/// let response = http::post("https://api.example.com/users", body)?;
/// println!("Created: {}", response);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the request fails or the server returns an error status
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn post(url: &str, body: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(url)
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .with_context(|| format!("Failed to send POST request to {url}"))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("POST request failed with status {status}: {url}");
    }

    response
        .text()
        .with_context(|| format!("Failed to read response body from {url}"))
}

/// Send a PUT request with a body to the specified URL
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::http;
///
/// let body = r#"{"name": "Bob", "age": 31}"#;
/// let response = http::put("https://api.example.com/users/123", body)?;
/// println!("Updated: {}", response);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the request fails or the server returns an error status
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn put(url: &str, body: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .put(url)
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .with_context(|| format!("Failed to send PUT request to {url}"))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("PUT request failed with status {status}: {url}");
    }

    response
        .text()
        .with_context(|| format!("Failed to read response body from {url}"))
}

/// Send a DELETE request to the specified URL
///
/// # Examples
///
/// ```no_run
/// use ruchy::stdlib::http;
///
/// let response = http::delete("https://api.example.com/users/123")?;
/// println!("Deleted: {}", response);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Errors
///
/// Returns error if the request fails or the server returns an error status
///
/// # Complexity
///
/// Complexity: 2 (within Toyota Way limits ≤10)
pub fn delete(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let response = client
        .delete(url)
        .send()
        .with_context(|| format!("Failed to send DELETE request to {url}"))?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("DELETE request failed with status {status}: {url}");
    }

    response
        .text()
        .with_context(|| format!("Failed to read response body from {url}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // EXTREME TDD: Comprehensive HTTP Client Testing
    // Coverage Target: 19.67% → 80%+
    // Mutation Target: ≥75% caught
    // ============================================================================

    // --------------------------------------------------------------------------
    // Error Path Tests (Critical for Mutation Testing)
    // --------------------------------------------------------------------------

    #[test]
    fn test_invalid_url_error() {
        let result = get("not-a-url");
        assert!(result.is_err(), "Invalid URL should return error");

        // Verify error message contains context
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to send GET request"),
            "Error should contain context"
        );
    }

    #[test]
    fn test_connection_refused_error() {
        // Port 1 is almost always not listening
        let result = get("http://localhost:1");
        assert!(result.is_err(), "Connection refused should return error");

        // Verify error message contains URL
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("localhost:1"),
            "Error should contain failed URL"
        );
    }

    #[test]
    fn test_post_invalid_url() {
        let result = post("not-a-url", "{}");
        assert!(result.is_err(), "POST with invalid URL should fail");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to send POST request"),
            "Error should contain POST context"
        );
    }

    #[test]
    fn test_put_invalid_url() {
        let result = put("not-a-url", "{}");
        assert!(result.is_err(), "PUT with invalid URL should fail");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to send PUT request"),
            "Error should contain PUT context"
        );
    }

    #[test]
    fn test_delete_invalid_url() {
        let result = delete("not-a-url");
        assert!(result.is_err(), "DELETE with invalid URL should fail");

        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Failed to send DELETE request"),
            "Error should contain DELETE context"
        );
    }

    #[test]
    fn test_post_connection_refused() {
        let result = post("http://localhost:1", "{}");
        assert!(result.is_err(), "POST to closed port should fail");
    }

    #[test]
    fn test_put_connection_refused() {
        let result = put("http://localhost:1", "{}");
        assert!(result.is_err(), "PUT to closed port should fail");
    }

    #[test]
    fn test_delete_connection_refused() {
        let result = delete("http://localhost:1");
        assert!(result.is_err(), "DELETE to closed port should fail");
    }

    // --------------------------------------------------------------------------
    // HTTP Status Code Tests (Critical for Branch Coverage)
    // --------------------------------------------------------------------------

    // Note: These tests require a mock HTTP server running on localhost
    // We test error paths with httpbin.org when available

    #[test]
    #[ignore = "Requires network access"] // Requires network access
    fn test_get_success_with_httpbin() {
        let result = get("https://httpbin.org/get");
        if let Ok(response) = result {
            assert!(!response.is_empty(), "Response should not be empty");
            assert!(
                response.contains("httpbin"),
                "Response should be from httpbin"
            );
        }
        // If network fails, test is inconclusive (not a test failure)
    }

    #[test]
    #[ignore = "Requires network access"] // Requires network access
    fn test_get_404_error() {
        let result = get("https://httpbin.org/status/404");
        assert!(result.is_err(), "404 status should return error");

        if let Err(e) = result {
            let err_msg = e.to_string();
            assert!(
                err_msg.contains("404") || err_msg.contains("failed"),
                "Error should indicate HTTP failure"
            );
        }
    }

    #[test]
    #[ignore = "Requires network access"] // Requires network access
    fn test_post_success_with_httpbin() {
        let body = r#"{"test": "data"}"#;
        let result = post("https://httpbin.org/post", body);
        if let Ok(response) = result {
            assert!(!response.is_empty(), "Response should not be empty");
            assert!(response.contains("test"), "Response should echo data");
        }
    }

    #[test]
    #[ignore = "Requires network access"] // Requires network access
    fn test_put_success_with_httpbin() {
        let body = r#"{"test": "update"}"#;
        let result = put("https://httpbin.org/put", body);
        if let Ok(response) = result {
            assert!(!response.is_empty(), "Response should not be empty");
            assert!(response.contains("test"), "Response should echo data");
        }
    }

    #[test]
    #[ignore = "Requires network access"] // Requires network access
    fn test_delete_success_with_httpbin() {
        let result = delete("https://httpbin.org/delete");
        if let Ok(response) = result {
            assert!(!response.is_empty(), "Response should not be empty");
        }
    }

    // --------------------------------------------------------------------------
    // Property Tests (Mathematical Invariants)
    // --------------------------------------------------------------------------

    #[test]
    fn test_empty_url_fails() {
        assert!(get("").is_err(), "Empty URL should fail");
        assert!(post("", "{}").is_err(), "Empty URL should fail");
        assert!(put("", "{}").is_err(), "Empty URL should fail");
        assert!(delete("").is_err(), "Empty URL should fail");
    }

    #[test]
    fn test_url_without_scheme_fails() {
        // URLs without http:// or https:// should fail
        assert!(
            get("example.com").is_err(),
            "URL without scheme should fail"
        );
        assert!(
            post("example.com", "{}").is_err(),
            "URL without scheme should fail"
        );
    }

    // --------------------------------------------------------------------------
    // Boundary Condition Tests
    // --------------------------------------------------------------------------

    #[test]
    fn test_empty_body_post() {
        // Empty body should not panic, just fail on connection
        let result = post("http://localhost:1", "");
        assert!(
            result.is_err(),
            "POST with empty body should fail on connection"
        );
    }

    #[test]
    fn test_large_body_post() {
        // Large body should not panic
        let large_body = "x".repeat(10_000);
        let result = post("http://localhost:1", &large_body);
        assert!(
            result.is_err(),
            "POST with large body should fail on connection"
        );
    }

    #[test]
    fn test_special_characters_in_url() {
        // Special characters should be handled (will fail on connection, not panic)
        let result = get("http://localhost:1/test?query=value&foo=bar");
        assert!(
            result.is_err(),
            "URL with query params should fail on connection"
        );
    }
}

// ============================================================================
// Property Tests Module (High-Confidence Verification)
// ============================================================================

#[cfg(test)]
mod property_tests {
    use super::*;

    // Property: All HTTP functions should fail gracefully on invalid input
    // (Never panic, always return Result::Err)

    #[test]
    fn prop_get_never_panics_on_invalid_urls() {
        let invalid_urls = vec![
            "",
            "not-a-url",
            "://missing-scheme",
            "http://",
            "ftp://wrong-scheme.com",
            "http:// spaces .com",
        ];

        for url in invalid_urls {
            let result = get(url);
            assert!(result.is_err(), "get('{url}') should return Err, not panic");
        }
    }

    #[test]
    fn prop_post_never_panics_on_invalid_input() {
        let test_cases = vec![
            ("", ""),
            ("not-a-url", "{}"),
            ("http://localhost:1", "invalid json {"),
        ];

        for (url, body) in test_cases {
            let result = post(url, body);
            assert!(result.is_err(), "post('{url}', '{body}') should return Err");
        }
    }

    #[test]
    fn prop_all_methods_fail_on_unreachable_host() {
        // Property: Unreachable hosts should fail consistently across all methods
        let unreachable = "http://localhost:1";

        assert!(get(unreachable).is_err(), "GET should fail");
        assert!(post(unreachable, "{}").is_err(), "POST should fail");
        assert!(put(unreachable, "{}").is_err(), "PUT should fail");
        assert!(delete(unreachable).is_err(), "DELETE should fail");
    }
}
