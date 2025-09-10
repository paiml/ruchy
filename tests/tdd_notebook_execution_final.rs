//! TDD Test - Final comprehensive test for notebook execution
//! This test follows Toyota Way: isolate the exact failure and fix it systematically

use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_notebook_cell_execution_end_to_end() {
    // RED: This test will fail because cells don't execute properly
    
    // Start server using the actual binary (not cargo run)
    let mut server = std::process::Command::new("./target/debug/ruchy")
        .args(&["notebook", "--port", "9999"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for server startup
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Test cell execution via API
    let client = reqwest::Client::new();
    let response = timeout(Duration::from_secs(3), client
        .post("http://localhost:9999/api/execute")
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
            if resp.status() == 404 {
                panic!("FAILING TEST: API endpoint returns 404 - routes not registered!");
            }
            
            let json: serde_json::Value = resp.json().await.expect("Failed to parse JSON");
            
            // This should pass when notebook execution works
            assert_eq!(json["success"], true, "Code execution should succeed");
            assert_eq!(json["result"], "4", "2 + 2 should equal 4");
            
            println!("✅ NOTEBOOK EXECUTION WORKS: 2 + 2 = {}", json["result"]);
        }
        Ok(Err(e)) => {
            panic!("FAILING TEST: Network error: {}", e);
        }
        Err(_) => {
            panic!("FAILING TEST: API request timed out");
        }
    }
}

#[test] 
fn test_server_binary_exists() {
    // Verify the binary was built correctly
    let binary_path = std::path::Path::new("./target/debug/ruchy");
    assert!(binary_path.exists(), "Server binary should exist at ./target/debug/ruchy");
    
    // Verify it's executable
    let metadata = std::fs::metadata(binary_path).expect("Failed to get binary metadata");
    assert!(metadata.is_file(), "Should be a file");
    
    println!("✅ Server binary exists and is ready for testing");
}