//! TDD Test to prove route registration issue
//! RED: This will fail because CLI doesn't call ruchy-notebook server

#[test]
fn test_notebook_server_prints_route_registration() {
    // RED: This test will FAIL because the server doesn't print the expected message
    
    use std::process::{Command, Stdio};
    use std::time::Duration;
    use std::thread;
    
    // Start server and capture output
    let mut cmd = Command::new("cargo")
        .args(&["run", "--bin", "ruchy", "--features", "notebook", "--", "notebook", "--port", "9003"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");
    
    // Wait for startup
    thread::sleep(Duration::from_secs(3));
    
    // Kill and get output
    cmd.kill().expect("Failed to kill server");
    let output = cmd.wait_with_output().expect("Failed to get output");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    
    println!("Server output: {}", combined);
    
    // This assertion will FAIL because CLI doesn't use ruchy-notebook::server::start_server
    assert!(combined.contains("📋 Registering API routes"), 
        "FAILING TEST: Server doesn't print route registration message");
    
    assert!(combined.contains("POST /api/execute - Code execution"), 
        "FAILING TEST: Server doesn't register /api/execute endpoint");
}

#[test]
fn test_cli_calls_correct_server_function() {
    // RED: This test proves the CLI calls the wrong server
    
    let handlers_file = std::fs::read_to_string("src/bin/handlers/mod.rs")
        .expect("Handlers file not found");
    
    // The CLI should call ruchy_notebook::server::start_server
    assert!(handlers_file.contains("ruchy_notebook::server::start_server"), 
        "CLI should call ruchy_notebook::server::start_server");
    
    // But it might be behind a feature flag that's not enabled
    // Or it might be calling a different server implementation
    
    // Let's check if it's properly feature-gated
    let has_proper_feature = handlers_file.contains("#[cfg(feature = \"notebook\")]") &&
                            handlers_file.contains("ruchy_notebook::server::start_server");
    
    assert!(has_proper_feature, 
        "FAILING TEST: Notebook server not properly feature-gated");
    
    println!("✅ CLI has correct server call with feature gate");
}