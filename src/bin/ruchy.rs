#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler};
use std::fs;
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ruchy")]
#[command(author, version, about = "The Ruchy programming language", long_about = None)]
struct Cli {
    /// Evaluate a one-liner expression
    #[arg(short = 'e', long = "eval", value_name = "EXPR")]
    eval: Option<String>,

    /// Output format for evaluation results (text, json)
    #[arg(long, default_value = "text")]
    format: String,

    /// Enable verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Script file to execute (alternative to subcommands)
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the interactive REPL
    Repl,

    /// Parse a Ruchy file and show the AST
    Parse {
        /// The file to parse
        file: PathBuf,
    },

    /// Transpile a Ruchy file to Rust
    Transpile {
        /// The file to transpile
        file: PathBuf,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Compile and run a Ruchy file
    Run {
        /// The file to run
        file: PathBuf,
    },

    /// Check syntax without running
    Check {
        /// The file to check
        file: PathBuf,
    },

    /// Show AST for a file
    Ast {
        /// The file to parse
        file: PathBuf,
    },

    /// Format Ruchy source code
    Fmt {
        /// The file to format
        file: PathBuf,

        /// Format all files in project
        #[arg(long)]
        all: bool,

        /// Check if files are formatted without modifying them
        #[arg(long)]
        check: bool,

        /// Write formatted output to stdout instead of modifying files
        #[arg(long)]
        stdout: bool,

        /// Show diff of changes
        #[arg(long)]
        diff: bool,
    },

    /// Lint Ruchy source code for issues and style violations
    Lint {
        /// The file to lint
        file: PathBuf,

        /// Lint all files in project
        #[arg(long)]
        all: bool,

        /// Show additional context for violations
        #[arg(long)]
        verbose: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,

        /// Fail on warnings as well as errors
        #[arg(long)]
        deny_warnings: bool,

        /// Maximum allowed complexity for functions
        #[arg(long, default_value = "10")]
        max_complexity: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle one-liner evaluation with -e flag
    if let Some(expr) = cli.eval {
        if cli.verbose {
            eprintln!("Parsing expression: {expr}");
        }

        let mut repl = Repl::new()?;
        match repl.eval(&expr) {
            Ok(result) => {
                if cli.verbose {
                    eprintln!("Evaluation successful");
                }

                if cli.format == "json" {
                    // Output as JSON
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": true,
                            "result": format!("{result}")
                        })
                    );
                } else {
                    // Default text output
                    println!("{result}");
                }
                return Ok(());
            }
            Err(e) => {
                if cli.verbose {
                    eprintln!("Evaluation failed: {e}");
                }

                if cli.format == "json" {
                    println!(
                        "{}",
                        serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        })
                    );
                } else {
                    eprintln!("Error: {e}");
                }
                std::process::exit(1);
            }
        }
    }

    // Handle script file execution (without subcommand)
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    // Check if stdin has input (piped mode)
    if !io::stdin().is_terminal() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        if !input.trim().is_empty() {
            let mut repl = Repl::new()?;
            match repl.eval(&input) {
                Ok(result) => {
                    println!("{result}");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Repl) | None => {
            let version_msg = format!("Welcome to Ruchy REPL v{}", env!("CARGO_PKG_VERSION"));
            println!("{}", version_msg.bright_cyan().bold());
            println!(
                "Type {} for commands, {} to exit\n",
                ":help".green(),
                ":quit".yellow()
            );

            let mut repl = Repl::new()?;
            repl.run()?;
        }
        Some(Commands::Parse { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(ast) => println!("{ast:#?}"),
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Transpile { file, output }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile(&ast)?;
            let rust_code_str = rust_code.to_string();

            if let Some(output_path) = output {
                fs::write(output_path, rust_code_str)?;
            } else {
                println!("{rust_code_str}");
            }
        }
        Some(Commands::Run { file }) => {
            run_file(&file)?;
        }
        Some(Commands::Check { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(_) => {
                    println!("{}", "✓ Syntax is valid".green());
                }
                Err(e) => {
                    eprintln!("{}", format!("✗ Syntax error: {e}").red());
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Ast { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);
            match parser.parse() {
                Ok(ast) => println!("{ast:#?}"),
                Err(e) => {
                    eprintln!("Parse error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Fmt { file, all, check, stdout, diff }) => {
            format_ruchy_code(&file, all, check, stdout, diff)?;
        }
        Some(Commands::Lint { file, all, verbose, format, deny_warnings, max_complexity }) => {
            lint_ruchy_code(&file, all, verbose, &format, deny_warnings, max_complexity)?;
        }
    }

    Ok(())
}

fn run_file(file: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(file)?;

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

/// Format Ruchy code
#[allow(clippy::fn_params_excessive_bools)]
fn format_ruchy_code(file: &PathBuf, all: bool, check: bool, stdout: bool, diff: bool) -> Result<()> {
    if all {
        println!("{} Format all functionality not yet implemented", "⚠".yellow());
        return Ok(());
    }
    
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    
    match parser.parse() {
        Ok(ast) => {
            let formatted = format!("{ast:#?}");
            
            if check {
                if source.trim() == formatted.trim() {
                    println!("{} {} is already formatted", "✓".bright_green(), file.display());
                } else {
                    println!("{} {} needs formatting", "✗".bright_red(), file.display());
                    if diff {
                        println!("Diff would be shown here");
                    }
                    std::process::exit(1);
                }
            } else if stdout {
                println!("{formatted}");
            } else {
                println!("{} Formatted output for {} (write-back not implemented)", "→".bright_cyan(), file.display());
                println!("{formatted}");
            }
        }
        Err(e) => {
            eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Lint Ruchy code
fn lint_ruchy_code(file: &PathBuf, all: bool, verbose: bool, format: &str, deny_warnings: bool, max_complexity: usize) -> Result<()> {
    if all {
        println!("{} Lint all functionality not yet implemented", "⚠".yellow());
        return Ok(());
    }
    
    let source = fs::read_to_string(file)?;
    let mut parser = RuchyParser::new(&source);
    
    match parser.parse() {
        Ok(ast) => {
            // Basic lint checks
            let mut violations = Vec::new();
            
            // Check 1: Basic complexity (simplified)
            let complexity = estimate_complexity(&source);
            if complexity > max_complexity {
                violations.push(format!("High complexity: estimated {complexity}, max {max_complexity}"));
            }
            
            // Check 2: Long lines
            for (line_num, line) in source.lines().enumerate() {
                if line.len() > 100 {
                    violations.push(format!("Line {} too long: {} characters", line_num + 1, line.len()));
                }
            }
            
            // Display results
            if violations.is_empty() {
                println!("{} {} is clean", "✓".bright_green(), file.display());
            } else {
                println!("\n{} Issues found in {}:", "⚠".yellow(), file.display());
                for violation in &violations {
                    println!("  {}: {violation}", "warning".yellow());
                }
                
                if format == "json" {
                    let json = serde_json::json!({
                        "file": file.display().to_string(),
                        "violations": violations
                    });
                    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "Invalid JSON".to_string()));
                }
                
                if deny_warnings {
                    std::process::exit(1);
                }
            }
            
            if verbose {
                println!("AST: {ast:#?}");
            }
        }
        Err(e) => {
            eprintln!("{} Parse error in {}: {e}", "✗".bright_red(), file.display());
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Estimate complexity of source code (simplified)
fn estimate_complexity(source: &str) -> usize {
    let mut complexity = 1; // Base complexity
    
    for line in source.lines() {
        if line.contains("if ") { complexity += 1; }
        if line.contains("for ") { complexity += 1; }
        if line.contains("while ") { complexity += 1; }
        if line.contains("match ") { complexity += 1; }
        if line.contains("=>") { complexity += 1; }
    }
    
    complexity
}
