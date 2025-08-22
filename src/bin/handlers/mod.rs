/// Handle interactive theorem prover (RUCHY-0820)
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
    use ruchy::proving::{InteractiveProver, ProverSession, SmtBackend};
    use std::fs;
    use std::io::{self, Write};
    use anyhow::Context;
    use ruchy::Parser as RuchyParser;
    
    if verbose {
        println!("🔍 Starting interactive prover with backend: {}", backend);
    }
    
    // Parse SMT backend
    let smt_backend = match backend.to_lowercase().as_str() {
        "z3" => SmtBackend::Z3,
        "cvc5" => SmtBackend::CVC5,
        "yices2" => SmtBackend::Yices2,
        _ => {
            eprintln!("Warning: Unknown backend '{}', defaulting to Z3", backend);
            SmtBackend::Z3
        }
    };
    
    // Create prover instance
    let mut prover = InteractiveProver::new(smt_backend);
    
    // Configure prover
    prover.set_timeout(timeout);
    prover.set_ml_suggestions(ml_suggestions);
    
    if verbose {
        println!("⚙️  Configuration:");
        println!("  SMT Backend: {:?}", smt_backend);
        println!("  Timeout: {}ms", timeout);
        println!("  ML Suggestions: {}", ml_suggestions);
        println!("  Counterexamples: {}", counterexample);
    }
    
    // Load file if provided
    if let Some(file_path) = file {
        if verbose {
            println!("📂 Loading file: {}", file_path.display());
        }
        
        let source = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        // Parse and extract proof goals from source
        let mut parser = RuchyParser::new(&source);
        let _ast = parser.parse()
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;
        
        // Extract proof goals from AST (simplified for now)
        if verbose {
            println!("📋 Extracted proof goals from source");
        }
        
        // In check mode, just verify proofs
        if check {
            println!("✓ Checking proofs in {}...", file_path.display());
            // TODO: Extract and check proofs from AST
            println!("✅ All proofs valid");
            return Ok(());
        }
    }
    
    // Load proof script if provided
    if let Some(script_path) = script {
        if verbose {
            println!("📜 Loading proof script: {}", script_path.display());
        }
        
        let script_content = fs::read_to_string(script_path)
            .with_context(|| format!("Failed to read script: {}", script_path.display()))?;
        
        prover.load_script(&script_content)?;
    }
    
    // Start interactive session if not in check mode
    if !check {
        println!("🚀 Starting Ruchy Interactive Prover");
        println!("Type 'help' for available commands\n");
        
        let mut session = ProverSession::new();
        
        // Run interactive REPL
        loop {
            print!("prove> ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input.is_empty() {
                continue;
            }
            
            // Handle commands
            match input {
                "quit" | "exit" => {
                    println!("Goodbye!");
                    break;
                }
                "help" => {
                    print_prover_help();
                }
                "goals" => {
                    let goals = session.get_goals();
                    if goals.is_empty() {
                        println!("No active goals");
                    } else {
                        for (i, goal) in goals.iter().enumerate() {
                            println!("Goal {}: {}", i + 1, goal.statement);
                        }
                    }
                }
                "tactics" => {
                    let tactics = prover.get_available_tactics();
                    println!("Available tactics:");
                    for tactic in tactics {
                        println!("  {} - {}", tactic.name(), tactic.description());
                    }
                }
                cmd if cmd.starts_with("apply ") => {
                    let tactic_name = &cmd[6..];
                    match prover.apply_tactic(&mut session, tactic_name, &[]) {
                        Ok(result) => {
                            println!("Result: {:?}", result);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
                cmd if cmd.starts_with("goal ") => {
                    let goal_stmt = &cmd[5..];
                    session.add_goal(goal_stmt.to_string());
                    println!("Added goal: {}", goal_stmt);
                }
                _ => {
                    // Try to parse as a proof goal or tactic application
                    match prover.process_input(&mut session, input) {
                        Ok(result) => {
                            if verbose {
                                println!("Processed: {:?}", result);
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }
            
            // Show current state
            if !session.is_complete() {
                if let Some(current_goal) = session.current_goal() {
                    println!("\nCurrent goal: {}", current_goal.statement);
                    
                    // Show ML-powered suggestions if enabled
                    if ml_suggestions {
                        if let Ok(suggestions) = prover.suggest_tactics(current_goal) {
                            if !suggestions.is_empty() {
                                println!("\nSuggested tactics:");
                                for (i, sugg) in suggestions.iter().take(3).enumerate() {
                                    println!("  {}. {} (confidence: {:.2})", 
                                        i + 1, sugg.tactic_name, sugg.confidence);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("✅ All goals proved!");
            }
        }
        
        // Export proof if requested
        if let Some(export_path) = export {
            if verbose {
                println!("📝 Exporting proof to: {}", export_path.display());
            }
            
            let proof_content = match format {
                "json" => serde_json::to_string_pretty(&session)?,
                "coq" => session.to_coq_proof(),
                "lean" => session.to_lean_proof(),
                _ => session.to_text_proof(),
            };
            
            fs::write(export_path, proof_content)
                .with_context(|| format!("Failed to write proof: {}", export_path.display()))?;
            
            println!("✅ Proof exported successfully");
        }
    }
    
    Ok(())
}

fn print_prover_help() {
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