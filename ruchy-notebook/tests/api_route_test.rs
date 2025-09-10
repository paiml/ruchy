//! TDD Test for API Route Registration
//! FAILING TEST: Proves /api/execute route returns 404

use axum::http::{Request, StatusCode};
use axum::body::Body;
use tower::ServiceExt;

#[tokio::test]
async fn test_api_execute_route_is_registered() {
    // This test should FAIL initially because route returns 404
    
    // Create the app using the server module
    let app = ruchy_notebook::server::create_app();
    
    // Make request to /api/execute
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"2 + 2","cell_id":"test-1","session_id":"test"}"#
        ))
        .unwrap();
    
    let response = app
        .oneshot(request)
        .await
        .unwrap();
    
    // This assertion should FAIL if route is not registered (404)
    assert_ne!(response.status(), StatusCode::NOT_FOUND, 
        "FAILING TEST: /api/execute returns 404 - route not registered!");
    
    // Should be 200 OK or at least not 404
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test] 
async fn test_health_route_works() {
    // This should PASS to prove our setup is correct
    let app = ruchy_notebook::server::create_app();
    
    let request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    
    let response = app
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}