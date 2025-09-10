//! TDD Test for Server Route Registration
//! Tests that all API routes are properly registered and accessible

use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;

// We need to import the server module functions
use ruchy_notebook::server::{create_app, start_server};

#[tokio::test]
async fn test_health_endpoint_exists() {
    let app = create_app();
    
    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_debug_endpoint_exists() {
    let app = create_app();
    
    let request = Request::builder()
        .method("GET")
        .uri("/api/debug")
        .body(Body::empty())
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let body_str = std::str::from_utf8(&body).unwrap();
    assert_eq!(body_str, "API is working!");
}

#[tokio::test]
async fn test_api_execute_endpoint_exists() {
    let app = create_app();
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"2 + 2","cell_id":"test","session_id":"test"}"#
        ))
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    // Should NOT be 404 - this is what we're testing
    assert_ne!(response.status(), StatusCode::NOT_FOUND, 
        "API execute endpoint returns 404 - route not registered!");
    
    // Should be OK (200) or some other valid status, not 404
    assert!(response.status().is_success() || response.status().is_client_error() && response.status() != StatusCode::NOT_FOUND);
}

#[tokio::test] 
async fn test_all_expected_routes_registered() {
    let app = create_app();
    
    // Test all the routes we expect to exist
    let routes_to_test = vec![
        ("GET", "/", StatusCode::OK),
        ("GET", "/health", StatusCode::OK), 
        ("GET", "/api/debug", StatusCode::OK),
        ("POST", "/api/execute", StatusCode::OK), // This should NOT be 404
    ];
    
    for (method, path, expected_status) in routes_to_test {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(r#"{"code":"test","cell_id":"test","session_id":"test"}"#))
            .unwrap();
        
        let response = app
            .clone()
            .oneshot(request)
            .await
            .unwrap();
        
        if path == "/api/execute" {
            // This is the failing case we want to fix
            assert_ne!(response.status(), StatusCode::NOT_FOUND, 
                "Route {} {} returns 404 - not registered!", method, path);
        } else {
            // Other routes should work
            assert_eq!(response.status(), expected_status, 
                "Route {} {} failed", method, path);
        }
    }
}

#[tokio::test]
async fn test_route_debug_info() {
    // This test helps us debug what routes are actually registered
    let app = create_app();
    
    // Try a definitely non-existent route to see the 404 behavior
    let request = Request::builder()
        .method("GET")
        .uri("/definitely/does/not/exist")
        .body(Body::empty())
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    // This should definitely be 404
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    // Now test our problematic route
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute") 
        .header("content-type", "application/json")
        .body(Body::from(r#"{"code":"println(\"hello\")","cell_id":"test","session_id":"test"}"#))
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    println!("API execute response status: {:?}", response.status());
    
    // If this fails, we know the route isn't registered
    assert_ne!(response.status(), StatusCode::NOT_FOUND, 
        "API execute endpoint is returning 404 - the route is not being registered properly!");
}