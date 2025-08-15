use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use ruchy::{Parser as RuchyParser, Repl, Transpiler};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ruchy")]
#[command(author, version, about = "The Ruchy programming language", long_about = None)]
struct Cli {
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

        /// Arguments to pass to the program
        args: Vec<String>,
    },

    /// Compile a Ruchy file to an executable
    Compile {
        /// The file to compile
        file: PathBuf,

        /// Output executable name
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Enable release mode
        #[arg(long)]
        release: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Repl) => {
            // Start REPL by default
            let mut repl = Repl::new()?;
            repl.run()?;
        }

        Some(Commands::Parse { file }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            match parser.parse() {
                Ok(ast) => {
                    println!("{}", "AST:".bright_cyan());
                    println!("{:#?}", ast);
                }
                Err(e) => {
                    eprintln!("{} {}", "Parse error:".bright_red(), e);
                    std::process::exit(1);
                }
            }
        }

        Some(Commands::Transpile { file, output }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            if let Some(output_path) = output {
                fs::write(output_path, rust_code)?;
                println!("{} Transpiled successfully", "✓".bright_green());
            } else {
                println!("{}", rust_code);
            }
        }

        Some(Commands::Run { file, args }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            // Create temporary Rust file
            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join("ruchy_temp.rs");
            let temp_exe = temp_dir.join("ruchy_temp");

            // Wrap in main function if needed
            let full_code = if rust_code.contains("fn main") {
                rust_code
            } else {
                format!("fn main() {{\n    {}\n}}", rust_code)
            };

            fs::write(&temp_file, full_code)?;

            // Compile
            let output = std::process::Command::new("rustc")
                .arg(&temp_file)
                .arg("-o")
                .arg(&temp_exe)
                .output()?;

            if !output.status.success() {
                eprintln!("{} Compilation failed:", "✗".bright_red());
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            // Run
            let mut cmd = std::process::Command::new(&temp_exe);
            for arg in args {
                cmd.arg(arg);
            }

            let status = cmd.status()?;
            std::process::exit(status.code().unwrap_or(1));
        }

        Some(Commands::Compile {
            file,
            output,
            release,
        }) => {
            let source = fs::read_to_string(&file)?;
            let mut parser = RuchyParser::new(&source);

            let ast = parser.parse()?;
            let transpiler = Transpiler::new();
            let rust_code = transpiler.transpile_to_string(&ast)?;

            // Generate output name
            let output_path = output.unwrap_or_else(|| {
                let mut path = file.clone();
                path.set_extension("");
                path
            });

            // Create temporary Rust file
            let temp_file = file.with_extension("rs");

            // Wrap in main function if needed
            let full_code = if rust_code.contains("fn main") {
                rust_code
            } else {
                format!("fn main() {{\n    {}\n}}", rust_code)
            };

            fs::write(&temp_file, full_code)?;

            // Compile
            let mut cmd = std::process::Command::new("rustc");
            cmd.arg(&temp_file).arg("-o").arg(&output_path);

            if release {
                cmd.arg("-O");
            }

            let output = cmd.output()?;

            if !output.status.success() {
                eprintln!("{} Compilation failed:", "✗".bright_red());
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                std::process::exit(1);
            }

            println!(
                "{} Compiled successfully to {}",
                "✓".bright_green(),
                output_path.display()
            );
        }
    }

    Ok(())
}
