//! TDD Test to verify create_app() doesn't panic
//! RED: This will test if create_app() is causing the issue

use ruchy_notebook::server::create_app;

#[test]
fn test_create_app_works() {
    // This test verifies create_app() executes without panic
    let app = create_app();
    
    // If this test fails, create_app() is the problem
    // If this test passes, the problem is elsewhere
    println!("✅ create_app() works correctly");
    
    // We can't easily inspect the router, but we can verify it exists
    // The fact that this doesn't panic means create_app() is fine
}

#[tokio::test]
async fn test_minimal_server_startup() {
    // Test the minimal server startup without full serving
    use std::net::SocketAddr;
    
    let app = create_app();
    let addr = SocketAddr::from(([127, 0, 0, 1], 9999));
    
    // Try to bind to a port - this is where issues often occur
    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            println!("✅ TCP binding works on port 9999");
            // Don't actually serve to avoid hanging
            drop(listener);
        }
        Err(e) => {
            panic!("TCP binding failed: {}", e);
        }
    }
    
    println!("✅ Minimal server startup components work");
}