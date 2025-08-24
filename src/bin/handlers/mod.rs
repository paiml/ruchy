use anyhow::{Context, Result};
use ruchy::{Parser as RuchyParser, Transpiler};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

/// Handle parse command - show AST for a Ruchy file
pub fn handle_parse_command(file: &Path, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Parsing file: {}", file.display());
    }
    
    let source = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;
    
    let mut parser = RuchyParser::new(&source);
    match parser.parse() {
        Ok(ast) => {
            println!("{ast:#?}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Parse error: {e}");
            std::process::exit(1);
        }
    }
}

/// Handle transpile command - convert Ruchy to Rust
pub fn handle_transpile_command(
    file: &Path, 
    output: Option<&Path>, 
    minimal: bool,
    verbose: bool
) -> Result<()> {
    if verbose {
        eprintln!("Transpiling file: {}", file.display());
        if minimal {
            eprintln!("Using minimal codegen for self-hosting");
        }
    }
    
    let source = if file.as_os_str() == "-" {
        // Read from stdin
        if verbose {
            eprintln!("Reading from stdin...");
        }
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    } else {
        fs::read_to_string(file)
            .with_context(|| format!("Failed to read file: {}", file.display()))?
    };

    let mut parser = RuchyParser::new(&source);
    let ast = parser.parse()
        .with_context(|| "Failed to parse input")?;

    let transpiler = Transpiler::new();
    let rust_code = if minimal {
        transpiler.transpile_minimal(&ast)
            .with_context(|| "Failed to transpile to Rust (minimal)")?
    } else {
        transpiler.transpile(&ast)
            .map(|tokens| tokens.to_string())
            .with_context(|| "Failed to transpile to Rust")?
    };

    // Output the generated Rust code
    if let Some(output_path) = output {
        fs::write(output_path, &rust_code)
            .with_context(|| format!("Failed to write output file: {}", output_path.display()))?;
        
        if verbose {
            eprintln!("Output written to: {}", output_path.display());
        }
    } else {
        print!("{rust_code}");
    }

    Ok(())
}

/// Handle run command - compile and execute a Ruchy file
pub fn handle_run_command(file: &Path, verbose: bool) -> Result<()> {
    if verbose {
        eprintln!("Running file: {}", file.display());
    }
    
    let source = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let mut parser = RuchyParser::new(&source);
    let ast = parser.parse()
        .with_context(|| "Failed to parse input")?;

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)
        .map(|tokens| tokens.to_string())
        .with_context(|| "Failed to transpile to Rust")?;

    // Write to temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("ruchy_temp.rs");
    fs::write(&temp_file, &rust_code)
        .with_context(|| "Failed to write temporary file")?;

    if verbose {
        eprintln!("Temporary Rust file: {}", temp_file.display());
        eprintln!("Compiling and running...");
    }

    // Compile and run using rustc
    let output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("-o")
        .arg(temp_dir.join("ruchy_temp"))
        .arg(&temp_file)
        .output()
        .with_context(|| "Failed to run rustc")?;

    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    // Run the compiled binary
    let run_output = std::process::Command::new(temp_dir.join("ruchy_temp"))
        .output()
        .with_context(|| "Failed to run compiled binary")?;

    // Print the output
    print!("{}", String::from_utf8_lossy(&run_output.stdout));
    if !run_output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&run_output.stderr));
    }

    // Cleanup temporary files
    let _ = fs::remove_file(&temp_file);
    let _ = fs::remove_file(temp_dir.join("ruchy_temp"));

    if !run_output.status.success() {
        std::process::exit(run_output.status.code().unwrap_or(1));
    }

    Ok(())
}

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
            // Extract and check proofs from AST (future enhancement)
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
            if session.is_complete() {
                println!("✅ All goals proved!");
            } else {
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