pub mod server;
#[cfg(feature = "notebook")]
pub mod testing;
pub use server::start_server;

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 11: Notebook module tests

    #[test]
    fn test_module_exports() {
        // Verify that start_server is exported
        // This is a compile-time test - if it compiles, the export exists
        let _ = start_server;
    }

    #[test]
    fn test_feature_gated_testing() {
        // The testing module is feature-gated
        #[cfg(feature = "notebook")]
        {
            // If notebook feature is enabled, testing module should be available
            // This is a compile-time test
        }
    }

    #[test]
    fn test_server_module_exists() {
        // Verify server module exists - this is a compile-time test
        // If it compiles, the module exists
    }
}