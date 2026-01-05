//! REPL Command Handler
//!
//! Handles the interactive Read-Eval-Print Loop.

use anyhow::Result;
use std::path::PathBuf;

/// Handle REPL command - start the interactive Read-Eval-Print Loop
///
/// # Arguments
/// * `record_file` - Optional path to record REPL session
///
/// # Errors
/// Returns error if REPL fails to initialize or run
pub fn handle_repl_command(record_file: Option<PathBuf>) -> Result<()> {
    use colored::Colorize;
    let version_msg = format!("Welcome to Ruchy REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", version_msg.bright_cyan().bold());
    println!(
        "Type {} for commands, {} to exit\n",
        ":help".green(),
        ":quit".yellow()
    );
    let mut repl = super::create_repl()?;
    if let Some(record_path) = record_file {
        repl.run_with_recording(&record_path)
    } else {
        repl.run()
    }
}

/// Print prover help - moved to separate function for clarity
pub(crate) fn print_prover_help() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_prover_help_does_not_panic() {
        // Just verify it doesn't panic
        print_prover_help();
    }
}
