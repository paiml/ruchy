//! TDD Test: Prove /api/execute returns 404
//! This test will FAIL until we fix the route registration

#[test]
fn test_api_execute_route_exists_in_server_code() {
    // RED: This test checks if the route is actually registered
    
    let server_code = std::fs::read_to_string("ruchy-notebook/src/server/mod.rs")
        .expect("Could not read server file");
    
    // Verify route is in source code
    assert!(server_code.contains("api/execute"), 
        "Route /api/execute not found in server source!");
    
    // Verify handler is connected  
    assert!(server_code.contains("execute_code_handler"),
        "Handler execute_code_handler not found!");
    
    // This should PASS - route is in source code
    println!("✅ Route registration found in source code");
}

#[test] 
fn test_notebook_command_calls_correct_server() {
    // RED: This test will FAIL because CLI doesn't use ruchy-notebook server
    
    // The issue: `ruchy notebook` command doesn't use ruchy_notebook::server::start_server
    // It probably uses some other server implementation
    
    // This is our hypothesis to test
    let main_rs = std::fs::read_to_string("src/main.rs")
        .unwrap_or_else(|_| String::new());
    
    let cli_rs = std::fs::read_to_string("src/cli.rs") 
        .unwrap_or_else(|_| String::new());
    
    // Check if CLI imports ruchy_notebook::server
    let imports_notebook_server = main_rs.contains("ruchy_notebook::server") 
        || cli_rs.contains("ruchy_notebook::server");
    
    assert!(imports_notebook_server, 
        "FAILING TEST: CLI doesn't import ruchy_notebook::server - this is why we get 404!");
    
    println!("✅ CLI correctly imports ruchy-notebook server");
}