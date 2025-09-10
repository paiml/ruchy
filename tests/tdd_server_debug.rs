//! TDD Test to debug why server routes aren't being registered
//! RED: This test captures the exact bug we're seeing

#[tokio::test]
async fn test_server_actually_prints_route_registration() {
    // RED: This test will PROVE that the server code isn't being called
    
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tokio::time::timeout;
    
    // Start server and capture ALL output
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--features", "notebook", "--", "notebook", "--port", "9004"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server startup
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // Kill server and get output
    let _ = server.kill();
    let output = server.wait_with_output().expect("Failed to get output");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    println!("CAPTURED OUTPUT:");
    println!("STDOUT: {}", stdout);
    println!("STDERR: {}", stderr);
    
    // This assertion SHOULD pass if the server code is called
    assert!(combined.contains("📋 Registering API routes"), 
        "BUG DETECTED: Server never prints route registration message!");
    
    assert!(combined.contains("POST /api/execute - Code execution"), 
        "BUG DETECTED: Server doesn't register /api/execute endpoint!");
}

#[tokio::test] 
async fn test_api_endpoint_returns_404() {
    // RED: This test proves the API endpoint returns 404
    
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tokio::time::timeout;
    
    // Start server
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--features", "notebook", "--", "notebook", "--port", "9005"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for startup
    tokio::time::sleep(Duration::from_secs(4)).await;
    
    // Test API call
    let client = reqwest::Client::new();
    let response = timeout(Duration::from_secs(3), client
        .post("http://localhost:9005/api/execute")
        .json(&serde_json::json!({
            "code": "2 + 2",
            "cell_id": "test-cell",
            "session_id": "test-session"
        }))
        .send()
    ).await;
    
    // Kill server
    let _ = server.kill();
    
    match response {
        Ok(Ok(resp)) => {
            // This assertion will FAIL because we expect 404
            assert_eq!(resp.status(), 404, 
                "BUG CONFIRMED: API endpoint returns 404 - routes not registered!");
            
            println!("✅ BUG CONFIRMED: /api/execute returns 404 as expected");
        }
        Ok(Err(e)) => {
            panic!("Network error: {}", e);
        }
        Err(_) => {
            panic!("Request timed out");
        }
    }
}

#[test]
fn test_notebook_feature_flag_enabled() {
    // This test ensures the notebook feature is properly enabled
    
    // Check that the feature flag is enabled in the build
    #[cfg(feature = "notebook")]
    {
        println!("✅ Notebook feature is enabled");
        return;
    }
    
    #[cfg(not(feature = "notebook"))]
    panic!("❌ BUG: Notebook feature not enabled during test!");
}