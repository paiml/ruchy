//! Refactored prove command handler
//! Complexity reduced from 390 to ≤10 per function
use anyhow::Result;
use super::prove_helpers::{parse_smt_backend, configure_prover, load_proof_script, load_proof_file, verify_proofs_from_ast, handle_prover_command, show_prover_state, export_proof};
use ruchy::proving::{InteractiveProver, ProverSession};
use std::io::{self, Write};
/// Handle interactive theorem prover - refactored with ≤10 complexity
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
) -> Result<()> {
    if verbose {
        println!("🔍 Starting interactive prover with backend: {}", backend);
    }
    // Parse backend and create prover
    let smt_backend = parse_smt_backend(backend, verbose);
    let mut prover = InteractiveProver::new(smt_backend);
    // Configure prover settings
    configure_prover(&mut prover, timeout, ml_suggestions, verbose);
    // Handle file-based proof checking
    if let Some(file_path) = file {
        return handle_file_proving(file_path, format, counterexample, verbose);
    }
    // Load script if provided
    if let Some(script_path) = script {
        load_proof_script(&mut prover, script_path, verbose)?;
    }
    // Run interactive session if not in check mode
    if !check {
        run_interactive_session(&mut prover, ml_suggestions, export, format, verbose)?;
    }
    Ok(())
}
/// Handle file-based proof checking
fn handle_file_proving(
    file_path: &std::path::Path,
    format: &str,
    counterexample: bool,
    verbose: bool,
) -> Result<()> {
    let ast = load_proof_file(file_path, verbose)?;
    println!("✓ Checking proofs in {}...", file_path.display());
    verify_proofs_from_ast(&ast, file_path, format, counterexample, verbose)
}
/// Run interactive prover session
fn run_interactive_session(
    prover: &mut InteractiveProver,
    ml_suggestions: bool,
    export: Option<&std::path::Path>,
    format: &str,
    verbose: bool,
) -> Result<()> {
    println!("🚀 Starting Ruchy Interactive Prover");
    println!("Type 'help' for available commands\n");
    let mut session = ProverSession::new();
    // Main interactive loop
    loop {
        prompt_user()?;
        let input = read_user_input()?;
        if input.is_empty() {
            continue;
        }
        // Process command
        let should_exit = handle_prover_command(&input, prover, &mut session, verbose)?;
        if should_exit {
            break;
        }
        // Show current state
        show_prover_state(&session, prover, ml_suggestions);
    }
    // Export proof if requested
    if let Some(export_path) = export {
        export_proof(&session, export_path, format, verbose)?;
    }
    Ok(())
}
/// Display prompt to user
fn prompt_user() -> Result<()> {
    print!("prove> ");
    io::stdout().flush()?;
    Ok(())
}
/// Read input from user
fn read_user_input() -> Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}