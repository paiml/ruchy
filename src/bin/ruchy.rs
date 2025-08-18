#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use ruchy::{runtime::repl::Repl, Parser as RuchyParser, Transpiler};
use std::fs;
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle one-liner evaluation with -e flag
    if let Some(expr) = cli.eval {
        let mut repl = Repl::new()?;
        match repl.eval(&expr) {
            Ok(result) => {
                // JSON format outputs the same as text for now
                println!("{result}");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
    }

    // Handle script file execution (without subcommand)
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Repl) | None => {
            println!("{}", "Welcome to Ruchy REPL v0.4.8".bright_cyan().bold());
            println!("Type {} for help, {} to exit\n", ":help".green(), ":quit".yellow());
            
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
    }

    Ok(())
}

fn run_file(file: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(file)?;
    let mut repl = Repl::new()?;
    
    // Execute the file line by line in REPL
    for line in source.lines() {
        if !line.trim().is_empty() && !line.trim().starts_with("//") {
            if let Err(e) = repl.eval(line) {
                eprintln!("Error on line: {line}");
                eprintln!("  {e}");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}