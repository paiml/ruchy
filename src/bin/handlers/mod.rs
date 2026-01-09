//! CLI Command Handlers
//!
//! This module contains all command handlers for the Ruchy CLI.
//! Each handler is in its own submodule for modularity and testability.
//!
//! **EXTREME TDD**: This mod.rs contains ZERO implementation code.
//! Only module declarations and re-exports.

// ============================================================================
// Module Declarations
// ============================================================================

// Core command handlers
pub mod add;
pub mod bench_handler;
pub mod build;
pub mod check_handler;
pub mod coverage_handler;
pub mod doc_handler;
pub mod eval;
pub mod execution_handler;
pub mod new;
pub mod parse_handler;
pub mod repl_handler;
pub mod run_handler;
pub mod transpile_handler;
pub mod wasm_handler;

// Testing handlers
pub mod fuzz_handler;
pub mod mutations_handler;
pub mod property_tests_handler;

// Server and tool handlers
pub mod actor_handler;
pub mod command_router;
pub mod compile_handler;
pub mod dataflow_handler;
pub mod mcp_handler;
pub mod notebook_handler;
pub mod optimize_handler;
pub mod oracle_handler;
pub mod publish_handler;
pub mod replay_handler;
pub mod serve_handler;

// Delegation handlers (thin wrappers to handlers_modules)
pub mod prove_handler;
pub mod test_handler;

// Utility modules
pub mod helpers;

// Internal modules (not re-exported)
mod commands;
mod handlers_modules;

// ============================================================================
// Re-exports - Public API
// ============================================================================

// Core command handlers
pub use bench_handler::handle_bench_command;
pub use check_handler::handle_check_command;
pub use coverage_handler::handle_coverage_command;
pub use doc_handler::handle_doc_command;
pub use eval::handle_eval_command;
pub use execution_handler::{handle_file_execution, handle_stdin_input};
pub use parse_handler::handle_parse_command;
pub use repl_handler::handle_repl_command;
pub use run_handler::{
    compile_rust_code, handle_run_command, prepare_compilation, transpile_for_execution, VmMode,
};
pub use transpile_handler::handle_transpile_command;
pub use wasm_handler::{compile_ruchy_to_wasm, handle_wasm_command};

// Testing command handlers
pub use fuzz_handler::handle_fuzz_command;
pub use mutations_handler::handle_mutations_command;
pub use property_tests_handler::handle_property_tests_command;

// Server and tool handlers
pub use actor_handler::handle_actor_observe_command;
pub use command_router::handle_complex_command;
pub use compile_handler::handle_compile_command;
pub use dataflow_handler::handle_dataflow_debug_command;
pub use mcp_handler::handle_mcp_command;
pub use notebook_handler::handle_notebook_command;
pub use optimize_handler::handle_optimize_command;
pub use oracle_handler::handle_oracle_command;
pub use publish_handler::handle_publish_command;
pub use replay_handler::handle_replay_to_tests_command;
pub use serve_handler::handle_serve_command;

// Delegation handlers
pub use prove_handler::handle_prove_command;
pub use test_handler::handle_test_command;

// Helper utilities (for use by other handlers)
pub use helpers::{
    create_repl, log_command_output, read_file_with_context,
    write_file_with_context,
};

// Internal re-exports (used by extracted handlers)
pub(crate) use transpile_handler::parse_source;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests;
