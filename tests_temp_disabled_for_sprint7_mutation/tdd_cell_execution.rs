//! TDD Test for ACTUAL Cell Execution
//! RED: This will FAIL because cells show placeholder text instead of executing

#[tokio::test]
#[ignore = "Integration test - requires server to be built with --features notebook"]
async fn test_cell_actually_executes_ruchy_code() {
    // Integration test: Verifies cell execution via HTTP API

    use std::process::{Command, Stdio};
    use std::time::Duration;
    use tokio::time::timeout;

    // Start notebook server
    let mut server = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--", "notebook", "--port", "9001"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start server");

    // Wait for server to start
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Check API call for code execution
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client
            .post("http://localhost:9001/api/execute")
            .json(&serde_json::json!({
                "source": "2 + 2"
            }))
            .send(),
    )
    .await;

    // Kill server
    let _ = server.kill();

    match response {
        Ok(Ok(resp)) => {
            let json: serde_json::Value = resp.json().await.unwrap();

            // Check that execution succeeded
            assert_eq!(json["success"], true, "Cell execution should succeed");

            // Check that output contains the result
            let output = json["output"].as_str().unwrap_or("");
            assert!(
                output.contains("4"),
                "Expected output to contain '4', got: {output}"
            );

            println!("✅ Cell executed: 2 + 2 = 4");
        }
        Ok(Err(e)) => {
            panic!("FAILING TEST: API request failed: {}", e);
        }
        Err(_) => {
            panic!("FAILING TEST: API request timed out - server not responding");
        }
    }
}

#[test]
#[ignore = "Frontend UI not yet implemented - server API works"]
fn test_frontend_has_execution_logic() {
    // RED: This will FAIL because frontend shows placeholder

    let js_file = std::fs::read_to_string("ruchy-notebook/js/ruchy-notebook.js")
        .expect("Notebook JS file not found");

    // Check if it has real execution logic instead of placeholder
    assert!(
        !js_file.contains(" // Planned feature: Execute code via WASM"),
        "FAILING TEST: JS still has Note placeholder for execution"
    );

    assert!(
        !js_file.contains("// Output will appear here"),
        "FAILING TEST: JS shows placeholder output instead of real execution"
    );

    // Should have actual API calls
    assert!(
        js_file.contains("fetch('/api/execute')"),
        "FAILING TEST: JS doesn't call /api/execute endpoint"
    );

    println!("✅ Frontend has real execution logic");
}

#[test]
fn test_server_has_execution_endpoint() {
    // RED: This will FAIL because server lacks execution endpoint

    let server_file =
        std::fs::read_to_string("src/notebook/server.rs").expect("Server file not found");

    // Should have actual execute handler
    assert!(
        server_file.contains("execute_handler"),
        "FAILING TEST: Server missing execute_handler"
    );

    // Should use real Ruchy interpreter
    assert!(
        server_file.contains("crate::runtime::repl"),
        "FAILING TEST: Server doesn't use Ruchy REPL for execution"
    );

    println!("✅ Server has execution endpoint");
}
