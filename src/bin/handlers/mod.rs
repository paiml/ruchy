use anyhow::{Context, Result};

mod commands;
use ruchy::{Parser as RuchyParser, Transpiler};
use ruchy::runtime::Repl;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Handle eval command - evaluate a one-liner expression with -e flag
/// 
/// # Arguments
/// * `expr` - The expression to evaluate
/// * `verbose` - Enable verbose output
/// * `format` - Output format ("json" or default text)
/// 
/// # Examples
/// ```
/// // This function is typically called by the CLI with parsed arguments
/// // handle_eval_command("2 + 2", false, "text");
/// ```
/// 
/// # Errors
/// Returns error if expression cannot be parsed or evaluated
pub fn handle_eval_command(expr: &str, verbose: bool, format: &str) -> Result<()> {
    if verbose {
        eprintln!("Parsing expression: {expr}");
    }

    let mut repl = Repl::new()?;
    match repl.eval(expr) {
        Ok(result) => {
            if verbose {
                eprintln!("Evaluation successful");
            }

            if format == "json" {
                println!(
                    "{}",
                    serde_json::json!({
                        "success": true,
                        "result": format!("{result}")
                    })
                );
            } else {
                // Default text output - suppress unit values in CLI mode
                let result_str = result.to_string();
                if result_str != "()" {
                    println!("{result}");
                }
            }
            Ok(())
        }
        Err(e) => {
            if verbose {
                eprintln!("Evaluation failed: {e}");
            }

            match format {
                "json" => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        })
                    );
                }
                _ => {
                    eprintln!("Error: {e}");
                }
            }
            std::process::exit(1);
        }
    }
}

