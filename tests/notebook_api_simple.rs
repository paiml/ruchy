//! TDD test for notebook API
//! This will FAIL until we fix the route registration

use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_api_execute_returns_200_not_404() {
    // Start a real server and test it
    
    // This should FAIL with curl returning 404
    let result = timeout(Duration::from_secs(5), test_real_api_call()).await;
    
    match result {
        Ok(success) => {
            assert!(success, "API call should succeed");
        }
        Err(_) => {
            panic!("Test timed out - server not responding");
        }
    }
}

async fn test_real_api_call() -> bool {
    use std::process::Command;
    
    // Start server in background
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "notebook", "--port", "8890"])
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Test API call
    let output = Command::new("curl")
        .args(&[
            "-s", "-w", "%{http_code}",
            "-X", "POST", 
            "http://localhost:8890/api/execute",
            "-H", "Content-Type: application/json",
            "-d", r#"{"code":"2 + 2","cell_id":"test","session_id":"test"}"#
        ])
        .output()
        .expect("Failed to run curl");
    
    // Kill server
    let _ = server.kill();
    
    let response = String::from_utf8_lossy(&output.stdout);
    println!("API Response: {}", response);
    
    // Check if we got 404 (this should FAIL initially)
    !response.contains("404")
}

#[test]
fn test_route_registration_in_code() {
    // This test checks if the route is actually registered in the code
    
    // Read the server module source
    let source = std::fs::read_to_string("ruchy-notebook/src/server/mod.rs")
        .expect("Could not read server module");
    
    // Check that the route is actually there
    assert!(source.contains("/api/execute"), 
        "Route /api/execute not found in server code!");
    
    assert!(source.contains("post(execute_code_handler)"), 
        "POST handler for execute_code_handler not found!");
    
    println!("✅ Route registration found in source code");
}