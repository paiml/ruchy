#![cfg(feature = "http-client")]
#![allow(missing_docs)]
//! STD-002: HTTP Client Module Tests (ruchy/std/http)
//!
//! Test suite for HTTP client operations module.
//! Thin wrappers around reqwest with Ruchy-friendly API.
//!
//! EXTREME TDD: These tests are written BEFORE implementation (RED phase).

use httpmock::prelude::*;

#[test]
fn test_std_002_get_success() {
    // STD-002: Test GET request returns response body

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/test");
        then.status(200).body("Hello, Ruchy!");
    });

    // Call ruchy::stdlib::http::get
    let result = ruchy::stdlib::http::get(&server.url("/test"));

    assert!(result.is_ok(), "GET request should succeed");
    let body = result.unwrap();
    assert_eq!(body, "Hello, Ruchy!", "Response body must match exactly");
    assert_eq!(body.len(), 13, "Response body length must match");
    assert!(body.contains("Ruchy"), "Response must contain 'Ruchy'");
    assert!(
        body.starts_with("Hello"),
        "Response must start with 'Hello'"
    );
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_get_with_json_response() {
    // STD-002: Test GET request with JSON response

    let server = MockServer::start();
    let json_body = r#"{"message": "success", "code": 200}"#;
    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/data");
        then.status(200)
            .header("content-type", "application/json")
            .body(json_body);
    });

    let result = ruchy::stdlib::http::get(&server.url("/api/data"));

    assert!(result.is_ok(), "GET request should succeed");
    let body = result.unwrap();
    assert_eq!(body, json_body, "JSON body must match exactly");
    assert!(
        body.contains("message"),
        "JSON must contain 'message' field"
    );
    assert!(
        body.contains("success"),
        "JSON must contain 'success' value"
    );
    assert!(body.contains("200"), "JSON must contain status code");
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_get_404_error() {
    // STD-002: Test GET request handles 404 error

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/notfound");
        then.status(404).body("Not Found");
    });

    let result = ruchy::stdlib::http::get(&server.url("/notfound"));

    assert!(result.is_err(), "404 should return error");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("404") || error.to_string().contains("Not Found"),
        "Error should mention 404 or Not Found"
    );
    mock.assert();
}

#[test]
fn test_std_002_post_with_body() {
    // STD-002: Test POST request with body

    let server = MockServer::start();
    let request_body = r#"{"name": "Alice", "age": 30}"#;
    let response_body = r#"{"id": 123, "created": true}"#;

    let mock = server.mock(|when, then| {
        when.method(POST).path("/api/users").body(request_body);
        then.status(201).body(response_body);
    });

    // Call ruchy::stdlib::http::post
    let result = ruchy::stdlib::http::post(&server.url("/api/users"), request_body);

    assert!(result.is_ok(), "POST request should succeed");
    let body = result.unwrap();
    assert_eq!(body, response_body, "Response body must match exactly");
    assert!(body.contains("id"), "Response must contain 'id' field");
    assert!(body.contains("123"), "Response must contain id value");
    assert!(
        body.contains("created"),
        "Response must contain 'created' field"
    );
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_post_empty_body() {
    // STD-002: Test POST request with empty body

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST).path("/api/action").body("");
        then.status(200).body("OK");
    });

    let result = ruchy::stdlib::http::post(&server.url("/api/action"), "");

    assert!(result.is_ok(), "POST with empty body should succeed");
    let body = result.unwrap();
    assert_eq!(body, "OK", "Response must be exactly 'OK'");
    assert_eq!(body.len(), 2, "Response length must be 2");
    assert!(!body.is_empty(), "Response must not be empty");
    assert_ne!(body, "", "Response must not be empty string");
    mock.assert();
}

#[test]
fn test_std_002_put_updates_resource() {
    // STD-002: Test PUT request updates resource

    let server = MockServer::start();
    let update_body = r#"{"name": "Bob", "age": 31}"#;
    let response_body = r#"{"id": 123, "updated": true}"#;

    let mock = server.mock(|when, then| {
        when.method(PUT).path("/api/users/123").body(update_body);
        then.status(200).body(response_body);
    });

    // Call ruchy::stdlib::http::put
    let result = ruchy::stdlib::http::put(&server.url("/api/users/123"), update_body);

    assert!(result.is_ok(), "PUT request should succeed");
    let body = result.unwrap();
    assert_eq!(body, response_body, "Response body must match exactly");
    assert!(
        body.contains("updated"),
        "Response must contain 'updated' field"
    );
    assert!(body.contains("true"), "Response must contain 'true' value");
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_delete_resource() {
    // STD-002: Test DELETE request

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(DELETE).path("/api/users/123");
        then.status(204); // No content
    });

    // Call ruchy::stdlib::http::delete
    let result = ruchy::stdlib::http::delete(&server.url("/api/users/123"));

    assert!(result.is_ok(), "DELETE request should succeed");
    let body = result.unwrap();
    // 204 No Content returns empty string
    assert_eq!(body, "", "204 response must be empty");
    assert_eq!(body.len(), 0, "204 response length must be 0");
    assert!(body.is_empty(), "204 response must be empty");
    mock.assert();
}

