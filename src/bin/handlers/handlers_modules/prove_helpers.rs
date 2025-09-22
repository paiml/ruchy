//! Helper functions for prove command
//! Extracted to maintain ≤10 complexity per function
use anyhow::{Context, Result};
use ruchy::proving::{InteractiveProver, ProverSession, SmtBackend};
use ruchy::utils::{add_success_indicator, read_file_with_context, write_file_with_context};
/// Parse SMT backend from string
pub fn parse_smt_backend(backend: &str, verbose: bool) -> SmtBackend {
    match backend.to_lowercase().as_str() {
        "z3" => SmtBackend::Z3,
        "cvc5" => SmtBackend::CVC5,
        "yices2" => SmtBackend::Yices2,
        _ => {
            if verbose {
                eprintln!("Warning: Unknown backend '{}', defaulting to Z3", backend);
            }
            SmtBackend::Z3
        }
    }
}
/// Configure prover with settings
pub fn configure_prover(
    prover: &mut InteractiveProver,
    timeout: u64,
    ml_suggestions: bool,
    verbose: bool,
) {
    prover.set_timeout(timeout);
    prover.set_ml_suggestions(ml_suggestions);
    if verbose {
        println!("⚙️  Configuration:");
        println!("  Timeout: {}ms", timeout);
        println!("  ML Suggestions: {}", ml_suggestions);
    }
}
/// Load and parse file for proof checking
pub fn load_proof_file(
    file_path: &std::path::Path,
    verbose: bool,
) -> Result<ruchy::frontend::ast::Expr> {
    use ruchy::Parser as RuchyParser;
    if verbose {
        println!("📂 Loading file: {}", file_path.display());
    }
    let source = read_file_with_context(file_path)?;
    let mut parser = RuchyParser::new(&source);
    let ast = parser
        .parse()
        .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;
    if verbose {
        println!("📋 Extracted proof goals from source");
    }
    Ok(ast)
}
/// Load proof script from file
pub fn load_proof_script(
    prover: &mut InteractiveProver,
    script_path: &std::path::Path,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("📜 Loading proof script: {}", script_path.display());
    }
    let script_content = read_file_with_context(script_path)?;
    prover.load_script(&script_content)?;
    Ok(())
}
/// Print interactive prover help
pub fn print_prover_help() {
    println!("\nInteractive Prover Commands:");
    println!("  help          - Show this help message");
    println!("  quit/exit     - Exit the prover");
    println!("  goals         - Show current proof goals");
    println!("  tactics       - List available tactics");
    println!("  goal <stmt>   - Add a new proof goal");
    println!("  apply <tactic> - Apply a tactic to current goal");
    println!("\nTactics:");
    println!("  intro         - Introduce hypothesis from implication");
    println!("  split         - Split conjunction into subgoals");
    println!("  induction     - Proof by induction");
    println!("  contradiction - Proof by contradiction");
    println!("  reflexivity   - Prove equality by reflexivity");
    println!("  simplify      - Simplify expression");
    println!("  assumption    - Prove using an assumption");
    println!("\nExamples:");
    println!("  goal x > 0 -> x + 1 > 1");
    println!("  apply intro");
    println!("  apply simplify\n");
}
/// Handle single prover command
pub fn handle_prover_command(
    input: &str,
    prover: &mut InteractiveProver,
    session: &mut ProverSession,
    verbose: bool,
) -> Result<bool> {
    match input {
        "quit" | "exit" => {
            println!("Goodbye!");
            Ok(true) // Signal to exit
        }
        "help" => {
            print_prover_help();
            Ok(false)
        }
        "goals" => {
            print_current_goals(session);
            Ok(false)
        }
        "tactics" => {
            print_available_tactics(prover);
            Ok(false)
        }
        cmd if cmd.starts_with("apply ") => {
            apply_tactic(cmd, prover, session)?;
            Ok(false)
        }
        cmd if cmd.starts_with("goal ") => {
            add_goal(cmd, session);
            Ok(false)
        }
        _ => {
            process_general_input(input, prover, session, verbose)?;
            Ok(false)
        }
    }
}
/// Print current proof goals
fn print_current_goals(session: &ProverSession) {
    let goals = session.get_goals();
    if goals.is_empty() {
        println!("No active goals");
    } else {
        for (i, goal) in goals.iter().enumerate() {
            println!("Goal {}: {}", i + 1, goal.statement);
        }
    }
}
/// Print available tactics
fn print_available_tactics(prover: &InteractiveProver) {
    let tactics = prover.get_available_tactics();
    println!("Available tactics:");
    for tactic in tactics {
        println!("  {} - {}", tactic.name(), tactic.description());
    }
}
/// Apply a tactic to current goal
fn apply_tactic(
    cmd: &str,
    prover: &mut InteractiveProver,
    session: &mut ProverSession,
) -> Result<()> {
    let tactic_name = &cmd[6..];
    match prover.apply_tactic(session, tactic_name, &[]) {
        Ok(result) => {
            println!("Result: {:?}", result);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
/// Add a new goal to session
fn add_goal(cmd: &str, session: &mut ProverSession) {
    let goal_stmt = &cmd[5..];
    session.add_goal(goal_stmt.to_string());
    println!("Added goal: {}", goal_stmt);
}
/// Process general input
fn process_general_input(
    input: &str,
    prover: &mut InteractiveProver,
    session: &mut ProverSession,
    verbose: bool,
) -> Result<()> {
    match prover.process_input(session, input) {
        Ok(result) => {
            if verbose {
                println!("Processed: {:?}", result);
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
/// Show current prover state with ML suggestions
pub fn show_prover_state(
    session: &ProverSession,
    prover: &mut InteractiveProver,
    ml_suggestions: bool,
) {
    if session.is_complete() {
        print!("{}", add_success_indicator("All goals proved!"));
    } else if let Some(current_goal) = session.current_goal() {
        println!("\nCurrent goal: {}", current_goal.statement);
        if ml_suggestions {
            show_ml_suggestions(prover, current_goal);
        }
    }
}
/// Show ML-powered tactic suggestions
fn show_ml_suggestions(prover: &mut InteractiveProver, goal: &ruchy::proving::ProofGoal) {
    if let Ok(suggestions) = prover.suggest_tactics(goal) {
        if !suggestions.is_empty() {
            println!("\nSuggested tactics:");
            for (i, sugg) in suggestions.iter().take(3).enumerate() {
                println!(
                    "  {}. {} (confidence: {:.2})",
                    i + 1,
                    sugg.tactic_name,
                    sugg.confidence
                );
            }
        }
    }
}
/// Export proof to file
pub fn export_proof(
    session: &ProverSession,
    export_path: &std::path::Path,
    format: &str,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("📝 Exporting proof to: {}", export_path.display());
    }
    let proof_content = match format {
        "json" => serde_json::to_string_pretty(session)?,
        "coq" => session.to_coq_proof(),
        "lean" => session.to_lean_proof(),
        _ => session.to_text_proof(),
    };
    write_file_with_context(export_path, &proof_content)?;
    print!("{}", add_success_indicator("Proof exported successfully"));
    Ok(())
}
/// Verify proofs extracted from AST
pub fn verify_proofs_from_ast(
    ast: &ruchy::frontend::ast::Expr,
    file_path: &std::path::Path,
    format: &str,
    counterexample: bool,
    verbose: bool,
) -> Result<()> {
    use ruchy::proving::{extract_assertions_from_ast, verify_assertions_batch};
    let assertions = extract_assertions_from_ast(ast);
    if assertions.is_empty() {
        handle_no_assertions(file_path, format, verbose)?;
        return Ok(());
    }
    if verbose {
        print_assertions(&assertions);
    }
    let results = verify_assertions_batch(&assertions, counterexample);
    output_verification_results(&results, file_path, format, verbose)?;
    check_verification_failures(&results)?;
    Ok(())
}
/// Handle case when no assertions found
fn handle_no_assertions(file_path: &std::path::Path, format: &str, verbose: bool) -> Result<()> {
    if verbose {
        println!("No assertions found in {}", file_path.display());
    }
    if format == "json" {
        let json_result = serde_json::json!({
            "file": file_path.display().to_string(),
            "status": "no_proofs",
            "total": 0,
            "passed": 0,
            "failed": 0,
            "proofs": []
        });
        println!("{}", serde_json::to_string_pretty(&json_result)?);
    } else {
        print!("{}", add_success_indicator("No proofs found (file valid)"));
    }
    Ok(())
}
/// Print discovered assertions
fn print_assertions(assertions: &[String]) {
    println!("Found {} assertions to verify", assertions.len());
    for (i, assertion) in assertions.iter().enumerate() {
        println!("  {}: {}", i + 1, assertion);
    }
}
/// Output verification results in requested format
fn output_verification_results(
    results: &[ruchy::proving::ProofVerificationResult],
    file_path: &std::path::Path,
    format: &str,
    verbose: bool,
) -> Result<()> {
    let total = results.len();
    let passed = results.iter().filter(|r| r.is_verified).count();
    let failed = total - passed;
    if format == "json" {
        output_json_results(results, file_path, total, passed, failed)?;
    } else {
        output_text_results(results, total, passed, failed, verbose);
    }
    Ok(())
}
/// Output results in JSON format
fn output_json_results(
    results: &[ruchy::proving::ProofVerificationResult],
    file_path: &std::path::Path,
    total: usize,
    passed: usize,
    failed: usize,
) -> Result<()> {
    let json_result = serde_json::json!({
        "file": file_path.display().to_string(),
        "status": if failed == 0 { "verified" } else { "failed" },
        "total": total,
        "passed": passed,
        "failed": failed,
        "proofs": results
    });
    println!("{}", serde_json::to_string_pretty(&json_result)?);
    Ok(())
}
/// Output results in text format
fn output_text_results(
    results: &[ruchy::proving::ProofVerificationResult],
    total: usize,
    _passed: usize,
    failed: usize,
    verbose: bool,
) {
    if failed == 0 {
        println!("✅ All {} proofs verified successfully", total);
        if verbose {
            for (i, result) in results.iter().enumerate() {
                println!(
                    "  ✅ Proof {}: {} ({:.1}ms)",
                    i + 1,
                    result.assertion,
                    result.verification_time_ms
                );
            }
        }
    } else {
        print_failed_proofs(results, total, failed);
        if verbose {
            print_passed_proofs(results);
        }
    }
}
/// Print failed proofs
fn print_failed_proofs(
    results: &[ruchy::proving::ProofVerificationResult],
    total: usize,
    failed: usize,
) {
    println!("❌ {} of {} proofs failed verification", failed, total);
    for (i, result) in results.iter().enumerate() {
        if !result.is_verified {
            println!("  ❌ Proof {}: {}", i + 1, result.assertion);
            if let Some(ref counterex) = result.counterexample {
                println!("     Counterexample: {}", counterex);
            }
            if let Some(ref error) = result.error {
                println!("     Error: {}", error);
            }
        }
    }
}
/// Print passed proofs
fn print_passed_proofs(results: &[ruchy::proving::ProofVerificationResult]) {
    println!("\nPassed proofs:");
    for (i, result) in results.iter().enumerate() {
        if result.is_verified {
            println!(
                "  ✅ Proof {}: {} ({:.1}ms)",
                i + 1,
                result.assertion,
                result.verification_time_ms
            );
        }
    }
}
/// Check if any verifications failed and return error if necessary
fn check_verification_failures(results: &[ruchy::proving::ProofVerificationResult]) -> Result<()> {
    let failed = results.iter().filter(|r| !r.is_verified).count();
    if failed > 0 {
        Err(anyhow::anyhow!(
            "Verification failures detected: {} proofs failed",
            failed
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_smt_backend_z3() {
        let backend = parse_smt_backend("z3", false);
        assert_eq!(backend, SmtBackend::Z3);
    }

    #[test]
    fn test_parse_smt_backend_cvc5() {
        let backend = parse_smt_backend("cvc5", false);
        assert_eq!(backend, SmtBackend::CVC5);
    }

    #[test]
    fn test_parse_smt_backend_yices2() {
        let backend = parse_smt_backend("yices2", false);
        assert_eq!(backend, SmtBackend::Yices2);
    }

    #[test]
    fn test_parse_smt_backend_case_insensitive() {
        let backend = parse_smt_backend("Z3", false);
        assert_eq!(backend, SmtBackend::Z3);

        let backend = parse_smt_backend("CVC5", false);
        assert_eq!(backend, SmtBackend::CVC5);

        let backend = parse_smt_backend("YICES2", false);
        assert_eq!(backend, SmtBackend::Yices2);
    }

    #[test]
    fn test_parse_smt_backend_unknown_defaults_to_z3() {
        let backend = parse_smt_backend("unknown", false);
        assert_eq!(backend, SmtBackend::Z3);
    }

    #[test]
    fn test_parse_smt_backend_empty_defaults_to_z3() {
        let backend = parse_smt_backend("", false);
        assert_eq!(backend, SmtBackend::Z3);
    }

    #[test]
    fn test_parse_smt_backend_verbose_mode() {
        let backend = parse_smt_backend("invalid", true);
        assert_eq!(backend, SmtBackend::Z3);
    }

    #[test]
    fn test_configure_prover_basic() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        configure_prover(&mut prover, 5000, false, false);
    }

    #[test]
    fn test_configure_prover_with_ml_suggestions() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        configure_prover(&mut prover, 10000, true, false);
    }

    #[test]
    fn test_configure_prover_verbose() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        configure_prover(&mut prover, 3000, true, true);
    }

    #[test]
    fn test_load_proof_file_valid() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = load_proof_file(temp_file.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_proof_file_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.ruchy");
        let result = load_proof_file(&path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_proof_file_verbose() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let result = load_proof_file(temp_file.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_proof_file_invalid_syntax() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "invalid syntax }}}}").unwrap();

        let result = load_proof_file(temp_file.path(), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_proof_script_valid() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "goal x > 0 -> x + 1 > 1").unwrap();

        let result = load_proof_script(&mut prover, temp_file.path(), false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_proof_script_nonexistent() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let path = PathBuf::from("/nonexistent/script.prove");

        let result = load_proof_script(&mut prover, &path, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_proof_script_verbose() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "goal x > 0").unwrap();

        let result = load_proof_script(&mut prover, temp_file.path(), true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_prover_help() {
        // This function only prints, no return value to test
        print_prover_help();
    }

    #[test]
    fn test_handle_prover_command_quit() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("quit", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true to exit
    }

    #[test]
    fn test_handle_prover_command_exit() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("exit", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should return true to exit
    }

    #[test]
    fn test_handle_prover_command_help() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("help", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false to continue
    }

    #[test]
    fn test_handle_prover_command_goals() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("goals", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false to continue
    }

    #[test]
    fn test_handle_prover_command_tactics() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("tactics", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false to continue
    }

    #[test]
    fn test_handle_prover_command_apply_tactic() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();
        session.add_goal("x > 0".to_string());

        let result = handle_prover_command("apply intro", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false to continue
    }

    #[test]
    fn test_handle_prover_command_add_goal() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        let result = handle_prover_command("goal x > 0", &mut prover, &mut session, false);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false to continue
    }

    #[test]
    fn test_handle_prover_command_general_input() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        // General input will try to process as a tactic, which may fail
        let result = handle_prover_command("unknown_input", &mut prover, &mut session, false);
        // The handle_prover_command should handle errors and still return Ok
        // but the error is propagated, so this will be an Err
        assert!(result.is_err()); // Unknown tactic will error
    }

    #[test]
    fn test_handle_prover_command_verbose() {
        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        let mut session = ProverSession::new();

        // In verbose mode, unknown input still errors
        let result = handle_prover_command("unknown_input", &mut prover, &mut session, true);
        assert!(result.is_err()); // Unknown tactic will error
    }

    #[test]
    fn test_show_prover_state_complete() {
        let mut session = ProverSession::new();
        session.add_goal("x > 0".to_string());
        // Session is ready for testing

        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        show_prover_state(&session, &mut prover, false);
    }

    #[test]
    fn test_show_prover_state_with_current_goal() {
        let mut session = ProverSession::new();
        session.add_goal("x > 0".to_string());

        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        show_prover_state(&session, &mut prover, false);
    }

    #[test]
    fn test_show_prover_state_with_ml_suggestions() {
        let mut session = ProverSession::new();
        session.add_goal("x > 0".to_string());

        let mut prover = InteractiveProver::new(SmtBackend::Z3);
        show_prover_state(&session, &mut prover, true);
    }

    #[test]
    fn test_export_proof_text_format() {
        let session = ProverSession::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = export_proof(&session, temp_file.path(), "text", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_proof_json_format() {
        let session = ProverSession::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = export_proof(&session, temp_file.path(), "json", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_proof_coq_format() {
        let session = ProverSession::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = export_proof(&session, temp_file.path(), "coq", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_proof_lean_format() {
        let session = ProverSession::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = export_proof(&session, temp_file.path(), "lean", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_proof_verbose() {
        let session = ProverSession::new();
        let temp_file = NamedTempFile::new().unwrap();

        let result = export_proof(&session, temp_file.path(), "text", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proofs_from_ast() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let ast = load_proof_file(temp_file.path(), false).unwrap();
        let result = verify_proofs_from_ast(&ast, temp_file.path(), "text", false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proofs_from_ast_json_format() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let ast = load_proof_file(temp_file.path(), false).unwrap();
        let result = verify_proofs_from_ast(&ast, temp_file.path(), "json", false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proofs_from_ast_with_counterexample() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let ast = load_proof_file(temp_file.path(), false).unwrap();
        let result = verify_proofs_from_ast(&ast, temp_file.path(), "text", true, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_proofs_from_ast_verbose() {
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(&temp_file, "let x = 42").unwrap();

        let ast = load_proof_file(temp_file.path(), false).unwrap();
        let result = verify_proofs_from_ast(&ast, temp_file.path(), "text", false, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_no_assertions_text_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = handle_no_assertions(temp_file.path(), "text", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_no_assertions_json_format() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = handle_no_assertions(temp_file.path(), "json", false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_no_assertions_verbose() {
        let temp_file = NamedTempFile::new().unwrap();
        let result = handle_no_assertions(temp_file.path(), "text", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_assertions_empty() {
        let assertions: Vec<String> = vec![];
        print_assertions(&assertions);
    }

    #[test]
    fn test_print_assertions_non_empty() {
        let assertions = vec!["x > 0".to_string(), "x + 1 > 1".to_string()];
        print_assertions(&assertions);
    }
}
