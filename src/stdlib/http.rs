//! HTTP Client Module (STD-002)
//!
//! Thin wrappers around `reqwest::blocking` for Ruchy-friendly API.
//!
//! # Examples
//!
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

    #[test]
    fn test_invalid_url_error() {
        let result = get("not-a-url");
        assert!(result.is_err(), "Invalid URL should return error");
    }

    #[test]
    fn test_connection_refused_error() {
        // Port 1 is almost always not listening
        let result = get("http://localhost:1");
        assert!(result.is_err(), "Connection refused should return error");
    }
}
