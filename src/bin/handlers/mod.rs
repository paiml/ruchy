use anyhow::{Context, Result};
pub mod add;
pub mod bench_handler;
pub mod build;
pub mod check_handler;
mod commands;
pub mod coverage_handler;
pub mod doc_handler;
pub mod eval;
pub mod execution_handler;
mod handlers_modules;
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

// Other extracted handlers
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

// Re-export from extracted modules
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

// Testing re-exports
pub use fuzz_handler::handle_fuzz_command;
pub use mutations_handler::handle_mutations_command;
pub use property_tests_handler::handle_property_tests_command;

// Other re-exports
pub use actor_handler::handle_actor_observe_command;
pub use command_router::handle_complex_command;
pub use compile_handler::handle_compile_command;
pub use dataflow_handler::handle_dataflow_debug_command;
pub use optimize_handler::handle_optimize_command;
pub use oracle_handler::handle_oracle_command;
pub use publish_handler::handle_publish_command;
pub use replay_handler::handle_replay_to_tests_command;
pub use mcp_handler::handle_mcp_command;
pub use notebook_handler::handle_notebook_command;
pub use serve_handler::handle_serve_command;

// Import for internal use
use transpile_handler::parse_source;
use ruchy::runtime::Repl;
// RuchyParser, Transpiler, WasmEmitter moved to individual handler modules
// Replay functionality imports removed - not needed in handler, used directly in REPL
// PARSER-077: Add syn and prettyplease for proper TokenStream formatting
use std::fs;
use std::path::{Path, PathBuf};

// handle_eval_command moved to eval.rs
// handle_file_execution, handle_stdin_input moved to execution_handler.rs
// handle_parse_command moved to parse_handler.rs
// handle_transpile_command moved to transpile_handler.rs

// ============================================================================
// Common Helper Functions (Complexity ≤5, reused across handlers)
// ============================================================================

/// Check if a result should be printed (filters out Unit values)
/// Complexity: 2
fn should_print_result(result: &str) -> bool {
    result != "Unit" && result != "()"
}

/// Read file contents with detailed error context
/// Complexity: 2
fn read_file_with_context(file: &Path) -> Result<String> {
    fs::read_to_string(file).map_err(|e| {
        // Include the OS error message (e.g., "No such file or directory")
        anyhow::anyhow!("{}: {}", file.display(), e)
    })
}

/// Create a REPL instance with temp directory
/// Complexity: 1
fn create_repl() -> Result<Repl> {
    Repl::new(std::env::temp_dir())
}

/// Log command output if verbose mode is enabled
/// Complexity: 2
fn log_command_output(output: &std::process::Output, verbose: bool) {
    if verbose {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command output:\n{}", stderr);
    }
}

/// Write file with detailed error context
/// Complexity: 2
fn write_file_with_context(path: &Path, content: &[u8]) -> Result<()> {
    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

// ============================================================================

// handle_run_command and VmMode moved to run_handler.rs
/// Handle interactive theorem prover (RUCHY-0820) - delegated to refactored module
pub fn handle_prove_command(
    file: Option<&std::path::Path>,
    backend: &str,
    ml_suggestions: bool,
    timeout: u64,
    script: Option<&std::path::Path>,
    export: Option<&std::path::Path>,
    check: bool,
    counterexample: bool,
    verbose: bool,
    format: &str,
) -> anyhow::Result<()> {
    // Delegate to refactored module with ≤10 complexity
    handlers_modules::prove::handle_prove_command(
        file,
        backend,
        ml_suggestions,
        timeout,
        script,
        export,
        check,
        counterexample,
        verbose,
        format,
    )
}

// print_prover_help moved to repl_handler.rs
// handle_repl_command moved to repl_handler.rs
// handle_compile_command moved to compile_handler.rs

// handle_check_command moved to check_handler.rs
/// Handle test command - run tests with various options (delegated to refactored module)
///
/// # Arguments
/// * `path` - Optional path to test directory
/// * `watch` - Enable watch mode
/// * `verbose` - Enable verbose output
/// * `filter` - Optional test filter
/// * `coverage` - Enable coverage reporting
/// * `coverage_format` - Coverage report format
/// * `parallel` - Number of parallel test threads
/// * `threshold` - Coverage threshold
/// * `format` - Output format
///
/// # Examples
/// ```
/// // This function is typically called by the CLI test command
/// // handle_test_command(None, false, false, None, false, &"text".to_string(), 1, 80.0, &"text".to_string());
/// ```
///
/// # Errors
/// Returns error if tests fail to run or coverage threshold is not met
pub fn handle_test_command(
    path: Option<PathBuf>,
    watch: bool,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    // Delegate to refactored module with ≤10 complexity
    handlers_modules::test::handle_test_command(
        path,
        watch,
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}

// handle_coverage_command moved to coverage_handler.rs
// handle_bench_command moved to bench_handler.rs
// handle_doc_command moved to doc_handler.rs
// handle_dataflow_debug_command moved to dataflow_handler.rs
// handle_actor_observe_command moved to actor_handler.rs
// handle_optimize_command moved to optimize_handler.rs

/// Watch and run tests on changes - delegated to refactored module
fn handle_watch_and_test(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        true, // watch mode
        verbose,
        filter,
        false, // coverage
        "text",
        1,
        0.0,
        "text",
    )
}
/// Run enhanced tests - delegated to refactored module
#[allow(clippy::unnecessary_wraps)]
fn handle_run_enhanced_tests(
    path: &Path,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    handlers_modules::test::handle_test_command(
        Some(path.to_path_buf()),
        false, // not watch mode
        verbose,
        filter,
        coverage,
        coverage_format,
        parallel,
        threshold,
        format,
    )
}
/// Run a single .ruchy test file - delegated to `test_helpers` module
fn run_ruchy_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    handlers_modules::test_helpers::run_test_file(test_file, verbose)
}
/// Verify proofs extracted from AST - delegated to `prove_helpers` module
fn verify_proofs_from_ast(
    ast: &ruchy::frontend::ast::Expr,
    file_path: &std::path::Path,
    format: &str,
    counterexample: bool,
    verbose: bool,
) -> Result<()> {
    handlers_modules::prove_helpers::verify_proofs_from_ast(
        ast,
        file_path,
        format,
        counterexample,
        verbose,
    )
}

// handle_complex_command moved to command_router.rs



// handle_wasm_command moved to wasm_handler.rs

// handle_property_tests_command moved to property_tests_handler.rs
// handle_mutations_command moved to mutations_handler.rs
// handle_fuzz_command moved to fuzz_handler.rs
// handle_oracle_command moved to oracle_handler.rs
// handle_publish_command moved to publish_handler.rs
// handle_replay_to_tests_command moved to replay_handler.rs

#[cfg(test)]
mod tests;
