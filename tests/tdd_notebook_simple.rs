//! TDD: RED - Simple failing test for notebook execution
//! This test defines EXACTLY what we need: execute "2 + 2" and get "4"

#[tokio::test]
async fn test_notebook_executes_ruchy_code() {
    // RED: This will fail because no server implementation exists
    
    // Start server
    let server_handle = std::process::Command::new("./target/debug/ruchy")
        .args(&["notebook", "--port", "9200"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for startup
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    // Test execution
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:9200/api/execute")
        .json(&serde_json::json!({
            "code": "2 + 2",
            "cell_id": "test"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["success"], true);
    assert_eq!(json["result"], "4");
    
    // Kill server
    let _ = std::process::Command::new("pkill")
        .arg("-f")
        .arg("ruchy.*notebook.*9200")
        .output();
}