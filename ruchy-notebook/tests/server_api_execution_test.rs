//! TDD Test for Server API Execution
//! Tests that the /api/execute endpoint actually runs Ruchy code

use ruchy_notebook::server::*;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

#[tokio::test]
async fn test_api_execute_runs_ruchy_code() {
    // Create the app
    let app = create_app();
    
    // Test simple arithmetic
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"2 + 2","cell_id":"test-1","session_id":"test"}"#
        ))
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["result"].as_str().unwrap(), "4");
}

#[tokio::test]
async fn test_api_execute_handles_variables() {
    let app = create_app();
    
    // First request: define a variable
    let request1 = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"let x = 10","cell_id":"test-1","session_id":"session-1"}"#
        ))
        .unwrap();
    
    let response1 = app
        .clone()
        .oneshot(request1)
        .await
        .unwrap();
    
    assert_eq!(response1.status(), StatusCode::OK);
    
    // Second request: use the variable (same session)
    let request2 = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"x * 2","cell_id":"test-2","session_id":"session-1"}"#
        ))
        .unwrap();
    
    let response2 = app
        .clone()
        .oneshot(request2)
        .await
        .unwrap();
    
    assert_eq!(response2.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response2.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["result"].as_str().unwrap(), "20");
}

#[tokio::test]
async fn test_api_execute_handles_errors() {
    let app = create_app();
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"undefined_variable","cell_id":"test-1","session_id":"test"}"#
        ))
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(!json["success"].as_bool().unwrap());
    assert!(json["error"].as_str().unwrap().contains("undefined"));
}

#[tokio::test]
async fn test_api_execute_runs_println() {
    let app = create_app();
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/execute")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"code":"println(\"Hello, World!\")","cell_id":"test-1","session_id":"test"}"#
        ))
        .unwrap();
    
    let response = app
        .clone()
        .oneshot(request)
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["result"].as_str().unwrap(), "Hello, World!\n");
}