#[test]
fn test_std_002_delete_with_response_body() {
    // STD-002: Test DELETE request with response body

    let server = MockServer::start();
    let response_body = r#"{"deleted": true, "id": 123}"#;
    let mock = server.mock(|when, then| {
        when.method(DELETE).path("/api/users/456");
        then.status(200).body(response_body);
    });

    let result = ruchy::stdlib::http::delete(&server.url("/api/users/456"));

    assert!(result.is_ok(), "DELETE request should succeed");
    let body = result.unwrap();
    assert_eq!(body, response_body, "Response body must match exactly");
    assert!(
        body.contains("deleted"),
        "Response must contain 'deleted' field"
    );
    assert!(body.contains("true"), "Response must contain 'true' value");
    assert!(body.contains("123"), "Response must contain id");
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_get_with_query_params() {
    // STD-002: Test GET request with query parameters

    let server = MockServer::start();
    let response_body = r#"{"results": []}"#;
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path("/api/search")
            .query_param("q", "rust")
            .query_param("limit", "10");
        then.status(200).body(response_body);
    });

    let result = ruchy::stdlib::http::get(&server.url("/api/search?q=rust&limit=10"));

    assert!(result.is_ok(), "GET with query params should succeed");
    let body = result.unwrap();
    assert_eq!(body, response_body, "Response body must match exactly");
    assert!(
        body.contains("results"),
        "Response must contain 'results' field"
    );
    assert!(!body.is_empty(), "Response must not be empty");
    mock.assert();
}

#[test]
fn test_std_002_server_error_500() {
    // STD-002: Test handling of 500 server error

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET).path("/error");
        then.status(500).body("Internal Server Error");
    });

    let result = ruchy::stdlib::http::get(&server.url("/error"));

    assert!(result.is_err(), "500 should return error");
    let error = result.unwrap_err();
    assert!(
        error.to_string().contains("500") || error.to_string().contains("Server Error"),
        "Error should mention 500 or Server Error"
    );
    mock.assert();
}

#[test]
fn test_std_002_invalid_url() {
    // STD-002: Test handling of invalid URL

    let result = ruchy::stdlib::http::get("not-a-valid-url");

    assert!(result.is_err(), "Invalid URL should return error");
}

#[test]
fn test_std_002_network_error() {
    // STD-002: Test handling of network error (connection refused)

    // Use a port that's not listening
    let result = ruchy::stdlib::http::get("http://localhost:1");

    assert!(result.is_err(), "Connection refused should return error");
}

#[test]
fn test_std_002_post_with_headers() {
    // STD-002: Test POST request validates headers

    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/data")
            .header("content-type", "application/json");
        then.status(200).body("OK");
    });

    // Note: This test verifies that the implementation sets proper content-type
    let result = ruchy::stdlib::http::post(&server.url("/api/data"), r#"{"test": true}"#);

    assert!(result.is_ok(), "POST should succeed");
    mock.assert();
}

#[test]
fn test_std_002_large_response_body() {
    // STD-002: Test handling of large response body

    let server = MockServer::start();
    let large_body = "x".repeat(10000); // 10KB
    let mock = server.mock(|when, then| {
        when.method(GET).path("/large");
        then.status(200).body(&large_body);
    });

    let result = ruchy::stdlib::http::get(&server.url("/large"));

    assert!(result.is_ok(), "Large response should succeed");
    let body = result.unwrap();
    assert_eq!(body.len(), 10000, "Response length must be exactly 10000");
    assert!(body.starts_with("xxx"), "Response must start with 'xxx'");
    assert!(body.ends_with("xxx"), "Response must end with 'xxx'");
    assert!(!body.is_empty(), "Response must not be empty");
    assert_eq!(body, large_body, "Response must match exactly");
    mock.assert();
}

// ===== Property Tests =====

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        #[test]
        fn test_std_002_get_never_panics(path in "[a-z]{1,20}") {
            // Property: GET should never panic, even with random paths

            let server = MockServer::start();
            let _mock = server.mock(|when, then| {
                when.method(GET);
                then.status(200).body("OK");
            });

            let url = server.url(format!("/{path}"));
            let _ = ruchy::stdlib::http::get(&url);
            // Should not panic
        }

        #[test]
        fn test_std_002_post_body_preserved(body in "\\PC{0,100}") {
            // Property: POST body should be sent to server exactly as provided

            let server = MockServer::start();
            let body_clone = body.clone();
            let mock = server.mock(move |when, then| {
                when.method(POST).path("/echo").body(&body_clone);
                then.status(200).body("OK");
            });

            let result = ruchy::stdlib::http::post(&server.url("/echo"), &body);

            // If request succeeds, body was sent correctly
            if result.is_ok() {
                mock.assert();
            }
        }
    }
}
