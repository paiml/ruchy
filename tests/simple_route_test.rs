//! Simple TDD test for route registration

#[test]
fn test_can_import_server_module() {
    // This test proves we can access the server module
    
    #[cfg(feature = "native")]
    {
        // Try to reference the server module
        let _ = ruchy_notebook::server::create_app;
        println!("✅ Server module accessible");
    }
    
    #[cfg(not(feature = "native"))]
    {
        panic!("❌ Native feature not enabled");
    }
}