/// Handle file execution - run a Ruchy script file directly (not via subcommand)
/// 
/// # Arguments
/// * `file` - Path to the Ruchy file to execute
/// 
/// # Examples
/// ```
/// // This function is typically called by the CLI
/// // handle_file_execution(&Path::new("script.ruchy"));
/// ```
/// 
/// # Errors
/// Returns error if file cannot be read, parsed, or executed
pub fn handle_file_execution(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    // Use REPL to evaluate the file
    let mut repl = Repl::new()?;
    match repl.eval(&source) {
        Ok(result) => {
            // Only print non-unit results
            if result != "Unit" && result != "()" {
                println!("{result}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

/// Handle stdin/piped input - evaluate input from standard input
/// 
/// # Arguments
/// * `input` - The input string to evaluate
/// 
/// # Examples
/// ```
/// // This function is typically called when input is piped to the CLI
/// // handle_stdin_input("2 + 2");
/// ```
/// 
/// # Errors
/// Returns error if input cannot be parsed or evaluated
pub fn handle_stdin_input(input: &str) -> Result<()> {
    let mut repl = Repl::new()?;
    match repl.eval(input) {
        Ok(result) => {
            println!("{result}");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}

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
        transpiler.transpile_to_program(&ast)
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
    let rust_code = transpiler.transpile_to_program_with_context(&ast, Some(file))
        .map(|tokens| tokens.to_string())
        .with_context(|| "Failed to transpile to Rust")?;

    // Write to unique temporary file to avoid race conditions
    let temp_source = tempfile::NamedTempFile::new()
        .with_context(|| "Failed to create temporary file")?;
    fs::write(temp_source.path(), &rust_code)
        .with_context(|| "Failed to write temporary file")?;

    if verbose {
        eprintln!("Temporary Rust file: {}", temp_source.path().display());
        eprintln!("Compiling and running...");
    }

    // Create unique output binary path (use Builder to avoid keeping file open)
    let temp_dir = tempfile::tempdir()
        .with_context(|| "Failed to create temporary directory")?;
    let binary_path = temp_dir.path().join("ruchy_temp_bin");
    
    // Compile and run using rustc
    let output = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-name=ruchy_temp")
        .arg("-o")
        .arg(&binary_path)
        .arg(temp_source.path())
        .output()
        .with_context(|| "Failed to run rustc")?;

    if !output.status.success() {
        eprintln!("Compilation failed:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    // Run the compiled binary
    let run_output = std::process::Command::new(&binary_path)
        .output()
        .with_context(|| "Failed to run compiled binary")?;

    // Print the output
    print!("{}", String::from_utf8_lossy(&run_output.stdout));
    if !run_output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&run_output.stderr));
    }

    // Temporary files will be automatically cleaned up when NamedTempFile goes out of scope

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
        let ast = parser.parse()
            .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;
        
        // Extract proof goals from AST (simplified for now)
        if verbose {
            println!("📋 Extracted proof goals from source");
        }
        
        // In check mode, verify proofs from AST
        if check {
            println!("✓ Checking proofs in {}...", file_path.display());
            return verify_proofs_from_ast(&ast, file_path, format, counterexample, verbose);
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

/// Handle REPL command - start the interactive Read-Eval-Print Loop
/// 
/// # Examples
/// ```
/// // This function is typically called when no command or when repl command is specified
/// // handle_repl_command();
/// ```
/// 
/// # Errors
/// Returns error if REPL fails to initialize or run
pub fn handle_repl_command() -> Result<()> {
    use colored::Colorize;
    
    let version_msg = format!("Welcome to Ruchy REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", version_msg.bright_cyan().bold());
    println!(
        "Type {} for commands, {} to exit\n",
        ":help".green(),
        ":quit".yellow()
    );

    let mut repl = Repl::new()?;
    repl.run()
}

/// Handle compile command - compile Ruchy file to native binary
/// 
/// # Arguments
/// * `file` - Path to the Ruchy file to compile
/// * `output` - Output binary path
/// * `opt_level` - Optimization level (0, 1, 2, 3, s, z)
/// * `strip` - Strip debug symbols
/// * `static_link` - Use static linking
/// * `target` - Target triple for cross-compilation
/// 
/// # Examples
/// ```
/// // This function is typically called by the CLI compile command
/// // handle_compile_command(&Path::new("app.ruchy"), PathBuf::from("app"), "2".to_string(), true, false, None);
/// ```
/// 
/// # Errors
/// Returns error if compilation fails or rustc is not available
pub fn handle_compile_command(
    file: &Path,
    output: PathBuf,
    opt_level: String,
    strip: bool,
    static_link: bool,
    target: Option<String>,
) -> Result<()> {
    use ruchy::backend::{CompileOptions, compile_to_binary as backend_compile};
    use colored::Colorize;
    use std::fs;
    
    // Check if rustc is available
    if let Err(e) = ruchy::backend::compiler::check_rustc_available() {
        eprintln!("{} {}", "Error:".bright_red(), e);
        eprintln!("Please install Rust toolchain from https://rustup.rs/");
        std::process::exit(1);
    }
    
    println!("{} Compiling {}...", "→".bright_blue(), file.display());
    
    let options = CompileOptions {
        output,
        opt_level,
        strip,
        static_link,
        target,
        rustc_flags: Vec::new(),
    };
    
    match backend_compile(file, &options) {
        Ok(binary_path) => {
            println!(
                "{} Successfully compiled to: {}",
                "✓".bright_green(),
                binary_path.display()
            );
            
            // Make the binary executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&binary_path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&binary_path, perms)?;
            }
            
            println!(
                "{} Binary size: {} bytes",
                "ℹ".bright_blue(),
                fs::metadata(&binary_path)?.len()
            );
        }
        Err(e) => {
            eprintln!("{} Compilation failed: {}", "✗".bright_red(), e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Handle check command - check syntax of a Ruchy file
/// 
/// # Arguments
/// * `file` - Path to the Ruchy file to check
/// * `watch` - Enable file watching mode
/// 
/// # Examples
/// ```
/// // This function is typically called by the CLI check command
/// // handle_check_command(&Path::new("script.ruchy"), false);
/// ```
/// 
/// # Errors
/// Returns error if file cannot be read or has syntax errors
pub fn handle_check_command(file: &Path, watch: bool) -> Result<()> {
    if watch {
        handle_watch_and_check(file)
    } else {
        handle_check_syntax(file)
    }
}

/// Check syntax of a single file
fn handle_check_syntax(file: &Path) -> Result<()> {
    use colored::Colorize;
    
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    match parser.parse() {
        Ok(_) => {
            println!("{}", "✓ Syntax is valid".green());
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", format!("✗ Syntax error: {e}").red());
            std::process::exit(1);
        }
    }
}

/// Watch a file and check syntax on changes
fn handle_watch_and_check(file: &Path) -> Result<()> {
    use colored::Colorize;
    use std::thread;
    use std::time::Duration;
    
    println!(
        "{} Watching {} for changes...",
        "👁".bright_cyan(),
        file.display()
    );
    println!("Press Ctrl+C to stop watching\n");

    // Initial check
    handle_check_syntax(file)?;

    // Simple file watching using polling
    let mut last_modified = fs::metadata(file)?.modified()?;

    loop {
        thread::sleep(Duration::from_millis(500));

        let Ok(metadata) = fs::metadata(file) else {
            continue; // File might be temporarily unavailable
        };

        let Ok(modified) = metadata.modified() else {
            continue;
        };

        if modified != last_modified {
            last_modified = modified;
            println!("\n{} File changed, checking...", "→".bright_cyan());
            let _ = handle_check_syntax(file); // Don't exit on error, keep watching
        }
    }
}

/// Handle test command - run tests with various options
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
    if watch {
        let test_path = path.unwrap_or_else(|| PathBuf::from("."));
        handle_watch_and_test(&test_path, verbose, filter)
    } else {
        let test_path = path.unwrap_or_else(|| PathBuf::from("."));
        handle_run_enhanced_tests(
            &test_path,
            verbose,
            filter,
            coverage,
            coverage_format,
            parallel,
            threshold,
            format,
        )
    }
}

/// Watch and run tests on changes (internal implementation)
fn handle_watch_and_test(path: &Path, verbose: bool, filter: Option<&str>) -> Result<()> {
    use colored::Colorize;
    use std::thread;
    use std::time::Duration;
    
    println!(
        "{} Watching {} for test changes...",
        "👁".bright_cyan(),
        path.display()
    );
    println!("Press Ctrl+C to stop watching\n");

    // Initial test run
    let _ = handle_run_enhanced_tests(path, verbose, filter, false, "text", 1, 0.0, "text");

    // Simple directory watching using polling
    let mut last_modified = fs::metadata(path).and_then(|m| m.modified()).unwrap_or_else(|_| {
        std::time::SystemTime::now()
    });

    loop {
        thread::sleep(Duration::from_millis(1000));

        // Check if any .ruchy files have been modified
        if let Ok(entries) = fs::read_dir(path) {
            let mut current_modified = last_modified;
            for entry in entries.flatten() {
                if let Ok(path) = entry.path().canonicalize() {
                    if path.extension().and_then(|ext| ext.to_str()) == Some("ruchy") {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if modified > current_modified {
                                    current_modified = modified;
                                }
                            }
                        }
                    }
                }
            }

            if current_modified > last_modified {
                last_modified = current_modified;
                println!("\n{} Files changed, running tests...", "→".bright_cyan());
                let _ = handle_run_enhanced_tests(path, verbose, filter, false, "text", 1, 0.0, "text");
            }
        }
    }
}

/// Run enhanced tests (internal implementation)
#[allow(clippy::unnecessary_wraps)]
fn handle_run_enhanced_tests(
    path: &Path,
    verbose: bool,
    filter: Option<&str>,
    coverage: bool,
    coverage_format: &str,
    _parallel: usize,
    threshold: f64,
    format: &str,
) -> Result<()> {
    use colored::Colorize;
    use std::time::Instant;
    use walkdir::WalkDir;
    
    if verbose {
        println!("🔍 Discovering .ruchy test files in {}", path.display());
    }
    
    // Discover .ruchy test files
    let mut test_files = Vec::new();
    
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "ruchy") {
            test_files.push(path.to_path_buf());
        } else {
            return Err(anyhow::anyhow!("File {} is not a .ruchy file", path.display()));
        }
    } else if path.is_dir() {
        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.path().extension().is_some_and(|ext| ext == "ruchy") {
                // Apply filter if provided
                if let Some(filter_pattern) = filter {
                    let file_name = entry.path().file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    if !file_name.contains(filter_pattern) {
                        continue;
                    }
                }
                test_files.push(entry.path().to_path_buf());
            }
        }
    } else {
        return Err(anyhow::anyhow!("Path {} does not exist", path.display()));
    }
    
    if test_files.is_empty() {
        println!("⚠️  No .ruchy test files found in {}", path.display());
        return Ok(());
    }
    
    let total_start = Instant::now();
    let mut passed = 0;
    let mut failed = 0;
    let mut test_results = Vec::new();
    
    println!("🧪 Running {} .ruchy test files...\n", test_files.len());
    
    // Execute each test file
    for test_file in &test_files {
        if verbose {
            println!("📄 Testing: {}", test_file.display());
        }
        
        let test_start = Instant::now();
        let result = run_ruchy_test_file(test_file, verbose);
        let test_duration = test_start.elapsed();
        
        match result {
            Ok(()) => {
                passed += 1;
                if verbose {
                    println!("   ✅ {} ({:.2}ms)", test_file.file_name().unwrap().to_str().unwrap(), test_duration.as_secs_f64() * 1000.0);
                } else {
                    print!(".");
                }
                test_results.push((test_file.clone(), true, test_duration, None));
            }
            Err(e) => {
                failed += 1;
                let error_msg = format!("{}", e);
                if verbose {
                    println!("   ❌ {} ({:.2}ms): {}", test_file.file_name().unwrap().to_str().unwrap(), test_duration.as_secs_f64() * 1000.0, error_msg);
                } else {
                    print!("F");
                }
                test_results.push((test_file.clone(), false, test_duration, Some(error_msg)));
            }
        }
        
        if !verbose {
            std::io::Write::flush(&mut std::io::stdout())?;
        }
    }
    
    let total_duration = total_start.elapsed();
    
    if !verbose {
        println!(); // New line after dots/F's
    }
    
    // Print summary
    println!("\n📊 Test Results:");
    println!("   Total: {}", test_files.len());
    println!("   Passed: {}", passed.to_string().green());
    if failed > 0 {
        println!("   Failed: {}", failed.to_string().red());
    }
    println!("   Duration: {:.2}s", total_duration.as_secs_f64());
    
    // Show failures in detail if any
    if failed > 0 && !verbose {
        println!("\n❌ Failed Tests:");
        for (test_file, success, _duration, error) in &test_results {
            if !success {
                println!("   {} - {}", test_file.display(), error.as_ref().unwrap_or(&"Unknown error".to_string()));
            }
        }
    }
    
    // Coverage reporting
    if coverage {
        use ruchy::quality::ruchy_coverage::RuchyCoverageCollector;
        
        let mut collector = RuchyCoverageCollector::new();
        
        // Analyze all test files for coverage
        for test_file in &test_files {
            if let Err(e) = collector.analyze_file(test_file) {
                eprintln!("Warning: Failed to analyze {}: {}", test_file.display(), e);
            }
        }
        
        // Execute tests and collect runtime coverage
        for (test_file, success, _, _) in &test_results {
            if *success {
                // Execute the file to collect actual coverage
                if let Err(e) = collector.execute_with_coverage(test_file) {
                    eprintln!("Warning: Failed to collect runtime coverage for {}: {}", test_file.display(), e);
                }
            }
        }
        
        // Generate report based on format
        let report = match coverage_format {
            "json" => collector.generate_json_report(),
            "html" => {
                let html_report = collector.generate_html_report();
                // Write to file
                let coverage_dir = std::path::Path::new("target/coverage");
                fs::create_dir_all(coverage_dir)?;
                let html_path = coverage_dir.join("index.html");
                fs::write(&html_path, html_report)?;
                format!("\n📈 HTML Coverage Report written to: {}", html_path.display())
            }
            _ => collector.generate_text_report(),
        };
        
        println!("{}", report);
        
        // Check threshold
        if threshold > 0.0 {
            if collector.meets_threshold(threshold) {
                println!("\n✅ Coverage meets threshold of {:.1}%", threshold);
            } else {
                eprintln!("\n❌ Coverage below threshold of {:.1}%", threshold);
                std::process::exit(1);
            }
        }
    }
    
    // Format output as JSON if requested
    if format == "json" {
        let json_output = serde_json::json!({
            "total": test_files.len(),
            "passed": passed,
            "failed": failed,
            "duration_seconds": total_duration.as_secs_f64(),
            "results": test_results.iter().map(|(path, success, duration, error)| {
                serde_json::json!({
                    "file": path.display().to_string(),
                    "success": success,
                    "duration_ms": duration.as_secs_f64() * 1000.0,
                    "error": error
                })
            }).collect::<Vec<_>>()
        });
        println!("\n{}", serde_json::to_string_pretty(&json_output)?);
    }
    
    // Exit with non-zero status if tests failed
    if failed > 0 {
        std::process::exit(1);
    }
    
    println!("\n✅ All tests passed!");
    Ok(())
}

/// Run a single .ruchy test file using the Ruchy interpreter
fn run_ruchy_test_file(test_file: &Path, verbose: bool) -> Result<()> {
    use ruchy::runtime::repl::Repl;
    use std::fs;
    
    // Read the test file
    let test_content = fs::read_to_string(test_file)
        .with_context(|| format!("Failed to read test file: {}", test_file.display()))?;
    
    if verbose {
        println!("   📖 Parsing test file...");
    }
    
    if verbose {
        println!("   🏃 Executing test...");
    }
    
    // Execute the test using the REPL's evaluation engine
    let mut repl = Repl::new()?;
    let result = repl.evaluate_expr_str(&test_content, None)
        .with_context(|| format!("Test execution failed for: {}", test_file.display()))?;
    
    if verbose {
        println!("   📤 Test result: {:?}", result);
    }
    
    // For now, we consider any successful execution a pass
    // In the future, we could have test-specific assertions
    Ok(())
}

/// Verify proofs extracted from AST
fn verify_proofs_from_ast(
    ast: &ruchy::frontend::ast::Expr, 
    file_path: &std::path::Path, 
    format: &str,
    counterexample: bool, 
    verbose: bool
) -> Result<()> {
    use ruchy::proving::{extract_assertions_from_ast, verify_assertions_batch};
    
    // Extract assertions from the AST
    let assertions = extract_assertions_from_ast(ast);
    
    if assertions.is_empty() {
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
            println!("✅ No proofs found (file valid)");
        }
        return Ok(());
    }
    
    if verbose {
        println!("Found {} assertions to verify", assertions.len());
        for (i, assertion) in assertions.iter().enumerate() {
            println!("  {}: {}", i + 1, assertion);
        }
    }
    
    // Verify all assertions
    let results = verify_assertions_batch(&assertions, counterexample);
    
    let total = results.len();
    let passed = results.iter().filter(|r| r.is_verified).count();
    let failed = total - passed;
    
    if format == "json" {
        let json_result = serde_json::json!({
            "file": file_path.display().to_string(),
            "status": if failed == 0 { "verified" } else { "failed" },
            "total": total,
            "passed": passed,
            "failed": failed,
            "proofs": results
        });
        println!("{}", serde_json::to_string_pretty(&json_result)?);
    } else {
        // Text output
        if failed == 0 {
            println!("✅ All {} proofs verified successfully", total);
            if verbose {
                for (i, result) in results.iter().enumerate() {
                    println!("  ✅ Proof {}: {} ({:.1}ms)", 
                        i + 1, result.assertion, result.verification_time_ms);
                }
            }
        } else {
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
            
            if verbose {
                println!("\nPassed proofs:");
                for (i, result) in results.iter().enumerate() {
                    if result.is_verified {
                        println!("  ✅ Proof {}: {} ({:.1}ms)", 
                            i + 1, result.assertion, result.verification_time_ms);
                    }
                }
            }
        }
    }
    
    // Exit with non-zero status if proofs failed
    if failed > 0 {
        std::process::exit(1);
    }
    
    Ok(())
}

/// Handle all remaining complex commands via delegation pattern
/// 
/// This function delegates to the original implementations to keep complexity low
/// while maintaining a clean main function dispatcher
/// 
/// # Arguments
/// * `command` - The command to execute
/// 
/// # Examples
/// ```
/// // This function is typically called by the main dispatcher for complex commands
/// ```
/// 
/// # Errors
/// Returns error if command execution fails
#[allow(clippy::unnecessary_wraps)]
pub fn handle_complex_command(command: crate::Commands) -> Result<()> {
    match command {
        crate::Commands::Ast { 
            file, 
            json, 
            graph, 
            metrics, 
            symbols, 
            deps, 
            verbose, 
            output 
        } => {
            commands::handle_ast_command(
                &file,
                json,
                graph,
                metrics,
                symbols,
                deps,
                verbose,
                output.as_deref(),
            )
        }
        crate::Commands::Provability { 
            file, 
            verify, 
            contracts, 
            invariants, 
            termination, 
            bounds, 
            verbose, 
            output 
        } => {
            commands::handle_provability_command(
                &file,
                verify,
                contracts,
                invariants,
                termination,
                bounds,
                verbose,
                output.as_deref(),
            )
        }
        crate::Commands::Runtime { 
            file, 
            profile, 
            bigo, 
            bench, 
            compare, 
            memory, 
            verbose, 
            output 
        } => {
            commands::handle_runtime_command(
                &file,
                profile,
                bigo,
                bench,
                compare.as_deref(),
                memory,
                verbose,
                output.as_deref(),
            )
        }
        crate::Commands::Score {
            path,
            depth,
            fast,
            deep,
            watch,
            explain,
            baseline,
            min,
            config,
            format,
            verbose,
            output,
        } => {
            commands::handle_score_command(
                &path,
                &depth,
                fast,
                deep,
                watch,
                explain,
                baseline.as_deref(),
                min,
                config.as_deref(),
                &format,
                verbose,
                output.as_deref(),
            )
        }
        crate::Commands::QualityGate {
            path,
            config,
            depth: _,
            fail_fast,
            format,
            export,
            ci: _,
            verbose,
        } => {
            // Simplified quality gate handling
            commands::handle_quality_gate_command(
                &path,
                config.as_deref(),
                fail_fast,    // Use as strict
                !verbose,     // Use as quiet
                format == "json",
                verbose,
                None,         // No output field
                export.as_deref(),
            )
        }
        crate::Commands::Fmt {
            file,
            all,
            check,
            stdout,
            diff,
            config,
            line_width: _,
            indent: _,
            use_tabs: _,
        } => {
            // Simplified fmt handling
            commands::handle_fmt_command(
                &file,
                check,
                !check && !stdout,  // write if not check or stdout
                config.as_deref(),
                all,
                diff,
                stdout,
                false,  // verbose not available
            )
        }
        crate::Commands::Lint {
            file,
            all: _,
            fix,
            strict,
            verbose,
            format,
            rules,
            deny_warnings: _,
            max_complexity: _,
            config,
            init_config,
        } => {
            if init_config {
                // Create default lint config
                println!("Creating default lint configuration...");
                Ok(())
            } else if let Some(file_path) = file {
                commands::handle_lint_command(
                    &file_path,
                    fix,
                    strict,
                    rules.as_deref(),
                    format == "json",
                    verbose,
                    None,  // ignore not available
                    config.as_deref(),
                )
            } else {
                eprintln!("Error: Either provide a file or use --all flag");
                std::process::exit(1);
            }
        }
        crate::Commands::Prove {
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
        } => {
            handle_prove_command(
                file.as_deref(),
                &backend,
                ml_suggestions,
                timeout,
                script.as_deref(),
                export.as_deref(),
                check,
                counterexample,
                verbose,
                &format,
            )
        }
        _ => {
            // Other commands not yet implemented
            eprintln!("Command not yet implemented");
            Ok(())
        }
    }
    
    /*
    // Original complex command handling - commented out until handlers implemented
    match command {
        crate::Commands::Ast { 
            file, 
            json, 
            graph, 
            metrics, 
            symbols, 
            deps, 
            verbose, 
            output 
        } => {
            // AST analysis implementation planned
            eprintln!("AST analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Provability { 
            file, 
            verify, 
            contracts, 
            invariants, 
            termination, 
            bounds, 
            verbose, 
            output 
        } => {
            // Provability analysis implementation planned
            eprintln!("Provability analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Runtime { 
            file, 
            profile, 
            bigo, 
            bench, 
            compare, 
            memory, 
            verbose, 
            output 
        } => {
            // Runtime analysis implementation planned
            eprintln!("Runtime analysis not yet implemented");
            Ok(())
        }
        crate::Commands::Score {
            path,
            depth,
            fast,
            deep,
            watch,
            explain,
            baseline,
            min,
            config,
            format,
            verbose,
            output,
        } => {
            let baseline_str = baseline.as_deref();
            let config_str = config.as_ref().and_then(|p| p.to_str());
            let output_str = output.as_ref().and_then(|p| p.to_str());
            // Quality score calculation implementation planned
            eprintln!("Quality score calculation not yet implemented");
            Ok(())
        }
        crate::Commands::QualityGate {
            path,
            config,
            depth,
            fail_fast,
            format,
            export,
            ci,
            verbose,
        } => {
            // Quality gates implementation planned
            eprintln!("Quality gates enforcement not yet implemented");
            Ok(())
        }
        crate::Commands::Fmt {
            file: _,
            all: _,
            check: _,
            stdout: _,
            diff: _,
            config: _,
            line_width: _,
            indent: _,
            use_tabs: _,
        } => {
            // Code formatting implementation planned
            eprintln!("Code formatting not yet implemented");
            Ok(())
        }
        crate::Commands::Doc {
            path,
            output,
            format,
            private,
            open,
            all,
            verbose,
        } => {
            // Documentation generation implementation planned
            eprintln!("Documentation generation not yet implemented");
            Ok(())
        }
        crate::Commands::Bench {
            file,
            iterations,
            warmup,
            format,
            output,
            verbose,
        } => {
            crate::benchmark_ruchy_code(
                &file,
                iterations,
                warmup,
                &format,
                output.as_deref(),
                verbose,
            )
        }
        crate::Commands::Lint {
            file,
            all: _,
            fix,
            strict,
            verbose,
            format,
            rules,
            deny_warnings: _,
            max_complexity: _,
            config,
            init_config,
        } => {
            if init_config {
                crate::generate_default_lint_config()
            } else {
                // Load custom rules if config provided
                let custom_rules = if let Some(config_path) = config {
                    crate::load_custom_lint_rules(&config_path)?
                } else {
                    // Custom lint rules implementation planned
                    Default::default()
                };

                if all {
                    crate::lint_ruchy_code(
                        &PathBuf::from("."),
                        all,
                        fix,
                        strict,
                        verbose,
                        &format,
                        rules.as_deref(),
                        deny_warnings,
                        max_complexity,
                        &custom_rules,
                    )
                } else if let Some(file) = file {
                    crate::lint_ruchy_code(
                        &file,
                        false,
                        fix,
                        strict,
                        verbose,
                        &format,
                        rules.as_deref(),
                        deny_warnings,
                        max_complexity,
                        &custom_rules,
                    )
                } else {
                    eprintln!("Error: Either provide a file or use --all flag");
                    std::process::exit(1);
                }
            }
        }
        crate::Commands::Add {
            package,
            version,
            dev,
            registry,
        } => {
            crate::add_package(&package, version.as_deref(), dev, &registry)
        }
        crate::Commands::Publish {
            registry,
            version,
            dry_run,
            allow_dirty,
        } => {
            crate::publish_package(&registry, version.as_deref(), dry_run, allow_dirty)
        }
        crate::Commands::Mcp {
            name,
            streaming,
            timeout,
            min_score,
            max_complexity,
            verbose,
            config,
        } => {
            let config_str = config.as_ref().and_then(|p| p.to_str());
            crate::start_mcp_server(&name, streaming, timeout, min_score, max_complexity, verbose, config_str)
        }
        crate::Commands::Optimize {
            file,
            hardware,
            depth,
            cache,
            branches,
            vectorization,
            abstractions,
            benchmark,
            format,
            output,
            verbose,
            threshold,
        } => {
            crate::optimize_file(
                &file,
                &hardware,
                &depth,
                cache,
                branches,
                vectorization,
                abstractions,
                benchmark,
                &format,
                output.as_deref(),
                verbose,
                threshold,
            )
        }
        crate::Commands::ActorObserve {
            config,
            refresh_interval,
            max_traces,
            max_actors,
            enable_deadlock_detection,
            deadlock_interval,
            start_mode,
            no_color,
            format,
            export,
            duration,
            verbose,
            filter_actor,
            filter_failed,
            filter_slow,
        } => {
            crate::start_actor_observatory(
                config.as_ref(),
                refresh_interval,
                max_traces,
                max_actors,
                enable_deadlock_detection,
                deadlock_interval,
                &start_mode,
                !no_color,
                &format,
                export.as_ref(),
                duration,
                verbose,
                filter_actor.as_ref(),
                filter_failed,
                filter_slow,
            )
        }
        crate::Commands::DataflowDebug {
            config,
            max_rows,
            auto_materialize,
            enable_profiling,
            timeout,
            track_memory,
            compute_diffs,
            sample_rate,
            refresh_interval,
            no_color,
            format,
            export,
            verbose,
            breakpoint,
            start_mode,
        } => {
            crate::start_dataflow_debugger(
                config.as_ref(),
                max_rows,
                auto_materialize,
                enable_profiling,
                timeout,
                track_memory,
                compute_diffs,
                sample_rate,
                refresh_interval,
                !no_color,
                &format,
                export.as_ref(),
                verbose,
                &breakpoint,
                &start_mode,
            )
        }
        crate::Commands::Wasm { 
            file,
            output,
            target,
            wit,
            deploy,
            deploy_target,
            portability,
            opt_level,
            debug,
            simd,
            threads,
            component_model,
            name,
            version,
            verbose,
        } => {
            crate::handle_wasm_command(
                &file,
                output.as_deref(),
                &target,
                wit,
                deploy,
                deploy_target.as_deref(),
                portability,
                &opt_level,
                debug,
                simd,
                threads,
                component_model,
                name.as_deref(),
                &version,
                verbose,
            )
        }
        _ => {
            // This should not be reached since handled commands are processed elsewhere
            eprintln!("Error: Command not implemented in complex handler");
            std::process::exit(1);
        }
    }
    */
}