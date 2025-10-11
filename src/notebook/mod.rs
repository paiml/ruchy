pub mod dataframe;
pub mod engine;
pub mod execution;
pub mod html;
pub mod persistence;
pub mod server;
#[cfg(feature = "notebook")]
pub mod testing;
pub mod wasm;

pub use dataframe::{ColumnType, DataFrame};
pub use engine::NotebookEngine;
pub use execution::CellExecutionResult;
pub use html::{html_escape, HtmlFormatter};
pub use persistence::{Checkpoint, TransactionResult};
pub use server::start_server;
pub use wasm::{NotebookPerformance, NotebookWasm};

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
