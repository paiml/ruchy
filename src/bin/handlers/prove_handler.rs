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
    use super::*;
    use std::io::Cursor;
    use tempfile::TempDir;

    // PMAT-104: `handle_prove_command` with `file: None` and `check: false`
    // enters the interactive prover, which reads the *process* stdin. Under
    // `cargo test` that never returns, so every such test blocked the whole
    // `--bin ruchy` target. These tests exercise the same handler through
    // `handle_prove_command_with_input`, which takes the reader explicitly, so
    // the real interactive path is still covered but terminates at EOF.
    use super::super::handlers_modules::prove::handle_prove_command_with_input;

    /// An interactive session that is immediately at end of input.
    fn eof_input() -> Cursor<Vec<u8>> {
        Cursor::new(Vec::new())
    }

    #[test]
    fn test_prove_handler_stub() {
        // Prove handler tests are in handlers_modules::prove
        // This is a placeholder for the delegation layer
    }

    // ===== EXTREME TDD Round 146 - Prove Handler Tests =====

    #[test]
    fn test_handle_prove_command_no_file() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_nonexistent_file() {
        let result = handle_prove_command(
            Some(Path::new("/nonexistent/file.ruchy")),
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            false,
            "text",
        );
        assert!(
            result.is_err(),
            "proving a file that does not exist must be an error"
        );
    }

    #[test]
    fn test_handle_prove_command_with_ml_suggestions() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            true, // ml_suggestions
            60,
            None,
            None,
            false,
            false,
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_check_mode() {
        let result = handle_prove_command(
            None, "default", false, 30, None, None, true, // check
            false, false, "text",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_counterexample() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            None,
            false,
            true, // counterexample
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_verbose() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            true, // verbose
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_json_format() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            false,
            "json",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_various_backends() {
        let backends = ["default", "z3", "smt", "custom"];
        for backend in &backends {
            let result = handle_prove_command_with_input(
                None,
                backend,
                false,
                30,
                None,
                None,
                false,
                false,
                false,
                "text",
                &mut eof_input(),
            );
            assert!(result.is_ok(), "backend {backend} must be accepted");
        }
    }

    #[test]
    fn test_handle_prove_command_various_timeouts() {
        let timeouts = [1, 10, 30, 60, 300, 3600];
        for timeout in &timeouts {
            let result = handle_prove_command_with_input(
                None,
                "default",
                false,
                *timeout,
                None,
                None,
                false,
                false,
                false,
                "text",
                &mut eof_input(),
            );
            assert!(result.is_ok(), "timeout {timeout} must be accepted");
        }
    }

    #[test]
    fn test_handle_prove_command_with_script() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("proof_script.txt");
        std::fs::write(&script_path, "intro\napply simplify").unwrap();
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            Some(&script_path),
            None,
            false,
            false,
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_with_export() {
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path().join("proof_export.txt");
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            Some(&export_path),
            false,
            false,
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
        assert!(
            export_path.exists(),
            "an export path must be written when the session ends"
        );
    }

    #[test]
    fn test_handle_prove_command_all_options() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ruchy");
        std::fs::write(&file_path, "42").unwrap();
        let result = handle_prove_command(
            Some(&file_path),
            "z3",
            true,
            120,
            None,
            None,
            true,
            true,
            true,
            "json",
        );
        assert!(result.is_ok());
    }

    // ===== EXTREME TDD Round 153 - Prove Handler Tests =====

    #[test]
    fn test_handle_prove_command_with_function() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("func.ruchy");
        std::fs::write(&file_path, "fun add(a, b) { a + b }").unwrap();
        let result = handle_prove_command(
            Some(&file_path),
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            false,
            "text",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_zero_timeout() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            0, // zero timeout
            None,
            None,
            false,
            false,
            false,
            "text",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_all_true_flags() {
        let result = handle_prove_command(
            None, "default", true, // ml_suggestions
            30, None, None, true, // check
            true, // counterexample
            true, // verbose
            "json",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_prove_command_xml_format() {
        let result = handle_prove_command_with_input(
            None,
            "default",
            false,
            30,
            None,
            None,
            false,
            false,
            false,
            "xml",
            &mut eof_input(),
        );
        assert!(result.is_ok());
    }

    /// The interactive session must also honour an explicit `quit` command
    /// coming from a non-tty reader, not just EOF.
    #[test]
    fn test_handle_prove_command_quit_from_reader() {
        let mut input = Cursor::new(b"help\nquit\n".to_vec());
        let result = handle_prove_command_with_input(
            None, "default", false, 30, None, None, false, false, false, "text", &mut input,
        );
        assert!(result.is_ok());
    }
}
