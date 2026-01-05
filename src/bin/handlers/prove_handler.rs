//! Prove Command Handler
//!
//! Handles interactive theorem proving and formal verification commands.

use anyhow::Result;
use std::path::Path;

/// Handle interactive theorem prover (RUCHY-0820)
///
/// Delegates to the refactored handlers_modules::prove module.
///
/// # Arguments
/// * `file` - Optional path to file to prove
/// * `backend` - Prover backend to use
/// * `ml_suggestions` - Enable ML-powered suggestions
/// * `timeout` - Proof timeout in seconds
/// * `script` - Optional proof script file
/// * `export` - Optional path to export proofs
/// * `check` - Check mode only
/// * `counterexample` - Generate counterexamples
/// * `verbose` - Enable verbose output
/// * `format` - Output format
///
/// # Errors
/// Returns error if proving fails
pub fn handle_prove_command(
    file: Option<&Path>,
    backend: &str,
    ml_suggestions: bool,
    timeout: u64,
    script: Option<&Path>,
    export: Option<&Path>,
    check: bool,
    counterexample: bool,
    verbose: bool,
    format: &str,
) -> Result<()> {
    // Delegate to refactored module with ≤10 complexity
    super::handlers_modules::prove::handle_prove_command(
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

/// Verify proofs extracted from AST
///
/// Delegates to the prove_helpers module for AST-based proof verification.
///
/// # Arguments
/// * `ast` - The AST expression to verify
/// * `file_path` - Path to the source file
/// * `format` - Output format
/// * `counterexample` - Generate counterexamples
/// * `verbose` - Enable verbose output
///
/// # Errors
/// Returns error if proof verification fails
pub fn verify_proofs_from_ast(
    ast: &ruchy::frontend::ast::Expr,
    file_path: &Path,
    format: &str,
    counterexample: bool,
    verbose: bool,
) -> Result<()> {
    super::handlers_modules::prove_helpers::verify_proofs_from_ast(
        ast,
        file_path,
        format,
        counterexample,
        verbose,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_prove_handler_stub() {
        // Prove handler tests are in handlers_modules::prove
        // This is a placeholder for the delegation layer
    }
}
